//! Integration tests for Bittensor decoder with realistic transaction fixtures
//!
//! These tests use properly SCALE-encoded extrinsics to validate the decoder.

use decoder_bittensor::*;
use decoder_primitives::prelude::*;

mod fixtures;
use fixtures::*;

#[test]
fn test_decode_tao_transfer() {
    let tx_bytes = create_tao_transfer();

    let result = BittensorDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode TAO transfer: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    assert_eq!(tx.raw_bytes.len(), tx_bytes.len());
    assert_eq!(tx.tx_hash.len(), 64); // Blake2b-512

    // Verify it's signed
    assert!(tx.extrinsic.is_signed());

    // Parse call
    let call = tx.call().unwrap();
    assert_eq!(call.pallet_index, 4);
    assert_eq!(call.call_index, 0);
    assert_eq!(call.pallet_name(), "Balances");
    assert_eq!(call.call_name(), "transfer");
}

#[test]
fn test_decode_set_weights() {
    let tx_bytes = create_set_weights();

    let result = BittensorDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode set_weights: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    assert!(tx.extrinsic.is_signed());

    // Parse call
    let call = tx.call().unwrap();
    assert_eq!(call.pallet_index, 7);
    assert_eq!(call.call_index, 0);
    assert_eq!(call.pallet_name(), "SubtensorModule");
    assert_eq!(call.call_name(), "set_weights");

    // Verify call has parameters
    assert!(!call.parameters.is_empty());
}

#[test]
fn test_decode_add_stake() {
    let tx_bytes = create_add_stake();

    let result = BittensorDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode add_stake: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    let call = tx.call().unwrap();

    assert_eq!(call.pallet_index, 7);
    assert_eq!(call.call_index, 1);
    assert_eq!(call.pallet_name(), "SubtensorModule");
    assert_eq!(call.call_name(), "add_stake");
}

#[test]
fn test_decode_register_neuron() {
    let tx_bytes = create_register_neuron();

    let result = BittensorDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode register: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    let call = tx.call().unwrap();

    assert_eq!(call.pallet_index, 7);
    assert_eq!(call.call_index, 5);
    assert_eq!(call.pallet_name(), "SubtensorModule");
    assert_eq!(call.call_name(), "register");

    // Verify this uses Ed25519 signature (from fixture)
    if let Extrinsic::Signed(signed) = &tx.extrinsic {
        assert!(matches!(signed.signature, BittensorSignature::Ed25519(_)));
    }
}

#[test]
fn test_decode_unsigned_remark() {
    let tx_bytes = create_unsigned_remark();

    let result = BittensorDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode unsigned remark: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    assert!(!tx.extrinsic.is_signed());

    let call = tx.call().unwrap();
    assert_eq!(call.pallet_index, 0);
    assert_eq!(call.pallet_name(), "System");
}

#[test]
fn test_decode_batch_transfer() {
    let tx_bytes = create_batch_transfer();

    let result = BittensorDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode batch transfer: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    let call = tx.call().unwrap();

    assert_eq!(call.pallet_index, 11); // Utility pallet
    assert_eq!(call.pallet_name(), "Utility");
}

#[test]
fn test_decode_large_transfer() {
    let tx_bytes = create_large_transfer();

    let result = BittensorDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Failed to decode large transfer: {:?}",
        result.err()
    );

    let tx = result.unwrap();

    // Verify this uses ECDSA signature (65 bytes)
    if let Extrinsic::Signed(signed) = &tx.extrinsic {
        assert!(matches!(signed.signature, BittensorSignature::Ecdsa(_)));
    }

    let call = tx.call().unwrap();
    assert_eq!(call.pallet_name(), "Balances");
    assert_eq!(call.call_name(), "transfer");
}

#[test]
fn test_canonicalize_tao_transfer() {
    let tx_bytes = create_tao_transfer();
    let tx = BittensorDecoder::decode(&tx_bytes).unwrap();

    let result = tx.canonicalize();
    assert!(result.is_ok(), "Failed to canonicalize: {:?}", result.err());

    let tx_ir = result.unwrap();

    // Verify metadata
    assert_eq!(tx_ir.metadata.size, tx_bytes.len());
    assert_eq!(tx_ir.metadata.tx_hash.len(), 64);
    assert!(tx_ir.metadata.extra.contains("Balances"));
    assert!(tx_ir.metadata.extra.contains("transfer"));

    // Verify operations (should have a transfer)
    assert!(!tx_ir.operations.is_empty());

    // Verify authorization (signed transaction)
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);

    // account_changes was removed from TxIR (docs/CONCEPTS_REVIEW.md C1):
    // effects are not byte-derivable and are no longer fabricated.
    assert!(tx_ir.state_deltas.inputs.is_empty());
}

