//! Zcash transaction types
//!
//! This module defines the data structures for Zcash transactions across all protocol versions.

use decoder_bitcoin::parsing::{TxInput, TxOutput};
use decoder_primitives::prelude::*;

// Re-export Sapling types from sapling module
pub use crate::sapling::{OutputDescription, SpendDescription};

/// Zcash transaction representation
///
/// Zcash supports multiple transaction types:
/// - Transparent: Bitcoin-compatible UTXO model
/// - Sapling: zk-SNARK shielded transactions (Phase 2)
/// - Orchard: Latest shielded protocol with Halo2 (Phase 4)
#[derive(Debug, Clone, PartialEq)]
pub enum ZcashTransaction {
    /// Transparent transaction (Bitcoin-compatible)
    ///
    /// Phase 1: Fully supported
    /// Used for: t→t transactions, transparent inputs/outputs in hybrid transactions
    Transparent(TransparentTransaction),

    /// Sapling shielded transaction
    ///
    /// Phase 2: Planned (Week 9, Days 3-6)
    /// Used for: t→z, z→t, z→z transactions
    #[allow(dead_code)]
    Sapling(SaplingTransaction),

    /// Orchard shielded transaction (NU5+)
    ///
    /// Phase 4: Planned (Week 11, Days 9-11)
    /// Used for: Latest shielded protocol, unified addresses
    #[allow(dead_code)]
    Orchard(OrchardTransaction),
}

/// Transparent Zcash transaction (Bitcoin-compatible)
///
/// This is structurally similar to Bitcoin transactions but includes
/// Zcash-specific fields like `version_group_id` and `expiry_height`.
#[derive(Debug, Clone, PartialEq)]
pub struct TransparentTransaction {
    /// Transaction version
    ///
    /// - v1-v3: Pre-Sapling (Sprout) - Not supported
    /// - v4: Sapling (with Overwinter bit set)
    /// - v5: Orchard (NU5+)
    pub version: u32,

    /// Version group ID (Zcash-specific)
    ///
    /// Indicates consensus branch:
    /// - 0x892F2085: Sapling
    /// - 0x26A7270A: Blossom
    /// - 0xF919A198: Heartwood
    /// - 0xC2D6D0B4: Canopy
    /// - 0x00000000: Pre-Overwinter
    pub version_group_id: u32,

    /// Transaction inputs (same format as Bitcoin)
    pub inputs: Vec<TxInput>,

    /// Transaction outputs (same format as Bitcoin)
    pub outputs: Vec<TxOutput>,

    /// Locktime (same as Bitcoin)
    pub locktime: u32,

    /// Expiry height (Zcash-specific)
    ///
    /// Block height after which transaction is invalid
    /// 0 = no expiry (not recommended)
    pub expiry_height: u32,

    /// SegWit flag (for hybrid SegWit support in transparent component)
    pub is_segwit: bool,

    /// Witness data (if SegWit)
    pub witnesses: Option<Vec<Vec<Vec<u8>>>>,

    /// Raw transaction bytes (for re-encoding)
    pub raw_bytes: Vec<u8>,
}

/// Sapling shielded transaction
///
/// Phase 2: Implemented
/// See: docs/ZCASH_INTEGRATION_PLAN.md, Phase 2
#[derive(Debug, Clone, PartialEq)]
pub struct SaplingTransaction {
    /// Base transparent transaction data
    ///
    /// Contains version, inputs, outputs, locktime, expiry_height
    pub transparent: TransparentTransaction,

    /// Sapling spend descriptions (consume shielded notes)
    ///
    /// Each spend reveals a nullifier and proves knowledge of a note
    /// without revealing the note's value or recipient.
    pub spends: Vec<SpendDescription>,

    /// Sapling output descriptions (create shielded notes)
    ///
    /// Each output creates an encrypted note that only the recipient
    /// (or someone with the viewing key) can decrypt.
    pub outputs: Vec<OutputDescription>,

    /// Net value balance (transparent ↔ shielded)
    ///
    /// Positive: Transparent → Shielded (shielding)
    /// Negative: Shielded → Transparent (deshielding)
    /// Zero: Pure shielded (z→z) or pure transparent (t→t)
    pub value_balance: i64,

    /// Binding signature (proves value conservation, 64 bytes)
    ///
    /// RedJubjub signature that proves:
    /// `sum(value_commitments) - value_balance * G = 0`
    ///
    /// Ensures no value is created or destroyed.
    pub binding_sig: [u8; 64],

    /// Raw transaction bytes (for re-encoding)
    pub raw_bytes: Vec<u8>,
}

