//! # Mina Protocol Transaction Decoder
//!
//! Pure Rust decoder for Mina Protocol transactions, including zkApp transactions
//! with recursive zkSNARK proofs.
//!
//! ## Overview
//!
//! Mina Protocol is the world's lightest blockchain with a constant 22KB size,
//! achieved through recursive zkSNARKs. This decoder supports:
//!
//! - **Payment Transactions**: Simple value transfers
//! - **zkApp Transactions**: Smart contract transactions with proofs
//! - **Delegation Transactions**: Stake delegation
//! - **Account Updates**: State changes in zkApps
//!
//! ## Cryptographic Primitives
//!
//! - **Pallas Curve**: Part of the Pasta curves (Pallas/Vesta cycle)
//! - **Poseidon Hash**: ZK-friendly hash function
//! - **Schnorr-like Signatures**: On the Pallas curve
//!
//! ## Usage
//!
//! ```rust,ignore
//! use decoder_mina::MinaDecoder;
//! use universal_decoder_core::Decoder;
//!
//! let decoder = MinaDecoder::new();
//! let tx_bytes = /* ... */;
//! let tx_ir = decoder.decode(&tx_bytes)?;
//! ```
//!
//! ## Architecture
//!
//! This crate is organized as follows:
//! - `types`: Transaction type definitions
//! - `parsing`: Binary parsing logic
//! - `conversion`: TxIR conversion
//! - `error`: Error types

// Modules
pub mod error;
pub mod types;

// Re-exports
pub use error::{MinaDecoderError, Result};
pub use types::{
    AccountUpdate, Authorization, DelegationTransaction, MinaTransaction, PaymentTransaction,
    Permissions, PublicKey, Signature, TokenId, ZkAppTransaction,
};

/// Mina Protocol transaction decoder
///
/// NOTE: Full ChainDecoder trait implementation will be added in subsequent tasks.
/// This is the foundational structure for Phase 3.9.
///
/// # Examples
///
/// ```rust,ignore
/// use decoder_mina::MinaDecoder;
///
/// let decoder = MinaDecoder::new();
/// let tx_bytes = /* Mina transaction bytes */;
/// let mina_tx = decoder.decode_mina_transaction(&tx_bytes)?;
/// ```
pub struct MinaDecoder;

impl MinaDecoder {
    /// Create a new Mina decoder
    pub fn new() -> Self {
        Self
    }

    /// Decode a Mina transaction from bytes
    ///
    /// NOTE: This is a placeholder implementation.
    /// Full parsing logic will be implemented in subsequent tasks.
    pub fn decode_mina_transaction(&self, _data: &[u8]) -> Result<MinaTransaction> {
        // TODO: Implement actual parsing in Phase 3.9 tasks
        Err(MinaDecoderError::UnsupportedTransactionType(
            "Full parsing not yet implemented - Phase 3.9 in progress".to_string(),
        ))
    }

    /// Get chain name
    pub fn chain_name(&self) -> &str {
        "Mina Protocol"
    }
}

impl Default for MinaDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mina_decoder_creation() {
        let decoder = MinaDecoder::new();
        assert_eq!(decoder.chain_name(), "Mina Protocol");
    }

    #[test]
    fn test_decoder_default() {
        let decoder = MinaDecoder;
        assert_eq!(decoder.chain_name(), "Mina Protocol");
    }

    #[test]
    fn test_decode_returns_error_for_now() {
        let decoder = MinaDecoder::new();
        let result = decoder.decode_mina_transaction(&[0x01, 0x02, 0x03]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MinaDecoderError::UnsupportedTransactionType(_)
        ));
    }
}
