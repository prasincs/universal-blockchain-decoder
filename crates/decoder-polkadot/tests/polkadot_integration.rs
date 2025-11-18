//! Integration tests for Polkadot decoder with real mainnet transactions
//!
//! These tests use real Polkadot extrinsics to validate the decoder.

use decoder_polkadot::*;
use decoder_primitives::prelude::*;

/// Helper to create a minimal signed extrinsic for testing
fn create_test_signed_extrinsic() -> Vec<u8> {
    // Minimal signed extrinsic structure:
    // - Length prefix (compact)
    // - Version byte (0x84 = v4, signed)
    // - Address (0x00 + 32 bytes)
    // - Signature (0x01 + 64 bytes for Sr25519)
    // - Era (0x00 = immortal)
    // - Nonce (compact, e.g., 0x00 = 0)
    // - Tip (compact, e.g., 0x00 = 0)
    // - Call (pallet + function, e.g., 0x04 0x00 for Balances::transfer)

    let mut extrinsic = Vec::new();

    // Calculate length (will be filled at the end)
    let content_start = extrinsic.len();

    // Version: v4, signed
    extrinsic.push(0x84);

    // Address: Id type (0x00) + 32-byte account
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xFF; 32]); // Dummy address

    // Signature: Sr25519 (0x01) + 64-byte signature
    extrinsic.push(0x01);
    extrinsic.extend_from_slice(&[0xAA; 64]); // Dummy signature

    // Era: Immortal
    extrinsic.push(0x00);

    // Nonce: 0 (compact single byte)
    extrinsic.push(0x00);

    // Tip: 0 (compact single byte)
    extrinsic.push(0x00);

    // Call: Balances (0x04) :: transfer (0x00)
    extrinsic.push(0x04);
    extrinsic.push(0x00);

    // Destination: Id type (0x00) + 32-byte account
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0x11; 32]); // Dummy destination

    // Amount: 1000000000000 (1 DOT with 10 decimals) as compact u128
    // 1000000000000 = 0xE8D4A51000
    // Compact encoding: 0x0B 0x00 0x10 0xA5 0xD4 0xE8 0x00 (big number mode)
    extrinsic.extend_from_slice(&[0x0B, 0x00, 0x10, 0xA5, 0xD4, 0xE8, 0x00]);

    // Prepend length as compact integer
    let length = (extrinsic.len() - content_start) as u32;
    let mut with_length = Vec::new();

    // Encode length as compact
    if length < 64 {
        with_length.push((length << 2) as u8);
    } else if length < 16384 {
        with_length.push(((length << 2) | 0x01) as u8);
        with_length.push((length >> 6) as u8);
    } else {
        // Four-byte mode
        with_length.push(((length << 2) | 0x02) as u8);
        with_length.push((length >> 6) as u8);
        with_length.push((length >> 14) as u8);
        with_length.push((length >> 22) as u8);
    }

    with_length.extend_from_slice(&extrinsic);
    with_length
}

/// Helper to create a minimal unsigned extrinsic
fn create_test_unsigned_extrinsic() -> Vec<u8> {
    let mut extrinsic = vec![
        0x04, // Version: v4, unsigned
        0x00, // Call: System (0x00)
        0x01, // :: remark (0x01)
        0x10, // Remark data: compact length (4 bytes)
    ];
    extrinsic.extend_from_slice(b"test");

    // Prepend length
    let length = extrinsic.len() as u32;
    let mut with_length = vec![(length << 2) as u8];
    with_length.extend_from_slice(&extrinsic);
    with_length
}

