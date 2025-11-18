//! SVM (Solana Virtual Machine) family decoder
//!
//! This module provides a unified decoder for all SVM-based blockchains, including:
//! - Solana (mainnet, devnet, testnet)
//! - Eclipse (Ethereum-Solana hybrid)
//! - Pyth Network (oracle network)
//! - Drift Protocol (derivatives DEX)
//! - Jito (MEV infrastructure)
//! - Future SVM-based chains
//!
//! ## Architecture
//!
//! The SVM decoder wraps the `decoder-solana` implementation and adds:
//! - Chain-specific identification via `SvmChainId`
//! - Chain registry with metadata (RPCs, explorers, etc.)
//! - Support for SVM variants with different features
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_svm::{SvmDecoder, SvmChainId};
//! use universal_decoder_core::prelude::*;
//!
//! // Decode a Solana mainnet transaction
//! let tx_bytes = &[...];
//! let tx = SvmDecoder::decode_with_chain(tx_bytes, SvmChainId::SolanaMainnet)?;
//!
//! // Auto-detect chain (defaults to Solana mainnet)
//! let tx = SvmDecoder::decode(tx_bytes)?;
//! ```
//!
//! ## Supported Chains
//!
//! - ✅ Solana Mainnet (chain ID 101)
//! - ✅ Solana Devnet (chain ID 102)
//! - ✅ Solana Testnet (chain ID 103)
//! - 🚧 Eclipse Mainnet (chain ID 201) - planned
//! - 🚧 Pyth Network (chain ID 301) - planned
//! - 🚧 Drift Protocol (chain ID 401) - planned
//! - 🚧 Jito (chain ID 501) - planned

use decoder_primitives::prelude::*;
use decoder_solana::{SolanaDecoder, SolanaTransaction};

pub mod registry;

pub use registry::{SvmChainId, SvmChainInfo, SvmChainRegistry};

/// SVM chain identity
///
/// This wraps a specific SVM chain (e.g., Solana Mainnet, Eclipse, etc.)
/// and provides chain-specific identification.
#[derive(Debug, Clone, Copy)]
pub struct SvmChain {
    chain_id: SvmChainId,
}

impl SvmChain {
    /// Create a new SVM chain identity
    pub fn new(chain_id: SvmChainId) -> Self {
        Self { chain_id }
    }

    /// Get the chain ID
    pub fn chain_id_enum(&self) -> SvmChainId {
        self.chain_id
    }
}

impl ChainIdentity for SvmChain {
    fn chain_id(&self) -> u64 {
        self.chain_id.to_u64()
    }

    fn chain_name(&self) -> &str {
        self.chain_id.name()
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account // All SVM chains use account-based model
    }
}

/// SVM transaction wrapper
///
/// Wraps a Solana transaction with chain-specific context.
#[derive(Debug, Clone)]
pub struct SvmTransaction {
    pub chain_id: SvmChainId,
    pub inner: SolanaTransaction,
}

impl SvmTransaction {
    /// Create a new SVM transaction
    pub fn new(chain_id: SvmChainId, inner: SolanaTransaction) -> Self {
        Self { chain_id, inner }
    }

    /// Get the wrapped Solana transaction
    pub fn solana_transaction(&self) -> &SolanaTransaction {
        &self.inner
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> SvmChainId {
        self.chain_id
    }

    /// Check if this is a Solana chain transaction
    pub fn is_solana(&self) -> bool {
        self.chain_id.is_solana()
    }

    /// Check if this is a mainnet transaction
    pub fn is_mainnet(&self) -> bool {
        self.chain_id.is_mainnet()
    }
}

impl ChainEncoder for SvmTransaction {
    /// Re-encode the SVM transaction back to its original byte format
    ///
    /// Since we store the original raw bytes during decoding, this simply
    /// returns a clone of those bytes, guaranteeing exact reconstruction.
    ///
    /// # Formal Properties
    ///
    /// This implementation trivially satisfies the injective property:
    /// ```text
    /// ∀ tx_bytes: SvmDecoder::decode(tx_bytes)?.to_bytes()? == tx_bytes
    /// ```
    ///
    /// Because we store `raw_bytes` during decode, the roundtrip is guaranteed.
    fn to_bytes(&self) -> Result<Vec<u8>> {
        // Delegate to the inner Solana transaction
        self.inner.to_bytes()
    }
}

impl<'a> Canonicalizer<'a> for SvmTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Delegate to the underlying Solana transaction's canonicalization
        // but override the chain identity
        let mut tx_ir = self.inner.canonicalize()?;

