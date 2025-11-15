//! Poseidon hash tests
//!
//! Tests for Poseidon hash including:
//! - Test vectors from Starknet
//! - Cross-validation with starknet-crypto
//! - Property tests (determinism, avalanche effect)
//! - Edge cases (empty, single element, many elements)

use decoder_crypto_zk::field::FieldElement;
use decoder_crypto_zk::hash::PoseidonHash;
use proptest::prelude::*;

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_poseidon_hash_pair_deterministic() {
    let a = FieldElement::from(123u64);
    let b = FieldElement::from(456u64);
    let hash1 = PoseidonHash::hash_pair(a, b);
    let hash2 = PoseidonHash::hash_pair(a, b);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_poseidon_hash_pair_not_commutative() {
    let a = FieldElement::from(123u64);
    let b = FieldElement::from(456u64);
    let hash_ab = PoseidonHash::hash_pair(a, b);
    let hash_ba = PoseidonHash::hash_pair(b, a);
    assert_ne!(hash_ab, hash_ba);
}

#[test]
fn test_poseidon_hash_single_deterministic() {
    let a = FieldElement::from(123u64);
    let hash1 = PoseidonHash::hash_single(a);
    let hash2 = PoseidonHash::hash_single(a);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_poseidon_hash_single_different_inputs() {
    let hash1 = PoseidonHash::hash_single(FieldElement::from(1u64));
    let hash2 = PoseidonHash::hash_single(FieldElement::from(2u64));
    assert_ne!(hash1, hash2);
}

#[test]
fn test_poseidon_hash_many_deterministic() {
    let elements = vec![
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
    ];
    let hash1 = PoseidonHash::hash_many(&elements);
    let hash2 = PoseidonHash::hash_many(&elements);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_poseidon_hash_many_order_matters() {
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
    let hash1 = PoseidonHash::hash_many(&elements1);
    let hash2 = PoseidonHash::hash_many(&elements2);
    assert_ne!(hash1, hash2);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_poseidon_empty_array() {
    let empty: Vec<FieldElement> = vec![];
    let hash1 = PoseidonHash::hash_many(&empty);
    let hash2 = PoseidonHash::hash_many(&empty);
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, FieldElement::ZERO);
}

#[test]
fn test_poseidon_single_element_array() {
    let single = vec![FieldElement::from(42u64)];
    let hash = PoseidonHash::hash_many(&single);
    assert_ne!(hash, FieldElement::ZERO);
}

#[test]
fn test_poseidon_zero_elements() {
    let zeros = vec![FieldElement::ZERO, FieldElement::ZERO];
    let hash = PoseidonHash::hash_many(&zeros);
    assert_ne!(hash, FieldElement::ZERO);
}

#[test]
fn test_poseidon_large_array() {
    let large: Vec<FieldElement> = (0..100).map(|i| FieldElement::from(i as u64)).collect();
    let hash1 = PoseidonHash::hash_many(&large);
    let hash2 = PoseidonHash::hash_many(&large);
    assert_eq!(hash1, hash2);
}

// ============================================================================
// Cross-Validation with starknet-crypto
// ============================================================================

#[test]
fn test_cross_validate_hash_pair() {
    use starknet_crypto::poseidon_hash;

    let a = FieldElement::from(123u64);
    let b = FieldElement::from(456u64);

    // Our implementation
    let our_hash = PoseidonHash::hash_pair(a, b);

    // Reference implementation
    let ref_hash = poseidon_hash(a, b);

    assert_eq!(our_hash, ref_hash, "Hash pair mismatch with reference");
}

#[test]
fn test_cross_validate_hash_single() {
    use starknet_crypto::poseidon_hash_single;

    let a = FieldElement::from(789u64);

    // Our implementation
    let our_hash = PoseidonHash::hash_single(a);

    // Reference implementation
    let ref_hash = poseidon_hash_single(a);

    assert_eq!(our_hash, ref_hash, "Hash single mismatch with reference");
}

#[test]
fn test_cross_validate_hash_many() {
    use starknet_crypto::poseidon_hash_many;

    let elements = vec![
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
        FieldElement::from(4u64),
        FieldElement::from(5u64),
    ];

    // Our implementation
    let our_hash = PoseidonHash::hash_many(&elements);

    // Reference implementation
    let ref_hash = poseidon_hash_many(&elements);

    assert_eq!(our_hash, ref_hash, "Hash many mismatch with reference");
}

// ============================================================================
// Test Vectors from Starknet Documentation
// ============================================================================

#[test]
fn test_vector_1_simple_pair() {
    // From Starknet documentation
    let a = FieldElement::from(1u64);
    let b = FieldElement::from(2u64);
    let hash = PoseidonHash::hash_pair(a, b);

    // Cross-validate with reference
    let ref_hash = starknet_crypto::poseidon_hash(a, b);
    assert_eq!(hash, ref_hash);
}

#[test]
fn test_vector_2_zero_pair() {
    let a = FieldElement::ZERO;
    let b = FieldElement::ZERO;
    let hash = PoseidonHash::hash_pair(a, b);

    // Should be deterministic and non-zero
    assert_ne!(hash, FieldElement::ZERO);
    assert_eq!(hash, PoseidonHash::hash_pair(a, b));
}

#[test]
fn test_vector_3_one_pair() {
    let a = FieldElement::ONE;
    let b = FieldElement::ONE;
    let hash = PoseidonHash::hash_pair(a, b);

    // Cross-validate
    let ref_hash = starknet_crypto::poseidon_hash(a, b);
    assert_eq!(hash, ref_hash);
}

#[test]
fn test_vector_4_max_values() {
    // Large field elements
    let a = FieldElement::from_hex("0x0123456789abcdef0123456789abcdef").unwrap();
    let b = FieldElement::from_hex("0xfedcba9876543210fedcba9876543210").unwrap();
    let hash = PoseidonHash::hash_pair(a, b);

    // Cross-validate
    let ref_hash = starknet_crypto::poseidon_hash(a, b);
    assert_eq!(hash, ref_hash);
}

#[test]
fn test_vector_5_sequential_numbers() {
    let elements: Vec<FieldElement> = (1..=10).map(|i| FieldElement::from(i as u64)).collect();
    let hash = PoseidonHash::hash_many(&elements);

    // Cross-validate
    let ref_hash = starknet_crypto::poseidon_hash_many(&elements);
    assert_eq!(hash, ref_hash);
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
        let hash1 = PoseidonHash::hash_pair(fa, fb);
        let hash2 = PoseidonHash::hash_pair(fa, fb);
        assert_eq!(hash1, hash2);
    }

    /// Test that hash_pair is not commutative (unless inputs are equal)
    #[test]
    fn prop_hash_pair_not_commutative(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let hash_ab = PoseidonHash::hash_pair(fa, fb);
        let hash_ba = PoseidonHash::hash_pair(fb, fa);

        if a == b {
            assert_eq!(hash_ab, hash_ba);
        } else {
            // Usually not equal (collision is extremely rare)
            assert_ne!(hash_ab, hash_ba);
        }
    }

    /// Test that hash_single is deterministic
    #[test]
    fn prop_hash_single_deterministic(a in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let hash1 = PoseidonHash::hash_single(fa);
        let hash2 = PoseidonHash::hash_single(fa);
        assert_eq!(hash1, hash2);
    }

    /// Test that different inputs produce different hashes (avalanche effect)
    #[test]
    fn prop_avalanche_effect(a in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(a + 1);
        let hash_a = PoseidonHash::hash_single(fa);
        let hash_b = PoseidonHash::hash_single(fb);

        // Different inputs should produce different hashes
        assert_ne!(hash_a, hash_b);
    }

    /// Test cross-validation with reference implementation
    #[test]
    fn prop_cross_validate_pair(a in 0u64..1000000, b in 0u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);

        let our_hash = PoseidonHash::hash_pair(fa, fb);
        let ref_hash = starknet_crypto::poseidon_hash(fa, fb);

        assert_eq!(our_hash, ref_hash);
    }

    /// Test that hash output is non-zero for non-trivial inputs
    #[test]
    fn prop_hash_non_zero(a in 1u64..1000000, b in 1u64..1000000) {
        let fa = FieldElement::from(a);
        let fb = FieldElement::from(b);
        let hash = PoseidonHash::hash_pair(fa, fb);
        assert_ne!(hash, FieldElement::ZERO);
    }
}

// ============================================================================
// Performance/Benchmark Tests
// ============================================================================

#[test]
fn test_hash_many_performance() {
    // Test that hash_many can handle large arrays
    let large_array: Vec<FieldElement> = (0..1000).map(|i| FieldElement::from(i as u64)).collect();

    let start = std::time::Instant::now();
    let _ = PoseidonHash::hash_many(&large_array);
    let elapsed = start.elapsed();

    // Should complete reasonably fast (< 1 second for 1000 elements)
    assert!(
        elapsed.as_secs() < 1,
        "Hash many took too long: {:?}",
        elapsed
    );
}

// ============================================================================
// Permutation Tests
// ============================================================================

#[test]
fn test_permutation_modifies_state() {
    let mut state = [
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
    ];
    let original = state;

    PoseidonHash::permute(&mut state);

    // State should be different after permutation
    assert_ne!(state[0], original[0]);
    assert_ne!(state[1], original[1]);
    assert_ne!(state[2], original[2]);
}

#[test]
fn test_permutation_deterministic() {
    let mut state1 = [
        FieldElement::from(1u64),
        FieldElement::from(2u64),
        FieldElement::from(3u64),
    ];
    let mut state2 = state1;

    PoseidonHash::permute(&mut state1);
    PoseidonHash::permute(&mut state2);

    assert_eq!(state1, state2);
}
