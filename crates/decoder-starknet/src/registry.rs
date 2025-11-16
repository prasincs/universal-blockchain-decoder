//! Starknet chain registry
//!
//! Provides chain information for different Starknet networks:
//! - Mainnet
//! - Sepolia Testnet
//! - Appchains (Kakarot, Madara-based chains, etc.)

use decoder_crypto_zk::FieldElement;
use std::collections::HashMap;

/// Starknet chain information
#[derive(Debug, Clone)]
pub struct StarknetChainInfo {
    /// Chain ID (ASCII encoding or numeric)
    pub chain_id: u64,
    /// Human-readable chain name
    pub name: String,
    /// Network type (mainnet, testnet, appchain)
    pub network_type: NetworkType,
    /// Chain ID as FieldElement (for transaction hashing)
    pub chain_id_felt: FieldElement,
}

/// Network type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Appchain,
}

/// Starknet chain registry
pub struct StarknetRegistry {
    chains: HashMap<u64, StarknetChainInfo>,
}

impl StarknetRegistry {
    /// Create a new registry with default chains
    pub fn new() -> Self {
        let mut chains = HashMap::new();

        // Starknet Mainnet
        chains.insert(
            23448594291968336,
            StarknetChainInfo {
                chain_id: 23448594291968336,
                name: "Starknet Mainnet".to_string(),
                network_type: NetworkType::Mainnet,
                chain_id_felt: FieldElement::from_hex_be(
                    "0x534e5f4d41494e", // "SN_MAIN" in ASCII
                )
                .unwrap(),
            },
        );

        // Starknet Sepolia Testnet
        chains.insert(
            393402133025997801415703418429829435,
            StarknetChainInfo {
                chain_id: 393402133025997801415703418429829435,
                name: "Starknet Sepolia".to_string(),
                network_type: NetworkType::Testnet,
                chain_id_felt: FieldElement::from_hex_be(
                    "0x534e5f5345504f4c4941", // "SN_SEPOLIA" in ASCII
                )
                .unwrap(),
            },
        );

        Self { chains }
    }

    /// Get chain info by chain ID
    pub fn get(&self, chain_id: u64) -> Option<&StarknetChainInfo> {
        self.chains.get(&chain_id)
    }

    /// Get mainnet info
    pub fn mainnet() -> StarknetChainInfo {
        StarknetChainInfo {
            chain_id: 23448594291968336,
            name: "Starknet Mainnet".to_string(),
            network_type: NetworkType::Mainnet,
            chain_id_felt: FieldElement::from_hex_be("0x534e5f4d41494e").unwrap(),
        }
    }

    /// Get Sepolia testnet info
    pub fn sepolia() -> StarknetChainInfo {
        StarknetChainInfo {
            chain_id: 393402133025997801415703418429829435,
            name: "Starknet Sepolia".to_string(),
            network_type: NetworkType::Testnet,
            chain_id_felt: FieldElement::from_hex_be("0x534e5f5345504f4c4941").unwrap(),
        }
    }

    /// Add custom appchain
    pub fn add_appchain(&mut self, chain_info: StarknetChainInfo) {
        self.chains.insert(chain_info.chain_id, chain_info);
    }

    /// List all registered chains
    pub fn all_chains(&self) -> Vec<&StarknetChainInfo> {
        self.chains.values().collect()
    }
}

impl Default for StarknetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_mainnet() {
        let registry = StarknetRegistry::new();
        let mainnet = registry.get(23448594291968336).unwrap();
        assert_eq!(mainnet.name, "Starknet Mainnet");
        assert_eq!(mainnet.network_type, NetworkType::Mainnet);
    }

    #[test]
    fn test_registry_sepolia() {
        let registry = StarknetRegistry::new();
        let sepolia = registry.get(393402133025997801415703418429829435).unwrap();
        assert_eq!(sepolia.name, "Starknet Sepolia");
        assert_eq!(sepolia.network_type, NetworkType::Testnet);
    }

    #[test]
    fn test_mainnet_helper() {
        let mainnet = StarknetRegistry::mainnet();
        assert_eq!(mainnet.chain_id, 23448594291968336);
    }

    #[test]
    fn test_sepolia_helper() {
        let sepolia = StarknetRegistry::sepolia();
        assert_eq!(sepolia.chain_id, 393402133025997801415703418429829435);
    }

    #[test]
    fn test_add_custom_appchain() {
        let mut registry = StarknetRegistry::new();

        let kakarot = StarknetChainInfo {
            chain_id: 12345,
            name: "Kakarot zkEVM".to_string(),
            network_type: NetworkType::Appchain,
            chain_id_felt: FieldElement::from(12345u64),
        };

        registry.add_appchain(kakarot);

        let retrieved = registry.get(12345).unwrap();
        assert_eq!(retrieved.name, "Kakarot zkEVM");
    }

    #[test]
    fn test_all_chains() {
        let registry = StarknetRegistry::new();
        let chains = registry.all_chains();
        assert_eq!(chains.len(), 2); // Mainnet + Sepolia
    }
}
