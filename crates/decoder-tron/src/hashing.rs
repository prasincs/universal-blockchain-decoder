/// TRON address and hashing utilities
use decoder_primitives::prelude::*;
use sha2::{Digest, Sha256};
use sha3::Keccak256;

/// Encode TRON address bytes to base58check format
/// TRON addresses: version byte (0x41) + 20-byte address + 4-byte checksum
pub fn encode_tron_address(address_bytes: &[u8]) -> Result<String> {
    if address_bytes.len() != 21 {
        return Err(DecoderError::invalid_structure(format!(
            "TRON address must be 21 bytes, got {}",
            address_bytes.len()
        )));
    }

    // Compute checksum: first 4 bytes of SHA256(SHA256(address))
    let hash1 = Sha256::digest(address_bytes);
    let hash2 = Sha256::digest(hash1);
    let checksum = &hash2[0..4];

    // Concatenate address + checksum
    let mut with_checksum = address_bytes.to_vec();
    with_checksum.extend_from_slice(checksum);

    // Encode to base58
    Ok(bs58::encode(with_checksum).into_string())
}

/// Decode TRON base58check address to bytes
pub fn decode_tron_address(address: &str) -> Result<Vec<u8>> {
    let decoded = bs58::decode(address)
        .into_vec()
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid base58: {}", e)))?;

    if decoded.len() != 25 {
        return Err(DecoderError::invalid_structure(format!(
            "Decoded TRON address must be 25 bytes, got {}",
            decoded.len()
        )));
    }

    // Split into payload (21 bytes) and checksum (4 bytes)
    let (payload, checksum) = decoded.split_at(21);

    // Verify checksum
    let hash1 = Sha256::digest(payload);
    let hash2 = Sha256::digest(hash1);
    let expected_checksum = &hash2[0..4];

    if checksum != expected_checksum {
        return Err(DecoderError::invalid_structure(
            "Invalid TRON address checksum",
        ));
    }

    Ok(payload.to_vec())
}

/// Compute TRON transaction hash (SHA-256 of raw_data)
pub fn compute_tx_hash(raw_data_bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(raw_data_bytes).to_vec()
}

/// Convert TRON address bytes to hex string (with 0x41 prefix)
pub fn address_to_hex(address_bytes: &[u8]) -> String {
    if address_bytes.is_empty() {
        return String::new();
    }

    // Convert to hex
    let hex_str = address_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    format!("0x{}", hex_str)
}

/// Compute Ethereum-style address from public key (for smart contracts)
/// Used for contract address derivation
pub fn keccak256_address(public_key: &[u8]) -> Vec<u8> {
    let hash = Keccak256::digest(public_key);
    // Take last 20 bytes
    hash[12..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_encoding_decoding() {
        // Example TRON address: TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMW
        let address_base58 = "TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMW";

        // Decode
        let decoded = decode_tron_address(address_base58).unwrap();
        assert_eq!(decoded.len(), 21);
        assert_eq!(decoded[0], 0x41); // TRON version byte

        // Re-encode
        let encoded = encode_tron_address(&decoded).unwrap();
        assert_eq!(encoded, address_base58);
    }

    #[test]
    fn test_address_to_hex() {
        let address_bytes = vec![
            0x41, 0x88, 0x40, 0xe6, 0xc5, 0x5b, 0x9a, 0xda, 0x32, 0x6d, 0x21, 0x1d, 0x81, 0x8c,
            0x34, 0xa9, 0x94, 0xae, 0xce, 0xd8, 0x08,
        ];
        let hex = address_to_hex(&address_bytes);
        assert!(hex.starts_with("0x41"));
    }

    #[test]
    fn test_invalid_address_checksum() {
        // Invalid checksum
        let result = decode_tron_address("TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMX");
        assert!(result.is_err());
    }
}
