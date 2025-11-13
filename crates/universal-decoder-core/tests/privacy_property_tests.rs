//! Property-based tests for privacy types
//!
//! This module uses proptest to verify properties of privacy primitives:
//! - Serialization roundtrips
//! - Consistency between observability levels and features
//! - Safety properties (no panics, bounded sizes)

use proptest::prelude::*;
use universal_decoder_core::privacy::*;

// Arbitrary generators for privacy types

fn arb_observability_level() -> impl Strategy<Value = ObservabilityLevel> {
    prop_oneof![
        Just(ObservabilityLevel::FullyObservable),
        Just(ObservabilityLevel::PartiallyObservable),
        Just(ObservabilityLevel::FullyPrivate),
    ]
}

fn arb_address_privacy_type() -> impl Strategy<Value = AddressPrivacyType> {
    prop_oneof![
        (0u32..=10000u32).prop_map(|scheme_id| AddressPrivacyType::Stealth { scheme_id }),
        (1usize..=100usize).prop_map(|ring_size| AddressPrivacyType::RingSig { ring_size }),
        ("[a-zA-Z]{1,20}", prop::collection::vec(any::<u8>(), 0..64)).prop_map(
            |(name, metadata)| AddressPrivacyType::Custom {
                mechanism_name: name,
                metadata,
            }
        ),
    ]
}

fn arb_private_address() -> impl Strategy<Value = PrivateAddress> {
    (
        arb_address_privacy_type(),
        prop::collection::vec(any::<u8>(), 1..64),
        prop::option::of(prop::collection::vec(any::<u8>(), 1..64)),
    )
        .prop_map(
            |(privacy_type, public_address, viewing_hint)| PrivateAddress {
                privacy_type,
                public_address,
                viewing_hint,
            },
        )
}

fn arb_range_proof_system() -> impl Strategy<Value = RangeProofSystem> {
    prop_oneof![
        Just(RangeProofSystem::Bulletproofs),
        Just(RangeProofSystem::BulletproofsPlus),
        Just(RangeProofSystem::Borromean),
        (0u32..1000u32).prop_map(RangeProofSystem::Custom),
    ]
}

fn arb_confidential_amount() -> impl Strategy<Value = ConfidentialAmount> {
    (
        prop::collection::vec(any::<u8>(), 32..64),
        prop::option::of(prop::collection::vec(any::<u8>(), 32..512)),
        arb_range_proof_system(),
    )
        .prop_map(
            |(commitment, range_proof, proof_system)| ConfidentialAmount {
                commitment,
                range_proof,
                proof_system,
            },
        )
}

fn arb_compliance_proof() -> impl Strategy<Value = ComplianceProof> {
    (
        prop::collection::vec(any::<u8>(), 32..256),
        prop::collection::vec(any::<u8>(), 4..32),
    )
        .prop_map(
            |(association_set_proof, association_set_id)| ComplianceProof {
                association_set_proof,
                association_set_id,
            },
        )
}

fn arb_privacy_pool() -> impl Strategy<Value = PrivacyPool> {
    (
        prop::collection::vec(any::<u8>(), 32..64),
        prop::collection::vec(any::<u8>(), 32..512),
        prop::option::of(arb_compliance_proof()),
    )
        .prop_map(
            |(anonymity_set_root, membership_proof, compliance_proof)| PrivacyPool {
                anonymity_set_root,
                membership_proof,
                compliance_proof,
            },
        )
}

fn arb_decryption_policy() -> impl Strategy<Value = DecryptionPolicy> {
    prop_oneof![
        Just(DecryptionPolicy::PostInclusion),
        (1u64..=1000000u64).prop_map(DecryptionPolicy::DelayedBy),
        Just(DecryptionPolicy::Never),
    ]
}

fn arb_encrypted_transaction() -> impl Strategy<Value = EncryptedTransaction> {
    (
        prop::collection::vec(any::<u8>(), 0..1024),
        prop::collection::vec(any::<u8>(), 32..256),
        arb_decryption_policy(),
    )
        .prop_map(|(encrypted_payload, validity_proof, decryption_policy)| {
            EncryptedTransaction {
                encrypted_payload,
                validity_proof,
                decryption_policy,
            }
        })
}

