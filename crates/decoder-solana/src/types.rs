//! Solana-specific transaction types
//!
//! This module defines the core types for Solana transactions, messages, and instructions
//! following a pure Rust implementation approach.

use serde::{Deserialize, Serialize};
use std::fmt;
use universal_decoder_core::prelude::*;

/// Solana public key (32-byte Ed25519 public key)
///
/// Note: Using Vec instead of [u8; 32] for better serde compatibility
pub type SolanaPubkey = Vec<u8>;

/// Solana signature (64-byte Ed25519 signature)
///
/// Note: Using Vec instead of [u8; 64] for better serde compatibility
pub type SolanaSignature = Vec<u8>;

/// Solana blockhash (32 bytes)
///
/// Note: Using Vec instead of [u8; 32] for better serde compatibility
pub type SolanaBlockhash = Vec<u8>;

/// Message header containing account metadata
///
/// The header specifies the number of signatures required and
/// identifies which accounts are read-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Number of signatures required for this message.
    /// Also indicates message version (legacy vs versioned).
    pub num_required_signatures: u8,

    /// Number of read-only account keys that require signatures.
    /// These accounts can be read but not written to.
    pub num_readonly_signed_accounts: u8,

    /// Number of read-only account keys that do not require signatures.
    /// These are typically program accounts.
    pub num_readonly_unsigned_accounts: u8,
}

impl MessageHeader {
    /// Total number of writable signed accounts
    pub fn num_writable_signed_accounts(&self) -> u8 {
        self.num_required_signatures
            .saturating_sub(self.num_readonly_signed_accounts)
    }

    /// Total number of signed accounts (writable + readonly)
    pub fn num_signed_accounts(&self) -> u8 {
        self.num_required_signatures
    }
}

/// Compiled instruction in a Solana transaction
///
/// Instructions are "compiled" by referencing accounts and programs
/// by index into the message's account_keys array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledInstruction {
    /// Index into the message's account_keys array indicating the program to execute.
    pub program_id_index: u8,

    /// Ordered indices into the message's account_keys array indicating which
    /// accounts are passed to the program.
    pub accounts: Vec<u8>,

    /// Opaque data passed to the program for execution.
    /// The program interprets this data according to its own format.
    pub data: Vec<u8>,
}

impl CompiledInstruction {
    /// Get the number of accounts this instruction uses
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Get the size of the instruction data
    pub fn data_len(&self) -> usize {
        self.data.len()
    }
}

/// Solana transaction message
///
/// The message contains all the information needed to execute a transaction,
/// except for the signatures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// The message header, identifying signed and read-only accounts.
    pub header: MessageHeader,

    /// All the account keys used by this transaction.
    /// These are referenced by index in the instructions.
    pub account_keys: Vec<SolanaPubkey>,

    /// Recent blockhash to prevent replay attacks.
    /// Transactions with old blockhashes are rejected.
    pub recent_blockhash: SolanaBlockhash,

    /// The instructions to execute in this transaction.
    /// Instructions are executed sequentially and atomically.
    pub instructions: Vec<CompiledInstruction>,
}

impl Message {
    /// Get the number of account keys in this message
    pub fn num_account_keys(&self) -> usize {
        self.account_keys.len()
    }

    /// Get the number of instructions in this message
    pub fn num_instructions(&self) -> usize {
        self.instructions.len()
    }

    /// Check if this message has valid structure
    pub fn is_valid(&self) -> bool {
        // All instruction program_id_index and account indices must be valid
        for instruction in &self.instructions {
            if instruction.program_id_index as usize >= self.account_keys.len() {
                return false;
            }
            for &account_idx in &instruction.accounts {
                if account_idx as usize >= self.account_keys.len() {
                    return false;
                }
            }
        }
        true
    }

    /// Get the program ID for an instruction by resolving the index
    pub fn program_id(&self, instruction: &CompiledInstruction) -> Option<&SolanaPubkey> {
        self.account_keys.get(instruction.program_id_index as usize)
    }

    /// Get account keys referenced by an instruction
    pub fn instruction_accounts(&self, instruction: &CompiledInstruction) -> Vec<&SolanaPubkey> {
        instruction
            .accounts
            .iter()
            .filter_map(|&idx| self.account_keys.get(idx as usize))
            .collect()
    }
}

