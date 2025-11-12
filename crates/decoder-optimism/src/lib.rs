//! Optimism transaction decoder
//!
//! This module provides a decoder for Optimism transactions, transforming them
//! from their native RLP format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! Optimism is EVM-compatible and uses the **exact same transaction format as Ethereum**.
//! This decoder **reuses the Ethereum decoder** with Optimism-specific chain ID validation.
//!
//! ## Transaction Format
//!
//! - RLP-encoded (identical to Ethereum)
//! - EIP-2718 transaction types (legacy, EIP-2930, EIP-1559)
//! - Chain ID: 10
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_optimism::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "f86c...";
//! let tx_bytes = hex::decode(tx_hex)?;
//!
//! let decoded = OptimismDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

use decoder_primitives::prelude::*;
use decoder_ethereum::{EthereumDecoder, types::EthereumTransaction};

/// Optimism chain identity
#[derive(Debug, Clone, Copy)]
pub struct OptimismChain;

impl ChainIdentity for OptimismChain {
    fn chain_id(&self) -> u64 {
        10
    }

    fn chain_name(&self) -> &str {
        "Optimism"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Optimism decoder implementing the ChainDecoder trait
///
/// **Reuses Ethereum decoder** with Optimism-specific chain ID validation.
pub struct OptimismDecoder;

impl ChainDecoder for OptimismDecoder {
    type TxSpecific = EthereumTransaction;  // Reuse Ethereum transaction type
    type Chain = OptimismChain;

    fn chain() -> Self::Chain {
        OptimismChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Decode using Ethereum decoder (same RLP format)
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID is for Optimism
        if let Some(chain_id) = tx.chain_id {
            if chain_id != 10 {
                return Err(DecoderError::invalid_structure(
                    format!("Invalid Optimism chain ID: {} (expected 10)", chain_id)
                ));
            }
        }

        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Use Ethereum's validation (same format)
        EthereumDecoder::validate_format(raw_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = OptimismDecoder::chain();
        assert_eq!(chain.chain_id(), 10);
        assert_eq!(chain.chain_name(), "Optimism");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(OptimismDecoder::validate_format(&[]).is_err());

        // Too small should fail (minimum is 5 bytes for Ethereum)
        assert!(OptimismDecoder::validate_format(&[0x01]).is_err());
        assert!(OptimismDecoder::validate_format(&[0x01, 0x02, 0x03, 0x04]).is_err());

        // Valid minimum length should pass basic validation
        let dummy_tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
        assert!(OptimismDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_decoder_reuses_ethereum() {
        // Verify that OptimismDecoder uses EthereumTransaction type
        use std::any::TypeId;

        fn assert_same_type<T: 'static, U: 'static>() {
            assert_eq!(TypeId::of::<T>(), TypeId::of::<U>());
        }

        type OptimismTxType = <OptimismDecoder as ChainDecoder>::TxSpecific;
        assert_same_type::<OptimismTxType, EthereumTransaction>();
    }
}
