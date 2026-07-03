//! Aptos transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for Aptos blockchain transactions, transforming them
//! from their native BCS (Binary Canonical Serialization) format into the universal
//! TxIR representation.
//!
//! ## Aptos Overview
//!
//! Aptos is a Move-based blockchain that uses:
//! - **Account model**: Transactions are sent from accounts with sequence numbers
//! - **BCS encoding**: Binary Canonical Serialization for deterministic encoding
//! - **Ed25519 signatures**: Primary signature scheme (also supports multi-sig)
//! - **Entry functions**: Main transaction type for calling Move functions
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. We implement BCS parsing directly using the
//! `decoder-encodings::bcs` module.
//!
//! ## Transaction Types Supported
//!
//! - ✅ Entry function calls (most common)
//! - ✅ Script transactions
//! - ✅ Multisig transactions
//! - ✅ Ed25519 signatures
//! - ✅ Multi-Ed25519 signatures
//! - ✅ Multi-agent transactions
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_aptos::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_bytes = /* BCS-encoded Aptos transaction */;
//!
//! let decoded = AptosDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//!
//! // Access Aptos-specific details
//! println!("Sender: {:?}", decoded.sender());
//! println!("Sequence number: {}", decoded.sequence_number());
//! ```

use decoder_primitives::prelude::*;

pub mod parsing;
pub mod types;

pub use types::AptosTransaction;

/// Aptos chain identity
#[derive(Debug, Clone, Copy)]
pub struct AptosChain;

impl ChainIdentity for AptosChain {
    fn chain_id(&self) -> u64 {
        1 // Aptos mainnet chain ID
    }

    fn chain_name(&self) -> &str {
        "Aptos"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Aptos decoder implementing the ChainDecoder trait
///
/// This decoder uses pure Rust BCS parsing to decode Aptos transactions
/// without depending on external blockchain libraries in production.
pub struct AptosDecoder;

impl ChainDecoder for AptosDecoder {
    type TxSpecific = AptosTransaction;
    type Chain = AptosChain;

    fn chain() -> Self::Chain {
        AptosChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Parse signed transaction using BCS
        let signed_txn = parsing::parse_signed_transaction(raw_bytes)?;

        Ok(AptosTransaction {
            signed_txn,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Aptos transaction cannot be empty",
            ));
        }

        // Minimum size check: 32 (address) + 8 (seq) + 1 (payload variant) + ... ≈ 50 bytes
        if raw_bytes.len() < 50 {
            return Err(DecoderError::invalid_structure(format!(
                "Aptos transaction too small: {} bytes (minimum ~50 bytes)",
                raw_bytes.len()
            )));
        }

        // Maximum size check (Aptos has a max transaction size limit)
        const MAX_TRANSACTION_SIZE: usize = 64 * 1024; // 64 KB
        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Aptos transaction too large: {} bytes (maximum {} bytes)",
                raw_bytes.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
    }
}

impl ChainEncoder for AptosTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for AptosTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let tx_hash = self.hash();

        let metadata = TxMetadata {
            tx_hash: tx_hash.clone(),
            block_height: None,
            timestamp: Some(self.signed_txn.raw_txn.expiration_timestamp_secs),
            size: self.raw_bytes.len(),
            extra: format!("chain_id={}", self.chain_id()),
        };

