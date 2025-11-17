//! Cardano transaction types
//!
//! This module defines the data structures for representing Cardano transactions.

use decoder_primitives::prelude::*;
use serde::{Deserialize, Serialize};
use universal_decoder_core::hex;

/// A complete Cardano transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardanoTransaction {
    /// Transaction body containing inputs, outputs, and metadata
    pub body: TransactionBody,
    /// Witness set containing signatures and scripts
    pub witness_set: WitnessSet,
    /// Optional auxiliary data (metadata)
    pub auxiliary_data: Option<AuxiliaryData>,
    /// Validity flag (for Alonzo era and later)
    pub is_valid: Option<bool>,
    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

/// Transaction body containing the core transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBody {
    /// Transaction inputs (UTXOs being spent)
    pub inputs: Vec<TransactionInput>,
    /// Transaction outputs (new UTXOs being created)
    pub outputs: Vec<TransactionOutput>,
    /// Transaction fee in lovelace
    pub fee: u64,
    /// Time-to-live (slot number)
    pub ttl: Option<u64>,
    /// Certificates (stake pool registration, delegation, etc.)
    pub certificates: Vec<Certificate>,
    /// Withdrawals from reward accounts
    pub withdrawals: Vec<Withdrawal>,
    /// Script data hash (for Plutus scripts)
    pub script_data_hash: Option<Vec<u8>>,
    /// Required signers (for Plutus scripts)
    pub required_signers: Vec<Vec<u8>>,
    /// Network ID
    pub network_id: Option<u8>,
    /// Collateral inputs (for Plutus scripts)
    pub collateral: Vec<TransactionInput>,
    /// Mint/burn multi-assets
    pub mint: Vec<MultiAsset>,
}

/// Transaction input (reference to a previous UTXO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Transaction hash of the UTXO being spent
    pub transaction_id: Vec<u8>,
    /// Index of the output in the referenced transaction
    pub index: u64,
}

/// Transaction output (new UTXO being created)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// Payment address
    pub address: Vec<u8>,
    /// Amount in lovelace
    pub amount: u64,
    /// Multi-asset tokens
    pub assets: Vec<MultiAsset>,
    /// Datum hash (for Plutus scripts)
    pub datum_hash: Option<Vec<u8>>,
    /// Inline datum (Babbage era)
    pub inline_datum: Option<Vec<u8>>,
}

/// Multi-asset token (native tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAsset {
    /// Policy ID
    pub policy_id: Vec<u8>,
    /// Asset name
    pub asset_name: Vec<u8>,
    /// Amount (can be negative for burning)
    pub amount: i64,
}

/// Certificate (stake pool operations, delegation, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Certificate {
    /// Stake registration
    StakeRegistration { stake_credential: Vec<u8> },
    /// Stake deregistration
    StakeDeregistration { stake_credential: Vec<u8> },
    /// Stake delegation
    StakeDelegation {
        stake_credential: Vec<u8>,
        pool_keyhash: Vec<u8>,
    },
    /// Pool registration
    PoolRegistration { pool_params: Vec<u8> },
    /// Pool retirement
    PoolRetirement { pool_keyhash: Vec<u8>, epoch: u64 },
    /// Other certificate types
    Other { raw: Vec<u8> },
}

/// Withdrawal from reward account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    /// Reward address
    pub address: Vec<u8>,
    /// Amount in lovelace
    pub amount: u64,
}

/// Witness set containing signatures and scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessSet {
    /// Verification key witnesses (signatures)
    pub vkey_witnesses: Vec<VKeyWitness>,
    /// Native scripts
    pub native_scripts: Vec<Vec<u8>>,
    /// Plutus V1 scripts
    pub plutus_v1_scripts: Vec<Vec<u8>>,
    /// Plutus V2 scripts
    pub plutus_v2_scripts: Vec<Vec<u8>>,
    /// Redeemers (for Plutus scripts)
    pub redeemers: Vec<Redeemer>,
    /// Plutus data
    pub plutus_data: Vec<Vec<u8>>,
}

/// Verification key witness (signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VKeyWitness {
    /// Public key
    pub vkey: Vec<u8>,
    /// Signature
    pub signature: Vec<u8>,
}

/// Redeemer for Plutus script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redeemer {
    /// Tag (spend, mint, cert, reward)
    pub tag: u8,
    /// Index
    pub index: u64,
    /// Plutus data
    pub data: Vec<u8>,
    /// Execution units
    pub ex_units: ExUnits,
}

/// Execution units for Plutus scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExUnits {
    /// Memory units
    pub mem: u64,
    /// Step units
    pub steps: u64,
}

