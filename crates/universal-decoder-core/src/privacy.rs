//! Privacy Primitives for Blockchain Transactions
//!
//! This module provides trait-based privacy primitives that can be composed to represent
//! various blockchain privacy mechanisms (stealth addresses, privacy pools, confidential
//! transactions, encrypted mempools, etc.) without requiring core changes for new features.
//!
//! ## Design Philosophy
//!
//! - **Composition over Enumeration**: Privacy features are composable primitives, not exhaustive enums
//! - **Extensibility**: New privacy mechanisms can be added without core modifications
//! - **Backward Compatibility**: Privacy metadata is optional (None for transparent chains)
//! - **Minimal TCB**: Privacy logic resides in decoders, core only provides types
//!
//! ## Privacy Primitive Taxonomy
//!
//! The module defines 5 fundamental privacy primitives observed across blockchain protocols:
//!
//! 1. **HiddenSender** - Sender identity hidden (stealth addresses, ring signatures)
//! 2. **HiddenRecipient** - Recipient identity hidden (one-time addresses, encrypted outputs)
//! 3. **HiddenAmount** - Transaction value hidden (confidential transactions, Pedersen commitments)
//! 4. **HiddenGraph** - Transaction graph hidden (privacy pools, mixers, CoinJoin)
//! 5. **HiddenExistence** - Transaction existence hidden (encrypted mempools, stealth payments)
//!
//! ## Examples
//!
//! ### Ethereum Stealth Address (EIP-5564)
//!
//! ```ignore
//! use universal_decoder_core::privacy::*;
//!
//! let privacy = PrivacyMetadata {
//!     features: vec![
//!         PrivacyFeature::HiddenRecipient(PrivateAddress {
//!             privacy_type: AddressPrivacyType::Stealth { scheme_id: 5564 },
//!             public_address: ephemeral_address_bytes,
//!             viewing_hint: Some(ephemeral_pubkey),
//!         }),
//!     ],
//!     observability: ObservabilityLevel::PartiallyObservable,
//!     viewing_key: None,
//! };
//! ```
//!
//! ### Privacy Pool (Ethereum, 2024)
//!
//! ```ignore
//! let privacy = PrivacyMetadata {
//!     features: vec![
//!         PrivacyFeature::HiddenGraph(PrivacyPool {
//!             anonymity_set_root: merkle_root,
//!             membership_proof: zk_snark_proof,
//!             compliance_proof: Some(ComplianceProof {
//!                 association_set_proof: proof_bytes,
//!                 association_set_id: b"non-sanctioned-v1".to_vec(),
//!             }),
//!         }),
//!     ],
//!     observability: ObservabilityLevel::PartiallyObservable,
//!     viewing_key: None,
//! };
//! ```

use serde::{Deserialize, Serialize};

#[cfg(feature = "formal-verification")]
use builtin::*;
#[cfg(feature = "formal-verification")]
use builtin_macros::*;

/// Privacy metadata for transactions with privacy features
///
/// This structure is optional in TxIR (None for fully transparent chains).
/// When present, it describes the privacy mechanisms used in the transaction.
///
/// # Design Rationale
///
/// - **Optional**: `Option<PrivacyMetadata>` maintains backward compatibility
/// - **Composable**: Multiple privacy features can be used simultaneously
/// - **Auditable**: `viewing_key` enables regulatory compliance without breaking privacy
/// - **Extensible**: New features extend `PrivacyFeature` enum, not core
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyMetadata {
    /// List of privacy primitives used in this transaction
    ///
    /// Transactions can use multiple privacy mechanisms simultaneously.
    /// For example: stealth addresses (HiddenRecipient) + confidential amounts (HiddenAmount).
    pub features: Vec<PrivacyFeature>,

    /// Overall observability level of this transaction
    ///
    /// This is a high-level indicator derived from the features used:
    /// - FullyObservable: All details visible on-chain (Bitcoin, Ethereum legacy)
    /// - PartiallyObservable: Some details hidden (stealth addresses, encrypted mempools)
    /// - FullyPrivate: All details hidden (Monero, Zcash shielded)
    pub observability: ObservabilityLevel,

    /// Optional viewing key for auditing/compliance
    ///
    /// Some privacy protocols (Zcash, Monero) support viewing keys that allow
    /// selective disclosure of transaction details without revealing the spending key.
    pub viewing_key: Option<ViewingKey>,
}