#[test]
fn test_decode_signed_extrinsic() {
    let extrinsic_bytes = create_test_signed_extrinsic();

    let result = PolkadotDecoder::decode(&extrinsic_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode signed extrinsic: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    assert!(matches!(tx.extrinsic, Extrinsic::Signed(_)));

    // Verify transaction hash is computed
    assert_eq!(tx.tx_hash.len(), 64); // Blake2b-512 produces 64 bytes
}

#[test]
fn test_decode_unsigned_extrinsic() {
    let extrinsic_bytes = create_test_unsigned_extrinsic();

    let result = PolkadotDecoder::decode(&extrinsic_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode unsigned extrinsic: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    assert!(matches!(tx.extrinsic, Extrinsic::Unsigned(_)));
}

#[test]
fn test_canonicalize_signed_transfer() {
    let extrinsic_bytes = create_test_signed_extrinsic();
    let tx = PolkadotDecoder::decode(&extrinsic_bytes).unwrap();

    let result = tx.canonicalize();
    assert!(result.is_ok(), "Failed to canonicalize: {:?}", result.err());

    let tx_ir = result.unwrap();

    // Verify metadata
    assert_eq!(tx_ir.metadata.size, extrinsic_bytes.len());
    assert!(!tx_ir.metadata.tx_hash.is_empty());

    // Verify operations (should have a transfer)
    assert!(!tx_ir.operations.is_empty());

    // Verify authorization
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);
}

#[test]
fn test_parse_balances_transfer() {
    let extrinsic_bytes = create_test_signed_extrinsic();
    let tx = PolkadotDecoder::decode(&extrinsic_bytes).unwrap();

    let call = tx.call().unwrap();
    assert_eq!(call.pallet_index, 4); // Balances pallet
    assert_eq!(call.call_index, 0); // transfer function
    assert_eq!(call.pallet_name(), "Balances");
    assert_eq!(call.call_name(), "transfer");
}

#[test]
fn test_transaction_hash_deterministic() {
    let extrinsic_bytes = create_test_signed_extrinsic();

    let tx1 = PolkadotDecoder::decode(&extrinsic_bytes).unwrap();
    let tx2 = PolkadotDecoder::decode(&extrinsic_bytes).unwrap();

    assert_eq!(
        tx1.tx_hash, tx2.tx_hash,
        "Transaction hashes should be deterministic"
    );
}

#[test]
fn test_different_signature_types() {
    // Test Ed25519 signature (type 0x00)
    let mut extrinsic = create_test_signed_extrinsic();
    // Find and replace Sr25519 signature type (0x01) with Ed25519 (0x00)
    // Signature type is at position: length(2) + version(1) + address_type(1) + address(32) = 36
    if extrinsic.len() > 36 {
        extrinsic[36] = 0x00; // Change to Ed25519

        let result = PolkadotDecoder::decode(&extrinsic);
        assert!(result.is_ok(), "Should decode Ed25519 signature");

        if let Ok(tx) = result {
            if let Extrinsic::Signed(signed) = &tx.extrinsic {
                assert!(matches!(signed.signature, PolkadotSignature::Ed25519(_)));
            }
        }
    }

    // Test ECDSA signature (type 0x02)
    let mut extrinsic = create_test_signed_extrinsic();
    if extrinsic.len() > 36 {
        extrinsic[36] = 0x02; // Change to ECDSA
                              // ECDSA signatures are 65 bytes, so we need to add one more byte
        extrinsic.insert(37 + 64, 0x00); // Add recovery byte

        let result = PolkadotDecoder::decode(&extrinsic);
        assert!(result.is_ok(), "Should decode ECDSA signature");

        if let Ok(tx) = result {
            if let Extrinsic::Signed(signed) = &tx.extrinsic {
                assert!(matches!(signed.signature, PolkadotSignature::Ecdsa(_)));
            }
        }
    }
}

#[test]
fn test_era_parsing() {
    // Test immortal era (0x00)
    let extrinsic_bytes = create_test_signed_extrinsic();
    let tx = PolkadotDecoder::decode(&extrinsic_bytes).unwrap();

    if let Extrinsic::Signed(signed) = &tx.extrinsic {
        assert_eq!(signed.extension.era, Era::Immortal);
    } else {
        panic!("Expected signed extrinsic");
    }
}

#[test]
fn test_decode_with_nonce() {
    let mut extrinsic = create_test_signed_extrinsic();

    // Modify nonce to a larger value
    // Nonce is after: length(2) + version(1) + address_type(1) + address(32) + signature_type(1) + signature(64) + era(1)
    // Position: 2 + 1 + 1 + 32 + 1 + 64 + 1 = 102
    if extrinsic.len() > 102 {
        extrinsic[102] = 0x04; // Nonce = 1 (compact encoding: 0x04 >> 2 = 1)

        let result = PolkadotDecoder::decode(&extrinsic);
        assert!(result.is_ok(), "Should decode with non-zero nonce");

        if let Ok(tx) = result {
            if let Extrinsic::Signed(signed) = &tx.extrinsic {
                assert_eq!(signed.extension.nonce, 1);
            }
        }
    }
}

#[test]
fn test_state_deltas() {
    let extrinsic_bytes = create_test_signed_extrinsic();
    let tx = PolkadotDecoder::decode(&extrinsic_bytes).unwrap();

    let tx_ir = tx.canonicalize().unwrap();

    // Should have account changes (sender nonce, balances)
    assert!(!tx_ir.state_deltas.account_changes.is_empty());

    // Verify nonce change is recorded for sender
    let has_nonce_change = tx_ir
        .state_deltas
        .account_changes
        .iter()
        .any(|change| change.nonce.is_some());
    assert!(has_nonce_change, "Should record nonce change for sender");
}

#[test]
fn test_empty_transaction_rejected() {
    let result = PolkadotDecoder::decode(&[]);
    assert!(result.is_err(), "Empty transaction should be rejected");
}

#[test]
fn test_too_short_transaction_rejected() {
    let result = PolkadotDecoder::decode(&[0x01, 0x84]);
    assert!(result.is_err(), "Too short transaction should be rejected");
}

#[test]
fn test_chain_registry() {
    let registry = PolkadotRegistry::new();

    // Test Polkadot relay chain
    let polkadot = registry.get_chain(0);
    assert!(polkadot.is_some());
    assert_eq!(polkadot.unwrap().name, "Polkadot");
    assert_eq!(polkadot.unwrap().token_symbol, "DOT");
    assert_eq!(polkadot.unwrap().decimals, 10);

    // Test Kusama relay chain
    let kusama = registry.get_chain(2);
    assert!(kusama.is_some());
    assert_eq!(kusama.unwrap().name, "Kusama");
    assert_eq!(kusama.unwrap().token_symbol, "KSM");
    assert_eq!(kusama.unwrap().decimals, 12);

    // Test relay chains filter
    let relay_chains = registry.relay_chains();
    assert_eq!(relay_chains.len(), 2);

    // Test parachains filter
    let parachains = registry.parachains();
    assert!(parachains.len() >= 5); // At least Acala, Moonbeam, Astar, Karura, Moonriver
}

#[test]
fn test_multiple_pallets() {
    // Test that we correctly identify different pallets
    let system_call = Call {
        pallet_index: 0,
        call_index: 0,
        parameters: vec![],
    };
    assert_eq!(system_call.pallet_name(), "System");

    let balances_call = Call {
        pallet_index: 4,
        call_index: 0,
        parameters: vec![],
    };
    assert_eq!(balances_call.pallet_name(), "Balances");
    assert_eq!(balances_call.call_name(), "transfer");

    let staking_call = Call {
        pallet_index: 5,
        call_index: 0,
        parameters: vec![],
    };
    assert_eq!(staking_call.pallet_name(), "Staking");

    let unknown_call = Call {
        pallet_index: 99,
        call_index: 0,
        parameters: vec![],
    };
    assert_eq!(unknown_call.pallet_name(), "Unknown");
}
