//! Move VM family decoder
//!
//! This module provides a unified decoder for all Move-based blockchains, including:
//! - Aptos (account-based Move blockchain)
//! - Sui (object-centric Move blockchain)
//! - Movement (Move on EVM) - planned
//!
//! ## Architecture
//!
//! The Move decoder wraps the `decoder-aptos` and `decoder-sui` implementations and adds:
//! - Chain-specific identification via `MoveChainId`
//! - Chain registry with metadata (RPCs, explorers, etc.)
//! - Support for Move variants with different transaction models
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_move::{MoveDecoder, MoveChainId};
//! use universal_decoder_core::prelude::*;
//!
//! // Decode an Aptos mainnet transaction
//! let tx_bytes = &[...];
//! let tx = MoveDecoder::decode_with_chain(tx_bytes, MoveChainId::AptosMainnet)?;
//!
//! // Decode a Sui mainnet transaction
//! let tx_bytes = &[...];
//! let tx = MoveDecoder::decode_with_chain(tx_bytes, MoveChainId::SuiMainnet)?;
//! ```
//!
//! ## Supported Chains
//!
//! - ✅ Aptos Mainnet (chain ID 1)
//! - ✅ Aptos Testnet (chain ID 2)
//! - ✅ Aptos Devnet (chain ID 3)
//! - ✅ Sui Mainnet (chain ID 100)
//! - ✅ Sui Testnet (chain ID 101)
//! - ✅ Sui Devnet (chain ID 102)
//! - 🚧 Movement (chain ID 1000) - planned

use decoder_aptos::{AptosDecoder, AptosTransaction};
use decoder_primitives::prelude::*;
use decoder_sui::{SuiDecoder, SuiTransaction};

pub mod registry;

pub use registry::{MoveChainId, MoveChainInfo, MoveChainRegistry, MoveVariant};

/// Move chain identity
///
/// This wraps a specific Move chain (e.g., Aptos Mainnet, Sui Mainnet, etc.)
/// and provides chain-specific identification.
#[derive(Debug, Clone, Copy)]
pub struct MoveChain {
    chain_id: MoveChainId,
}

impl MoveChain {
    /// Create a new Move chain identity
    pub fn new(chain_id: MoveChainId) -> Self {
        Self { chain_id }
    }

    /// Get the chain ID enum
    pub fn chain_id_enum(&self) -> MoveChainId {
        self.chain_id
    }

    /// Get the Move variant (Aptos, Sui, Movement)
    pub fn variant(&self) -> MoveVariant {
        self.chain_id.variant()
    }
}

impl ChainIdentity for MoveChain {
    fn chain_id(&self) -> u64 {
        self.chain_id.to_u64()
    }

    fn chain_name(&self) -> &str {
        self.chain_id.name()
    }

    fn chain_family(&self) -> ChainFamily {
        match self.chain_id.variant() {
            MoveVariant::Aptos => ChainFamily::Account,
            MoveVariant::Sui => ChainFamily::Instruction,
            MoveVariant::Movement => ChainFamily::Account,
        }
    }
}

/// Move transaction wrapper
///
/// Wraps either an Aptos or Sui transaction with chain-specific context.
#[derive(Debug, Clone)]
pub enum MoveTransaction {
    /// Aptos transaction
    Aptos {
        chain_id: MoveChainId,
        inner: AptosTransaction,
    },
    /// Sui transaction
    Sui {
        chain_id: MoveChainId,
        inner: SuiTransaction,
    },
    /// Movement transaction (planned)
    Movement {
        chain_id: MoveChainId,
        // TODO: Add Movement transaction type when implemented
        raw_bytes: Vec<u8>,
    },
}

impl MoveTransaction {
    /// Create a new Aptos Move transaction
    pub fn new_aptos(chain_id: MoveChainId, inner: AptosTransaction) -> Self {
        assert!(chain_id.is_aptos(), "Chain ID must be Aptos");
        Self::Aptos { chain_id, inner }
    }

    /// Create a new Sui Move transaction
    pub fn new_sui(chain_id: MoveChainId, inner: SuiTransaction) -> Self {
        assert!(chain_id.is_sui(), "Chain ID must be Sui");
        Self::Sui { chain_id, inner }
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> MoveChainId {
        match self {
            MoveTransaction::Aptos { chain_id, .. } => *chain_id,
            MoveTransaction::Sui { chain_id, .. } => *chain_id,
            MoveTransaction::Movement { chain_id, .. } => *chain_id,
        }
    }

    /// Get the Move variant
    pub fn variant(&self) -> MoveVariant {
        self.chain_id().variant()
    }

    /// Check if this is an Aptos transaction
    pub fn is_aptos(&self) -> bool {
        matches!(self, MoveTransaction::Aptos { .. })
    }

    /// Check if this is a Sui transaction
    pub fn is_sui(&self) -> bool {
        matches!(self, MoveTransaction::Sui { .. })
    }

    /// Get the underlying Aptos transaction if this is Aptos
    pub fn as_aptos(&self) -> Option<&AptosTransaction> {
        match self {
            MoveTransaction::Aptos { inner, .. } => Some(inner),
            _ => None,
        }
    }

    /// Get the underlying Sui transaction if this is Sui
    pub fn as_sui(&self) -> Option<&SuiTransaction> {
        match self {
            MoveTransaction::Sui { inner, .. } => Some(inner),
            _ => None,
        }
    }
}

impl ChainEncoder for MoveTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            MoveTransaction::Aptos { inner, .. } => inner.to_bytes(),
            MoveTransaction::Sui { inner, .. } => inner.to_bytes(),
            MoveTransaction::Movement { raw_bytes, .. } => Ok(raw_bytes.clone()),
        }
    }
}

