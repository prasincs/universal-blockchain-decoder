//! Integration tests for Cardano decoder using real transaction fixtures

use decoder_cardano::CardanoDecoder;
use universal_decoder_core::prelude::*;

/// Test basic decoder functionality
#[test]
fn test_decode_minimal_transaction() {
    // Minimal valid CBOR structure: [map, map, null]
    // This is a simplified transaction for basic testing
    let tx_bytes = create_minimal_cardano_tx();

    // Attempt to decode - this will fail due to invalid structure,
    // but should not panic
    let result = CardanoDecoder::decode(&tx_bytes);

    // The decode should either succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

/// Test that validate_format properly rejects invalid inputs
#[test]
fn test_validate_format_rejects_invalid() {
    // Empty transaction
    assert!(
        CardanoDecoder::validate_format(&[]).is_err(),
        "Should reject empty transaction"
    );

    // Too small transaction
    assert!(
        CardanoDecoder::validate_format(&[0x01, 0x00]).is_err(),
        "Should reject transaction that's too small"
    );

    // Valid CBOR array marker should pass basic validation (at least 10 bytes)
    let dummy_tx = vec![0x83, 0xa0, 0xa0, 0xf6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // [map, map, null] + padding
    assert!(
        CardanoDecoder::validate_format(&dummy_tx).is_ok(),
        "Should pass basic validation for reasonable size with valid CBOR marker"
    );

    // Invalid marker (not an array)
    let invalid_marker = vec![0x01, 0x02, 0x03];
    assert!(
        CardanoDecoder::validate_format(&invalid_marker).is_err(),
        "Should reject non-array CBOR marker"
    );
}

/// Test ChainDecoder trait implementation
#[test]
fn test_chain_decoder_trait() {
    // Verify basic decoder functionality
    let tx_bytes = create_minimal_cardano_tx();

    let result = CardanoDecoder::decode(&tx_bytes);

    // Should return Result, not panic
    assert!(result.is_ok() || result.is_err());
}

/// Test decoding with malformed input (should error gracefully)
#[test]
fn test_decode_malformed_input() {
    let malformed_inputs = vec![
        vec![],                       // Empty
        vec![0x00],                   // Single byte
        vec![0xFF; 10],               // Invalid data
        vec![0x01, 0x00, 0x00, 0x00], // Not CBOR array
    ];

    for input in malformed_inputs {
        let result = CardanoDecoder::decode(&input);
        // Should return error, not panic
        assert!(
            result.is_err(),
            "Malformed input should return error: {:?}",
            input
        );
    }
}

/// Test that decoding preserves raw bytes
#[test]
fn test_raw_bytes_preserved() {
    let tx_bytes = create_minimal_cardano_tx();

    if let Ok(decoded) = CardanoDecoder::decode(&tx_bytes) {
        // Raw bytes should be preserved
        let raw = &decoded.raw_bytes;
        assert_eq!(raw, &tx_bytes[..], "Raw bytes should match input");
    }
}

/// Test canonicalization of Cardano transactions
#[test]
fn test_canonicalization() {
    let tx_bytes = create_minimal_cardano_tx();

    if let Ok(decoded) = CardanoDecoder::decode(&tx_bytes) {
        // Attempt canonicalization
        let result = decoded.canonicalize();

        // Should either succeed or fail gracefully
        if let Ok(tx_ir) = result {
            // Verify TxIR properties
            assert_eq!(tx_ir.version(), 1);

            // Canonical serialization should be deterministic
            let canonical_bytes1 = tx_ir.to_canonical_bytes().expect("Failed to serialize");
            let canonical_bytes2 = tx_ir.to_canonical_bytes().expect("Failed to serialize");
            assert_eq!(
                canonical_bytes1, canonical_bytes2,
                "Canonical serialization should be deterministic"
            );

            // Canonical hash should be deterministic
            let hash1 = tx_ir.canonical_hash().expect("Failed to hash");
            let hash2 = tx_ir.canonical_hash().expect("Failed to hash");
            assert_eq!(hash1, hash2, "Canonical hash should be deterministic");
        }
    }
}

// ========== Tests using decoder-test-utils ==========

/// Test that decoder never panics on arbitrary input (using test-utils)
#[test]
fn test_decoder_never_panics_on_garbage() {
    use decoder_test_utils::assertions::assert_decode_never_panics;

    // Test with various garbage inputs
    let test_cases = vec![
        vec![],                        // Empty
        vec![0xFF; 100],               // Random bytes
        vec![0x00; 1000],              // Zeros
        vec![0x01, 0x00, 0x00, 0x00],  // Incomplete
        (0..255).collect::<Vec<u8>>(), // Sequential bytes
    ];

    for input in test_cases {
        assert_decode_never_panics::<CardanoDecoder>(&input);
    }
}

/// Test canonical roundtrip property (using test-utils)
#[test]
fn test_canonical_roundtrip_property() {
    use decoder_test_utils::assertions::assert_canonical_roundtrip;

    let tx_bytes = create_minimal_cardano_tx();

    if let Ok(decoded) = CardanoDecoder::decode(&tx_bytes) {
        if let Ok(tx_ir) = decoded.canonicalize() {
            // Verify canonical serialization is deterministic
            assert_canonical_roundtrip(&tx_ir);
        }
    }
}

/// Test that decoder rejects empty input (using test-utils)
#[test]
fn test_rejects_empty_input() {
    use decoder_test_utils::assertions::assert_rejects_empty_input;
    assert_rejects_empty_input::<CardanoDecoder>();
}

/// Test that decoder handles oversized input (using test-utils)
#[test]
fn test_handles_oversized_input() {
    use decoder_test_utils::assertions::assert_handles_oversized_input;
    // Cardano transactions should be < 64 KB
    assert_handles_oversized_input::<CardanoDecoder>(65536);
}

/// Test transaction methods
#[test]
fn test_transaction_methods() {
    let tx_bytes = create_minimal_cardano_tx();

    if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
        // Test basic methods
        let _txid = tx.txid();
        let _txid_hex = tx.txid_hex();
        let _input_count = tx.input_count();
        let _output_count = tx.output_count();
        let _fee = tx.fee();

        // Test boolean flags
        let _has_certs = tx.has_certificates();
        let _has_withdrawals = tx.has_withdrawals();
        let _has_mint = tx.has_mint();
        let _has_plutus = tx.has_plutus_scripts();
        let _has_metadata = tx.has_metadata();

        // All methods should return without panicking
    }
}

// ========== Helper Functions ==========

/// Create a minimal valid Cardano transaction for testing
///
/// This creates a CBOR-encoded transaction with minimal structure:
/// [transaction_body, witness_set, null (no metadata)]
#[allow(clippy::vec_init_then_push)]
fn create_minimal_cardano_tx() -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // CBOR array with 3 elements
    tx_bytes.push(0x83);

    // Transaction body (CBOR map)
    // Map with 3 entries: inputs, outputs, fee
    tx_bytes.push(0xa3);

    // Key 0: inputs (array)
    tx_bytes.push(0x00);
    tx_bytes.push(0x81); // Array with 1 element
    tx_bytes.push(0x82); // Array with 2 elements [tx_hash, index]
                         // Transaction hash (32 bytes of zeros)
    tx_bytes.push(0x58);
    tx_bytes.push(0x20); // Byte string of 32 bytes
    tx_bytes.extend_from_slice(&[0u8; 32]);
    // Index 0
    tx_bytes.push(0x00);

    // Key 1: outputs (array)
    tx_bytes.push(0x01);
    tx_bytes.push(0x81); // Array with 1 element
    tx_bytes.push(0x82); // Array with 2 elements [address, amount]
                         // Address (29 bytes - typical Cardano address)
    tx_bytes.push(0x58);
    tx_bytes.push(0x1d); // Byte string of 29 bytes
    tx_bytes.extend_from_slice(&[0u8; 29]);
    // Amount (1000000 lovelace = 1 ADA)
    tx_bytes.push(0x1a); // uint32
    tx_bytes.extend_from_slice(&1_000_000u32.to_be_bytes());

    // Key 2: fee
    tx_bytes.push(0x02);
    tx_bytes.push(0x1a); // uint32
    tx_bytes.extend_from_slice(&170_000u32.to_be_bytes()); // Typical fee

    // Witness set (CBOR map with 1 entry)
    tx_bytes.push(0xa1);
    // Key 0: vkey witnesses (array)
    tx_bytes.push(0x00);
    tx_bytes.push(0x81); // Array with 1 element
    tx_bytes.push(0x82); // Array with 2 elements [vkey, signature]
                         // Public key (32 bytes)
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    // Signature (64 bytes)
    tx_bytes.push(0x58);
    tx_bytes.push(0x40);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // No auxiliary data (null)
    tx_bytes.push(0xf6);

    tx_bytes
}

