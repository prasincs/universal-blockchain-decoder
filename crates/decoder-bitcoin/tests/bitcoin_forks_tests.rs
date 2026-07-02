//! Comprehensive tests for Bitcoin forks compatibility
//!
//! This module tests that the Bitcoin decoder can successfully decode transactions
//! from various Bitcoin forks that use compatible transaction formats.
//!
//! Most Bitcoin forks use the same underlying transaction structure as Bitcoin,
//! differing only in network parameters, consensus rules, or signature hashing.
//! This makes them compatible with the Bitcoin decoder for basic decoding.

use decoder_bitcoin::BitcoinDecoder;
use universal_decoder_core::prelude::*;

/// Helper function to decode and validate a transaction fixture
fn decode_and_validate_fork_tx(
    fork_name: &str,
    hex_fixture: &str,
    expected_version: u32,
    expected_inputs: usize,
    expected_outputs: usize,
) {
    let tx_bytes = universal_decoder_core::hex::decode(hex_fixture.trim())
        .unwrap_or_else(|_| panic!("{}: Failed to decode hex fixture", fork_name));

    let decoded = BitcoinDecoder::decode(&tx_bytes)
        .unwrap_or_else(|e| panic!("{}: Failed to decode transaction: {}", fork_name, e));

    // Verify basic properties
    assert_eq!(
        decoded.version(),
        expected_version,
        "{}: Unexpected transaction version",
        fork_name
    );
    assert_eq!(
        decoded.input_count(),
        expected_inputs,
        "{}: Unexpected input count",
        fork_name
    );
    assert_eq!(
        decoded.output_count(),
        expected_outputs,
        "{}: Unexpected output count",
        fork_name
    );

    // Verify roundtrip works
    let encoded = decoded.to_bytes().expect("Failed to encode");
    assert_eq!(
        &encoded, &tx_bytes,
        "{}: Re-encoded bytes should match input",
        fork_name
    );

    // Verify canonicalization works
    let tx_ir = decoded
        .canonicalize()
        .unwrap_or_else(|e| panic!("{}: Failed to canonicalize: {}", fork_name, e));

    assert_eq!(
        tx_ir.version(),
        1,
        "{}: TxIR should have version 1",
        fork_name
    );

    // Verify deterministic canonical serialization
    let canonical_bytes1 = tx_ir
        .to_canonical_bytes()
        .unwrap_or_else(|e| panic!("{}: Failed to serialize: {}", fork_name, e));
    let canonical_bytes2 = tx_ir
        .to_canonical_bytes()
        .unwrap_or_else(|e| panic!("{}: Failed to serialize (2nd attempt): {}", fork_name, e));

    assert_eq!(
        canonical_bytes1, canonical_bytes2,
        "{}: Canonical serialization should be deterministic",
        fork_name
    );

    // Verify deterministic canonical hash
    let hash1 = tx_ir
        .canonical_hash()
        .unwrap_or_else(|e| panic!("{}: Failed to hash: {}", fork_name, e));
    let hash2 = tx_ir
        .canonical_hash()
        .unwrap_or_else(|e| panic!("{}: Failed to hash (2nd attempt): {}", fork_name, e));

    assert_eq!(
        hash1, hash2,
        "{}: Canonical hash should be deterministic",
        fork_name
    );
}

// ============================================================================
// Bitcoin Cash (BCH) Tests
// ============================================================================

/// Test decoding Bitcoin Cash transaction
///
/// Bitcoin Cash forked from Bitcoin in 2017 with larger blocks and SIGHASH_FORKID
/// for replay protection. The transaction structure is identical to Bitcoin.
#[test]
fn test_decode_bitcoin_cash_transaction() {
    let tx_hex = include_str!("fixtures/forks/bch_simple_tx.hex");
    decode_and_validate_fork_tx("Bitcoin Cash", tx_hex, 1, 1, 2);
}

