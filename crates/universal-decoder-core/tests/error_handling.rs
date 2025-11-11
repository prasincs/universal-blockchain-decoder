//! Tests for error handling and edge cases
//!
//! Verifies that the decoder handles errors gracefully and provides
//! useful error messages for debugging.

use universal_decoder_core::error::{DecoderError, Result};

#[test]
fn test_error_creation() {
    let err = DecoderError::invalid_structure("test message");
    assert!(err.to_string().contains("test message"));

    let err = DecoderError::chain_decoding("Failed to decode Bitcoin transaction");
    assert!(err.to_string().contains("Bitcoin"));

    let err = DecoderError::serialization("borsh failed".to_string());
    assert!(err.to_string().contains("borsh failed"));

    let err = DecoderError::signature_verification("ECDSA signature invalid");
    assert!(err.to_string().contains("ECDSA"));

    let err = DecoderError::canonicalization("Cannot convert to TxIR");
    assert!(err.to_string().contains("TxIR"));
}

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DecoderError>();
}

#[test]
fn test_result_type_alias() {
    fn returns_result() -> Result<u32> {
        Ok(42)
    }

    assert_eq!(returns_result().unwrap(), 42);
}

#[test]
fn test_error_propagation() {
    fn inner() -> Result<()> {
        Err(DecoderError::invalid_structure("inner error"))
    }

    fn outer() -> Result<()> {
        inner()?;
        Ok(())
    }

    let result = outer();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("inner error"));
}

#[test]
fn test_all_error_variants() {
    let errors = vec![
        DecoderError::chain_decoding("test"),
        DecoderError::canonicalization("test"),
        DecoderError::invalid_structure("test"),
        DecoderError::signature_verification("test"),
        DecoderError::length_constraint("test"),
        DecoderError::overflow("test"),
        DecoderError::missing_field("test"),
        DecoderError::invalid_encoding("test"),
        DecoderError::hook_execution("test"),
        DecoderError::chain_specific("test"),
        DecoderError::serialization("test".to_string()),
        DecoderError::VersionMismatch {
            expected: 1,
            actual: 2,
        },
    ];

    for err in errors {
        // Ensure all errors can be converted to strings
        let _ = err.to_string();
    }
}