fn arb_privacy_feature() -> impl Strategy<Value = PrivacyFeature> {
    prop_oneof![
        arb_private_address().prop_map(PrivacyFeature::HiddenSender),
        arb_private_address().prop_map(PrivacyFeature::HiddenRecipient),
        arb_confidential_amount().prop_map(PrivacyFeature::HiddenAmount),
        arb_privacy_pool().prop_map(PrivacyFeature::HiddenGraph),
        arb_encrypted_transaction().prop_map(PrivacyFeature::HiddenExistence),
        (
            "[a-zA-Z]{1,30}",
            "[a-zA-Z ]{10,100}",
            prop::collection::vec(any::<u8>(), 0..128)
        )
            .prop_map(|(name, description, metadata)| PrivacyFeature::Custom {
                name,
                description,
                metadata,
            }),
    ]
}

fn arb_viewing_key_type() -> impl Strategy<Value = ViewingKeyType> {
    prop_oneof![
        Just(ViewingKeyType::Zcash),
        Just(ViewingKeyType::Monero),
        "[a-zA-Z]{1,20}".prop_map(ViewingKeyType::Custom),
    ]
}

fn arb_viewing_key() -> impl Strategy<Value = ViewingKey> {
    (
        arb_viewing_key_type(),
        prop::collection::vec(any::<u8>(), 32..128),
    )
        .prop_map(|(key_type, key_data)| ViewingKey { key_type, key_data })
}

fn arb_privacy_metadata() -> impl Strategy<Value = PrivacyMetadata> {
    (
        prop::collection::vec(arb_privacy_feature(), 0..5),
        arb_observability_level(),
        prop::option::of(arb_viewing_key()),
    )
        .prop_map(|(features, observability, viewing_key)| PrivacyMetadata {
            features,
            observability,
            viewing_key,
        })
}

// Property tests