/// Orchard shielded transaction (Phase 4)
///
/// Phase 4: Not yet implemented
/// See: docs/ZCASH_INTEGRATION_PLAN.md, Phase 4
#[derive(Debug, Clone, PartialEq)]
pub struct OrchardTransaction {
    /// Base transparent transaction data
    pub transparent: TransparentTransaction,

    /// Orchard actions (combined spend+output)
    pub actions: Vec<ActionDescription>,

    /// Flags (enable spends/outputs)
    pub flags: u8,

    /// Net value balance (transparent ↔ shielded)
    pub value_balance: i64,

    /// Merkle tree anchor (32 bytes)
    pub anchor: Vec<u8>,

    /// Halo2 proof
    pub proof: Vec<u8>,

    /// Binding signature (64 bytes)
    pub binding_sig: Vec<u8>,
}

/// Orchard action description (Phase 4)
#[derive(Debug, Clone, PartialEq)]
pub struct ActionDescription {
    /// Net value commitment (32 bytes)
    pub cv_net: Vec<u8>,

    /// Nullifier (32 bytes)
    pub nullifier: Vec<u8>,

    /// Randomized verification key (32 bytes)
    pub rk: Vec<u8>,

    /// Note commitment (32 bytes)
    pub cmx: Vec<u8>,

    /// Ephemeral public key (32 bytes)
    pub ephemeral_key: Vec<u8>,

    /// Encrypted note ciphertext (580 bytes)
    pub enc_ciphertext: Vec<u8>,

    /// Encrypted outgoing ciphertext (80 bytes)
    pub out_ciphertext: Vec<u8>,
}

impl ChainEncoder for ZcashTransaction {
    /// Re-encode the Zcash transaction back to its original byte format
    ///
    /// Since we store the original raw bytes during decoding, this simply
    /// returns a clone of those bytes, guaranteeing exact reconstruction.
    ///
    /// # Formal Properties
    ///
    /// This implementation trivially satisfies the injective property:
    /// ```text
    /// ∀ tx_bytes: ZcashDecoder::decode(tx_bytes)?.to_bytes()? == tx_bytes
    /// ```
    ///
    /// Because we store `raw_bytes` during decode, the roundtrip is guaranteed.
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            ZcashTransaction::Transparent(tx) => Ok(tx.raw_bytes.clone()),
            ZcashTransaction::Sapling(tx) => Ok(tx.raw_bytes.clone()),
            ZcashTransaction::Orchard(_) => Err(DecoderError::chain_specific(
                "Orchard transaction re-encoding not yet implemented (Phase 4)".to_string(),
            )),
        }
    }
}

impl<'a> Canonicalizer<'a> for ZcashTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        match self {
            ZcashTransaction::Transparent(tx) => tx.canonicalize(),
            ZcashTransaction::Sapling(tx) => tx.canonicalize(),
            ZcashTransaction::Orchard(_) => Err(DecoderError::chain_specific(
                "Orchard transaction canonicalization not yet implemented (Phase 4)".to_string(),
            )),
        }
    }
}

