//! C-Chain (Contract Chain) decoder
//!
//! The C-Chain is EVM-compatible and uses the exact same transaction format as Ethereum.

use decoder_ethereum::{types::EthereumTransaction, EthereumDecoder};
use decoder_primitives::prelude::*;

/// C-Chain identity
#[derive(Debug, Clone, Copy)]
pub struct CChain;

impl ChainIdentity for CChain {
    fn chain_id(&self) -> u64 {
        43114 // Avalanche C-Chain ID
    }

    fn chain_name(&self) -> &str {
        "Avalanche-C"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// C-Chain decoder implementing the ChainDecoder trait
///
/// **Reuses Ethereum decoder** with Avalanche-specific chain ID validation.
pub struct CChainDecoder;

impl ChainDecoder for CChainDecoder {
    type TxSpecific = EthereumTransaction; // Reuse Ethereum transaction type
    type Chain = CChain;

    fn chain() -> Self::Chain {
        CChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Decode using Ethereum decoder (same RLP format)
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID is for Avalanche C-Chain
        if let Some(chain_id) = tx.chain_id {
            if chain_id != 43114 {
                return Err(DecoderError::invalid_structure(format!(
                    "Invalid Avalanche C-Chain ID: {} (expected 43114)",
                    chain_id
                )));
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
        let chain = CChainDecoder::chain();
        assert_eq!(chain.chain_id(), 43114);
        assert_eq!(chain.chain_name(), "Avalanche-C");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(CChainDecoder::validate_format(&[]).is_err());

        // Too small should fail (minimum is 5 bytes for Ethereum)
        assert!(CChainDecoder::validate_format(&[0x01]).is_err());
        assert!(CChainDecoder::validate_format(&[0x01, 0x02, 0x03, 0x04]).is_err());

        // Valid minimum length should pass basic validation
        let dummy_tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
        assert!(CChainDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_decoder_reuses_ethereum() {
        // Verify that CChainDecoder uses EthereumTransaction type
        use std::any::TypeId;

        fn assert_same_type<T: 'static, U: 'static>() {
            assert_eq!(TypeId::of::<T>(), TypeId::of::<U>());
        }

        type CChainTxType = <CChainDecoder as ChainDecoder>::TxSpecific;
        assert_same_type::<CChainTxType, EthereumTransaction>();
    }
}
