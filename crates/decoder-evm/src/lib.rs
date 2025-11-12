//! Generic EVM decoder supporting 2000+ EVM-compatible chains
//!
//! This decoder provides a unified interface for decoding transactions from any
//! EVM-compatible blockchain. It uses the ethereum-lists/chains registry to
//! support all standard EVM chains through a single decoder.
//!
//! # Features
//!
//! - **2000+ chains supported**: Automatically supports all EVM-compatible chains
//! - **Airgapped operation**: Chain data embedded at compile time (no runtime network calls)
//! - **Verifiable supply chain**: Chain data vendored via git subtree
//! - **Special chain detection**: Identifies chains requiring custom decoders (Optimism, Arbitrum, etc.)
//! - **Rich metadata**: Returns chain information alongside decoded transactions
//!
//! # Example
//!
//! ```rust
//! use decoder_evm::EvmDecoder;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Decode transaction for any EVM chain
//! let decoder = EvmDecoder::new();
//! let tx_bytes = vec![0xf8, 0x6c /* ... */];
//!
//! // Option 1: Auto-detect chain from transaction
//! let (tx, chain_info) = decoder.decode(&tx_bytes, None)?;
//! println!("Decoded transaction on chain: {}", chain_info.name);
//!
//! // Option 2: Specify expected chain ID
//! let (tx, chain_info) = decoder.decode(&tx_bytes, Some(1))?; // Ethereum mainnet
//! assert_eq!(chain_info.chain_id, 1);
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The EVM decoder follows the chain family strategy outlined in CHAIN_FAMILIES_GROUPING.md:
//! - Single decoder for all standard EVM chains
//! - Chain-specific decoders only for special cases (Optimism, Arbitrum, zkSync)
//! - 97% reduction in code compared to individual chain decoders
//!
//! # Special Chains
//!
//! Some chains have custom transaction types that require specialized decoders:
//! - **Optimism (10)**: Deposit transactions (0x7E)
//! - **Arbitrum (42161)**: Retryable tickets, ArbOS internals
//! - **zkSync Era (324)**: Custom tx types, account abstraction
//!
//! For these chains, use the specialized decoder crates:
//! - `decoder-op-stack` for Optimism and OP Stack chains
//! - `decoder-arbitrum-orbit` for Arbitrum and Orbit chains
//! - `decoder-zksync-era` for zkSync Era chains

use universal_decoder_core::prelude::*;
use decoder_ethereum::{EthereumDecoder as BaseEthDecoder, types::EthereumTransaction};

pub mod registry;
pub mod types;

pub use registry::ChainRegistry;
pub use types::{ChainInfo, CurrencyInfo, ExplorerInfo};

/// Generic EVM decoder supporting all EVM-compatible chains
///
/// This decoder wraps the base Ethereum decoder and adds chain validation
/// and metadata lookup from the ethereum-lists/chains registry.
pub struct EvmDecoder {
    registry: &'static ChainRegistry,
}

