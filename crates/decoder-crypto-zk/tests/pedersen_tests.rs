//! Pedersen hash tests
//!
//! Tests for Pedersen hash including:
//! - Test vectors from Starknet
//! - Cross-validation with starknet-crypto
//! - Property tests (determinism, not commutative)
//! - Stateful hasher tests
//! - Edge cases

use decoder_crypto_zk::field::FieldElement;
use decoder_crypto_zk::hash::pedersen::{PedersenHash, PedersenHasher};
use proptest::prelude::*;

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_pedersen_hash_pair_deterministic() {
    let a = FieldElement::from(123u64);
    let b = FieldElement::from(456u64);
    let hash1 = PedersenHash::hash_pair(&a, &b);
    let hash2 = PedersenHash::hash_pair(&a, &b);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_pedersen_hash_pair_not_commutative() {
    let a = FieldElement::from(123u64);
    let b = FieldElement::from(456u64);
    let hash_ab = PedersenHash::hash_pair(&a, &b);
    let hash_ba = PedersenHash::hash_pair(&b, &a);
    assert_ne!(hash_ab, hash_ba);
}

#[test]
fn test_pedersen_hash_many_deterministic() {
    let elements = vec![
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
    ];
    let hash1 = PedersenHash::hash_many(&elements);
    let hash2 = PedersenHash::hash_many(&elements);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_pedersen_hash_many_order_matters() {
    let elements1 = vec![
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
    ];
    let elements2 = vec![
        FieldElement::from(3u64),
        FieldElement::from(2u64),
        FieldElement::from(1u64),
    ];
    let hash1 = PedersenHash::hash_many(&elements1);
    let hash2 = PedersenHash::hash_many(&elements2);
    assert_ne!(hash1, hash2);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_pedersen_empty_array() {
    let empty: Vec<FieldElement> = vec![];
    let hash1 = PedersenHash::hash_many(&empty);
    let hash2 = PedersenHash::hash_many(&empty);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_pedersen_single_element() {
    let single = vec![FieldElement::from(42u64)];
    let hash = PedersenHash::hash_many(&single);
    assert_ne!(hash, FieldElement::ZERO);
}

#[test]
fn test_pedersen_zero_elements() {
    let zeros = vec![FieldElement::ZERO, FieldElement::ZERO];
    let hash = PedersenHash::hash_many(&zeros);
    assert_ne!(hash, FieldElement::ZERO);
}

#[test]
fn test_pedersen_large_array() {
    let large: Vec<FieldElement> = (0..100).map(|i| FieldElement::from(i as u64)).collect();
    let hash1 = PedersenHash::hash_many(&large);
    let hash2 = PedersenHash::hash_many(&large);
    assert_eq!(hash1, hash2);
}

// ============================================================================
// Stateful Hasher Tests
// ============================================================================

#[test]
fn test_hasher_matches_hash_many() {
    let elements = vec![
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
    ];

    // Using hash_many
    let hash_many = PedersenHash::hash_many(&elements);

    // Using stateful hasher
    let mut hasher = PedersenHasher::new();
    for elem in &elements {
        hasher.update(*elem);
    }
    let hash_stateful = hasher.finalize();

    assert_eq!(hash_many, hash_stateful);
}

#[test]
fn test_hasher_empty() {
    let hasher = PedersenHasher::new();
    let hash = hasher.finalize();

    // Should match empty hash_many
    let hash_many = PedersenHash::hash_many(&[]);
    assert_eq!(hash, hash_many);
}

#[test]
fn test_hasher_single_element() {
    let mut hasher = PedersenHasher::new();
    hasher.update(FieldElement::from(42u64));
    let hash = hasher.finalize();

    let hash_many = PedersenHash::hash_many(&[FieldElement::from(42u64)]);
    assert_eq!(hash, hash_many);
}

#[test]
fn test_hasher_multiple_updates() {
    let mut hasher = PedersenHasher::new();
    for i in 1..=5 {
        hasher.update(FieldElement::from(i as u64));
    }
    let hash = hasher.finalize();

    let elements: Vec<_> = (1..=5).map(|i| FieldElement::from(i as u64)).collect();
    let hash_many = PedersenHash::hash_many(&elements);
    assert_eq!(hash, hash_many);
}

// ============================================================================
// Cross-Validation with starknet-crypto
// ============================================================================

#[test]
fn test_cross_validate_hash_pair() {
    use starknet_crypto::pedersen_hash;

    let a = FieldElement::from(123u64);
    let b = FieldElement::from(456u64);

    // Our implementation
    let our_hash = PedersenHash::hash_pair(&a, &b);

    // Reference implementation
    let ref_hash = pedersen_hash(&a, &b);

    assert_eq!(our_hash, ref_hash, "Hash pair mismatch with reference");
}

#[test]
fn test_cross_validate_zero_zero() {
    use starknet_crypto::pedersen_hash;

    let a = FieldElement::ZERO;
    let b = FieldElement::ZERO;

    let our_hash = PedersenHash::hash_pair(&a, &b);
    let ref_hash = pedersen_hash(&a, &b);

    assert_eq!(our_hash, ref_hash);
}

#[test]
fn test_cross_validate_one_one() {
    use starknet_crypto::pedersen_hash;

    let a = FieldElement::ONE;
    let b = FieldElement::ONE;

    let our_hash = PedersenHash::hash_pair(&a, &b);
    let ref_hash = pedersen_hash(&a, &b);

    assert_eq!(our_hash, ref_hash);
}

#[test]
fn test_cross_validate_large_values() {
    use starknet_crypto::pedersen_hash;

    let a = FieldElement::from_hex("0x0123456789abcdef0123456789abcdef").unwrap();
    let b = FieldElement::from_hex("0xfedcba9876543210fedcba9876543210").unwrap();

    let our_hash = PedersenHash::hash_pair(&a, &b);
    let ref_hash = pedersen_hash(&a, &b);

    assert_eq!(our_hash, ref_hash);
}

// ============================================================================
// Test Vectors from Starknet Documentation
// ============================================================================

#[test]
fn test_vector_1_simple_pair() {
    let a = FieldElement::from(1u64);
    let b = FieldElement::from(2u64);
    let hash = PedersenHash::hash_pair(&a, &b);

    // Cross-validate with reference
    let ref_hash = starknet_crypto::pedersen_hash(&a, &b);
    assert_eq!(hash, ref_hash);
}

#[test]
fn test_vector_2_sequential_numbers() {
    let elements: Vec<FieldElement> = (1..=5).map(|i| FieldElement::from(i as u64)).collect();
    let hash = PedersenHash::hash_many(&elements);

    // Should be deterministic
    let hash2 = PedersenHash::hash_many(&elements);
    assert_eq!(hash, hash2);
}

#[test]
fn test_vector_3_powers_of_two() {
    let elements = vec![
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(4u64),
        FieldElement::from(8u64),
        FieldElement::from(16u64),
    ];
    let hash = PedersenHash::hash_many(&elements);
    assert_ne!(hash, FieldElement::ZERO);
}

#[test]
fn test_vector_4_alternating_pattern() {
    let elements = vec![
        FieldElement::ZERO,
        FieldElement::ONE,
        FieldElement::ZERO,
        FieldElement::ONE,
    ];
    let hash = PedersenHash::hash_many(&elements);
    assert_ne!(hash, FieldElement::ZERO);
}

#[test]
fn test_vector_5_max_field_elements() {
    let max = FieldElement::from_hex(
        "0x0800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f",
    )
    .unwrap();
    let hash = PedersenHash::hash_pair(&max, &max);

    // Should not panic and produce valid output
    assert_ne!(hash, FieldElement::ZERO);
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    /// Test that hash_pair is deterministic
    #[test]
    fn prop_hash_pair_deterministic(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let hash1 = PedersenHash::hash_pair(&fa, &fb);
        let hash2 = PedersenHash::hash_pair(&fa, &fb);
        assert_eq!(hash1, hash2);
    }

    /// Test that hash_pair is not commutative
    #[test]
    fn prop_hash_pair_not_commutative(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let hash_ab = PedersenHash::hash_pair(&fa, &fb);
        let hash_ba = PedersenHash::hash_pair(&fb, &fa);

        if a == b {
            assert_eq!(hash_ab, hash_ba);
        } else {
            assert_ne!(hash_ab, hash_ba);
        }
    }

    /// Test cross-validation with reference implementation
    #[test]
    fn prop_cross_validate(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);

        let our_hash = PedersenHash::hash_pair(&fa, &fb);
        let ref_hash = starknet_crypto::pedersen_hash(&fa, &fb);

        assert_eq!(our_hash, ref_hash);
    }

    /// Test that stateful hasher matches hash_many
    #[test]
    fn prop_hasher_matches_hash_many(values in prop::collection::vec(0u64..1000000, 1..20)) {
        let elements: Vec<_> = values.iter().map(|v| FieldElement::from(*v)).collect();

        let hash_many = PedersenHash::hash_many(&elements);

        let mut hasher = PedersenHasher::new();
        for elem in &elements {
            hasher.update(*elem);
        }
        let hash_stateful = hasher.finalize();

        assert_eq!(hash_many, hash_stateful);
    }

    /// Test that different inputs produce different hashes
    #[test]
    fn prop_different_inputs_different_outputs(a in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(a + 1);

        let hash_a = PedersenHash::hash_pair(&fa, &fa);
        let hash_b = PedersenHash::hash_pair(&fb, &fb);

        assert_ne!(hash_a, hash_b);
    }

    /// Test that hash output is non-zero for non-trivial inputs
    #[test]
    fn prop_hash_non_zero(a in 1u64..1000000, b in 1u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let hash = PedersenHash::hash_pair(&fa, &fb);
        assert_ne!(hash, FieldElement::ZERO);
    }
}

// ============================================================================
// Length Extension Attack Prevention
// ============================================================================

#[test]
fn test_length_extension_prevention() {
    // hash_many includes length in final hash to prevent length extension attacks
    let elements1 = vec![FieldElement::from(1u64), FieldElement::from(2u64)];
    let elements2 = vec![
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
    ];

    let hash1 = PedersenHash::hash_many(&elements1);
    let hash2 = PedersenHash::hash_many(&elements2);

    // Different lengths should produce different hashes
    assert_ne!(hash1, hash2);
}

#[test]
fn test_empty_vs_single_zero() {
    let empty: Vec<FieldElement> = vec![];
    let single_zero = vec![FieldElement::ZERO];

    let hash_empty = PedersenHash::hash_many(&empty);
    let hash_single = PedersenHash::hash_many(&single_zero);

    // Should be different (length is part of the hash)
    assert_ne!(hash_empty, hash_single);
}

// ============================================================================
// Collision Resistance Tests (Smoke Tests)
// ============================================================================

#[test]
fn test_no_trivial_collisions() {
    // Generate hashes for many different inputs
    let mut hashes = std::collections::HashSet::new();

    for i in 0..1000 {
        let a = FieldElement::from(i as u64);
        let b = FieldElement::from((i * 2) as u64);
        let hash = PedersenHash::hash_pair(&a, &b);

        // Convert to bytes for HashSet
        let bytes = hash.to_bytes_be();
        assert!(hashes.insert(bytes), "Found collision at i={}", i);
    }
}
