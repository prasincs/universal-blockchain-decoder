/// Simple unit tests for TRON decoder
use decoder_primitives::prelude::*;
use decoder_tron::TronDecoder;

#[test]
fn test_chain_identity() {
    let chain = TronDecoder::chain();
    assert_eq!(chain.chain_id(), 195);
    assert_eq!(chain.chain_name(), "Tron");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
}

#[test]
fn test_validate_empty_transaction() {
    let result = TronDecoder::decode(&[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));
}

#[test]
fn test_validate_invalid_protobuf() {
    let invalid_data = vec![0xFF; 100];
    let result = TronDecoder::decode(&invalid_data);
    assert!(result.is_err());
}

#[test]
fn test_validate_too_large() {
    let large_data = vec![0u8; 2_000_000];
    let result = TronDecoder::decode(&large_data);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large"));
}

#[test]
fn test_address_encoding() {
    use decoder_tron::hashing::{decode_tron_address, encode_tron_address};

    // Example TRON address with version byte
    let address_bytes = vec![
        0x41, 0x88, 0x40, 0xe6, 0xc5, 0x5b, 0x9a, 0xda, 0x32, 0x6d, 0x21, 0x1d, 0x81, 0x8c, 0x34,
        0xa9, 0x94, 0xae, 0xce, 0xd8, 0x08,
    ];

    let encoded = encode_tron_address(&address_bytes).expect("Failed to encode");
    assert!(!encoded.is_empty());

    // Verify round-trip
    let decoded = decode_tron_address(&encoded).expect("Failed to decode");
    assert_eq!(decoded, address_bytes);
}

#[test]
fn test_address_hex_conversion() {
    use decoder_tron::hashing::address_to_hex;

    let address_bytes = vec![0x41, 0x88, 0x40, 0xe6];
    let hex_str = address_to_hex(&address_bytes);
    assert_eq!(hex_str, "0x418840e6");
}

#[test]
fn test_tx_hash_computation() {
    use decoder_tron::hashing::compute_tx_hash;

    let data = b"test data";
    let hash = compute_tx_hash(data);

    // SHA-256 produces 32 bytes
    assert_eq!(hash.len(), 32);

    // Hash should be deterministic
    let hash2 = compute_tx_hash(data);
    assert_eq!(hash, hash2);
}

// Property-based test: decoder never panics on arbitrary input
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_decoder_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
            // Should either succeed or return an error, never panic
            let _result = TronDecoder::decode(&bytes);
            // If we get here without panicking, test passes
        }

        #[test]
        fn prop_decoder_rejects_empty(_unit in 0u8..1) {
            let result = TronDecoder::decode(&[]);
            prop_assert!(result.is_err());
        }
    }
}
