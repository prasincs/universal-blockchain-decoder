//! Ethereum transaction decoder
//!
//! This module provides a decoder for Ethereum transactions, transforming them
//! from their native RLP format into the universal TxIR representation.

use universal_decoder_core::prelude::*;

pub mod rlp;
pub mod types;

use types::EthereumTransaction;

/// Ethereum chain identity
#[derive(Debug, Clone, Copy)]
pub struct EthereumChain;

impl ChainIdentity for EthereumChain {
    fn chain_id(&self) -> u64 {
        1 // Ethereum chain ID
    }

    fn chain_name(&self) -> &str {
        "Ethereum"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Ethereum decoder implementing the ChainDecoder trait
pub struct EthereumDecoder;

impl ChainDecoder for EthereumDecoder {
    type TxSpecific = EthereumTransaction;
    type Chain = EthereumChain;

    fn chain() -> Self::Chain {
        EthereumChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Parse RLP-encoded transaction
        let tx = EthereumTransaction::from_raw_bytes(raw_bytes)?;
        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Ethereum transaction cannot be empty",
            ));
        }

        // Ethereum transactions encoded with RLP
        // Basic sanity check
        if raw_bytes.len() < 5 {
            return Err(DecoderError::invalid_structure(
                "Ethereum transaction too small",
            ));
        }

        Ok(())
    }
}

/// Helper function to decode an Ethereum transaction with hooks
pub fn decode_with_hooks(raw_bytes: &[u8], registry: &HookRegistry) -> Result<EthereumTransaction> {
    // Execute pre-decode hooks
    let context = HookContext::new(HookStage::PreDecode, raw_bytes);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    // Perform decoding
    let tx = EthereumDecoder::decode(raw_bytes)?;

    // Execute post-decode hooks
    let context = HookContext::new(HookStage::PostDecode, raw_bytes).with_chain_specific(&tx);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(EthereumDecoder::validate_format(&[]).is_err());

        // Too small transaction should fail
        assert!(EthereumDecoder::validate_format(&[0x01]).is_err());

        // Reasonable size should pass basic validation
        let dummy_tx = vec![0u8; 100];
        assert!(EthereumDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_chain() {
        let chain = EthereumDecoder::chain();
        assert_eq!(chain.chain_id(), 1);
        assert_eq!(chain.chain_name(), "Ethereum");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_decode_with_hooks() {
        let registry = HookRegistryBuilder::new().with_size_limit(10000).build();

        // This would need a valid Ethereum transaction
        // For now, we just test the hook mechanism with dummy data
        let tx_bytes = vec![0xf8, 0x6c]; // RLP prefix for a transaction
        let _result = decode_with_hooks(&tx_bytes, &registry);
    }
}
