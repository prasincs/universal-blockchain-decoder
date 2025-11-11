//! Error types for the universal blockchain decoder.
//!
//! This module provides a unified error handling system that encompasses
//! chain-specific decoding errors, canonicalization failures, and validation errors.

use thiserror::Error;

/// The top-level error type for the universal decoder system.
///
/// This enum encapsulates all possible error conditions across the entire
/// decoding pipeline, from raw byte parsing to canonical IR construction.
#[derive(Error, Debug)]
pub enum DecoderError {
    /// Error during chain-specific decoding (e.g., malformed RLP, invalid SCALE)
    #[error("Chain-specific decoding failed: {0}")]
    ChainDecoding(String),

    /// Error during canonicalization (TxSpecific -> TxIR transformation)
    #[error("Canonicalization failed: {0}")]
    Canonicalization(String),

    /// Invalid transaction structure detected
    #[error("Invalid transaction structure: {0}")]
    InvalidStructure(String),

    /// Signature verification failure
    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),

    /// Version mismatch or unsupported version
    #[error("Unsupported transaction version: expected {expected}, got {actual}")]
    VersionMismatch { expected: u8, actual: u8 },

    /// Length constraint violation
    #[error("Length constraint violation: {0}")]
    LengthConstraint(String),

    /// Overflow detected during computation
    #[error("Arithmetic overflow: {0}")]
    Overflow(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid encoding detected
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    /// Hook execution error
    #[error("Hook execution failed: {0}")]
    HookExecution(String),

    /// Generic error for chain-specific issues
    #[error("Chain-specific error: {0}")]
    ChainSpecific(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/Deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type alias for decoder operations
pub type Result<T> = std::result::Result<T, DecoderError>;

impl DecoderError {
    /// Creates a chain-specific decoding error
    pub fn chain_decoding<S: Into<String>>(msg: S) -> Self {
        DecoderError::ChainDecoding(msg.into())
    }

    /// Creates a canonicalization error
    pub fn canonicalization<S: Into<String>>(msg: S) -> Self {
        DecoderError::Canonicalization(msg.into())
    }

    /// Creates an invalid structure error
    pub fn invalid_structure<S: Into<String>>(msg: S) -> Self {
        DecoderError::InvalidStructure(msg.into())
    }

    /// Creates a signature verification error
    pub fn signature_verification<S: Into<String>>(msg: S) -> Self {
        DecoderError::SignatureVerification(msg.into())
    }

    /// Creates a length constraint violation error
    pub fn length_constraint<S: Into<String>>(msg: S) -> Self {
        DecoderError::LengthConstraint(msg.into())
    }

    /// Creates an overflow error
    pub fn overflow<S: Into<String>>(msg: S) -> Self {
        DecoderError::Overflow(msg.into())
    }

    /// Creates a missing field error
    pub fn missing_field<S: Into<String>>(msg: S) -> Self {
        DecoderError::MissingField(msg.into())
    }

    /// Creates an invalid encoding error
    pub fn invalid_encoding<S: Into<String>>(msg: S) -> Self {
        DecoderError::InvalidEncoding(msg.into())
    }

    /// Creates a hook execution error
    pub fn hook_execution<S: Into<String>>(msg: S) -> Self {
        DecoderError::HookExecution(msg.into())
    }

    /// Creates a chain-specific error
    pub fn chain_specific<S: Into<String>>(msg: S) -> Self {
        DecoderError::ChainSpecific(msg.into())
    }

    /// Creates a serialization error
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        DecoderError::Serialization(msg.into())
    }
}
