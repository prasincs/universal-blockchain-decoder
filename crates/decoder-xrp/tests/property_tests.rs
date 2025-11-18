//! Property-based tests for XRP decoder
//!
//! Tests that the decoder never panics and handles all input gracefully

use decoder_primitives::prelude::*;
use decoder_xrp::*;
use proptest::prelude::*;

/// Generate arbitrary byte sequences for fuzzing
fn arb_small_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1024)
}

/// Generate larger byte sequences
fn arb_large_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..10240)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Decoder never panics on any input
    #[test]
    fn prop_decoder_never_panics(bytes in arb_small_bytes()) {
        let result = XrpDecoder::decode(&bytes);
        // Either Ok or Err, never panic
        prop_assert!(result.is_ok() || result.is_err());
    }

    /// Property: Decoder never panics on large inputs
    #[test]
    fn prop_decoder_handles_large_input(bytes in arb_large_bytes()) {
        let result = XrpDecoder::decode(&bytes);
        prop_assert!(result.is_ok() || result.is_err());
    }

    /// Property: Validate format never panics
    #[test]
    fn prop_validate_format_never_panics(bytes in arb_small_bytes()) {
        let result = XrpDecoder::validate_format(&bytes);
        prop_assert!(result.is_ok() || result.is_err());
    }

    /// Property: Empty and very small inputs are rejected
    #[test]
    fn prop_small_inputs_rejected(size in 0usize..4) {
        let bytes = vec![0u8; size];
        let result = XrpDecoder::validate_format(&bytes);
        prop_assert!(result.is_err());
    }

    /// Property: If decode succeeds, canonicalize should not panic
    #[test]
    fn prop_canonicalize_on_valid_decode(bytes in arb_small_bytes()) {
        if let Ok(tx) = XrpDecoder::decode(&bytes) {
            let result = tx.canonicalize();
            // May fail validation, but should not panic
            prop_assert!(result.is_ok() || result.is_err());
        }
    }

    /// Property: Transaction type parsing is deterministic
    #[test]
    fn prop_transaction_type_deterministic(tx_type in 0u16..30) {
        // If we can parse a transaction type, it should always be the same
        let bytes = vec![0x12, (tx_type >> 8) as u8, (tx_type & 0xFF) as u8];
        let result1 = XrpDecoder::validate_format(&bytes);
        let result2 = XrpDecoder::validate_format(&bytes);
        prop_assert_eq!(result1.is_ok(), result2.is_ok());
    }
}

#[test]
fn test_empty_input_rejected() {
    let result = XrpDecoder::decode(&[]);
    assert!(result.is_err());
}

#[test]
fn test_too_small_input_rejected() {
    let result = XrpDecoder::decode(&[0x12]);
    assert!(result.is_err());
}

#[test]
fn test_chain_identity() {
    let chain = XrpDecoder::chain();
    assert_eq!(chain.chain_id(), 144);
    assert_eq!(chain.chain_name(), "XRP Ledger");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
}
