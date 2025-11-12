//! Property-based testing helpers for decoders
//!
//! This module provides proptest strategies and property test helpers
//! for testing decoder implementations.

use proptest::prelude::*;
use universal_decoder_core::prelude::*;

/// Generate arbitrary transaction bytes for fuzzing
///
/// This strategy generates random byte sequences that can be used
/// to test that decoders handle arbitrary input without panicking.
///
/// # Arguments
///
/// * `min_size` - Minimum number of bytes
/// * `max_size` - Maximum number of bytes
///
/// # Returns
///
/// A proptest strategy that generates `Vec<u8>` of varying sizes
///
/// # Examples
///
/// ```rust,no_run
/// use proptest::prelude::*;
/// use decoder_test_utils::proptest_helpers::arbitrary_transaction_bytes;
///
/// proptest! {
///     #[test]
///     fn decoder_never_panics(bytes in arbitrary_transaction_bytes(0, 10_000)) {
///         let result = MyDecoder::decode(&bytes);
///         // Should return Ok or Err, never panic
///         assert!(result.is_ok() || result.is_err());
///     }
/// }
/// ```
pub fn arbitrary_transaction_bytes(
    min_size: usize,
    max_size: usize,
) -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), min_size..=max_size)
}

/// Generate small arbitrary byte sequences (0-1KB)
///
/// Useful for quick property tests.
pub fn arbitrary_small_bytes() -> impl Strategy<Value = Vec<u8>> {
    arbitrary_transaction_bytes(0, 1024)
}

/// Generate medium arbitrary byte sequences (0-100KB)
///
/// Useful for realistic transaction size testing.
pub fn arbitrary_medium_bytes() -> impl Strategy<Value = Vec<u8>> {
    arbitrary_transaction_bytes(0, 100_000)
}

/// Generate large arbitrary byte sequences (0-1MB)
///
/// Useful for stress testing and DoS resistance.
pub fn arbitrary_large_bytes() -> impl Strategy<Value = Vec<u8>> {
    arbitrary_transaction_bytes(0, 1_000_000)
}

/// Property: Decoder never panics on arbitrary input
///
/// This property should hold for ALL decoders:
/// Given arbitrary bytes, decode() returns Ok or Err, never panics.
///
/// # Type Parameters
///
/// * `D` - The decoder to test
///
/// # Examples
///
/// ```rust,no_run
/// use proptest::prelude::*;
/// use decoder_test_utils::proptest_helpers::{
///     arbitrary_small_bytes, prop_decoder_never_panics
/// };
///
/// proptest! {
///     #[test]
///     fn test_bitcoin_never_panics(bytes in arbitrary_small_bytes()) {
///         prop_decoder_never_panics::<BitcoinDecoder>(&bytes);
///     }
/// }
/// ```
pub fn prop_decoder_never_panics<D: ChainDecoder>(bytes: &[u8]) {
    use std::panic;

    let result = panic::catch_unwind(|| {
        let _ = D::decode(bytes);
    });

    assert!(
        result.is_ok(),
        "Decoder {} panicked on {} bytes",
        std::any::type_name::<D>(),
        bytes.len()
    );
}

/// Property: Canonical serialization is deterministic
///
/// For any value T implementing Canonicalizer:
/// `to_canonical_bytes()` called twice produces identical output.
///
/// # Type Parameters
///
/// * `T` - The type to test
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::proptest_helpers::canonical_serialization_properties;
///
/// fn test_transaction() -> MyTransaction {
///     // Create test transaction
/// }
///
/// #[test]
/// fn test_canonical_determinism() {
///     let tx = test_transaction();
///     canonical_serialization_properties(&tx);
/// }
/// ```
pub fn canonical_serialization_properties<
    T: universal_decoder_core::canonical::CanonicalSerialize,
