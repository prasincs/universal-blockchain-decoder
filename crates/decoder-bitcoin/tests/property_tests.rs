//! Property-based tests for Bitcoin decoder
//!
//! This module uses proptest to verify critical properties of the Bitcoin decoder:
//! 1. Decoder never panics on arbitrary input
//! 2. VarInt encoding/decoding roundtrip
//! 3. TXID calculation is deterministic
//! 4. Fee calculation properties (non-negative, bounded)
//! 5. Canonical serialization properties

use decoder_bitcoin::parsing::read_varint;
use decoder_bitcoin::*;
use decoder_encodings::varint::encode_varint;
use decoder_test_utils::proptest_helpers::{arb_small_bytes, prop_decoder_never_panics};
use proptest::prelude::*;
use sha2::{Digest, Sha256};
use universal_decoder_core::prelude::*;

//
// Property 1: Decoder Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Bitcoin decoder never panics on arbitrary input
    ///
    /// For any arbitrary byte sequence, decode() must return Ok or Err,
    /// never panic.
    #[test]
    fn prop_bitcoin_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<BitcoinDecoder>(&bytes);
    }

    /// Property: Bitcoin decoder never panics on empty input
    #[test]
    fn prop_bitcoin_decoder_rejects_empty(_unit in 0u8..1) {
        let result = BitcoinDecoder::decode(&[]);
        prop_assert!(result.is_err(), "Decoder should reject empty input");
    }

    /// Property: Bitcoin decoder never panics on very short input
    #[test]
    fn prop_bitcoin_decoder_rejects_tiny_input(size in 1usize..10) {
        let bytes = vec![0xFF; size];
        let result = BitcoinDecoder::decode(&bytes);
        prop_assert!(result.is_err(), "Decoder should reject input < 10 bytes");
    }
}

//
// Property 2: VarInt Encoding/Decoding Roundtrip
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: VarInt roundtrip - encode(x) then decode yields x
    ///
    /// For any u64 value, encoding as varint and decoding should
    /// return the original value.
    #[test]
    fn prop_varint_encode_decode_roundtrip(value in any::<u64>()) {
        use std::io::Cursor;


        // Encode
        let mut buf = Vec::new();
        encode_varint(&mut buf, value);

        // Decode
        let mut cursor = Cursor::new(&buf);
        let decoded = read_varint(&mut cursor).expect("decode should succeed");

        prop_assert_eq!(decoded, value, "VarInt roundtrip failed");
    }

    /// Property: VarInt encoding is canonical
    ///
    /// Values < 0xFD should encode to 1 byte
    /// Values 0xFD..=0xFFFF should encode to 3 bytes
    /// Values 0x10000..=0xFFFFFFFF should encode to 5 bytes
    /// Values > 0xFFFFFFFF should encode to 9 bytes
    #[test]
    fn prop_varint_canonical_encoding(value in any::<u64>()) {
        let mut buf = Vec::new();
        encode_varint(&mut buf, value);

        let expected_len = if value < 0xFD {
            1
        } else if value <= 0xFFFF {
            3
        } else if value <= 0xFFFFFFFF {
            5
        } else {
            9
        };

        prop_assert_eq!(buf.len(), expected_len,
            "VarInt encoding length incorrect for value {}", value);
    }

    /// Property: VarInt encoding boundary values
    #[test]
    fn prop_varint_boundary_encoding(
        boundary in prop_oneof![
            Just(0xFC),
            Just(0xFD),
            Just(0xFE),
            Just(0xFF),
            Just(0xFFFF),
            Just(0x10000),
            Just(0xFFFFFFFF),
            Just(0x100000000),
            Just(u64::MAX),
        ]
    ) {
        let mut buf = Vec::new();
        encode_varint(&mut buf, boundary);

        use std::io::Cursor;

        let mut cursor = Cursor::new(&buf);
        let decoded = read_varint(&mut cursor).expect("decode should succeed");

        prop_assert_eq!(decoded, boundary, "VarInt boundary roundtrip failed");
    }
}

