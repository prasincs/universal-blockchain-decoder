//! Sui transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for Sui blockchain transactions, transforming them
//! from their native BCS (Binary Canonical Serialization) format into the universal
//! TxIR representation.
//!
//! ## Sui Overview
//!
//! Sui is a Move-based blockchain that uses an object-centric model:
//! - **Object model**: All assets are objects with unique IDs
//! - **Programmable transactions**: Composable commands for complex operations
//! - **BCS encoding**: Binary Canonical Serialization for deterministic encoding
//! - **Multiple signature schemes**: Ed25519, Secp256k1, Secp256r1
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. We implement BCS parsing directly using the
//! `decoder-encodings::bcs` module.
//!
//! ## Transaction Types Supported
//!
//! - ✅ Programmable transactions (most common)
//! - ✅ MoveCall commands
//! - ✅ TransferObjects commands
//! - ✅ SplitCoins / MergeCoins commands
//! - ✅ Publish commands
//! - ✅ Ed25519, Secp256k1, Secp256r1 signatures
//! - ✅ System transactions (ChangeEpoch, Genesis)
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_sui::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_bytes = /* BCS-encoded Sui transaction */;
//!
//! let decoded = SuiDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//!
//! // Access Sui-specific details
//! println!("Sender: {:?}", decoded.sender());
//! println!("Commands: {}", decoded.command_count());
//! ```

use decoder_primitives::prelude::*;

pub mod parsing;
pub mod types;

pub use types::SuiTransaction;

/// Sui chain identity
#[derive(Debug, Clone, Copy)]
pub struct SuiChain;

impl ChainIdentity for SuiChain {
    fn chain_id(&self) -> u64 {
        0 // Sui uses object IDs, not a numeric chain ID
    }

    fn chain_name(&self) -> &str {
        "Sui"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Instruction
    }
}

/// Sui decoder implementing the ChainDecoder trait
///
/// This decoder uses pure Rust BCS parsing to decode Sui transactions
/// without depending on external blockchain libraries in production.
pub struct SuiDecoder;

impl ChainDecoder for SuiDecoder {
    type TxSpecific = SuiTransaction;
    type Chain = SuiChain;

    fn chain() -> Self::Chain {
        SuiChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Parse transaction using BCS
        let transaction = parsing::parse_transaction(raw_bytes)?;

        Ok(transaction)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Sui transaction cannot be empty",
            ));
        }

        // Minimum size check: variant (1) + sender (32) + gas data (~50) ≈ 100 bytes
        if raw_bytes.len() < 100 {
            return Err(DecoderError::invalid_structure(format!(
                "Sui transaction too small: {} bytes (minimum ~100 bytes)",
                raw_bytes.len()
            )));
        }

        // Maximum size check (Sui has a max transaction size limit)
        const MAX_TRANSACTION_SIZE: usize = 128 * 1024; // 128 KB
        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Sui transaction too large: {} bytes (maximum {} bytes)",
                raw_bytes.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for SuiTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let tx_digest = self.digest();

        let metadata = TxMetadata {
            tx_hash: tx_digest,
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!("commands={}", self.command_count()),
        };

        // Extract signatures and public keys
        let (signatures, public_keys, signature_scheme) = self.extract_authorization();

        let authorization = AuthorizationPackage {
            signatures,
            public_keys,
            signature_scheme,
        };

        // Build operations based on commands
        let operations = self.build_operations();

        // Build state deltas
        let state_deltas = self.build_state_deltas();

        Ok(TxIR::new(
            &SuiChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Validate sender address is not all zeros
        if self.sender().iter().all(|&b| b == 0) {
            return Err(DecoderError::invalid_structure("Sender address is zero"));
        }

        // Validate gas parameters
        if self.gas_budget() == 0 {
            return Err(DecoderError::invalid_structure("Gas budget is zero"));
        }

        if self.gas_price() == 0 {
            return Err(DecoderError::invalid_structure("Gas price is zero"));
        }

        // Validate at least one signature
        if self.signatures.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Transaction has no signatures",
            ));
        }

        Ok(())
    }
}

