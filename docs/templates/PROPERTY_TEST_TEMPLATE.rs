// Property-based tests for {{CHAIN}} decoder
//
// This file provides comprehensive property-based testing using proptest.
// These tests verify fundamental properties that should hold for ALL inputs.

use proptest::prelude::*;
use decoder_{{chain}}::*;
use decoder_test_utils::{arb_small_bytes, arb_large_bytes};

// =============================================================================
// PROPERTY 1: Decoder Never Panics
// =============================================================================
// The decoder should gracefully handle ALL byte sequences, returning Ok or Err
// but NEVER panicking (no index out of bounds, no integer overflow, etc.)

proptest! {
    #[test]
    fn prop_decoder_never_panics(bytes in arb_small_bytes()) {
        let result = {{CHAIN}}Decoder::decode(&bytes);
        // Should return Result::Ok or Result::Err, never panic
        prop_assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn prop_decoder_never_panics_large_input(bytes in arb_large_bytes()) {
        let result = {{CHAIN}}Decoder::decode(&bytes);
        prop_assert!(result.is_ok() || result.is_err());
    }
}

// =============================================================================
// PROPERTY 2: Validation Is Consistent
// =============================================================================
// If decode() succeeds, validate() on the result should also succeed

proptest! {
    #[test]
    fn prop_decode_implies_valid(bytes in arb_small_bytes()) {
        if let Ok(tx) = {{CHAIN}}Decoder::decode(&bytes) {
            // If decoding succeeded, validation should also succeed
            prop_assert!(tx.validate().is_ok());
        }
    }
}

// =============================================================================
// PROPERTY 3: Canonical Bytes Are Deterministic
// =============================================================================
// Calling to_canonical_bytes() multiple times on the same transaction
// should produce identical results

proptest! {
    #[test]
    fn prop_canonical_bytes_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = {{CHAIN}}Decoder::decode(&bytes) {
            if let Ok(canonical1) = tx.to_canonical_bytes() {
                let canonical2 = tx.to_canonical_bytes().unwrap();
                prop_assert_eq!(canonical1, canonical2);
            }
        }
    }
}

// =============================================================================
// PROPERTY 4: Canonical Hash Is Deterministic
// =============================================================================
// Hashing should be deterministic (same input → same hash)

proptest! {
    #[test]
    fn prop_canonical_hash_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = {{CHAIN}}Decoder::decode(&bytes) {
            if let Ok(hash1) = tx.canonical_hash() {
                let hash2 = tx.canonical_hash().unwrap();
                prop_assert_eq!(hash1, hash2);
            }
        }
    }
}

// =============================================================================
// PROPERTY 5: TxIR Canonicalization Roundtrip
// =============================================================================
// Converting to TxIR and back to canonical bytes should be deterministic

proptest! {
    #[test]
    fn prop_txir_canonicalization_roundtrip(bytes in arb_small_bytes()) {
        if let Ok(tx) = {{CHAIN}}Decoder::decode(&bytes) {
            if let Ok(ir1) = tx.canonicalize() {
                let canonical1 = ir1.to_canonical_bytes().unwrap();

                // Second canonicalization should produce identical bytes
                let ir2 = tx.canonicalize().unwrap();
                let canonical2 = ir2.to_canonical_bytes().unwrap();

                prop_assert_eq!(canonical1, canonical2);
            }
        }
    }
}

// =============================================================================
// PROPERTY 6: Empty Input Rejected
// =============================================================================
// Empty byte slices should be rejected (not valid transactions)

proptest! {
    #[test]
    fn prop_empty_input_rejected(_dummy in 0u8..1u8) {
        let result = {{CHAIN}}Decoder::decode(&[]);
        prop_assert!(result.is_err());
    }
}

// =============================================================================
// PROPERTY 7: Format Validation Consistency
// =============================================================================
// If validate_format() succeeds, decode() should also succeed

proptest! {
    #[test]
    fn prop_validate_format_consistency(bytes in arb_small_bytes()) {
        let format_valid = {{CHAIN}}Decoder::validate_format(&bytes).is_ok();
        let decode_valid = {{CHAIN}}Decoder::decode(&bytes).is_ok();

        // If format validation passes, decode should also pass
        if format_valid {
            prop_assert!(decode_valid);
        }
    }
}

// =============================================================================
// PROPERTY 8: Decoded Size Bounded
// =============================================================================
// The decoded transaction should not exceed a reasonable size multiplier
// of the input (prevents memory exhaustion attacks)

proptest! {
    #[test]
    fn prop_decoded_size_bounded(bytes in arb_small_bytes()) {
        if let Ok(tx) = {{CHAIN}}Decoder::decode(&bytes) {
            if let Ok(canonical) = tx.to_canonical_bytes() {
                // Canonical representation should not be more than 10x input size
                // (Adjust multiplier based on chain-specific compression ratios)
                prop_assert!(canonical.len() <= bytes.len() * 10);
            }
        }
    }
}

// =============================================================================
// Custom Arbitrary Generators (Optional - Customize for Your Chain)
// =============================================================================

// Generate valid {{CHAIN}} transactions (if you know the format)
#[cfg(feature = "expensive_tests")]
fn arb_valid_{{chain}}_tx() -> impl Strategy<Value = Vec<u8>> {
    // TODO: Implement chain-specific valid transaction generator
    // Example structure:
    // (version, inputs, outputs, locktime).prop_map(|(v, i, o, l)| {
    //     // Serialize to {{CHAIN}} format
    // })

    // For now, just use small bytes
    arb_small_bytes()
}

// =============================================================================
// CONFIGURATION
// =============================================================================

// Adjust proptest configuration for faster/slower testing
// Default: 256 cases per test
// CI: 1000 cases (set via PROPTEST_CASES env var)