/// Solana transaction - complete structure with signatures
///
/// This represents a fully-formed Solana transaction that can be submitted to the network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolanaTransaction {
    /// Ed25519 signatures for this transaction.
    /// The number of signatures matches message.header.num_required_signatures.
    pub signatures: Vec<SolanaSignature>,

    /// The transaction message containing instructions and account references.
    pub message: Message,

    /// Raw transaction bytes (for canonical representation)
    pub raw_bytes: Vec<u8>,
}

impl SolanaTransaction {
    /// Get the number of signatures
    pub fn num_signatures(&self) -> usize {
        self.signatures.len()
    }

    /// Get the first signature (transaction ID)
    pub fn signature(&self) -> Option<&SolanaSignature> {
        self.signatures.first()
    }

    /// Check if the transaction structure is valid
    pub fn is_valid(&self) -> bool {
        // Number of signatures must match header
        if self.signatures.len() != self.message.header.num_required_signatures as usize {
            return false;
        }

        // Message must be valid
        self.message.is_valid()
    }

    /// Get all instructions from the message
    pub fn instructions(&self) -> &[CompiledInstruction] {
        &self.message.instructions
    }

    /// Get account keys from the message
    pub fn account_keys(&self) -> &[SolanaPubkey] {
        &self.message.account_keys
    }

    /// Get recent blockhash
    pub fn recent_blockhash(&self) -> &SolanaBlockhash {
        &self.message.recent_blockhash
    }
}

impl fmt::Display for SolanaTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SolanaTransaction {{ signatures: {}, accounts: {}, instructions: {} }}",
            self.num_signatures(),
            self.message.num_account_keys(),
            self.message.num_instructions()
        )
    }
}

/// Implement Canonicalizer to transform Solana transactions into TxIR
impl<'a> Canonicalizer<'a> for SolanaTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Create a SolanaChain instance for chain identification
        // Note: We create it inline to avoid circular dependency issues
        #[derive(Debug, Clone, Copy)]
        struct SolanaChain;
        impl ChainIdentity for SolanaChain {
            fn chain_id(&self) -> u64 {
                101
            }
            fn chain_name(&self) -> &str {
                "Solana"
            }
            fn chain_family(&self) -> ChainFamily {
                ChainFamily::Account
            }
        }

        // Create metadata
        let tx_hash = self.signature().map(|sig| sig.to_vec()).unwrap_or_default();

        let metadata = TxMetadata {
            tx_hash,
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!(
                r#"{{"num_signatures":{},"num_accounts":{},"num_instructions":{},"num_required_signatures":{},"num_readonly_signed":{},"num_readonly_unsigned":{}}}"#,
                self.num_signatures(),
                self.message.num_account_keys(),
                self.message.num_instructions(),
                self.message.header.num_required_signatures,
                self.message.header.num_readonly_signed_accounts,
                self.message.header.num_readonly_unsigned_accounts,
            ),
        };

        // Create authorization package
        let mut authorization = AuthorizationPackage {
            signatures: Vec::new(),
            public_keys: Vec::new(),
            signature_scheme: SignatureScheme::EdDsa, // Solana uses Ed25519
        };

        // Add signatures
        for (idx, signature) in self.signatures.iter().enumerate() {
            authorization.signatures.push(Signature {
                data: signature.clone(),
                key_index: idx,
                metadata: None,
            });
        }

        // Add public keys from account keys (signed accounts come first)
        let num_signed = self.message.header.num_required_signatures as usize;
        for i in 0..num_signed {
            if let Some(pubkey) = self.message.account_keys.get(i) {
                authorization.public_keys.push(PublicKey {
                    data: pubkey.clone(),
                    key_type: KeyType::Ed25519, // Solana uses Ed25519
                });
            }
        }

        // Create operations from instructions
        let mut operations = Vec::new();
        for instruction in self.message.instructions.iter() {
            // Get program ID
            let program_id = self
                .message
                .program_id(instruction)
                .map(|pk| pk.to_vec())
                .unwrap_or_default();

            // Solana instructions are contract/program calls
            operations.push(Operation::ContractCall(ContractCall {
                contract: Address {
                    bytes: program_id,
                    human_readable: None,
                },
                method: vec![], // Solana doesn't have explicit method selectors in the transaction
                data: instruction.data.clone(),
                value: None, // Solana transfers are done via system program instructions
                resource_limits: ResourceLimits {
                    max_units: 200_000, // Default compute unit limit for Solana instructions
                    unit_price: 0,      // We don't parse priority fees in minimal decoder
                    resource_type: ResourceType::ComputeUnits,
                },
            }));
        }

        // Create state deltas (account-based model)
        let mut state_deltas = StateDeltas {
            inputs: Vec::new(),
            outputs: Vec::new(),
            account_changes: Vec::new(),
        };

        // Add account changes for all writable accounts
        let num_writable = self.message.header.num_writable_signed_accounts() as usize;
        for i in 0..num_writable {
            if let Some(pubkey) = self.message.account_keys.get(i) {
                state_deltas.account_changes.push(AccountChange {
                    address: Address {
                        bytes: pubkey.to_vec(),
                        human_readable: None,
                    },
                    nonce: None,       // We don't parse nonces in minimal decoder
                    balance_change: 0, // We don't parse balance changes in minimal decoder
                    storage_changes: vec![],
                });
            }
        }

        Ok(TxIR::new(
            &SolanaChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        if !self.is_valid() {
            return Err(DecoderError::invalid_structure(
                "Solana transaction failed validation",
            ));
        }
        Ok(())
    }
}

