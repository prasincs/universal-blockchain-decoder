//! Move chain registry
//!
//! Hardcoded list of Move-based blockchains with metadata.
//!
//! This registry includes all known Move VM based chains:
//! - Aptos: Layer 1 Move blockchain
//! - Sui: Object-centric Move blockchain
//! - Movement: Move on EVM (planned)

use serde::{Deserialize, Serialize};

/// Move chain identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveChainId {
    /// Aptos Mainnet (chain ID 1)
    AptosMainnet,
    /// Aptos Testnet (chain ID 2)
    AptosTestnet,
    /// Aptos Devnet
    AptosDevnet,
    /// Sui Mainnet
    SuiMainnet,
    /// Sui Testnet
    SuiTestnet,
    /// Sui Devnet
    SuiDevnet,
    /// Movement (Move on EVM) - planned
    Movement,
}

impl MoveChainId {
    /// Get the numeric chain ID
    pub fn to_u64(self) -> u64 {
        match self {
            MoveChainId::AptosMainnet => 1,
            MoveChainId::AptosTestnet => 2,
            MoveChainId::AptosDevnet => 3,
            MoveChainId::SuiMainnet => 100,
            MoveChainId::SuiTestnet => 101,
            MoveChainId::SuiDevnet => 102,
            MoveChainId::Movement => 1000,
        }
    }

    /// Get the human-readable chain name
    pub fn name(self) -> &'static str {
        match self {
            MoveChainId::AptosMainnet => "Aptos Mainnet",
            MoveChainId::AptosTestnet => "Aptos Testnet",
            MoveChainId::AptosDevnet => "Aptos Devnet",
            MoveChainId::SuiMainnet => "Sui Mainnet",
            MoveChainId::SuiTestnet => "Sui Testnet",
            MoveChainId::SuiDevnet => "Sui Devnet",
            MoveChainId::Movement => "Movement",
        }
    }

    /// Check if this is an Aptos chain
    pub fn is_aptos(self) -> bool {
        matches!(
            self,
            MoveChainId::AptosMainnet | MoveChainId::AptosTestnet | MoveChainId::AptosDevnet
        )
    }

    /// Check if this is a Sui chain
    pub fn is_sui(self) -> bool {
        matches!(
            self,
            MoveChainId::SuiMainnet | MoveChainId::SuiTestnet | MoveChainId::SuiDevnet
        )
    }

    /// Check if this is a mainnet chain
    pub fn is_mainnet(self) -> bool {
        matches!(self, MoveChainId::AptosMainnet | MoveChainId::SuiMainnet)
    }

    /// Get chain variant (Aptos, Sui, Movement)
    pub fn variant(self) -> MoveVariant {
        match self {
            MoveChainId::AptosMainnet | MoveChainId::AptosTestnet | MoveChainId::AptosDevnet => {
                MoveVariant::Aptos
            }
            MoveChainId::SuiMainnet | MoveChainId::SuiTestnet | MoveChainId::SuiDevnet => {
                MoveVariant::Sui
            }
            MoveChainId::Movement => MoveVariant::Movement,
        }
    }
}

/// Move chain variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveVariant {
    /// Aptos-based chains (account model with BCS)
    Aptos,
    /// Sui-based chains (object model with BCS)
    Sui,
    /// Movement (Move on EVM) - planned
    Movement,
}

/// Move chain information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveChainInfo {
    pub chain_id: MoveChainId,
    pub name: &'static str,
    pub variant: MoveVariant,
    pub rpc_url: Option<&'static str>,
    pub explorer_url: Option<&'static str>,
}

impl MoveChainInfo {
    /// Get chain info from chain ID
    pub fn from_chain_id(chain_id: MoveChainId) -> Self {
        MOVE_CHAIN_REGISTRY
            .iter()
            .find(|info| info.chain_id == chain_id)
            .cloned()
            .unwrap_or_else(|| panic!("Chain ID {:?} not found in registry", chain_id))
    }
}