#[test]
fn test_canonicalize_set_weights() {
    let tx_bytes = create_set_weights();
    let tx = BittensorDecoder::decode(&tx_bytes).unwrap();

    let tx_ir = tx.canonicalize().unwrap();

    // Should have contract call operation for SubtensorModule
    assert!(!tx_ir.operations.is_empty());
    assert!(tx_ir.metadata.extra.contains("SubtensorModule"));

    // Mortal era should be reflected in authorization
    if let Extrinsic::Signed(signed) = &tx.extrinsic {
        assert!(matches!(signed.extension.era, Era::Mortal(_, _)));
    }
}

#[test]
fn test_canonicalize_unsigned() {
    let tx_bytes = create_unsigned_remark();
    let tx = BittensorDecoder::decode(&tx_bytes).unwrap();

    let tx_ir = tx.canonicalize().unwrap();

    // Unsigned transaction should have no signatures
    assert_eq!(tx_ir.authorization.signatures.len(), 0);
    assert_eq!(tx_ir.authorization.public_keys.len(), 0);
}

#[test]
fn test_hash_consistency_across_fixtures() {
    let fixtures = vec![
        ("tao_transfer", create_tao_transfer()),
        ("set_weights", create_set_weights()),
        ("add_stake", create_add_stake()),
        ("register", create_register_neuron()),
        ("batch", create_batch_transfer()),
    ];

    for (name, tx_bytes) in fixtures {
        let tx1 = BittensorDecoder::decode(&tx_bytes).unwrap();
        let tx2 = BittensorDecoder::decode(&tx_bytes).unwrap();

        assert_eq!(
            tx1.tx_hash, tx2.tx_hash,
            "Hash should be deterministic for {}",
            name
        );
        assert_eq!(tx1.tx_hash.len(), 64, "Blake2b-512 should be 64 bytes");
    }
}

