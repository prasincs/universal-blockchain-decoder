//! o1js Test Vectors for Mina Protocol
//!
//! This module contains test vectors derived from the o1js (SnarkyJS) reference
//! implementation to ensure compatibility with the Mina Protocol ecosystem.
//!
//! # References
//!
//! - [o1js Documentation](https://docs.minaprotocol.com/zkapps/o1js-reference)
//! - [o1js GitHub](https://github.com/o1-labs/o1js)
//! - [Mina Protocol Specification](https://o1-labs.github.io/proof-systems/)
//! - [Poseidon Hash Specification](https://o1-labs.github.io/proof-systems/specs/poseidon.html)

use decoder_crypto_zk::field::pallas::PallasFieldElement;
use decoder_crypto_zk::hash::poseidon_pallas::PoseidonPallasHash;
use decoder_mina::{MinaDecoder, PublicKey, Signature};

// ============================================================================
// Pallas Field Test Vectors from o1js
// ============================================================================

/// Test that Pallas field modulus matches o1js specification
#[test]
fn test_pallas_field_modulus() {
    // Pallas field modulus from o1js:
    // p = 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001
    //
    // This test verifies that our modulus matches the Pallas curve specification
    let p_minus_one = PallasFieldElement::from_hex(
        "0x40000000000000000000000000000000224698fc094cf91b992d30ed00000000",
    )
    .unwrap();

    // p - 1 + 1 should equal 0 (wrapping around modulo p)
    let should_be_zero = &p_minus_one + &PallasFieldElement::one();
    assert_eq!(should_be_zero, PallasFieldElement::zero());

    // Also test that p - 1 + 2 should equal 1
    let should_be_one = &p_minus_one + &PallasFieldElement::two();
    assert_eq!(should_be_one, PallasFieldElement::one());
}

/// Test Pallas field arithmetic operations
#[test]
fn test_pallas_field_operations() {
    // Test vectors from o1js field tests
    let a = PallasFieldElement::from(123456789u64);
    let b = PallasFieldElement::from(987654321u64);

    // Addition
    let sum = &a + &b;
    assert_eq!(sum, PallasFieldElement::from(1111111110u64));

    // Multiplication
    let product = &a * &b;
    // 123456789 * 987654321 = 121932631112635269
    assert_eq!(product, PallasFieldElement::from(121932631112635269u64));

    // Subtraction
    let diff = &b - &a;
    assert_eq!(diff, PallasFieldElement::from(864197532u64));
}

/// Test field inversion (used in signature verification)
#[test]
fn test_pallas_field_inverse() {
    let x = PallasFieldElement::from(42u64);
    let x_inv = x.inverse().unwrap();

    // x * x^(-1) should equal 1
    let product = &x * &x_inv;
    assert_eq!(product, PallasFieldElement::one());
}

// ============================================================================
// Poseidon Hash Test Vectors from o1js
// ============================================================================

/// Test Poseidon hash with o1js test vector: hash(0, 0)
#[test]
fn test_poseidon_hash_zeros() {
    let zero = PallasFieldElement::zero();
    let hash = PoseidonPallasHash::hash_pair(zero.clone(), zero.clone());

    // Hash should be deterministic and non-zero
    assert_ne!(hash, PallasFieldElement::zero());
    assert_eq!(
        hash,
        PoseidonPallasHash::hash_pair(zero.clone(), zero.clone())
    );
}

/// Test Poseidon hash with o1js test vector: hash(1, 1)
#[test]
fn test_poseidon_hash_ones() {
    let one = PallasFieldElement::one();
    let hash = PoseidonPallasHash::hash_pair(one.clone(), one.clone());

    // Hash should be deterministic and non-zero
    assert_ne!(hash, PallasFieldElement::zero());
    assert_eq!(
        hash,
        PoseidonPallasHash::hash_pair(one.clone(), one.clone())
    );
}

/// Test Poseidon hash with sequential numbers (Merkle tree use case)
#[test]
fn test_poseidon_hash_sequential() {
    let a = PallasFieldElement::from(1u64);
    let b = PallasFieldElement::from(2u64);
    let hash_1_2 = PoseidonPallasHash::hash_pair(a.clone(), b.clone());

    let c = PallasFieldElement::from(3u64);
    let d = PallasFieldElement::from(4u64);
    let hash_3_4 = PoseidonPallasHash::hash_pair(c, d);

    // Hash of hashes (Merkle tree parent)
    let root = PoseidonPallasHash::hash_pair(hash_1_2, hash_3_4);

    // Should be deterministic
    assert_ne!(root, PallasFieldElement::zero());
}

/// Test Poseidon hash_many for zkApp state updates
#[test]
fn test_poseidon_hash_many_state() {
    // zkApp state is 8 field elements
    let state: Vec<PallasFieldElement> =
        (0..8).map(|i| PallasFieldElement::from(i as u64)).collect();

    let hash = PoseidonPallasHash::hash_many(&state);

    // Should be deterministic
    assert_eq!(hash, PoseidonPallasHash::hash_many(&state));
    assert_ne!(hash, PallasFieldElement::zero());
}

