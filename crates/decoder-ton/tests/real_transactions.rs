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
fn test_corrupt_boc_rejected_by_both_implementations() {
    // CORPUS FINDING (2026-07): this base64 string was checked in as a "real
    // TON StateInit from the TON cookbook", but it is TRUNCATED: its BoC
    // header declares 27 cells with 875 bytes of cell data, while only 163
    // bytes remain in the file. Our parser was correct to reject it - the
    // three tests that previously used it were asserting successful parses
    // of corrupt data. Differential check: tonlib-core (upstream) must
    // reject it too. Backlog: fetch a real StateInit via a corpus tool and
    // add a positive multi-cell test from it.
    let boc_base64 = "te6cckECGwEAA2sAART/APSkE/S88sgLAQIBIAIDAgFIBAUCAvIMDQ8E9PzoYXX4ZPgfoTjIU7L/AgEgYH74BgYEBIMH/8IAIbGBn4YD4B4f/wQGpCGYfQIBBhITFBUCASAWFwIBSBgZGhsWAAgcPH6BkAIBSBgZAgJ7GhsACNrIULTQjkJwgBDIywVQB88WIfAHy//J0AT0BPQE9ATwAvoA9AQlYfQFPwX/BtQFBg==";

    let boc_bytes = decode_base64_boc(boc_base64);
    assert_eq!(&boc_bytes[0..4], &[0xb5, 0xee, 0x9c, 0x72]);

    let ours = boc::parse_boc(&boc_bytes);
    assert!(
        ours.is_err(),
        "our parser accepted a BoC whose header claims more cell data than the file contains"
    );

    let theirs = tonlib_core::cell::BagOfCells::parse(&boc_bytes);
    assert!(
        theirs.is_err(),
        "DISAGREEMENT: tonlib-core accepted a BoC we reject: {theirs:?}"
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
            .unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", name, e));

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
    // A verified-valid multi-cell BoC (4 cells; header sizes are internally
    // consistent). The previous fixture here was the corrupt StateInit - see
    // test_corrupt_boc_rejected_by_both_implementations.
    let boc_base64 = "te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==";

    let boc_bytes = decode_base64_boc(boc_base64);
    let cells = boc::parse_boc(&boc_bytes).expect("Failed to parse multi-cell BoC");

    println!("\nMulti-cell BoC details:");
    println!("  Total cells: {}", cells.len());
    assert!(cells.len() > 1, "should contain multiple cells");

    // Verify cells can reference each other
    let has_refs = cells.iter().any(|c| !c.refs.is_empty());
    assert!(has_refs, "Should have cells with references");

    // Differential: tonlib-core must parse the same bytes and agree on the
    // total number of distinct cells (roots plus all referenced cells).
    let theirs = tonlib_core::cell::BagOfCells::parse(&boc_bytes)
        .expect("tonlib-core rejected a BoC we accept - DISAGREEMENT");
    fn count_cells(
        cell: &std::sync::Arc<tonlib_core::cell::Cell>,
        seen: &mut Vec<*const tonlib_core::cell::Cell>,
    ) {
        let ptr = std::sync::Arc::as_ptr(cell);
        if seen.contains(&ptr) {
            return;
        }
        seen.push(ptr);
        for r in cell.references() {
            count_cells(r, seen);
        }
    }
    let mut seen = Vec::new();
    for root in &theirs.roots {
        count_cells(root, &mut seen);
    }
    assert_eq!(
        cells.len(),
        seen.len(),
        "cell count disagreement with tonlib-core"
    );
}
