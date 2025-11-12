//! BNB Chain (Binance Smart Chain) transaction decoder
//!
//! This module provides a decoder for BNB Chain transactions, transforming them
//! from their native RLP format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! BNB Chain is EVM-compatible and uses the **exact same transaction format as Ethereum**.
//! This decoder **reuses the Ethereum decoder** with BNB-specific chain ID validation.
//!
//! ## Transaction Format
//!
//! - RLP-encoded (identical to Ethereum)
//! - EIP-2718 transaction types (legacy, EIP-2930, EIP-1559)
//! - Chain ID: 56 (mainnet), 97 (testnet)
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_bnb::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "f86c...";
//! let tx_bytes = hex::decode(tx_hex)?;
//!
//! let decoded = BnbDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

use decoder_primitives::prelude::*;
use decoder_ethereum::{EthereumDecoder, types::EthereumTransaction};

/// BNB Chain identity
#[derive(Debug, Clone, Copy)]
pub struct BnbChain;

impl ChainIdentity for BnbChain {
    fn chain_id(&self) -> u64 {
        56 // BNB Chain mainnet ID
    }

    fn chain_name(&self) -> &str {
        "BNB Chain"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// BNB Chain decoder implementing the ChainDecoder trait
///
/// **Reuses Ethereum decoder** with BNB-specific chain ID validation.
pub struct BnbDecoder;

impl ChainDecoder for BnbDecoder {
    type TxSpecific = EthereumTransaction;  // Reuse Ethereum transaction type
    type Chain = BnbChain;

    fn chain() -> Self::Chain {
        BnbChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Decode using Ethereum decoder (same RLP format)
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID is for BNB Chain
        if let Some(chain_id) = tx.chain_id {
            if chain_id != 56 && chain_id != 97 {
                return Err(DecoderError::invalid_structure(
                    format!("Invalid BNB Chain ID: {} (expected 56 for mainnet or 97 for testnet)", chain_id)
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
        let chain = BnbDecoder::chain();
        assert_eq!(chain.chain_id(), 56);
        assert_eq!(chain.chain_name(), "BNB Chain");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(BnbDecoder::validate_format(&[]).is_err());

        // Too small should fail (minimum is 5 bytes for Ethereum)
        assert!(BnbDecoder::validate_format(&[0x01]).is_err());
        assert!(BnbDecoder::validate_format(&[0x01, 0x02, 0x03, 0x04]).is_err());

        // Valid minimum length should pass basic validation
        let dummy_tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
        assert!(BnbDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_decoder_reuses_ethereum() {
        // Verify that BnbDecoder uses EthereumTransaction type
        use std::any::TypeId;

        // This is a compile-time check that the types match
        fn assert_same_type<T: 'static, U: 'static>() {
            assert_eq!(TypeId::of::<T>(), TypeId::of::<U>());
        }

        type BnbTxType = <BnbDecoder as ChainDecoder>::TxSpecific;
        assert_same_type::<BnbTxType, EthereumTransaction>();
    }
}