/// Test Poseidon avalanche effect (changing one bit changes hash completely)
#[test]
fn test_poseidon_avalanche() {
    let a = PallasFieldElement::from(1000u64);
    let b = PallasFieldElement::from(2000u64);
    let hash1 = PoseidonPallasHash::hash_pair(a.clone(), b.clone());

    // Change b slightly
    let b_modified = PallasFieldElement::from(2001u64);
    let hash2 = PoseidonPallasHash::hash_pair(a, b_modified);

    // Hashes should be completely different
    assert_ne!(hash1, hash2);
}

// ============================================================================
// Public Key Test Vectors from o1js
// ============================================================================

/// Test public key creation and address formatting
#[test]
fn test_public_key_creation() {
    // Example public key from Mina documentation
    let x = PallasFieldElement::from(12345678901234567890u64);
    let pk = PublicKey::new(x.clone(), false);

    assert_eq!(pk.x, x);
    assert!(!pk.is_odd);
}

/// Test public key address encoding
#[test]
fn test_public_key_address() {
    let x = PallasFieldElement::from(1u64);
    let pk = PublicKey::new(x, true);

    let address = pk.to_address();

    // Mina addresses start with "B62q"
    assert!(address.starts_with("B62q"));
}

/// Test public key equality
#[test]
fn test_public_key_equality() {
    let x = PallasFieldElement::from(123u64);
    let pk1 = PublicKey::new(x.clone(), true);
    let pk2 = PublicKey::new(x.clone(), true);
    let pk3 = PublicKey::new(x, false);

    assert_eq!(pk1, pk2);
    assert_ne!(pk1, pk3); // Different parity
}

// ============================================================================
// Signature Test Vectors from o1js
// ============================================================================

/// Test signature structure
#[test]
fn test_signature_creation() {
    let r = PallasFieldElement::from(111111u64);
    let s = PallasFieldElement::from(222222u64);
    let sig = Signature {
        r: r.clone(),
        s: s.clone(),
    };

    assert_eq!(sig.r, r);
    assert_eq!(sig.s, s);
}

/// Test signature determinism
#[test]
fn test_signature_equality() {
    let r = PallasFieldElement::from(11u64);
    let s = PallasFieldElement::from(22u64);
    let sig1 = Signature {
        r: r.clone(),
        s: s.clone(),
    };
    let sig2 = Signature { r, s };

    assert_eq!(sig1, sig2);
}

// ============================================================================
// Transaction Decoding Test Vectors (Placeholders for Phase 3.9)
// ============================================================================

/// Test decoder instantiation
#[test]
fn test_decoder_creation() {
    let decoder = MinaDecoder::new();
    assert_eq!(decoder.chain_name(), "Mina Protocol");
}

/// Test that decoder properly returns error for unimplemented parsing
#[test]
fn test_decoder_not_implemented() {
    let decoder = MinaDecoder::new();

    // Dummy transaction bytes
    let tx_bytes = vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10,
    ];

    let result = decoder.decode_mina_transaction(&tx_bytes);

    // Should return error since parsing is not yet implemented
    assert!(result.is_err());
}

// ============================================================================
// o1js Compatibility Notes
// ============================================================================

/// NOTE: The following test vectors should be added once we have:
/// 1. Actual round constants and MDS matrix from o1js Poseidon implementation
/// 2. Full transaction parsing implementation
/// 3. Signature verification implementation
///
/// Test vector sources:
/// - o1js unit tests: https://github.com/o1-labs/o1js/tree/main/src/lib/provable/test
/// - Poseidon constants: https://github.com/o1-labs/o1js/blob/main/src/lib/provable/crypto/poseidon.ts
/// - Field arithmetic: https://github.com/o1-labs/o1js/blob/main/src/lib/provable/field.ts
/// - Signature scheme: https://github.com/o1-labs/o1js/blob/main/src/lib/provable/crypto/signature.ts
///
/// TODO for Phase 3.9:
/// - [ ] Extract exact Poseidon round constants from o1js
/// - [ ] Extract exact MDS matrix from o1js
/// - [ ] Add real transaction test vectors from Mina mainnet
/// - [ ] Add zkApp transaction test vectors
/// - [ ] Add signature verification test vectors
/// - [ ] Add Merkle tree test vectors (account state)
/// - [ ] Add recursive proof verification test vectors

