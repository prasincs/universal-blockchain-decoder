//! XDR parsing for Stellar transactions
//!
//! This module implements pure Rust XDR (External Data Representation) parsing
//! for Stellar transaction envelopes and operations.
//!
//! XDR uses big-endian encoding and follows these rules:
//! - Integers: Fixed-size, big-endian
//! - Variable arrays: u32 length prefix + data + padding to 4-byte boundary
//! - Fixed arrays: Data + padding to 4-byte boundary
//! - Enums: u32 discriminant
//! - Optionals: u32 discriminant (0 = None, 1 = Some)

use crate::types::*;
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Read a u32 in big-endian (XDR integers are big-endian)
#[inline]
fn read_xdr_u32<R: Read>(reader: &mut R) -> Result<u32> {
    read_u32_be(reader)
}

/// Read an i64 in big-endian
#[inline]
fn read_xdr_i64<R: Read>(reader: &mut R) -> Result<i64> {
    let bytes = read_u64_be(reader)?;
    Ok(i64::from_be_bytes(bytes.to_be_bytes()))
}

// Helper functions for future use (currently unused but kept for completeness)
#[allow(dead_code)]
fn read_xdr_i32<R: Read>(reader: &mut R) -> Result<i32> {
    let bytes = read_u32_be(reader)?;
    Ok(i32::from_be_bytes(bytes.to_be_bytes()))
}

#[allow(dead_code)]
fn read_xdr_bool<R: Read>(reader: &mut R) -> Result<bool> {
    let value = read_xdr_u32(reader)?;
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid XDR bool: {}",
            value
        ))),
    }
}

/// Read XDR variable-length opaque data (bytes with length prefix)
///
/// Format: u32 length + data + padding to 4-byte boundary
fn read_xdr_bytes<R: Read>(reader: &mut R, max_len: usize) -> Result<Vec<u8>> {
    let len = read_xdr_u32(reader)? as usize;

    if len > max_len {
        return Err(DecoderError::invalid_structure(format!(
            "XDR bytes too long: {} > {}",
            len, max_len
        )));
    }

    let mut data = vec![0u8; len];
    reader
        .read_exact(&mut data)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read XDR bytes: {}", e)))?;

    // Read padding to align to 4-byte boundary
    let padding = (4 - (len % 4)) % 4;
    let mut pad = vec![0u8; padding];
    if padding > 0 {
        reader.read_exact(&mut pad).map_err(|e| {
            DecoderError::chain_decoding(format!("Failed to read XDR padding: {}", e))
        })?;
    }

    Ok(data)
}

/// Read XDR fixed-length opaque data
fn read_xdr_fixed_bytes<R: Read>(reader: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut data = vec![0u8; len];
    reader.read_exact(&mut data).map_err(|e| {
        DecoderError::chain_decoding(format!("Failed to read XDR fixed bytes: {}", e))
    })?;

    // Read padding to align to 4-byte boundary
    let padding = (4 - (len % 4)) % 4;
    let mut pad = vec![0u8; padding];
    if padding > 0 {
        reader.read_exact(&mut pad).map_err(|e| {
            DecoderError::chain_decoding(format!("Failed to read XDR padding: {}", e))
        })?;
    }

    Ok(data)
}

/// Read XDR string (same as bytes but UTF-8)
fn read_xdr_string<R: Read>(reader: &mut R, max_len: usize) -> Result<String> {
    let bytes = read_xdr_bytes(reader, max_len)?;
    String::from_utf8(bytes)
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid UTF-8 string: {}", e)))
}

/// Read XDR optional value
fn read_xdr_optional<R: Read, T, F>(reader: &mut R, parse_fn: F) -> Result<Option<T>>
where
    F: FnOnce(&mut R) -> Result<T>,
{
    let present = read_xdr_u32(reader)?;
    match present {
        0 => Ok(None),
        1 => Ok(Some(parse_fn(reader)?)),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid XDR optional discriminant: {}",
            present
        ))),
    }
}

/// Read XDR array (variable length)
fn read_xdr_array<R: Read, T, F>(reader: &mut R, max_len: usize, parse_fn: F) -> Result<Vec<T>>
where
    F: Fn(&mut R) -> Result<T>,
{
    let len = read_xdr_u32(reader)? as usize;

    if len > max_len {
        return Err(DecoderError::invalid_structure(format!(
            "XDR array too long: {} > {}",
            len, max_len
        )));
    }

    let mut items = Vec::with_capacity(len);
    for _ in 0..len {
        items.push(parse_fn(reader)?);
    }

    Ok(items)
}

