//! SVM (Solana Virtual Machine) chain registry
//!
//! This module provides information about all SVM-based chains.

/// SVM chain identifier
///
/// These IDs are used to identify different SVM-based chains.
/// Note: Some of these are unofficial and subject to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvmChainId {
    /// Solana Mainnet Beta
    SolanaMainnet = 101,

    /// Solana Devnet
    SolanaDevnet = 102,

    /// Solana Testnet
    SolanaTestnet = 103,

    /// Eclipse Mainnet (Ethereum-Solana hybrid)
    EclipseMainnet = 201,

    /// Eclipse Testnet
    EclipseTestnet = 202,

    /// Pyth Network
    PythNetwork = 301,

    /// Drift Protocol
    DriftProtocol = 401,

    /// Jito (MEV infrastructure)
    Jito = 501,

    /// Sonic SVM (gaming-focused L2)
    Sonic = 601,

    /// Firedancer (high-performance Solana validator)
    Firedancer = 701,

    /// Neon EVM (Ethereum compatibility on Solana)
    NeonEvm = 801,
}

impl SvmChainId {
    /// Convert from u64 chain ID
    pub fn from_u64(id: u64) -> Option<Self> {
        match id {
            101 => Some(Self::SolanaMainnet),
            102 => Some(Self::SolanaDevnet),
            103 => Some(Self::SolanaTestnet),
            201 => Some(Self::EclipseMainnet),
            202 => Some(Self::EclipseTestnet),
            301 => Some(Self::PythNetwork),
            401 => Some(Self::DriftProtocol),
            501 => Some(Self::Jito),
            601 => Some(Self::Sonic),
            701 => Some(Self::Firedancer),
            801 => Some(Self::NeonEvm),
            _ => None,
        }
    }

    /// Convert to u64
    pub fn to_u64(self) -> u64 {
        self as u64
    }

    /// Get chain name
    pub fn name(self) -> &'static str {
        match self {
            Self::SolanaMainnet => "Solana Mainnet",
            Self::SolanaDevnet => "Solana Devnet",
            Self::SolanaTestnet => "Solana Testnet",
            Self::EclipseMainnet => "Eclipse Mainnet",
            Self::EclipseTestnet => "Eclipse Testnet",
            Self::PythNetwork => "Pyth Network",
            Self::DriftProtocol => "Drift Protocol",
            Self::Jito => "Jito",
            Self::Sonic => "Sonic SVM",
            Self::Firedancer => "Firedancer",
            Self::NeonEvm => "Neon EVM",
        }
    }

    /// Check if this is a Solana chain (mainnet, devnet, or testnet)
    pub fn is_solana(self) -> bool {
        matches!(
            self,
            Self::SolanaMainnet | Self::SolanaDevnet | Self::SolanaTestnet
        )
    }

    /// Check if this is a mainnet chain
    pub fn is_mainnet(self) -> bool {
        matches!(
            self,
            Self::SolanaMainnet
                | Self::EclipseMainnet
                | Self::PythNetwork
                | Self::DriftProtocol
                | Self::Jito
                | Self::Sonic
                | Self::Firedancer
                | Self::NeonEvm
        )
    }

    /// Check if this is a testnet or devnet
    pub fn is_testnet(self) -> bool {
        matches!(
            self,
            Self::SolanaDevnet | Self::SolanaTestnet | Self::EclipseTestnet
        )
    }

    /// Get RPC endpoint (if known)
    ///
    /// Note: These are public RPC endpoints. For production use,
    /// you should use your own dedicated RPC nodes.
    pub fn rpc_endpoint(self) -> Option<&'static str> {
        match self {
            Self::SolanaMainnet => Some("https://api.mainnet-beta.solana.com"),
            Self::SolanaDevnet => Some("https://api.devnet.solana.com"),
            Self::SolanaTestnet => Some("https://api.testnet.solana.com"),
            _ => None, // Other chains don't have publicly known RPCs yet
        }
    }

    /// Get explorer URL template
    pub fn explorer_url(self) -> Option<&'static str> {
        match self {
            Self::SolanaMainnet => Some("https://explorer.solana.com/tx/{txid}"),
            Self::SolanaDevnet => Some("https://explorer.solana.com/tx/{txid}?cluster=devnet"),
            Self::SolanaTestnet => Some("https://explorer.solana.com/tx/{txid}?cluster=testnet"),
            _ => None,
        }
    }
}

/// SVM chain information
#[derive(Debug, Clone)]
pub struct SvmChainInfo {
    pub chain_id: SvmChainId,
    pub name: String,
    pub is_mainnet: bool,
    pub rpc_endpoint: Option<String>,
    pub explorer_url: Option<String>,
}

impl SvmChainInfo {
    /// Create chain info from chain ID
    pub fn from_chain_id(chain_id: SvmChainId) -> Self {
        Self {
            chain_id,
            name: chain_id.name().to_string(),
            is_mainnet: chain_id.is_mainnet(),
            rpc_endpoint: chain_id.rpc_endpoint().map(String::from),
            explorer_url: chain_id.explorer_url().map(String::from),
        }
    }
}

/// SVM chain registry
///
/// Provides access to information about all supported SVM chains.
pub struct SvmChainRegistry {
    chains: Vec<SvmChainInfo>,
}