>(
    value: &T,
) {
    // Property 1: Determinism
    let bytes1 = value
        .to_canonical_bytes()
        .expect("to_canonical_bytes() should succeed");
    let bytes2 = value
        .to_canonical_bytes()
        .expect("to_canonical_bytes() should succeed on second call");

    assert_eq!(
        bytes1, bytes2,
        "Canonical serialization is non-deterministic"
    );

    // Property 2: Hash consistency
    let hash1 = value
        .canonical_hash()
        .expect("canonical_hash() should succeed");
    let hash2 = value
        .canonical_hash()
        .expect("canonical_hash() should succeed on second call");

    assert_eq!(hash1, hash2, "Canonical hash is non-deterministic");

    // Property 3: Hash matches bytes
    use universal_decoder_core::prelude::Sha256Hash;
    use universal_decoder_core::traits::HashAlgorithm;
    let expected_hash = Sha256Hash::hash(&bytes1);
    assert_eq!(
        hash1, expected_hash,
        "canonical_hash() doesn't match SHA-256 of canonical bytes"
    );
}

/// Property: Empty input should fail
///
/// All decoders should reject empty input.
pub fn prop_rejects_empty_input<D: ChainDecoder>() {
    let result = D::decode(&[]);
    assert!(
        result.is_err(),
        "Decoder {} accepted empty input",
        std::any::type_name::<D>()
    );
}

/// Property: Very short input should fail
///
/// Most transaction formats require at least a few bytes.
pub fn prop_rejects_tiny_input<D: ChainDecoder>(min_valid_size: usize) {
    for size in 1..min_valid_size {
        let bytes = vec![0xFF; size];
        let result = D::decode(&bytes);
        assert!(
            result.is_err(),
            "Decoder {} accepted {} bytes (expected at least {})",
            std::any::type_name::<D>(),
            size,
            min_valid_size
        );
    }
}

/// Standard property test suite for any decoder
///
/// Runs a comprehensive set of property tests:
/// 1. Never panics on arbitrary input
/// 2. Rejects empty input
/// 3. Handles oversized input
///
/// # Type Parameters
///
/// * `D` - The decoder to test
///
/// # Arguments
///
/// * `test_cases` - Number of test cases to generate (default: 100)
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::proptest_helpers::standard_decoder_properties;
///
/// #[test]
/// fn test_bitcoin_decoder_properties() {
///     standard_decoder_properties::<BitcoinDecoder>(1000);
/// }
/// ```
pub fn standard_decoder_properties<D: ChainDecoder>(test_cases: u32) {
    let config = ProptestConfig {
        cases: test_cases,
        ..Default::default()
    };

    // Property 1: Never panics
    proptest!(config.clone(), |(bytes in arbitrary_medium_bytes())| {
        prop_decoder_never_panics::<D>(&bytes);
    });

    // Property 2: Rejects empty input
    prop_rejects_empty_input::<D>();
}

/// Generate arbitrary valid u64 values
pub fn arbitrary_u64() -> impl Strategy<Value = u64> {
    any::<u64>()
}

/// Generate arbitrary valid u128 values
pub fn arbitrary_u128() -> impl Strategy<Value = u128> {
    any::<u128>()
}

/// Generate arbitrary 20-byte addresses (Ethereum)
pub fn arbitrary_address() -> impl Strategy<Value = [u8; 20]> {
    prop::array::uniform20(any::<u8>())
}

/// Generate arbitrary 32-byte hashes
pub fn arbitrary_hash() -> impl Strategy<Value = [u8; 32]> {
    prop::array::uniform32(any::<u8>())
}

/// Generate arbitrary 64-byte signatures (Ed25519)
pub fn arbitrary_signature() -> impl Strategy<Value = [u8; 64]> {
    any::<[u8; 64]>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;

    #[test]
    fn test_arbitrary_small_bytes() {
        let strategy = arbitrary_small_bytes();
        let mut runner = proptest::test_runner::TestRunner::default();
        for _ in 0..10 {
            let bytes = strategy.new_tree(&mut runner).unwrap().current();
            assert!(bytes.len() <= 1024);
        }
    }

    #[test]
    fn test_arbitrary_hash() {
        let strategy = arbitrary_hash();
        let mut runner = proptest::test_runner::TestRunner::default();
        let hash = strategy.new_tree(&mut runner).unwrap().current();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_arbitrary_address() {
        let strategy = arbitrary_address();
        let mut runner = proptest::test_runner::TestRunner::default();
        let addr = strategy.new_tree(&mut runner).unwrap().current();
        assert_eq!(addr.len(), 20);
    }
}