impl<'a> Canonicalizer<'a> for TransparentTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        use universal_decoder_core::ir::*;
        use universal_decoder_core::privacy::{ObservabilityLevel, PrivacyMetadata};

        // Metadata (minimal for Phase 1)
        let metadata = TxMetadata {
            tx_hash: vec![], // Will be computed by caller
            block_height: None,
            timestamp: None,
            size: 0, // Will be populated by decoder
            extra: format!(
                "{{\"version\":{},\"version_group_id\":{},\"locktime\":{},\"expiry_height\":{}}}",
                self.version, self.version_group_id, self.locktime, self.expiry_height
            ),
        };

        // Authorization package (Phase 1: empty, no signature extraction yet)
        let authorization = AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Operations: Convert UTXO inputs/outputs to Transfer operations
        let mut operations = Vec::new();

        for (idx, input) in self.inputs.iter().enumerate() {
            // Create a generic operation for UTXO consumption
            operations.push(Operation::Generic(GenericOperation {
                op_type: "UTXO_Input".to_string(),
                data: format!(
                    "{{\"index\":{},\"prev_txid\":\"{}\",\"prev_vout\":{},\"sequence\":{}}}",
                    idx,
                    universal_decoder_core::hex::encode(input.prev_hash),
                    input.prev_index,
                    input.sequence
                )
                .as_bytes()
                .to_vec(),
                metadata: "{}".to_string(),
            }));
        }

        for (idx, output) in self.outputs.iter().enumerate() {
            // Create a generic operation for UTXO creation
            operations.push(Operation::Generic(GenericOperation {
                op_type: "UTXO_Output".to_string(),
                data: format!(
                    "{{\"index\":{},\"value\":{},\"script_pubkey_hex\":\"{}\"}}",
                    idx,
                    output.value,
                    universal_decoder_core::hex::encode(&output.script_pubkey)
                )
                .as_bytes()
                .to_vec(),
                metadata: "{}".to_string(),
            }));
        }

        // State deltas (UTXO inputs and outputs)
        let mut inputs = Vec::new();
        for input in &self.inputs {
            inputs.push(InputReference {
                prev_tx: input.prev_hash.to_vec(),
                output_index: input.prev_index,
                value: Amount::new(0, 8), // Unknown without UTXO set, ZEC has 8 decimals
                script: input.script_sig.clone(),
            });
        }

        let mut outputs = Vec::new();
        for (idx, output) in self.outputs.iter().enumerate() {
            outputs.push(OutputValue {
                index: idx as u32,
                address: Address {
                    bytes: output.script_pubkey.clone(),
                    human_readable: None,
                },
                value: Amount::new(output.value as u128, 8), // ZEC has 8 decimals
                script: output.script_pubkey.clone(),
            });
        }

        let state_deltas = StateDeltas {
            inputs,
            outputs,
            account_changes: vec![],
        };

        // Privacy metadata: Transparent transactions are fully observable
        let privacy = Some(PrivacyMetadata {
            features: vec![],
            observability: ObservabilityLevel::FullyObservable,
            viewing_key: None,
        });

        Ok(TxIR::with_privacy(
            &decoder_chains_common::chains::ZCASH,
            metadata,
            authorization,
            operations,
            state_deltas,
            privacy,
        ))
    }
}