impl SvmChainRegistry {
    /// Create a new registry with all known SVM chains
    pub fn new() -> Self {
        let chains = vec![
            SvmChainInfo::from_chain_id(SvmChainId::SolanaMainnet),
            SvmChainInfo::from_chain_id(SvmChainId::SolanaDevnet),
            SvmChainInfo::from_chain_id(SvmChainId::SolanaTestnet),
            SvmChainInfo::from_chain_id(SvmChainId::EclipseMainnet),
            SvmChainInfo::from_chain_id(SvmChainId::EclipseTestnet),
            SvmChainInfo::from_chain_id(SvmChainId::PythNetwork),
            SvmChainInfo::from_chain_id(SvmChainId::DriftProtocol),
            SvmChainInfo::from_chain_id(SvmChainId::Jito),
            SvmChainInfo::from_chain_id(SvmChainId::Sonic),
            SvmChainInfo::from_chain_id(SvmChainId::Firedancer),
            SvmChainInfo::from_chain_id(SvmChainId::NeonEvm),
        ];

        Self { chains }
    }

    /// Get chain info by chain ID
    pub fn get_chain(&self, chain_id: SvmChainId) -> Option<&SvmChainInfo> {
        self.chains.iter().find(|c| c.chain_id == chain_id)
    }

    /// Get chain info by numeric ID
    pub fn get_chain_by_id(&self, id: u64) -> Option<&SvmChainInfo> {
        let chain_id = SvmChainId::from_u64(id)?;
        self.get_chain(chain_id)
    }

    /// Get all chains
    pub fn all_chains(&self) -> impl Iterator<Item = &SvmChainInfo> {
        self.chains.iter()
    }

    /// Get number of chains in registry
    pub fn chain_count(&self) -> usize {
        self.chains.len()
    }

    /// Check if a chain ID exists
    pub fn has_chain(&self, chain_id: SvmChainId) -> bool {
        self.chains.iter().any(|c| c.chain_id == chain_id)
    }

    /// Get only mainnet chains
    pub fn mainnet_chains(&self) -> impl Iterator<Item = &SvmChainInfo> {
        self.chains.iter().filter(|c| c.is_mainnet)
    }

    /// Get only testnet/devnet chains
    pub fn testnet_chains(&self) -> impl Iterator<Item = &SvmChainInfo> {
        self.chains.iter().filter(|c| !c.is_mainnet)
    }
}

impl Default for SvmChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_conversion() {
        assert_eq!(SvmChainId::from_u64(101), Some(SvmChainId::SolanaMainnet));
        assert_eq!(SvmChainId::from_u64(102), Some(SvmChainId::SolanaDevnet));
        assert_eq!(SvmChainId::from_u64(201), Some(SvmChainId::EclipseMainnet));
        assert_eq!(SvmChainId::from_u64(999), None);

        assert_eq!(SvmChainId::SolanaMainnet.to_u64(), 101);
        assert_eq!(SvmChainId::EclipseMainnet.to_u64(), 201);
    }

    #[test]
    fn test_chain_properties() {
        assert!(SvmChainId::SolanaMainnet.is_solana());
        assert!(SvmChainId::SolanaMainnet.is_mainnet());
        assert!(!SvmChainId::SolanaMainnet.is_testnet());

        assert!(SvmChainId::SolanaDevnet.is_solana());
        assert!(!SvmChainId::SolanaDevnet.is_mainnet());
        assert!(SvmChainId::SolanaDevnet.is_testnet());

        assert!(!SvmChainId::EclipseMainnet.is_solana());
        assert!(SvmChainId::EclipseMainnet.is_mainnet());
    }

    #[test]
    fn test_registry_initialization() {
        let registry = SvmChainRegistry::new();
        assert!(registry.chain_count() >= 8, "Should have at least 8 chains");
    }

    #[test]
    fn test_registry_lookups() {
        let registry = SvmChainRegistry::new();

        let solana = registry.get_chain(SvmChainId::SolanaMainnet);
        assert!(solana.is_some());
        assert_eq!(solana.unwrap().name, "Solana Mainnet");

        let by_id = registry.get_chain_by_id(101);
        assert!(by_id.is_some());
        assert_eq!(by_id.unwrap().name, "Solana Mainnet");
    }

    #[test]
    fn test_registry_filters() {
        let registry = SvmChainRegistry::new();

        let mainnet_count = registry.mainnet_chains().count();
        let testnet_count = registry.testnet_chains().count();

        assert!(mainnet_count > 0, "Should have mainnet chains");
        assert!(testnet_count > 0, "Should have testnet chains");
        assert_eq!(
            mainnet_count + testnet_count,
            registry.chain_count(),
            "All chains should be either mainnet or testnet"
        );
    }

    #[test]
    fn test_rpc_endpoints() {
        assert!(SvmChainId::SolanaMainnet.rpc_endpoint().is_some());
        assert!(SvmChainId::SolanaDevnet.rpc_endpoint().is_some());
        assert!(SvmChainId::SolanaTestnet.rpc_endpoint().is_some());
    }

    #[test]
    fn test_explorer_urls() {
        let url = SvmChainId::SolanaMainnet.explorer_url();
        assert!(url.is_some());
        assert!(url.unwrap().contains("explorer.solana.com"));
    }
}