        // Extract signatures and public keys
        let (signatures, public_keys, signature_scheme) = match &self.signed_txn.authenticator {
            types::TransactionAuthenticator::Ed25519 {
                public_key,
                signature,
            } => (
                vec![Signature {
                    data: signature.to_vec(),
                    key_index: 0,
                    metadata: None,
                }],
                vec![PublicKey {
                    data: public_key.to_vec(),
                    key_type: KeyType::Ed25519,
                }],
                SignatureScheme::EdDsa,
            ),
            types::TransactionAuthenticator::MultiEd25519 {
                public_keys,
                signatures,
                ..
            } => {
                let sigs: Vec<_> = signatures
                    .iter()
                    .enumerate()
                    .map(|(i, sig)| Signature {
                        data: sig.to_vec(),
                        key_index: i,
                        metadata: None,
                    })
                    .collect();

                let keys: Vec<_> = public_keys
                    .iter()
                    .map(|key| PublicKey {
                        data: key.to_vec(),
                        key_type: KeyType::Ed25519,
                    })
                    .collect();

                (sigs, keys, SignatureScheme::EdDsa)
            }
            types::TransactionAuthenticator::MultiAgent { sender, .. } => {
                // For multi-agent, use the sender authenticator
                if let types::TransactionAuthenticator::Ed25519 {
                    public_key,
                    signature,
                } = sender.as_ref()
                {
                    (
                        vec![Signature {
                            data: signature.to_vec(),
                            key_index: 0,
                            metadata: None,
                        }],
                        vec![PublicKey {
                            data: public_key.to_vec(),
                            key_type: KeyType::Ed25519,
                        }],
                        SignatureScheme::EdDsa,
                    )
                } else {
                    (vec![], vec![], SignatureScheme::EdDsa)
                }
            }
        };

        let authorization = AuthorizationPackage {
            signatures,
            public_keys,
            signature_scheme,
        };

        // Build operations based on payload type
        let operations = self.build_operations();

        // Build state deltas
        let state_deltas = self.build_state_deltas();

        Ok(TxIR::new(
            &AptosChain,
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
        if self.max_gas_amount() == 0 {
            return Err(DecoderError::invalid_structure("Max gas amount is zero"));
        }

        if self.gas_unit_price() == 0 {
            return Err(DecoderError::invalid_structure("Gas unit price is zero"));
        }

        Ok(())
    }
}

impl AptosTransaction {
    /// Build operations from transaction payload
    fn build_operations(&self) -> Vec<Operation> {
        let mut operations = Vec::new();

        match &self.signed_txn.raw_txn.payload {
            types::TransactionPayload::EntryFunction {
                module,
                function,
                type_args: _,
                args,
            } => {
                // Create function selector from module and function name
                let method_str = format!("{}::{}", module.name, function);
                let method_bytes = method_str.as_bytes().to_vec();

                // Encode arguments
                let mut data = Vec::new();
                for arg in args {
                    data.extend_from_slice(arg);
                }

                // Create a contract call operation
                operations.push(Operation::ContractCall(ContractCall {
                    contract: Address {
                        bytes: module.address.to_vec(),
                        human_readable: Some(universal_decoder_core::hex::encode(module.address)),
                    },
                    method: method_bytes,
                    data,
                    value: None,
                    resource_limits: ResourceLimits {
                        max_units: self.max_gas_amount(),
                        unit_price: self.gas_unit_price(),
                        resource_type: ResourceType::Gas,
                    },
                }));
            }
            types::TransactionPayload::Script { code, .. } => {
                // For scripts, create a generic operation
                operations.push(Operation::Generic(GenericOperation {
                    op_type: "Script".to_string(),
                    data: code.clone(),
                    metadata: "Move script execution".to_string(),
                }));
            }
            types::TransactionPayload::Multisig { .. } => {
                // For multisig, create a generic operation
                operations.push(Operation::Generic(GenericOperation {
                    op_type: "Multisig".to_string(),
                    data: vec![],
                    metadata: "Multisig transaction".to_string(),
                }));
            }
        }

        operations
    }

    /// Build state deltas from transaction
    fn build_state_deltas(&self) -> StateDeltas {
        // Gas-cost balance guesses are NOT byte-derivable state effects and
        // were removed from TxIR (docs/CONCEPTS_REVIEW.md C1).
        StateDeltas {
            inputs: vec![],
            outputs: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = AptosDecoder::chain();
        assert_eq!(chain.chain_id(), 1);
        assert_eq!(chain.chain_name(), "Aptos");
        assert!(matches!(chain.chain_family(), ChainFamily::Account));
    }

    #[test]
    fn test_validate_format_empty() {
        let result = AptosDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_too_small() {
        let data = vec![0u8; 40];
        let result = AptosDecoder::validate_format(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_too_large() {
        let data = vec![0u8; 100_000];
        let result = AptosDecoder::validate_format(&data);
        assert!(result.is_err());
    }
}
