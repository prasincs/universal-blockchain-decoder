// Test suite for REAL Polkadot/Substrate mainnet transactions
// Only includes tests using verified real SCALE-encoded extrinsics from actual blockchains
//
// Additional real mainnet transactions can be added as they are found and verified.

use decoder_polkadot::*;
use decoder_primitives::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Helper to load test fixtures
fn load_fixture_hex(name: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(format!("{}.hex", name));

    let hex_str = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", path.display(), e));

    hex::decode(hex_str.trim())
        .unwrap_or_else(|e| panic!("Failed to decode hex from {}: {}", path.display(), e))
}

/// Helper to load expected values from JSON fixture
fn load_fixture_json(name: &str) -> serde_json::Value {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(format!("{}.json", name));

    let json_str = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", path.display(), e));

    serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("Failed to parse JSON from {}: {}", path.display(), e))
}

#[test]
fn test_polkadot_real_mainnet_transfer() {
    // This is a REAL Polkadot mainnet transaction from the official Polkadot Developer Docs
    // Source: https://docs.polkadot.com/develop/toolkit/integrations/transaction-construction/
    //
    // Transaction details:
    // - Type: Signed extrinsic (version 4)
    // - Signature: Sr25519
    // - Era: Mortal
    // - Call: Balances::transfer_keep_alive
    // - Amount: 10000000000 Planck (1.0 DOT)

    let tx_bytes = load_fixture_hex("polkadot_transfer_basic");
    let expected = load_fixture_json("polkadot_transfer_basic");

    // Decode the transaction
    let tx = PolkadotDecoder::decode(&tx_bytes)
        .expect("Failed to decode real Polkadot mainnet transfer");

    // Validate it's a signed transaction
    assert!(tx.extrinsic.is_signed(), "Expected signed extrinsic");

    // Validate call can be parsed
    let call = tx.call().expect("Failed to parse call");
    assert!(
        !call.pallet_name().is_empty(),
        "Pallet name should not be empty"
    );

    // Validate we can canonicalize to TxIR
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize to TxIR");

    // Validate TxIR structure
    assert!(
        !tx_ir.metadata.tx_hash.is_empty(),
        "TxIR should have transaction hash"
    );
    assert!(!tx_ir.operations.is_empty(), "TxIR should have operations");

    // Validate canonical hash is deterministic
    let hash1 = tx_ir
        .canonical_hash()
        .expect("Failed to compute canonical hash");
    let hash2 = tx_ir
        .canonical_hash()
        .expect("Failed to compute canonical hash");
    assert_eq!(hash1, hash2, "Canonical hash should be deterministic");

    println!("✓ Successfully decoded real Polkadot mainnet transfer");
    println!("  Chain: {}", expected["chain"]);
    println!("  Amount: {} DOT", expected["amount_dot"]);
    println!("  Source: {}", expected["url"]);
}

#[test]
fn test_polkadot_real_mainnet_canonical_serialization() {
    // Verify that canonical serialization works correctly for real mainnet transactions
    let tx_bytes = load_fixture_hex("polkadot_transfer_basic");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode");
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize");

    // Test Borsh canonical serialization
    let canonical_bytes = tx_ir
        .to_canonical_bytes()
        .expect("Failed to serialize to Borsh");
    assert!(
        !canonical_bytes.is_empty(),
        "Canonical bytes should not be empty"
    );

    // Test determinism: serialize twice, should get same result
    let canonical_bytes2 = tx_ir
        .to_canonical_bytes()
        .expect("Failed to serialize to Borsh");
    assert_eq!(
        canonical_bytes, canonical_bytes2,
        "Canonical serialization must be deterministic"
    );

    println!("✓ Canonical serialization verified for real mainnet transaction");
    println!("  Original size: {} bytes", tx_bytes.len());
    println!("  Canonical size: {} bytes", canonical_bytes.len());
}

#[test]
fn test_polkadot_real_mainnet_hash_stability() {
    // Verify that transaction hashes are stable across decoder runs
    let tx_bytes = load_fixture_hex("polkadot_transfer_basic");

    // Decode and hash multiple times
    let mut hashes = vec![];
    for _ in 0..5 {
        let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode");
        let tx_ir = tx.canonicalize().expect("Failed to canonicalize");
        let hash = tx_ir.canonical_hash().expect("Failed to hash");
        hashes.push(hash);
    }

    // All hashes should be identical
    for (i, hash) in hashes.iter().enumerate().skip(1) {
        assert_eq!(
            &hashes[0], hash,
            "Hash mismatch at iteration {}: expected {:?}, got {:?}",
            i, hashes[0], hash
        );
    }

    println!("✓ Transaction hash stability verified");
    println!("  Hash (hex): {}", hex::encode(&hashes[0]));
}

#[test]
fn test_polkadot_registry_mainnet_chains() {
    // Verify that Polkadot registry has correct chain information
    let registry = PolkadotRegistry::new();

    // Test Polkadot relay chain
    let polkadot = registry
        .get_chain(0)
        .expect("Polkadot should be in registry");
    assert_eq!(polkadot.chain_id, 0);
    assert_eq!(polkadot.name, "Polkadot");
    assert_eq!(polkadot.token_symbol, "DOT");
    assert_eq!(polkadot.decimals, 10, "DOT uses 10 decimals");
    assert_eq!(
        polkadot.network_type,
        NetworkType::Relay,
        "Polkadot is a relay chain"
    );

    // Test Kusama relay chain
    let kusama = registry.get_chain(2).expect("Kusama should be in registry");
    assert_eq!(kusama.chain_id, 2);
    assert_eq!(kusama.name, "Kusama");
    assert_eq!(kusama.token_symbol, "KSM");
    assert_eq!(kusama.decimals, 12, "KSM uses 12 decimals");
    assert_eq!(
        kusama.network_type,
        NetworkType::Relay,
        "Kusama is a relay chain"
    );

    println!("✓ Polkadot registry verified with correct mainnet chain data");
}