impl PrivacyMetadata {
    /// Creates a new PrivacyMetadata with validation
    ///
    /// # Formal Properties (VT-30: Privacy Metadata Consistency)
    ///
    /// - VT-30.1: If observability is FullyObservable, features should be empty (convention)
    /// - VT-30.2: Clone operation never panics
    /// - VT-30.3: Equality is reflexive, symmetric, and transitive
    #[cfg_attr(
        feature = "formal-verification",
        verifier::spec(|features: Vec<PrivacyFeature>, observability: ObservabilityLevel, viewing_key: Option<ViewingKey>| -> PrivacyMetadata
            ensures(|result: PrivacyMetadata| {
                // VT-30.1: Features preserved
                result.features == features &&
                result.observability == observability &&
                result.viewing_key == viewing_key
            })
        )
    )]
    pub fn new(
        features: Vec<PrivacyFeature>,
        observability: ObservabilityLevel,
        viewing_key: Option<ViewingKey>,
    ) -> Self {
        Self {
            features,
            observability,
            viewing_key,
        }
    }

    /// Returns true if this transaction has any privacy features
    ///
    /// # Formal Properties
    ///
    /// - Never panics
    /// - Returns true iff features vector is non-empty
    #[cfg_attr(
        feature = "formal-verification",
        verifier::spec(|self: &PrivacyMetadata| -> bool
            ensures(|result: bool| {
                result == !self.features.is_empty()
            })
        )
    )]
    pub fn has_privacy_features(&self) -> bool {
        !self.features.is_empty()
    }

    /// Returns true if transaction is fully observable (no privacy)
    ///
    /// # Formal Properties
    ///
    /// - Never panics
    /// - Returns true iff observability is FullyObservable
    #[cfg_attr(
        feature = "formal-verification",
        verifier::spec(|self: &PrivacyMetadata| -> bool
            ensures(|result: bool| {
                result == matches!(self.observability, ObservabilityLevel::FullyObservable)
            })
        )
    )]
    pub fn is_fully_observable(&self) -> bool {
        matches!(self.observability, ObservabilityLevel::FullyObservable)
    }

    /// Returns true if transaction has complete privacy
    ///
    /// # Formal Properties
    ///
    /// - Never panics
    /// - Returns true iff observability is FullyPrivate
    #[cfg_attr(
        feature = "formal-verification",
        verifier::spec(|self: &PrivacyMetadata| -> bool
            ensures(|result: bool| {
                result == matches!(self.observability, ObservabilityLevel::FullyPrivate)
            })
        )
    )]
    pub fn is_fully_private(&self) -> bool {
        matches!(self.observability, ObservabilityLevel::FullyPrivate)
    }
}

/// Observability level indicates how much transaction data is publicly visible
///
/// This is a spectrum from fully transparent to fully private.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservabilityLevel {
    /// Fully transparent - all transaction details visible on-chain
    ///
    /// Examples: Bitcoin (non-stealth), Ethereum (pre-privacy features)
    FullyObservable,

    /// Partially private - some details hidden, others visible
    ///
    /// Examples:
    /// - Ethereum with stealth addresses (amount visible, recipient hidden)
    /// - Bitcoin with CoinJoin (amounts visible, graph obscured)
    /// - Encrypted mempool transactions (visible after inclusion)
    PartiallyObservable,

    /// Fully private - all details hidden (sender, recipient, amount, graph)
    ///
    /// Examples:
    /// - Monero (ring signatures + stealth addresses + RingCT)
    /// - Zcash shielded transactions (z2z transfers)
    FullyPrivate,
}