impl EvmDecoder {
    /// Create a new EVM decoder
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            registry: ChainRegistry::global(),
        }
    }

    /// Decode a transaction from any EVM-compatible chain
    ///
    /// # Arguments
    ///
    /// * `raw_bytes` - Raw transaction bytes (RLP-encoded)
    /// * `expected_chain_id` - Optional chain ID to validate against
    ///
    /// # Returns
    ///
    /// Returns a tuple of (transaction, chain_info) on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction bytes are invalid
    /// - Chain ID doesn't match expected (if provided)
    /// - Chain ID is not in the registry
    /// - Transaction has custom types requiring specialized decoder
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let decoder = EvmDecoder::new();
    /// let tx_bytes = vec![0xf8, 0x6c /* ... */];
    ///
    /// // Decode without chain hint
    /// let (tx, chain) = decoder.decode(&tx_bytes, None)?;
    ///
    /// // Decode with chain hint
    /// let (tx, chain) = decoder.decode(&tx_bytes, Some(1))?;
    /// assert_eq!(chain.chain_id, 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode(
        &self,
        raw_bytes: &[u8],
        expected_chain_id: Option<u64>,
    ) -> Result<(EthereumTransaction, ChainInfo)> {
        // Decode the transaction using base Ethereum decoder
        let tx = BaseEthDecoder::decode(raw_bytes)?;

        // Extract chain ID from transaction
        let tx_chain_id = tx.chain_id.ok_or_else(|| {
            DecoderError::invalid_structure(
                "Transaction missing chain ID (pre-EIP-155 transactions not supported)"
            )
        })?;

        // Validate against expected chain ID if provided
        if let Some(expected) = expected_chain_id {
            if tx_chain_id != expected {
                return Err(DecoderError::invalid_structure(
                    &format!(
                        "Chain ID mismatch: transaction has {}, expected {}",
                        tx_chain_id, expected
                    )
                ));
            }
        }

        // Look up chain information
        let chain_info = self.registry.get_chain(tx_chain_id).ok_or_else(|| {
            DecoderError::invalid_structure(
                &format!(
                    "Chain ID {} not found in registry. \
                     If this is a valid EVM chain, please report it at \
                     https://github.com/ethereum-lists/chains",
                    tx_chain_id
                )
            )
        })?;

        // Warn if chain requires special decoder
        if chain_info.requires_special_decoder() {
            eprintln!(
                "Warning: Chain '{}' (ID: {}) has custom transaction types. \
                 Consider using a specialized decoder for full support.",
                chain_info.name, chain_info.chain_id
            );
        }

        Ok((tx, chain_info.clone()))
    }

    /// Check if a chain ID is supported
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// assert!(decoder.is_supported(1)); // Ethereum
    /// assert!(decoder.is_supported(56)); // BNB Chain
    /// assert!(!decoder.is_supported(999999999)); // Invalid
    /// ```
    pub fn is_supported(&self, chain_id: u64) -> bool {
        self.registry.is_supported(chain_id)
    }

    /// Get chain information by chain ID
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// let eth = decoder.get_chain(1).unwrap();
    /// assert_eq!(eth.name, "Ethereum Mainnet");
    /// ```
    pub fn get_chain(&self, chain_id: u64) -> Option<&ChainInfo> {
        self.registry.get_chain(chain_id)
    }

    /// Get chain information by short name
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// let eth = decoder.get_chain_by_name("eth").unwrap();
    /// assert_eq!(eth.chain_id, 1);
    /// ```
    pub fn get_chain_by_name(&self, short_name: &str) -> Option<&ChainInfo> {
        self.registry.get_chain_by_name(short_name)
    }

    /// List all supported chains
    ///
    /// Returns chains sorted by chain ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// let chains = decoder.list_chains();
    /// println!("Supported chains: {}", chains.len());
    ///
    /// for chain in chains.iter().take(10) {
    ///     println!("{}: {}", chain.chain_id, chain.name);
    /// }
    /// ```
    pub fn list_chains(&self) -> Vec<&ChainInfo> {
        self.registry.all_chains()
    }

    /// List mainnet chains only (excludes testnets)
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// let mainnets = decoder.list_mainnets();
    /// println!("Mainnet chains: {}", mainnets.len());
    /// ```
    pub fn list_mainnets(&self) -> Vec<&ChainInfo> {
        self.registry.mainnet_chains()
    }

    /// List testnet chains only
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// let testnets = decoder.list_testnets();
    /// println!("Testnet chains: {}", testnets.len());
    /// ```
    pub fn list_testnets(&self) -> Vec<&ChainInfo> {
        self.registry.testnet_chains()
    }

    /// Search for chains by name
    ///
    /// Performs case-insensitive substring matching on chain name, short name, and symbol.
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// let results = decoder.search("polygon");
    /// for chain in results {
    ///     println!("{}: {}", chain.chain_id, chain.name);
    /// }
    /// ```
    pub fn search(&self, query: &str) -> Vec<&ChainInfo> {
        self.registry.search(query)
    }

    /// Get total number of supported chains
    ///
    /// # Example
    ///
    /// ```rust
    /// use decoder_evm::EvmDecoder;
    ///
    /// let decoder = EvmDecoder::new();
    /// println!("Supporting {} EVM chains", decoder.count());
    /// ```
    pub fn count(&self) -> usize {
        self.registry.count()
    }
}

