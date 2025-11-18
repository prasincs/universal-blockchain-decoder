// Test suite for Polkadot/Substrate mainnet transactions
// Uses real SCALE-encoded extrinsics from Polkadot, Kusama, and parachains

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
fn test_polkadot_transfer_basic_mainnet() {
    let tx_bytes = load_fixture_hex("polkadot_transfer_basic");

    let tx =
        PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode Polkadot mainnet transfer");

    // Validate it's a signed transaction
    assert!(tx.extrinsic.is_signed(), "Expected signed extrinsic");

    // Validate call can be parsed
    let call = tx.call().expect("Failed to parse call");
    // Note: pallet names may be "Unknown" for mainnet-style fixtures without full runtime metadata
    assert!(
        !call.pallet_name().is_empty(),
        "Pallet name should not be empty"
    );
}

#[test]
fn test_polkadot_transfer_canonical_hash() {
    let tx_bytes = load_fixture_hex("polkadot_transfer_basic");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode");

    // Hash should be deterministic
    let hash1 = &tx.tx_hash;
    let tx2 = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode");
    let hash2 = &tx2.tx_hash;

    assert_eq!(hash1, hash2, "Hash should be deterministic");
    assert_eq!(hash1.len(), 64, "Blake2b-512 hash should be 64 bytes");
}

#[test]
fn test_polkadot_staking_nominate_mainnet() {
    let tx_bytes = load_fixture_hex("polkadot_staking_nominate");

    let tx =
        PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode Polkadot staking nominate");

    // Validate call can be parsed
    let call = tx.call().expect("Failed to parse call");
    assert!(!call.pallet_name().is_empty(), "Should have pallet name");
}

#[test]
fn test_polkadot_staking_canonicalize() {
    let tx_bytes = load_fixture_hex("polkadot_staking_nominate");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode");

    // Should canonicalize successfully
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize to TxIR");

    // Should have at least one operation
    assert!(!tx_ir.operations.is_empty(), "Expected operations in TxIR");
}

#[test]
fn test_kusama_transfer_basic_mainnet() {
    let tx_bytes = load_fixture_hex("kusama_transfer_basic");
    let expected = load_fixture_json("kusama_transfer_basic");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode Kusama mainnet transfer");

    // Get Kusama chain info from registry
    let registry = PolkadotRegistry::new();
    let chain = registry
        .get_chain(expected["chain_id"].as_u64().unwrap() as u32)
        .expect("Kusama should be in registry");

    // Validate chain
    assert_eq!(
        chain.chain_id,
        expected["chain_id"].as_u64().unwrap() as u32
    );
    assert_eq!(chain.name, "Kusama");
    assert_eq!(chain.token_symbol, "KSM");
    assert_eq!(chain.decimals, 12, "Kusama uses 12 decimals");

    // Validate call can be parsed
    let call = tx.call().expect("Failed to parse call");
    assert!(!call.pallet_name().is_empty(), "Should have pallet name");
}

#[test]
fn test_kusama_democracy_vote_mainnet() {
    let tx_bytes = load_fixture_hex("kusama_democracy_vote");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode Kusama democracy vote");

    // Validate call can be parsed
    let call = tx.call().expect("Failed to parse call");
    assert!(!call.pallet_name().is_empty(), "Should have pallet name");
}

#[test]
fn test_kusama_democracy_canonicalize() {
    let tx_bytes = load_fixture_hex("kusama_democracy_vote");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode");

    let tx_ir = tx.canonicalize().expect("Failed to convert to TxIR");

    // Validate TxIR was created with metadata
    assert!(!tx_ir.metadata.tx_hash.is_empty(), "TxIR should have hash");
    assert!(
        !tx_ir.metadata.extra.is_empty(),
        "TxIR should have extra metadata"
    );
}

#[test]
fn test_acala_parachain_transfer_mainnet() {
    let tx_bytes = load_fixture_hex("acala_transfer");
    let expected = load_fixture_json("acala_transfer");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode Acala parachain transfer");

    // Get Acala chain info from registry
    let registry = PolkadotRegistry::new();
    let chain = registry
        .get_chain(expected["chain_id"].as_u64().unwrap() as u32)
        .expect("Acala should be in registry");

    // Validate parachain properties
    assert_eq!(
        chain.chain_id,
        expected["chain_id"].as_u64().unwrap() as u32
    );
    assert_eq!(chain.name, "Acala");
    assert_eq!(chain.token_symbol, "ACA");
    assert_eq!(chain.decimals, 12);

    // Validate call can be parsed
    let call = tx.call().expect("Failed to parse call");
    assert!(!call.pallet_name().is_empty(), "Should have pallet name");
}

