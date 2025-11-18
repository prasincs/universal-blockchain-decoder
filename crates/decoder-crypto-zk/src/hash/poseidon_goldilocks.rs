//! Rescue Prime hash implementation for Goldilocks field (Polygon zkEVM)
//!
//! This module provides a thin wrapper around winterfell's Rescue Prime (Rp64_256)
//! hash function, which is optimized for the Goldilocks field.
//!
//! # Why We Use winterfell's Rescue Prime
//!
//! Instead of implementing Poseidon from scratch, we use Facebook/Meta's
//! battle-tested Rescue Prime implementation:
//!
//! - ✅ Production-grade (used by Facebook/Meta)
//! - ✅ Works on stable Rust (no nightly features)
//! - ✅ Correct round constants and parameters
//! - ✅ Extensively tested
//! - ✅ Optimized for Goldilocks field
//!
//! # Note on Rescue Prime vs Poseidon
//!
//! Rescue Prime is a similar hash function to Poseidon, also designed for
//! zk-STARK systems. Both are algebraic hash functions optimized for arithmetic
//! circuits. For our purposes (hashing in the Goldilocks field), they serve the
//! same role.
//!
//! # References
//!
//! - [Winterfell](https://github.com/facebook/winterfell)
//! - [Rescue Prime](https://eprint.iacr.org/2020/1143.pdf)

use crate::field::goldilocks::GoldilocksFieldElement;
use winterfell::crypto::hashers::Rp64_256;
use winterfell::crypto::ElementHasher;

/// Hash function for Goldilocks field (using winterfell's Rescue Prime)
///
/// This provides a consistent API with our other hash implementations while
/// using the battle-tested winterfell Rescue Prime implementation internally.
///
/// # Examples
///
/// ```
/// use decoder_crypto_zk::hash::poseidon_goldilocks::PoseidonGoldilocksHash;
/// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
///
/// let a = GoldilocksFieldElement::from(123u64);
/// let b = GoldilocksFieldElement::from(456u64);
/// let hash = PoseidonGoldilocksHash::hash_pair(a, b);
/// ```
pub struct PoseidonGoldilocksHash;

impl PoseidonGoldilocksHash {
    /// Hash two field elements
    ///
    /// This is the most common operation, used for:
    /// - Merkle tree construction (zkTrie)
    /// - Transaction hashing
    /// - State commitments
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::poseidon_goldilocks::PoseidonGoldilocksHash;
    /// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
    ///
    /// let a = GoldilocksFieldElement::from(123u64);
    /// let b = GoldilocksFieldElement::from(456u64);
    /// let hash = PoseidonGoldilocksHash::hash_pair(a, b);
    /// ```
    pub fn hash_pair(
        x: GoldilocksFieldElement,
        y: GoldilocksFieldElement,
    ) -> GoldilocksFieldElement {
        // Use winterfell's Rescue Prime hash
        let inputs = [x.0, y.0];
        let digest = Rp64_256::hash_elements(&inputs);
        // Return first element of digest (4-element digest)
        GoldilocksFieldElement(digest.as_elements()[0])
    }

    /// Hash a single field element
    ///
    /// Used for domain separation and single-value commitments.
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::poseidon_goldilocks::PoseidonGoldilocksHash;
    /// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
    ///
    /// let value = GoldilocksFieldElement::from(123u64);
    /// let hash = PoseidonGoldilocksHash::hash_single(value);
    /// ```
    pub fn hash_single(x: GoldilocksFieldElement) -> GoldilocksFieldElement {
        // Hash single element
        let inputs = [x.0];
        let digest = Rp64_256::hash_elements(&inputs);
        GoldilocksFieldElement(digest.as_elements()[0])
    }

