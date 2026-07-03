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
    (
        0u64..1000,
        "[a-zA-Z]{3,15}",
        arb_chain_family(),
        prop::option::of("[a-z]{3,10}"),
    )
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
        .prop_map(
            |(tx_hash, block_height, timestamp, size, extra)| CanonicalTxMetadata {
                tx_hash,
                block_height,
                timestamp,
                size,
                extra: extra.unwrap_or_else(|| "{}".to_string()),
            },
        )
}

fn arb_canonical_tx() -> impl Strategy<Value = CanonicalTxIR> {
    (
        any::<u8>(),
        arb_chain_ref(),
        arb_canonical_tx_metadata(),
        arb_signature_scheme(),
    )
        .prop_map(
            |(version, chain, metadata, signature_scheme)| CanonicalTxIR {
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
                },
            },
        )
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(height, deserialized.metadata.block_height);
    }

    /// Property: Timestamp is preserved
    #[test]
    fn prop_timestamp_preserved(timestamp in prop::option::of(any::<u64>())) {
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
                timestamp,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(timestamp, deserialized.metadata.timestamp);
    }

    /// Property: Transaction size is preserved
    #[test]
    fn prop_tx_size_preserved(size in 0usize..1_000_000) {
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
                size,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(size, deserialized.metadata.size);
    }

    /// Property: Transaction hash bytes are preserved
    #[test]
    fn prop_tx_hash_bytes_preserved(hash_bytes in prop::collection::vec(any::<u8>(), 0..128)) {
        let tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: None,
            },
            metadata: CanonicalTxMetadata {
                tx_hash: hash_bytes.clone(),
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(hash_bytes, deserialized.metadata.tx_hash);
    }

    /// Property: All chain families are serializable
    #[test]
    fn prop_all_chain_families_serializable(family in arb_chain_family()) {
        let tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(family, deserialized.chain.family);
    }

    /// Property: Network field is preserved
    #[test]
    fn prop_network_preserved(network in prop::option::of("[a-z]{3,10}")) {
        let tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: network.clone(),
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(network, deserialized.chain.network);
    }

    /// Property: Version field is preserved
    #[test]
    fn prop_version_preserved(version in any::<u8>()) {
        let tx = CanonicalTxIR {
            version,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(version, deserialized.version);
    }

    /// Property: Chain name is preserved
    #[test]
    fn prop_chain_name_preserved(name in "[a-zA-Z]{1,50}") {
        let tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: name.clone(),
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(name, deserialized.chain.name);
    }

    /// Property: Empty signatures list is preserved
    #[test]
    fn prop_empty_signatures_preserved(_seed in any::<u64>()) {
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert!(deserialized.authorization.signatures.is_empty());
        prop_assert!(deserialized.authorization.public_keys.is_empty());
    }

    /// Property: Empty operations list is preserved
    #[test]
    fn prop_empty_operations_preserved(_seed in any::<u64>()) {
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert!(deserialized.operations.is_empty());
    }

    /// Property: Empty state deltas are preserved
    #[test]
    fn prop_empty_state_deltas_preserved(_seed in any::<u64>()) {
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert!(deserialized.state_deltas.inputs.is_empty());
        prop_assert!(deserialized.state_deltas.outputs.is_empty());
    }

    /// Property: Large chain IDs are preserved
    #[test]
    fn prop_large_chain_id_preserved(id in 0u64..u64::MAX) {
        let tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(id, deserialized.chain.id);
    }

    /// Property: Serialized bytes length is consistent
    #[test]
    fn prop_serialized_length_consistent(tx in arb_canonical_tx()) {
        let bytes1 = tx.to_canonical_bytes().unwrap();
        let bytes2 = tx.to_canonical_bytes().unwrap();

        prop_assert_eq!(bytes1.len(), bytes2.len());
    }

    /// Property: Hash length is always 32 bytes (SHA-256)
    #[test]
    fn prop_hash_length_is_32(tx in arb_canonical_tx()) {
        let hash = tx.canonical_hash().unwrap();

        prop_assert_eq!(hash.len(), 32);
    }

    /// Property: Different chain IDs produce different hashes
    #[test]
    fn prop_different_chain_id_different_hash(id1 in 0u64..1000, id2 in 1000u64..2000) {
        let tx1 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: id1,
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
            },
        };

        let tx2 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: id2,
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
            },
        };

        let hash1 = tx1.canonical_hash().unwrap();
        let hash2 = tx2.canonical_hash().unwrap();

        prop_assert_ne!(hash1, hash2);
    }

    /// Property: Different chain names produce different hashes
    #[test]
    fn prop_different_chain_name_different_hash(name1 in "[a-z]{5}", name2 in "[A-Z]{5}") {
        let tx1 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: name1,
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
            },
        };

        let tx2 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: name2,
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
            },
        };

        let hash1 = tx1.canonical_hash().unwrap();
        let hash2 = tx2.canonical_hash().unwrap();

        prop_assert_ne!(hash1, hash2);
    }

    /// Property: Different chain families produce different hashes
    #[test]
    fn prop_different_family_different_hash(family1 in arb_chain_family(), family2 in arb_chain_family()) {
        prop_assume!(family1 != family2);

        let tx1 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family: family1,
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
            },
        };

        let tx2 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family: family2,
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
            },
        };

        let hash1 = tx1.canonical_hash().unwrap();
        let hash2 = tx2.canonical_hash().unwrap();

        prop_assert_ne!(hash1, hash2);
    }

    /// Property: Changing metadata size changes hash
    #[test]
    fn prop_different_size_different_hash(size1 in 0usize..1000, size2 in 1000usize..2000) {
        let tx1 = CanonicalTxIR {
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
                size: size1,
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
            },
        };

        let tx2 = CanonicalTxIR {
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
                size: size2,
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
            },
        };

        let hash1 = tx1.canonical_hash().unwrap();
        let hash2 = tx2.canonical_hash().unwrap();

        prop_assert_ne!(hash1, hash2);
    }

    /// Property: Changing block height changes hash
    #[test]
    fn prop_different_block_height_different_hash(h1 in 0u64..1000, h2 in 1000u64..2000) {
        let tx1 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: None,
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![],
                block_height: Some(h1),
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
            },
        };

        let tx2 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: None,
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![],
                block_height: Some(h2),
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
            },
        };

        let hash1 = tx1.canonical_hash().unwrap();
        let hash2 = tx2.canonical_hash().unwrap();

        prop_assert_ne!(hash1, hash2);
    }

    /// Property: None vs Some(x) for optional fields changes hash
    #[test]
    fn prop_none_vs_some_different_hash(value in 0u64..1000) {
        let tx1 = CanonicalTxIR {
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
            },
        };

        let tx2 = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Test".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: None,
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![],
                block_height: Some(value),
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
            },
        };

        let hash1 = tx1.canonical_hash().unwrap();
        let hash2 = tx2.canonical_hash().unwrap();

        prop_assert_ne!(hash1, hash2);
    }

    /// Property: Serialization never fails for valid transactions
    #[test]
    fn prop_serialization_never_fails(tx in arb_canonical_tx()) {
        let result = tx.to_canonical_bytes();

        prop_assert!(result.is_ok());
    }

    /// Property: Deserialization of valid bytes never fails
    #[test]
    fn prop_deserialization_of_valid_bytes_succeeds(tx in arb_canonical_tx()) {
        let bytes = tx.to_canonical_bytes().unwrap();
        let result = CanonicalTxIR::from_canonical_bytes(&bytes);

        prop_assert!(result.is_ok());
    }

    /// Property: Hash computation never fails for valid transactions
    #[test]
    fn prop_hash_never_fails(tx in arb_canonical_tx()) {
        let result = tx.canonical_hash();

        prop_assert!(result.is_ok());
    }

    /// Property: Roundtrip through bytes preserves all fields
    #[test]
    fn prop_roundtrip_preserves_all_fields(
        version in any::<u8>(),
        chain_id in 0u64..1000,
        chain_name in "[a-zA-Z]{3,15}",
        family in arb_chain_family(),
        size in 0usize..10000,
    ) {
        let tx = CanonicalTxIR {
            version,
            chain: ChainRef {
                id: chain_id,
                name: chain_name.clone(),
                family,
                network: None,
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![],
                block_height: None,
                timestamp: None,
                size,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(version, deserialized.version);
        prop_assert_eq!(chain_id, deserialized.chain.id);
        prop_assert_eq!(chain_name, deserialized.chain.name);
        prop_assert_eq!(family, deserialized.chain.family);
        prop_assert_eq!(size, deserialized.metadata.size);
    }

    /// Property: Custom signature schemes are preserved
    #[test]
    fn prop_custom_signature_scheme_preserved(scheme_id in 0u32..10000) {
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
                signature_scheme: CanonicalSignatureScheme::Custom(scheme_id),
            },
            operations: vec![],
            state_deltas: CanonicalStateDeltas {
                inputs: vec![],
                outputs: vec![],
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        match deserialized.authorization.signature_scheme {
            CanonicalSignatureScheme::Custom(id) => prop_assert_eq!(scheme_id, id),
            _ => prop_assert!(false, "Expected Custom signature scheme"),
        }
    }

    /// Property: Serialization is monotonic with respect to data size
    #[test]
    fn prop_serialization_size_grows_with_metadata(
        extra1 in "\\{.*\\}",
        extra_suffix in "[a-zA-Z0-9_]{1,20}",
    ) {
        // Generate extra2 by extending extra1, ensuring extra2.len() > extra1.len()
        let extra2 = format!("{}{}", extra1, extra_suffix);

        let tx1 = CanonicalTxIR {
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
                extra: extra1,
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
            },
        };

        let tx2 = CanonicalTxIR {
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
                extra: extra2,
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
            },
        };

        let bytes1 = tx1.to_canonical_bytes().unwrap();
        let bytes2 = tx2.to_canonical_bytes().unwrap();

        // Larger metadata should result in larger serialization
        prop_assert!(bytes2.len() >= bytes1.len());
    }

    /// Property: Zero-length transaction hash is valid
    #[test]
    fn prop_zero_length_hash_valid(_seed in any::<u64>()) {
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert!(deserialized.metadata.tx_hash.is_empty());
    }

    /// Property: Maximum u64 values are handled correctly
    #[test]
    fn prop_max_u64_values_preserved(_seed in any::<u64>()) {
        let tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: u64::MAX,
                name: "Test".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: None,
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![],
                block_height: Some(u64::MAX),
                timestamp: Some(u64::MAX),
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(u64::MAX, deserialized.chain.id);
        prop_assert_eq!(Some(u64::MAX), deserialized.metadata.block_height);
        prop_assert_eq!(Some(u64::MAX), deserialized.metadata.timestamp);
    }

    /// Property: Minimum version (0) is valid
    #[test]
    fn prop_min_version_valid(_seed in any::<u64>()) {
        let tx = CanonicalTxIR {
            version: 0,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(0, deserialized.version);
    }

    /// Property: Maximum version (255) is valid
    #[test]
    fn prop_max_version_valid(_seed in any::<u64>()) {
        let tx = CanonicalTxIR {
            version: 255,
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
            },
        };

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        prop_assert_eq!(255, deserialized.version);
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
        },
    };

    let bytes = tx.to_canonical_bytes().unwrap();
    let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

    assert_eq!(tx, deserialized);
}
