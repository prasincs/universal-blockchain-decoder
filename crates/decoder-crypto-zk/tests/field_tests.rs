//! Field arithmetic tests
//!
//! Tests for STARK field operations including:
//! - Basic arithmetic (add, sub, mul, div)
//! - Property tests (associativity, commutativity, etc.)
//! - Hex conversions
//! - Edge cases (zero, one, overflow)

use decoder_crypto_zk::field::{FieldElement, FieldExt};
use proptest::prelude::*;

// ============================================================================
// Basic Arithmetic Tests
// ============================================================================

#[test]
fn test_field_addition() {
    let a = FieldElement::from(100u64);
    let b = FieldElement::from(200u64);
    let sum = a + b;
    assert_eq!(sum, FieldElement::from(300u64));
}

#[test]
fn test_field_subtraction() {
    let a = FieldElement::from(200u64);
    let b = FieldElement::from(100u64);
    let diff = a - b;
    assert_eq!(diff, FieldElement::from(100u64));
}

#[test]
fn test_field_multiplication() {
    let a = FieldElement::from(10u64);
    let b = FieldElement::from(20u64);
    let product = a * b;
    assert_eq!(product, FieldElement::from(200u64));
}

#[test]
fn test_field_division() {
    let a = FieldElement::from(100u64);
    let b = FieldElement::from(10u64);
    // Division is multiplication by inverse in field arithmetic
    let quotient = a * b.inverse().unwrap();
    assert_eq!(quotient, FieldElement::from(10u64));
}

// ============================================================================
// Constants Tests
// ============================================================================

#[test]
fn test_field_constants() {
    assert_eq!(FieldElement::ZERO, FieldElement::from(0u64));
    assert_eq!(FieldElement::ONE, FieldElement::from(1u64));
    assert_eq!(FieldElement::TWO, FieldElement::from(2u64));
    assert_eq!(FieldElement::THREE, FieldElement::from(3u64));
}

#[test]
fn test_zero_identity() {
    let a = FieldElement::from(42u64);
    assert_eq!(a + FieldElement::ZERO, a);
    assert_eq!(a - FieldElement::ZERO, a);
    assert_eq!(a * FieldElement::ZERO, FieldElement::ZERO);
}

#[test]
fn test_one_identity() {
    let a = FieldElement::from(42u64);
    assert_eq!(a * FieldElement::ONE, a);
    assert_eq!(a * FieldElement::ONE.inverse().unwrap(), a);
}

// ============================================================================
// Hex Conversion Tests
// ============================================================================

#[test]
fn test_hex_conversion_basic() {
    let fe = FieldElement::from_hex("0x123").unwrap();
    assert_eq!(fe, FieldElement::from(0x123u64));
}

#[test]
fn test_hex_conversion_without_prefix() {
    let fe = FieldElement::from_hex("123").unwrap();
    assert_eq!(fe, FieldElement::from(0x123u64));
}