proptest! {
    /// Property: PrivacyMetadata serialization roundtrips correctly
    #[test]
    fn prop_privacy_metadata_serialization_roundtrip(metadata in arb_privacy_metadata()) {
        let json = serde_json::to_string(&metadata).expect("Serialization should not fail");
        let deserialized: PrivacyMetadata = serde_json::from_str(&json)
            .expect("Deserialization should not fail");
        prop_assert_eq!(metadata, deserialized);
    }

    /// Property: PrivacyMetadata serialization is deterministic
    #[test]
    fn prop_privacy_metadata_serialization_deterministic(metadata in arb_privacy_metadata()) {
        let json1 = serde_json::to_string(&metadata).expect("Serialization should not fail");
        let json2 = serde_json::to_string(&metadata).expect("Serialization should not fail");
        prop_assert_eq!(json1, json2);
    }

    /// Property: PrivateAddress serialization roundtrips
    #[test]
    fn prop_private_address_roundtrip(addr in arb_private_address()) {
        let json = serde_json::to_string(&addr).expect("Serialization should not fail");
        let deserialized: PrivateAddress = serde_json::from_str(&json)
            .expect("Deserialization should not fail");
        prop_assert_eq!(addr, deserialized);
    }

    /// Property: ConfidentialAmount serialization roundtrips
    #[test]
    fn prop_confidential_amount_roundtrip(amount in arb_confidential_amount()) {
        let json = serde_json::to_string(&amount).expect("Serialization should not fail");
        let deserialized: ConfidentialAmount = serde_json::from_str(&json)
            .expect("Deserialization should not fail");
        prop_assert_eq!(amount, deserialized);
    }

    /// Property: PrivacyPool serialization roundtrips
    #[test]
    fn prop_privacy_pool_roundtrip(pool in arb_privacy_pool()) {
        let json = serde_json::to_string(&pool).expect("Serialization should not fail");
        let deserialized: PrivacyPool = serde_json::from_str(&json)
            .expect("Deserialization should not fail");
        prop_assert_eq!(pool, deserialized);
    }

    /// Property: EncryptedTransaction serialization roundtrips
    #[test]
    fn prop_encrypted_transaction_roundtrip(tx in arb_encrypted_transaction()) {
        let json = serde_json::to_string(&tx).expect("Serialization should not fail");
        let deserialized: EncryptedTransaction = serde_json::from_str(&json)
            .expect("Deserialization should not fail");
        prop_assert_eq!(tx, deserialized);
    }

    /// Property: ViewingKey serialization roundtrips
    #[test]
    fn prop_viewing_key_roundtrip(key in arb_viewing_key()) {
        let json = serde_json::to_string(&key).expect("Serialization should not fail");
        let deserialized: ViewingKey = serde_json::from_str(&json)
            .expect("Deserialization should not fail");
        prop_assert_eq!(key, deserialized);
    }

    /// Property: Observability levels are consistent
    #[test]
    fn prop_observability_level_copy(level in arb_observability_level()) {
        let copied = level;
        prop_assert_eq!(level, copied);
    }

    /// Property: DecryptionPolicy is consistent
    #[test]
    fn prop_decryption_policy_copy(policy in arb_decryption_policy()) {
        let copied = policy;
        prop_assert_eq!(policy, copied);
    }

    /// Property: RangeProofSystem is consistent
    #[test]
    fn prop_range_proof_system_copy(system in arb_range_proof_system()) {
        let copied = system;
        prop_assert_eq!(system, copied);
    }

    /// Property: Stealth address scheme IDs are preserved
    #[test]
    fn prop_stealth_scheme_id_preserved(scheme_id in 0u32..=10000u32) {
        let addr_type = AddressPrivacyType::Stealth { scheme_id };
        match addr_type {
            AddressPrivacyType::Stealth { scheme_id: id } => {
                prop_assert_eq!(scheme_id, id);
            }
            _ => panic!("Expected Stealth variant"),
        }
    }

    /// Property: Ring signature size is preserved and valid
    #[test]
    fn prop_ring_sig_size_preserved(ring_size in 1usize..=100usize) {
        let addr_type = AddressPrivacyType::RingSig { ring_size };
        match addr_type {
            AddressPrivacyType::RingSig { ring_size: size } => {
                prop_assert_eq!(ring_size, size);
                prop_assert!(size > 0, "Ring size must be positive");
            }
            _ => panic!("Expected RingSig variant"),
        }
    }

    /// Property: Multiple privacy features can coexist
    #[test]
    fn prop_multiple_features_allowed(
        features in prop::collection::vec(arb_privacy_feature(), 0..10)
    ) {
        let metadata = PrivacyMetadata {
            features: features.clone(),
            observability: ObservabilityLevel::PartiallyObservable,
            viewing_key: None,
        };

        prop_assert_eq!(metadata.features.len(), features.len());
    }

    /// Property: FullyObservable with empty features is valid
    #[test]
    fn prop_fully_observable_empty_features_valid(
        viewing_key in prop::option::of(arb_viewing_key())
    ) {
        let metadata = PrivacyMetadata {
            features: vec![],
            observability: ObservabilityLevel::FullyObservable,
            viewing_key,
        };

        prop_assert!(metadata.features.is_empty());
        prop_assert_eq!(metadata.observability, ObservabilityLevel::FullyObservable);
    }

    /// Property: Serialized size is bounded
    #[test]
    fn prop_privacy_metadata_size_bounded(metadata in arb_privacy_metadata()) {
        let json = serde_json::to_string(&metadata).expect("Serialization should not fail");
        // With up to 5 features, each with bounded sizes, total should be reasonable
        prop_assert!(json.len() < 50_000, "Serialized size should be bounded");
    }

    /// Property: DecryptionPolicy::DelayedBy preserves block count
    #[test]
    fn prop_delayed_by_preserves_blocks(blocks in 1u64..=1000000u64) {
        let policy = DecryptionPolicy::DelayedBy(blocks);
        match policy {
            DecryptionPolicy::DelayedBy(b) => {
                prop_assert_eq!(blocks, b);
            }
            _ => panic!("Expected DelayedBy variant"),
        }
    }

    /// Property: Custom privacy features preserve metadata
    #[test]
    fn prop_custom_feature_preserves_metadata(
        name in "[a-zA-Z]{1,30}",
        description in "[a-zA-Z ]{10,100}",
        metadata in prop::collection::vec(any::<u8>(), 0..128)
    ) {
        let feature = PrivacyFeature::Custom {
            name: name.clone(),
            description: description.clone(),
            metadata: metadata.clone(),
        };

        match feature {
            PrivacyFeature::Custom { name: n, description: d, metadata: m } => {
                prop_assert_eq!(name, n);
                prop_assert_eq!(description, d);
                prop_assert_eq!(metadata, m);
            }
            _ => panic!("Expected Custom variant"),
        }
    }

    /// Property: Privacy metadata never panics on clone
    #[test]
    fn prop_privacy_metadata_clone_safe(metadata in arb_privacy_metadata()) {
        let cloned = metadata.clone();
        prop_assert_eq!(metadata, cloned);
    }

    /// Property: Privacy metadata equality is reflexive
    #[test]
    fn prop_privacy_metadata_reflexive(metadata in arb_privacy_metadata()) {
        let cloned = metadata.clone();
        prop_assert_eq!(metadata, cloned);
    }

    /// Property: Privacy metadata equality is symmetric
    #[test]
    fn prop_privacy_metadata_symmetric(metadata in arb_privacy_metadata()) {
        let cloned = metadata.clone();
        prop_assert_eq!(metadata == cloned, cloned == metadata);
    }

    /// Property: Viewing key data is preserved
    #[test]
    fn prop_viewing_key_data_preserved(
        key_type in arb_viewing_key_type(),
        key_data in prop::collection::vec(any::<u8>(), 32..128)
    ) {
        let key = ViewingKey {
            key_type: key_type.clone(),
            key_data: key_data.clone(),
        };

        prop_assert_eq!(key.key_type, key_type);
        prop_assert_eq!(key.key_data, key_data);
    }
}

