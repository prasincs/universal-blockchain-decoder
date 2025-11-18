//! CBOR parsing functions for Filecoin transactions using minicbor
//!
//! Filecoin messages are encoded using CBOR (DAG-CBOR for some structures).
//! This module provides pure Rust parsing without external Filecoin libraries.

use crate::types::*;
use decoder_primitives::prelude::*;
use minicbor::Decoder;

/// Parse a FilecoinTransaction from CBOR-encoded bytes
///
/// Filecoin signed messages are encoded as a CBOR array:
/// [Message, Signature]
pub fn parse_signed_message(raw_bytes: &[u8]) -> Result<FilecoinTransaction> {
    let mut decoder = Decoder::new(raw_bytes);

    // Decode outer array [Message, Signature]
    let array_len = decoder
        .array()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid CBOR array: {}", e)))?
        .ok_or_else(|| DecoderError::invalid_structure("Expected definite-length array"))?;

    if array_len != 2 {
        return Err(DecoderError::invalid_structure(format!(
            "Expected 2 elements in signed message, got {}",
            array_len
        )));
    }

    // Parse message
    let message = parse_message(&mut decoder)?;

    // Parse signature
    let signature = parse_signature(&mut decoder)?;

    let signed_message = FilecoinSignedMessage::new(message, signature, raw_bytes.to_vec());

    Ok(FilecoinTransaction {
        signed_message,
        raw_bytes: raw_bytes.to_vec(),
    })
}

/// Parse a Filecoin message from CBOR
///
/// Message structure (CBOR array):
/// [Version, To, From, Sequence, Value, GasLimit, GasFeeCap, GasPremium, Method, Params]
fn parse_message(decoder: &mut Decoder) -> Result<FilecoinMessage> {
    // Decode message array
    let array_len = decoder
        .array()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid message array: {}", e)))?
        .ok_or_else(|| DecoderError::invalid_structure("Expected definite-length array"))?;

    if array_len != 10 {
        return Err(DecoderError::invalid_structure(format!(
            "Expected 10 elements in message, got {}",
            array_len
        )));
    }

    // Field 0: Version
    let version = decoder
        .u64()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid version: {}", e)))?;

    // Field 1: To address
    let to = parse_address(decoder)?;

    // Field 2: From address
    let from = parse_address(decoder)?;

    // Field 3: Sequence (nonce)
    let sequence = decoder
        .u64()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid sequence: {}", e)))?;

    // Field 4: Value (BigInt)
    let value = parse_bigint(decoder)?;

    // Field 5: Gas limit
    let gas_limit = decoder
        .u64()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid gas_limit: {}", e)))?;

    // Field 6: Gas fee cap (BigInt)
    let gas_fee_cap = parse_bigint(decoder)?;

    // Field 7: Gas premium (BigInt)
    let gas_premium = parse_bigint(decoder)?;

    // Field 8: Method number
    let method_num = decoder
        .u64()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid method_num: {}", e)))?;

    // Field 9: Params (bytes)
    let params = decoder
        .bytes()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid params: {}", e)))?
        .to_vec();

    Ok(FilecoinMessage {
        version,
        from,
        to,
        sequence,
        value,
        gas_limit,
        gas_fee_cap,
        gas_premium,
        method_num,
        params,
    })
}

/// Parse a Filecoin address from CBOR
///
/// Address is encoded as a CBOR byte string
fn parse_address(decoder: &mut Decoder) -> Result<FilecoinAddress> {
    let bytes = decoder
        .bytes()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid address bytes: {}", e)))?;

    if bytes.is_empty() {
        return Err(DecoderError::invalid_structure("Address cannot be empty"));
    }

    let protocol = AddressProtocol::from_byte(bytes[0])?;
    let payload = bytes[1..].to_vec();

    Ok(FilecoinAddress { protocol, payload })
}

/// Parse a Filecoin signature from CBOR
///
/// Signature structure (CBOR array): [Type, Data]
fn parse_signature(decoder: &mut Decoder) -> Result<FilecoinSignature> {
    // Decode signature array
    let array_len = decoder
        .array()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid signature array: {}", e)))?
        .ok_or_else(|| DecoderError::invalid_structure("Expected definite-length array"))?;

    if array_len != 2 {
        return Err(DecoderError::invalid_structure(format!(
            "Expected 2 elements in signature, got {}",
            array_len
        )));
    }

    // Field 0: Signature type
    let sig_type_byte = decoder
        .u8()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid signature type: {}", e)))?;
    let sig_type = SignatureType::from_byte(sig_type_byte)?;

    // Field 1: Signature data
    let data = decoder
        .bytes()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid signature data: {}", e)))?
        .to_vec();

    Ok(FilecoinSignature { sig_type, data })
}

/// Parse a BigInt from CBOR
///
/// Filecoin uses CBOR byte strings to encode arbitrary-precision integers.
/// Positive integers are encoded as-is, negative integers have a leading 0x01 byte.
fn parse_bigint(decoder: &mut Decoder) -> Result<Vec<u8>> {
    let bytes = decoder
        .bytes()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid bigint bytes: {}", e)))?;

    // For simplicity, we store the raw bytes
    // Empty bytes represents 0
    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bigint_zero() {
        let cbor_zero = vec![0x40]; // CBOR empty byte string
        let mut decoder = Decoder::new(&cbor_zero);
        let result = parse_bigint(&mut decoder).unwrap();
        assert_eq!(result, Vec::<u8>::new());
    }

    #[test]
    fn test_parse_address_id() {
        // CBOR byte string: protocol 0 (ID), payload [0x01]
        let cbor_addr = vec![0x42, 0x00, 0x01]; // byte string of length 2
        let mut decoder = Decoder::new(&cbor_addr);
        let addr = parse_address(&mut decoder).unwrap();

        assert_eq!(addr.protocol, AddressProtocol::Id);
        assert_eq!(addr.payload, vec![0x01]);
    }

    #[test]
    fn test_parse_signature() {
        // CBOR array: [1, <signature_data>]
        // Secp256k1 signature (65 bytes typical)
        let sig_data = vec![0x42; 65]; // Placeholder signature data

        // Manually construct CBOR: array(2), u8(1), bytes(65)
        let mut cbor = vec![0x82]; // Array of 2 elements
        cbor.push(0x01); // u8: 1 (Secp256k1)
        cbor.push(0x58); // Byte string follows, 1-byte length
        cbor.push(65); // Length: 65
        cbor.extend_from_slice(&sig_data);

        let mut decoder = Decoder::new(&cbor);
        let sig = parse_signature(&mut decoder).unwrap();

        assert_eq!(sig.sig_type, SignatureType::Secp256k1);
        assert_eq!(sig.data.len(), 65);
    }
}