/// Auxiliary data (transaction metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxiliaryData {
    /// Metadata map
    pub metadata: Vec<(u64, MetadataValue)>,
    /// Native scripts
    pub native_scripts: Vec<Vec<u8>>,
    /// Plutus scripts
    pub plutus_scripts: Vec<Vec<u8>>,
}

/// Metadata value (can be nested)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataValue {
    Int(i64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<MetadataValue>),
    Map(Vec<(MetadataValue, MetadataValue)>),
}

impl CardanoTransaction {
    /// Get the transaction ID (hash of transaction body)
    pub fn txid(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};

        // In Cardano, the TXID is the hash of the transaction body CBOR
        // For now, we hash the entire transaction (this is simplified)
        // A proper implementation would hash just the body
        let mut hasher = Sha256::new();
        hasher.update(&self.raw_bytes);
        hasher.finalize().to_vec()
    }

    /// Get the transaction ID as a hex string
    pub fn txid_hex(&self) -> String {
        hex::encode(&self.txid())
    }

    /// Get the number of inputs
    pub fn input_count(&self) -> usize {
        self.body.inputs.len()
    }

    /// Get the number of outputs
    pub fn output_count(&self) -> usize {
        self.body.outputs.len()
    }

    /// Get the transaction fee in lovelace
    pub fn fee(&self) -> u64 {
        self.body.fee
    }

    /// Check if transaction has certificates
    pub fn has_certificates(&self) -> bool {
        !self.body.certificates.is_empty()
    }

    /// Check if transaction has withdrawals
    pub fn has_withdrawals(&self) -> bool {
        !self.body.withdrawals.is_empty()
    }

    /// Check if transaction mints/burns assets
    pub fn has_mint(&self) -> bool {
        !self.body.mint.is_empty()
    }

    /// Check if transaction has Plutus scripts
    pub fn has_plutus_scripts(&self) -> bool {
        !self.witness_set.plutus_v1_scripts.is_empty()
            || !self.witness_set.plutus_v2_scripts.is_empty()
    }

    /// Check if transaction has metadata
    pub fn has_metadata(&self) -> bool {
        self.auxiliary_data.is_some()
    }
}

impl<'a> Canonicalizer<'a> for CardanoTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Compute transaction hash
        let tx_hash = self.txid();

        let metadata = TxMetadata {
            tx_hash,
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!(
                "fee={} inputs={} outputs={}",
                self.fee(),
                self.input_count(),
                self.output_count()
            ),
        };

        // Extract signatures from witness set
        let signatures = self
            .witness_set
            .vkey_witnesses
            .iter()
            .enumerate()
            .map(|(idx, w)| Signature {
                data: w.signature.clone(),
                key_index: idx,
                metadata: None,
            })
            .collect();

        let public_keys = self
            .witness_set
            .vkey_witnesses
            .iter()
            .map(|w| PublicKey {
                data: w.vkey.clone(),
                key_type: KeyType::Ed25519,
            })
            .collect();

        let authorization = AuthorizationPackage {
            signatures,
            public_keys,
            signature_scheme: SignatureScheme::EdDsa,
        };

        // Convert inputs to input references
        let inputs = self
            .body
            .inputs
            .iter()
            .map(|input| InputReference {
                prev_tx: input.transaction_id.clone(),
                output_index: input.index as u32,
                value: Amount::new(0, 6), // We don't have the amount here
                script: vec![],
            })
            .collect();

        // Convert outputs to output values
        let outputs = self
            .body
            .outputs
            .iter()
            .enumerate()
            .map(|(idx, output)| OutputValue {
                index: idx as u32,
                address: Address {
                    bytes: output.address.clone(),
                    human_readable: None,
                },
                value: Amount::new(output.amount as u128, 6), // 6 decimals for ADA
                script: vec![],
            })
            .collect();

        let state_deltas = StateDeltas {
            inputs,
            outputs,
            account_changes: vec![],
        };

        // Create operations
        let mut operations = vec![];

        // Add fee operation
        operations.push(Operation::Transfer(Transfer {
            from: Address {
                bytes: vec![],
                human_readable: None,
            },
            to: Address {
                bytes: vec![],
                human_readable: None,
            },
            amount: Amount::new(self.body.fee as u128, 6),
            asset: AssetId::Native,
        }));

        Ok(TxIR::new(
            &super::CardanoChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Basic validation
        if self.body.inputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Transaction must have at least one input",
            ));
        }

        if self.body.outputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Transaction must have at least one output",
            ));
        }

        if self.body.fee == 0 {
            return Err(DecoderError::invalid_structure(
                "Transaction fee cannot be zero",
            ));
        }

        Ok(())
    }
}
