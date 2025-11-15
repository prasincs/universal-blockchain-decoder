//! Zcash transaction types
//!
//! This module defines the data structures for Zcash transactions across all protocol versions.

use decoder_bitcoin::parsing::{TxInput, TxOutput};
use decoder_primitives::prelude::*;

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
}

/// Sapling shielded transaction
///
/// Phase 2: Not yet implemented
/// See: docs/ZCASH_INTEGRATION_PLAN.md, Phase 2
#[derive(Debug, Clone, PartialEq)]
pub struct SaplingTransaction {
    /// Base transparent transaction data
    pub transparent: TransparentTransaction,

    /// Sapling spend descriptions (consume shielded notes)
    pub spends: Vec<SpendDescription>,

    /// Sapling output descriptions (create shielded notes)
    pub outputs: Vec<OutputDescription>,

    /// Net value balance (transparent ↔ shielded)
    pub value_balance: i64,

    /// Binding signature (proves value conservation, 64 bytes)
    pub binding_sig: Vec<u8>,
}

/// Sapling spend description (Phase 2)
#[derive(Debug, Clone, PartialEq)]
pub struct SpendDescription {
    /// Value commitment (32 bytes)
    pub cv: Vec<u8>,

    /// Merkle tree anchor (32 bytes)
    pub anchor: Vec<u8>,

    /// Nullifier (prevents double-spend, 32 bytes)
    pub nullifier: Vec<u8>,

    /// Randomized public key (32 bytes)
    pub rk: Vec<u8>,

    /// zk-SNARK proof (Groth16, 192 bytes)
    pub zkproof: Vec<u8>,

    /// Spend authorization signature (64 bytes)
    pub spend_auth_sig: Vec<u8>,
}

/// Sapling output description (Phase 2)
#[derive(Debug, Clone, PartialEq)]
pub struct OutputDescription {
    /// Value commitment (32 bytes)
    pub cv: Vec<u8>,

    /// Note commitment (32 bytes)
    pub cmu: Vec<u8>,

    /// Ephemeral public key (for ECDH, 32 bytes)
    pub ephemeral_key: Vec<u8>,

    /// Encrypted note ciphertext (ChaCha20-Poly1305, 580 bytes)
    pub enc_ciphertext: Vec<u8>,

    /// Encrypted outgoing ciphertext (for sender recovery, 80 bytes)
    pub out_ciphertext: Vec<u8>,

    /// zk-SNARK proof (Groth16, 192 bytes)
    pub zkproof: Vec<u8>,
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

impl<'a> Canonicalizer<'a> for ZcashTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        match self {
            ZcashTransaction::Transparent(tx) => tx.canonicalize(),
            ZcashTransaction::Sapling(_) => Err(DecoderError::chain_specific(
                "Sapling transaction canonicalization not yet implemented (Phase 2)".to_string(),
            )),
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
