//! Chain identity and extensibility traits
//!
//! This module provides trait-based chain identification, enabling unlimited
//! blockchain support without modifying the core library.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Identifies a blockchain network
///
/// This trait allows blockchain-specific decoders to provide their chain identity
/// without requiring changes to the core library. This follows the open-closed principle:
/// open for extension (new chains), closed for modification (core stays unchanged).
///
/// # Example
///
/// ```rust
/// use universal_decoder_core::chain::*;
///
/// #[derive(Debug)]
/// pub struct BitcoinChain;
///
/// impl ChainIdentity for BitcoinChain {
///     fn chain_id(&self) -> u64 { 0 }
///     fn chain_name(&self) -> &str { "Bitcoin" }
///     fn chain_family(&self) -> ChainFamily { ChainFamily::Utxo }
/// }
/// ```
pub trait ChainIdentity: Send + Sync + Debug {
    /// Unique chain identifier
    ///
    /// Should use a consistent registry (e.g., CAIP-2, SLIP-44, or custom).
    /// For well-known chains:
    /// - Bitcoin: 0
    /// - Ethereum: 1
    /// - Solana: 501
    fn chain_id(&self) -> u64;

    /// Human-readable chain name
    fn chain_name(&self) -> &str;

    /// Semantic grouping by transaction model
    fn chain_family(&self) -> ChainFamily;

    /// Network type (mainnet, testnet, etc.)
    fn network(&self) -> Option<&str> {
        None
    }

    /// Optional: Chain-specific metadata (JSON string)
    fn metadata(&self) -> Option<String> {
        None
    }
}

/// Semantic grouping of blockchain transaction models
///
/// Different blockchains use different state models, which affects
/// how transactions are structured and how state transitions occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainFamily {
    /// UTXO model: Transactions consume inputs and create outputs
    ///
    /// Examples: Bitcoin, Litecoin, Dogecoin
    Utxo,

    /// Account model: Transactions modify account balances and state
    ///
    /// Examples: Ethereum, Polygon, BSC
    Account,

    /// Instruction-based: Transactions contain program instructions
    ///
    /// Examples: Solana, Aptos
    Instruction,

    /// Privacy-focused: Shielded transactions with zero-knowledge proofs
    ///
    /// Examples: Zcash (shielded), Monero (RingCT), Aleo (Leo VM)
    Privacy,

    /// Actor model: Async message-passing between autonomous actors
    ///
    /// Examples: Internet Computer (ICP), Arweave AO
    Actor,

    /// Hybrid or other models
    Other,
}

/// Serializable chain reference for canonical encoding
///
/// This struct is created from a `ChainIdentity` trait object and can be
/// serialized using Borsh for canonical representation in TxIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ChainRef {
    /// Unique chain identifier
    pub id: u64,

    /// Human-readable chain name
    pub name: String,

    /// Chain family (UTXO, Account, Instruction, Other)
    pub family: ChainFamilyEncoded,

    /// Optional network identifier (mainnet, testnet, etc.)
    pub network: Option<String>,
}

/// Encoded version of ChainFamily for Borsh serialization
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum ChainFamilyEncoded {
    Utxo = 0,
    Account = 1,
    Instruction = 2,
    Privacy = 3,
    Actor = 5,
    Other = 4,
}

impl From<ChainFamily> for ChainFamilyEncoded {
    fn from(family: ChainFamily) -> Self {
        match family {
            ChainFamily::Utxo => ChainFamilyEncoded::Utxo,
            ChainFamily::Account => ChainFamilyEncoded::Account,
            ChainFamily::Instruction => ChainFamilyEncoded::Instruction,
            ChainFamily::Privacy => ChainFamilyEncoded::Privacy,
            ChainFamily::Actor => ChainFamilyEncoded::Actor,
            ChainFamily::Other => ChainFamilyEncoded::Other,
        }
    }
}

impl From<ChainFamilyEncoded> for ChainFamily {
    fn from(encoded: ChainFamilyEncoded) -> Self {
        match encoded {
            ChainFamilyEncoded::Utxo => ChainFamily::Utxo,
            ChainFamilyEncoded::Account => ChainFamily::Account,
            ChainFamilyEncoded::Instruction => ChainFamily::Instruction,
            ChainFamilyEncoded::Privacy => ChainFamily::Privacy,
            ChainFamilyEncoded::Actor => ChainFamily::Actor,
            ChainFamilyEncoded::Other => ChainFamily::Other,
        }
    }
}

impl<C: ChainIdentity> From<&C> for ChainRef {
    fn from(chain: &C) -> Self {
        Self {
            id: chain.chain_id(),
            name: chain.chain_name().to_string(),
            family: chain.chain_family().into(),
            network: chain.network().map(|s| s.to_string()),
        }
    }
}

impl ChainRef {
    /// Create a ChainRef from a ChainIdentity trait object
    pub fn from_chain<C: ChainIdentity>(chain: &C) -> Self {
        Self::from(chain)
    }

    /// Get the chain family as the high-level enum
    pub fn family(&self) -> ChainFamily {
        self.family.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestChain;

    impl ChainIdentity for TestChain {
        fn chain_id(&self) -> u64 {
            42
        }

        fn chain_name(&self) -> &str {
            "Test Chain"
        }

        fn chain_family(&self) -> ChainFamily {
            ChainFamily::Account
        }

        fn network(&self) -> Option<&str> {
            Some("testnet")
        }
    }

    #[test]
    fn test_chain_ref_from_identity() {
        let chain = TestChain;
        let chain_ref = ChainRef::from(&chain);

        assert_eq!(chain_ref.id, 42);
        assert_eq!(chain_ref.name, "Test Chain");
        assert_eq!(chain_ref.family, ChainFamilyEncoded::Account);
        assert_eq!(chain_ref.network, Some("testnet".to_string()));
    }

    #[test]
    fn test_chain_family_encoding() {
        assert_eq!(
            ChainFamilyEncoded::from(ChainFamily::Utxo),
            ChainFamilyEncoded::Utxo
        );
        assert_eq!(
            ChainFamily::from(ChainFamilyEncoded::Account),
            ChainFamily::Account
        );
    }

    #[test]
    fn test_chain_ref_borsh_serialization() {
        let chain_ref = ChainRef {
            id: 1,
            name: "Ethereum".to_string(),
            family: ChainFamilyEncoded::Account,
            network: Some("mainnet".to_string()),
        };

        // Serialize
        let bytes = borsh::to_vec(&chain_ref).unwrap();

        // Deserialize
        let deserialized: ChainRef = borsh::from_slice(&bytes).unwrap();

        assert_eq!(chain_ref, deserialized);
    }
}