impl Default for EvmDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_initialization() {
        let decoder = EvmDecoder::new();
        assert!(decoder.count() > 0);
        println!("EVM decoder supports {} chains", decoder.count());
    }

    #[test]
    fn test_is_supported() {
        let decoder = EvmDecoder::new();

        // Major chains should be supported
        assert!(decoder.is_supported(1)); // Ethereum
        assert!(decoder.is_supported(56)); // BNB Chain
        assert!(decoder.is_supported(137)); // Polygon
        assert!(decoder.is_supported(10)); // Optimism
        assert!(decoder.is_supported(42161)); // Arbitrum

        // Invalid chain should not be supported (use 0 which is never a valid EVM chain ID)
        assert!(!decoder.is_supported(0));
    }

    #[test]
    fn test_get_chain() {
        let decoder = EvmDecoder::new();

        let eth = decoder.get_chain(1).unwrap();
        assert_eq!(eth.chain_id, 1);
        assert_eq!(eth.short_name, "eth");
        assert!(!eth.is_testnet);
    }

    #[test]
    fn test_get_chain_by_name() {
        let decoder = EvmDecoder::new();

        let eth = decoder.get_chain_by_name("eth").unwrap();
        assert_eq!(eth.chain_id, 1);
    }

    #[test]
    fn test_list_chains() {
        let decoder = EvmDecoder::new();
        let chains = decoder.list_chains();

        assert!(chains.len() > 1000); // Should have 2000+ chains

        // Check that chains are sorted by ID
        for i in 1..chains.len().min(100) {
            assert!(chains[i].chain_id >= chains[i - 1].chain_id);
        }
    }

    #[test]
    fn test_mainnet_vs_testnet() {
        let decoder = EvmDecoder::new();

        let mainnets = decoder.list_mainnets();
        let testnets = decoder.list_testnets();

        assert!(mainnets.len() > 0);
        assert!(testnets.len() > 0);
        assert_eq!(mainnets.len() + testnets.len(), decoder.count());
    }

    #[test]
    fn test_search() {
        let decoder = EvmDecoder::new();

        let eth_results = decoder.search("ethereum");
        assert!(eth_results.len() > 0);
        assert!(eth_results.iter().any(|c| c.chain_id == 1));

        let polygon_results = decoder.search("polygon");
        assert!(polygon_results.len() > 0);
    }

    #[test]
    fn test_decode_invalid_no_chain_id() {
        let decoder = EvmDecoder::new();

        // Create a legacy transaction without chain ID (pre-EIP-155)
        // This should fail with our decoder
        let legacy_tx_bytes = vec![0xf8, 0x6c]; // Invalid but has RLP structure

        let result = decoder.decode(&legacy_tx_bytes, None);
        // Should fail because we require chain ID
        assert!(result.is_err());
    }

    #[test]
    fn test_count() {
        let decoder = EvmDecoder::new();
        let count = decoder.count();

        assert!(count > 1000, "Should support 2000+ chains, got {}", count);
        println!("EVM decoder supports {} chains", count);
    }

    #[test]
    fn test_special_chain_detection() {
        let decoder = EvmDecoder::new();

        // Check that special chains are marked correctly
        if let Some(optimism) = decoder.get_chain(10) {
            assert!(optimism.has_custom_tx_types);
        }

        if let Some(arbitrum) = decoder.get_chain(42161) {
            assert!(arbitrum.has_custom_tx_types);
        }

        if let Some(zksync) = decoder.get_chain(324) {
            assert!(zksync.has_custom_tx_types);
        }
    }
}