#[test]
fn test_signature_types() {
    // Sr25519 (most common)
    let tx_sr25519 = BittensorDecoder::decode(&create_tao_transfer()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_sr25519.extrinsic {
        assert!(matches!(signed.signature, BittensorSignature::Sr25519(_)));
        assert_eq!(signed.signature.clone().into_bytes().len(), 64);
    }

    // Ed25519
    let tx_ed25519 = BittensorDecoder::decode(&create_register_neuron()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_ed25519.extrinsic {
        assert!(matches!(signed.signature, BittensorSignature::Ed25519(_)));
    }

    // ECDSA
    let tx_ecdsa = BittensorDecoder::decode(&create_large_transfer()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_ecdsa.extrinsic {
        assert!(matches!(signed.signature, BittensorSignature::Ecdsa(_)));
        assert_eq!(signed.signature.clone().into_bytes().len(), 65);
    }
}

#[test]
fn test_era_types() {
    // Immortal
    let tx_immortal = BittensorDecoder::decode(&create_tao_transfer()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_immortal.extrinsic {
        assert_eq!(signed.extension.era, Era::Immortal);
    }

    // Mortal
    let tx_mortal = BittensorDecoder::decode(&create_set_weights()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_mortal.extrinsic {
        assert!(matches!(signed.extension.era, Era::Mortal(_, _)));
    }
}

#[test]
fn test_nonce_values() {
    let test_cases = vec![
        ("tao_transfer", create_tao_transfer(), 5),
        ("set_weights", create_set_weights(), 10),
        ("register", create_register_neuron(), 1),
        ("batch", create_batch_transfer(), 2),
    ];

    for (name, tx_bytes, expected_nonce) in test_cases {
        let tx = BittensorDecoder::decode(&tx_bytes).unwrap();
        if let Extrinsic::Signed(signed) = &tx.extrinsic {
            assert_eq!(
                signed.extension.nonce, expected_nonce,
                "Nonce mismatch for {}",
                name
            );
        }
    }
}

#[test]
fn test_tip_values() {
    // Zero tip
    let tx_no_tip = BittensorDecoder::decode(&create_tao_transfer()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_no_tip.extrinsic {
        assert_eq!(signed.extension.tip, 0);
    }

    // With tip
    let tx_with_tip = BittensorDecoder::decode(&create_set_weights()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_with_tip.extrinsic {
        assert_eq!(signed.extension.tip, 100);
    }

    // Large tip
    let tx_large_tip = BittensorDecoder::decode(&create_large_transfer()).unwrap();
    if let Extrinsic::Signed(signed) = &tx_large_tip.extrinsic {
        assert_eq!(signed.extension.tip, 10000);
    }
}

#[test]
fn test_all_bittensor_pallets() {
    let test_cases = vec![
        (create_tao_transfer(), "Balances", "transfer"),
        (create_set_weights(), "SubtensorModule", "set_weights"),
        (create_add_stake(), "SubtensorModule", "add_stake"),
        (create_register_neuron(), "SubtensorModule", "register"),
        (create_unsigned_remark(), "System", "unknown"),
        (create_batch_transfer(), "Utility", "unknown"),
    ];

    for (tx_bytes, expected_pallet, expected_call) in test_cases {
        let tx = BittensorDecoder::decode(&tx_bytes).unwrap();
        let call = tx.call().unwrap();

        assert_eq!(call.pallet_name(), expected_pallet);
        assert_eq!(call.call_name(), expected_call);
    }
}

#[test]
fn test_state_deltas_for_transfers() {
    let tx_bytes = create_tao_transfer();
    let tx = BittensorDecoder::decode(&tx_bytes).unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // account_changes was removed from TxIR (docs/CONCEPTS_REVIEW.md C1):
    // effects are not byte-derivable and are no longer fabricated.
    assert!(tx_ir.state_deltas.inputs.is_empty());
}

#[test]
fn test_validation_passes() {
    let fixtures = vec![
        create_tao_transfer(),
        create_set_weights(),
        create_add_stake(),
        create_register_neuron(),
        create_unsigned_remark(),
        create_batch_transfer(),
        create_large_transfer(),
    ];

    for tx_bytes in fixtures {
        let tx = BittensorDecoder::decode(&tx_bytes).unwrap();
        let result = tx.validate();
        assert!(result.is_ok(), "Validation failed: {:?}", result.err());
    }
}

#[test]
fn test_chain_identity_consistency() {
    let chain = BittensorDecoder::chain();

    assert_eq!(chain.chain_name(), "Bittensor");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
    assert!(chain.chain_id() > 0);

    // Verify consistent across multiple calls
    let chain2 = BittensorDecoder::chain();
    assert_eq!(chain.chain_id(), chain2.chain_id());
}

#[test]
fn test_error_handling() {
    // Empty
    assert!(BittensorDecoder::decode(&[]).is_err());

    // Too short
    assert!(BittensorDecoder::decode(&[0x01]).is_err());
    assert!(BittensorDecoder::decode(&[0x01, 0x84]).is_err());

    // Invalid compact length
    let invalid = vec![0xFF, 0xFF, 0xFF, 0xFF];
    assert!(BittensorDecoder::decode(&invalid).is_err());
}

#[test]
fn test_canonicalize_determinism() {
    let fixtures = vec![
        create_tao_transfer(),
        create_set_weights(),
        create_add_stake(),
    ];

    for tx_bytes in fixtures {
        let tx1 = BittensorDecoder::decode(&tx_bytes).unwrap();
        let tx2 = BittensorDecoder::decode(&tx_bytes).unwrap();

        let ir1 = tx1.canonicalize().unwrap();
        let ir2 = tx2.canonicalize().unwrap();

        assert_eq!(ir1.metadata.tx_hash, ir2.metadata.tx_hash);
        assert_eq!(ir1.metadata.size, ir2.metadata.size);
        assert_eq!(ir1.operations.len(), ir2.operations.len());
        assert_eq!(
            ir1.authorization.signatures.len(),
            ir2.authorization.signatures.len()
        );
    }
}

// Helper trait extension
trait SignatureExt {
    fn into_bytes(self) -> Vec<u8>;
}

impl SignatureExt for BittensorSignature {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            BittensorSignature::Sr25519(b) => b,
            BittensorSignature::Ed25519(b) => b,
            BittensorSignature::Ecdsa(b) => b,
        }
    }
}
