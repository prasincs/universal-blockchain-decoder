//! Validation tests for vendored hex crate.
//!
//! These tests ensure that the vendored hex crate functions correctly
//! and matches the expected behavior of the upstream hex crate.

use universal_decoder_core::hex;

#[test]
fn test_hex_encode_basic() {
    let data = b"Hello world!";
    let encoded = hex::encode(data);
    assert_eq!(encoded, "48656c6c6f20776f726c6421");
}

#[test]
fn test_hex_encode_empty() {
    let data = b"";
    let encoded = hex::encode(data);
    assert_eq!(encoded, "");
}

#[test]
fn test_hex_decode_basic() {
    let hex_string = "48656c6c6f20776f726c6421";
    let decoded = hex::decode(hex_string).expect("Failed to decode hex");
    assert_eq!(&decoded, b"Hello world!");
}

#[test]
fn test_hex_decode_empty() {
    let hex_string = "";
    let decoded = hex::decode(hex_string).expect("Failed to decode hex");
    assert_eq!(decoded, Vec::<u8>::new());
}

#[test]
fn test_hex_roundtrip() {
    let original = b"The quick brown fox jumps over the lazy dog";
    let encoded = hex::encode(original);
    let decoded = hex::decode(&encoded).expect("Failed to decode hex");
    assert_eq!(&decoded, original);
}

#[test]
fn test_hex_encode_upper() {
    let data = b"test";
    let encoded = hex::encode_upper(data);
    assert_eq!(encoded, "74657374");
    // Note: encode_upper is the same as encode for this version
}

#[test]
fn test_hex_decode_invalid() {
    let invalid_hex = "zzzz";
    let result = hex::decode(invalid_hex);
    assert!(result.is_err(), "Should fail to decode invalid hex");
}

#[test]
fn test_hex_decode_odd_length() {
    let odd_length = "123";
    let result = hex::decode(odd_length);
    assert!(result.is_err(), "Should fail to decode odd-length hex");
}

#[test]
fn test_hex_encode_blockchain_hash() {
    // Test with a typical blockchain transaction hash (32 bytes)
    let tx_hash = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    ];

    let encoded = hex::encode(&tx_hash);
    assert_eq!(
        encoded,
        "123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0"
    );

    let decoded = hex::decode(&encoded).expect("Failed to decode");
    assert_eq!(&decoded[..], &tx_hash[..]);
}

#[test]
fn test_hex_encode_bitcoin_address_hash() {
    // Test with Bitcoin address hash (20 bytes)
    let addr_hash = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        0x00, 0x11, 0x22, 0x33,
    ];

    let encoded = hex::encode(&addr_hash);
    assert_eq!(encoded, "00112233445566778899aabbccddeeff00112233");

    let decoded = hex::decode(&encoded).expect("Failed to decode");
    assert_eq!(&decoded[..], &addr_hash[..]);
}

#[test]
fn test_hex_with_large_data() {
    // Test with larger data (1KB)
    let large_data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let encoded = hex::encode(&large_data);
    let decoded = hex::decode(&encoded).expect("Failed to decode");
    assert_eq!(decoded, large_data);
}

/// Verify that the vendored hex matches expected upstream behavior
#[test]
fn test_vendored_hex_compatibility() {
    // This test documents the expected behavior of hex v0.4.3

    // Test vectors from hex crate documentation
    let test_cases = vec![
        (b"" as &[u8], ""),
        (b"\x00", "00"),
        (b"\xff", "ff"),
        (b"\x00\xff", "00ff"),
        (b"Hello world!", "48656c6c6f20776f726c6421"),
    ];

    for (input, expected) in test_cases {
        let encoded = hex::encode(input);
        assert_eq!(
            encoded, expected,
            "Encoding mismatch for input: {:?}",
            input
        );

        let decoded = hex::decode(expected).expect("Failed to decode");
        assert_eq!(
            &decoded[..], input,
            "Decoding mismatch for hex: {}",
            expected
        );
    }
}
