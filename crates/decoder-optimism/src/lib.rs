//! Optimism transaction decoder
//!
//! This module provides a decoder for Optimism transactions, transforming them
//! from their native RLP format into the universal TxIR representation.
//!
//! ## Supported Transaction Types
//!
//! - **Standard Ethereum transactions** (types 0x00, 0x01, 0x02)
//!   - Legacy transactions
//!   - EIP-2930 (access list)
//!   - EIP-1559 (dynamic fee)
//! - **Deposit transactions** (type 0x7E) - Optimism-specific
//!   - L1 attributes deposits (first tx in every block)
//!   - User deposits (from OptimismPortal contract on L1)
//!
//! ## OP Stack Support
//!
//! This decoder supports the entire OP Stack ecosystem (35+ chains):
//! - Optimism (chain ID 10)
//! - Base (chain ID 8453)
//! - Zora (chain ID 7777777)
//! - Mode (chain ID 34443)
//! - And 30+ more OP Stack chains
//!
//! ## Deposit Transactions
//!
//! Deposit transactions (0x7E) are unique to Optimism and enable L1→L2 communication:
//!
//! - **No signatures**: Authorization via chain derivation, not cryptographic signatures
//! - **ETH minting**: Can mint ETH on L2 to match L1 deposits
//! - **L1 attributes**: First transaction in every L2 block sets L1 metadata
//!
//! ## Example
//!
//! ```rust,no_run
//! use decoder_optimism::*;
//! use universal_decoder_core::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Standard Ethereum transaction on Optimism
//! let eth_tx_bytes = hex::decode("02...")?;
//! let tx = OptimismDecoder::decode(&eth_tx_bytes)?;
//!
//! // Deposit transaction (0x7E)
//! let deposit_tx_bytes = hex::decode("7e...")?;
//! let deposit = OptimismDecoder::decode(&deposit_tx_bytes)?;
//!
//! match deposit {
//!     OptimismTransaction::Deposit(d) => {
//!         println!("Deposit from {} to {:?}", hex::encode(d.from), d.to);
//!         println!("Mint: {}, Value: {}", d.mint, d.value);
//!     }
//!     OptimismTransaction::Standard(eth) => {
//!         println!("Standard Ethereum transaction");
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Specification
//!
//! See: https://specs.optimism.io/protocol/deposits.html

use decoder_primitives::prelude::*;

pub mod parsing;
pub mod registry;
pub mod types;

pub use types::{DepositTransaction, OptimismTransaction};

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
/// Supports both standard Ethereum transactions and Optimism-specific deposit transactions.
pub struct OptimismDecoder;

impl ChainDecoder for OptimismDecoder {
    type TxSpecific = OptimismTransaction;
    type Chain = OptimismChain;

    fn chain() -> Self::Chain {
        OptimismChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Parse transaction (handles both standard and deposit types)
        let tx = parsing::parse_optimism_transaction(raw_bytes)?;

        // Validate chain ID for standard transactions
        if let OptimismTransaction::Standard(ref eth_tx) = tx {
            if let Some(chain_id) = eth_tx.chain_id {
                // Allow any OP Stack chain ID (not just 10)
                // The registry will handle chain-specific validation
                if !is_op_stack_chain(chain_id) {
                    eprintln!(
                        "Warning: Chain ID {} may not be an OP Stack chain. \
                             Known OP Stack chains: 10 (Optimism), 8453 (Base), 7777777 (Zora), etc.",
                        chain_id
                    );
                }
            }
        }

        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Empty transaction bytes".to_string(),
            ));
        }

        // Minimum size check
        if raw_bytes.len() < 5 {
            return Err(DecoderError::invalid_structure(format!(
                "Transaction too small: {} bytes (minimum 5)",
                raw_bytes.len()
            )));
        }

        Ok(())
    }
}

/// Check if a chain ID belongs to the OP Stack ecosystem
///
/// This is a heuristic check. Use the registry for definitive chain information.
fn is_op_stack_chain(chain_id: u64) -> bool {
    matches!(
        chain_id,
        10        // Optimism
        | 8453    // Base
        | 7777777 // Zora
        | 34443   // Mode
        | 424     // PGN (Public Goods Network)
        | 888888  // Orderly
        | 81457   // Blast
        | 690     // Redstone
        | 255     // Kroma
        | 5000    // Mantle
        | 957     // Lyra
        | 1750    // Metal
        | 1135 // Lisk
    ) || (chain_id >= 900000 && chain_id < 910000) // OP Stack testnet range
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DepositTransaction;

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

        // Too small should fail
        assert!(OptimismDecoder::validate_format(&[0x01]).is_err());
        assert!(OptimismDecoder::validate_format(&[0x01, 0x02, 0x03, 0x04]).is_err());

        // Valid minimum length should pass
        let dummy_tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
        assert!(OptimismDecoder::validate_format(&dummy_tx).is_ok());

        // Deposit transaction (0x7E + RLP)
        let deposit_tx = vec![0x7E, 0xf8, 0x6c, 0x00, 0x00];
        assert!(OptimismDecoder::validate_format(&deposit_tx).is_ok());
    }

    #[test]
    fn test_is_op_stack_chain() {
        // Known OP Stack chains
        assert!(is_op_stack_chain(10)); // Optimism
        assert!(is_op_stack_chain(8453)); // Base
        assert!(is_op_stack_chain(7777777)); // Zora
        assert!(is_op_stack_chain(34443)); // Mode
        assert!(is_op_stack_chain(424)); // PGN

        // Testnet range
        assert!(is_op_stack_chain(900000));
        assert!(is_op_stack_chain(905000));
        assert!(is_op_stack_chain(909999));

        // Non-OP Stack chains
        assert!(!is_op_stack_chain(1)); // Ethereum
        assert!(!is_op_stack_chain(56)); // BNB
        assert!(!is_op_stack_chain(137)); // Polygon
        assert!(!is_op_stack_chain(42161)); // Arbitrum
        assert!(!is_op_stack_chain(999999)); // Unknown
    }

    #[test]
    fn test_optimism_transaction_type_usage() {
        // Verify that OptimismDecoder uses OptimismTransaction type
        use std::any::TypeId;

        fn assert_same_type<T: 'static, U: 'static>() {
            assert_eq!(TypeId::of::<T>(), TypeId::of::<U>());
        }

        type OpTxType = <OptimismDecoder as ChainDecoder>::TxSpecific;
        assert_same_type::<OpTxType, OptimismTransaction>();
    }

    #[test]
    fn test_deposit_transaction_detection() {
        // Deposit transaction starts with 0x7E
        let deposit_bytes = vec![0x7E, 0xc8, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];

        // This will fail parsing (invalid RLP structure), but format validation should pass
        assert!(OptimismDecoder::validate_format(&deposit_bytes).is_ok());
    }

    #[test]
    fn test_deposit_transaction_constant() {
        assert_eq!(DepositTransaction::TYPE_ID, 0x7E);
    }
}
