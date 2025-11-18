//! Optimism Transaction Parsing
//!
//! Parses Optimism-specific transaction types, including deposit transactions (0x7E).

use crate::types::{DepositTransaction, OptimismTransaction};
use decoder_encodings::rlp::RlpItem;
use decoder_ethereum::EthereumDecoder;
use decoder_primitives::prelude::*;

/// Parse an Optimism transaction from raw bytes
///
/// Optimism supports two transaction types:
/// - Standard Ethereum transactions (0x00, 0x01, 0x02)
/// - Deposit transactions (0x7E)
pub fn parse_optimism_transaction(bytes: &[u8]) -> Result<OptimismTransaction> {
    if bytes.is_empty() {
        return Err(DecoderError::invalid_structure(
            "Empty transaction bytes".to_string(),
        ));
    }

    // Check transaction type
    let tx_type = bytes[0];

    match tx_type {
        // EIP-2718 typed transactions (0x00-0x7F)
        0x00..=0x7D => {
            // Standard Ethereum transaction types
            let eth_tx = EthereumDecoder::decode(bytes)?;
            Ok(OptimismTransaction::Standard(eth_tx))
        }
        0x7E => {
            // Optimism deposit transaction
            parse_deposit_transaction(&bytes[1..], bytes)
        }
        0x7F => {
            // Reserved for future use
            Err(DecoderError::invalid_structure(
                "Transaction type 0x7F is reserved".to_string(),
            ))
        }
        0x80..=0xFF => {
            // Legacy RLP-encoded transaction (first byte ≥ 0xC0 for valid RLP list)
            if tx_type >= 0xC0 {
                let eth_tx = EthereumDecoder::decode(bytes)?;
                Ok(OptimismTransaction::Standard(eth_tx))
            } else {
                Err(DecoderError::invalid_structure(format!(
                    "Invalid transaction type: 0x{:02X}",
                    tx_type
                )))
            }
        }
    }
}

/// Parse a deposit transaction from RLP-encoded bytes (after 0x7E prefix)
///
/// ## RLP Structure
///
/// A deposit transaction is RLP-encoded as a list of 8 fields:
/// [source_hash, from, to, mint, value, gas_limit, is_creation, data]
///
/// ## Field Types
///
/// - source_hash: bytes32
/// - from: address (20 bytes)
/// - to: address or empty (20 bytes or 0 bytes)
/// - mint: uint256
/// - value: uint256
/// - gas_limit: uint64
/// - is_creation: bool (0x00 or 0x01)
/// - data: bytes
fn parse_deposit_transaction(rlp_bytes: &[u8], raw_bytes: &[u8]) -> Result<OptimismTransaction> {
    // Deposit transaction must be an RLP list
    let rlp = RlpItem::decode(rlp_bytes)?;
    let items = match rlp {
        RlpItem::List(items) => items,
        RlpItem::Data(_) => {
            return Err(DecoderError::invalid_structure(
                "Deposit transaction must be an RLP list".to_string(),
            ))
        }
    };

    // Must have exactly 8 fields
    if items.len() != 8 {
        return Err(DecoderError::invalid_structure(format!(
            "Deposit transaction must have 8 fields, got {}",
            items.len()
        )));
    }

    // Parse source_hash (bytes32)
    let source_hash = parse_bytes32(&items[0], "source_hash")?;

    // Parse from (address)
    let from = parse_address(&items[1], "from")?;

    // Parse to (optional address)
    let to = parse_optional_address(&items[2], "to")?;

    // Parse mint (uint256, but store as u128)
    let mint = parse_u128(&items[3], "mint")?;

    // Parse value (uint256, but store as u128)
    let value = parse_u128(&items[4], "value")?;

    // Parse gas_limit (uint64)
    let gas_limit = parse_u64(&items[5], "gas_limit")?;

    // Parse is_creation (bool)
    let is_creation = parse_bool(&items[6], "is_creation")?;

    // Parse data (bytes)
    let data = parse_bytes(&items[7], "data")?;

    let deposit = DepositTransaction::new(
        source_hash,
        from,
        to,
        mint,
        value,
        gas_limit,
        is_creation,
        data,
        raw_bytes.to_vec(),
    );

    // Validate invariants
    deposit.validate()?;

    Ok(OptimismTransaction::Deposit(deposit))
}

