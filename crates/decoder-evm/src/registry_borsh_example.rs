//! Example of using Borsh-serialized chain registry
//!
//! This approach embeds a compact binary file instead of generating Rust code.

use crate::types::ChainInfo;
use borsh::BorshDeserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

// Embed the Borsh-serialized chain data at compile time
// This file is ~1-2MB instead of ~3-5MB of generated Rust code
const CHAINS_BORSH: &[u8] = include_bytes!("../data/chains.borsh");

/// Global chain registry singleton
static CHAIN_REGISTRY: OnceLock<ChainRegistry> = OnceLock::new();

/// Serializable wrapper for the chain registry
#[derive(borsh::BorshDeserialize)]
struct SerializedRegistry {
    chains: HashMap<u64, ChainInfo>,
}

/// Registry of EVM-compatible chains
pub struct ChainRegistry {
    chains: HashMap<u64, ChainInfo>,
    by_short_name: HashMap<String, u64>,
}

impl ChainRegistry {
    /// Get the global chain registry instance
    pub fn global() -> &'static ChainRegistry {
        CHAIN_REGISTRY.get_or_init(|| {
            Self::from_borsh(CHAINS_BORSH)
                .expect("Failed to deserialize embedded chain registry")
        })
    }

    /// Deserialize from Borsh bytes
    fn from_borsh(bytes: &[u8]) -> Result<Self, borsh::io::Error> {
        let serialized: SerializedRegistry = SerializedRegistry::try_from_slice(bytes)?;

        // Build reverse index by short name
        let by_short_name: HashMap<String, u64> = serialized.chains
            .iter()
            .map(|(id, info)| (info.short_name.clone(), *id))
            .collect();

        Ok(Self {
            chains: serialized.chains,
            by_short_name,
        })
    }

    // ... rest of the methods stay the same ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_borsh() {
        // Deserialize the embedded binary
        let registry = ChainRegistry::from_borsh(CHAINS_BORSH).unwrap();

        assert!(registry.chains.len() > 2000);

        // Verify Ethereum mainnet
        let eth = registry.chains.get(&1).unwrap();
        assert_eq!(eth.name, "Ethereum Mainnet");
        assert_eq!(eth.short_name, "eth");
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
}