#[cfg(test)]
mod consistency_tests {
    use super::*;

    #[test]
    fn test_observability_level_ordering() {
        // These levels represent a spectrum, but we don't enforce ordering
        // Just ensure they are distinct
        assert_ne!(
            ObservabilityLevel::FullyObservable,
            ObservabilityLevel::PartiallyObservable
        );
        assert_ne!(
            ObservabilityLevel::PartiallyObservable,
            ObservabilityLevel::FullyPrivate
        );
        assert_ne!(
            ObservabilityLevel::FullyObservable,
            ObservabilityLevel::FullyPrivate
        );
    }

    #[test]
    fn test_all_privacy_features_serializable() {
        let features = vec![
            PrivacyFeature::HiddenSender(PrivateAddress {
                privacy_type: AddressPrivacyType::RingSig { ring_size: 11 },
                public_address: vec![1, 2, 3],
                viewing_hint: None,
            }),
            PrivacyFeature::HiddenRecipient(PrivateAddress {
                privacy_type: AddressPrivacyType::Stealth { scheme_id: 5564 },
                public_address: vec![4, 5, 6],
                viewing_hint: Some(vec![7, 8]),
            }),
            PrivacyFeature::HiddenAmount(ConfidentialAmount {
                commitment: vec![9, 10, 11],
                range_proof: Some(vec![12, 13, 14]),
                proof_system: RangeProofSystem::Bulletproofs,
            }),
            PrivacyFeature::HiddenGraph(PrivacyPool {
                anonymity_set_root: vec![15, 16, 17],
                membership_proof: vec![18, 19, 20],
                compliance_proof: None,
            }),
            PrivacyFeature::HiddenExistence(EncryptedTransaction {
                encrypted_payload: vec![21, 22, 23],
                validity_proof: vec![24, 25, 26],
                decryption_policy: DecryptionPolicy::PostInclusion,
            }),
            PrivacyFeature::Custom {
                name: "Test".to_string(),
                description: "Test feature".to_string(),
                metadata: vec![27, 28, 29],
            },
        ];

        let metadata = PrivacyMetadata {
            features,
            observability: ObservabilityLevel::FullyPrivate,
            viewing_key: None,
        };

        let json = serde_json::to_string(&metadata).expect("Should serialize");
        let deserialized: PrivacyMetadata =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(metadata, deserialized);
    }
}
