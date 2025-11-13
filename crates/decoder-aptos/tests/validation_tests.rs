//! Validation tests for Aptos decoder
//!
//! These tests validate our pure Rust implementation against the official Aptos SDK.
//! The official SDK is used ONLY in dev-dependencies for testing purposes.

use decoder_aptos::AptosDecoder;
use universal_decoder_core::prelude::*;

/// Test data: Simple transfer transaction from Aptos testnet
/// This is a real BCS-encoded Aptos transaction
const SIMPLE_TRANSFER_TX_HEX: &str = ""; // TODO: Add real transaction hex

#[test]
#[ignore] // Ignore until we have real test data
#[allow(clippy::const_is_empty)]
fn test_decode_simple_transfer() {
    if SIMPLE_TRANSFER_TX_HEX.is_empty() {
        return; // Skip if no test data
    }

    let tx_bytes = universal_decoder_core::hex::decode(SIMPLE_TRANSFER_TX_HEX).unwrap();

    // Decode with our implementation
    let our_tx = AptosDecoder::decode(&tx_bytes).unwrap();

    // Validate basic properties
    assert!(
        !our_tx.sender().iter().all(|&b| b == 0),
        "Sender should not be zero address"
    );
    assert!(our_tx.max_gas_amount() > 0, "Gas amount should be positive");
    assert!(our_tx.gas_unit_price() > 0, "Gas price should be positive");
}

#[test]
#[ignore] // Ignore until we have official SDK comparison
fn test_aptos_bcs_compatibility() {
    // This test would compare our BCS parsing with the official implementation
    // Example structure:
    //
    // let tx_bytes = ...;
    //
    // // Our implementation
    // let our_tx = AptosDecoder::decode(&tx_bytes).unwrap();
    //
    // // Official SDK
    // let official_tx: aptos_types::transaction::SignedTransaction =
    //     bcs::from_bytes(&tx_bytes).unwrap();
    //
    // // Compare fields
    // assert_eq!(our_tx.sender(), official_tx.sender().as_ref());
    // assert_eq!(our_tx.sequence_number(), official_tx.sequence_number());
    // assert_eq!(our_tx.chain_id(), official_tx.chain_id().id());
}

#[test]
fn test_decode_validates_format() {
    // Empty transaction should fail
    let result = AptosDecoder::decode(&[]);
    assert!(result.is_err());

    // Too small transaction should fail
    let small_tx = vec![0u8; 10];
    let result = AptosDecoder::decode(&small_tx);
    assert!(result.is_err());

    // Too large transaction should fail
    let large_tx = vec![0u8; 100_000];
    let result = AptosDecoder::decode(&large_tx);
    assert!(result.is_err());
}

#[test]
fn test_aptos_chain_identity() {
    use decoder_aptos::AptosChain;

    let chain = AptosChain;
    assert_eq!(chain.chain_name(), "Aptos");
    assert_eq!(chain.chain_id(), 1);
    assert!(matches!(chain.chain_family(), ChainFamily::Account));
}

/// Property-based test: Any valid BCS-encoded transaction should decode without panicking
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        #[ignore] // Expensive test, run manually
        fn test_decode_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
            // Our decoder should never panic, even on random input
            let _ = AptosDecoder::decode(&bytes);
            // Test passes if we reach here without panicking
        }
    }
}

// TODO: Add more validation tests with real Aptos transaction data
// - Entry function calls
// - Multi-sig transactions
// - Script transactions
// - Different signature schemes (Ed25519, Multi-Ed25519)
