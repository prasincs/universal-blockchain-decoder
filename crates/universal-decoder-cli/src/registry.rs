//! Dynamic decoder registry for chain-agnostic transaction decoding.
//!
//! This module provides a registry pattern to discover and dispatch to available
//! blockchain decoders without hardcoded enums.

use anyhow::{anyhow, Result};
use universal_decoder_core::prelude::*;

/// Metadata about a registered chain decoder
#[derive(Debug, Clone)]
pub struct ChainInfo {
    /// Chain identifier (e.g., 0 for Bitcoin, 1 for Ethereum)
    pub chain_id: u64,
    /// Human-readable name (e.g., "Bitcoin", "Ethereum")
    pub name: &'static str,
    /// Short name for CLI (e.g., "btc", "eth")
    pub short_name: &'static str,
    /// Chain family (UTXO, Account, Privacy, etc.)
    pub family: ChainFamily,
    /// Whether this chain supports privacy features (viewing keys, shielded txs)
    pub has_privacy_features: bool,
    /// Network type (mainnet, testnet, etc.)
    #[allow(dead_code)]
    pub network: Option<&'static str>,
}

impl ChainInfo {
    pub const fn new(
        chain_id: u64,
        name: &'static str,
        short_name: &'static str,
        family: ChainFamily,
        has_privacy_features: bool,
        network: Option<&'static str>,
    ) -> Self {
        Self {
            chain_id,
            name,
            short_name,
            family,
            has_privacy_features,
            network,
        }
    }
}

/// Result of decoding a transaction (chain-agnostic)
#[allow(dead_code)]
pub struct DecodedTransaction {
    pub chain_info: ChainInfo,
    pub tx_ir: Box<dyn std::any::Any>,
    pub canonical_ir: Option<TxIR<'static, 1>>,
}

/// Chain decoder registry for dynamic dispatch
pub struct DecoderRegistry {
    chains: Vec<ChainInfo>,
}

impl DecoderRegistry {
    /// Create a new registry with all available decoders
    pub fn new() -> Self {
        let chains = vec![
            // Bitcoin and forks
            ChainInfo::new(
                0,
                "Bitcoin",
                "btc",
                ChainFamily::Utxo,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                2,
                "Litecoin",
                "ltc",
                ChainFamily::Utxo,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                3,
                "Dogecoin",
                "doge",
                ChainFamily::Utxo,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(5, "Dash", "dash", ChainFamily::Utxo, false, Some("mainnet")),
            ChainInfo::new(
                145,
                "Bitcoin Cash",
                "bch",
                ChainFamily::Utxo,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                236,
                "Bitcoin SV",
                "bsv",
                ChainFamily::Utxo,
                false,
                Some("mainnet"),
            ),
            // Privacy chains
            ChainInfo::new(
                133,
                "Zcash",
                "zec",
                ChainFamily::Privacy,
                true,
                Some("mainnet"),
            ),
            // Ethereum and EVM chains
            ChainInfo::new(
                1,
                "Ethereum",
                "eth",
                ChainFamily::Account,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                56,
                "BNB Smart Chain",
                "bnb",
                ChainFamily::Account,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                137,
                "Polygon",
                "matic",
                ChainFamily::Account,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                43114,
                "Avalanche C-Chain",
                "avax",
                ChainFamily::Account,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                10,
                "Optimism",
                "op",
                ChainFamily::Account,
                false,
                Some("mainnet"),
            ),
            ChainInfo::new(
                42161,
                "Arbitrum One",
                "arb",
                ChainFamily::Account,
                false,
                Some("mainnet"),
            ),
            // Solana ecosystem (SVM)
            ChainInfo::new(
                900,
                "Solana",
                "sol",
                ChainFamily::Instruction,
                false,
                Some("mainnet"),
            ),
        ];

        Self { chains }
    }

    /// Find chain by short name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Result<&ChainInfo> {
        let name_lower = name.to_lowercase();
        self.chains
            .iter()
            .find(|c| c.short_name == name_lower || c.name.to_lowercase() == name_lower)
            .ok_or_else(|| anyhow!("Unknown chain: {}", name))
    }

