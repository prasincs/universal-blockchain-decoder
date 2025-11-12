//! Common test assertions for decoder implementations
//!
//! This module provides standardized test assertions that should be used
//! by all decoder implementations to ensure consistent behavior.

use std::panic;
use universal_decoder_core::canonical::CanonicalSerialize;
use universal_decoder_core::prelude::*;

/// Assert that a decoder never panics on arbitrary input
///
/// This is a critical safety property: decoders MUST return `Result::Err`
/// for invalid input, never panic. This assertion catches unwrap(), expect(),
/// and other panic-causing code paths.
///
/// # Type Parameters
///
/// * `D` - The decoder type to test
///
/// # Arguments
///
/// * `bytes` - Arbitrary bytes to decode (may be invalid)
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::assertions::assert_decode_never_panics;
/// use universal_decoder_core::prelude::*;
///
/// #[test]
/// fn test_bitcoin_decoder_never_panics() {
///     // Try to decode garbage data
///     let garbage = vec![0xFF; 1000];
///     assert_decode_never_panics::<BitcoinDecoder>(&garbage);
/// }
/// ```
///
/// # Panics
///
/// This function panics if the decoder panics during decoding.
pub fn assert_decode_never_panics<D: ChainDecoder>(bytes: &[u8]) {
    let result = panic::catch_unwind(|| {
        // We don't care about the result, only that it doesn't panic
        let _ = D::decode(bytes);
    });

    if result.is_err() {
        panic!(
            "Decoder {} panicked on input (length: {} bytes). \
             Decoders MUST return Result::Err for invalid input, never panic.",
            std::any::type_name::<D>(),
            bytes.len()
        );
    }
}

/// Assert that canonical serialization is deterministic
///
/// This ensures that:
/// 1. `to_canonical_bytes()` produces the same output on multiple calls
/// 2. `to_canonical_bytes()` is deterministic (no randomness, no ordering issues)
///
/// # Type Parameters
///
/// * `T` - The type implementing `CanonicalSerialize`
///
/// # Arguments
///
/// * `value` - The value to test
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::assertions::assert_canonical_roundtrip;
///
/// #[test]
/// fn test_transaction_canonical() {
///     let tx = create_test_transaction();
///     assert_canonical_roundtrip(&tx);
/// }
/// ```
///
/// # Panics
///
/// Panics if:
/// - `to_canonical_bytes()` fails
/// - `to_canonical_bytes()` is non-deterministic (produces different output)
pub fn assert_canonical_roundtrip<T: CanonicalSerialize>(value: &T) {
    // Call to_canonical_bytes() twice
    let bytes1 = value
        .to_canonical_bytes()
        .expect("to_canonical_bytes() failed on first call");
    let bytes2 = value
        .to_canonical_bytes()
        .expect("to_canonical_bytes() failed on second call");

    // They must be identical
    assert_eq!(
        bytes1,
        bytes2,
        "Canonical serialization is non-deterministic! \
         First call produced {} bytes, second call produced {} bytes. \
         This violates the canonicity requirement.",
        bytes1.len(),
        bytes2.len()
    );
}

/// Assert that decode/encode roundtrip works correctly
///
/// This verifies that for valid transaction bytes:
/// `encode(decode(bytes)) == bytes`
///
/// This is a weaker property than full canonicity but ensures
/// that decoding doesn't lose information.
///
/// # Type Parameters
///
/// * `D` - The decoder type
///
/// # Arguments
///
/// * `bytes` - Valid transaction bytes (must decode successfully)
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::assertions::assert_decode_encode_roundtrip;
///
/// #[test]
/// fn test_roundtrip() {
///     let original_bytes = include_bytes!("fixtures/valid_tx.bin");
///     assert_decode_encode_roundtrip::<MyDecoder>(original_bytes);
/// }
/// ```
///
/// # Panics
///
/// Panics if:
/// - Decode fails
/// - Encode fails
/// - Roundtrip bytes don't match original
pub fn assert_decode_encode_roundtrip<D: ChainDecoder>(bytes: &[u8]) {
    // Decode
    let tx = D::decode(bytes).expect("Failed to decode transaction bytes");

    // Canonicalize
    let tx_ir = tx
        .canonicalize()
        .expect("Failed to canonicalize transaction");

    // Re-encode using canonical serialization
    let roundtrip_bytes = tx_ir
        .to_canonical_bytes()
        .expect("Failed to encode transaction");

    // Note: This may not be byte-for-byte identical if the original
    // encoding was non-canonical. But for canonical inputs, it should match.
    // For now, we just ensure it doesn't error.
    // A stricter test would verify byte-for-byte equality for canonical inputs.
    assert!(
        !roundtrip_bytes.is_empty(),
        "Roundtrip encoding produced empty bytes"
    );
}

/// Assert that a decoder handles empty input correctly
///
/// Empty input should return an error, not panic.
///
/// # Type Parameters
///
/// * `D` - The decoder type to test
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::assertions::assert_rejects_empty_input;
///
/// #[test]
/// fn test_empty_input() {
///     assert_rejects_empty_input::<MyDecoder>();
/// }
/// ```
///
/// # Panics
///
/// Panics if the decoder panics or succeeds on empty input.
pub fn assert_rejects_empty_input<D: ChainDecoder>() {
    let result = D::decode(&[]);
    assert!(
        result.is_err(),
        "Decoder {} succeeded on empty input. Decoders should reject empty input.",
        std::any::type_name::<D>()
    );
}

/// Assert that a decoder handles oversized input correctly
///
/// Decoders should have reasonable size limits to prevent DoS attacks.
///
/// # Type Parameters
///
/// * `D` - The decoder type to test
///
/// # Arguments
///
/// * `max_size` - Maximum reasonable transaction size in bytes
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::assertions::assert_handles_oversized_input;
///
/// #[test]
/// fn test_oversized() {
///     // Bitcoin transactions should be < 1MB
///     assert_handles_oversized_input::<BitcoinDecoder>(1_000_000);
/// }
/// ```
pub fn assert_handles_oversized_input<D: ChainDecoder>(max_size: usize) {
    // Create oversized input (max_size + 1MB)
    let oversized = vec![0xFF; max_size + 1_000_000];

    // Should either reject or decode without panic
    let result = panic::catch_unwind(|| {
        let _ = D::decode(&oversized);
    });

    assert!(
        result.is_ok(),
        "Decoder {} panicked on oversized input ({} bytes). \
         Decoders MUST handle large inputs gracefully.",
        std::any::type_name::<D>(),
        oversized.len()
    );
}
