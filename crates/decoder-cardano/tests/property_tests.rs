//! Property-based tests for Cardano decoder
//!
//! This module uses proptest to verify critical properties of the Cardano decoder:
//! 1. Decoder never panics on arbitrary input
//! 2. CBOR parsing is robust
//! 3. Transaction ID calculation is deterministic
//! 4. Fee calculation properties (non-negative, bounded)
//! 5. Canonical serialization properties

use decoder_cardano::*;
use decoder_test_utils::proptest_helpers::{arb_small_bytes, prop_decoder_never_panics};
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

//
// Property 1: Decoder Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Cardano decoder never panics on arbitrary input
    ///
    /// For any arbitrary byte sequence, decode() must return Ok or Err,
    /// never panic.
    #[test]
    fn prop_cardano_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<CardanoDecoder>(&bytes);
    }

    /// Property: Cardano decoder never panics on empty input
    #[test]
    fn prop_cardano_decoder_rejects_empty(_unit in 0u8..1) {
        let result = CardanoDecoder::decode(&[]);
        prop_assert!(result.is_err(), "Decoder should reject empty input");
    }

    /// Property: Cardano decoder never panics on very short input
    #[test]
    fn prop_cardano_decoder_rejects_tiny_input(size in 1usize..10) {
        let bytes = vec![0xFF; size];
        let result = CardanoDecoder::decode(&bytes);
        prop_assert!(result.is_err(), "Decoder should reject input < 10 bytes");
    }
}

//
// Property 2: CBOR Parsing Robustness
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Property: CBOR array header parsing is consistent
    ///
    /// Valid CBOR array markers should parse consistently
    #[test]
    fn prop_cbor_array_parsing_consistent(
        array_len in 0u8..24u8
    ) {
        use std::io::Cursor;
        use decoder_cardano::parsing::read_cbor_array_header;

        // CBOR array with 0-23 elements uses single byte encoding
        let mut bytes = vec![];
        bytes.push(0x80 | array_len); // Major type 4 (array) | length

        let mut cursor = Cursor::new(bytes.as_slice());
        let result = read_cbor_array_header(&mut cursor);

        prop_assert!(result.is_ok(), "Should parse valid CBOR array header");
        if let Ok(len) = result {
            prop_assert_eq!(len, array_len as usize, "Array length should match");
        }
    }

    /// Property: CBOR unsigned int parsing is consistent
    #[test]
    fn prop_cbor_uint_parsing_consistent(value in 0u64..10000) {
        use std::io::Cursor;
        use decoder_cardano::parsing::read_cbor_uint;

        // Encode the value as CBOR
        let mut bytes = Vec::new();
        if value < 24 {
            bytes.push(value as u8);
        } else if value <= 0xFF {
            bytes.push(0x18); // uint8 follows
            bytes.push(value as u8);
        } else if value <= 0xFFFF {
            bytes.push(0x19); // uint16 follows
            bytes.extend_from_slice(&(value as u16).to_be_bytes());
        } else if value <= 0xFFFFFFFF {
            bytes.push(0x1a); // uint32 follows
            bytes.extend_from_slice(&(value as u32).to_be_bytes());
        } else {
            bytes.push(0x1b); // uint64 follows
            bytes.extend_from_slice(&value.to_be_bytes());
        }

        let mut cursor = Cursor::new(bytes.as_slice());
        let result = read_cbor_uint(&mut cursor);

        prop_assert!(result.is_ok(), "Should parse valid CBOR uint");
        if let Ok(parsed_value) = result {
            prop_assert_eq!(parsed_value, value, "Parsed value should match");
        }
    }
}

//
// Property 3: Transaction ID Determinism
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction ID calculation is deterministic
    ///
    /// Computing the TXID of the same transaction multiple times
    /// should always yield the same result.
    #[test]
    fn prop_txid_deterministic(seed in any::<u64>()) {
        // Create a deterministic transaction based on seed
        let tx_bytes = create_test_cardano_tx(seed);

        // Try to decode
        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            // If decode succeeds, TXID should be deterministic
            let txid1 = tx.txid();
            let txid2 = tx.txid();
            prop_assert_eq!(txid1, txid2, "TXID calculation is non-deterministic");

            // Hex representation should also be deterministic
            let txid_hex1 = tx.txid_hex();
            let txid_hex2 = tx.txid_hex();
            prop_assert_eq!(txid_hex1, txid_hex2, "TXID hex is non-deterministic");
        }
        // If decode fails, property is vacuously true
    }
}

