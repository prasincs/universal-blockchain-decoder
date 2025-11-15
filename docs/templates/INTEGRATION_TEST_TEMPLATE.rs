// Integration tests for {{CHAIN}} decoder
//
// These tests verify the decoder against real blockchain transaction data.
// Add test fixtures to tests/fixtures/ directory.

use decoder_{{chain}}::*;
use std::fs;
use std::path::Path;

// =============================================================================
// FIXTURE-BASED TESTS
// =============================================================================

#[test]
fn test_decode_fixture_001_simple_transfer() {
    let fixture_path = "tests/fixtures/mainnet/tx_001_simple.hex";

    // Skip if fixture doesn't exist yet
    if !Path::new(fixture_path).exists() {
        eprintln!("⚠️  Fixture not found: {}", fixture_path);
        eprintln!("   Add real transaction hex to this file to enable test");
        return;
    }

    let hex = fs::read_to_string(fixture_path)
        .expect("Failed to read fixture")
        .trim()
        .to_string();

    let bytes = universal_decoder_core::hex::decode(&hex)
        .expect("Failed to decode hex fixture");

    // Decode transaction
    let tx = {{CHAIN}}Decoder::decode(&bytes)
        .expect("Failed to decode fixture tx_001");

    // Basic assertions (customize based on known transaction properties)
    assert!(tx.validate().is_ok(), "Transaction should be valid");

    // TODO: Add specific assertions for this transaction:
    // assert_eq!(tx.version, 1);
    // assert_eq!(tx.sender, "0x...");
    // etc.
}

#[test]
fn test_decode_fixture_002_complex() {
    let fixture_path = "tests/fixtures/mainnet/tx_002_complex.hex";

    if !Path::new(fixture_path).exists() {
        eprintln!("⚠️  Fixture not found: {}", fixture_path);
        return;
    }

    let hex = fs::read_to_string(fixture_path)
        .expect("Failed to read fixture")
        .trim()
        .to_string();

    let bytes = universal_decoder_core::hex::decode(&hex)
        .expect("Failed to decode hex");

    let tx = {{CHAIN}}Decoder::decode(&bytes)
        .expect("Failed to decode fixture tx_002");

    assert!(tx.validate().is_ok());

    // TODO: Add specific assertions
}

#[test]
fn test_decode_fixture_003_edge_case() {
    let fixture_path = "tests/fixtures/mainnet/tx_003_edge_case.hex";

    if !Path::new(fixture_path).exists() {
        eprintln!("⚠️  Fixture not found: {}", fixture_path);
        return;
    }

    let hex = fs::read_to_string(fixture_path)
        .expect("Failed to read fixture")
        .trim()
        .to_string();

    let bytes = universal_decoder_core::hex::decode(&hex)
        .expect("Failed to decode hex");

    let tx = {{CHAIN}}Decoder::decode(&bytes)
        .expect("Failed to decode fixture tx_003");

    assert!(tx.validate().is_ok());

    // TODO: Add specific assertions for edge case
}

// =============================================================================
// INVALID TRANSACTION TESTS
// =============================================================================

#[test]
fn test_decode_invalid_empty() {
    let result = {{CHAIN}}Decoder::decode(&[]);
    assert!(result.is_err(), "Empty input should be rejected");
}

#[test]
fn test_decode_invalid_truncated() {
    let fixture_path = "tests/fixtures/invalid/invalid_truncated.hex";

    if !Path::new(fixture_path).exists() {
        // Create a minimal truncated transaction for testing
        let bytes = vec![0x01]; // Single byte (obviously truncated)
        let result = {{CHAIN}}Decoder::decode(&bytes);
        assert!(result.is_err(), "Truncated transaction should be rejected");
        return;
    }

    let hex = fs::read_to_string(fixture_path)
        .expect("Failed to read fixture")
        .trim()
        .to_string();

    let bytes = universal_decoder_core::hex::decode(&hex)
        .expect("Failed to decode hex");

    let result = {{CHAIN}}Decoder::decode(&bytes);
    assert!(result.is_err(), "Truncated transaction should be rejected");
}

#[test]
fn test_decode_invalid_wrong_version() {
    let fixture_path = "tests/fixtures/invalid/invalid_wrong_version.hex";

    if !Path::new(fixture_path).exists() {
        eprintln!("⚠️  Fixture not found: {}", fixture_path);
        eprintln!("   Add invalid transaction with wrong version");
        return;
    }

    let hex = fs::read_to_string(fixture_path)
        .expect("Failed to read fixture")
        .trim()
        .to_string();

    let bytes = universal_decoder_core::hex::decode(&hex)
        .expect("Failed to decode hex");

    let result = {{CHAIN}}Decoder::decode(&bytes);
    assert!(result.is_err(), "Invalid version should be rejected");
}

