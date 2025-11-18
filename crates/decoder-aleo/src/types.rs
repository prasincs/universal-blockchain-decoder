//! Aleo transaction type definitions

use crate::error::AleoDecoderError;
use decoder_primitives::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use universal_decoder_core::privacy::PrivacyMetadata;

/// An Aleo transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AleoTransaction {
    /// Transaction ID (hash of the transaction)
    pub id: Vec<u8>,

    /// Transaction type and content
    pub transaction_type: TransactionType,

    /// Optional fee payment
    pub fee: Option<Fee>,

    /// Raw transaction bytes (for signature verification)
    pub raw_bytes: Vec<u8>,
}

/// Aleo transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Deploy a new program to the blockchain
    Deploy(Deployment),

    /// Execute a program function
    Execute(Execution),

    /// Standalone fee transaction
    Fee(Fee),
}

/// Program deployment transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    /// Edition number (versioning for program updates)
    pub edition: u16,

    /// Program ID (unique identifier)
    pub program_id: String,

    /// Program source code (Leo language)
    pub program: String,

    /// Verifying keys for the program functions
    pub verifying_keys: Vec<VerifyingKey>,
}

/// Program execution transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    /// List of transitions (state changes)
    pub transitions: Vec<Transition>,

    /// Global state root (Merkle root)
    pub global_state_root: Vec<u8>,

    /// Optional proof for the execution
    pub proof: Option<Vec<u8>>,
}

/// Fee payment transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    /// Global state root at time of fee payment
    pub global_state_root: Vec<u8>,

    /// Fee amount in gates (Aleo's smallest unit)
    pub amount: u64,

    /// Priority fee (additional fee for faster processing)
    pub priority_fee: u64,

    /// Transition for the fee payment
    pub transition: Option<Transition>,
}

/// A transition represents a single state change in Aleo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Transition ID
    pub id: Vec<u8>,

    /// Program ID that this transition executes
    pub program_id: String,

    /// Function name being called
    pub function_name: String,

    /// Input records/values consumed by this transition
    pub inputs: Vec<TransitionInput>,

    /// Output records/values produced by this transition
    pub outputs: Vec<TransitionOutput>,

    /// zkSNARK proof for this transition
    pub proof: Option<Vec<u8>>,

    /// Finalize operation (on-chain state update)
    pub finalize: Vec<FinalizeOperation>,
}

/// Input to a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionInput {
    /// Constant value (public input)
    Constant { value: Vec<u8> },

    /// Public input (visible on-chain)
    Public { value: Vec<u8> },

    /// Private input (encrypted record)
    Private { ciphertext: Vec<u8> },

    /// Record input (consumes a UTXO-like record)
    Record {
        serial_number: Vec<u8>,
        tag: Vec<u8>,
    },

    /// External record (from another program)
    ExternalRecord { commitment: Vec<u8> },
}

/// Output from a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionOutput {
    /// Constant output
    Constant { value: Vec<u8> },

    /// Public output
    Public { value: Vec<u8> },

    /// Private output (encrypted)
    Private {
        ciphertext: Vec<u8>,
        commitment: Vec<u8>,
    },

    /// Record output (creates a new UTXO-like record)
    Record {
        commitment: Vec<u8>,
        nonce: Vec<u8>,
        checksum: Vec<u8>,
        ciphertext: Vec<u8>,
    },

    /// External record
    ExternalRecord { commitment: Vec<u8> },
}

/// Verifying key for a program function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyingKey {
    /// Function name
    pub function_name: String,

    /// Verifying key bytes
    pub key: Vec<u8>,
}

/// Finalize operation (on-chain state update)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinalizeOperation {
    /// Initialize a mapping
    InitializeMapping { name: String },

    /// Insert into mapping
    InsertMapping {
        name: String,
        key: Vec<u8>,
        value: Vec<u8>,
    },

    /// Update mapping
    UpdateMapping {
        name: String,
        key: Vec<u8>,
        value: Vec<u8>,
    },

    /// Remove from mapping
    RemoveMapping { name: String, key: Vec<u8> },
}

impl fmt::Display for AleoTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AleoTransaction {{ id: {}, type: {} }}",
            universal_decoder_core::hex::encode(&self.id),
            match &self.transaction_type {
                TransactionType::Deploy(_) => "Deploy",
                TransactionType::Execute(_) => "Execute",
                TransactionType::Fee(_) => "Fee",
            }
        )
    }
}

impl ChainEncoder for AleoTransaction {
    fn to_bytes(&self) -> decoder_primitives::Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for AleoTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> decoder_primitives::Result<TxIR<'a, 1>> {
        use universal_decoder_core::prelude::*;

        // Build metadata
        let metadata = TxMetadata {
            tx_hash: self.id.clone(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!(
                "{{\"type\":\"{}\",\"has_fee\":{}}}",
                self.transaction_type_str(),
                self.fee.is_some()
            ),
        };

        // Build authorization
        let authorization = AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Custom(0),
        };

        // Build operations
        let operations = self.build_operations();

        // Build state deltas
        let state_deltas = self.build_state_deltas();

        // Build privacy metadata
        let privacy = self.build_privacy_metadata();

        // Create chain reference
        let chain = decoder_chains_common::chains::ALEO;