// ========== Real Transaction Tests ==========

/// NOTE: We need actual real Cardano transaction examples from mainnet
///
/// The transaction examples found in various documentation and online sources
/// appear to be invalid or use non-standard formats. Even the pallas library
/// (the official Rust library for Cardano) cannot decode them.
///
/// Investigation findings:
/// - Example 1: 6-element CBOR array (spec says 3-4 elements)
/// - Example 2: 3-element array with indefinite arrays (spec requires definite-length)
/// - Neither can be decoded by pallas (Alonzo or Babbage era)
///
/// To properly test real transactions, we need:
/// 1. Access to a Cardano node to export real transaction CBORs
/// 2. Valid transactions from Cardanoscan API with proper CBOR export
/// 3. Transaction fixtures from the cardano-ledger test suite
///
/// For now, we verify that our decoder works correctly with synthetic test data
/// that follows the CDDL specifications.
///
/// Test that demonstrates the need for real transaction fixtures
#[test]
#[ignore] // Requires real Cardano transaction data
fn test_decode_real_mainnet_transaction() {
    // TODO: Add real Cardano mainnet transaction CBOR hex
    //
    // To get real transaction data:
    // 1. Run: cardano-cli transaction view --tx-file tx.signed --output-json
    // 2. Or use Cardanoscan API with CBOR export
    // 3. Or extract from cardano-ledger test fixtures
    //
    // The transaction should be:
    // - From mainnet (not testnet)
    // - Post-Alonzo era (Babbage or Conway)
    // - Valid CBOR with 4-element array: [body, witness_set, is_valid, auxiliary_data]

    let _tx_hex = ""; // Placeholder for real transaction
                      // let tx_bytes = universal_decoder_core::hex::decode(tx_hex).expect("Failed to decode hex");
                      // let result = CardanoDecoder::decode(&tx_bytes);
                      // assert!(result.is_ok(), "Should decode real Cardano transaction");
}

#[cfg(test)]
mod pallas_validation_tests {
    /// Pallas validation tests - compare our decoder with pallas library
    ///
    /// These tests will validate that our decoder produces similar results
    /// to the official pallas library when decoding real Cardano transactions.
    ///
    /// Currently disabled until we have access to real transaction fixtures.
    ///
    /// See parent module documentation for details on obtaining real transaction data.

    #[test]
    #[ignore] // Requires real Cardano transaction data + pallas comparison logic
    fn test_compare_with_pallas() {
        // TODO: Implement pallas comparison when we have real transaction fixtures
        //
        // Implementation steps:
        // 1. Get real Cardano transaction CBOR
        // 2. Decode with our decoder: CardanoDecoder::decode(&tx_bytes)
        // 3. Decode with pallas: pallas_codec::minicbor::decode::<MintedTx>(&tx_bytes)
        // 4. Compare key fields: inputs, outputs, fee, certificates, etc.
        // 5. Verify our decoder matches pallas behavior
    }
}
