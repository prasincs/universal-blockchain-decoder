//! TRON transaction decoder
//!
//! This decoder implements support for TRON blockchain transactions using pure Rust.
//! It parses protobuf-encoded TRON transactions and converts them to the universal TxIR format.
//!
//! ## Features
//! - Supports all major TRON contract types (Transfer, TriggerSmartContract, Freeze, etc.)
//! - Base58check address encoding/decoding
//! - Transaction hash computation (SHA-256 of raw_data)
//! - Pure Rust implementation (no external blockchain libraries)
//!
//! ## Example
//! ```no_run
//! use decoder_tron::TronDecoder;
//! use decoder_primitives::prelude::*;
//!
//! let tx_bytes = hex::decode("0a02...").unwrap();
//! let tx = TronDecoder::decode(&tx_bytes).unwrap();
//! let tx_ir = tx.canonicalize().unwrap();
//! ```
use decoder_primitives::prelude::*;
use prost::Message;

// Re-export hex for tests and internal use
pub(crate) use universal_decoder_core::hex;

pub mod hashing;
pub mod operations;
pub mod types;

pub use types::{ContractType, Transaction as TronTransaction};

/// TRON chain identity (mainnet)
#[derive(Debug, Clone, Copy)]
pub struct TronChain;

impl ChainIdentity for TronChain {
    fn chain_id(&self) -> u64 {
        195 // TRON mainnet chain ID
    }

    fn chain_name(&self) -> &str {
        "Tron"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Wrapper around TRON transaction for decoding
#[derive(Debug, Clone)]
pub struct TronTransactionWrapper {
    pub raw_bytes: Vec<u8>,
    pub transaction: TronTransaction,
}

/// TRON decoder
pub struct TronDecoder;

impl ChainDecoder for TronDecoder {
    type TxSpecific = TronTransactionWrapper;
    type Chain = TronChain;

    fn chain() -> Self::Chain {
        TronChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;

        // Decode protobuf transaction
        let transaction = TronTransaction::decode(raw_bytes).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to decode TRON transaction: {}", e))
        })?;

        // Validate basic structure
        if transaction.raw_data.is_none() {
            return Err(DecoderError::invalid_structure(
                "TRON transaction missing raw_data",
            ));
        }

        Ok(TronTransactionWrapper {
            raw_bytes: raw_bytes.to_vec(),
            transaction,
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "TRON transaction cannot be empty",
            ));
        }

        // TRON transactions are typically 100 bytes to 1MB
        if raw_bytes.len() > 1_000_000 {
            return Err(DecoderError::invalid_structure(format!(
                "TRON transaction too large: {} bytes (max 1MB)",
                raw_bytes.len()
            )));
        }

        // Try to parse as protobuf (quick validation)
        TronTransaction::decode(raw_bytes).map_err(|e| {
            DecoderError::invalid_structure(format!("Invalid TRON protobuf: {}", e))
        })?;

        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for TronTransactionWrapper {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let raw_data = self
            .transaction
            .raw_data
            .as_ref()
            .ok_or_else(|| DecoderError::invalid_structure("Missing raw_data"))?;

        // Compute transaction hash (SHA-256 of raw_data)
        let raw_data_bytes = raw_data.encode_to_vec();
        let tx_hash = hashing::compute_tx_hash(&raw_data_bytes);

        // Build metadata
        let metadata = TxMetadata {
            tx_hash,
            block_height: None,
            timestamp: if raw_data.timestamp > 0 {
                Some(raw_data.timestamp as u64)
            } else {
                None
            },
            size: self.raw_bytes.len(),
            extra: format!(
                "expiration: {}, fee_limit: {}, contracts: {}",
                raw_data.expiration,
                raw_data.fee_limit,
                raw_data.contract.len()
            ),
        };

        // Build authorization
        let authorization = AuthorizationPackage {
            signatures: self
                .transaction
                .signature
                .iter()
                .enumerate()
                .map(|(idx, sig)| Signature {
                    data: sig.clone(),
                    key_index: idx,
                    metadata: Some(format!("0x{}", hex::encode(sig))),
                })
                .collect(),
            public_keys: vec![], // Public keys are recovered from signatures in TRON
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Parse operations from contracts
        let operations = operations::parse_operations(&raw_data.contract)?;

        // Parse state deltas
        let state_deltas = operations::parse_state_deltas(&raw_data.contract)?;

        Ok(TxIR::new(
            &TronChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        let raw_data = self
            .transaction
            .raw_data
            .as_ref()
            .ok_or_else(|| DecoderError::invalid_structure("Missing raw_data"))?;

        // Validate contracts exist
        if raw_data.contract.is_empty() {
            return Err(DecoderError::invalid_structure(
                "TRON transaction must have at least one contract",
            ));
        }

        // Validate ref_block_bytes
        if raw_data.ref_block_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Missing ref_block_bytes"));
        }

        // Validate timestamp
        if raw_data.timestamp == 0 {
            return Err(DecoderError::invalid_structure("Invalid timestamp"));
        }

        // Validate expiration
        if raw_data.expiration == 0 {
            return Err(DecoderError::invalid_structure("Invalid expiration"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = TronDecoder::chain();
        assert_eq!(chain.chain_id(), 195);
        assert_eq!(chain.chain_name(), "Tron");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_empty() {
        let result = TronDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_too_large() {
        let large_data = vec![0u8; 2_000_000];
        let result = TronDecoder::validate_format(&large_data);
        assert!(result.is_err());
    }
}
