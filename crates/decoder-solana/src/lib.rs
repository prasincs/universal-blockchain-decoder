//! Solana transaction decoder
//!
//! This module provides a decoder for Solana transactions, transforming them
//! from their native format into the universal TxIR representation.
//!
//! Note: This is a stub implementation. Full Solana support coming soon.

use universal_decoder_core::prelude::*;

pub mod types;

use types::SolanaTransaction;

/// Solana decoder implementing the ChainDecoder trait
pub struct SolanaDecoder;

impl ChainDecoder for SolanaDecoder {
    type TxSpecific = SolanaTransaction;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        SolanaTransaction::from_raw_bytes(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Solana transaction cannot be empty",
            ));
        }
        Ok(())
    }

    fn chain_id() -> ChainId {
        ChainId::Solana
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id() {
        assert_eq!(SolanaDecoder::chain_id(), ChainId::Solana);
    }
}