        Ok(TxIR::with_privacy(
            &chain,
            metadata,
            authorization,
            operations,
            state_deltas,
            privacy,
        ))
    }

    fn validate(&self) -> decoder_primitives::Result<()> {
        match &self.transaction_type {
            TransactionType::Deploy(deploy) => {
                if deploy.program_id.is_empty() {
                    return Err(AleoDecoderError::InvalidProgram(
                        "Program ID cannot be empty".to_string(),
                    )
                    .into());
                }
                if deploy.program.is_empty() {
                    return Err(AleoDecoderError::InvalidProgram(
                        "Program source cannot be empty".to_string(),
                    )
                    .into());
                }
            }
            TransactionType::Execute(exec) => {
                if exec.transitions.is_empty() {
                    return Err(AleoDecoderError::InvalidTransition(
                        "Execution must have at least one transition".to_string(),
                    )
                    .into());
                }
            }
            TransactionType::Fee(_) => {
                // Fee transactions are always valid if parsed
            }
        }

        Ok(())
    }
}

impl AleoTransaction {
    fn transaction_type_str(&self) -> &str {
        match &self.transaction_type {
            TransactionType::Deploy(_) => "Deploy",
            TransactionType::Execute(_) => "Execute",
            TransactionType::Fee(_) => "Fee",
        }
    }

    fn build_operations(&self) -> Vec<Operation> {
        let mut operations = Vec::new();

        match &self.transaction_type {
            TransactionType::Deploy(deploy) => {
                operations.push(Operation::ContractDeploy(ContractDeploy {
                    bytecode: deploy.program.as_bytes().to_vec(),
                    constructor_args: vec![],
                    value: Amount::new(0, 0),
                }));
            }
            TransactionType::Execute(exec) => {
                for transition in &exec.transitions {
                    operations.push(Operation::ContractCall(ContractCall {
                        contract: Address {
                            bytes: transition.program_id.as_bytes().to_vec(),
                            human_readable: Some(transition.program_id.clone()),
                        },
                        method: transition.function_name.as_bytes().to_vec(),
                        data: vec![],
                        value: None,
                        resource_limits: ResourceLimits {
                            max_units: 0,
                            unit_price: 0,
                            resource_type: ResourceType::Gas,
                        },
                    }));
                }
            }
            TransactionType::Fee(fee) => {
                operations.push(Operation::Generic(GenericOperation {
                    op_type: "Fee".to_string(),
                    data: format!(
                        "{{\"amount\":{},\"priority\":{}}}",
                        fee.amount, fee.priority_fee
                    )
                    .into_bytes(),
                    metadata: String::new(),
                }));
            }
        }

        operations
    }

    fn build_state_deltas(&self) -> StateDeltas {
        let mut account_changes = Vec::new();

        if let TransactionType::Execute(exec) = &self.transaction_type {
            for transition in &exec.transitions {
                if !transition.finalize.is_empty() {
                    let mut storage_changes = Vec::new();

                    for finalize_op in &transition.finalize {
                        match finalize_op {
                            FinalizeOperation::InsertMapping {
                                name: _,
                                key,
                                value,
                            }
                            | FinalizeOperation::UpdateMapping {
                                name: _,
                                key,
                                value,
                            } => {
                                storage_changes.push(StorageChange {
                                    key: key.clone(),
                                    value: Some(value.clone()),
                                });
                            }
                            FinalizeOperation::RemoveMapping { name: _, key } => {
                                storage_changes.push(StorageChange {
                                    key: key.clone(),
                                    value: None,
                                });
                            }
                            FinalizeOperation::InitializeMapping { name: _ } => {
                                // No-op for initialization
                            }
                        }
                    }

                    if !storage_changes.is_empty() {
                        account_changes.push(AccountChange {
                            address: Address {
                                bytes: transition.program_id.as_bytes().to_vec(),
                                human_readable: Some(transition.program_id.clone()),
                            },
                            nonce: None,
                            balance_change: 0,
                            storage_changes,
                        });
                    }
                }
            }
        }

        StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes,
        }
    }

    fn build_privacy_metadata(&self) -> Option<PrivacyMetadata> {
        use universal_decoder_core::privacy::*;

        let has_private_data = match &self.transaction_type {
            TransactionType::Execute(exec) => exec
                .transitions
                .iter()
                .any(|t| self.has_private_inputs_or_outputs(t)),
            _ => false,
        };

        if has_private_data {
            Some(PrivacyMetadata {
                features: vec![
                    PrivacyFeature::HiddenAmount(ConfidentialAmount {
                        commitment: vec![],
                        range_proof: None,
                        proof_system: RangeProofSystem::Bulletproofs,
                    }),
                    PrivacyFeature::HiddenSender(PrivateAddress {
                        privacy_type: AddressPrivacyType::Custom {
                            mechanism_name: "Aleo Records".to_string(),
                            metadata: vec![],
                        },
                        public_address: vec![],
                        viewing_hint: None,
                    }),
                    PrivacyFeature::HiddenRecipient(PrivateAddress {
                        privacy_type: AddressPrivacyType::Custom {
                            mechanism_name: "Aleo Records".to_string(),
                            metadata: vec![],
                        },
                        public_address: vec![],
                        viewing_hint: None,
                    }),
                ],
                observability: ObservabilityLevel::FullyPrivate,
                viewing_key: Some(ViewingKey {
                    key_type: ViewingKeyType::Custom("Aleo".to_string()),
                    key_data: vec![],
                }),
            })
        } else {
            None
        }
    }

    fn has_private_inputs_or_outputs(&self, transition: &Transition) -> bool {
        let has_private_inputs = transition.inputs.iter().any(|input| {
            matches!(
                input,
                TransitionInput::Private { .. } | TransitionInput::Record { .. }
            )
        });

        let has_private_outputs = transition.outputs.iter().any(|output| {
            matches!(
                output,
                TransitionOutput::Private { .. } | TransitionOutput::Record { .. }
            )
        });

        has_private_inputs || has_private_outputs
    }
}
