//! Pre-defined chain identity registry.
//!
//! This module provides standard chain identities for well-known blockchains,
//! reducing boilerplate in decoder implementations.
//!
//! # Example
//!
//! ```rust
//! use decoder_chains_common::chains;
//! use universal_decoder_core::prelude::ChainIdentity;
//!
//! let bitcoin = chains::BITCOIN;
//! assert_eq!(bitcoin.chain_id(), 0);
//! assert_eq!(bitcoin.chain_name(), "Bitcoin");
//! ```

use universal_decoder_core::prelude::{ChainFamily, ChainIdentity};

/// A pre-defined chain identity.
///
/// This struct implements [`ChainIdentity`] and can be used directly in decoders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainInfo {
    /// Unique chain identifier
    pub id: u64,
    /// Human-readable chain name
    pub name: &'static str,
    /// Chain family (UTXO, Account, Instruction, etc.)
    pub family: ChainFamily,
}

impl ChainInfo {
    /// Creates a new chain info.
    pub const fn new(id: u64, name: &'static str, family: ChainFamily) -> Self {
        Self { id, name, family }
    }
}

impl ChainIdentity for ChainInfo {
    fn chain_id(&self) -> u64 {
        self.id
    }

    fn chain_name(&self) -> &str {
        self.name
    }

    fn chain_family(&self) -> ChainFamily {
        self.family
    }
}

// ============================================================================
// UTXO-based Chains
// ============================================================================

/// Bitcoin mainnet
pub const BITCOIN: ChainInfo = ChainInfo::new(0, "Bitcoin", ChainFamily::Utxo);

/// Litecoin mainnet
pub const LITECOIN: ChainInfo = ChainInfo::new(2, "Litecoin", ChainFamily::Utxo);

/// Dogecoin mainnet
pub const DOGECOIN: ChainInfo = ChainInfo::new(3, "Dogecoin", ChainFamily::Utxo);

// ============================================================================
// EVM-compatible Chains
// ============================================================================

/// Ethereum mainnet
pub const ETHEREUM: ChainInfo = ChainInfo::new(1, "Ethereum", ChainFamily::Account);

/// Polygon (formerly Matic)
pub const POLYGON: ChainInfo = ChainInfo::new(137, "Polygon", ChainFamily::Account);

/// BNB Smart Chain (formerly Binance Smart Chain)
pub const BNB: ChainInfo = ChainInfo::new(56, "BNB", ChainFamily::Account);

/// Avalanche C-Chain
pub const AVALANCHE: ChainInfo = ChainInfo::new(43114, "Avalanche", ChainFamily::Account);

/// Arbitrum One
pub const ARBITRUM: ChainInfo = ChainInfo::new(42161, "Arbitrum", ChainFamily::Account);

/// Optimism
pub const OPTIMISM: ChainInfo = ChainInfo::new(10, "Optimism", ChainFamily::Account);

/// Base (Coinbase L2)
pub const BASE: ChainInfo = ChainInfo::new(8453, "Base", ChainFamily::Account);

// ============================================================================
// Account-based Chains (Non-EVM)
// ============================================================================

/// Solana mainnet
pub const SOLANA: ChainInfo = ChainInfo::new(101, "Solana", ChainFamily::Instruction);

/// Aptos mainnet
pub const APTOS: ChainInfo = ChainInfo::new(1001, "Aptos", ChainFamily::Account);

/// Sui mainnet
pub const SUI: ChainInfo = ChainInfo::new(1002, "Sui", ChainFamily::Account);

/// NEAR Protocol
pub const NEAR: ChainInfo = ChainInfo::new(1003, "NEAR", ChainFamily::Account);

/// Stellar
pub const STELLAR: ChainInfo = ChainInfo::new(1004, "Stellar", ChainFamily::Account);

/// XRP Ledger
pub const XRP: ChainInfo = ChainInfo::new(1005, "XRP", ChainFamily::Account);

/// Algorand
pub const ALGORAND: ChainInfo = ChainInfo::new(1006, "Algorand", ChainFamily::Account);

/// Tron
pub const TRON: ChainInfo = ChainInfo::new(1007, "Tron", ChainFamily::Account);

// ============================================================================
// Cosmos Ecosystem
// ============================================================================

/// Cosmos Hub
pub const COSMOS: ChainInfo = ChainInfo::new(118, "Cosmos", ChainFamily::Account);

/// Osmosis
pub const OSMOSIS: ChainInfo = ChainInfo::new(1008, "Osmosis", ChainFamily::Account);

// ============================================================================
// Other Chains
// ============================================================================

/// Polkadot
pub const POLKADOT: ChainInfo = ChainInfo::new(1009, "Polkadot", ChainFamily::Account);

/// Cardano
pub const CARDANO: ChainInfo = ChainInfo::new(1010, "Cardano", ChainFamily::Utxo);