/// Parse a 32-byte hash from RLP item
fn parse_bytes32(item: &RlpItem, field_name: &str) -> Result<[u8; 32]> {
    match item {
        RlpItem::Data(bytes) => {
            if bytes.len() != 32 {
                return Err(DecoderError::invalid_structure(format!(
                    "{} must be 32 bytes, got {}",
                    field_name,
                    bytes.len()
                )));
            }
            let mut result = [0u8; 32];
            result.copy_from_slice(bytes);
            Ok(result)
        }
        RlpItem::List(_) => Err(DecoderError::invalid_structure(format!(
            "{} must be data, not list",
            field_name
        ))),
    }
}

/// Parse a 20-byte address from RLP item
fn parse_address(item: &RlpItem, field_name: &str) -> Result<[u8; 20]> {
    match item {
        RlpItem::Data(bytes) => {
            if bytes.len() != 20 {
                return Err(DecoderError::invalid_structure(format!(
                    "{} must be 20 bytes, got {}",
                    field_name,
                    bytes.len()
                )));
            }
            let mut result = [0u8; 20];
            result.copy_from_slice(bytes);
            Ok(result)
        }
        RlpItem::List(_) => Err(DecoderError::invalid_structure(format!(
            "{} must be data, not list",
            field_name
        ))),
    }
}

/// Parse an optional address (empty = None, 20 bytes = Some)
fn parse_optional_address(item: &RlpItem, field_name: &str) -> Result<Option<[u8; 20]>> {
    match item {
        RlpItem::Data(bytes) => {
            if bytes.is_empty() {
                Ok(None)
            } else if bytes.len() == 20 {
                let mut result = [0u8; 20];
                result.copy_from_slice(bytes);
                Ok(Some(result))
            } else {
                Err(DecoderError::invalid_structure(format!(
                    "{} must be 0 or 20 bytes, got {}",
                    field_name,
                    bytes.len()
                )))
            }
        }
        RlpItem::List(_) => Err(DecoderError::invalid_structure(format!(
            "{} must be data, not list",
            field_name
        ))),
    }
}

/// Parse u128 from RLP item (handles uint256 but truncates to u128)
fn parse_u128(item: &RlpItem, field_name: &str) -> Result<u128> {
    match item {
        RlpItem::Data(bytes) => {
            if bytes.is_empty() {
                return Ok(0);
            }
            if bytes.len() > 16 {
                // Check if high bytes are zero
                if bytes.iter().take(bytes.len() - 16).any(|&b| b != 0) {
                    return Err(DecoderError::invalid_structure(format!(
                        "{} value too large for u128",
                        field_name
                    )));
                }
                // Take last 16 bytes
                let mut result_bytes = [0u8; 16];
                result_bytes.copy_from_slice(&bytes[bytes.len() - 16..]);
                Ok(u128::from_be_bytes(result_bytes))
            } else {
                // Pad to 16 bytes
                let mut result_bytes = [0u8; 16];
                result_bytes[16 - bytes.len()..].copy_from_slice(bytes);
                Ok(u128::from_be_bytes(result_bytes))
            }
        }
        RlpItem::List(_) => Err(DecoderError::invalid_structure(format!(
            "{} must be data, not list",
            field_name
        ))),
    }
}

/// Parse u64 from RLP item
fn parse_u64(item: &RlpItem, field_name: &str) -> Result<u64> {
    match item {
        RlpItem::Data(bytes) => {
            if bytes.is_empty() {
                return Ok(0);
            }
            if bytes.len() > 8 {
                return Err(DecoderError::invalid_structure(format!(
                    "{} value too large for u64",
                    field_name
                )));
            }
            // Pad to 8 bytes
            let mut result_bytes = [0u8; 8];
            result_bytes[8 - bytes.len()..].copy_from_slice(bytes);
            Ok(u64::from_be_bytes(result_bytes))
        }
        RlpItem::List(_) => Err(DecoderError::invalid_structure(format!(
            "{} must be data, not list",
            field_name
        ))),
    }
}

/// Parse bool from RLP item (0x00 = false, 0x01 = true)
fn parse_bool(item: &RlpItem, field_name: &str) -> Result<bool> {
    match item {
        RlpItem::Data(bytes) => {
            if bytes.is_empty() || bytes == &[0x00] {
                Ok(false)
            } else if bytes == &[0x01] {
                Ok(true)
            } else {
                Err(DecoderError::invalid_structure(format!(
                    "{} must be 0x00 or 0x01, got {} bytes",
                    field_name,
                    bytes.len()
                )))
            }
        }
        RlpItem::List(_) => Err(DecoderError::invalid_structure(format!(
            "{} must be data, not list",
            field_name
        ))),
    }
}