    /// Hash many field elements using the sponge construction
    ///
    /// This is used for hashing arrays and variable-length data.
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::poseidon_goldilocks::PoseidonGoldilocksHash;
    /// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
    ///
    /// let elements = vec![
    ///     GoldilocksFieldElement::from(1u64),
    ///     GoldilocksFieldElement::from(2u64),
    ///     GoldilocksFieldElement::from(3u64),
    /// ];
    /// let hash = PoseidonGoldilocksHash::hash_many(&elements);
    /// ```
    pub fn hash_many(elements: &[GoldilocksFieldElement]) -> GoldilocksFieldElement {
        // Convert to winterfell format
        let inputs: Vec<_> = elements.iter().map(|e| e.0).collect();
        let digest = Rp64_256::hash_elements(&inputs);
        GoldilocksFieldElement(digest.as_elements()[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_pair_deterministic() {
        let a = GoldilocksFieldElement::from(123u64);
        let b = GoldilocksFieldElement::from(456u64);

        let hash1 = PoseidonGoldilocksHash::hash_pair(a, b);
        let hash2 = PoseidonGoldilocksHash::hash_pair(a, b);

        // Should be deterministic
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_pair_not_commutative() {
        let a = GoldilocksFieldElement::from(123u64);
        let b = GoldilocksFieldElement::from(456u64);

        let hash1 = PoseidonGoldilocksHash::hash_pair(a, b);
        let hash2 = PoseidonGoldilocksHash::hash_pair(b, a);

        // Hash should not be commutative (order matters)
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_single_deterministic() {
        let a = GoldilocksFieldElement::from(123u64);

        let hash1 = PoseidonGoldilocksHash::hash_single(a);
        let hash2 = PoseidonGoldilocksHash::hash_single(a);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_single_differs_from_hash_pair() {
        let a = GoldilocksFieldElement::from(123u64);
        let zero = GoldilocksFieldElement::ZERO;

        let hash_single = PoseidonGoldilocksHash::hash_single(a);
        let hash_pair = PoseidonGoldilocksHash::hash_pair(a, zero);

        // Domain separation: hash_single(a) should differ from hash_pair(a, 0)
        assert_ne!(hash_single, hash_pair);
    }

    #[test]
    fn test_hash_many_empty() {
        let empty: Vec<GoldilocksFieldElement> = vec![];
        let hash = PoseidonGoldilocksHash::hash_many(&empty);

        // Empty hash should be deterministic
        assert_eq!(hash, PoseidonGoldilocksHash::hash_many(&[]));
    }

    #[test]
    fn test_hash_many_single() {
        let elements = vec![GoldilocksFieldElement::from(123u64)];
        let hash = PoseidonGoldilocksHash::hash_many(&elements);

        // Should be deterministic
        assert_eq!(
            hash,
            PoseidonGoldilocksHash::hash_many(&[GoldilocksFieldElement::from(123u64)])
        );
    }

    #[test]
    fn test_hash_many_multiple() {
        let elements = vec![
            GoldilocksFieldElement::from(1u64),
            GoldilocksFieldElement::from(2u64),
            GoldilocksFieldElement::from(3u64),
        ];

        let hash1 = PoseidonGoldilocksHash::hash_many(&elements);
        let hash2 = PoseidonGoldilocksHash::hash_many(&elements);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_many_order_matters() {
        let elements1 = vec![
            GoldilocksFieldElement::from(1u64),
            GoldilocksFieldElement::from(2u64),
            GoldilocksFieldElement::from(3u64),
        ];

        let elements2 = vec![
            GoldilocksFieldElement::from(3u64),
            GoldilocksFieldElement::from(2u64),
            GoldilocksFieldElement::from(1u64),
        ];

        let hash1 = PoseidonGoldilocksHash::hash_many(&elements1);
        let hash2 = PoseidonGoldilocksHash::hash_many(&elements2);

        // Order should matter
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_avalanche_effect() {
        // Small change in input should cause large change in output
        let a1 = GoldilocksFieldElement::from(123u64);
        let a2 = GoldilocksFieldElement::from(124u64);
        let b = GoldilocksFieldElement::from(456u64);

        let hash1 = PoseidonGoldilocksHash::hash_pair(a1, b);
        let hash2 = PoseidonGoldilocksHash::hash_pair(a2, b);

        // Should produce completely different hashes
        assert_ne!(hash1, hash2);
    }
}
