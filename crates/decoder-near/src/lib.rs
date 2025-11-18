//! NEAR transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for NEAR Protocol transactions, transforming them
//! from their native Borsh-encoded format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. The `near-primitives` crate may be used only in
//! `dev-dependencies` for validation testing.
//!
//! ## Transaction Format Support
//!
//! - ✅ SignedTransaction parsing (signature + transaction)
//! - ✅ Ed25519 signature extraction
//! - ✅ Action parsing (Transfer, FunctionCall, CreateAccount, etc.)
//! - ✅ Account-based state changes
//! - 🚧 Full Borsh deserialization (simplified for initial version)
//!
//! ## NEAR Protocol Overview
//!
//! NEAR is an account-based blockchain that uses:
//! - **Borsh serialization** for deterministic encoding
//! - **Ed25519 signatures** for authentication
//! - **Named accounts** (e.g., "alice.near") instead of hex addresses
//! - **Actions** instead of raw transactions (CreateAccount, Transfer, FunctionCall, etc.)
//! - **Gas-based execution model** with yoctoNEAR (10^-24 NEAR) as the smallest unit
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_near::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_bytes = &[...]; // NEAR transaction bytes (Borsh-encoded)
//!
//! let decoded = NearDecoder::decode(tx_bytes)?;
//!
//! // Access the parsed transaction
//! println!("Signer: {}", decoded.signed_tx.signer_id());
//! println!("Receiver: {}", decoded.signed_tx.receiver_id());
//! println!("Actions: {}", decoded.signed_tx.num_actions());
//! ```

use decoder_primitives::prelude::*;

pub mod parsing;
pub mod types;

use parsing::parse_signed_transaction;
pub use types::{Action, KeyType, SignedTransaction, Transaction};

/// NEAR Protocol chain identity
///
/// This type implements `ChainIdentity` and is used to identify NEAR transactions
/// in the universal decoder system.
#[derive(Debug, Clone, Copy)]
pub struct NearChain;

impl ChainIdentity for NearChain {
    fn chain_id(&self) -> u64 {
        // NEAR Protocol chain ID
        397
    }

    fn chain_name(&self) -> &str {
        "NEAR"
    }

    fn chain_family(&self) -> ChainFamily {
        // NEAR uses an account-based model (like Ethereum)
        ChainFamily::Account
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet")
    }

    fn metadata(&self) -> Option<String> {
        Some(r#"{"consensus":"PoS","sharding":true,"vm":"WASM"}"#.to_string())
    }
}

/// Parsed NEAR transaction
///
/// This structure holds the parsed SignedTransaction along with the raw bytes.
/// It implements `Canonicalizer` to convert to the universal TxIR format.
#[derive(Debug, Clone)]
pub struct NearTransaction {
    /// The parsed signed transaction
    pub signed_tx: SignedTransaction,

    /// Original raw bytes
    pub raw_bytes: Vec<u8>,
}

/// NEAR decoder implementing the ChainDecoder trait
///
/// This decoder uses a pure Rust implementation to parse NEAR transactions
/// without depending on external blockchain libraries in production.
pub struct NearDecoder;

impl ChainDecoder for NearDecoder {
    type TxSpecific = NearTransaction;
    type Chain = NearChain;

    fn chain() -> Self::Chain {
        NearChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Parse the signed transaction
        let signed_tx = parse_signed_transaction(raw_bytes)?;

        Ok(NearTransaction {
            signed_tx,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "NEAR transaction cannot be empty",
            ));
        }

        // NEAR transactions must be at least 64 bytes (for signature)
        if raw_bytes.len() < 64 {
            return Err(DecoderError::invalid_structure(
                "NEAR transaction too short (minimum 64 bytes for signature)",
            ));
        }

        Ok(())
    }
}

impl ChainEncoder for NearTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for NearTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let tx = &self.signed_tx;
        let inner_tx = &tx.transaction;

        // Compute transaction hash (SHA-256 of raw bytes)
        let tx_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&self.raw_bytes);
            hasher.finalize().to_vec()
        };

        // Build metadata
        let metadata = TxMetadata {
            tx_hash,
            block_height: None, // Not available from transaction itself
            timestamp: None,    // Not available from transaction itself
            size: self.raw_bytes.len(),
            extra: serde_json::json!({
                "signer_id": inner_tx.signer_id,
                "receiver_id": inner_tx.receiver_id,
                "nonce": inner_tx.nonce,
                "block_hash": universal_decoder_core::hex::encode(inner_tx.block_hash),
            })
            .to_string(),
        };

        // Build authorization package (Ed25519 signature)
        let authorization = AuthorizationPackage {
            signatures: vec![Signature {
                data: tx.signature.clone(),
                key_index: 0,
                metadata: None,
            }],
            public_keys: vec![PublicKey {
                data: inner_tx.public_key.data.clone(),
                key_type: decoder_primitives::KeyType::Ed25519,
            }],
            signature_scheme: SignatureScheme::EdDsa, // NEAR uses Ed25519
        };

        // Map NEAR actions to universal operations
        let operations = self.map_actions_to_operations()?;

        // Build state deltas (account changes)
        let state_deltas = self.build_state_deltas()?;

        Ok(TxIR::new(
            &NearChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        let tx = &self.signed_tx;

        // Validate signature length (Ed25519 = 64 bytes)
        if tx.signature.len() != 64 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid signature length: expected 64, got {}",
                tx.signature.len()
            )));
        }

        // Validate public key length (Ed25519 = 32 bytes)
        if tx.transaction.public_key.data.len() != 32 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid public key length: expected 32, got {}",
                tx.transaction.public_key.data.len()
            )));
        }

        // Validate account IDs are not empty
        if tx.transaction.signer_id.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Signer account ID cannot be empty",
            ));
        }

        if tx.transaction.receiver_id.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Receiver account ID cannot be empty",
            ));
        }

        // Validate at least one action
        if tx.transaction.actions.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Transaction must have at least one action",
            ));
        }

        Ok(())
    }
}