/// Implement TxHashable for canonical byte representation
impl TxHashable for SolanaTransaction {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        // For Solana, the canonical bytes are just the raw transaction bytes
        self.raw_bytes.clone()
    }

    fn compute_hash(&self) -> Vec<u8> {
        // Solana uses the first signature as the transaction ID
        self.signature().map(|sig| sig.to_vec()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_header() {
        let header = MessageHeader {
            num_required_signatures: 2,
            num_readonly_signed_accounts: 1,
            num_readonly_unsigned_accounts: 3,
        };

        assert_eq!(header.num_signed_accounts(), 2);
        assert_eq!(header.num_writable_signed_accounts(), 1); // 2 - 1 = 1
    }

    #[test]
    fn test_compiled_instruction() {
        let instruction = CompiledInstruction {
            program_id_index: 5,
            accounts: vec![0, 1, 2],
            data: vec![1, 2, 3, 4],
        };

        assert_eq!(instruction.account_count(), 3);
        assert_eq!(instruction.data_len(), 4);
    }

    #[test]
    fn test_message_validation() {
        let valid_message = Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 0,
            },
            account_keys: vec![vec![0; 32], vec![1; 32], vec![2; 32]],
            recent_blockhash: vec![0; 32],
            instructions: vec![CompiledInstruction {
                program_id_index: 2,
                accounts: vec![0, 1],
                data: vec![],
            }],
        };

        assert!(valid_message.is_valid());

        // Invalid: program_id_index out of bounds
        let invalid_message = Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 0,
            },
            account_keys: vec![vec![0; 32], vec![1; 32]],
            recent_blockhash: vec![0; 32],
            instructions: vec![CompiledInstruction {
                program_id_index: 5, // Out of bounds!
                accounts: vec![0],
                data: vec![],
            }],
        };

        assert!(!invalid_message.is_valid());
    }

    #[test]
    fn test_transaction_validation() {
        let message = Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 0,
            },
            account_keys: vec![vec![0; 32]],
            recent_blockhash: vec![0; 32],
            instructions: vec![],
        };

        let valid_tx = SolanaTransaction {
            signatures: vec![vec![0; 64]],
            message: message.clone(),
            raw_bytes: vec![],
        };

        assert!(valid_tx.is_valid());

        // Invalid: wrong number of signatures
        let invalid_tx = SolanaTransaction {
            signatures: vec![vec![0; 64], vec![1; 64]], // 2 signatures but header says 1
            message,
            raw_bytes: vec![],
        };

        assert!(!invalid_tx.is_valid());
    }
}
