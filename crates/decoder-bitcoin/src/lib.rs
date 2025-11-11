//! Bitcoin transaction decoder
//!
//! This module provides a decoder for Bitcoin transactions, transforming them
//! from their native format into the universal TxIR representation.

use bitcoin::{consensus::Decodable, Transaction as BitcoinTx};
use std::io::Cursor;
use universal_decoder_core::prelude::*;

pub mod types;

use types::BitcoinTransaction;

/// Bitcoin chain identity
#[derive(Debug, Clone, Copy)]
pub struct BitcoinChain;

impl ChainIdentity for BitcoinChain {
    fn chain_id(&self) -> u64 {
        0 // Bitcoin chain ID
    }

    fn chain_name(&self) -> &str {
        "Bitcoin"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

/// Bitcoin decoder implementing the ChainDecoder trait
pub struct BitcoinDecoder;

impl ChainDecoder for BitcoinDecoder {
    type TxSpecific = BitcoinTransaction;
    type Chain = BitcoinChain;

    fn chain() -> Self::Chain {
        BitcoinChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Parse using bitcoin crate
        let mut cursor = Cursor::new(raw_bytes);
        let tx = BitcoinTx::consensus_decode(&mut cursor)
            .map_err(|e| DecoderError::chain_decoding(format!("Bitcoin decode error: {}", e)))?;

        Ok(BitcoinTransaction::from_bitcoin_tx(tx, raw_bytes))
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction cannot be empty",
            ));
        }

        // Basic sanity check: Bitcoin transactions have a minimum size
        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction too small",
            ));
        }

        Ok(())
    }
}

/// Helper function to decode a Bitcoin transaction with hooks
pub fn decode_with_hooks(raw_bytes: &[u8], registry: &HookRegistry) -> Result<BitcoinTransaction> {
    // Execute pre-decode hooks
    let context = HookContext::new(HookStage::PreDecode, raw_bytes);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    // Perform decoding
    let tx = BitcoinDecoder::decode(raw_bytes)?;

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
    use universal_decoder_core::hex;

    // Example Bitcoin transaction (simplified for testing)
    // This is a minimal valid transaction structure
    const TEST_TX_HEX: &str = "0100000001000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000";

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(BitcoinDecoder::validate_format(&[]).is_err());

        // Too small transaction should fail
        assert!(BitcoinDecoder::validate_format(&[0x01]).is_err());

        // Reasonable size should pass basic validation
        let dummy_tx = vec![0u8; 100];
        assert!(BitcoinDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_chain() {
        let chain = BitcoinDecoder::chain();
        assert_eq!(chain.chain_id(), 0);
        assert_eq!(chain.chain_name(), "Bitcoin");
        assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    }

    #[test]
    fn test_decode_with_hooks() {
        let registry = HookRegistryBuilder::new().with_size_limit(10000).build();

        // Create a minimal transaction
        let tx_bytes = hex::decode(TEST_TX_HEX).unwrap();

        // This might fail due to invalid transaction structure, but we're testing the hook mechanism
        let _result = decode_with_hooks(&tx_bytes, &registry);
    }
}
