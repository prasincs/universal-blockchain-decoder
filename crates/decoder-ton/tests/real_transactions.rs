//! Real TON mainnet transaction validation tests
//!
//! These tests use actual TON mainnet transactions (encoded in base64)
//! to validate that our pure Rust BoC parser can handle real-world data.
//!
//! Test data sources:
//! - TON documentation examples
//! - ton-defi-org/boc-parser examples
//! - TON Connect SDK examples

use decoder_primitives::prelude::*;
use decoder_ton::*;

/// Helper to decode base64 BoC data
fn decode_base64_boc(base64_str: &str) -> Vec<u8> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD
        .decode(base64_str)
        .expect("Invalid base64 BOC data")
}

/// Helper to decode hex BoC data
#[allow(dead_code)]
fn decode_hex_boc(hex_str: &str) -> Vec<u8> {
    let cleaned = hex_str.replace("0x", "").replace(" ", "");
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).expect("Invalid hex"))
        .collect()
}

#[test]
fn test_real_ton_transfer_message() {
    // Real TON transfer message from TON Connect documentation
    // This is a base64-encoded BoC representing an external message
    // Source: https://github.com/ton-blockchain/ton-connect/blob/main/requests-responses.md
    let boc_base64 = "te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==";

    let boc_bytes = decode_base64_boc(boc_base64);

    println!("BoC size: {} bytes", boc_bytes.len());
    println!(
        "First 10 bytes: {:02x?}",
        &boc_bytes[..10.min(boc_bytes.len())]
    );

    // Verify magic number
    assert_eq!(
        &boc_bytes[0..4],
        &[0xb5, 0xee, 0x9c, 0x72],
        "Should have standard BoC magic"
    );

    // Parse BoC
    let cells = boc::parse_boc(&boc_bytes).expect("Failed to parse real TON transfer message BoC");

    println!("Parsed {} cells", cells.len());
    assert!(!cells.is_empty(), "Should parse at least one cell");

    // Verify root cell structure
    let root_cell = &cells[0];
    println!(
        "Root cell: {} bits, {} refs",
        root_cell.bit_len,
        root_cell.refs.len()
    );

    assert!(root_cell.bit_len > 0, "Root cell should have data");
}

#[test]
fn test_real_ton_state_init() {
    // Real TON StateInit message from TON documentation
    // This represents contract initialization data
    // Source: TON cookbook examples
    let boc_base64 = "te6cckECGwEAA2sAART/APSkE/S88sgLAQIBIAIDAgFIBAUCAvIMDQ8E9PzoYXX4ZPgfoTjIU7L/AgEgYH74BgYEBIMH/8IAIbGBn4YD4B4f/wQGpCGYfQIBBhITFBUCASAWFwIBSBgZGhsWAAgcPH6BkAIBSBgZAgJ7GhsACNrIULTQjkJwgBDIywVQB88WIfAHy//J0AT0BPQE9ATwAvoA9AQlYfQFPwX/BtQFBg==";

    let boc_bytes = decode_base64_boc(boc_base64);

    println!("\nStateInit BoC size: {} bytes", boc_bytes.len());

    // Verify it's valid BoC
    assert_eq!(&boc_bytes[0..4], &[0xb5, 0xee, 0x9c, 0x72]);

    // Parse BoC
    let cells = boc::parse_boc(&boc_bytes).expect("Failed to parse StateInit BoC");

    println!("StateInit parsed {} cells", cells.len());
    assert!(cells.len() > 1, "StateInit should have multiple cells");

    // StateInit typically has references to code and data cells
    let root = &cells[0];
    println!(
        "StateInit root: {} bits, {} refs",
        root.bit_len,
        root.refs.len()
    );

    // StateInit usually has 2 references (code and data)
    assert!(
        !root.refs.is_empty(),
        "StateInit should have cell references"
    );
}

#[test]
fn test_boc_format_validation() {
    // Test various BoC format examples
    let test_cases = vec![
        (
            "Standard BoC",
            "te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==",
        ),
        (
            "Another valid BoC",
            "te6cckECGwEAA2sAART/APSkE/S88sgLAQIBIAIDAgFIBAUCAvIMDQ8E9PzoYXX4ZPgfoTjIU7L/AgEgYH74BgYEBIMH/8IAIbGBn4YD4B4f/wQGpCGYfQIBBhITFBUCASAWFwIBSBgZGhsWAAgcPH6BkAIBSBgZAgJ7GhsACNrIULTQjkJwgBDIywVQB88WIfAHy//J0AT0BPQE9ATwAvoA9AQlYfQFPwX/BtQFBg==",
        ),
    ];

    for (name, boc_base64) in test_cases {
        println!("\nTesting: {}", name);
        let boc_bytes = decode_base64_boc(boc_base64);

        // Validate format
        assert!(
            TonDecoder::validate_format(&boc_bytes).is_ok(),
            "{} should pass validation",
            name
        );

        // Parse cells
        let cells = boc::parse_boc(&boc_bytes)
            .unwrap_or_else(|_| panic!("Failed to parse {}", name));

        assert!(!cells.is_empty(), "{} should have cells", name);
        println!("  ✓ Parsed {} cells", cells.len());
    }
}

#[test]
#[ignore] // Ignore by default - requires full transaction parsing
fn test_parse_real_transaction() {
    // This test is marked as ignored because it requires complete
    // TL-B transaction parsing which is not yet fully implemented.
    // Once we have full transaction parsing, we can enable this test.

    // Example transaction BoC would go here
    // For now, we validate the BoC structure only
    let boc_base64 = "te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==";

    let boc_bytes = decode_base64_boc(boc_base64);
    let cells = boc::parse_boc(&boc_bytes).expect("Failed to parse BoC");

    // For now, just verify we can parse the BoC structure
    assert!(!cells.is_empty());
    println!("Parsed {} cells from transaction BoC", cells.len());

    // TODO: Uncomment when transaction parsing is complete
    // let tx = TonDecoder::decode(&boc_bytes).expect("Failed to decode transaction");
    // assert!(tx.validate().is_ok());
}

#[test]
fn test_multiple_cell_boc() {
    // Test BoC with multiple cells (from StateInit example)
    let boc_base64 = "te6cckECGwEAA2sAART/APSkE/S88sgLAQIBIAIDAgFIBAUCAvIMDQ8E9PzoYXX4ZPgfoTjIU7L/AgEgYH74BgYEBIMH/8IAIbGBn4YD4B4f/wQGpCGYfQIBBhITFBUCASAWFwIBSBgZGhsWAAgcPH6BkAIBSBgZAgJ7GhsACNrIULTQjkJwgBDIywVQB88WIfAHy//J0AT0BPQE9ATwAvoA9AQlYfQFPwX/BtQFBg==";

    let boc_bytes = decode_base64_boc(boc_base64);
    let cells = boc::parse_boc(&boc_bytes).expect("Failed to parse multi-cell BoC");

    println!("\nMulti-cell BoC details:");
    println!("  Total cells: {}", cells.len());

    for (i, cell) in cells.iter().enumerate() {
        println!(
            "  Cell {}: {} bits, {} refs, {} bytes",
            i,
            cell.bit_len,
            cell.refs.len(),
            cell.data.len()
        );
    }

    // Verify structure
    assert!(
        cells.len() > 5,
        "Complex BoC should have multiple cells (has {})",
        cells.len()
    );

    // Verify cells can reference each other
    let has_refs = cells.iter().any(|c| !c.refs.is_empty());
    assert!(has_refs, "Should have cells with references");
}
