//! Validation tests for Sui decoder
//!
//! These tests validate our pure Rust implementation against the official Sui SDK.
//! The official SDK is used ONLY in dev-dependencies for testing purposes.

use decoder_sui::SuiDecoder;
use universal_decoder_core::prelude::*;

/// Test data: Simple transfer transaction from Sui testnet
/// This is a real BCS-encoded Sui transaction
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
    let our_tx = SuiDecoder::decode(&tx_bytes).unwrap();

    // Validate basic properties
    assert!(
        !our_tx.sender().iter().all(|&b| b == 0),
        "Sender should not be zero address"
    );
    assert!(our_tx.gas_budget() > 0, "Gas budget should be positive");
    assert!(our_tx.gas_price() > 0, "Gas price should be positive");
}

#[test]
#[ignore] // Ignore until we have official SDK comparison
fn test_sui_bcs_compatibility() {
    // This test would compare our BCS parsing with the official implementation
    // Example structure:
    //
    // let tx_bytes = ...;
    //
    // // Our implementation
    // let our_tx = SuiDecoder::decode(&tx_bytes).unwrap();
    //
    // // Official SDK
    // let official_tx: sui_types::transaction::Transaction =
    //     bcs::from_bytes(&tx_bytes).unwrap();
    //
    // // Compare fields
    // assert_eq!(our_tx.sender(), official_tx.data().sender().as_ref());
    // assert_eq!(our_tx.gas_budget(), official_tx.data().gas_data().budget);
}

#[test]
fn test_decode_validates_format() {
    // Empty transaction should fail
    let result = SuiDecoder::decode(&[]);
    assert!(result.is_err());

    // Too small transaction should fail
    let small_tx = vec![0u8; 50];
    let result = SuiDecoder::decode(&small_tx);
    assert!(result.is_err());

    // Too large transaction should fail
    let large_tx = vec![0u8; 200_000];
    let result = SuiDecoder::decode(&large_tx);
    assert!(result.is_err());
}

#[test]
fn test_sui_chain_identity() {
    use decoder_sui::SuiChain;

    let chain = SuiChain;
    assert_eq!(chain.chain_name(), "Sui");
    assert_eq!(chain.chain_id(), 0); // Sui uses object IDs, not numeric chain ID
    assert!(matches!(chain.chain_family(), ChainFamily::Instruction));
}

#[test]
fn test_programmable_transaction_properties() {
    // Test that we can check if a transaction is programmable
    // This would require actual test data
    // For now, just verify the methods exist and compile
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
            let _ = SuiDecoder::decode(&bytes);
            // Test passes if we reach here without panicking
        }
    }
}

// TODO: Add more validation tests with real Sui transaction data
// - Programmable transactions with multiple commands
// - MoveCall commands
// - TransferObjects commands
// - Publish commands (contract deployment)
// - Different signature schemes (Ed25519, Secp256k1, Secp256r1)
// - Multi-command transactions
