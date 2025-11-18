//! Arbitrum Orbit transaction decoder
//!
//! This module provides a decoder for Arbitrum Orbit transactions, supporting:
//! - Standard Ethereum transactions (types 0x00, 0x01, 0x02)
//! - Arbitrum-specific transactions (types 0x64-0x6A)
//!
//! ## Arbitrum Orbit Chains
//!
//! - **Arbitrum One** (Chain ID: 42161) - Main Arbitrum L2
//! - **Arbitrum Nova** (Chain ID: 42170) - Gaming/social-focused L2
//! - **Arbitrum Sepolia** (Chain ID: 421614) - Testnet
//! - **Other Orbit chains** - Custom Arbitrum deployments
//!
//! ## Arbitrum Transaction Types
//!
//! ### Standard Ethereum (0x00-0x02)
//! - Legacy, EIP-2930, EIP-1559 transactions
//!
//! ### Arbitrum-Specific (0x64-0x6A)
//! - **0x64**: Deposit - L1→L2 deposit
//! - **0x65**: Unsigned - EOA via bridge (no signature)
//! - **0x66**: Contract - L1 contract calling L2
//! - **0x68**: Retry - Retry failed retryable ticket
//! - **0x69**: SubmitRetryable - New retryable ticket (most common)
//! - **0x6A**: Internal - ArbOS system transaction
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_arbitrum::*;
//! use universal_decoder_core::prelude::*;
//!
//! // Decode standard Ethereum transaction on Arbitrum
//! let tx_hex = "02f8..."; // EIP-1559 transaction
//! let tx_bytes = hex::decode(tx_hex)?;
//! let decoded = ArbitrumDecoder::decode(&tx_bytes)?;
//!
//! match decoded {
//!     ArbitrumTransaction::Standard(eth_tx) => {
//!         println!("Standard EVM tx: {:?}", eth_tx);
//!     }
//!     ArbitrumTransaction::SubmitRetryable(retryable) => {
//!         println!("Retryable ticket: {:?}", retryable);
//!     }
//!     _ => println!("Other Arbitrum transaction type"),
//! }
//! ```
//!
//! ## Specification
//!
//! - ArbOS documentation: <https://docs.arbitrum.io/arbos/>
//! - Retryable tickets: <https://docs.arbitrum.io/arbos/l1-to-l2-messaging>
//! - Delayed inbox: <https://docs.arbitrum.io/arbos/geth>

pub mod parsing;
pub mod types;

use decoder_primitives::prelude::*;
use parsing::parse_arbitrum_transaction;
pub use types::*;

/// Arbitrum chain identity
///
/// Represents various Arbitrum Orbit chains.
#[derive(Debug, Clone, Copy)]
pub struct ArbitrumChain {
    /// Chain ID for this Arbitrum chain
    pub chain_id: u64,
    /// Human-readable chain name
    pub name: &'static str,
}

impl ArbitrumChain {
    /// Arbitrum One (main L2)
    pub const ONE: Self = Self {
        chain_id: 42161,
        name: "Arbitrum One",
    };

    /// Arbitrum Nova (gaming/social)
    pub const NOVA: Self = Self {
        chain_id: 42170,
        name: "Arbitrum Nova",
    };

    /// Arbitrum Sepolia (testnet)
    pub const SEPOLIA: Self = Self {
        chain_id: 421614,
        name: "Arbitrum Sepolia",
    };

    /// Arbitrum Goerli (deprecated testnet)
    pub const GOERLI: Self = Self {
        chain_id: 421613,
        name: "Arbitrum Goerli",
    };

    /// Create a custom Arbitrum Orbit chain
    pub const fn custom(chain_id: u64, name: &'static str) -> Self {
        Self { chain_id, name }
    }

    /// Detect Arbitrum chain from chain ID
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            42161 => Some(Self::ONE),
            42170 => Some(Self::NOVA),
            421614 => Some(Self::SEPOLIA),
            421613 => Some(Self::GOERLI),
            _ => {
                // Check if it's in Arbitrum chain ID range
                // Arbitrum typically uses 42xxx for mainnet and 421xxx for testnets
                if (42000..43000).contains(&chain_id) {
                    Some(Self::custom(chain_id, "Arbitrum Orbit"))
                } else {
                    None
                }
            }
        }
    }

    /// Check if this is a mainnet chain
    pub const fn is_mainnet(&self) -> bool {
        matches!(self.chain_id, 42161 | 42170)
    }

    /// Check if this is a testnet chain
    pub const fn is_testnet(&self) -> bool {
        matches!(self.chain_id, 421613 | 421614)
    }
}

