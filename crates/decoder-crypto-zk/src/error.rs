//! Error types for cryptographic operations

use thiserror::Error;

/// Errors that can occur during cryptographic operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// Invalid field element (value exceeds field modulus)
    #[error("Invalid field element: {0}")]
    InvalidFieldElement(String),

    /// Invalid curve point (point not on curve)
    #[error("Invalid curve point: {0}")]
    InvalidCurvePoint(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid input length
    #[error("Invalid input length: expected {expected}, got {actual}")]
    InvalidInputLength { expected: usize, actual: usize },

    /// Hex decoding error
    #[error("Hex decoding error: {0}")]
    HexError(String),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Point at infinity
    #[error("Point at infinity")]
    PointAtInfinity,

    /// Generic cryptographic error
    #[error("Cryptographic error: {0}")]
    Generic(String),
}

/// Result type for cryptographic operations
pub type Result<T> = std::result::Result<T, CryptoError>;