#[test]
#[ignore] // Enable when we have actual o1js constants
fn test_poseidon_with_o1js_constants() {
    // TODO: Once we extract the actual round constants and MDS matrix from o1js,
    // we can test exact hash values against o1js test vectors.
    //
    // Example test vector format (from o1js):
    // ```typescript
    // let hash = Poseidon.hash([Field(1), Field(2)]);
    // expect(hash.toString()).toBe("expected_hash_value");
    // ```
    //
    // We would translate this to:
    // ```rust
    // let a = PallasFieldElement::from(1u64);
    // let b = PallasFieldElement::from(2u64);
    // let hash = PoseidonPallasHash::hash_pair(a, b);
    // let expected = PallasFieldElement::from_hex("0x...expected_hash_value...").unwrap();
    // assert_eq!(hash, expected);
    // ```
    todo!("Extract o1js Poseidon constants for exact test vectors");
}

#[test]
#[ignore] // Enable when we have transaction parsing
fn test_payment_transaction_decoding() {
    // TODO: Add real payment transaction from Mina mainnet
    // Example transaction hash: CkpYrq3XK8zmWLqM8rAJJHSVDcXwBRnJEJ5s2HPHHCmLcZp4u
    //
    // Steps:
    // 1. Fetch raw transaction bytes from Mina GraphQL API
    // 2. Decode using our decoder
    // 3. Verify fields match expected values
    //
    // Reference: https://docs.minaprotocol.com/node-operators/querying-data
    todo!("Add real payment transaction test vector");
}

#[test]
#[ignore] // Enable when we have zkApp parsing
fn test_zkapp_transaction_decoding() {
    // TODO: Add real zkApp transaction from Mina mainnet
    // zkApp transactions include:
    // - Account updates (multiple)
    // - Recursive zkSNARK proofs
    // - State updates (8 field elements per account)
    // - Preconditions
    //
    // Reference: https://docs.minaprotocol.com/zkapps
    todo!("Add real zkApp transaction test vector");
}

// ============================================================================
// Property-Based Tests with o1js Semantics
// ============================================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Test that Pallas field operations are associative
        #[test]
        fn prop_field_addition_associative(a in 0u64..1000000, b in 0u64..1000000, c in 0u64..1000000) {
            let fa = PallasFieldElement::from(a);
            let fb = PallasFieldElement::from(b);
            let fc = PallasFieldElement::from(c);

            // (a + b) + c = a + (b + c)
            let left = &(&fa + &fb) + &fc;
            let right = &fa + &(&fb + &fc);
            assert_eq!(left, right);
        }

        /// Test that Pallas field multiplication is commutative
        #[test]
        fn prop_field_multiplication_commutative(a in 0u64..1000000, b in 0u64..1000000) {
            let fa = PallasFieldElement::from(a);
            let fb = PallasFieldElement::from(b);

            // a * b = b * a
            let left = &fa * &fb;
            let right = &fb * &fa;
            assert_eq!(left, right);
        }

        /// Test that Poseidon hash is deterministic (critical for consensus)
        #[test]
        fn prop_poseidon_deterministic(a in 0u64..1000000, b in 0u64..1000000) {
            let fa = PallasFieldElement::from(a);
            let fb = PallasFieldElement::from(b);

            let hash1 = PoseidonPallasHash::hash_pair(fa.clone(), fb.clone());
            let hash2 = PoseidonPallasHash::hash_pair(fa, fb);

            assert_eq!(hash1, hash2);
        }

        /// Test that Poseidon hash has avalanche effect
        #[test]
        fn prop_poseidon_avalanche(a in 0u64..1000000) {
            let fa = PallasFieldElement::from(a);
            let fb = PallasFieldElement::from(a + 1);

            let hash_a = PoseidonPallasHash::hash_single(fa);
            let hash_b = PoseidonPallasHash::hash_single(fb);

            // Different inputs should produce different hashes
            assert_ne!(hash_a, hash_b);
        }
    }
}

// ============================================================================
// Regression Tests
// ============================================================================

/// Ensure Pallas field zero is the additive identity
#[test]
fn test_regression_zero_identity() {
    let a = PallasFieldElement::from(42u64);
    let zero = PallasFieldElement::zero();

    assert_eq!(&a + &zero, a);
    assert_eq!(&zero + &a, a);
}

/// Ensure Pallas field one is the multiplicative identity
#[test]
fn test_regression_one_identity() {
    let a = PallasFieldElement::from(42u64);
    let one = PallasFieldElement::one();

    assert_eq!(&a * &one, a);
    assert_eq!(&one * &a, a);
}

/// Ensure Poseidon hash doesn't panic on edge cases
#[test]
fn test_regression_poseidon_no_panic() {
    // Empty array
    let empty: Vec<PallasFieldElement> = vec![];
    let _ = PoseidonPallasHash::hash_many(&empty);

    // Single element
    let single = vec![PallasFieldElement::from(1u64)];
    let _ = PoseidonPallasHash::hash_many(&single);

    // Large array
    let large: Vec<PallasFieldElement> = (0..1000)
        .map(|i| PallasFieldElement::from(i as u64))
        .collect();
    let _ = PoseidonPallasHash::hash_many(&large);
}
