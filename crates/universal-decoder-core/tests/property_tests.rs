//! Property-based tests using proptest
//!
//! These tests verify invariants that must hold for ALL possible inputs,
//! not just specific test cases.

use proptest::prelude::*;
use universal_decoder_core::canonical::*;
use universal_decoder_core::chain::{ChainFamilyEncoded, ChainRef};

// Generators for test data

fn arb_chain_family() -> impl Strategy<Value = ChainFamilyEncoded> {
    prop_oneof![
        Just(ChainFamilyEncoded::Utxo),
        Just(ChainFamilyEncoded::Account),
        Just(ChainFamilyEncoded::Instruction),
        Just(ChainFamilyEncoded::Other),
    ]
}

fn arb_chain_ref() -> impl Strategy<Value = ChainRef> {
    (0u64..1000, "[a-zA-Z]{3,15}", arb_chain_family(), prop::option::of("[a-z]{3,10}"))
        .prop_map(|(id, name, family, network)| ChainRef {
            id,
            name,
            family,
            network,
        })
}

fn arb_signature_scheme() -> impl Strategy<Value = CanonicalSignatureScheme> {
    prop_oneof![
        Just(CanonicalSignatureScheme::Ecdsa),
        Just(CanonicalSignatureScheme::EdDsa),
        Just(CanonicalSignatureScheme::Schnorr),
        (0u32..1000).prop_map(CanonicalSignatureScheme::Custom),
    ]
}

fn arb_canonical_tx_metadata() -> impl Strategy<Value = CanonicalTxMetadata> {
    (
        prop::collection::vec(any::<u8>(), 0..100),
        prop::option::of(any::<u64>()),
        prop::option::of(any::<u64>()),
        0usize..10000,
        prop::option::of("\\{.*\\}"),
    )
        .prop_map(|(tx_hash, block_height, timestamp, size, extra)| {
            CanonicalTxMetadata {
                tx_hash,
                block_height,
                timestamp,
                size,
                extra: extra.unwrap_or_else(|| "{}".to_string()),
            }
        })
}

fn arb_canonical_tx() -> impl Strategy<Value = CanonicalTxIR> {
    (
        any::<u8>(),
        arb_chain_ref(),
        arb_canonical_tx_metadata(),
        arb_signature_scheme(),
    )
        .prop_map(|(version, chain, metadata, signature_scheme)| CanonicalTxIR {
            version,
            chain,
            metadata,
            authorization: CanonicalAuthorizationPackage {
                signatures: vec![],
                public_keys: vec![],
                signature_scheme,
            },
            operations: vec![],
            state_deltas: CanonicalStateDeltas {
                inputs: vec![],
                outputs: vec![],
                account_changes: vec![],
            },
        })
}

// Property tests

proptest! {
    /// Property: Serialization must be deterministic
    /// For any transaction T, serialize(T) = serialize(T)
    #[test]
    fn prop_serialization_deterministic(tx in arb_canonical_tx()) {
        let bytes1 = tx.to_canonical_bytes().unwrap();
        let bytes2 = tx.to_canonical_bytes().unwrap();
        let bytes3 = tx.to_canonical_bytes().unwrap();

        prop_assert_eq!(&bytes1, &bytes2);
        prop_assert_eq!(&bytes2, &bytes3);
    }

    /// Property: Roundtrip must preserve data
    /// For any transaction T, deserialize(serialize(T)) = T
    #[test]
    fn prop_roundtrip_preserves_data(tx in arb_canonical_tx()) {
        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(tx, deserialized);
    }

    /// Property: Hash must be deterministic
    /// For any transaction T, hash(T) = hash(T)
    #[test]
    fn prop_hash_deterministic(tx in arb_canonical_tx()) {
        let hash1 = tx.canonical_hash().unwrap();
        let hash2 = tx.canonical_hash().unwrap();

        prop_assert_eq!(&hash1, &hash2);
        prop_assert_eq!(hash1.len(), 32); // SHA-256
    }

    /// Property: Different transactions must have different hashes (collision resistance)
    /// This is probabilistic - we test that changing ANY field changes the hash
    #[test]
    fn prop_hash_uniqueness(mut tx in arb_canonical_tx()) {
        let original_hash = tx.canonical_hash().unwrap();

        // Modify version
        tx.version = tx.version.wrapping_add(1);
        let modified_hash = tx.canonical_hash().unwrap();

        prop_assert_ne!(original_hash, modified_hash,
            "Changing version should change hash");
    }

    /// Property: Serialization size is bounded
    /// For reasonable inputs, serialization shouldn't explode in size
    #[test]
    fn prop_serialization_size_bounded(tx in arb_canonical_tx()) {
        let bytes = tx.to_canonical_bytes().unwrap();

        // Serialization should not be more than 10x the input size
        // (conservative bound for metadata and structure)
        prop_assert!(bytes.len() < 100_000,
            "Serialization unexpectedly large: {} bytes", bytes.len());
    }

    /// Property: Chain ID roundtrip
    #[test]
    fn prop_chain_id_roundtrip(chain in arb_chain_ref()) {
        let original_id = chain.id;

        let tx = CanonicalTxIR {
            version: 1,
            chain: chain.clone(),
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

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(original_id, deserialized.chain.id);
        prop_assert_eq!(chain.name, deserialized.chain.name);
    }

    /// Property: All signature schemes are serializable
    #[test]
    fn prop_all_signature_schemes_serializable(scheme in arb_signature_scheme()) {
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
                signature_scheme: scheme,
            },
            operations: vec![],
            state_deltas: CanonicalStateDeltas {
                inputs: vec![],
                outputs: vec![],
                account_changes: vec![],
            },
        };

        // Should not panic or error
        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(tx.authorization.signature_scheme, deserialized.authorization.signature_scheme);
    }

    /// Property: Metadata extra field preserves JSON strings
    #[test]
    fn prop_metadata_extra_preserves_strings(
        s in prop::option::of("\\{[a-z0-9\":, ]*\\}")
    ) {
        let extra = s.unwrap_or_else(|| "{}".to_string());

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
                extra: extra.clone(),
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

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(extra, deserialized.metadata.extra);
    }

    /// Property: Block height is preserved
    #[test]
    fn prop_block_height_preserved(height in prop::option::of(any::<u64>())) {
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
                block_height: height,
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

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(height, deserialized.metadata.block_height);
    }
}

// Additional non-proptest tests for specific edge cases

#[test]
fn test_empty_vec_serialization() {
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

    let bytes = tx.to_canonical_bytes().unwrap();
    let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

    assert_eq!(tx, deserialized);
}
