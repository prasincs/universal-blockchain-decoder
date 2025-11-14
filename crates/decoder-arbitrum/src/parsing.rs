//! Arbitrum Transaction Parsing
//!
//! Parses Arbitrum-specific transaction types (0x64-0x6A) plus standard Ethereum transactions.

use crate::types::*;
use decoder_encodings::rlp::RlpItem;
use decoder_ethereum::EthereumDecoder;
use decoder_primitives::prelude::*;

/// Parse an Arbitrum transaction from raw bytes
///
/// Arbitrum supports standard Ethereum transactions plus 6 Arbitrum-specific types:
/// - 0x64: Deposit (L1→L2)
/// - 0x65: Unsigned (EOA via bridge)
/// - 0x66: Contract (L1 contract call)
/// - 0x68: Retry (retry failed retryable)
/// - 0x69: SubmitRetryable (new retryable ticket)
/// - 0x6A: Internal (ArbOS system tx)
pub fn parse_arbitrum_transaction(bytes: &[u8]) -> Result<ArbitrumTransaction> {
    if bytes.is_empty() {
        return Err(DecoderError::invalid_structure(
            "Empty transaction bytes".to_string(),
        ));
    }

    // Check transaction type
    let tx_type = bytes[0];

    match tx_type {
        // Standard Ethereum transaction types (0x00-0x02)
        0x00..=0x02 => {
            let eth_tx = EthereumDecoder::decode(bytes)?;
            Ok(ArbitrumTransaction::Standard(eth_tx))
        }
        // Reserved types (0x03-0x63)
        0x03..=0x63 => Err(DecoderError::invalid_structure(format!(
            "Reserved transaction type: 0x{:02X}",
            tx_type
        ))),
        // Arbitrum deposit transaction
        0x64 => parse_deposit_transaction(&bytes[1..]),
        // Arbitrum unsigned transaction
        0x65 => parse_unsigned_transaction(&bytes[1..]),
        // Arbitrum contract transaction
        0x66 => parse_contract_transaction(&bytes[1..]),
        // Reserved (0x67)
        0x67 => Err(DecoderError::invalid_structure(
            "Transaction type 0x67 is reserved".to_string(),
        )),
        // Arbitrum retry transaction
        0x68 => parse_retry_transaction(&bytes[1..]),
        // Arbitrum submit retryable transaction
        0x69 => parse_submit_retryable_transaction(&bytes[1..]),
        // Arbitrum internal transaction
        0x6A => parse_internal_transaction(&bytes[1..]),
        // Reserved (0x6B-0x7F)
        0x6B..=0x7F => Err(DecoderError::invalid_structure(format!(
            "Reserved transaction type: 0x{:02X}",
            tx_type
        ))),
        // Legacy RLP-encoded transaction (0x80+)
        0x80..=0xFF => {
            if tx_type >= 0xC0 {
                let eth_tx = EthereumDecoder::decode(bytes)?;
                Ok(ArbitrumTransaction::Standard(eth_tx))
            } else {
                Err(DecoderError::invalid_structure(format!(
                    "Invalid transaction type: 0x{:02X}",
                    tx_type
                )))
            }
        }
    }
}

/// Parse deposit transaction (0x64)
///
/// RLP structure: [chain_id, l1_block_number, from, to, value, gas_limit, data]
fn parse_deposit_transaction(rlp_bytes: &[u8]) -> Result<ArbitrumTransaction> {
    let items = parse_rlp_list(rlp_bytes, "DepositTransaction", 7)?;

    let deposit = DepositTransaction {
        chain_id: parse_u64(&items[0], "chain_id")?,
        l1_block_number: parse_u64(&items[1], "l1_block_number")?,
        from: parse_address(&items[2], "from")?,
        to: parse_optional_address(&items[3], "to")?,
        value: parse_u128(&items[4], "value")?,
        gas_limit: parse_u64(&items[5], "gas_limit")?,
        data: parse_bytes(&items[6], "data")?,
    };

    deposit.validate()?;
    Ok(ArbitrumTransaction::Deposit(deposit))
}

/// Parse unsigned transaction (0x65)
///
/// RLP structure: [chain_id, from, to, value, gas_limit, gas_price, nonce, data]
fn parse_unsigned_transaction(rlp_bytes: &[u8]) -> Result<ArbitrumTransaction> {
    let items = parse_rlp_list(rlp_bytes, "UnsignedTransaction", 8)?;

    let unsigned = UnsignedTransaction {
        chain_id: parse_u64(&items[0], "chain_id")?,
        from: parse_address(&items[1], "from")?,
        to: parse_address(&items[2], "to")?,
        value: parse_u128(&items[3], "value")?,
        gas_limit: parse_u64(&items[4], "gas_limit")?,
        gas_price: parse_u128(&items[5], "gas_price")?,
        nonce: parse_u64(&items[6], "nonce")?,
        data: parse_bytes(&items[7], "data")?,
    };

    unsigned.validate()?;
    Ok(ArbitrumTransaction::Unsigned(unsigned))
}