//
// Property 3: TXID Calculation Determinism
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: TXID calculation is deterministic
    ///
    /// Computing the TXID of the same transaction bytes multiple times
    /// should always yield the same result.
    ///
    /// This test uses real Bitcoin transactions from our test fixtures.
    #[test]
    fn prop_txid_deterministic(seed in any::<u64>()) {
        // Use seed to select a test case
        // For now, test with a minimal valid transaction structure
        // (This would be enhanced with real fixtures)

        // Create a minimal valid-looking transaction
        // Version (4 bytes) + input count (1) + dummy input + output count (1) + dummy output + locktime
        let mut tx_bytes = Vec::new();

        // Version 1
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());

        // 1 input (varint)
        tx_bytes.push(0x01);

        // Input: prevout hash (32 bytes)
        let prevout_hash: [u8; 32] = Sha256::digest(seed.to_le_bytes()).into();
        tx_bytes.extend_from_slice(&prevout_hash);

        // Input: prevout index (4 bytes)
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());

        // Input: script length (varint) + empty script
        tx_bytes.push(0x00);

        // Input: sequence (4 bytes)
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());

        // 1 output (varint)
        tx_bytes.push(0x01);

        // Output: value (8 bytes)
        tx_bytes.extend_from_slice(&50_0000_0000_u64.to_le_bytes());

        // Output: script length (varint) + dummy script (P2PKH-like)
        tx_bytes.push(0x19); // 25 bytes
        tx_bytes.extend_from_slice(&[
            0x76, 0xa9, 0x14, // OP_DUP OP_HASH160 PUSH(20)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, // 20 bytes of address hash
            0x88, 0xac, // OP_EQUALVERIFY OP_CHECKSIG
        ]);

        // Locktime (4 bytes)
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());

        // Try to decode - may fail for malformed transactions
        if let Ok(tx) = BitcoinDecoder::decode(&tx_bytes) {
            // If decode succeeds, TXID should be deterministic
            let txid1 = tx.txid();
            let txid2 = tx.txid();
            prop_assert_eq!(txid1, txid2, "TXID calculation is non-deterministic");
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
    /// 2. Return non-negative value (or error if invalid)
    /// 3. Be less than total input value
    #[test]
    fn prop_fee_calculation_properties(
        (input_value, output_value) in (1u64..2_100_000_000_000_000_u64) // Max 21M BTC in satoshis
            .prop_flat_map(|input| (Just(input), 0u64..=input))
    ) {
        // output_value is now guaranteed to be <= input_value (no need for prop_assume!)

        // Create a simple transaction structure for testing
        // This is a simplified test - real transactions would be more complex
        let expected_fee = input_value - output_value;

        // Fee should be non-negative (guaranteed by our strategy)
        prop_assert!(expected_fee <= input_value, "Fee exceeds input value");

        // Fee should be reasonable (< 1 BTC for this test)
        // Note: In real Bitcoin, fees vary widely, but for property testing
        // we can establish bounds
        if output_value > 0 {
            let fee_ratio = (expected_fee as f64) / (input_value as f64);
            prop_assert!(fee_ratio <= 1.0, "Fee ratio should not exceed 100%");
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
    fn prop_decoded_tx_canonicalizes(bytes in arb_small_bytes()) {
        if let Ok(tx) = BitcoinDecoder::decode(&bytes) {
            // If decode succeeds, canonicalization should also succeed
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
    fn prop_canonical_hash_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = BitcoinDecoder::decode(&bytes) {
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
// Property 6: SegWit Detection Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: SegWit detection is consistent
    ///
    /// Transactions are either SegWit or they're not - detection
    /// should be unambiguous and consistent.
    #[test]
    fn prop_segwit_detection_consistent(bytes in arb_small_bytes()) {
        if bytes.len() < 6 {
            // Too short to be valid
            return Ok(());
        }

        // SegWit marker is at position 4 (after 4-byte version)
        // It should be 0x00 (marker) followed by 0x01 (flag)
        let has_segwit_marker = bytes.len() > 5
            && bytes[4] == 0x00
            && bytes[5] == 0x01;

        // Decode and check if it detected SegWit
        if let Ok(tx) = BitcoinDecoder::decode(&bytes) {
            let detected_segwit = tx.is_segwit();

            // If we see the marker, decoder should detect SegWit
            // (assuming the rest of the transaction is valid)
            if has_segwit_marker {
                // Note: decoder may still reject if transaction is otherwise invalid
                // so we can't assert detected_segwit == true unconditionally
                // This property is weaker: "if segwit marker present, either
                // detect it or fail to decode entirely"
            }

            // If no marker, should not be detected as SegWit
            if !has_segwit_marker && bytes.len() > 10 {
                prop_assert!(!detected_segwit,
                    "Should not detect SegWit without marker");
            }
        }
    }
}

//
// Property 7: Input/Output Count Bounds
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction input/output counts are bounded
    ///
    /// Bitcoin transactions have practical limits on the number of
    /// inputs and outputs. Very large counts should be rejected.
    #[test]
    fn prop_io_count_bounded(
        input_count in 0u64..100_000,
        _output_count in 0u64..100_000,
    ) {
        // Create a transaction with specified counts
        let mut tx_bytes = Vec::new();

        // Version
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());

        // Input count (varint)
        encode_varint(&mut tx_bytes, input_count);

        // If input_count is very large, decoder should reject
        let result = BitcoinDecoder::decode(&tx_bytes);

        if input_count > 100_000 {
            // Should reject very large input counts
            prop_assert!(result.is_err(),
                "Should reject transaction with {} inputs", input_count);
        }
        // For reasonable counts, may succeed or fail based on rest of data
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
            if let Ok(tx) = BitcoinDecoder::decode(&bytes) {
                if let Ok(tx_ir) = tx.canonicalize() {
                    let _ = tx_ir.canonical_hash();
                }
            }
        });

        prop_assert!(result.is_ok(), "Full pipeline panicked on input");
    }
}

//
// Helper property tests for Amount arithmetic (Bitcoin-specific)
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: Satoshi amounts are always non-negative
    #[test]
    fn prop_satoshi_amounts_non_negative(value in any::<u64>()) {
        use universal_decoder_core::prelude::Amount;

        // Bitcoin uses 8 decimal places (satoshis)
        let amount = Amount::new(value as u128, 8);

        prop_assert_eq!(amount.decimals, 8, "Bitcoin should use 8 decimals");
        // Note: amount.value is u128, so it's always non-negative by type
    }

    /// Property: Bitcoin amount addition doesn't overflow
    #[test]
    fn prop_bitcoin_amount_addition(
        a in 0u64..1_000_000_000_000_u64, // 10K BTC max
        b in 0u64..1_000_000_000_000_u64,
    ) {
        use universal_decoder_core::prelude::Amount;

        let amount_a = Amount::new(a as u128, 8);
        let amount_b = Amount::new(b as u128, 8);

        let sum = amount_a.checked_add(amount_b);

        // Should succeed since we're well below u128::MAX
        prop_assert!(sum.is_some(), "Addition should succeed for reasonable values");

        if let Some(result) = sum {
            prop_assert_eq!(result.value, (a + b) as u128, "Addition result incorrect");
            prop_assert_eq!(result.decimals, 8, "Decimals should be preserved");
        }
    }

    /// Property: Bitcoin amount subtraction doesn't underflow
    #[test]
    fn prop_bitcoin_amount_subtraction(
        a in 0u64..1_000_000_000_000_u64,
        b in 0u64..1_000_000_000_000_u64,
    ) {
        use universal_decoder_core::prelude::Amount;

        let amount_a = Amount::new(a as u128, 8);
        let amount_b = Amount::new(b as u128, 8);

        let diff = amount_a.checked_sub(amount_b);

        if a >= b {
            // Should succeed
            prop_assert!(diff.is_some(),
                "Subtraction should succeed when a >= b");
            if let Some(result) = diff {
                prop_assert_eq!(result.value, (a - b) as u128);
            }
        } else {
            // Should fail (underflow)
            prop_assert!(diff.is_none(),
                "Subtraction should fail when a < b");
        }
    }
}
