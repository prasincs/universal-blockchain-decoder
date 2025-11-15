//! Integration tests for Zcash transparent transaction decoder
//!
//! These tests verify:
//! 1. Version detection (Sapling v4, Sprout v1-3, Orchard v5)
//! 2. Overwinter flag parsing
//! 3. Version group ID extraction
//! 4. Error handling for unsupported versions
//! 5. Chain identity
//!
//! ## Note on Test Data
//!
//! **TODO (Phase 1.5)**: Add tests with real Zcash mainnet transparent transaction data
//! - Block 419200+ (Sapling activation)
//! - Transactions with varying inputs/outputs
//! - Transactions with expiry heights
//! - Full roundtrip: decode → canonicalize → hash
//!
//! For now, these tests focus on header parsing, version detection, and error handling.

use decoder_primitives::prelude::*;
use decoder_zcash::{parsing::parse_zcash_header, ZcashDecoder};
use std::io::Cursor;

/// Helper to decode hex string to bytes
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex = hex.trim().replace([' ', '\n', '\r'], "");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Test Case 1: Parse Zcash v4 header (Sapling)
///
/// Verifies Overwinter flag and version_group_id extraction
#[test]
fn test_parse_v4_sapling_header() {
    // Zcash v4 header: version=4 with Overwinter bit (0x80000004)
    // version_group_id=0x892f2085 (Sapling)
    let header_hex = "0400008085202f89";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (version, version_group_id) = result.unwrap();
    assert_eq!(version, 4);
    assert_eq!(version_group_id, 0x892f2085); // Sapling version group
}

/// Test Case 2: Parse Zcash v5 header (Orchard)
///
/// Verifies version 5 detection
#[test]
fn test_parse_v5_orchard_header() {
    // Zcash v5 header: version=5 with Overwinter bit (0x80000005)
    // version_group_id=0x26A7270A (NU5/Orchard)
    let header_hex = "050000800A27A726";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (version, version_group_id) = result.unwrap();
    assert_eq!(version, 5);
    assert_eq!(version_group_id, 0x26A7270A); // Orchard version group
}

/// Test Case 3: Parse Sprout v1 header (no Overwinter)
///
/// Pre-Overwinter transactions have version without the high bit
#[test]
fn test_parse_v1_sprout_header() {
    // Sprout v1: version=1, no Overwinter bit
    // No version_group_id (defaults to 0x00000000)
    let header_hex = "01000000";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (version, version_group_id) = result.unwrap();
    assert_eq!(version, 1);
    assert_eq!(version_group_id, 0x00000000); // No version group
}

/// Test Case 4: Parse Sprout v2 header
#[test]
fn test_parse_v2_sprout_header() {
    let header_hex = "02000000";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (version, version_group_id) = result.unwrap();
    assert_eq!(version, 2);
    assert_eq!(version_group_id, 0x00000000);
}

/// Test Case 5: Reject Sprout v1 in decoder
#[test]
fn test_reject_sprout_v1_decoder() {
    // Minimal Sprout v1 transaction header
    let tx_hex = "01000000";
    let tx_bytes = hex_to_bytes(tx_hex);

    let result = ZcashDecoder::decode(&tx_bytes);

    // Should reject Sprout transactions (any error is acceptable)
    assert!(
        result.is_err(),
        "Should reject Sprout v1 (not supported in Phase 1)"
    );
}

/// Test Case 6: Reject Sprout v2 in decoder
#[test]
fn test_reject_sprout_v2_decoder() {
    let tx_hex = "02000000";
    let tx_bytes = hex_to_bytes(tx_hex);

    let result = ZcashDecoder::decode(&tx_bytes);
    assert!(result.is_err(), "Should reject Sprout v2");
}

/// Test Case 7: Reject Sprout v3 in decoder
#[test]
fn test_reject_sprout_v3_decoder() {
    let tx_hex = "03000000";
    let tx_bytes = hex_to_bytes(tx_hex);

    let result = ZcashDecoder::decode(&tx_bytes);
    assert!(result.is_err(), "Should reject Sprout v3");
}

/// Test Case 8: Reject Orchard v5 in decoder (Phase 4 feature)
#[test]
fn test_reject_orchard_v5_decoder() {
    // Orchard v5 header
    let tx_hex = "050000800A27A726";
    let tx_bytes = hex_to_bytes(tx_hex);

    let result = ZcashDecoder::decode(&tx_bytes);

    // Should reject Orchard transactions (Phase 4, any error is acceptable)
    assert!(
        result.is_err(),
        "Should reject Orchard v5 (not supported yet)"
    );
}

/// Test Case 9: Reject unknown version
#[test]
fn test_reject_unknown_version() {
    // Version 99 (invalid)
    let tx_hex = "63000080DEADBEEF";
    let tx_bytes = hex_to_bytes(tx_hex);

    let result = ZcashDecoder::decode(&tx_bytes);
    assert!(result.is_err(), "Should reject unknown version");
}