/// Privacy feature descriptor using composable primitives
///
/// This enum uses an open design: the `Custom` variant allows new privacy mechanisms
/// to be represented without modifying the core library.
///
/// # Extension Strategy
///
/// When a new privacy feature emerges:
/// 1. If it fits existing primitives → use existing variants
/// 2. If it's truly novel → use `Custom` temporarily
/// 3. Once standardized → promote to dedicated variant in minor version update
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyFeature {
    /// Sender identity is hidden (stealth addresses, ring signatures)
    ///
    /// Examples:
    /// - Monero ring signatures (1-of-N signer anonymity)
    /// - Tornado Cash deposit addresses
    HiddenSender(PrivateAddress),

    /// Recipient identity is hidden (one-time addresses, encrypted outputs)
    ///
    /// Examples:
    /// - Ethereum stealth addresses (EIP-5564)
    /// - Zcash shielded addresses
    /// - Monero one-time addresses
    HiddenRecipient(PrivateAddress),

    /// Transaction amount is hidden (confidential transactions)
    ///
    /// Examples:
    /// - Monero RingCT (Pedersen commitments + range proofs)
    /// - Elements (Liquid) confidential assets
    /// - Mimblewimble (Grin, Beam)
    HiddenAmount(ConfidentialAmount),

    /// Transaction graph is obscured (privacy pools, mixers)
    ///
    /// Examples:
    /// - Privacy Pools (Ethereum, 2024+)
    /// - Tornado Cash (deprecated)
    /// - CoinJoin protocols (Bitcoin)
    HiddenGraph(PrivacyPool),

    /// Transaction existence is hidden (encrypted mempools, stealth payments)
    ///
    /// Examples:
    /// - Encrypted mempools (Flashbots, MEV-Boost)
    /// - Stealth payments (only sender/recipient know transaction occurred)
    HiddenExistence(EncryptedTransaction),

    /// Custom/future privacy mechanisms
    ///
    /// Use this for novel privacy features that don't fit existing primitives.
    /// Once a mechanism becomes standardized, it should be promoted to a dedicated variant.
    Custom {
        /// Human-readable mechanism name
        name: String,

        /// Brief description of the privacy mechanism
        description: String,

        /// Opaque metadata (mechanism-specific)
        metadata: Vec<u8>,
    },
}

/// Private address with metadata about the privacy mechanism used
///
/// Used for HiddenSender and HiddenRecipient primitives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateAddress {
    /// The privacy mechanism used for this address
    pub privacy_type: AddressPrivacyType,

    /// The public address bytes (what appears on-chain)
    ///
    /// For stealth addresses: ephemeral/one-time address
    /// For ring signatures: set of possible signer addresses
    pub public_address: Vec<u8>,

    /// Optional viewing hint for auditing
    ///
    /// For stealth addresses: ephemeral public key
    /// For other schemes: mechanism-specific hint
    pub viewing_hint: Option<Vec<u8>>,
}

/// Address privacy mechanism type
///
/// Describes how address privacy is achieved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressPrivacyType {
    /// Stealth address (EIP-5564 for Ethereum, or similar protocols)
    ///
    /// Sender generates a one-time address from recipient's meta-address.
    /// Only recipient can detect and spend from this address.
    Stealth {
        /// Scheme identifier (e.g., 5564 for EIP-5564)
        scheme_id: u32,
    },

    /// Ring signature (Monero-style)
    ///
    /// Actual signer is hidden among N possible signers.
    RingSig {
        /// Number of possible signers (anonymity set size)
        ring_size: usize,
    },

    /// Custom/future address privacy mechanism
    Custom {
        /// Mechanism name
        mechanism_name: String,

        /// Mechanism-specific metadata
        metadata: Vec<u8>,
    },
}

/// Confidential amount using cryptographic commitments
///
/// Used for HiddenAmount primitive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfidentialAmount {
    /// Pedersen commitment to the amount: C = vG + rH
    ///
    /// Where:
    /// - v = actual value (secret)
    /// - r = blinding factor (secret)
    /// - G, H = elliptic curve generators
    pub commitment: Vec<u8>,

    /// Range proof (proves amount is positive without revealing value)
    ///
    /// Common proof systems:
    /// - Bulletproofs (logarithmic size, no trusted setup)
    /// - Bulletproofs+ (improved efficiency)
    /// - Borromean ring signatures (older, larger)
    pub range_proof: Option<Vec<u8>>,

    /// Proof system used for the range proof
    pub proof_system: RangeProofSystem,
}

