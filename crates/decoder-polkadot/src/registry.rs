//! Polkadot ecosystem chain registry
//!
//! Maintains information about Polkadot relay chain, Kusama, and major parachains.

use serde::{Deserialize, Serialize};

/// Polkadot ecosystem chain information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotChainInfo {
    /// Chain ID (genesis hash or parachain ID)
    pub chain_id: u32,
    /// Chain name
    pub name: String,
    /// Network type (relay, parachain)
    pub network_type: NetworkType,
    /// SS58 address prefix
    pub ss58_prefix: u16,
    /// Token symbol
    pub token_symbol: String,
    /// Token decimals
    pub decimals: u8,
}

/// Network type in Polkadot ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// Relay chain (Polkadot or Kusama)
    Relay,
    /// Parachain on Polkadot
    Parachain,
    /// Parachain on Kusama
    KusamaParachain,
}

/// Polkadot chain registry
pub struct PolkadotRegistry {
    chains: Vec<PolkadotChainInfo>,
}

impl PolkadotRegistry {
    /// Create a new registry with default chains
    pub fn new() -> Self {
        Self {
            chains: vec![
                // Polkadot relay chain
                PolkadotChainInfo {
                    chain_id: 0,
                    name: "Polkadot".to_string(),
                    network_type: NetworkType::Relay,
                    ss58_prefix: 0,
                    token_symbol: "DOT".to_string(),
                    decimals: 10,
                },
                // Kusama relay chain
                PolkadotChainInfo {
                    chain_id: 2,
                    name: "Kusama".to_string(),
                    network_type: NetworkType::Relay,
                    ss58_prefix: 2,
                    token_symbol: "KSM".to_string(),
                    decimals: 12,
                },
                // Acala (Polkadot parachain)
                PolkadotChainInfo {
                    chain_id: 2000,
                    name: "Acala".to_string(),
                    network_type: NetworkType::Parachain,
                    ss58_prefix: 10,
                    token_symbol: "ACA".to_string(),
                    decimals: 12,
                },
                // Moonbeam (Polkadot parachain)
                PolkadotChainInfo {
                    chain_id: 2004,
                    name: "Moonbeam".to_string(),
                    network_type: NetworkType::Parachain,
                    ss58_prefix: 1284,
                    token_symbol: "GLMR".to_string(),
                    decimals: 18,
                },
                // Astar (Polkadot parachain)
                PolkadotChainInfo {
                    chain_id: 2006,
                    name: "Astar".to_string(),
                    network_type: NetworkType::Parachain,
                    ss58_prefix: 5,
                    token_symbol: "ASTR".to_string(),
                    decimals: 18,
                },
                // Karura (Kusama parachain)
                PolkadotChainInfo {
                    chain_id: 2000,
                    name: "Karura".to_string(),
                    network_type: NetworkType::KusamaParachain,
                    ss58_prefix: 8,
                    token_symbol: "KAR".to_string(),
                    decimals: 12,
                },
                // Moonriver (Kusama parachain)
                PolkadotChainInfo {
                    chain_id: 2023,
                    name: "Moonriver".to_string(),
                    network_type: NetworkType::KusamaParachain,
                    ss58_prefix: 1285,
                    token_symbol: "MOVR".to_string(),
                    decimals: 18,
                },
            ],
        }
    }

    /// Get chain by ID
    pub fn get_chain(&self, chain_id: u32) -> Option<&PolkadotChainInfo> {
        self.chains.iter().find(|c| c.chain_id == chain_id)
    }

    /// Get chain by name
    pub fn get_chain_by_name(&self, name: &str) -> Option<&PolkadotChainInfo> {
        self.chains
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// Get all chains
    pub fn all_chains(&self) -> &[PolkadotChainInfo] {
        &self.chains
    }

    /// Get relay chains only
    pub fn relay_chains(&self) -> Vec<&PolkadotChainInfo> {
        self.chains
            .iter()
            .filter(|c| c.network_type == NetworkType::Relay)
            .collect()
    }

    /// Get parachains only
    pub fn parachains(&self) -> Vec<&PolkadotChainInfo> {
        self.chains
            .iter()
            .filter(|c| {
                c.network_type == NetworkType::Parachain
                    || c.network_type == NetworkType::KusamaParachain
            })
            .collect()
    }
}

impl Default for PolkadotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = PolkadotRegistry::new();
        assert!(!registry.all_chains().is_empty());
        assert_eq!(registry.all_chains().len(), 7);
    }

    #[test]
    fn test_get_chain() {
        let registry = PolkadotRegistry::new();

        let polkadot = registry.get_chain(0);
        assert!(polkadot.is_some());
        assert_eq!(polkadot.unwrap().name, "Polkadot");
        assert_eq!(polkadot.unwrap().token_symbol, "DOT");

        let kusama = registry.get_chain(2);
        assert!(kusama.is_some());
        assert_eq!(kusama.unwrap().name, "Kusama");
        assert_eq!(kusama.unwrap().token_symbol, "KSM");
    }

    #[test]
    fn test_get_chain_by_name() {
        let registry = PolkadotRegistry::new();

        let moonbeam = registry.get_chain_by_name("Moonbeam");
        assert!(moonbeam.is_some());
        assert_eq!(moonbeam.unwrap().chain_id, 2004);

        let moonbeam_lower = registry.get_chain_by_name("moonbeam");
        assert!(moonbeam_lower.is_some());
    }

    #[test]
    fn test_relay_chains() {
        let registry = PolkadotRegistry::new();
        let relay_chains = registry.relay_chains();
        assert_eq!(relay_chains.len(), 2);
        assert!(relay_chains.iter().any(|c| c.name == "Polkadot"));
        assert!(relay_chains.iter().any(|c| c.name == "Kusama"));
    }

    #[test]
    fn test_parachains() {
        let registry = PolkadotRegistry::new();
        let parachains = registry.parachains();
        assert_eq!(parachains.len(), 5);
        assert!(parachains.iter().any(|c| c.name == "Acala"));
        assert!(parachains.iter().any(|c| c.name == "Moonbeam"));
        assert!(parachains.iter().any(|c| c.name == "Karura"));
    }
}
