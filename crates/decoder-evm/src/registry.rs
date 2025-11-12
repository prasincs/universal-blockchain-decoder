//! Chain registry for EVM-compatible chains
//!
//! This module provides a registry of all known EVM-compatible chains,
//! loaded from the ethereum-lists/chains repository at compile time.

use crate::types::ChainInfo;
use std::collections::HashMap;
use std::sync::OnceLock;

// Include the generated chain registry
include!(concat!(env!("OUT_DIR"), "/chain_registry.rs"));

/// Global chain registry singleton
static CHAIN_REGISTRY: OnceLock<ChainRegistry> = OnceLock::new();

/// Registry of EVM-compatible chains
pub struct ChainRegistry {
    chains: HashMap<u64, ChainInfo>,
    by_short_name: HashMap<String, u64>,
}

impl ChainRegistry {
    /// Get the global chain registry instance
    pub fn global() -> &'static ChainRegistry {
        CHAIN_REGISTRY.get_or_init(|| {
            Self::new()
        })
    }

    /// Create a new chain registry from embedded data
    pub fn new() -> Self {
        let chains = get_embedded_chains();

        // Build reverse index by short name
        let by_short_name: HashMap<String, u64> = chains
            .iter()
            .map(|(id, info)| (info.short_name.clone(), *id))
            .collect();

        Self {
            chains,
            by_short_name,
        }
    }

    /// Get chain information by chain ID
    pub fn get_chain(&self, chain_id: u64) -> Option<&ChainInfo> {
        self.chains.get(&chain_id)
    }

    /// Get chain information by short name (e.g., "eth", "bnb", "matic")
    pub fn get_chain_by_name(&self, short_name: &str) -> Option<&ChainInfo> {
        self.by_short_name
            .get(short_name)
            .and_then(|id| self.chains.get(id))
    }

    /// Check if a chain ID is supported
    pub fn is_supported(&self, chain_id: u64) -> bool {
        self.chains.contains_key(&chain_id)
    }

    /// Get all supported chain IDs
    pub fn all_chain_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.chains.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// Get all chains (sorted by chain ID)
    pub fn all_chains(&self) -> Vec<&ChainInfo> {
        let mut chains: Vec<&ChainInfo> = self.chains.values().collect();
        chains.sort_by_key(|c| c.chain_id);
        chains
    }

    /// Get mainnet chains only (excludes testnets)
    pub fn mainnet_chains(&self) -> Vec<&ChainInfo> {
        let mut chains: Vec<&ChainInfo> = self
            .chains
            .values()
            .filter(|c| !c.is_testnet)
            .collect();
        chains.sort_by_key(|c| c.chain_id);
        chains
    }

    /// Get testnet chains only
    pub fn testnet_chains(&self) -> Vec<&ChainInfo> {
        let mut chains: Vec<&ChainInfo> = self
            .chains
            .values()
            .filter(|c| c.is_testnet)
            .collect();
        chains.sort_by_key(|c| c.chain_id);
        chains
    }

    /// Get chains that require special decoders
    pub fn special_chains(&self) -> Vec<&ChainInfo> {
        let mut chains: Vec<&ChainInfo> = self
            .chains
            .values()
            .filter(|c| c.has_custom_tx_types)
            .collect();
        chains.sort_by_key(|c| c.chain_id);
        chains
    }

    /// Get total number of supported chains
    pub fn count(&self) -> usize {
        self.chains.len()
    }

    /// Search chains by name (case-insensitive substring match)
    pub fn search(&self, query: &str) -> Vec<&ChainInfo> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&ChainInfo> = self
            .chains
            .values()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower) ||
                c.short_name.to_lowercase().contains(&query_lower) ||
                c.chain.to_lowercase().contains(&query_lower)
            })
            .collect();
        results.sort_by_key(|c| c.chain_id);
        results
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_initialization() {
        let registry = ChainRegistry::new();
        assert!(registry.count() > 0, "Registry should contain chains");
        println!("Registry contains {} chains", registry.count());
    }

    #[test]
    fn test_get_ethereum_mainnet() {
        let registry = ChainRegistry::new();

        let eth = registry.get_chain(1);
        assert!(eth.is_some());

        let eth = eth.unwrap();
        assert_eq!(eth.chain_id, 1);
        assert_eq!(eth.short_name, "eth");
        assert!(!eth.is_testnet);
    }

    #[test]
    fn test_get_by_short_name() {
        let registry = ChainRegistry::new();

        let eth = registry.get_chain_by_name("eth");
        assert!(eth.is_some());
        assert_eq!(eth.unwrap().chain_id, 1);

        let bnb = registry.get_chain_by_name("bnb");
        if bnb.is_some() {
            assert_eq!(bnb.unwrap().chain_id, 56);
        }
    }

    #[test]
    fn test_is_supported() {
        let registry = ChainRegistry::new();

        assert!(registry.is_supported(1)); // Ethereum
        assert!(!registry.is_supported(0)); // Invalid chain (0 is never a valid EVM chain ID)
    }

    #[test]
    fn test_mainnet_vs_testnet() {
        let registry = ChainRegistry::new();

        let mainnets = registry.mainnet_chains();
        let testnets = registry.testnet_chains();

        assert!(mainnets.len() > 0);
        assert!(testnets.len() > 0);
        assert_eq!(mainnets.len() + testnets.len(), registry.count());

        // Check that all mainnets are not testnets
        for chain in mainnets {
            assert!(!chain.is_testnet);
        }

        // Check that all testnets are testnets
        for chain in testnets {
            assert!(chain.is_testnet);
        }
    }

    #[test]
    fn test_special_chains() {
        let registry = ChainRegistry::new();
        let special = registry.special_chains();

        // Optimism (10), Arbitrum (42161), zkSync Era (324) should be marked as special
        let special_ids: Vec<u64> = special.iter().map(|c| c.chain_id).collect();

        if registry.is_supported(10) {
            assert!(special_ids.contains(&10), "Optimism should be special");
        }
        if registry.is_supported(42161) {
            assert!(special_ids.contains(&42161), "Arbitrum should be special");
        }
        if registry.is_supported(324) {
            assert!(special_ids.contains(&324), "zkSync Era should be special");
        }
    }

    #[test]
    fn test_search() {
        let registry = ChainRegistry::new();

        let eth_results = registry.search("ethereum");
        assert!(eth_results.len() > 0);
        assert!(eth_results.iter().any(|c| c.chain_id == 1));

        let polygon_results = registry.search("polygon");
        if registry.is_supported(137) {
            assert!(polygon_results.iter().any(|c| c.chain_id == 137));
        }
    }

    #[test]
    fn test_global_singleton() {
        let registry1 = ChainRegistry::global();
        let registry2 = ChainRegistry::global();

        // Should be the same instance
        assert_eq!(
            registry1 as *const ChainRegistry,
            registry2 as *const ChainRegistry
        );
    }

    #[test]
    fn test_all_chain_ids_sorted() {
        let registry = ChainRegistry::new();
        let ids = registry.all_chain_ids();

        // Check that IDs are sorted
        for i in 1..ids.len() {
            assert!(ids[i] > ids[i - 1], "Chain IDs should be sorted");
        }
    }
}