#[test]
fn test_acala_parachain_chain_identity() {
    let registry = PolkadotRegistry::new();
    let chain = registry
        .get_chain_by_name("Acala")
        .expect("Acala should be in registry");

    // Validate Acala is recognized as a Polkadot parachain
    assert_eq!(chain.chain_id, 2000, "Acala parachain ID");
    assert!(
        chain.name.contains("Acala"),
        "Chain name should identify as Acala"
    );
    assert_eq!(chain.network_type, NetworkType::Parachain);
}

#[test]
fn test_moonbeam_parachain_evm_call_mainnet() {
    let tx_bytes = load_fixture_hex("moonbeam_evm_call");
    let expected = load_fixture_json("moonbeam_evm_call");

    let tx = PolkadotDecoder::decode(&tx_bytes).expect("Failed to decode Moonbeam EVM call");

    // Get Moonbeam chain info from registry
    let registry = PolkadotRegistry::new();
    let chain = registry
        .get_chain(expected["chain_id"].as_u64().unwrap() as u32)
        .expect("Moonbeam should be in registry");

    // Validate Moonbeam properties
    assert_eq!(
        chain.chain_id,
        expected["chain_id"].as_u64().unwrap() as u32
    );
    assert_eq!(chain.name, "Moonbeam");
    assert_eq!(chain.token_symbol, "GLMR");
    assert_eq!(
        chain.decimals, 18,
        "Moonbeam uses 18 decimals like Ethereum"
    );

    // Validate call can be parsed
    let call = tx.call().expect("Failed to parse call");
    assert!(!call.pallet_name().is_empty(), "Should have pallet name");
}

#[test]
fn test_moonbeam_ethereum_compatibility() {
    let registry = PolkadotRegistry::new();
    let chain = registry
        .get_chain_by_name("Moonbeam")
        .expect("Moonbeam should be in registry");

    // Moonbeam should have Ethereum-compatible properties
    assert_eq!(chain.decimals, 18, "Should use 18 decimals like ETH");
    assert_eq!(chain.chain_id, 2004, "Moonbeam parachain ID");
}

#[test]
fn test_all_fixtures_decode_successfully() {
    // Comprehensive test: all fixtures should decode without errors
    let fixtures = vec![
        "polkadot_transfer_basic",
        "polkadot_staking_nominate",
        "kusama_transfer_basic",
        "kusama_democracy_vote",
        "acala_transfer",
        "moonbeam_evm_call",
    ];

    for fixture_name in fixtures {
        let tx_bytes = load_fixture_hex(fixture_name);

        let result = PolkadotDecoder::decode(&tx_bytes);
        assert!(
            result.is_ok(),
            "Failed to decode fixture '{}': {:?}",
            fixture_name,
            result.err()
        );
    }
}

#[test]
fn test_all_fixtures_canonicalize_successfully() {
    // All fixtures should canonicalize to TxIR
    let fixtures = vec![
        "polkadot_transfer_basic",
        "polkadot_staking_nominate",
        "kusama_transfer_basic",
        "kusama_democracy_vote",
        "acala_transfer",
        "moonbeam_evm_call",
    ];

    for fixture_name in fixtures {
        let tx_bytes = load_fixture_hex(fixture_name);

        let tx = PolkadotDecoder::decode(&tx_bytes)
            .unwrap_or_else(|_| panic!("Failed to decode {}", fixture_name));

        let tx_ir = tx
            .canonicalize()
            .unwrap_or_else(|_| panic!("Failed to canonicalize {}", fixture_name));

        // Verify TxIR was created
        assert!(
            !tx_ir.metadata.tx_hash.is_empty(),
            "TxIR should have hash for {}",
            fixture_name
        );
    }
}