#[test]
fn test_hex_roundtrip() {
    let original = FieldElement::from(0xABCDEF123456u64);
    let hex = original.to_hex();
    let decoded = FieldElement::from_hex(&hex).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_hex_to_string() {
    let fe = FieldElement::from(0x123u64);
    let hex = fe.to_hex();
    assert!(hex.starts_with("0x"));
}

// ============================================================================
// Large Number Tests
// ============================================================================

#[test]
fn test_large_number_hex() {
    // Test with a large hex number
    let hex = "0x0800000000000011000000000000000000000000000000000000000000000000";
    let fe = FieldElement::from_hex(hex).unwrap();
    assert_ne!(fe, FieldElement::ZERO);
}

#[test]
fn test_max_field_element() {
    // STARK field prime minus 1
    let max_hex = "0x0800000000000011000000000000000000000000000000000000000000000000";
    let fe = FieldElement::from_hex(max_hex);
    assert!(fe.is_ok());
}

// ============================================================================
// Bytes Conversion Tests
// ============================================================================

#[test]
fn test_bytes_roundtrip() {
    let original = FieldElement::from(0x123456789ABCDEFu64);
    let bytes = original.to_bytes_be();
    let decoded = FieldElement::from_bytes_be(&bytes);
    assert_eq!(original, decoded);
}

#[test]
fn test_bytes_length() {
    let fe = FieldElement::from(42u64);
    let bytes = fe.to_bytes_be();
    assert_eq!(bytes.len(), 32); // 256 bits = 32 bytes
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_zero_squared() {
    let zero = FieldElement::ZERO;
    assert_eq!(zero.square(), FieldElement::ZERO);
}

#[test]
fn test_one_squared() {
    let one = FieldElement::ONE;
    assert_eq!(one.square(), FieldElement::ONE);
}

#[test]
fn test_sqrt_of_perfect_square() {
    let four = FieldElement::from(4u64);
    let sqrt = four.sqrt();
    assert!(sqrt.is_some());
    assert_eq!(sqrt.unwrap().square(), four);
}

#[test]
fn test_invert_non_zero() {
    let a = FieldElement::from(5u64);
    let inv = a.inverse();
    assert!(inv.is_some());
    assert_eq!(a * inv.unwrap(), FieldElement::ONE);
}

#[test]
fn test_zero_has_no_inverse() {
    let zero = FieldElement::ZERO;
    assert!(zero.inverse().is_none());
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    /// Test that addition is commutative: a + b = b + a
    #[test]
    fn prop_addition_commutative(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        assert_eq!(fa + fb, fb + fa);
    }

    /// Test that addition is associative: (a + b) + c = a + (b + c)
    #[test]
    fn prop_addition_associative(a in 0u64..1000000, b in 0u64..1000000, c in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let fc = FieldElement::from(c);
        assert_eq!((fa + fb) + fc, fa + (fb + fc));
    }

    /// Test that multiplication is commutative: a * b = b * a
    #[test]
    fn prop_multiplication_commutative(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        assert_eq!(fa * fb, fb * fa);
    }

    /// Test that multiplication is associative: (a * b) * c = a * (b * c)
    #[test]
    fn prop_multiplication_associative(a in 1u64..1000, b in 1u64..1000, c in 1u64..1000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let fc = FieldElement::from(c);
        assert_eq!((fa * fb) * fc, fa * (fb * fc));
    }

    /// Test distributive law: a * (b + c) = a * b + a * c
    #[test]
    fn prop_distributive(a in 0u64..10000, b in 0u64..10000, c in 0u64..10000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let fc = FieldElement::from(c);
        assert_eq!(fa * (fb + fc), fa * fb + fa * fc);
    }

    /// Test hex roundtrip: from_hex(to_hex(x)) = x
    #[test]
    fn prop_hex_roundtrip(a in 0u64..u64::MAX) {
        let original = FieldElement::from(a);
        let hex = original.to_hex();
        let decoded = FieldElement::from_hex(&hex).unwrap();
        assert_eq!(original, decoded);
    }

    /// Test bytes roundtrip: from_bytes(to_bytes(x)) = x
    #[test]
    fn prop_bytes_roundtrip(a in 0u64..u64::MAX) {
        let original = FieldElement::from(a);
        let bytes = original.to_bytes_be();
        let decoded = FieldElement::from_bytes_be(&bytes);
        assert_eq!(original, decoded);
    }

    /// Test that subtraction is inverse of addition: (a + b) - b = a
    #[test]
    fn prop_subtraction_inverse_of_addition(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        assert_eq!((fa + fb) - fb, fa);
    }

    /// Test that multiplication by inverse equals one (for non-zero elements)
    #[test]
    fn prop_multiplication_inverse(a in 1u64..1000000) {
        let fa = FieldElement::from(a);
        if let Some(inv) = fa.inverse() {
            assert_eq!(fa * inv, FieldElement::ONE);
        }
    }

    /// Test that squaring is same as multiplying by self
    #[test]
    fn prop_square_equals_multiply(a in 0u64..1000000) {
        let fa = FieldElement::from(a);
        assert_eq!(fa.square(), fa * fa);
    }

    /// Test that double is same as adding to self
    #[test]
    fn prop_double_equals_add(a in 0u64..1000000) {
        let fa = FieldElement::from(a);
        assert_eq!(fa.double(), fa + fa);
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

#[test]
fn test_operations_deterministic() {
    let a = FieldElement::from(123u64);
    let b = FieldElement::from(456u64);

    // Run operations multiple times
    let sum1 = a + b;
    let sum2 = a + b;
    assert_eq!(sum1, sum2);

    let product1 = a * b;
    let product2 = a * b;
    assert_eq!(product1, product2);
}

#[test]
fn test_hex_conversion_deterministic() {
    let fe = FieldElement::from(0x123456u64);
    let hex1 = fe.to_hex();
    let hex2 = fe.to_hex();
    assert_eq!(hex1, hex2);
}
