//! Avalanche transaction decoder
//!
//! This module provides a decoder for Avalanche transactions, transforming them
//! from their native RLP format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! Avalanche is EVM-compatible and uses the **exact same transaction format as Ethereum**.
//! This decoder **reuses the Ethereum decoder** with Avalanche-specific chain ID validation.
//!
//! ## Transaction Format
//!
//! - RLP-encoded (identical to Ethereum)
//! - EIP-2718 transaction types (legacy, EIP-2930, EIP-1559)
//! - Chain ID: 43114
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_avalanche::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "f86c...";
//! let tx_bytes = hex::decode(tx_hex)?;
//!
//! let decoded = AvalancheDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

use decoder_primitives::prelude::*;
use decoder_ethereum::{EthereumDecoder, types::EthereumTransaction};

/// Avalanche chain identity
#[derive(Debug, Clone, Copy)]
pub struct AvalancheChain;

impl ChainIdentity for AvalancheChain {
    fn chain_id(&self) -> u64 {
        43114
    }

    fn chain_name(&self) -> &str {
        "Avalanche"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Avalanche decoder implementing the ChainDecoder trait
///
/// **Reuses Ethereum decoder** with Avalanche-specific chain ID validation.
pub struct AvalancheDecoder;

impl ChainDecoder for AvalancheDecoder {
    type TxSpecific = EthereumTransaction;  // Reuse Ethereum transaction type
    type Chain = AvalancheChain;

    fn chain() -> Self::Chain {
        AvalancheChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Decode using Ethereum decoder (same RLP format)
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID is for Avalanche
        if let Some(chain_id) = tx.chain_id {
            if chain_id != 43114 {
                return Err(DecoderError::invalid_structure(
                    format!("Invalid Avalanche chain ID: {} (expected 43114)", chain_id)
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
        let chain = AvalancheDecoder::chain();
        assert_eq!(chain.chain_id(), 43114);
        assert_eq!(chain.chain_name(), "Avalanche");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(AvalancheDecoder::validate_format(&[]).is_err());

        // Too small should fail (minimum is 5 bytes for Ethereum)
        assert!(AvalancheDecoder::validate_format(&[0x01]).is_err());
        assert!(AvalancheDecoder::validate_format(&[0x01, 0x02, 0x03, 0x04]).is_err());

        // Valid minimum length should pass basic validation
        let dummy_tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
        assert!(AvalancheDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_decoder_reuses_ethereum() {
        // Verify that AvalancheDecoder uses EthereumTransaction type
        use std::any::TypeId;

        fn assert_same_type<T: 'static, U: 'static>() {
            assert_eq!(TypeId::of::<T>(), TypeId::of::<U>());
        }

        type AvalancheTxType = <AvalancheDecoder as ChainDecoder>::TxSpecific;
        assert_same_type::<AvalancheTxType, EthereumTransaction>();
    }
}