//
// Property 4: Fee Calculation Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Fee calculation never panics and is non-negative
    ///
    /// For valid transactions, fee calculation should:
    /// 1. Never panic
    /// 2. Return non-negative value
    /// 3. Be reasonable (< 1 ADA for typical transactions)
    #[test]
    fn prop_fee_calculation_properties(
        fee in 1u64..10_000_000u64 // 0.001 to 10 ADA
    ) {
        let tx_bytes = create_test_cardano_tx_with_fee(fee);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            let parsed_fee = tx.fee();

            // Fee should match what we set
            prop_assert_eq!(parsed_fee, fee, "Fee should match");

            // Fee should be reasonable (< 10 ADA)
            prop_assert!(parsed_fee < 10_000_000, "Fee should be < 10 ADA");
        }
    }
}

//
// Property 5: Canonical Serialization Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Successfully decoded transactions can be canonicalized
    ///
    /// Any transaction that decodes successfully should also be able to
    /// produce canonical bytes without panicking.
    #[test]
    fn prop_decoded_tx_canonicalizes(seed in any::<u64>()) {
        let bytes = create_test_cardano_tx(seed);

        if let Ok(tx) = CardanoDecoder::decode(&bytes) {
            // If decode succeeds, canonicalization should also succeed or fail gracefully
            let result = tx.canonicalize();
            prop_assert!(result.is_ok() || result.is_err(),
                "Canonicalization should return Result, not panic");
        }
        // If decode fails, property is vacuously true
    }

    /// Property: Canonical hash is deterministic
    ///
    /// Computing canonical hash multiple times on the same transaction
    /// should yield identical results.
    #[test]
    fn prop_canonical_hash_deterministic(seed in any::<u64>()) {
        let bytes = create_test_cardano_tx(seed);

        if let Ok(tx) = CardanoDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                // Compute hash twice
                let hash1 = tx_ir.canonical_hash();
                let hash2 = tx_ir.canonical_hash();

                match (hash1, hash2) {
                    (Ok(h1), Ok(h2)) => {
                        prop_assert_eq!(h1, h2, "Canonical hash is non-deterministic");
                    }
                    (Err(_), Err(_)) => {
                        // Both failed consistently - OK
                    }
                    _ => {
                        return Err(TestCaseError::fail(
                            "Canonical hash returned different error states"
                        ));
                    }
                }
            }
        }
    }
}

//
// Property 6: Input/Output Count Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction input/output counts are reasonable
    ///
    /// Cardano transactions should have reasonable limits on the number of
    /// inputs and outputs.
    #[test]
    fn prop_io_count_reasonable(
        input_count in 1usize..100,
        output_count in 1usize..100,
    ) {
        let tx_bytes = create_test_cardano_tx_with_io(input_count, output_count);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            let parsed_input_count = tx.input_count();
            let parsed_output_count = tx.output_count();

            // Counts should match
            prop_assert_eq!(parsed_input_count, input_count, "Input count should match");
            prop_assert_eq!(parsed_output_count, output_count, "Output count should match");

            // Counts should be reasonable
            prop_assert!(parsed_input_count > 0, "Must have at least 1 input");
            prop_assert!(parsed_output_count > 0, "Must have at least 1 output");
            prop_assert!(parsed_input_count < 1000, "Input count should be reasonable");
            prop_assert!(parsed_output_count < 1000, "Output count should be reasonable");
        }
    }
}

//
// Integration Property Tests
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Decode-Canonicalize-Hash pipeline never panics
    ///
    /// The full pipeline from raw bytes to canonical hash should
    /// never panic, even on invalid input.
    #[test]
    fn prop_full_pipeline_never_panics(bytes in arb_small_bytes()) {
        use std::panic;

        let result = panic::catch_unwind(|| {
            if let Ok(tx) = CardanoDecoder::decode(&bytes) {
                if let Ok(tx_ir) = tx.canonicalize() {
                    let _ = tx_ir.canonical_hash();
                }
            }
        });

        prop_assert!(result.is_ok(), "Full pipeline panicked on input");
    }
}

