//! Decoder-specific property-based tests
//!
//! Tests for decoder implementations and encoding/decoding logic.

use decoder_encodings::rlp::RlpItem;
use proptest::prelude::*;

/// Generate arbitrary RLP data items (non-list)
#[allow(dead_code)]
fn arb_rlp_data() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..100)
}

/// Generate arbitrary small u64 values for RLP encoding
fn arb_rlp_u64() -> impl Strategy<Value = u64> {
    0u64..1_000_000
}

/// Generate arbitrary small u128 values for RLP encoding
fn arb_rlp_u128() -> impl Strategy<Value = u128> {
    0u128..1_000_000
}

proptest! {
    /// Property: RLP decoding never panics on any input
    #[test]
    fn prop_rlp_decode_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        use std::panic;

        let result = panic::catch_unwind(|| {
            let _ = RlpItem::decode(&bytes);
        });

        prop_assert!(result.is_ok(), "RLP decoder panicked on input");
    }

    /// Property: RLP single byte values decode correctly
    #[test]
    fn prop_rlp_single_byte(byte in 0u8..=0x7f) {
        let encoded = vec![byte];
        let decoded = RlpItem::decode(&encoded).unwrap();

        match decoded {
            RlpItem::Data(data) => {
                prop_assert_eq!(data.len(), 1);
                prop_assert_eq!(data[0], byte);
            }
            _ => prop_assert!(false, "Expected Data item"),
        }
    }

    /// Property: RLP empty data decodes correctly
    #[test]
    fn prop_rlp_empty_data(_seed in any::<u64>()) {
        let encoded = vec![0x80]; // RLP encoding of empty string
        let decoded = RlpItem::decode(&encoded).unwrap();

        match decoded {
            RlpItem::Data(data) => prop_assert!(data.is_empty()),
            _ => prop_assert!(false, "Expected Data item"),
        }
    }

    /// Property: RLP empty list decodes correctly
    #[test]
    fn prop_rlp_empty_list(_seed in any::<u64>()) {
        let encoded = vec![0xc0]; // RLP encoding of empty list
        let decoded = RlpItem::decode(&encoded).unwrap();

        match decoded {
            RlpItem::List(list) => prop_assert!(list.is_empty()),
            _ => prop_assert!(false, "Expected List item"),
        }
    }

    /// Property: RLP short strings (0-55 bytes) encode/decode consistently
    #[test]
    fn prop_rlp_short_string(data in prop::collection::vec(any::<u8>(), 1..=55)) {
        // Canonical encoding: a single byte below 0x80 is encoded as itself;
        // everything else uses the 0x80 + length prefix. The decoder is
        // strict and rejects the non-canonical prefixed single-byte form.
        let encoded = if data.len() == 1 && data[0] < 0x80 {
            vec![data[0]]
        } else {
            let mut encoded = vec![0x80 + data.len() as u8];
            encoded.extend_from_slice(&data);
            encoded
        };

        let decoded = RlpItem::decode(&encoded).unwrap();

        match decoded {
            RlpItem::Data(decoded_data) => prop_assert_eq!(data, decoded_data),
            _ => prop_assert!(false, "Expected Data item"),
        }
    }

    /// Property: RLP u64 conversion never panics for valid data
    #[test]
    fn prop_rlp_u64_no_panic(value in arb_rlp_u64()) {
        use std::panic;

        // Encode u64 as big-endian bytes
        let bytes = if value == 0 {
            vec![]
        } else {
            let mut b = vec![];
            let mut v = value;
            while v > 0 {
                b.insert(0, (v & 0xFF) as u8);
                v >>= 8;
            }
            b
        };

        // Encode as RLP (canonical: single byte < 0x80 encoded as itself)
        let encoded = if bytes.is_empty() {
            vec![0x80]
        } else if bytes.len() == 1 && bytes[0] < 0x80 {
            vec![bytes[0]]
        } else if bytes.len() <= 55 {
            let mut e = vec![0x80 + bytes.len() as u8];
            e.extend_from_slice(&bytes);
            e
        } else {
            // Should not happen for small values
            return Ok(());
        };

        let decoded = RlpItem::decode(&encoded).unwrap();

        let result = panic::catch_unwind(|| {
            let _ = decoded.as_u64();
        });

        prop_assert!(result.is_ok(), "RLP u64 conversion panicked");
    }

    /// Property: RLP u128 conversion handles values correctly
    #[test]
    fn prop_rlp_u128_handling(value in arb_rlp_u128()) {
        // Encode u128 as big-endian bytes
        let bytes = if value == 0 {
            vec![]
        } else {
            let mut b = vec![];
            let mut v = value;
            while v > 0 {
                b.insert(0, (v & 0xFF) as u8);
                v >>= 8;
            }
            b
        };

        // Encode as RLP (canonical: single byte < 0x80 encoded as itself)
        let encoded = if bytes.is_empty() {
            vec![0x80]
        } else if bytes.len() == 1 && bytes[0] < 0x80 {
            vec![bytes[0]]
        } else if bytes.len() <= 55 {
            let mut e = vec![0x80 + bytes.len() as u8];
            e.extend_from_slice(&bytes);
            e
        } else {
            return Ok(()); // Skip long encodings
        };

        let decoded = RlpItem::decode(&encoded).unwrap();
        let result = decoded.as_u128();

        prop_assert!(result.is_ok(), "RLP u128 conversion failed");
        prop_assert_eq!(value, result.unwrap());
    }

    /// Property: RLP rejects data with leading zeros for integers
    #[test]
    fn prop_rlp_rejects_leading_zeros(value in 1u8..=255) {
        // Create RLP encoding with leading zero: [0x82, 0x00, value]
        let encoded = vec![0x82, 0x00, value];

        let decoded = RlpItem::decode(&encoded).unwrap();

        // Should decode as Data (valid RLP for a string)
        match &decoded {
            RlpItem::Data(_) => {}
            _ => prop_assert!(false, "Expected Data item"),
        }

        // But conversion to integer should fail due to leading zero
        let u64_result = decoded.as_u64();
        let u128_result = decoded.as_u128();

        prop_assert!(u64_result.is_err(), "Should reject leading zeros for u64");
        prop_assert!(u128_result.is_err(), "Should reject leading zeros for u128");
    }

    /// Property: RLP list/data type is preserved
    #[test]
    fn prop_rlp_list_vs_data_distinction(is_list in any::<bool>()) {
        let encoded = if is_list {
            vec![0xc0] // Empty list
        } else {
            vec![0x80] // Empty data
        };

        let decoded = RlpItem::decode(&encoded).unwrap();

        if is_list {
            prop_assert!(matches!(decoded, RlpItem::List(_)));
            prop_assert!(decoded.as_list().is_ok());
            prop_assert!(decoded.as_data().is_err());
        } else {
            prop_assert!(matches!(decoded, RlpItem::Data(_)));
            prop_assert!(decoded.as_data().is_ok());
            prop_assert!(decoded.as_list().is_err());
        }
    }

    /// Property: RLP handles maximum single byte (0x7f)
    #[test]
    fn prop_rlp_max_single_byte(_seed in any::<u64>()) {
        let encoded = vec![0x7f];
        let decoded = RlpItem::decode(&encoded).unwrap();

        match decoded {
            RlpItem::Data(data) => {
                prop_assert_eq!(data.len(), 1);
                prop_assert_eq!(data[0], 0x7f);
            }
            _ => prop_assert!(false, "Expected Data item"),
        }
    }

    /// Property: RLP handles minimum non-single byte (0x80)
    #[test]
    fn prop_rlp_min_non_single_byte(_seed in any::<u64>()) {
        // 0x80 is empty string
        let encoded = vec![0x80];
        let decoded = RlpItem::decode(&encoded).unwrap();

        match decoded {
            RlpItem::Data(data) => prop_assert!(data.is_empty()),
            _ => prop_assert!(false, "Expected Data item"),
        }
    }

    /// Property: RLP length overflow is handled gracefully
    #[test]
    fn prop_rlp_length_overflow_safe(prefix in 0xb8u8..=0xbf) {
        // Create malicious RLP with large length indicator
        let length_of_length = (prefix - 0xb7) as usize;
        let mut encoded = vec![prefix];

        // Add maximum length bytes (all 0xFF)
        encoded.extend(std::iter::repeat_n(0xFF, length_of_length));

        // This should either:
        // 1. Return error for length overflow (correct)
        // 2. Return error for incomplete data (correct)
        // 3. Never panic (critical)
        let result = RlpItem::decode(&encoded);

        // The important property: it should never panic
        prop_assert!(result.is_ok() || result.is_err());
    }
}