/// Hardcoded registry of all Move chains
pub const MOVE_CHAIN_REGISTRY: &[MoveChainInfo] = &[
    // Aptos chains
    MoveChainInfo {
        chain_id: MoveChainId::AptosMainnet,
        name: "Aptos Mainnet",
        variant: MoveVariant::Aptos,
        rpc_url: Some("https://fullnode.mainnet.aptoslabs.com/v1"),
        explorer_url: Some("https://explorer.aptoslabs.com"),
    },
    MoveChainInfo {
        chain_id: MoveChainId::AptosTestnet,
        name: "Aptos Testnet",
        variant: MoveVariant::Aptos,
        rpc_url: Some("https://fullnode.testnet.aptoslabs.com/v1"),
        explorer_url: Some("https://explorer.aptoslabs.com/testnet"),
    },
    MoveChainInfo {
        chain_id: MoveChainId::AptosDevnet,
        name: "Aptos Devnet",
        variant: MoveVariant::Aptos,
        rpc_url: Some("https://fullnode.devnet.aptoslabs.com/v1"),
        explorer_url: Some("https://explorer.aptoslabs.com/devnet"),
    },
    // Sui chains
    MoveChainInfo {
        chain_id: MoveChainId::SuiMainnet,
        name: "Sui Mainnet",
        variant: MoveVariant::Sui,
        rpc_url: Some("https://fullnode.mainnet.sui.io:443"),
        explorer_url: Some("https://suiscan.xyz/mainnet"),
    },
    MoveChainInfo {
        chain_id: MoveChainId::SuiTestnet,
        name: "Sui Testnet",
        variant: MoveVariant::Sui,
        rpc_url: Some("https://fullnode.testnet.sui.io:443"),
        explorer_url: Some("https://suiscan.xyz/testnet"),
    },
    MoveChainInfo {
        chain_id: MoveChainId::SuiDevnet,
        name: "Sui Devnet",
        variant: MoveVariant::Sui,
        rpc_url: Some("https://fullnode.devnet.sui.io:443"),
        explorer_url: Some("https://suiscan.xyz/devnet"),
    },
    // Movement (planned)
    MoveChainInfo {
        chain_id: MoveChainId::Movement,
        name: "Movement",
        variant: MoveVariant::Movement,
        rpc_url: None,
        explorer_url: None,
    },
];

/// Move chain registry
pub struct MoveChainRegistry;

impl MoveChainRegistry {
    /// Get all Move chains
    pub fn all_chains() -> &'static [MoveChainInfo] {
        MOVE_CHAIN_REGISTRY
    }

    /// Get chain info by chain ID
    pub fn get_chain(chain_id: MoveChainId) -> &'static MoveChainInfo {
        MOVE_CHAIN_REGISTRY
            .iter()
            .find(|info| info.chain_id == chain_id)
            .expect("Chain ID not found in registry")
    }

    /// Get all Aptos chains
    pub fn aptos_chains() -> impl Iterator<Item = &'static MoveChainInfo> {
        MOVE_CHAIN_REGISTRY
            .iter()
            .filter(|info| info.chain_id.is_aptos())
    }

    /// Get all Sui chains
    pub fn sui_chains() -> impl Iterator<Item = &'static MoveChainInfo> {
        MOVE_CHAIN_REGISTRY
            .iter()
            .filter(|info| info.chain_id.is_sui())
    }

    /// Get all mainnet chains
    pub fn mainnet_chains() -> impl Iterator<Item = &'static MoveChainInfo> {
        MOVE_CHAIN_REGISTRY
            .iter()
            .filter(|info| info.chain_id.is_mainnet())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_conversion() {
        assert_eq!(MoveChainId::AptosMainnet.to_u64(), 1);
        assert_eq!(MoveChainId::SuiMainnet.to_u64(), 100);
    }

    #[test]
    fn test_chain_name() {
        assert_eq!(MoveChainId::AptosMainnet.name(), "Aptos Mainnet");
        assert_eq!(MoveChainId::SuiMainnet.name(), "Sui Mainnet");
    }

    #[test]
    fn test_is_aptos() {
        assert!(MoveChainId::AptosMainnet.is_aptos());
        assert!(MoveChainId::AptosTestnet.is_aptos());
        assert!(!MoveChainId::SuiMainnet.is_aptos());
    }

    #[test]
    fn test_is_sui() {
        assert!(MoveChainId::SuiMainnet.is_sui());
        assert!(MoveChainId::SuiTestnet.is_sui());
        assert!(!MoveChainId::AptosMainnet.is_sui());
    }

    #[test]
    fn test_is_mainnet() {
        assert!(MoveChainId::AptosMainnet.is_mainnet());
        assert!(MoveChainId::SuiMainnet.is_mainnet());
        assert!(!MoveChainId::AptosTestnet.is_mainnet());
        assert!(!MoveChainId::SuiTestnet.is_mainnet());
    }

    #[test]
    fn test_variant() {
        assert_eq!(MoveChainId::AptosMainnet.variant(), MoveVariant::Aptos);
        assert_eq!(MoveChainId::SuiMainnet.variant(), MoveVariant::Sui);
        assert_eq!(MoveChainId::Movement.variant(), MoveVariant::Movement);
    }

    #[test]
    fn test_registry_all_chains() {
        let chains = MoveChainRegistry::all_chains();
        assert_eq!(chains.len(), 7); // 3 Aptos + 3 Sui + 1 Movement
    }

    #[test]
    fn test_registry_aptos_chains() {
        let aptos_chains: Vec<_> = MoveChainRegistry::aptos_chains().collect();
        assert_eq!(aptos_chains.len(), 3); // Mainnet, Testnet, Devnet
    }

    #[test]
    fn test_registry_sui_chains() {
        let sui_chains: Vec<_> = MoveChainRegistry::sui_chains().collect();
        assert_eq!(sui_chains.len(), 3); // Mainnet, Testnet, Devnet
    }

    #[test]
    fn test_registry_mainnet_chains() {
        let mainnet_chains: Vec<_> = MoveChainRegistry::mainnet_chains().collect();
        assert_eq!(mainnet_chains.len(), 2); // Aptos Mainnet + Sui Mainnet
    }
}
