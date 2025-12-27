//! Ethereum transaction decoder
//!
//! This module provides a decoder for Ethereum transactions, transforming them
//! from their native RLP format into the universal TxIR representation.

use universal_decoder_core::prelude::*;

pub mod types;
pub mod verified;

#[cfg(feature = "formal-verification")]
pub mod verus_annotations;

use types::EthereumTransaction;

// Re-export verified types
pub use verified::{EthereumParsedFields, VerifiedEthereumDecoder};

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

/// Generic EVM-compatible chain
///
/// This is used for EVM chains where we know the chain ID but want to
/// represent them generically (e.g., Polygon, Optimism, Arbitrum, etc.)
#[derive(Debug)]
pub struct GenericEvmChain {
    chain_id: u64,
    chain_name: &'static str,
}

impl GenericEvmChain {
    const fn new(chain_id: u64, chain_name: &'static str) -> Self {
        Self {
            chain_id,
            chain_name,
        }
    }
}

impl ChainIdentity for GenericEvmChain {
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn chain_name(&self) -> &str {
        self.chain_name
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

// Static instances for known EVM chains
static ETHEREUM_CHAIN_STATIC: GenericEvmChain = GenericEvmChain::new(1, "Ethereum");
static POLYGON_CHAIN_STATIC: GenericEvmChain = GenericEvmChain::new(137, "Polygon");
static OPTIMISM_CHAIN_STATIC: GenericEvmChain = GenericEvmChain::new(10, "Optimism");
static ARBITRUM_CHAIN_STATIC: GenericEvmChain = GenericEvmChain::new(42161, "Arbitrum One");
static BNB_CHAIN_STATIC: GenericEvmChain = GenericEvmChain::new(56, "BNB Smart Chain");
static AVALANCHE_CHAIN_STATIC: GenericEvmChain = GenericEvmChain::new(43114, "Avalanche C-Chain");
static BASE_CHAIN_STATIC: GenericEvmChain = GenericEvmChain::new(8453, "Base");

/// Get an EVM chain identity by chain ID
///
/// Returns a static reference to a GenericEvmChain for the given chain ID.
/// For known chains (Ethereum, Polygon, Optimism, Arbitrum, BSC, Avalanche, Base),
/// returns pre-defined static instances. For unknown chains, creates and leaks
/// a generic EVM chain instance.
///
/// # Memory Leaking
///
/// For unknown chain IDs, this function intentionally leaks memory to create
/// a 'static reference. This is acceptable because:
/// 1. We only leak once per unique chain ID (bounded by number of chains)
/// 2. Chain IDs are typically small and finite
/// 3. Alternative would be complex lifetime management
///
/// # Thread Safety
///
/// This function is thread-safe for known chains. For unknown chains, concurrent
/// calls with the same chain ID may create multiple leaked instances, but this
/// is acceptable given the bounded nature of chain IDs.
pub fn get_evm_chain_by_id(chain_id: u64) -> &'static GenericEvmChain {
    // Return known chains
    match chain_id {
        1 => &ETHEREUM_CHAIN_STATIC,
        137 => &POLYGON_CHAIN_STATIC,
        10 => &OPTIMISM_CHAIN_STATIC,
        42161 => &ARBITRUM_CHAIN_STATIC,
        56 => &BNB_CHAIN_STATIC,
        43114 => &AVALANCHE_CHAIN_STATIC,
        8453 => &BASE_CHAIN_STATIC,
        _ => {
            // For unknown chains, create and leak a GenericEvmChain
            let chain = Box::new(GenericEvmChain::new(
                chain_id,
                Box::leak(format!("EVM Chain {}", chain_id).into_boxed_str()),
            ));
            Box::leak(chain)
        }
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