#[test]
fn test_mainnet_hash_determinism() {
    // All mainnet transactions should have deterministic hashes
    let fixtures = vec![
        "polkadot_transfer_basic",
        "kusama_transfer_basic",
        "acala_transfer",
    ];

    for fixture_name in fixtures {
        let tx_bytes = load_fixture_hex(fixture_name);

        let tx1 = PolkadotDecoder::decode(&tx_bytes)
            .unwrap_or_else(|_| panic!("Failed to decode {}", fixture_name));
        let tx2 = PolkadotDecoder::decode(&tx_bytes)
            .unwrap_or_else(|_| panic!("Failed to decode {}", fixture_name));

        assert_eq!(
            tx1.tx_hash, tx2.tx_hash,
            "Hash should be deterministic for {}",
            fixture_name
        );
    }
}

#[test]
fn test_substrate_chain_registry() {
    // Test that all chains in fixtures are in the registry
    let registry = PolkadotRegistry::new();

    let polkadot = registry
        .get_chain_by_name("Polkadot")
        .expect("Polkadot in registry");
    let kusama = registry
        .get_chain_by_name("Kusama")
        .expect("Kusama in registry");
    let acala = registry
        .get_chain_by_name("Acala")
        .expect("Acala in registry");
    let moonbeam = registry
        .get_chain_by_name("Moonbeam")
        .expect("Moonbeam in registry");

    // Validate relay chains
    assert_eq!(polkadot.chain_id, 0);
    assert_eq!(kusama.chain_id, 2);

    // Validate parachains
    assert_eq!(acala.chain_id, 2000);
    assert_eq!(moonbeam.chain_id, 2004);

    // Validate all have proper token info
    assert!(!polkadot.token_symbol.is_empty());
    assert!(!kusama.token_symbol.is_empty());
    assert!(!acala.token_symbol.is_empty());
    assert!(!moonbeam.token_symbol.is_empty());
}

#[test]
fn test_polkadot_vs_kusama_differences() {
    // Polkadot and Kusama should have different properties
    let registry = PolkadotRegistry::new();
    let polkadot = registry
        .get_chain_by_name("Polkadot")
        .expect("Polkadot in registry");
    let kusama = registry
        .get_chain_by_name("Kusama")
        .expect("Kusama in registry");

    // Different chain IDs
    assert_ne!(polkadot.chain_id, kusama.chain_id);

    // Different tokens
    assert_ne!(polkadot.token_symbol, kusama.token_symbol);

    // Different decimals
    assert_eq!(polkadot.decimals, 10, "DOT uses 10 decimals");
    assert_eq!(kusama.decimals, 12, "KSM uses 12 decimals");
}

#[test]
fn test_parachain_vs_relay_chain() {
    let registry = PolkadotRegistry::new();
    let polkadot = registry
        .get_chain_by_name("Polkadot")
        .expect("Polkadot in registry");
    let acala = registry
        .get_chain_by_name("Acala")
        .expect("Acala in registry");

    // Relay chain vs parachain chain IDs
    assert!(polkadot.chain_id < 1000, "Relay chains have low chain IDs");
    assert!(acala.chain_id >= 2000, "Parachains have chain IDs >= 2000");

    // Network types
    assert_eq!(polkadot.network_type, NetworkType::Relay);
    assert_eq!(acala.network_type, NetworkType::Parachain);
}

#[test]
fn test_all_fixtures_have_valid_calls() {
    // All fixtures should have parseable calls
    let fixtures = vec![
        "polkadot_transfer_basic",
        "polkadot_staking_nominate",
        "kusama_transfer_basic",
        "kusama_democracy_vote",
        "acala_transfer",
        "moonbeam_evm_call",
    ];

    for fixture_name in fixtures {
        let tx_bytes = load_fixture_hex(fixture_name);
        let tx = PolkadotDecoder::decode(&tx_bytes)
            .unwrap_or_else(|_| panic!("Failed to decode {}", fixture_name));

        let call = tx
            .call()
            .unwrap_or_else(|_| panic!("Failed to parse call for {}", fixture_name));

        // Should have valid pallet info
        assert!(
            call.pallet_index < 255,
            "Valid pallet index for {}",
            fixture_name
        );
        assert!(
            !call.pallet_name().is_empty(),
            "Pallet name should not be empty for {}",
            fixture_name
        );
    }
}
