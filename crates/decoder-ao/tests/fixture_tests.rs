//! Fixture-based tests for AO decoder
//!
//! Tests using realistic ANS-104 DataItem fixtures

use decoder_ao::AODecoder;
use decoder_primitives::prelude::*;
use std::fs;

#[test]
fn test_eth_eval_message_fixture() {
    let bytes = fs::read("tests/fixtures/ao_message_eth_eval.bin").expect(
        "Fixture file not found. Run: cargo test --test fixtures generate_fixtures -- --ignored",
    );

    let tx = AODecoder::decode(&bytes).expect("Failed to decode Ethereum Eval message");

    // Verify signature type
    assert_eq!(
        tx.message.signature_type,
        decoder_ao::types::SignatureType::Ethereum
    );

    // Verify field lengths
    assert_eq!(tx.message.signature.len(), 65);
    assert_eq!(tx.message.owner.len(), 65);

    // Verify target present
    assert!(tx.message.target.is_some());
    assert_eq!(tx.message.target.as_ref().unwrap().len(), 32);

    // Verify no anchor
    assert!(tx.message.anchor.is_none());

    // Verify tags
    assert_eq!(tx.message.tags.len(), 2);
    assert_eq!(tx.action(), Some("Eval"));
    assert_eq!(tx.message.get_tag("Data-Protocol"), Some("ao"));

    // Verify data
    assert!(tx.message.data.starts_with(b"return"));

    // Verify message ID
    assert_eq!(tx.message_id.len(), 32);

    // Canonicalize
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize");
    assert_eq!(tx_ir.chain.family(), ChainFamily::Actor);
    assert_eq!(tx_ir.operations.len(), 1);
    assert_eq!(tx_ir.authorization.signature_scheme, SignatureScheme::Ecdsa);
}

#[test]
fn test_eth_transfer_message_fixture() {
    let bytes =
        fs::read("tests/fixtures/ao_message_eth_transfer.bin").expect("Fixture file not found");

    let tx = AODecoder::decode(&bytes).expect("Failed to decode Ethereum Transfer message");

    assert_eq!(
        tx.message.signature_type,
        decoder_ao::types::SignatureType::Ethereum
    );

    // Verify tags
    assert_eq!(tx.action(), Some("Transfer"));
    assert_eq!(tx.message.get_tag("Amount"), Some("1000"));

    // Verify data is JSON
    assert!(tx.message.data.starts_with(b"{"));

    // Canon icalize
    let tx_ir = tx.canonicalize().unwrap();

    // Verify operation
    if let Operation::ContractCall(call) = &tx_ir.operations[0] {
        assert_eq!(call.method, b"Transfer");
    } else {
        panic!("Expected ContractCall operation");
    }
}

#[test]
fn test_solana_spawn_message_fixture() {
    let bytes =
        fs::read("tests/fixtures/ao_message_solana_spawn.bin").expect("Fixture file not found");

    let tx = AODecoder::decode(&bytes).expect("Failed to decode Solana Spawn message");

    // Verify signature type
    assert_eq!(
        tx.message.signature_type,
        decoder_ao::types::SignatureType::Solana
    );

    // Verify Solana lengths
    assert_eq!(tx.message.signature.len(), 64);
    assert_eq!(tx.message.owner.len(), 32);

    // Verify anchor present
    assert!(tx.message.anchor.is_some());

    // Verify tags
    assert_eq!(tx.message.tags.len(), 3);
    assert_eq!(tx.action(), Some("Spawn-Process"));
    assert_eq!(tx.message.get_tag("Data-Protocol"), Some("ao"));
    assert_eq!(tx.message.get_tag("Type"), Some("Process"));

    // Verify data is Lua code
    assert!(tx.message.data.starts_with(b"-- Lua"));

    // Canonicalize
    let tx_ir = tx.canonicalize().unwrap();
    assert_eq!(tx_ir.authorization.signature_scheme, SignatureScheme::EdDsa);
}