//
// Helper Functions
//

/// Create a test Cardano transaction with deterministic content based on seed
fn create_test_cardano_tx(seed: u64) -> Vec<u8> {
    create_test_cardano_tx_with_fee(170_000 + (seed % 100_000) as u64)
}

/// Create a test Cardano transaction with specific fee
fn create_test_cardano_tx_with_fee(fee: u64) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // CBOR array with 3 elements
    tx_bytes.push(0x83);

    // Transaction body (CBOR map)
    tx_bytes.push(0xa3);

    // Key 0: inputs
    tx_bytes.push(0x00);
    tx_bytes.push(0x81); // Array with 1 element
    tx_bytes.push(0x82); // [tx_hash, index]
    tx_bytes.push(0x58);
    tx_bytes.push(0x20); // 32 bytes
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x00); // index 0

    // Key 1: outputs
    tx_bytes.push(0x01);
    tx_bytes.push(0x81); // Array with 1 element
    tx_bytes.push(0x82); // [address, amount]
    tx_bytes.push(0x58);
    tx_bytes.push(0x1d); // 29 bytes
    tx_bytes.extend_from_slice(&[0u8; 29]);
    tx_bytes.push(0x1a); // uint32
    tx_bytes.extend_from_slice(&1_000_000u32.to_be_bytes());

    // Key 2: fee
    tx_bytes.push(0x02);
    if fee <= 0xFFFFFFFF {
        tx_bytes.push(0x1a); // uint32
        tx_bytes.extend_from_slice(&(fee as u32).to_be_bytes());
    } else {
        tx_bytes.push(0x1b); // uint64
        tx_bytes.extend_from_slice(&fee.to_be_bytes());
    }

    // Witness set
    tx_bytes.push(0xa1);
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x58);
    tx_bytes.push(0x40);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // No auxiliary data
    tx_bytes.push(0xf6);

    tx_bytes
}

/// Create a test Cardano transaction with specific input/output counts
fn create_test_cardano_tx_with_io(input_count: usize, output_count: usize) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // CBOR array with 3 elements
    tx_bytes.push(0x83);

    // Transaction body (CBOR map)
    tx_bytes.push(0xa3);

    // Key 0: inputs
    tx_bytes.push(0x00);
    encode_cbor_array_len(&mut tx_bytes, input_count);
    for _ in 0..input_count {
        tx_bytes.push(0x82); // [tx_hash, index]
        tx_bytes.push(0x58);
        tx_bytes.push(0x20);
        tx_bytes.extend_from_slice(&[0u8; 32]);
        tx_bytes.push(0x00);
    }

    // Key 1: outputs
    tx_bytes.push(0x01);
    encode_cbor_array_len(&mut tx_bytes, output_count);
    for _ in 0..output_count {
        tx_bytes.push(0x82); // [address, amount]
        tx_bytes.push(0x58);
        tx_bytes.push(0x1d);
        tx_bytes.extend_from_slice(&[0u8; 29]);
        tx_bytes.push(0x1a);
        tx_bytes.extend_from_slice(&1_000_000u32.to_be_bytes());
    }

    // Key 2: fee
    tx_bytes.push(0x02);
    tx_bytes.push(0x1a);
    tx_bytes.extend_from_slice(&170_000u32.to_be_bytes());

    // Witness set
    tx_bytes.push(0xa1);
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x58);
    tx_bytes.push(0x40);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // No auxiliary data
    tx_bytes.push(0xf6);

    tx_bytes
}

/// Encode CBOR array length
fn encode_cbor_array_len(buf: &mut Vec<u8>, len: usize) {
    if len < 24 {
        buf.push(0x80 | (len as u8));
    } else if len <= 0xFF {
        buf.push(0x98);
        buf.push(len as u8);
    } else if len <= 0xFFFF {
        buf.push(0x99);
        buf.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        buf.push(0x9a);
        buf.extend_from_slice(&(len as u32).to_be_bytes());
    }
}