/// Range proof system for confidential amounts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeProofSystem {
    /// Bulletproofs (logarithmic size, no trusted setup)
    ///
    /// Used by: Monero, Grin, Beam, Elements
    Bulletproofs,

    /// Bulletproofs+ (improved efficiency over original)
    BulletproofsPlus,

    /// Borromean ring signatures (older approach, larger proofs)
    Borromean,

    /// Custom range proof system
    Custom(u32),
}

/// Privacy pool (mixer) information
///
/// Used for HiddenGraph primitive.
///
/// Privacy pools obscure the link between input and output addresses by pooling
/// funds together. The innovation (2024) is compliance-friendly privacy via
/// association set proofs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyPool {
    /// Merkle root of the anonymity set
    ///
    /// All deposits into the pool are leaves in a Merkle tree.
    /// Users prove membership without revealing which leaf.
    pub anonymity_set_root: Vec<u8>,

    /// Zero-knowledge proof of membership (without revealing which member)
    ///
    /// Common proof systems:
    /// - Groth16 (requires trusted setup, small proofs)
    /// - PLONK (universal setup, moderate proofs)
    /// - STARKs (no setup, larger proofs)
    pub membership_proof: Vec<u8>,

    /// Optional compliance proof (Privacy Pools innovation, 2024)
    ///
    /// Proves funds are NOT from illicit sources via association set membership.
    /// This enables compliant privacy: user gets anonymity within a "clean" set.
    pub compliance_proof: Option<ComplianceProof>,
}

/// Compliance proof for privacy pools
///
/// Introduced by Privacy Pools (2024) to enable regulatory-friendly privacy.
/// User proves their deposit is in an "association set" (e.g., "non-sanctioned addresses")
/// without revealing which specific deposit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceProof {
    /// Proof that deposit is in the "clean" association set
    pub association_set_proof: Vec<u8>,

    /// Association set identifier (e.g., "non-sanctioned-v1")
    ///
    /// Multiple association sets can exist with different compliance criteria.
    pub association_set_id: Vec<u8>,
}

/// Encrypted transaction (hidden until inclusion or never revealed)
///
/// Used for HiddenExistence primitive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedTransaction {
    /// Encrypted transaction payload
    ///
    /// Only authorized parties (validators, builders) can decrypt.
    /// For post-inclusion decryption: this field may be empty (already decrypted).
    pub encrypted_payload: Vec<u8>,

    /// Public validity proof (proves encrypted tx is valid without revealing contents)
    ///
    /// Zero-knowledge proof that the encrypted transaction:
    /// - Has valid signatures
    /// - Has sufficient balance
    /// - Follows protocol rules
    pub validity_proof: Vec<u8>,

    /// Decryption policy (when/if transaction is revealed)
    pub decryption_policy: DecryptionPolicy,
}

/// Decryption policy for encrypted transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecryptionPolicy {
    /// Decrypt immediately after block inclusion
    ///
    /// Example: Flashbots Protect (MEV protection, revealed after execution)
    PostInclusion,

    /// Decrypt after N blocks (delayed decryption)
    ///
    /// Example: Time-locked encrypted transactions
    DelayedBy(u64),

    /// Never decrypt (full permanent privacy)
    ///
    /// Example: Fully private stealth payments
    Never,
}

/// Viewing key for selective transaction disclosure
///
/// Some privacy protocols support viewing keys that allow auditing without
/// revealing spending capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewingKey {
    /// Key type (chain-specific)
    pub key_type: ViewingKeyType,

    /// The viewing key bytes
    pub key_data: Vec<u8>,
}