        // Update metadata to include SVM-specific chain information
        tx_ir.metadata.extra = format!(
            r#"{{"svm_chain":"{}","svm_chain_id":{},"is_solana":{},"is_mainnet":{}}}"#,
            self.chain_id.name(),
            self.chain_id.to_u64(),
            self.is_solana(),
            self.is_mainnet()
        );

        Ok(tx_ir)
    }

    fn validate(&self) -> Result<()> {
        // Validate the underlying Solana transaction
        self.inner.validate()
    }
}

/// SVM decoder implementing the ChainDecoder trait
///
/// This decoder wraps the Solana decoder and adds chain-specific context.
pub struct SvmDecoder {
    chain_id: SvmChainId,
}

impl SvmDecoder {
    /// Create a new SVM decoder for a specific chain
    pub fn new(chain_id: SvmChainId) -> Self {
        Self { chain_id }
    }

    /// Decode a transaction with explicit chain ID
    pub fn decode_with_chain(raw_bytes: &[u8], chain_id: SvmChainId) -> Result<SvmTransaction> {
        // Use the underlying Solana decoder
        let solana_tx = SolanaDecoder::decode(raw_bytes)?;

        // Wrap with chain context
        Ok(SvmTransaction::new(chain_id, solana_tx))
    }

    /// Get the chain this decoder is configured for
    pub fn chain_id(&self) -> SvmChainId {
        self.chain_id
    }
}

impl ChainDecoder for SvmDecoder {
    type TxSpecific = SvmTransaction;
    type Chain = SvmChain;

    fn chain() -> Self::Chain {
        // Default to Solana Mainnet
        SvmChain::new(SvmChainId::SolanaMainnet)
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Default to Solana Mainnet
        Self::decode_with_chain(raw_bytes, SvmChainId::SolanaMainnet)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Delegate to Solana decoder
        SolanaDecoder::validate_format(raw_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svm_chain_identity() {
        let solana = SvmChain::new(SvmChainId::SolanaMainnet);
        assert_eq!(solana.chain_id(), 101);
        assert_eq!(solana.chain_name(), "Solana Mainnet");
        assert_eq!(solana.chain_family(), ChainFamily::Account);

        let eclipse = SvmChain::new(SvmChainId::EclipseMainnet);
        assert_eq!(eclipse.chain_id(), 201);
        assert_eq!(eclipse.chain_name(), "Eclipse Mainnet");
    }

    #[test]
    fn test_svm_decoder_default_chain() {
        let chain = SvmDecoder::chain();
        assert_eq!(chain.chain_id(), 101);
        assert_eq!(chain.chain_name(), "Solana Mainnet");
    }

    #[test]
    fn test_svm_decoder_creation() {
        let decoder = SvmDecoder::new(SvmChainId::SolanaDevnet);
        assert_eq!(decoder.chain_id(), SvmChainId::SolanaDevnet);

        let mainnet_decoder = SvmDecoder::new(SvmChainId::SolanaMainnet);
        assert_eq!(mainnet_decoder.chain_id(), SvmChainId::SolanaMainnet);
    }

    #[test]
    fn test_validate_format_empty_input() {
        let result = SvmDecoder::validate_format(&[]);
        assert!(result.is_err(), "Empty input should fail validation");
    }

    #[test]
    fn test_validate_format_too_small() {
        let result = SvmDecoder::validate_format(&[0x01, 0x02]);
        assert!(result.is_err(), "Too small input should fail validation");
    }

    #[test]
    fn test_svm_transaction_properties() {
        use decoder_solana::types::{Message, MessageHeader};

        // Create a minimal Solana transaction
        let message = Message {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 0,
            },
            account_keys: vec![vec![0u8; 32]], // Vec<Vec<u8>>
            recent_blockhash: vec![0u8; 32],   // Vec<u8>
            instructions: vec![],
        };

        let solana_tx = SolanaTransaction {
            signatures: vec![vec![0u8; 64]],
            message,
            raw_bytes: vec![], // Add raw_bytes field
        };

        let svm_tx = SvmTransaction::new(SvmChainId::SolanaMainnet, solana_tx);

        assert_eq!(svm_tx.chain_id(), SvmChainId::SolanaMainnet);
        assert!(svm_tx.is_solana());
        assert!(svm_tx.is_mainnet());

        let eclipse_tx = SvmTransaction::new(SvmChainId::EclipseMainnet, svm_tx.inner.clone());
        assert_eq!(eclipse_tx.chain_id(), SvmChainId::EclipseMainnet);
        assert!(!eclipse_tx.is_solana());
        assert!(eclipse_tx.is_mainnet());
    }
}