    /// Find chain by ID
    pub fn find_by_id(&self, chain_id: u64) -> Result<&ChainInfo> {
        self.chains
            .iter()
            .find(|c| c.chain_id == chain_id)
            .ok_or_else(|| anyhow!("Unknown chain ID: {}", chain_id))
    }

    /// List all supported chains
    pub fn list_chains(&self) -> &[ChainInfo] {
        &self.chains
    }

    /// List only privacy-enabled chains
    pub fn list_privacy_chains(&self) -> Vec<&ChainInfo> {
        self.chains
            .iter()
            .filter(|c| c.has_privacy_features)
            .collect()
    }

    /// Decode a transaction based on chain info
    #[allow(dead_code)]
    pub fn decode(&self, chain_info: &ChainInfo, raw_bytes: &[u8]) -> Result<Vec<u8>> {
        // Dispatch to appropriate decoder based on chain_id
        match chain_info.chain_id {
            // Bitcoin and forks (UTXO family)
            0 => {
                let tx = decoder_bitcoin::BitcoinDecoder::decode(raw_bytes)?;
                // Return encoded transaction for display (simplified)
                Ok(format!("{:?}", tx).into_bytes())
            }
            2 => {
                let tx = decoder_litecoin::LitecoinDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            3 => {
                let tx = decoder_dogecoin::DogecoinDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            5 => {
                let tx = decoder_dash::DashDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            145 => {
                let tx = decoder_bitcoin_cash::BitcoinCashDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            236 => {
                let tx = decoder_bitcoin_sv::BitcoinSvDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            // Privacy chains
            133 => {
                let tx = decoder_zcash::ZcashDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            // Ethereum and EVM chains
            1 => {
                let tx = decoder_ethereum::EthereumDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            56 => {
                let tx = decoder_bnb::BnbDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            137 => {
                let tx = decoder_polygon::PolygonDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            43114 => {
                let tx = decoder_avalanche::AvalancheDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            10 => {
                let tx = decoder_optimism::OptimismDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            42161 => {
                let tx = decoder_arbitrum::ArbitrumDecoder::decode(raw_bytes)?;
                Ok(format!("{:?}", tx).into_bytes())
            }
            _ => Err(anyhow!(
                "Decoder not yet implemented for chain ID: {}",
                chain_info.chain_id
            )),
        }
    }
}

impl Default for DecoderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_by_name() {
        let registry = DecoderRegistry::new();

        // Case insensitive
        assert!(registry.find_by_name("bitcoin").is_ok());
        assert!(registry.find_by_name("BITCOIN").is_ok());
        assert!(registry.find_by_name("Bitcoin").is_ok());

        // Short names
        assert!(registry.find_by_name("btc").is_ok());
        assert!(registry.find_by_name("eth").is_ok());
        assert!(registry.find_by_name("zec").is_ok());

        // Unknown chains
        assert!(registry.find_by_name("unknown").is_err());
    }

    #[test]
    fn test_find_by_id() {
        let registry = DecoderRegistry::new();

        assert_eq!(registry.find_by_id(0).unwrap().name, "Bitcoin");
        assert_eq!(registry.find_by_id(1).unwrap().name, "Ethereum");
        assert_eq!(registry.find_by_id(133).unwrap().name, "Zcash");

        assert!(registry.find_by_id(99999).is_err());
    }

    #[test]
    fn test_privacy_chains() {
        let registry = DecoderRegistry::new();
        let privacy_chains = registry.list_privacy_chains();

        assert!(!privacy_chains.is_empty());
        assert!(privacy_chains.iter().any(|c| c.name == "Zcash"));
    }

    #[test]
    fn test_chain_families() {
        let registry = DecoderRegistry::new();

        let btc = registry.find_by_name("btc").unwrap();
        assert_eq!(btc.family, ChainFamily::Utxo);

        let eth = registry.find_by_name("eth").unwrap();
        assert_eq!(eth.family, ChainFamily::Account);

        let zec = registry.find_by_name("zec").unwrap();
        assert_eq!(zec.family, ChainFamily::Privacy);
    }
}