/// Parse bytes from RLP item
fn parse_bytes(item: &RlpItem, field_name: &str) -> Result<Vec<u8>> {
    match item {
        RlpItem::Data(bytes) => Ok(bytes.to_vec()),
        RlpItem::List(_) => Err(DecoderError::invalid_structure(format!(
            "{} must be data, not list",
            field_name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bytes32() {
        let data = RlpItem::Data(vec![1u8; 32]);
        let result = parse_bytes32(&data, "test_field").unwrap();
        assert_eq!(result, [1u8; 32]);

        // Wrong length
        let data = RlpItem::Data(vec![1u8; 31]);
        assert!(parse_bytes32(&data, "test_field").is_err());

        // List instead of data
        let list = RlpItem::List(vec![]);
        assert!(parse_bytes32(&list, "test_field").is_err());
    }

    #[test]
    fn test_parse_address() {
        let data = RlpItem::Data(vec![2u8; 20]);
        let result = parse_address(&data, "test_field").unwrap();
        assert_eq!(result, [2u8; 20]);

        // Wrong length
        let data = RlpItem::Data(vec![2u8; 19]);
        assert!(parse_address(&data, "test_field").is_err());
    }

    #[test]
    fn test_parse_optional_address() {
        // Empty = None
        let data = RlpItem::Data(vec![]);
        let result = parse_optional_address(&data, "test_field").unwrap();
        assert_eq!(result, None);

        // 20 bytes = Some
        let data = RlpItem::Data(vec![3u8; 20]);
        let result = parse_optional_address(&data, "test_field").unwrap();
        assert_eq!(result, Some([3u8; 20]));

        // Invalid length
        let data = RlpItem::Data(vec![3u8; 10]);
        assert!(parse_optional_address(&data, "test_field").is_err());
    }

    #[test]
    fn test_parse_u128() {
        // Empty = 0
        let data = RlpItem::Data(vec![]);
        let result = parse_u128(&data, "test_field").unwrap();
        assert_eq!(result, 0);

        // Small value
        let data = RlpItem::Data(vec![0x01, 0x00]); // 256
        let result = parse_u128(&data, "test_field").unwrap();
        assert_eq!(result, 256);

        // Max u128
        let data = RlpItem::Data(vec![0xFF; 16]);
        let result = parse_u128(&data, "test_field").unwrap();
        assert_eq!(result, u128::MAX);

        // Value too large (non-zero high bytes)
        let mut large_bytes = vec![0u8; 32];
        large_bytes[0] = 0x01; // Set high byte
        let data = RlpItem::Data(large_bytes);
        assert!(parse_u128(&data, "test_field").is_err());

        // 32 bytes with zero high bytes should work
        let large_bytes = vec![0u8; 32];
        let data = RlpItem::Data(large_bytes);
        let result = parse_u128(&data, "test_field").unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_parse_u64() {
        // Empty = 0
        let data = RlpItem::Data(vec![]);
        let result = parse_u64(&data, "test_field").unwrap();
        assert_eq!(result, 0);

        // Small value
        let data = RlpItem::Data(vec![0x01, 0x00]); // 256
        let result = parse_u64(&data, "test_field").unwrap();
        assert_eq!(result, 256);

        // Max u64
        let data = RlpItem::Data(vec![0xFF; 8]);
        let result = parse_u64(&data, "test_field").unwrap();
        assert_eq!(result, u64::MAX);

        // Value too large
        let data = RlpItem::Data(vec![0xFF; 9]);
        assert!(parse_u64(&data, "test_field").is_err());
    }

    #[test]
    fn test_parse_bool() {
        // Empty = false
        let data = RlpItem::Data(vec![]);
        let result = parse_bool(&data, "test_field").unwrap();
        assert!(!result);

        // 0x00 = false
        let data = RlpItem::Data(vec![0x00]);
        let result = parse_bool(&data, "test_field").unwrap();
        assert!(!result);

        // 0x01 = true
        let data = RlpItem::Data(vec![0x01]);
        let result = parse_bool(&data, "test_field").unwrap();
        assert!(result);

        // Invalid value
        let data = RlpItem::Data(vec![0x02]);
        assert!(parse_bool(&data, "test_field").is_err());
    }

    #[test]
    fn test_parse_bytes() {
        let data = RlpItem::Data(vec![1, 2, 3, 4]);
        let result = parse_bytes(&data, "test_field").unwrap();
        assert_eq!(result, vec![1, 2, 3, 4]);

        // Empty is valid
        let data = RlpItem::Data(vec![]);
        let result = parse_bytes(&data, "test_field").unwrap();
        assert!(result.is_empty());
    }
}