impl<'a> Canonicalizer<'a> for SaplingTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        use universal_decoder_core::ir::*;
        use universal_decoder_core::privacy::{
            ObservabilityLevel, PrivacyFeature, PrivacyMetadata,
        };

        // Metadata (include Sapling-specific info)
        let metadata = TxMetadata {
            tx_hash: vec![], // Will be computed by caller
            block_height: None,
            timestamp: None,
            size: 0, // Will be populated by decoder
            extra: format!(
                "{{\"version\":{},\"version_group_id\":{},\"locktime\":{},\"expiry_height\":{},\"sapling_spends\":{},\"sapling_outputs\":{},\"value_balance\":{}}}",
                self.transparent.version,
                self.transparent.version_group_id,
                self.transparent.locktime,
                self.transparent.expiry_height,
                self.spends.len(),
                self.outputs.len(),
                self.value_balance
            ),
        };

        // Authorization package (Phase 1: empty, no signature extraction yet)
        let authorization = AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Operations: Include both transparent and shielded operations
        let mut operations = Vec::new();

        // Transparent inputs
        for (idx, input) in self.transparent.inputs.iter().enumerate() {
            operations.push(Operation::Generic(GenericOperation {
                op_type: "UTXO_Input".to_string(),
                data: format!(
                    "{{\"index\":{},\"prev_txid\":\"{}\",\"prev_vout\":{},\"sequence\":{}}}",
                    idx,
                    universal_decoder_core::hex::encode(input.prev_hash),
                    input.prev_index,
                    input.sequence
                )
                .as_bytes()
                .to_vec(),
                metadata: "{}".to_string(),
            }));
        }

        // Transparent outputs
        for (idx, output) in self.transparent.outputs.iter().enumerate() {
            operations.push(Operation::Generic(GenericOperation {
                op_type: "UTXO_Output".to_string(),
                data: format!(
                    "{{\"index\":{},\"value\":{},\"script_pubkey_hex\":\"{}\"}}",
                    idx,
                    output.value,
                    universal_decoder_core::hex::encode(&output.script_pubkey)
                )
                .as_bytes()
                .to_vec(),
                metadata: "{}".to_string(),
            }));
        }

        // Sapling spends (shielded inputs)
        for (idx, spend) in self.spends.iter().enumerate() {
            operations.push(Operation::Generic(GenericOperation {
                op_type: "Sapling_Spend".to_string(),
                data: format!(
                    "{{\"index\":{},\"nullifier\":\"{}\",\"cv\":\"{}\",\"anchor\":\"{}\"}}",
                    idx,
                    universal_decoder_core::hex::encode(spend.nullifier),
                    universal_decoder_core::hex::encode(spend.cv),
                    universal_decoder_core::hex::encode(spend.anchor)
                )
                .as_bytes()
                .to_vec(),
                metadata: "{}".to_string(),
            }));
        }

        // Sapling outputs (shielded outputs)
        for (idx, output) in self.outputs.iter().enumerate() {
            operations.push(Operation::Generic(GenericOperation {
                op_type: "Sapling_Output".to_string(),
                data: format!(
                    "{{\"index\":{},\"cmu\":\"{}\",\"cv\":\"{}\",\"ephemeral_key\":\"{}\"}}",
                    idx,
                    universal_decoder_core::hex::encode(output.cmu),
                    universal_decoder_core::hex::encode(output.cv),
                    universal_decoder_core::hex::encode(output.ephemeral_key)
                )
                .as_bytes()
                .to_vec(),
                metadata: "{}".to_string(),
            }));
        }

        // State deltas (UTXO inputs and outputs)
        let mut inputs = Vec::new();
        for input in &self.transparent.inputs {
            inputs.push(InputReference {
                prev_tx: input.prev_hash.to_vec(),
                output_index: input.prev_index,
                value: Amount::new(0, 8), // Unknown without UTXO set, ZEC has 8 decimals
                script: input.script_sig.clone(),
            });
        }

        let mut outputs = Vec::new();
        for (idx, output) in self.transparent.outputs.iter().enumerate() {
            outputs.push(OutputValue {
                index: idx as u32,
                address: Address {
                    bytes: output.script_pubkey.clone(),
                    human_readable: None,
                },
                value: Amount::new(output.value as u128, 8), // ZEC has 8 decimals
                script: output.script_pubkey.clone(),
            });
        }

        let state_deltas = StateDeltas {
            inputs,
            outputs,
            account_changes: vec![],
        };

        // Privacy metadata: Sapling transactions have privacy features
        let mut privacy_features = Vec::new();

        // Add privacy features based on transaction structure
        if !self.spends.is_empty() {
            // Sapling spends use nullifiers (not directly linkable to addresses)
            privacy_features.push(PrivacyFeature::HiddenSender(
                universal_decoder_core::privacy::PrivateAddress {
                    privacy_type: universal_decoder_core::privacy::AddressPrivacyType::Custom {
                        mechanism_name: "Sapling_Nullifier".to_string(),
                        metadata: self.spends[0].nullifier.to_vec(),
                    },
                    public_address: self.spends[0].nullifier.to_vec(),
                    viewing_hint: None,
                },
            ));
        }
        if !self.outputs.is_empty() {
            // Sapling outputs use encrypted note commitments
            privacy_features.push(PrivacyFeature::HiddenRecipient(
                universal_decoder_core::privacy::PrivateAddress {
                    privacy_type: universal_decoder_core::privacy::AddressPrivacyType::Custom {
                        mechanism_name: "Sapling_NoteCommitment".to_string(),
                        metadata: self.outputs[0].cmu.to_vec(),
                    },
                    public_address: self.outputs[0].cmu.to_vec(),
                    viewing_hint: Some(self.outputs[0].ephemeral_key.to_vec()),
                },
            ));
        }
        if !self.spends.is_empty() || !self.outputs.is_empty() {
            // Sapling uses homomorphic commitments to hide amounts
            privacy_features.push(PrivacyFeature::HiddenAmount(
                universal_decoder_core::privacy::ConfidentialAmount {
                    commitment: if !self.spends.is_empty() {
                        self.spends[0].cv.to_vec()
                    } else {
                        self.outputs[0].cv.to_vec()
                    },
                    range_proof: Some(if !self.spends.is_empty() {
                        self.spends[0].zkproof.to_vec()
                    } else {
                        self.outputs[0].zkproof.to_vec()
                    }),
                    proof_system: universal_decoder_core::privacy::RangeProofSystem::Custom(
                        16, // Groth16 zk-SNARK (custom ID for Sapling)
                    ),
                },
            ));
        }

        // Determine observability level
        let observability =
            if self.transparent.inputs.is_empty() && self.transparent.outputs.is_empty() {
                // Pure shielded (z→z)
                ObservabilityLevel::FullyPrivate
            } else if !self.spends.is_empty() || !self.outputs.is_empty() {
                // Mixed (t→z, z→t, or complex)
                ObservabilityLevel::PartiallyObservable
            } else {
                // Pure transparent (fallback, should not happen in SaplingTransaction)
                ObservabilityLevel::FullyObservable
            };

        let privacy = Some(PrivacyMetadata {
            features: privacy_features,
            observability,
            viewing_key: None, // Phase 3: Will support viewing key decryption
        });

        Ok(TxIR::with_privacy(
            &decoder_chains_common::chains::ZCASH,
            metadata,
            authorization,
            operations,
            state_deltas,
            privacy,
        ))
    }
}