impl SuiTransaction {
    /// Extract authorization data from signatures
    fn extract_authorization(&self) -> (Vec<Signature>, Vec<PublicKey>, SignatureScheme) {
        let mut sigs = Vec::new();
        let mut keys = Vec::new();
        let mut scheme = SignatureScheme::EdDsa;

        for (i, sig) in self.signatures.iter().enumerate() {
            match sig {
                types::SuiSignature::Ed25519 {
                    signature,
                    public_key,
                } => {
                    sigs.push(Signature {
                        data: signature.to_vec(),
                        key_index: i,
                        metadata: None,
                    });
                    keys.push(PublicKey {
                        data: public_key.to_vec(),
                        key_type: KeyType::Ed25519,
                    });
                    scheme = SignatureScheme::EdDsa;
                }
                types::SuiSignature::Secp256k1 {
                    signature,
                    public_key,
                } => {
                    sigs.push(Signature {
                        data: signature.clone(),
                        key_index: i,
                        metadata: None,
                    });
                    keys.push(PublicKey {
                        data: public_key.clone(),
                        key_type: KeyType::Secp256k1,
                    });
                    scheme = SignatureScheme::Ecdsa;
                }
                types::SuiSignature::Secp256r1 {
                    signature,
                    public_key,
                } => {
                    sigs.push(Signature {
                        data: signature.clone(),
                        key_index: i,
                        metadata: None,
                    });
                    keys.push(PublicKey {
                        data: public_key.clone(),
                        key_type: KeyType::P256,
                    });
                    scheme = SignatureScheme::Ecdsa;
                }
            }
        }

        (sigs, keys, scheme)
    }

    /// Build operations from transaction commands
    fn build_operations(&self) -> Vec<Operation> {
        let mut operations = Vec::new();

        if let Some(pt) = self.programmable_transaction() {
            for command in &pt.commands {
                match command {
                    types::Command::MoveCall {
                        package,
                        module,
                        function,
                        ..
                    } => {
                        let method_str = format!("{}::{}", module, function);
                        operations.push(Operation::ContractCall(ContractCall {
                            contract: Address {
                                bytes: package.to_vec(),
                                human_readable: Some(universal_decoder_core::hex::encode(package)),
                            },
                            method: method_str.as_bytes().to_vec(),
                            data: vec![],
                            value: None,
                            resource_limits: ResourceLimits {
                                max_units: self.gas_budget(),
                                unit_price: self.gas_price(),
                                resource_type: ResourceType::Gas,
                            },
                        }));
                    }
                    types::Command::TransferObjects { .. } => {
                        operations.push(Operation::Transfer(Transfer {
                            from: Address {
                                bytes: self.sender().to_vec(),
                                human_readable: Some(universal_decoder_core::hex::encode(
                                    self.sender(),
                                )),
                            },
                            to: Address {
                                bytes: vec![],
                                human_readable: None,
                            },
                            amount: Amount {
                                value: 0,
                                decimals: 9,
                            },
                            asset: AssetId::Native,
                        }));
                    }
                    types::Command::Publish { modules, .. } => {
                        operations.push(Operation::ContractDeploy(ContractDeploy {
                            bytecode: modules.concat(),
                            constructor_args: vec![],
                            value: Amount {
                                value: 0,
                                decimals: 9,
                            },
                        }));
                    }
                    _ => {
                        operations.push(Operation::Generic(GenericOperation {
                            op_type: command.command_type().to_string(),
                            data: vec![],
                            metadata: format!("{} command", command.command_type()),
                        }));
                    }
                }
            }
        }

        operations
    }

    /// Build state deltas from transaction
    fn build_state_deltas(&self) -> StateDeltas {
        let sender_address = Address {
            bytes: self.sender().to_vec(),
            human_readable: Some(universal_decoder_core::hex::encode(self.sender())),
        };

        // Calculate gas cost as i128 (negative for spending)
        let gas_cost = (self.gas_budget() * self.gas_price()) as i128;

        // Create account change for gas payment
        let account_changes = vec![AccountChange {
            address: sender_address,
            nonce: None,
            balance_change: -gas_cost,
            storage_changes: vec![],
        }];

        StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = SuiDecoder::chain();
        assert_eq!(chain.chain_id(), 0);
        assert_eq!(chain.chain_name(), "Sui");
        assert!(matches!(chain.chain_family(), ChainFamily::Instruction));
    }

    #[test]
    fn test_validate_format_empty() {
        let result = SuiDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_too_small() {
        let data = vec![0u8; 50];
        let result = SuiDecoder::validate_format(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_too_large() {
        let data = vec![0u8; 200_000];
        let result = SuiDecoder::validate_format(&data);
        assert!(result.is_err());
    }
}
