//! Integration tests for Bitcoin decoder using real transaction fixtures

use decoder_bitcoin::BitcoinDecoder;
use universal_decoder_core::prelude::*;

/// Test decoding Bitcoin genesis block coinbase transaction
#[test]
fn test_decode_genesis_coinbase() {
    // Load real Bitcoin genesis coinbase transaction
    let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
    let tx_bytes =
        universal_decoder_core::hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    // Decode the transaction
    let decoded = BitcoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Verify basic properties
    assert_eq!(
        decoded.version(),
        1,
        "Genesis coinbase should have version 1"
    );
    assert_eq!(decoded.input_count(), 1, "Coinbase has 1 input");
    assert_eq!(decoded.output_count(), 1, "Genesis coinbase has 1 output");
    assert!(decoded.is_coinbase(), "Should be identified as coinbase");

    // Verify output value (50 BTC = 5,000,000,000 satoshis)
    let outputs = &decoded.outputs;
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0].value, 5_000_000_000,
        "Genesis block reward is 50 BTC"
    );

    // Verify locktime
    assert_eq!(decoded.locktime, 0, "Genesis transaction has locktime 0");
}

/// Test decoding simple P2PKH transaction
#[test]
fn test_decode_simple_p2pkh() {
    // Load first real Bitcoin transaction (Satoshi -> Hal Finney)
    let tx_hex = include_str!("fixtures/btc_simple_p2pkh.hex");
    let tx_bytes =
        universal_decoder_core::hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    // Decode the transaction
    let decoded = BitcoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Verify basic properties
    assert_eq!(decoded.version(), 1, "Should have version 1");
    assert_eq!(decoded.input_count(), 1, "Has 1 input");
    assert_eq!(
        decoded.output_count(),
        2,
        "Has 2 outputs (payment + change)"
    );
    assert!(!decoded.is_coinbase(), "Should not be coinbase");

    // Verify outputs
    let outputs = &decoded.outputs;
    assert_eq!(outputs.len(), 2);

    // First output: 10 BTC
    assert_eq!(outputs[0].value, 1_000_000_000, "First output is 10 BTC");

    // Second output: 40 BTC (change)
    assert_eq!(outputs[1].value, 4_000_000_000, "Second output is 40 BTC");

    // Verify total output value
    let total_output: u64 = outputs.iter().map(|o| o.value).sum();
    assert_eq!(total_output, 5_000_000_000, "Total output is 50 BTC");
}

/// Test that validate_format properly rejects invalid inputs
#[test]
fn test_validate_format_rejects_invalid() {
    // Empty transaction
    assert!(
        BitcoinDecoder::validate_format(&[]).is_err(),
        "Should reject empty transaction"
    );

    // Too small transaction
    assert!(
        BitcoinDecoder::validate_format(&[0x01, 0x00]).is_err(),
        "Should reject transaction that's too small"
    );

    // Valid size should pass basic validation
    let dummy_tx = vec![0u8; 100];
    assert!(
        BitcoinDecoder::validate_format(&dummy_tx).is_ok(),
        "Should pass basic validation for reasonable size"
    );
}

/// Test ChainDecoder trait implementation
#[test]
fn test_chain_decoder_trait() {
    // Verify basic decoder functionality
    let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
    let tx_bytes =
        universal_decoder_core::hex::decode(tx_hex.trim()).expect("Failed to decode hex");

    let result = BitcoinDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Decoder should successfully decode valid transaction"
    );
}

/// Test decoding with malformed input (should error gracefully)
#[test]
fn test_decode_malformed_input() {
    let malformed_inputs = vec![
        vec![],                       // Empty
        vec![0x00],                   // Single byte
        vec![0xFF; 10],               // Invalid data
        vec![0x01, 0x00, 0x00, 0x00], // Incomplete transaction
    ];

    for input in malformed_inputs {
        let result = BitcoinDecoder::decode(&input);
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
    let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
    let tx_bytes =
        universal_decoder_core::hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    let decoded = BitcoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Raw bytes should be preserved
    let raw = &decoded.raw_bytes;
    assert_eq!(raw, &tx_bytes[..], "Raw bytes should match input");
}

/// Test canonicalization of Bitcoin transactions
#[test]
fn test_canonicalization() {
    let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
    let tx_bytes =
        universal_decoder_core::hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    let decoded = BitcoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Canonicalize the transaction
    let tx_ir = decoded.canonicalize().expect("Failed to canonicalize");

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

/// Benchmark-style test: Decode multiple transactions
#[test]
fn test_decode_multiple_transactions() {
    let fixtures = [
        include_str!("fixtures/btc_genesis_coinbase.hex"),
        include_str!("fixtures/btc_simple_p2pkh.hex"),
    ];

    for (i, tx_hex) in fixtures.iter().enumerate() {
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim())
            .unwrap_or_else(|_| panic!("Failed to decode hex fixture {}", i));

        let decoded = BitcoinDecoder::decode(&tx_bytes)
            .unwrap_or_else(|_| panic!("Failed to decode transaction {}", i));

        // Basic sanity checks
        assert!(decoded.version() > 0, "Version should be positive");
        assert!(decoded.input_count() > 0, "Should have at least 1 input");
        assert!(decoded.output_count() > 0, "Should have at least 1 output");
    }
}

/// Test that transaction size is reasonable
#[test]
fn test_transaction_size_bounds() {
    let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
    let tx_bytes =
        universal_decoder_core::hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    let decoded = BitcoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Bitcoin transactions are typically < 100 KB
    let size = decoded.raw_bytes.len();
    assert!(
        size > 0 && size < 100_000,
        "Transaction size should be reasonable: {} bytes",
        size
    );
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
        assert_decode_never_panics::<BitcoinDecoder>(&input);
    }
}

/// Test canonical roundtrip property (using test-utils)
#[test]
fn test_canonical_roundtrip_property() {
    use decoder_test_utils::assertions::assert_canonical_roundtrip;

    let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
    let tx_bytes =
        universal_decoder_core::hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    let decoded = BitcoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");
    let tx_ir = decoded.canonicalize().expect("Failed to canonicalize");

    // Verify canonical serialization is deterministic
    assert_canonical_roundtrip(&tx_ir);
}

/// Test that decoder rejects empty input (using test-utils)
#[test]
fn test_rejects_empty_input() {
    use decoder_test_utils::assertions::assert_rejects_empty_input;
    assert_rejects_empty_input::<BitcoinDecoder>();
}

/// Test that decoder handles oversized input (using test-utils)
#[test]
fn test_handles_oversized_input() {
    use decoder_test_utils::assertions::assert_handles_oversized_input;
    // Bitcoin transactions should be < 1MB
    assert_handles_oversized_input::<BitcoinDecoder>(1_000_000);
}
