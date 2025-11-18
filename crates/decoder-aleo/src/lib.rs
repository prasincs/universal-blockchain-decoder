//! # Aleo Blockchain Transaction Decoder
//!
//! Pure Rust decoder for Aleo blockchain transactions, supporting zero-knowledge
//! transactions with Leo VM programs and privacy-preserving computations.
//!
//! ## Overview
//!
//! Aleo is a privacy-focused blockchain using zero-knowledge proofs (zkSNARKs) for
//! decentralized private computations. This decoder supports:
//!
//! - **Execution Transactions**: Program execution with transitions
//! - **Deployment Transactions**: Program deployment to the blockchain
//! - **Fee Transactions**: Network fee payments
//! - **Records**: UTXO-like encrypted state (inputs/outputs)
//! - **Transitions**: Individual state changes within a transaction
//!
//! ## Cryptographic Primitives
//!
//! - **BLS12-377**: Pairing-friendly elliptic curve for zkSNARKs
//! - **Poseidon Hash**: ZK-friendly hash function (BLS12-377 variant)
//! - **Varuna**: zk-SNARK proof system
//! - **Account Model**: Address-based with encrypted records
//!
//! ## Transaction Types
//!
//! 1. **Deploy**: Deploy a new Leo program to the blockchain
//! 2. **Execute**: Execute a program function with inputs
//! 3. **Fee**: Pay transaction fees (can be standalone or part of another tx)
//!
//! ## Architecture
//!
//! - `types`: Transaction type definitions (Transaction, Transition, Record)
//! - `parsing`: Binary parsing logic for Aleo serialization format
//! - `error`: Error types
//! - Tests: Mainnet fixtures and property-based tests
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_aleo::AleoDecoder;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_bytes = /* Aleo transaction bytes */;
//! let decoded = AleoDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

// Modules
pub mod error;
pub mod parsing;
pub mod types;

// Re-exports
pub use error::{AleoDecoderError, Result};
pub use types::{
    AleoTransaction, Deployment, Execution, Fee, FinalizeOperation, TransactionType, Transition,
    TransitionInput, TransitionOutput, VerifyingKey,
};

use decoder_primitives::prelude::*;
use std::io::Cursor;

/// Aleo mainnet chain identity
pub use decoder_chains_common::chains::ALEO as AleoChain;

/// Aleo transaction decoder implementing the ChainDecoder trait
///
/// This decoder uses a pure Rust implementation to parse Aleo transactions
/// without depending on the snarkVM library in production (only in dev-dependencies
/// for validation testing).
///
/// ## Supported Features
///
/// - ✅ Transaction parsing (Deploy, Execute, Fee)
/// - ✅ Transition parsing (state changes)
/// - ✅ Record parsing (inputs/outputs)
/// - ✅ Program ID extraction
/// - ✅ Proof structure parsing (not verification)
/// - ⏳ Full zkSNARK proof verification (future work)
pub struct AleoDecoder;

impl ChainDecoder for AleoDecoder {
    type TxSpecific = AleoTransaction;
    type Chain = decoder_chains_common::chains::ChainInfo;

    fn chain() -> Self::Chain {
        decoder_chains_common::chains::ALEO
    }

    fn decode(raw_bytes: &[u8]) -> decoder_primitives::Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        let mut cursor = Cursor::new(raw_bytes);

        // Parse transaction
        parsing::parse_transaction(&mut cursor).map_err(|e| e.into())
    }

    fn validate_format(raw_bytes: &[u8]) -> decoder_primitives::Result<()> {
        // Minimum transaction size: version(1) + type(1) + minimal content
        if raw_bytes.len() < 16 {
            return Err(DecoderError::invalid_structure(format!(
                "Aleo transaction too small: {} bytes (minimum 16)",
                raw_bytes.len()
            )));
        }

        // Maximum transaction size (2MB for programs)
        const MAX_TRANSACTION_SIZE: usize = 2 * 1024 * 1024;
        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Aleo transaction too large: {} bytes (maximum {})",
                raw_bytes.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = AleoDecoder::chain();
        assert_eq!(chain.name, "Aleo");
        assert_eq!(chain.id, 368); // Aleo chain ID (from SLIP-44 or custom)
    }

    #[test]
    fn test_validate_format_too_small() {
        let result = AleoDecoder::validate_format(&[0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_too_large() {
        let large_data = vec![0u8; 3 * 1024 * 1024]; // 3MB
        let result = AleoDecoder::validate_format(&large_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_acceptable_size() {
        let data = vec![0u8; 1024]; // 1KB
        let result = AleoDecoder::validate_format(&data);
        assert!(result.is_ok());
    }
}