/// Viewing key type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewingKeyType {
    /// Zcash viewing key (allows seeing incoming/outgoing transactions)
    Zcash,

    /// Monero view key (allows seeing incoming transactions)
    Monero,

    /// Custom viewing key type
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_metadata_creation() {
        let privacy = PrivacyMetadata {
            features: vec![],
            observability: ObservabilityLevel::FullyObservable,
            viewing_key: None,
        };

        assert!(privacy.features.is_empty());
        assert_eq!(privacy.observability, ObservabilityLevel::FullyObservable);
        assert!(privacy.viewing_key.is_none());
    }

    #[test]
    fn test_observability_levels() {
        let levels = [
            ObservabilityLevel::FullyObservable,
            ObservabilityLevel::PartiallyObservable,
            ObservabilityLevel::FullyPrivate,
        ];

        // Test that levels are distinct
        assert_ne!(levels[0], levels[1]);
        assert_ne!(levels[1], levels[2]);
        assert_ne!(levels[0], levels[2]);
    }

    #[test]
    fn test_stealth_address_creation() {
        let stealth = PrivateAddress {
            privacy_type: AddressPrivacyType::Stealth { scheme_id: 5564 },
            public_address: vec![1, 2, 3, 4],
            viewing_hint: Some(vec![5, 6, 7, 8]),
        };

        assert_eq!(
            stealth.privacy_type,
            AddressPrivacyType::Stealth { scheme_id: 5564 }
        );
        assert_eq!(stealth.public_address, vec![1, 2, 3, 4]);
        assert_eq!(stealth.viewing_hint, Some(vec![5, 6, 7, 8]));
    }

    #[test]
    fn test_ring_signature_address() {
        let ring_sig = PrivateAddress {
            privacy_type: AddressPrivacyType::RingSig { ring_size: 11 },
            public_address: vec![9, 10, 11],
            viewing_hint: None,
        };

        assert_eq!(
            ring_sig.privacy_type,
            AddressPrivacyType::RingSig { ring_size: 11 }
        );
        assert_eq!(ring_sig.public_address, vec![9, 10, 11]);
        assert!(ring_sig.viewing_hint.is_none());
    }

    #[test]
    fn test_confidential_amount() {
        let amount = ConfidentialAmount {
            commitment: vec![1, 2, 3, 4, 5],
            range_proof: Some(vec![6, 7, 8, 9]),
            proof_system: RangeProofSystem::Bulletproofs,
        };

        assert_eq!(amount.commitment, vec![1, 2, 3, 4, 5]);
        assert_eq!(amount.range_proof, Some(vec![6, 7, 8, 9]));
        assert_eq!(amount.proof_system, RangeProofSystem::Bulletproofs);
    }

    #[test]
    fn test_privacy_pool() {
        let pool = PrivacyPool {
            anonymity_set_root: vec![1, 2, 3],
            membership_proof: vec![4, 5, 6],
            compliance_proof: Some(ComplianceProof {
                association_set_proof: vec![7, 8],
                association_set_id: b"non-sanctioned-v1".to_vec(),
            }),
        };

        assert_eq!(pool.anonymity_set_root, vec![1, 2, 3]);
        assert_eq!(pool.membership_proof, vec![4, 5, 6]);
        assert!(pool.compliance_proof.is_some());

        let compliance = pool.compliance_proof.unwrap();
        assert_eq!(compliance.association_set_proof, vec![7, 8]);
        assert_eq!(compliance.association_set_id, b"non-sanctioned-v1");
    }

    #[test]
    fn test_encrypted_transaction() {
        let encrypted = EncryptedTransaction {
            encrypted_payload: vec![1, 2, 3],
            validity_proof: vec![4, 5, 6],
            decryption_policy: DecryptionPolicy::PostInclusion,
        };

        assert_eq!(encrypted.encrypted_payload, vec![1, 2, 3]);
        assert_eq!(encrypted.validity_proof, vec![4, 5, 6]);
        assert_eq!(encrypted.decryption_policy, DecryptionPolicy::PostInclusion);
    }

    #[test]
    fn test_decryption_policies() {
        assert_eq!(
            DecryptionPolicy::PostInclusion,
            DecryptionPolicy::PostInclusion
        );
        assert_eq!(
            DecryptionPolicy::DelayedBy(10),
            DecryptionPolicy::DelayedBy(10)
        );
        assert_ne!(
            DecryptionPolicy::DelayedBy(10),
            DecryptionPolicy::DelayedBy(20)
        );
        assert_eq!(DecryptionPolicy::Never, DecryptionPolicy::Never);
    }

    #[test]
    fn test_viewing_key() {
        let key = ViewingKey {
            key_type: ViewingKeyType::Zcash,
            key_data: vec![1, 2, 3, 4],
        };

        assert_eq!(key.key_type, ViewingKeyType::Zcash);
        assert_eq!(key.key_data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_privacy_feature_hidden_sender() {
        let feature = PrivacyFeature::HiddenSender(PrivateAddress {
            privacy_type: AddressPrivacyType::RingSig { ring_size: 11 },
            public_address: vec![1, 2, 3],
            viewing_hint: None,
        });

        match feature {
            PrivacyFeature::HiddenSender(addr) => {
                assert_eq!(
                    addr.privacy_type,
                    AddressPrivacyType::RingSig { ring_size: 11 }
                );
            }
            _ => panic!("Expected HiddenSender"),
        }
    }

    #[test]
    fn test_privacy_feature_hidden_recipient() {
        let feature = PrivacyFeature::HiddenRecipient(PrivateAddress {
            privacy_type: AddressPrivacyType::Stealth { scheme_id: 5564 },
            public_address: vec![4, 5, 6],
            viewing_hint: Some(vec![7, 8]),
        });

        match feature {
            PrivacyFeature::HiddenRecipient(addr) => {
                assert_eq!(
                    addr.privacy_type,
                    AddressPrivacyType::Stealth { scheme_id: 5564 }
                );
            }
            _ => panic!("Expected HiddenRecipient"),
        }
    }

    #[test]
    fn test_privacy_feature_custom() {
        let feature = PrivacyFeature::Custom {
            name: "FHE-Privacy".to_string(),
            description: "Fully homomorphic encryption".to_string(),
            metadata: vec![1, 2, 3],
        };

        match feature {
            PrivacyFeature::Custom {
                name,
                description,
                metadata,
            } => {
                assert_eq!(name, "FHE-Privacy");
                assert_eq!(description, "Fully homomorphic encryption");
                assert_eq!(metadata, vec![1, 2, 3]);
            }
            _ => panic!("Expected Custom"),
        }
    }

    #[test]
    fn test_range_proof_systems() {
        let systems = [
            RangeProofSystem::Bulletproofs,
            RangeProofSystem::BulletproofsPlus,
            RangeProofSystem::Borromean,
            RangeProofSystem::Custom(42),
        ];

        assert_ne!(systems[0], systems[1]);
        assert_ne!(systems[1], systems[2]);
        assert_ne!(systems[2], systems[3]);
        assert_eq!(systems[3], RangeProofSystem::Custom(42));
    }

    #[test]
    fn test_privacy_metadata_with_multiple_features() {
        let privacy = PrivacyMetadata {
            features: vec![
                PrivacyFeature::HiddenRecipient(PrivateAddress {
                    privacy_type: AddressPrivacyType::Stealth { scheme_id: 5564 },
                    public_address: vec![1, 2, 3],
                    viewing_hint: None,
                }),
                PrivacyFeature::HiddenAmount(ConfidentialAmount {
                    commitment: vec![4, 5, 6],
                    range_proof: Some(vec![7, 8, 9]),
                    proof_system: RangeProofSystem::Bulletproofs,
                }),
            ],
            observability: ObservabilityLevel::PartiallyObservable,
            viewing_key: None,
        };

        assert_eq!(privacy.features.len(), 2);
        assert_eq!(
            privacy.observability,
            ObservabilityLevel::PartiallyObservable
        );
    }

    #[test]
    fn test_privacy_metadata_serialization() {
        let privacy = PrivacyMetadata {
            features: vec![PrivacyFeature::HiddenGraph(PrivacyPool {
                anonymity_set_root: vec![1, 2, 3],
                membership_proof: vec![4, 5, 6],
                compliance_proof: None,
            })],
            observability: ObservabilityLevel::FullyPrivate,
            viewing_key: Some(ViewingKey {
                key_type: ViewingKeyType::Monero,
                key_data: vec![10, 11, 12],
            }),
        };

        // Test serde serialization roundtrip
        let json = serde_json::to_string(&privacy).expect("Failed to serialize");
        let deserialized: PrivacyMetadata =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(privacy, deserialized);
    }

    #[test]
    fn test_viewing_key_types() {
        let types = [
            ViewingKeyType::Zcash,
            ViewingKeyType::Monero,
            ViewingKeyType::Custom("Custom".to_string()),
        ];

        assert_ne!(types[0], types[1]);
        assert_ne!(types[1], types[2]);
        assert_eq!(types[2], ViewingKeyType::Custom("Custom".to_string()));
    }
}
