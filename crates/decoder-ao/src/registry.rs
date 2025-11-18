//! AO chain registry
//!
//! Arweave AO is a single hyper-parallel computer, but it can have different
//! network deployments (mainnet, testnet, etc.)

use serde::{Deserialize, Serialize};

/// AO network information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AONetwork {
    /// Network ID
    pub id: u64,

    /// Network name
    pub name: String,

    /// Network type (mainnet, testnet, devnet)
    pub network_type: String,

    /// Optional RPC endpoints
    pub rpc_endpoints: Vec<String>,

    /// Optional block explorer URL
    pub explorer_url: Option<String>,
}

/// Get all supported AO networks
pub fn get_ao_networks() -> Vec<AONetwork> {
    vec![
        AONetwork {
            id: 1000000, // Custom ID for AO mainnet
            name: "AO".to_string(),
            network_type: "mainnet".to_string(),
            rpc_endpoints: vec![
                "https://cu.ao-testnet.xyz".to_string(),
                "https://mu.ao-testnet.xyz".to_string(),
                "https://su.ao-testnet.xyz".to_string(),
            ],
            explorer_url: Some("https://ao.arweave.dev".to_string()),
        },
        AONetwork {
            id: 1000001, // Custom ID for AO testnet
            name: "AO Testnet".to_string(),
            network_type: "testnet".to_string(),
            rpc_endpoints: vec!["https://cu-testnet.ao-testnet.xyz".to_string()],
            explorer_url: Some("https://ao-testnet.arweave.dev".to_string()),
        },
    ]
}

/// Get AO network by ID
pub fn get_network_by_id(id: u64) -> Option<AONetwork> {
    get_ao_networks().into_iter().find(|n| n.id == id)
}

/// Get AO network by name
pub fn get_network_by_name(name: &str) -> Option<AONetwork> {
    get_ao_networks()
        .into_iter()
        .find(|n| n.name.to_lowercase() == name.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ao_networks() {
        let networks = get_ao_networks();
        assert!(!networks.is_empty());
        assert!(networks.iter().any(|n| n.network_type == "mainnet"));
    }

    #[test]
    fn test_get_network_by_id() {
        let network = get_network_by_id(1000000).unwrap();
        assert_eq!(network.name, "AO");
        assert_eq!(network.network_type, "mainnet");
    }

    #[test]
    fn test_get_network_by_name() {
        let network = get_network_by_name("AO").unwrap();
        assert_eq!(network.id, 1000000);

        let network_testnet = get_network_by_name("ao testnet").unwrap();
        assert_eq!(network_testnet.id, 1000001);
    }
}