// ============================================================================
// Chain Lookup
// ============================================================================

/// All registered chains in a static array for easy iteration.
pub const ALL_CHAINS: &[ChainInfo] = &[
    BITCOIN, ETHEREUM, LITECOIN, DOGECOIN, POLYGON, BNB, AVALANCHE, ARBITRUM, OPTIMISM, BASE,
    SOLANA, APTOS, SUI, NEAR, STELLAR, XRP, ALGORAND, TRON, COSMOS, OSMOSIS, POLKADOT, CARDANO,
];

/// Looks up a chain by its ID.
///
/// # Arguments
///
/// * `id` - The chain ID to look up
///
/// # Returns
///
/// * `Some(ChainInfo)` if the chain is found
/// * `None` if the chain is not registered
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::chains;
/// use universal_decoder_core::prelude::ChainIdentity;
///
/// let bitcoin = chains::lookup_by_id(0);
/// assert!(bitcoin.is_some());
/// assert_eq!(bitcoin.unwrap().chain_name(), "Bitcoin");
///
/// let unknown = chains::lookup_by_id(99999);
/// assert!(unknown.is_none());
/// ```
pub fn lookup_by_id(id: u64) -> Option<ChainInfo> {
    ALL_CHAINS.iter().find(|c| c.id == id).copied()
}

/// Looks up a chain by its name (case-insensitive).
///
/// # Arguments
///
/// * `name` - The chain name to look up
///
/// # Returns
///
/// * `Some(ChainInfo)` if the chain is found
/// * `None` if the chain is not registered
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::chains;
/// use universal_decoder_core::prelude::ChainIdentity;
///
/// let bitcoin = chains::lookup_by_name("Bitcoin");
/// assert!(bitcoin.is_some());
/// assert_eq!(bitcoin.unwrap().chain_id(), 0);
///
/// let bitcoin_lowercase = chains::lookup_by_name("bitcoin");
/// assert!(bitcoin_lowercase.is_some());
///
/// let unknown = chains::lookup_by_name("UnknownChain");
/// assert!(unknown.is_none());
/// ```
pub fn lookup_by_name(name: &str) -> Option<ChainInfo> {
    ALL_CHAINS
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_identity() {
        assert_eq!(BITCOIN.chain_id(), 0);
        assert_eq!(BITCOIN.chain_name(), "Bitcoin");
        assert_eq!(BITCOIN.chain_family(), ChainFamily::Utxo);
    }

    #[test]
    fn test_ethereum_identity() {
        assert_eq!(ETHEREUM.chain_id(), 1);
        assert_eq!(ETHEREUM.chain_name(), "Ethereum");
        assert_eq!(ETHEREUM.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_solana_identity() {
        assert_eq!(SOLANA.chain_id(), 101);
        assert_eq!(SOLANA.chain_name(), "Solana");
        assert_eq!(SOLANA.chain_family(), ChainFamily::Instruction);
    }

    #[test]
    fn test_lookup_by_id() {
        assert_eq!(lookup_by_id(0), Some(BITCOIN));
        assert_eq!(lookup_by_id(1), Some(ETHEREUM));
        assert_eq!(lookup_by_id(101), Some(SOLANA));
        assert_eq!(lookup_by_id(99999), None);
    }

    #[test]
    fn test_lookup_by_name() {
        assert_eq!(lookup_by_name("Bitcoin"), Some(BITCOIN));
        assert_eq!(lookup_by_name("bitcoin"), Some(BITCOIN));
        assert_eq!(lookup_by_name("BITCOIN"), Some(BITCOIN));
        assert_eq!(lookup_by_name("Ethereum"), Some(ETHEREUM));
        assert_eq!(lookup_by_name("Solana"), Some(SOLANA));
        assert_eq!(lookup_by_name("UnknownChain"), None);
    }

    #[test]
    fn test_all_chains_contains_bitcoin() {
        assert!(ALL_CHAINS.contains(&BITCOIN));
    }

    #[test]
    fn test_all_chains_unique_ids() {
        let mut ids: Vec<u64> = ALL_CHAINS.iter().map(|c| c.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ALL_CHAINS.len(), "Chain IDs must be unique");
    }

    #[test]
    fn test_all_chains_unique_names() {
        let mut names: Vec<&str> = ALL_CHAINS.iter().map(|c| c.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), ALL_CHAINS.len(), "Chain names must be unique");
    }

    #[test]
    fn test_chain_info_implements_traits() {
        // Test that ChainInfo is Debug, Clone, Copy, PartialEq, Eq, Hash
        let chain = BITCOIN;
        let cloned = chain;
        assert_eq!(chain, cloned);
        assert_eq!(format!("{:?}", chain), format!("{:?}", cloned));

        // Test it can be used in a HashMap
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(chain, "Bitcoin");
        assert_eq!(map.get(&BITCOIN), Some(&"Bitcoin"));
    }
}