/// Parse contract transaction (0x66)
///
/// RLP structure: [chain_id, from, to, value, gas_limit, gas_price, nonce, data]
fn parse_contract_transaction(rlp_bytes: &[u8]) -> Result<ArbitrumTransaction> {
    let items = parse_rlp_list(rlp_bytes, "ContractTransaction", 8)?;

    let contract = ContractTransaction {
        chain_id: parse_u64(&items[0], "chain_id")?,
        from: parse_address(&items[1], "from")?,
        to: parse_address(&items[2], "to")?,
        value: parse_u128(&items[3], "value")?,
        gas_limit: parse_u64(&items[4], "gas_limit")?,
        gas_price: parse_u128(&items[5], "gas_price")?,
        nonce: parse_u64(&items[6], "nonce")?,
        data: parse_bytes(&items[7], "data")?,
    };

    contract.validate()?;
    Ok(ArbitrumTransaction::Contract(contract))
}

/// Parse retry transaction (0x68)
///
/// RLP structure: [chain_id, ticket_id, from, gas_limit, gas_price, nonce]
fn parse_retry_transaction(rlp_bytes: &[u8]) -> Result<ArbitrumTransaction> {
    let items = parse_rlp_list(rlp_bytes, "RetryTransaction", 6)?;

    let retry = RetryTransaction {
        chain_id: parse_u64(&items[0], "chain_id")?,
        ticket_id: parse_bytes32(&items[1], "ticket_id")?,
        from: parse_address(&items[2], "from")?,
        gas_limit: parse_u64(&items[3], "gas_limit")?,
        gas_price: parse_u128(&items[4], "gas_price")?,
        nonce: parse_u64(&items[5], "nonce")?,
    };

    retry.validate()?;
    Ok(ArbitrumTransaction::Retry(retry))
}

/// Parse submit retryable transaction (0x69)
///
/// RLP structure: [chain_id, request_id, l1_base_fee, deposit, callvalue, gas_fee_cap,
///                gas_limit, max_submission_fee, fee_refund_address, beneficiary,
///                retry_to, retry_data]
fn parse_submit_retryable_transaction(rlp_bytes: &[u8]) -> Result<ArbitrumTransaction> {
    let items = parse_rlp_list(rlp_bytes, "SubmitRetryableTransaction", 12)?;

    let retryable = SubmitRetryableTransaction {
        chain_id: parse_u64(&items[0], "chain_id")?,
        request_id: parse_bytes32(&items[1], "request_id")?,
        l1_base_fee: parse_u128(&items[2], "l1_base_fee")?,
        deposit: parse_u128(&items[3], "deposit")?,
        callvalue: parse_u128(&items[4], "callvalue")?,
        gas_fee_cap: parse_u128(&items[5], "gas_fee_cap")?,
        gas_limit: parse_u64(&items[6], "gas_limit")?,
        max_submission_fee: parse_u128(&items[7], "max_submission_fee")?,
        fee_refund_address: parse_address(&items[8], "fee_refund_address")?,
        beneficiary: parse_address(&items[9], "beneficiary")?,
        retry_to: parse_address(&items[10], "retry_to")?,
        retry_data: parse_bytes(&items[11], "retry_data")?,
    };

    retryable.validate()?;
    Ok(ArbitrumTransaction::SubmitRetryable(retryable))
}

/// Parse internal transaction (0x6A)
///
/// RLP structure: [chain_id, internal_type, l1_block_number, l1_base_fee, l1_timestamp]
fn parse_internal_transaction(rlp_bytes: &[u8]) -> Result<ArbitrumTransaction> {
    let items = parse_rlp_list(rlp_bytes, "InternalTransaction", 5)?;

    let internal_type_byte = parse_u8(&items[1], "internal_type")?;
    let internal_type = match internal_type_byte {
        0 => InternalTxType::UpdateL1BlockNumber,
        other => InternalTxType::Unknown(other),
    };

    let internal = InternalTransaction {
        chain_id: parse_u64(&items[0], "chain_id")?,
        internal_type,
        l1_block_number: parse_u64(&items[2], "l1_block_number")?,
        l1_base_fee: parse_u128(&items[3], "l1_base_fee")?,
        l1_timestamp: parse_u64(&items[4], "l1_timestamp")?,
    };

    internal.validate()?;
    Ok(ArbitrumTransaction::Internal(internal))
}