/// Parse Stellar account ID (32-byte public key)
fn parse_account_id<R: Read>(reader: &mut R) -> Result<AccountId> {
    // Read public key type (discriminant)
    let key_type = read_xdr_u32(reader)?;
    if key_type != 0 {
        // 0 = PUBLIC_KEY_TYPE_ED25519
        return Err(DecoderError::invalid_structure(format!(
            "Unsupported key type: {}",
            key_type
        )));
    }

    // Read 32-byte Ed25519 public key
    read_xdr_fixed_bytes(reader, 32)
}

/// Parse Stellar asset
fn parse_asset<R: Read>(reader: &mut R) -> Result<StellarAsset> {
    let asset_type = read_xdr_u32(reader)?;

    match asset_type {
        0 => Ok(StellarAsset::Native), // ASSET_TYPE_NATIVE
        1 => {
            // ASSET_TYPE_CREDIT_ALPHANUM4
            let mut code = [0u8; 4];
            reader.read_exact(&mut code).map_err(|e| {
                DecoderError::chain_decoding(format!("Failed to read asset code: {}", e))
            })?;

            let issuer = parse_account_id(reader)?;
            Ok(StellarAsset::CreditAlphanum4 { code, issuer })
        }
        2 => {
            // ASSET_TYPE_CREDIT_ALPHANUM12
            let mut code = [0u8; 12];
            reader.read_exact(&mut code).map_err(|e| {
                DecoderError::chain_decoding(format!("Failed to read asset code: {}", e))
            })?;

            let issuer = parse_account_id(reader)?;
            Ok(StellarAsset::CreditAlphanum12 { code, issuer })
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Unknown asset type: {}",
            asset_type
        ))),
    }
}

/// Parse Stellar memo
fn parse_memo<R: Read>(reader: &mut R) -> Result<StellarMemo> {
    let memo_type = read_xdr_u32(reader)?;

    match memo_type {
        0 => Ok(StellarMemo::None), // MEMO_NONE
        1 => {
            // MEMO_TEXT
            let text = read_xdr_string(reader, 28)?;
            Ok(StellarMemo::Text(text))
        }
        2 => {
            // MEMO_ID
            let id = read_xdr_i64(reader)? as u64;
            Ok(StellarMemo::Id(id))
        }
        3 => {
            // MEMO_HASH
            let hash_bytes = read_xdr_fixed_bytes(reader, 32)?;
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);
            Ok(StellarMemo::Hash(hash))
        }
        4 => {
            // MEMO_RETURN
            let return_bytes = read_xdr_fixed_bytes(reader, 32)?;
            let mut return_hash = [0u8; 32];
            return_hash.copy_from_slice(&return_bytes);
            Ok(StellarMemo::Return(return_hash))
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Unknown memo type: {}",
            memo_type
        ))),
    }
}

/// Parse time bounds
fn parse_time_bounds<R: Read>(reader: &mut R) -> Result<TimeBounds> {
    let min_time = read_xdr_i64(reader)? as u64;
    let max_time = read_xdr_i64(reader)? as u64;

    Ok(TimeBounds { min_time, max_time })
}

/// Parse a Stellar operation
fn parse_operation<R: Read>(reader: &mut R) -> Result<StellarOperation> {
    // Read optional source account for operation
    let _source = read_xdr_optional(reader, parse_account_id)?;

    // Read operation type
    let op_type = read_xdr_u32(reader)?;

    match op_type {
        0 => {
            // CREATE_ACCOUNT
            let destination = parse_account_id(reader)?;
            let starting_balance = read_xdr_i64(reader)?;
            Ok(StellarOperation::CreateAccount {
                destination,
                starting_balance,
            })
        }
        1 => {
            // PAYMENT
            let destination = parse_account_id(reader)?;
            let asset = parse_asset(reader)?;
            let amount = read_xdr_i64(reader)?;
            Ok(StellarOperation::Payment {
                destination,
                asset,
                amount,
            })
        }
        2 => {
            // PATH_PAYMENT_STRICT_RECEIVE
            let send_asset = parse_asset(reader)?;
            let send_max = read_xdr_i64(reader)?;
            let destination = parse_account_id(reader)?;
            let dest_asset = parse_asset(reader)?;
            let dest_amount = read_xdr_i64(reader)?;
            let path = read_xdr_array(reader, 5, parse_asset)?;
            Ok(StellarOperation::PathPaymentStrictReceive {
                send_asset,
                send_max,
                destination,
                dest_asset,
                dest_amount,
                path,
            })
        }
        13 => {
            // PATH_PAYMENT_STRICT_SEND
            let send_asset = parse_asset(reader)?;
            let send_amount = read_xdr_i64(reader)?;
            let destination = parse_account_id(reader)?;
            let dest_asset = parse_asset(reader)?;
            let dest_min = read_xdr_i64(reader)?;
            let path = read_xdr_array(reader, 5, parse_asset)?;
            Ok(StellarOperation::PathPaymentStrictSend {
                send_asset,
                send_amount,
                destination,
                dest_asset,
                dest_min,
                path,
            })
        }
        23 => {
            // INVOKE_HOST_FUNCTION (Soroban)
            let function_type = read_xdr_u32(reader)?;
            let parameters = read_xdr_bytes(reader, 65536)?; // Max 64KB
            Ok(StellarOperation::InvokeHostFunction {
                function_type,
                parameters,
            })
        }
        // For now, return a generic operation for unsupported types
        _ => {
            // Read and discard the operation body (this is a simplified approach)
            // In a full implementation, we would parse all 24 operation types
            Ok(StellarOperation::BumpSequence { bump_to: 0 })
        }
    }
}