/// Test Case 10: Chain identity validation
#[test]
fn test_chain_identity() {
    let chain = ZcashDecoder::chain();
    assert_eq!(chain.chain_id(), 133); // SLIP-44 Zcash
    assert_eq!(chain.chain_name(), "Zcash");
}

/// Test Case 11: Validate format - too short
#[test]
fn test_validate_format_too_short() {
    let tx_bytes = vec![0x04, 0x00, 0x00]; // Only 3 bytes
    let result = ZcashDecoder::decode(&tx_bytes);
    assert!(result.is_err(), "Should reject too-short transactions");
}

/// Test Case 12: Validate format - empty
#[test]
fn test_validate_format_empty() {
    let tx_bytes = vec![];
    let result = ZcashDecoder::decode(&tx_bytes);
    assert!(result.is_err(), "Should reject empty input");
}

/// Test Case 13: Different version group IDs (hypothetical)
///
/// Tests parser's ability to extract arbitrary version_group_id values
#[test]
fn test_different_version_group_ids() {
    // Hypothetical version_group_id = 0x12345678
    let header_hex = "0400008078563412";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (version, version_group_id) = result.unwrap();
    assert_eq!(version, 4);
    assert_eq!(version_group_id, 0x12345678);
}

/// Test Case 14: Header parsing with Overwinter bit
#[test]
fn test_overwinter_bit_detection() {
    // Version 4 without Overwinter bit (invalid for Zcash)
    let header_no_overwinter = "04000000";
    let bytes = hex_to_bytes(header_no_overwinter);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (version, version_group_id) = result.unwrap();
    assert_eq!(version, 4);
    assert_eq!(version_group_id, 0x00000000); // No Overwinter = no version_group_id
}

/// Test Case 15: Maximum version number
#[test]
fn test_max_version_number() {
    // Maximum version without Overwinter bit: 0x7FFFFFFF
    let header_hex = "FFFFFF7F";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (version, _) = result.unwrap();
    assert_eq!(version, 0x7FFFFFFF);
}

/// Test Case 16: Version group ID extraction (little-endian)
#[test]
fn test_version_group_id_little_endian() {
    // Test little-endian byte order for version_group_id
    // Bytes: 0x01 0x02 0x03 0x04 → value: 0x04030201
    let header_hex = "0400008001020304";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    assert!(result.is_ok());

    let (_, version_group_id) = result.unwrap();
    assert_eq!(version_group_id, 0x04030201);
}

/// Test Case 17: Insufficient bytes for header
#[test]
fn test_insufficient_bytes_header() {
    // Only 4 bytes (version), missing version_group_id
    let header_hex = "04000080";
    let bytes = hex_to_bytes(header_hex);
    let mut cursor = Cursor::new(bytes.as_slice());

    let result = parse_zcash_header(&mut cursor);
    // Should fail trying to read version_group_id
    assert!(result.is_err());
}

//
// ==============================================================================
// TODO: Add tests with real Zcash mainnet transparent transaction data
// ==============================================================================
//
// The following test scenarios need real transaction hex from Zcash mainnet:
//
// 1. **Simple v4 transparent transaction (1 input, 2 outputs)**
//    - Block: 419200+ (post-Sapling activation)
//    - Verify: inputs, outputs, expiry_height, locktime
//    - TxIR canonicalization
//
// 2. **Transaction with expiry height**
//    - Common in wallet transactions
//    - Verify expiry_height field parsing
//
// 3. **Multi-input multi-output transaction**
//    - 3+ inputs, 5+ outputs
//    - Stress test parser
//
// 4. **Deterministic canonicalization**
//    - Decode same transaction twice
//    - Verify canonical hashes are identical
//
// 5. **Full pipeline: decode → canonicalize → hash**
//    - End-to-end test
//    - Verify TxIR creation and hashing
//
// 6. **Privacy metadata**
//    - Transparent transactions should have FullyObservable
//    - Will be implemented in Phase 2+
//
// ## How to add real transaction data:
//
// 1. Use Zcash block explorer (e.g., https://zcashblockexplorer.com/)
// 2. Find transparent transactions in blocks 419200+
// 3. Extract raw transaction hex
// 4. Create test fixtures:
//    ```rust
//    #[test]
//    fn test_mainnet_tx_block_419200_tx0() {
//        let tx_hex = "0400008085202f89..."; // Real hex data
//        let tx_bytes = hex_to_bytes(tx_hex);
//        let tx = ZcashDecoder::decode(&tx_bytes).expect("Should decode");
//
//        // Verify transaction properties
//        match tx {
//            ZcashTransaction::Transparent(t) => {
//                assert_eq!(t.version, 4);
//                assert_eq!(t.inputs.len(), expected_inputs);
//                // ... more assertions
//            }
//            _ => panic!("Expected Transparent transaction"),
//        }
//    }
//    ```
//
// 5. Add at least 15+ tests covering:
//    - Different block heights
//    - Varying input/output counts
//    - Different expiry heights
//    - Edge cases (max values, zero expiry, etc.)
//