#[test]
fn test_solana_minimal_message_fixture() {
    let bytes =
        fs::read("tests/fixtures/ao_message_solana_minimal.bin").expect("Fixture file not found");

    let tx = AODecoder::decode(&bytes).expect("Failed to decode minimal message");

    // Verify no optional fields
    assert!(tx.message.target.is_none());
    assert!(tx.message.anchor.is_none());
    assert_eq!(tx.message.tags.len(), 0);

    // Verify data
    assert_eq!(tx.message.data, b"ping");

    // Canonicalize - should work even without tags/target
    let tx_ir = tx.canonicalize().unwrap();

    // No operations without Action tag
    assert_eq!(tx_ir.operations.len(), 0);

    // No state deltas without target
    assert_eq!(tx_ir.state_deltas.account_changes.len(), 0);
}

#[test]
fn test_message_with_anchor_fixture() {
    let bytes =
        fs::read("tests/fixtures/ao_message_with_anchor.bin").expect("Fixture file not found");

    let tx = AODecoder::decode(&bytes).expect("Failed to decode message with anchor");

    // Verify anchor present
    assert!(tx.message.anchor.is_some());
    assert_eq!(tx.message.anchor.as_ref().unwrap().len(), 32);

    // Verify this is for replay protection
    let anchor = tx.message.anchor.as_ref().unwrap();
    // Anchor typically starts with timestamp or nonce
    assert!(anchor[0] == 0x00); // Our fixture pattern
}

#[test]
fn test_multi_tag_message_fixture() {
    let bytes =
        fs::read("tests/fixtures/ao_message_multi_tags.bin").expect("Fixture file not found");

    let tx = AODecoder::decode(&bytes).expect("Failed to decode multi-tag message");

    // Verify all 5 tags
    assert_eq!(tx.message.tags.len(), 5);

    assert_eq!(tx.action(), Some("Transfer"));
    assert_eq!(tx.message.get_tag("From"), Some("user_alice"));
    assert_eq!(tx.message.get_tag("To"), Some("user_bob"));
    assert_eq!(tx.message.get_tag("Amount"), Some("5000"));
    assert_eq!(tx.message.get_tag("Data-Protocol"), Some("ao"));

    // Verify data is JSON memo
    assert!(tx.message.data.starts_with(b"{"));
    assert!(String::from_utf8_lossy(&tx.message.data).contains("memo"));

    // Canonicalize
    let tx_ir = tx.canonicalize().unwrap();

    // Verify metadata includes tag information
    assert!(tx_ir.metadata.extra.contains("tags_count"));
    assert!(tx_ir.metadata.extra.contains("5"));
}

#[test]
fn test_all_fixtures_decode_successfully() {
    let fixtures = vec![
        "ao_message_eth_eval.bin",
        "ao_message_eth_transfer.bin",
        "ao_message_solana_spawn.bin",
        "ao_message_solana_minimal.bin",
        "ao_message_with_anchor.bin",
        "ao_message_multi_tags.bin",
    ];

    for fixture in fixtures {
        let path = format!("tests/fixtures/{}", fixture);
        let bytes =
            fs::read(&path).unwrap_or_else(|_| panic!("Failed to read fixture: {}", fixture));

        let tx = AODecoder::decode(&bytes)
            .unwrap_or_else(|e| panic!("Failed to decode {}: {}", fixture, e));

        // All fixtures should canonicalize successfully
        tx.canonicalize()
            .unwrap_or_else(|e| panic!("Failed to canonicalize {}: {}", fixture, e));

        println!("✓ {} decoded and canonicalized successfully", fixture);
    }
}

#[test]
fn test_fixture_message_id_determinism() {
    let bytes = fs::read("tests/fixtures/ao_message_eth_eval.bin").expect("Fixture file not found");

    let tx1 = AODecoder::decode(&bytes).unwrap();
    let tx2 = AODecoder::decode(&bytes).unwrap();

    // Message IDs should be identical
    assert_eq!(tx1.message_id, tx2.message_id);

    // TX IRs should have same hash
    let tx_ir1 = tx1.canonicalize().unwrap();
    let tx_ir2 = tx2.canonicalize().unwrap();

    assert_eq!(tx_ir1.metadata.tx_hash, tx_ir2.metadata.tx_hash);
}

#[test]
fn test_fixture_validation() {
    let bytes = fs::read("tests/fixtures/ao_message_eth_eval.bin").expect("Fixture file not found");

    let tx = AODecoder::decode(&bytes).unwrap();

    // Validate should pass for valid fixtures
    tx.validate().expect("Fixture validation failed");
}