impl<'a> Canonicalizer<'a> for MoveTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        match self {
            MoveTransaction::Aptos { chain_id, inner } => {
                // Delegate to the underlying Aptos transaction's canonicalization
                let mut tx_ir = inner.canonicalize()?;

                // Update metadata to include Move-specific chain information
                tx_ir.metadata.extra = format!(
                    r#"{{"move_chain":"{}","move_chain_id":{},"move_variant":"Aptos","is_mainnet":{}}}"#,
                    chain_id.name(),
                    chain_id.to_u64(),
                    chain_id.is_mainnet()
                );

                Ok(tx_ir)
            }
            MoveTransaction::Sui { chain_id, inner } => {
                // Delegate to the underlying Sui transaction's canonicalization
                let mut tx_ir = inner.canonicalize()?;

                // Update metadata to include Move-specific chain information
                tx_ir.metadata.extra = format!(
                    r#"{{"move_chain":"{}","move_chain_id":{},"move_variant":"Sui","is_mainnet":{}}}"#,
                    chain_id.name(),
                    chain_id.to_u64(),
                    chain_id.is_mainnet()
                );

                Ok(tx_ir)
            }
            MoveTransaction::Movement { chain_id, .. } => {
                // TODO: Implement Movement transaction canonicalization
                Err(DecoderError::chain_decoding(format!(
                    "Movement transactions not yet supported for chain {}",
                    chain_id.name()
                )))
            }
        }
    }

    fn validate(&self) -> Result<()> {
        match self {
            MoveTransaction::Aptos { inner, .. } => inner.validate(),
            MoveTransaction::Sui { inner, .. } => inner.validate(),
            MoveTransaction::Movement { .. } => Err(DecoderError::chain_decoding(
                "Movement transactions not yet supported",
            )),
        }
    }
}

/// Move decoder implementing the ChainDecoder trait
///
/// This decoder wraps the Aptos and Sui decoders and adds chain-specific context.
pub struct MoveDecoder {
    chain_id: MoveChainId,
}

impl MoveDecoder {
    /// Create a new Move decoder for a specific chain
    pub fn new(chain_id: MoveChainId) -> Self {
        Self { chain_id }
    }

    /// Decode a transaction with explicit chain specification
    pub fn decode_with_chain(raw_bytes: &[u8], chain_id: MoveChainId) -> Result<MoveTransaction> {
        let decoder = Self::new(chain_id);
        decoder.decode_transaction(raw_bytes)
    }

    /// Decode a transaction (uses the decoder's configured chain)
    fn decode_transaction(&self, raw_bytes: &[u8]) -> Result<MoveTransaction> {
        match self.chain_id.variant() {
            MoveVariant::Aptos => {
                let tx = AptosDecoder::decode(raw_bytes)?;
                Ok(MoveTransaction::new_aptos(self.chain_id, tx))
            }
            MoveVariant::Sui => {
                let tx = SuiDecoder::decode(raw_bytes)?;
                Ok(MoveTransaction::new_sui(self.chain_id, tx))
            }
            MoveVariant::Movement => {
                // TODO: Implement Movement decoder
                Err(DecoderError::chain_decoding(
                    "Movement transactions not yet supported",
                ))
            }
        }
    }
}

impl ChainDecoder for MoveDecoder {
    type TxSpecific = MoveTransaction;
    type Chain = MoveChain;

    fn chain() -> Self::Chain {
        // Default to Aptos Mainnet
        MoveChain::new(MoveChainId::AptosMainnet)
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Default to Aptos Mainnet for backwards compatibility
        Self::decode_with_chain(raw_bytes, MoveChainId::AptosMainnet)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Basic validation - check if it's not empty
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Move transaction cannot be empty",
            ));
        }

        // Minimum size check (transactions are at least 50 bytes)
        if raw_bytes.len() < 50 {
            return Err(DecoderError::invalid_structure(format!(
                "Move transaction too small: {} bytes (minimum ~50 bytes)",
                raw_bytes.len()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let aptos_chain = MoveChain::new(MoveChainId::AptosMainnet);
        assert_eq!(aptos_chain.chain_id(), 1);
        assert_eq!(aptos_chain.chain_name(), "Aptos Mainnet");
        assert!(matches!(aptos_chain.chain_family(), ChainFamily::Account));

        let sui_chain = MoveChain::new(MoveChainId::SuiMainnet);
        assert_eq!(sui_chain.chain_id(), 100);
        assert_eq!(sui_chain.chain_name(), "Sui Mainnet");
        assert!(matches!(sui_chain.chain_family(), ChainFamily::Instruction));
    }

    #[test]
    fn test_validate_format_empty() {
        let result = MoveDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_format_too_small() {
        let data = vec![0u8; 40];
        let result = MoveDecoder::validate_format(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_move_chain_variant() {
        let aptos_chain = MoveChain::new(MoveChainId::AptosMainnet);
        assert_eq!(aptos_chain.variant(), MoveVariant::Aptos);

        let sui_chain = MoveChain::new(MoveChainId::SuiMainnet);
        assert_eq!(sui_chain.variant(), MoveVariant::Sui);
    }

    #[test]
    fn test_decoder_creation() {
        let decoder = MoveDecoder::new(MoveChainId::AptosMainnet);
        assert_eq!(decoder.chain_id, MoveChainId::AptosMainnet);

        let decoder = MoveDecoder::new(MoveChainId::SuiMainnet);
        assert_eq!(decoder.chain_id, MoveChainId::SuiMainnet);
    }
}
