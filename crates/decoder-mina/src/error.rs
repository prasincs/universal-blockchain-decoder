//! Error types for Mina transaction decoding

use thiserror::Error;

/// Errors that can occur during Mina transaction decoding
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MinaDecoderError {
    /// Invalid transaction structure
    #[error("Invalid transaction structure: {0}")]
    InvalidStructure(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid public key
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Invalid field element
    #[error("Invalid field element: {0}")]
    InvalidFieldElement(String),

    /// Invalid proof
    #[error("Invalid proof: {0}")]
    InvalidProof(String),

    /// Unsupported transaction type
    #[error("Unsupported transaction type: {0}")]
    UnsupportedTransactionType(String),

    /// Invalid account update
    #[error("Invalid account update: {0}")]
    InvalidAccountUpdate(String),

    /// Invalid memo
    #[error("Invalid memo: memo must be at most 32 bytes, got {0}")]
    InvalidMemo(usize),

    /// Insufficient data
    #[error("Insufficient data: expected at least {expected} bytes, got {actual}")]
    InsufficientData { expected: usize, actual: usize },

    /// Crypto error
    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    /// Parsing error
    #[error("Parsing error: {0}")]
    ParsingError(String),
}

/// Result type for Mina decoder operations
pub type Result<T> = std::result::Result<T, MinaDecoderError>;

impl From<decoder_crypto_zk::error::CryptoError> for MinaDecoderError {
    fn from(err: decoder_crypto_zk::error::CryptoError) -> Self {
        MinaDecoderError::CryptoError(err.to_string())
    }
}