impl ChainIdentity for ArbitrumChain {
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn chain_name(&self) -> &str {
        self.name
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Arbitrum decoder implementing the ChainDecoder trait
///
/// Supports all Arbitrum Orbit chains and Arbitrum-specific transaction types.
pub struct ArbitrumDecoder {
    /// Chain identity (Arbitrum One, Nova, etc.)
    pub chain: ArbitrumChain,
}

impl ArbitrumDecoder {
    /// Create a decoder for Arbitrum One
    pub fn new() -> Self {
        Self {
            chain: ArbitrumChain::ONE,
        }
    }

    /// Create a decoder for a specific Arbitrum chain
    pub fn for_chain(chain: ArbitrumChain) -> Self {
        Self { chain }
    }

    /// Create a decoder for a chain ID
    pub fn for_chain_id(chain_id: u64) -> Result<Self> {
        let chain = ArbitrumChain::from_chain_id(chain_id).ok_or_else(|| {
            DecoderError::invalid_structure(format!(
                "Chain ID {} is not a recognized Arbitrum chain",
                chain_id
            ))
        })?;
        Ok(Self { chain })
    }
}

impl Default for ArbitrumDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainDecoder for ArbitrumDecoder {
    type TxSpecific = ArbitrumTransaction;
    type Chain = ArbitrumChain;

    fn chain() -> Self::Chain {
        ArbitrumChain::ONE
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Parse Arbitrum transaction (handles all types)
        let tx = parse_arbitrum_transaction(raw_bytes)?;

        // Validate chain ID if present
        let tx_chain_id = match &tx {
            ArbitrumTransaction::Standard(eth_tx) => eth_tx.chain_id,
            ArbitrumTransaction::Deposit(d) => Some(d.chain_id),
            ArbitrumTransaction::Unsigned(u) => Some(u.chain_id),
            ArbitrumTransaction::Contract(c) => Some(c.chain_id),
            ArbitrumTransaction::Retry(r) => Some(r.chain_id),
            ArbitrumTransaction::SubmitRetryable(s) => Some(s.chain_id),
            ArbitrumTransaction::Internal(i) => Some(i.chain_id),
        };

        // Validate chain ID is in Arbitrum range
        if let Some(chain_id) = tx_chain_id {
            if ArbitrumChain::from_chain_id(chain_id).is_none() {
                return Err(DecoderError::invalid_structure(format!(
                    "Chain ID {} is not a recognized Arbitrum chain (expected 42xxx or 421xxx)",
                    chain_id
                )));
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

        // Minimum transaction size check
        if raw_bytes.len() < 2 {
            return Err(DecoderError::invalid_structure(format!(
                "Transaction too small: {} bytes (minimum 2)",
                raw_bytes.len()
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
        let chain = ArbitrumChain::ONE;
        assert_eq!(chain.chain_id(), 42161);
        assert_eq!(chain.chain_name(), "Arbitrum One");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
        assert!(chain.is_mainnet());
        assert!(!chain.is_testnet());
    }

    #[test]
    fn test_chain_detection() {
        assert_eq!(
            ArbitrumChain::from_chain_id(42161).unwrap().chain_id(),
            42161
        );
        assert_eq!(
            ArbitrumChain::from_chain_id(42170).unwrap().chain_id(),
            42170
        );
        assert_eq!(
            ArbitrumChain::from_chain_id(421614).unwrap().chain_id(),
            421614
        );

        // Unknown Arbitrum chain in range
        let custom = ArbitrumChain::from_chain_id(42999).unwrap();
        assert_eq!(custom.chain_id(), 42999);
        assert_eq!(custom.chain_name(), "Arbitrum Orbit");

        // Non-Arbitrum chain
        assert!(ArbitrumChain::from_chain_id(1).is_none()); // Ethereum mainnet
    }

    #[test]
    fn test_decoder_creation() {
        let decoder = ArbitrumDecoder::new();
        assert_eq!(decoder.chain.chain_id(), 42161);

        let decoder = ArbitrumDecoder::for_chain(ArbitrumChain::NOVA);
        assert_eq!(decoder.chain.chain_id(), 42170);

        let decoder = ArbitrumDecoder::for_chain_id(421614).unwrap();
        assert_eq!(decoder.chain.chain_id(), 421614);

        // Invalid chain ID
        assert!(ArbitrumDecoder::for_chain_id(1).is_err());
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(ArbitrumDecoder::validate_format(&[]).is_err());

        // Too small should fail
        assert!(ArbitrumDecoder::validate_format(&[0x01]).is_err());

        // Minimum valid size
        assert!(ArbitrumDecoder::validate_format(&[0x01, 0x02]).is_ok());

        // Standard transaction size
        let tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
        assert!(ArbitrumDecoder::validate_format(&tx).is_ok());
    }

    #[test]
    fn test_decode_empty() {
        let result = ArbitrumDecoder::decode(&[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty transaction"));
    }

    #[test]
    fn test_arbitrum_chain_constants() {
        assert_eq!(ArbitrumChain::ONE.chain_id, 42161);
        assert_eq!(ArbitrumChain::NOVA.chain_id, 42170);
        assert_eq!(ArbitrumChain::SEPOLIA.chain_id, 421614);
        assert_eq!(ArbitrumChain::GOERLI.chain_id, 421613);
    }

    #[test]
    fn test_mainnet_testnet_detection() {
        assert!(ArbitrumChain::ONE.is_mainnet());
        assert!(ArbitrumChain::NOVA.is_mainnet());
        assert!(!ArbitrumChain::SEPOLIA.is_mainnet());
        assert!(!ArbitrumChain::GOERLI.is_mainnet());

        assert!(!ArbitrumChain::ONE.is_testnet());
        assert!(!ArbitrumChain::NOVA.is_testnet());
        assert!(ArbitrumChain::SEPOLIA.is_testnet());
        assert!(ArbitrumChain::GOERLI.is_testnet());
    }
}
