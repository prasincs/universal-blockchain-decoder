//! Error types for Aleo decoder

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AleoDecoderError>;

/// Errors that can occur during Aleo transaction decoding
#[derive(Error, Debug)]
pub enum AleoDecoderError {
    /// Invalid transaction structure
    #[error("Invalid Aleo transaction structure: {0}")]
    InvalidStructure(String),

    /// Unsupported transaction type
    #[error("Unsupported Aleo transaction type: {0}")]
    UnsupportedTransactionType(String),

    /// Parsing error
    #[error("Failed to parse Aleo transaction: {0}")]
    ParsingError(String),

    /// Invalid program
    #[error("Invalid Aleo program: {0}")]
    InvalidProgram(String),

    /// Invalid record
    #[error("Invalid Aleo record: {0}")]
    InvalidRecord(String),

    /// Invalid transition
    #[error("Invalid Aleo transition: {0}")]
    InvalidTransition(String),

    /// Invalid proof
    #[error("Invalid Aleo proof: {0}")]
    InvalidProof(String),

    /// Invalid address
    #[error("Invalid Aleo address: {0}")]
    InvalidAddress(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Decoder error (from universal-decoder-core)
    #[error("Decoder error: {0}")]
    DecoderError(#[from] decoder_primitives::DecoderError),
}

impl From<AleoDecoderError> for decoder_primitives::DecoderError {
    fn from(err: AleoDecoderError) -> Self {
        match err {
            AleoDecoderError::InvalidStructure(msg) => {
                decoder_primitives::DecoderError::invalid_structure(msg)
            }
            AleoDecoderError::UnsupportedTransactionType(msg) => {
                decoder_primitives::DecoderError::chain_specific(msg)
            }
            AleoDecoderError::ParsingError(msg) => {
                decoder_primitives::DecoderError::parsing_failed(msg)
            }
            AleoDecoderError::InvalidProgram(msg) => {
                decoder_primitives::DecoderError::chain_specific(format!(
                    "Invalid program: {}",
                    msg
                ))
            }
            AleoDecoderError::InvalidRecord(msg) => {
                decoder_primitives::DecoderError::chain_specific(format!("Invalid record: {}", msg))
            }
            AleoDecoderError::InvalidTransition(msg) => {
                decoder_primitives::DecoderError::chain_specific(format!(
                    "Invalid transition: {}",
                    msg
                ))
            }
            AleoDecoderError::InvalidProof(msg) => {
                decoder_primitives::DecoderError::chain_specific(format!("Invalid proof: {}", msg))
            }
            AleoDecoderError::InvalidAddress(msg) => {
                decoder_primitives::DecoderError::invalid_signature(msg)
            }
            AleoDecoderError::Io(err) => {
                decoder_primitives::DecoderError::parsing_failed(err.to_string())
            }
            AleoDecoderError::DecoderError(err) => err,
        }
    }
}