impl NearTransaction {
    /// Map NEAR actions to universal operations
    fn map_actions_to_operations(&self) -> Result<Vec<Operation>> {
        let tx = &self.signed_tx.transaction;
        let mut operations = Vec::new();

        for action in &tx.actions {
            let op = match action {
                Action::Transfer(transfer) => Operation::Transfer(Transfer {
                    from: Address {
                        bytes: tx.signer_id.clone().into_bytes(),
                        human_readable: Some(tx.signer_id.clone()),
                    },
                    to: Address {
                        bytes: tx.receiver_id.clone().into_bytes(),
                        human_readable: Some(tx.receiver_id.clone()),
                    },
                    amount: Amount::new(transfer.deposit, 24), // yoctoNEAR = 10^-24
                    asset: AssetId::Native,
                }),
                Action::FunctionCall(call) => Operation::ContractCall(ContractCall {
                    contract: Address {
                        bytes: tx.receiver_id.clone().into_bytes(),
                        human_readable: Some(tx.receiver_id.clone()),
                    },
                    method: call.method_name.clone().into_bytes(),
                    data: call.args.clone(),
                    value: if call.deposit > 0 {
                        Some(Amount::new(call.deposit, 24))
                    } else {
                        None
                    },
                    resource_limits: ResourceLimits {
                        max_units: call.gas,
                        unit_price: 0,
                        resource_type: ResourceType::Gas,
                    },
                }),
                Action::DeployContract(deploy) => Operation::ContractDeploy(ContractDeploy {
                    bytecode: deploy.code.clone(),
                    constructor_args: vec![],
                    value: Amount::new(0, 24), // No value attached to deployment by default
                }),
                Action::CreateAccount(_) => Operation::Generic(GenericOperation {
                    op_type: "CreateAccount".to_string(),
                    data: vec![],
                    metadata: serde_json::json!({
                        "account_id": tx.receiver_id,
                    })
                    .to_string(),
                }),
                Action::Stake(stake) => Operation::Stake(Stake {
                    validator: Address {
                        bytes: tx.receiver_id.clone().into_bytes(),
                        human_readable: Some(tx.receiver_id.clone()),
                    },
                    amount: Amount::new(stake.stake, 24),
                    operation_type: StakeOperationType::Delegate,
                }),
                Action::AddKey(_) | Action::DeleteKey(_) | Action::DeleteAccount(_) => {
                    // Map other actions as generic operations
                    let op_name = format!("{:?}", action)
                        .split('(')
                        .next()
                        .unwrap_or("Unknown")
                        .to_string();
                    Operation::Generic(GenericOperation {
                        op_type: op_name,
                        data: vec![],
                        metadata: format!("{:?}", action),
                    })
                }
            };

            operations.push(op);
        }

        Ok(operations)
    }

    /// Build state deltas (account changes)
    fn build_state_deltas(&self) -> Result<StateDeltas> {
        let tx = &self.signed_tx.transaction;

        // Calculate total balance change (sum of all transfers/deposits)
        let total_sent = self.signed_tx.total_transfer_amount();

        let account_changes = vec![
            // Signer account (sender)
            AccountChange {
                address: Address {
                    bytes: tx.signer_id.clone().into_bytes(),
                    human_readable: Some(tx.signer_id.clone()),
                },
                nonce: Some(tx.nonce),
                balance_change: -(total_sent as i128), // Negative for sender
                storage_changes: vec![],
            },
            // Receiver account
            AccountChange {
                address: Address {
                    bytes: tx.receiver_id.clone().into_bytes(),
                    human_readable: Some(tx.receiver_id.clone()),
                },
                nonce: None,
                balance_change: total_sent as i128, // Positive for receiver
                storage_changes: vec![],
            },
        ];

        Ok(StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = NearDecoder::chain();
        assert_eq!(chain.chain_id(), 397);
        assert_eq!(chain.chain_name(), "NEAR");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
        assert_eq!(chain.network(), Some("mainnet"));
    }

    #[test]
    fn test_validate_format_empty() {
        let result = NearDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_too_short() {
        let bytes = vec![1, 2, 3]; // Less than 64 bytes
        let result = NearDecoder::validate_format(&bytes);
        assert!(result.is_err());
    }
}
