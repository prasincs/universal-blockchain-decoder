//! EVM chain types and metadata

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Information about an EVM-compatible chain
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ChainInfo {
    /// Chain ID (EIP-155)
    pub chain_id: u64,
    /// Full chain name (e.g., "Ethereum Mainnet")
    pub name: String,
    /// Short name (e.g., "eth")
    pub short_name: String,
    /// Chain identifier (e.g., "ETH")
    pub chain: String,
    /// Network ID
    pub network_id: u64,
    /// Whether this is a testnet
    pub is_testnet: bool,
    /// Whether this chain has custom transaction types
    pub has_custom_tx_types: bool,
    /// Native currency information
    pub native_currency: CurrencyInfo,
    /// Information URL
    pub info_url: String,
    /// RPC endpoints
    pub rpc: Vec<String>,
    /// Block explorers
    pub explorers: Vec<ExplorerInfo>,
}

/// Native currency information
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct CurrencyInfo {
    /// Currency name (e.g., "Ether")
    pub name: String,
    /// Currency symbol (e.g., "ETH")
    pub symbol: String,
    /// Number of decimals
    pub decimals: u8,
}

/// Block explorer information
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ExplorerInfo {
    /// Explorer name (e.g., "etherscan")
    pub name: String,
    /// Explorer URL
    pub url: String,
    /// Standard (e.g., "EIP3091")
    pub standard: String,
}

impl ChainInfo {
    /// Check if this chain has custom transaction types that require special handling
    pub fn requires_special_decoder(&self) -> bool {
        self.has_custom_tx_types
    }

    /// Get the primary RPC endpoint
    pub fn primary_rpc(&self) -> Option<&str> {
        self.rpc.first().map(|s| s.as_str())
    }

    /// Get the primary block explorer
    pub fn primary_explorer(&self) -> Option<&ExplorerInfo> {
        self.explorers.first()
    }

    /// Format the transaction URL for the primary explorer
    pub fn tx_url(&self, tx_hash: &str) -> Option<String> {
        self.primary_explorer().map(|explorer| {
            format!("{}/tx/{}", explorer.url, tx_hash)
        })
    }

    /// Format the address URL for the primary explorer
    pub fn address_url(&self, address: &str) -> Option<String> {
        self.primary_explorer().map(|explorer| {
            format!("{}/address/{}", explorer.url, address)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_info_methods() {
        let chain = ChainInfo {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            short_name: "eth".to_string(),
            chain: "ETH".to_string(),
            network_id: 1,
            is_testnet: false,
            has_custom_tx_types: false,
            native_currency: CurrencyInfo {
                name: "Ether".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            info_url: "https://ethereum.org".to_string(),
            rpc: vec!["https://mainnet.infura.io".to_string()],
            explorers: vec![ExplorerInfo {
                name: "etherscan".to_string(),
                url: "https://etherscan.io".to_string(),
                standard: "EIP3091".to_string(),
            }],
        };

        assert!(!chain.requires_special_decoder());
        assert_eq!(chain.primary_rpc(), Some("https://mainnet.infura.io"));
        assert_eq!(
            chain.tx_url("0x123"),
            Some("https://etherscan.io/tx/0x123".to_string())
        );
        assert_eq!(
            chain.address_url("0xabc"),
            Some("https://etherscan.io/address/0xabc".to_string())
        );
    }

    #[test]
    fn test_special_decoder_detection() {
        let optimism = ChainInfo {
            chain_id: 10,
            name: "Optimism".to_string(),
            short_name: "oeth".to_string(),
            chain: "ETH".to_string(),
            network_id: 10,
            is_testnet: false,
            has_custom_tx_types: true,
            native_currency: CurrencyInfo {
                name: "Ether".to_string(),
                symbol: "ETH".to_string(),
                decimals: 18,
            },
            info_url: "https://optimism.io".to_string(),
            rpc: vec![],
            explorers: vec![],
        };

        assert!(optimism.requires_special_decoder());
    }
}
