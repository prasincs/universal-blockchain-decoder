//! Comprehensive tests for canonical serialization
//!
//! These tests verify that:
//! 1. Serialization is deterministic
//! 2. Deserialization is the inverse of serialization
//! 3. Hash computation is stable
//! 4. Edge cases are handled correctly

use universal_decoder_core::canonical::*;
use universal_decoder_core::chain::{ChainFamily, ChainFamilyEncoded, ChainRef};
use universal_decoder_core::prelude::*;

fn create_test_canonical_tx() -> CanonicalTxIR {
    CanonicalTxIR {
        version: 1,
        chain: ChainRef {
            id: 0,
            name: "Bitcoin".to_string(),
            family: ChainFamilyEncoded::Utxo,
            network: Some("mainnet".to_string()),
        },
        metadata: CanonicalTxMetadata {
            tx_hash: vec![0xde, 0xad, 0xbe, 0xef],
            block_height: Some(800000),
            timestamp: Some(1699999999),
            size: 250,
            extra: "{}".to_string(),
        },
        authorization: CanonicalAuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: CanonicalSignatureScheme::Ecdsa,
        },
        operations: vec![],
        state_deltas: CanonicalStateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        },
    }
}

#[test]
fn test_serialization_is_deterministic() {
    let tx = create_test_canonical_tx();

    let bytes1 = tx.to_canonical_bytes().unwrap();
    let bytes2 = tx.to_canonical_bytes().unwrap();
    let bytes3 = tx.to_canonical_bytes().unwrap();

    assert_eq!(bytes1, bytes2, "Serialization must be deterministic");
    assert_eq!(bytes2, bytes3, "Serialization must be deterministic");
}

#[test]
fn test_roundtrip_preserves_data() {
    let original = create_test_canonical_tx();

    let bytes = original.to_canonical_bytes().unwrap();
    let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

    assert_eq!(original, deserialized, "Roundtrip must preserve all data");
}

#[test]
fn test_hash_is_deterministic() {
    let tx = create_test_canonical_tx();

    let hash1 = tx.canonical_hash().unwrap();
    let hash2 = tx.canonical_hash().unwrap();
    let hash3 = tx.canonical_hash().unwrap();

    assert_eq!(hash1, hash2, "Hash must be deterministic");
    assert_eq!(hash2, hash3, "Hash must be deterministic");
    assert_eq!(hash1.len(), 32, "SHA-256 hash should be 32 bytes");
}

#[test]
fn test_different_transactions_have_different_hashes() {
    let tx1 = create_test_canonical_tx();

    let mut tx2 = create_test_canonical_tx();
    tx2.metadata.block_height = Some(800001);

    let hash1 = tx1.canonical_hash().unwrap();
    let hash2 = tx2.canonical_hash().unwrap();

    assert_ne!(
        hash1, hash2,
        "Different transactions must have different hashes"
    );
}

#[test]
fn test_empty_transaction() {
    let tx = CanonicalTxIR {
        version: 1,
        chain: ChainRef {
            id: 0,
            name: "Test".to_string(),
            family: ChainFamilyEncoded::Utxo,
            network: None,
        },
        metadata: CanonicalTxMetadata {
            tx_hash: vec![],
            block_height: None,
            timestamp: None,
            size: 0,
            extra: "{}".to_string(),
        },
        authorization: CanonicalAuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: CanonicalSignatureScheme::Ecdsa,
        },
        operations: vec![],
        state_deltas: CanonicalStateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        },
    };

    // Empty transaction should still serialize/deserialize correctly
    let bytes = tx.to_canonical_bytes().unwrap();
    let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(tx, deserialized);
}

#[test]
fn test_large_transaction() {
    let mut tx = create_test_canonical_tx();

    // Add many operations
    for i in 0..100 {
        tx.operations
            .push(CanonicalOperation::Generic(CanonicalGenericOperation {
                op_type: format!("op_{}", i),
                data: vec![i as u8; 100],
                metadata: "{}".to_string(),
            }));
    }

    // Should handle large transactions
    let bytes = tx.to_canonical_bytes().unwrap();
    assert!(
        bytes.len() > 10000,
        "Large transaction should produce large serialization"
    );

    let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(tx.operations.len(), deserialized.operations.len());
}

#[test]
fn test_all_signature_schemes() {
    let schemes = vec![
        CanonicalSignatureScheme::Ecdsa,
        CanonicalSignatureScheme::EdDsa,
        CanonicalSignatureScheme::Schnorr,
        CanonicalSignatureScheme::Custom(42),
    ];

    for scheme in schemes {
        let mut tx = create_test_canonical_tx();
        tx.authorization.signature_scheme = scheme;

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        assert_eq!(
            tx.authorization.signature_scheme,
            deserialized.authorization.signature_scheme
        );
    }
}

#[test]
fn test_all_operation_types() {
    let operations = vec![
        CanonicalOperation::Transfer(CanonicalTransfer {
            from: CanonicalAddress {
                bytes: vec![1, 2, 3],
                human_readable: Some("addr1".to_string()),
            },
            to: CanonicalAddress {
                bytes: vec![4, 5, 6],
                human_readable: Some("addr2".to_string()),
            },
            amount: CanonicalAmount {
                value: 1000,
                decimals: 8,
            },
            asset: CanonicalAssetId::Native,
        }),
        CanonicalOperation::ContractCall(CanonicalContractCall {
            contract: CanonicalAddress {
                bytes: vec![7, 8, 9],
                human_readable: None,
            },
            method: b"transfer".to_vec(),
            data: vec![0xde, 0xad],
            value: None,
            resource_limits: CanonicalResourceLimits {
                max_units: 21000,
                unit_price: 20,
                resource_type: CanonicalResourceType::Gas,
            },
        }),
        CanonicalOperation::Stake(CanonicalStake {
            validator: CanonicalAddress {
                bytes: vec![10, 11, 12],
                human_readable: Some("validator1".to_string()),
            },
            amount: CanonicalAmount {
                value: 32000000000,
                decimals: 9,
            },
            operation_type: CanonicalStakeOperationType::Delegate,
        }),
    ];

    for op in operations {
        let mut tx = create_test_canonical_tx();
        tx.operations = vec![op.clone()];

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        assert_eq!(tx.operations.len(), 1);
        assert_eq!(deserialized.operations.len(), 1);
    }
}

#[test]
fn test_chain_families() {
    let families = vec![
        ChainFamilyEncoded::Utxo,
        ChainFamilyEncoded::Account,
        ChainFamilyEncoded::Instruction,
        ChainFamilyEncoded::Other,
    ];

    for family in families {
        let mut tx = create_test_canonical_tx();
        tx.chain.family = family;

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        assert_eq!(tx.chain.family, deserialized.chain.family);
    }
}

#[test]
fn test_invalid_deserialization() {
    let invalid_bytes = vec![0xff; 100];
    let result = CanonicalTxIR::from_canonical_bytes(&invalid_bytes);
    assert!(result.is_err(), "Invalid bytes should fail deserialization");
}

#[test]
fn test_empty_bytes_deserialization() {
    let empty_bytes: &[u8] = &[];
    let result = CanonicalTxIR::from_canonical_bytes(empty_bytes);
    assert!(result.is_err(), "Empty bytes should fail deserialization");
}