// =============================================================================
// CANONICALIZATION TESTS
// =============================================================================

#[test]
fn test_canonicalization_fixture_001() {
    let fixture_path = "tests/fixtures/mainnet/tx_001_simple.hex";

    if !Path::new(fixture_path).exists() {
        return;
    }

    let hex = fs::read_to_string(fixture_path)
        .expect("Failed to read fixture")
        .trim()
        .to_string();

    let bytes = universal_decoder_core::hex::decode(&hex)
        .expect("Failed to decode hex");

    let tx = {{CHAIN}}Decoder::decode(&bytes)
        .expect("Failed to decode");

    // Test canonicalization
    let ir = tx.canonicalize()
        .expect("Failed to canonicalize");

    // Test deterministic serialization
    let canonical1 = ir.to_canonical_bytes()
        .expect("Failed to serialize to canonical bytes");

    let canonical2 = ir.to_canonical_bytes()
        .expect("Failed to serialize to canonical bytes (2nd time)");

    assert_eq!(canonical1, canonical2, "Canonical bytes should be deterministic");

    // Test hash determinism
    let hash1 = ir.canonical_hash()
        .expect("Failed to compute canonical hash");

    let hash2 = ir.canonical_hash()
        .expect("Failed to compute canonical hash (2nd time)");

    assert_eq!(hash1, hash2, "Canonical hash should be deterministic");
}

// =============================================================================
// CHAIN IDENTITY TESTS
// =============================================================================

#[test]
fn test_chain_identity() {
    let chain = {{CHAIN}}Decoder::chain();

    // TODO: Update with actual chain values
    // assert_eq!(chain.chain_id(), 12345);
    assert!(!chain.chain_name().is_empty());
    // assert_eq!(chain.chain_family(), ChainFamily::Account);
}

// =============================================================================
// HELPERS
// =============================================================================

/// Helper to load all fixtures from a directory
#[allow(dead_code)]
fn load_fixtures(dir: &str) -> Vec<(String, Vec<u8>)> {
    let path = Path::new(dir);
    if !path.exists() {
        return vec![];
    }

    fs::read_dir(path)
        .expect("Failed to read fixtures directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.extension()? != "hex" {
                return None;
            }

            let name = path.file_name()?.to_str()?.to_string();
            let hex = fs::read_to_string(&path).ok()?;
            let bytes = universal_decoder_core::hex::decode(hex.trim()).ok()?;

            Some((name, bytes))
        })
        .collect()
}

// Batch test all mainnet fixtures
#[test]
fn test_all_mainnet_fixtures() {
    let fixtures = load_fixtures("tests/fixtures/mainnet");

    if fixtures.is_empty() {
        eprintln!("⚠️  No fixtures found in tests/fixtures/mainnet/");
        eprintln!("   Add .hex files to enable batch testing");
        return;
    }

    for (name, bytes) in fixtures {
        let result = {{CHAIN}}Decoder::decode(&bytes);
        assert!(
            result.is_ok(),
            "Failed to decode fixture: {} - Error: {:?}",
            name,
            result.err()
        );

        let tx = result.unwrap();
        assert!(
            tx.validate().is_ok(),
            "Fixture {} failed validation",
            name
        );
    }

    println!("✅ Decoded {} mainnet fixtures successfully", fixtures.len());
}

// Batch test all invalid fixtures (should all fail to decode)
#[test]
fn test_all_invalid_fixtures() {
    let fixtures = load_fixtures("tests/fixtures/invalid");

    if fixtures.is_empty() {
        return;
    }

    for (name, bytes) in &fixtures {
        let result = {{CHAIN}}Decoder::decode(bytes);
        assert!(
            result.is_err(),
            "Invalid fixture {} should have failed to decode, but succeeded",
            name
        );
    }

    println!("✅ Rejected {} invalid fixtures correctly", fixtures.len());
}

// =============================================================================
// VALIDATION AGAINST REFERENCE IMPLEMENTATION (Optional)
// =============================================================================

// If you have a reference implementation in dev-dependencies:
// (e.g., bitcoin crate, alloy, solana-transaction-status)

#[cfg(feature = "reference_validation")]
#[test]
fn test_against_reference_implementation() {
    // Example for Bitcoin:
    // let tx_bytes = include_bytes!("fixtures/mainnet/tx_001_simple.hex");
    // let our_tx = {{CHAIN}}Decoder::decode(tx_bytes).unwrap();
    // let ref_tx = reference_crate::Transaction::decode(tx_bytes).unwrap();
    // assert_eq!(our_tx.version, ref_tx.version);

    todo!("Implement reference validation when reference crate available")
}