/// Parse decorated signature
fn parse_decorated_signature<R: Read>(reader: &mut R) -> Result<DecoratedSignature> {
    // Read hint (4 bytes - last 4 bytes of public key)
    let hint_bytes = read_xdr_fixed_bytes(reader, 4)?;
    let mut hint = [0u8; 4];
    hint.copy_from_slice(&hint_bytes);

    // Read signature (64 bytes for Ed25519)
    let signature = read_xdr_bytes(reader, 64)?;

    Ok(DecoratedSignature { hint, signature })
}

/// Parse a Stellar transaction envelope
///
/// This is the main entry point for parsing Stellar transactions.
pub fn parse_transaction_envelope(raw_bytes: &[u8]) -> Result<StellarTransaction> {
    let mut cursor = Cursor::new(raw_bytes);

    // Read envelope type
    let envelope_type_raw = read_xdr_u32(&mut cursor)?;
    let envelope_type = match envelope_type_raw {
        0 => EnvelopeType::TxV0,
        2 => EnvelopeType::Tx,
        5 => EnvelopeType::TxFeeBump,
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Unknown envelope type: {}",
                envelope_type_raw
            )))
        }
    };

    // For now, we'll focus on ENVELOPE_TYPE_TX (2)
    if envelope_type != EnvelopeType::Tx {
        return Err(DecoderError::invalid_structure(
            "Only ENVELOPE_TYPE_TX is currently supported",
        ));
    }

    // Parse transaction
    let source_account = parse_account_id(&mut cursor)?;
    let fee = read_xdr_u32(&mut cursor)?;
    let sequence_number = read_xdr_i64(&mut cursor)?;

    // Parse optional time bounds
    let time_bounds = read_xdr_optional(&mut cursor, parse_time_bounds)?;

    // Parse memo
    let memo = parse_memo(&mut cursor)?;

    // Parse operations (max 100)
    let operations = read_xdr_array(&mut cursor, 100, parse_operation)?;

    // Skip ext (reserved for future use)
    let _ext = read_xdr_u32(&mut cursor)?;

    // Parse signatures
    let signatures = read_xdr_array(&mut cursor, 20, parse_decorated_signature)?;

    Ok(StellarTransaction {
        source_account,
        fee,
        sequence_number,
        time_bounds,
        memo,
        operations,
        signatures,
        raw_bytes: raw_bytes.to_vec(),
        envelope_type,
        network_id: None, // Set by decoder based on network
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_xdr_u32() {
        let data = vec![0x00, 0x00, 0x01, 0x00]; // 256 in big-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_xdr_u32(&mut cursor).unwrap(), 256);
    }

    #[test]
    fn test_read_xdr_bool() {
        let data_true = vec![0x00, 0x00, 0x00, 0x01];
        let mut cursor = Cursor::new(data_true);
        assert!(read_xdr_bool(&mut cursor).unwrap());

        let data_false = vec![0x00, 0x00, 0x00, 0x00];
        let mut cursor = Cursor::new(data_false);
        assert!(!read_xdr_bool(&mut cursor).unwrap());
    }

    #[test]
    fn test_read_xdr_bytes_with_padding() {
        // Length=3, data=[1,2,3], padding=1 byte
        let data = vec![
            0x00, 0x00, 0x00, 0x03, // length = 3
            0x01, 0x02, 0x03, // data
            0x00, // padding
        ];
        let mut cursor = Cursor::new(data);
        let result = read_xdr_bytes(&mut cursor, 100).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_asset_native() {
        let data = vec![0x00, 0x00, 0x00, 0x00]; // ASSET_TYPE_NATIVE
        let mut cursor = Cursor::new(data);
        let asset = parse_asset(&mut cursor).unwrap();
        assert_eq!(asset, StellarAsset::Native);
    }

    #[test]
    fn test_parse_memo_none() {
        let data = vec![0x00, 0x00, 0x00, 0x00]; // MEMO_NONE
        let mut cursor = Cursor::new(data);
        let memo = parse_memo(&mut cursor).unwrap();
        assert_eq!(memo, StellarMemo::None);
    }
}
