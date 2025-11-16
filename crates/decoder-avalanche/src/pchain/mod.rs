//! P-Chain (Platform Chain) decoder
//!
//! The P-Chain is used for staking AVAX and creating/managing subnets and validators.

pub mod canonicalizer;
pub mod parsing;
pub mod types;

use crate::common::*;
use decoder_primitives::prelude::*;
use parsing::*;
pub use types::*;

/// P-Chain identity
#[derive(Debug, Clone, Copy)]
pub struct PChain;

impl ChainIdentity for PChain {
    fn chain_id(&self) -> u64 {
        // P-Chain doesn't use numeric chain IDs like EVM chains
        // Using 0 as placeholder
        0
    }

    fn chain_name(&self) -> &str {
        "Avalanche-P"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account // Platform operations are account-like
    }
}

/// P-Chain decoder
pub struct PChainDecoder;

impl ChainDecoder for PChainDecoder {
    type TxSpecific = PChainTransaction;
    type Chain = PChain;

    fn chain() -> Self::Chain {
        PChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        parse_pchain_transaction(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Minimum size check: codec_id (2) + type_id (4) = 6 bytes minimum
        if raw_bytes.len() < 6 {
            return Err(DecoderError::invalid_structure(
                "Transaction too small for P-Chain format",
            ));
        }

        // Validate codec ID
        let codec_id = u16::from_be_bytes([raw_bytes[0], raw_bytes[1]]);
        if codec_id != CODEC_ID {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid codec ID: 0x{:04x} (expected 0x{:04x})",
                codec_id, CODEC_ID
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
        let chain = PChainDecoder::chain();
        assert_eq!(chain.chain_name(), "Avalanche-P");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format_too_small() {
        assert!(PChainDecoder::validate_format(&[]).is_err());
        assert!(PChainDecoder::validate_format(&[0x00]).is_err());
        assert!(PChainDecoder::validate_format(&[0x00, 0x00, 0x00]).is_err());
    }

    #[test]
    fn test_validate_format_invalid_codec() {
        let invalid = vec![0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
        assert!(PChainDecoder::validate_format(&invalid).is_err());
    }

    #[test]
    fn test_validate_format_valid() {
        let valid = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x0c];
        assert!(PChainDecoder::validate_format(&valid).is_ok());
    }
}
