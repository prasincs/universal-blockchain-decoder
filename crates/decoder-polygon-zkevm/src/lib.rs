//! Polygon zkEVM transaction decoder
//!
//! This module provides a decoder for Polygon zkEVM transactions, transforming them
//! from their native RLP format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! Polygon zkEVM is EVM-compatible and uses the **exact same transaction format as Ethereum**.
//! This decoder **reuses the Ethereum decoder** with Polygon zkEVM-specific chain ID validation.
//!
//! ## Transaction Format
//!
//! - RLP-encoded (identical to Ethereum)
//! - EIP-2718 transaction types (legacy, EIP-2930, EIP-1559, EIP-4844)
//! - Chain IDs:
//!   - 1101 = Polygon zkEVM Mainnet
//!   - 1442 = Polygon zkEVM Testnet (Cardona)
//!
//! ## Zero-Knowledge Proof System
//!
//! While transactions use standard EVM format, Polygon zkEVM uses:
//! - **Goldilocks field** (p = 2^64 - 2^32 + 1) for proof generation
//! - **Poseidon hash** (via Rescue Prime) for zkTrie state commitments
//! - **zkTrie** (Poseidon-based Merkle tree) instead of Ethereum's Keccak MPT
//!
//! These cryptographic primitives are available in `decoder-crypto-zk` for advanced
//! analysis and proof verification.
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_polygon_zkevm::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "f86c...";
//! let tx_bytes = hex::decode(tx_hex)?;
//!
//! let decoded = PolygonZkevmDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

use decoder_ethereum::{types::EthereumTransaction, EthereumDecoder};
use decoder_primitives::prelude::*;

pub mod zktrie;

/// Polygon zkEVM chain identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkevmChainId {
    /// Polygon zkEVM Mainnet (Chain ID: 1101)
    Mainnet = 1101,
    /// Polygon zkEVM Testnet Cardona (Chain ID: 1442)
    Testnet = 1442,
}

impl ZkevmChainId {
    /// Check if a given chain ID is a valid Polygon zkEVM chain
    pub fn from_chain_id(id: u64) -> Option<Self> {
        match id {
            1101 => Some(Self::Mainnet),
            1442 => Some(Self::Testnet),
            _ => None,
        }
    }

    /// Get the chain ID as u64
    pub fn as_u64(&self) -> u64 {
        *self as u64
    }

    /// Get the network name
    pub fn network_name(&self) -> &str {
        match self {
            Self::Mainnet => "Polygon zkEVM Mainnet",
            Self::Testnet => "Polygon zkEVM Testnet (Cardona)",
        }
    }
}

/// Polygon zkEVM chain identity
#[derive(Debug, Clone, Copy)]
pub struct PolygonZkevmChain;

impl ChainIdentity for PolygonZkevmChain {
    fn chain_id(&self) -> u64 {
        1101 // Default to mainnet
    }

    fn chain_name(&self) -> &str {
        "Polygon zkEVM"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Polygon zkEVM decoder implementing the ChainDecoder trait
///
/// **Reuses Ethereum decoder** with Polygon zkEVM-specific chain ID validation.
///
/// # Supported Networks
///
/// - Chain ID 1101: Polygon zkEVM Mainnet
/// - Chain ID 1442: Polygon zkEVM Testnet (Cardona)
pub struct PolygonZkevmDecoder;

impl ChainDecoder for PolygonZkevmDecoder {
    type TxSpecific = EthereumTransaction; // Reuse Ethereum transaction type
    type Chain = PolygonZkevmChain;

    fn chain() -> Self::Chain {
        PolygonZkevmChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Decode using Ethereum decoder (same RLP format)
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID is for Polygon zkEVM
        if let Some(chain_id) = tx.chain_id {
            if ZkevmChainId::from_chain_id(chain_id).is_none() {
                return Err(DecoderError::invalid_structure(format!(
                    "Invalid Polygon zkEVM chain ID: {} (expected 1101 or 1442)",
                    chain_id
                )));
            }
        }

        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Use Ethereum's validation (same format)
        EthereumDecoder::validate_format(raw_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = PolygonZkevmDecoder::chain();
        assert_eq!(chain.chain_id(), 1101);
        assert_eq!(chain.chain_name(), "Polygon zkEVM");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_zkevm_chain_id_enum() {
        // Test mainnet
        assert_eq!(ZkevmChainId::Mainnet.as_u64(), 1101);
        assert_eq!(
            ZkevmChainId::Mainnet.network_name(),
            "Polygon zkEVM Mainnet"
        );

        // Test testnet
        assert_eq!(ZkevmChainId::Testnet.as_u64(), 1442);
        assert_eq!(
            ZkevmChainId::Testnet.network_name(),
            "Polygon zkEVM Testnet (Cardona)"
        );

        // Test from_chain_id
        assert_eq!(
            ZkevmChainId::from_chain_id(1101),
            Some(ZkevmChainId::Mainnet)
        );
        assert_eq!(
            ZkevmChainId::from_chain_id(1442),
            Some(ZkevmChainId::Testnet)
        );
        assert_eq!(ZkevmChainId::from_chain_id(137), None); // Regular Polygon
        assert_eq!(ZkevmChainId::from_chain_id(1), None); // Ethereum
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(PolygonZkevmDecoder::validate_format(&[]).is_err());

        // Too small should fail (minimum is 5 bytes for Ethereum)
        assert!(PolygonZkevmDecoder::validate_format(&[0x01]).is_err());
        assert!(PolygonZkevmDecoder::validate_format(&[0x01, 0x02, 0x03, 0x04]).is_err());

        // Valid minimum length should pass basic validation
        let dummy_tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
        assert!(PolygonZkevmDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_decoder_reuses_ethereum() {
        // Verify that PolygonZkevmDecoder uses EthereumTransaction type
        use std::any::TypeId;

        fn assert_same_type<T: 'static, U: 'static>() {
            assert_eq!(TypeId::of::<T>(), TypeId::of::<U>());
        }

        type PolygonZkevmTxType = <PolygonZkevmDecoder as ChainDecoder>::TxSpecific;
        assert_same_type::<PolygonZkevmTxType, EthereumTransaction>();
    }

    #[test]
    fn test_chain_id_validation() {
        // This test would require creating actual RLP-encoded transactions
        // For now, we test the chain ID enum validation logic
        assert!(ZkevmChainId::from_chain_id(1101).is_some());
        assert!(ZkevmChainId::from_chain_id(1442).is_some());
        assert!(ZkevmChainId::from_chain_id(1).is_none());
        assert!(ZkevmChainId::from_chain_id(137).is_none());
    }
}