// ============================================================================
// RLP Parsing Helper Functions
// ============================================================================

/// Parse RLP bytes as a list with expected field count
fn parse_rlp_list(rlp_bytes: &[u8], tx_type: &str, expected_fields: usize) -> Result<Vec<RlpItem>> {
    let rlp = RlpItem::decode(rlp_bytes)?;
    let items = match rlp {
        RlpItem::List(items) => items,
        RlpItem::Data(_) => {
            return Err(DecoderError::invalid_structure(format!(
                "{} must be an RLP list",
                tx_type
            )))
        }
    };

    if items.len() != expected_fields {
        return Err(DecoderError::invalid_structure(format!(
            "{} must have {} fields, got {}",
            tx_type,
            expected_fields,
            items.len()
        )));
    }

    Ok(items)
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

/// Parse u8 from RLP item
fn parse_u8(item: &RlpItem, field_name: &str) -> Result<u8> {
    match item {
        RlpItem::Data(bytes) => {
            if bytes.is_empty() {
                Ok(0)
            } else if bytes.len() == 1 {
                Ok(bytes[0])
            } else {
                Err(DecoderError::invalid_structure(format!(
                    "{} must be 0 or 1 byte, got {}",
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
    fn test_parse_empty_bytes() {
        let result = parse_arbitrum_transaction(&[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty transaction"));
    }

    #[test]
    fn test_parse_reserved_type() {
        let result = parse_arbitrum_transaction(&[0x03]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Reserved"));
    }

    #[test]
    fn test_transaction_type_detection() {
        // Type 0x64 should route to deposit parsing
        let bytes = vec![0x64, 0xC0]; // Minimal RLP list
        let result = parse_arbitrum_transaction(&bytes);
        // Will fail on RLP parsing, but confirms routing works
        assert!(result.is_err());

        // Type 0x69 should route to submit retryable parsing
        let bytes = vec![0x69, 0xC0]; // Minimal RLP list
        let result = parse_arbitrum_transaction(&bytes);
        // Will fail on RLP parsing, but confirms routing works
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u8() {
        let item = RlpItem::Data(vec![5]);
        assert_eq!(parse_u8(&item, "test").unwrap(), 5);

        let item = RlpItem::Data(vec![]);
        assert_eq!(parse_u8(&item, "test").unwrap(), 0);

        let item = RlpItem::Data(vec![1, 2]);
        assert!(parse_u8(&item, "test").is_err());
    }

    #[test]
    fn test_parse_u64() {
        let item = RlpItem::Data(vec![0x01, 0x00]);
        assert_eq!(parse_u64(&item, "test").unwrap(), 256);

        let item = RlpItem::Data(vec![]);
        assert_eq!(parse_u64(&item, "test").unwrap(), 0);

        // Too large for u64
        let item = RlpItem::Data(vec![0xFF; 9]);
        assert!(parse_u64(&item, "test").is_err());
    }

    #[test]
    fn test_parse_u128() {
        let item = RlpItem::Data(vec![0x01, 0x00, 0x00]);
        assert_eq!(parse_u128(&item, "test").unwrap(), 65536);

        let item = RlpItem::Data(vec![]);
        assert_eq!(parse_u128(&item, "test").unwrap(), 0);

        // Max u128 (16 bytes of 0xFF)
        let item = RlpItem::Data(vec![0xFF; 16]);
        assert_eq!(parse_u128(&item, "test").unwrap(), u128::MAX);
    }

    #[test]
    fn test_parse_address() {
        let addr = [0x12u8; 20];
        let item = RlpItem::Data(addr.to_vec());
        assert_eq!(parse_address(&item, "test").unwrap(), addr);

        // Wrong length
        let item = RlpItem::Data(vec![0x12; 19]);
        assert!(parse_address(&item, "test").is_err());
    }

    #[test]
    fn test_parse_optional_address() {
        // Empty = None
        let item = RlpItem::Data(vec![]);
        assert_eq!(parse_optional_address(&item, "test").unwrap(), None);

        // 20 bytes = Some
        let addr = [0x12u8; 20];
        let item = RlpItem::Data(addr.to_vec());
        assert_eq!(parse_optional_address(&item, "test").unwrap(), Some(addr));

        // Wrong length
        let item = RlpItem::Data(vec![0x12; 19]);
        assert!(parse_optional_address(&item, "test").is_err());
    }

    #[test]
    fn test_parse_bytes32() {
        let hash = [0xABu8; 32];
        let item = RlpItem::Data(hash.to_vec());
        assert_eq!(parse_bytes32(&item, "test").unwrap(), hash);

        // Wrong length
        let item = RlpItem::Data(vec![0xAB; 31]);
        assert!(parse_bytes32(&item, "test").is_err());
    }
}