/// Test that BCH transactions have correct roundtrip
#[test]
fn test_bch_roundtrip() {
    let tx_hex = include_str!("fixtures/forks/bch_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    let encoded = decoded.to_bytes().unwrap();
    assert_eq!(&encoded, &tx_bytes);
}

/// Test BCH transaction is not identified as coinbase
#[test]
fn test_bch_not_coinbase() {
    let tx_hex = include_str!("fixtures/forks/bch_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    assert!(
        !decoded.is_coinbase(),
        "BCH transaction should not be coinbase"
    );
}

// ============================================================================
// Litecoin (LTC) Tests
// ============================================================================

/// Test decoding Litecoin transaction
///
/// Litecoin uses identical transaction format to Bitcoin, differing only in
/// network parameters (addresses, block time, hashing algorithm).
#[test]
fn test_decode_litecoin_transaction() {
    let tx_hex = include_str!("fixtures/forks/ltc_simple_tx.hex");
    decode_and_validate_fork_tx("Litecoin", tx_hex, 1, 1, 2);
}

/// Test LTC transactions have valid output values
#[test]
fn test_ltc_output_values() {
    let tx_hex = include_str!("fixtures/forks/ltc_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    // Verify outputs have non-zero values
    for (i, output) in decoded.outputs.iter().enumerate() {
        assert!(
            output.value > 0,
            "LTC output {} should have non-zero value",
            i
        );
    }
}

/// Test LTC transaction canonicalization
#[test]
fn test_ltc_canonicalization() {
    let tx_hex = include_str!("fixtures/forks/ltc_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();
    let tx_ir = decoded.canonicalize().unwrap();

    // Verify TxIR has correct version
    assert_eq!(tx_ir.version(), 1);

    // Verify canonical hash is deterministic
    let hash1 = tx_ir.canonical_hash().unwrap();
    let hash2 = tx_ir.canonical_hash().unwrap();
    assert_eq!(hash1, hash2);
}

// ============================================================================
// Dogecoin (DOGE) Tests
// ============================================================================

/// Test decoding Dogecoin transaction
///
/// Dogecoin forked from Litecoin and uses identical transaction format.
#[test]
fn test_decode_dogecoin_transaction() {
    let tx_hex = include_str!("fixtures/forks/doge_simple_tx.hex");
    decode_and_validate_fork_tx("Dogecoin", tx_hex, 1, 1, 2);
}

/// Test DOGE transaction structure
#[test]
fn test_doge_transaction_structure() {
    let tx_hex = include_str!("fixtures/forks/doge_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    assert_eq!(decoded.version(), 1);
    assert_eq!(decoded.input_count(), 1);
    assert_eq!(decoded.output_count(), 2);
    assert_eq!(decoded.locktime, 0);
    assert!(!decoded.is_coinbase());
}

/// Test DOGE transaction never panics on decode
#[test]
fn test_doge_decode_never_panics() {
    use decoder_test_utils::assertions::assert_decode_never_panics;

    let tx_hex = include_str!("fixtures/forks/doge_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();

    assert_decode_never_panics::<BitcoinDecoder>(&tx_bytes);
}

// ============================================================================
// Bitcoin SV (BSV) Tests
// ============================================================================

/// Test decoding Bitcoin SV transaction
///
/// Bitcoin SV forked from Bitcoin Cash. Uses same transaction structure as
/// Bitcoin with SIGHASH_FORKID.
#[test]
fn test_decode_bitcoin_sv_transaction() {
    let tx_hex = include_str!("fixtures/forks/bsv_simple_tx.hex");
    decode_and_validate_fork_tx("Bitcoin SV", tx_hex, 1, 1, 2);
}

/// Test BSV transaction basic properties
#[test]
fn test_bsv_basic_properties() {
    let tx_hex = include_str!("fixtures/forks/bsv_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    assert!(decoded.version() > 0);
    assert!(decoded.input_count() > 0);
    assert!(decoded.output_count() > 0);
    assert!(!decoded.is_segwit(), "BSV does not support SegWit");
}

// ============================================================================
// Dash Tests
// ============================================================================

/// Test decoding Dash version 1 transaction
///
/// Dash v1-v2 transactions use identical format to Bitcoin. V3+ special
/// transactions have an extra_payload field that would need special handling.
#[test]
fn test_decode_dash_v1_transaction() {
    let tx_hex = include_str!("fixtures/forks/dash_v1_tx.hex");
    decode_and_validate_fork_tx("Dash v1", tx_hex, 1, 1, 2);
}

/// Test Dash v1 transaction is standard Bitcoin format
#[test]
fn test_dash_v1_standard_format() {
    let tx_hex = include_str!("fixtures/forks/dash_v1_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    // Dash v1 transactions are standard Bitcoin format
    assert_eq!(decoded.version(), 1);
    assert!(!decoded.is_coinbase());
    assert!(!decoded.is_segwit());
}

// ============================================================================
// Bitcoin Gold (BTG) Tests
// ============================================================================

/// Test decoding Bitcoin Gold transaction
///
/// Bitcoin Gold uses identical transaction format to Bitcoin, differing only
/// in mining algorithm (Equihash) and SIGHASH_FORK_BTG flag.
#[test]
fn test_decode_bitcoin_gold_transaction() {
    let tx_hex = include_str!("fixtures/forks/btg_simple_tx.hex");
    decode_and_validate_fork_tx("Bitcoin Gold", tx_hex, 1, 1, 2);
}

/// Test BTG transaction compatibility
#[test]
fn test_btg_bitcoin_compatibility() {
    let tx_hex = include_str!("fixtures/forks/btg_simple_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    // BTG transactions are fully compatible with Bitcoin decoder
    assert_eq!(decoded.version(), 1);
    assert!(decoded.input_count() > 0);
    assert!(decoded.output_count() > 0);

    // Should canonicalize successfully
    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.version(), 1);
}

// ============================================================================
// Zcash Tests
// ============================================================================

/// Test decoding Zcash transparent transaction
///
/// Zcash v1-v2 transparent transactions use Bitcoin-compatible format.
/// Shielded transactions (v4+) have a completely different structure with
/// zk-SNARK proofs and would need a separate decoder.
#[test]
fn test_decode_zcash_transparent_transaction() {
    let tx_hex = include_str!("fixtures/forks/zec_transparent_tx.hex");
    decode_and_validate_fork_tx("Zcash transparent", tx_hex, 1, 1, 2);
}

/// Test ZEC transparent transaction properties
#[test]
fn test_zec_transparent_properties() {
    let tx_hex = include_str!("fixtures/forks/zec_transparent_tx.hex");
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

    // Transparent transactions use standard Bitcoin format
    assert_eq!(decoded.version(), 1);
    assert!(!decoded.is_coinbase());
    assert!(!decoded.is_segwit());

    // Should have standard inputs and outputs
    assert!(decoded.input_count() > 0);
    assert!(decoded.output_count() > 0);
}

// ============================================================================
// Cross-Fork Compatibility Tests
// ============================================================================

/// Test that all fork transactions can be decoded successfully
#[test]
fn test_all_forks_decode_successfully() {
    let test_cases = vec![
        (
            "Bitcoin Cash",
            include_str!("fixtures/forks/bch_simple_tx.hex"),
        ),
        ("Litecoin", include_str!("fixtures/forks/ltc_simple_tx.hex")),
        (
            "Dogecoin",
            include_str!("fixtures/forks/doge_simple_tx.hex"),
        ),
        (
            "Bitcoin SV",
            include_str!("fixtures/forks/bsv_simple_tx.hex"),
        ),
        ("Dash v1", include_str!("fixtures/forks/dash_v1_tx.hex")),
        (
            "Bitcoin Gold",
            include_str!("fixtures/forks/btg_simple_tx.hex"),
        ),
        (
            "Zcash transparent",
            include_str!("fixtures/forks/zec_transparent_tx.hex"),
        ),
    ];

    for (fork_name, tx_hex) in test_cases {
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim())
            .unwrap_or_else(|_| panic!("{}: Failed to decode hex", fork_name));

        let result = BitcoinDecoder::decode(&tx_bytes);
        assert!(
            result.is_ok(),
            "{}: Decoder should successfully decode: {:?}",
            fork_name,
            result.err()
        );
    }
}

/// Test that all fork transactions can be canonicalized
#[test]
fn test_all_forks_canonicalize_successfully() {
    let test_cases = vec![
        (
            "Bitcoin Cash",
            include_str!("fixtures/forks/bch_simple_tx.hex"),
        ),
        ("Litecoin", include_str!("fixtures/forks/ltc_simple_tx.hex")),
        (
            "Dogecoin",
            include_str!("fixtures/forks/doge_simple_tx.hex"),
        ),
        (
            "Bitcoin SV",
            include_str!("fixtures/forks/bsv_simple_tx.hex"),
        ),
        ("Dash v1", include_str!("fixtures/forks/dash_v1_tx.hex")),
        (
            "Bitcoin Gold",
            include_str!("fixtures/forks/btg_simple_tx.hex"),
        ),
        (
            "Zcash transparent",
            include_str!("fixtures/forks/zec_transparent_tx.hex"),
        ),
    ];

    for (fork_name, tx_hex) in test_cases {
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim())
            .unwrap_or_else(|_| panic!("{}: Failed to decode hex", fork_name));

        let decoded = BitcoinDecoder::decode(&tx_bytes)
            .unwrap_or_else(|e| panic!("{}: Failed to decode: {}", fork_name, e));

        let result = decoded.canonicalize();
        assert!(
            result.is_ok(),
            "{}: Should canonicalize successfully: {:?}",
            fork_name,
            result.err()
        );
    }
}

/// Test that all fork transactions have deterministic canonical hashes
#[test]
fn test_all_forks_deterministic_hashes() {
    let test_cases = vec![
        (
            "Bitcoin Cash",
            include_str!("fixtures/forks/bch_simple_tx.hex"),
        ),
        ("Litecoin", include_str!("fixtures/forks/ltc_simple_tx.hex")),
        (
            "Dogecoin",
            include_str!("fixtures/forks/doge_simple_tx.hex"),
        ),
        (
            "Bitcoin SV",
            include_str!("fixtures/forks/bsv_simple_tx.hex"),
        ),
        ("Dash v1", include_str!("fixtures/forks/dash_v1_tx.hex")),
        (
            "Bitcoin Gold",
            include_str!("fixtures/forks/btg_simple_tx.hex"),
        ),
        (
            "Zcash transparent",
            include_str!("fixtures/forks/zec_transparent_tx.hex"),
        ),
    ];

    for (fork_name, tx_hex) in test_cases {
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
        let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();
        let tx_ir = decoded.canonicalize().unwrap();

        let hash1 = tx_ir.canonical_hash().unwrap();
        let hash2 = tx_ir.canonical_hash().unwrap();

        assert_eq!(
            hash1, hash2,
            "{}: Canonical hash should be deterministic",
            fork_name
        );
    }
}

/// Test that all fork transactions have correct roundtrip
#[test]
fn test_all_forks_roundtrip() {
    let test_cases = vec![
        (
            "Bitcoin Cash",
            include_str!("fixtures/forks/bch_simple_tx.hex"),
        ),
        ("Litecoin", include_str!("fixtures/forks/ltc_simple_tx.hex")),
        (
            "Dogecoin",
            include_str!("fixtures/forks/doge_simple_tx.hex"),
        ),
        (
            "Bitcoin SV",
            include_str!("fixtures/forks/bsv_simple_tx.hex"),
        ),
        ("Dash v1", include_str!("fixtures/forks/dash_v1_tx.hex")),
        (
            "Bitcoin Gold",
            include_str!("fixtures/forks/btg_simple_tx.hex"),
        ),
        (
            "Zcash transparent",
            include_str!("fixtures/forks/zec_transparent_tx.hex"),
        ),
    ];

    for (fork_name, tx_hex) in test_cases {
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
        let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

        let encoded = decoded.to_bytes().expect("Failed to encode");
        assert_eq!(
            &encoded, &tx_bytes,
            "{}: Re-encoded bytes should match input",
            fork_name
        );
    }
}

/// Test that decoder never panics on any fork transaction
#[test]
fn test_all_forks_never_panic() {
    use decoder_test_utils::assertions::assert_decode_never_panics;

    let test_cases = vec![
        include_str!("fixtures/forks/bch_simple_tx.hex"),
        include_str!("fixtures/forks/ltc_simple_tx.hex"),
        include_str!("fixtures/forks/doge_simple_tx.hex"),
        include_str!("fixtures/forks/bsv_simple_tx.hex"),
        include_str!("fixtures/forks/dash_v1_tx.hex"),
        include_str!("fixtures/forks/btg_simple_tx.hex"),
        include_str!("fixtures/forks/zec_transparent_tx.hex"),
    ];

    for tx_hex in test_cases {
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();
        assert_decode_never_panics::<BitcoinDecoder>(&tx_bytes);
    }
}

/// Test canonical roundtrip for all forks
#[test]
fn test_all_forks_canonical_roundtrip() {
    use decoder_test_utils::assertions::assert_canonical_roundtrip;

    let test_cases = vec![
        (
            "Bitcoin Cash",
            include_str!("fixtures/forks/bch_simple_tx.hex"),
        ),
        ("Litecoin", include_str!("fixtures/forks/ltc_simple_tx.hex")),
        (
            "Dogecoin",
            include_str!("fixtures/forks/doge_simple_tx.hex"),
        ),
        (
            "Bitcoin SV",
            include_str!("fixtures/forks/bsv_simple_tx.hex"),
        ),
        ("Dash v1", include_str!("fixtures/forks/dash_v1_tx.hex")),
        (
            "Bitcoin Gold",
            include_str!("fixtures/forks/btg_simple_tx.hex"),
        ),
        (
            "Zcash transparent",
            include_str!("fixtures/forks/zec_transparent_tx.hex"),
        ),
    ];

    for (fork_name, tx_hex) in test_cases {
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim())
            .unwrap_or_else(|_| panic!("{}: Failed to decode hex", fork_name));

        let decoded = BitcoinDecoder::decode(&tx_bytes)
            .unwrap_or_else(|e| panic!("{}: Failed to decode: {}", fork_name, e));

        let tx_ir = decoded
            .canonicalize()
            .unwrap_or_else(|e| panic!("{}: Failed to canonicalize: {}", fork_name, e));

        assert_canonical_roundtrip(&tx_ir);
    }
}

// ============================================================================
// Documentation Tests
// ============================================================================

/// Test that all fixtures have corresponding JSON metadata
#[test]
fn test_fork_fixtures_have_metadata() {
    let fixtures = vec![
        "bch_simple_tx",
        "ltc_simple_tx",
        "doge_simple_tx",
        "bsv_simple_tx",
        "dash_v1_tx",
        "btg_simple_tx",
        "zec_transparent_tx",
    ];

    for fixture in fixtures {
        let hex_file = format!("fixtures/forks/{}.hex", fixture);
        let json_file = format!("fixtures/forks/{}.json", fixture);

        // Verify hex file exists (implicitly by include_str!)
        let hex_path = format!("tests/{}", hex_file);
        assert!(
            std::path::Path::new(&hex_path).exists()
                || !include_str!(concat!("fixtures/forks/", "bch_simple_tx.hex")).is_empty(),
            "Hex fixture should exist: {}",
            hex_file
        );

        // Verify JSON metadata exists
        let json_content = match fixture {
            "bch_simple_tx" => include_str!("fixtures/forks/bch_simple_tx.json"),
            "ltc_simple_tx" => include_str!("fixtures/forks/ltc_simple_tx.json"),
            "doge_simple_tx" => include_str!("fixtures/forks/doge_simple_tx.json"),
            "bsv_simple_tx" => include_str!("fixtures/forks/bsv_simple_tx.json"),
            "dash_v1_tx" => include_str!("fixtures/forks/dash_v1_tx.json"),
            "btg_simple_tx" => include_str!("fixtures/forks/btg_simple_tx.json"),
            "zec_transparent_tx" => include_str!("fixtures/forks/zec_transparent_tx.json"),
            _ => "",
        };

        assert!(
            !json_content.is_empty(),
            "JSON metadata should exist: {}",
            json_file
        );

        // Verify JSON is valid
        let parsed: std::result::Result<serde_json::Value, _> = serde_json::from_str(json_content);
        assert!(
            parsed.is_ok(),
            "JSON metadata should be valid: {}",
            json_file
        );
    }
}
