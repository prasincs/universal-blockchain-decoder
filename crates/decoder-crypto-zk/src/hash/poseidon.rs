//! Poseidon hash implementation for Starknet
//!
//! Poseidon is a cryptographic hash function designed for efficient use in zero-knowledge proof systems.
//! It uses the Hades permutation strategy with a sponge construction.
//!
//! This implementation is specifically for the Starknet variant of Poseidon.
//!
//! # Chains Using This Implementation
//!
//! - Starknet (230+ chains): Mainnet, testnet, and Madara/SN Stack appchains
//! - Kakarot zkEVM
//! - And other Starknet ecosystem chains
//!
//! # References
//!
//! - [Poseidon Paper](https://eprint.iacr.org/2019/458.pdf)
//! - [Starknet Poseidon Spec](https://docs.starknet.io/documentation/architecture_and_concepts/Cryptography/hash-functions/)

use crate::field::FieldElement;
use starknet_types_core::hash::Poseidon;

/// Poseidon hash function for Starknet
///
/// This is a zero-copy, stateless wrapper around the vendored implementation.
pub struct PoseidonHash;

impl PoseidonHash {
    /// Hash two field elements
    ///
    /// This is the most common operation in Starknet, used for:
    /// - Merkle tree construction
    /// - Transaction hashing
    /// - State commitments
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::PoseidonHash;
    /// use decoder_crypto_zk::field::FieldElement;
    ///
    /// let a = FieldElement::from(123u64);
    /// let b = FieldElement::from(456u64);
    /// let hash = PoseidonHash::hash_pair(a, b);
    /// ```
    pub fn hash_pair(x: FieldElement, y: FieldElement) -> FieldElement {
        let mut state = [x, y, FieldElement::TWO];
        Poseidon::hades_permutation(&mut state);
        state[0]
    }

    /// Hash a single field element
    ///
    /// Used for domain separation and single-value commitments.
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::PoseidonHash;
    /// use decoder_crypto_zk::field::FieldElement;
    ///
    /// let value = FieldElement::from(123u64);
    /// let hash = PoseidonHash::hash_single(value);
    /// ```
    pub fn hash_single(x: FieldElement) -> FieldElement {
        let mut state = [x, FieldElement::ZERO, FieldElement::ONE];
        Poseidon::hades_permutation(&mut state);
        state[0]
    }

    /// Hash many field elements
    ///
    /// Uses the sponge construction to hash an arbitrary number of elements.
    /// This is used for hashing arrays and variable-length data.
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::PoseidonHash;
    /// use decoder_crypto_zk::field::FieldElement;
    ///
    /// let elements = vec![
    ///     FieldElement::from(1u64),
    ///     FieldElement::from(2u64),
    ///     FieldElement::from(3u64),
    /// ];
    /// let hash = PoseidonHash::hash_many(&elements);
    /// ```
    pub fn hash_many(elements: &[FieldElement]) -> FieldElement {
        let mut state = [FieldElement::ZERO, FieldElement::ZERO, FieldElement::ZERO];
        let mut iter = elements.iter();

        loop {
            match iter.next() {
                Some(v) => state[0] += *v,
                None => {
                    state[0] += FieldElement::ONE;
                    break;
                }
            }

            match iter.next() {
                Some(v) => state[1] += *v,
                None => {
                    state[1] += FieldElement::ONE;
                    break;
                }
            }

            Poseidon::hades_permutation(&mut state);
        }
        Poseidon::hades_permutation(&mut state);

        state[0]
    }

    /// Poseidon permutation (advanced use)
    ///
    /// Direct access to the Hades permutation for advanced cryptographic protocols.
    /// Most users should use `hash_pair`, `hash_single`, or `hash_many` instead.
    pub fn permute(state: &mut [FieldElement; 3]) {
        Poseidon::hades_permutation(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_hash_pair() {
        let a = FieldElement::from(123u64);
        let b = FieldElement::from(456u64);
        let hash1 = PoseidonHash::hash_pair(a, b);
        let hash2 = PoseidonHash::hash_pair(a, b);

        // Deterministic
        assert_eq!(hash1, hash2);

        // Not commutative
        let hash3 = PoseidonHash::hash_pair(b, a);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_poseidon_hash_single() {
        let a = FieldElement::from(123u64);
        let hash1 = PoseidonHash::hash_single(a);
        let hash2 = PoseidonHash::hash_single(a);

        // Deterministic
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_hash_many() {
        let elements = vec![
            FieldElement::from(1u64),
            FieldElement::from(2u64),
            FieldElement::from(3u64),
        ];
        let hash1 = PoseidonHash::hash_many(&elements);
        let hash2 = PoseidonHash::hash_many(&elements);

        // Deterministic
        assert_eq!(hash1, hash2);

        // Different for different inputs
        let elements2 = vec![
            FieldElement::from(3u64),
            FieldElement::from(2u64),
            FieldElement::from(1u64),
        ];
        let hash3 = PoseidonHash::hash_many(&elements2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_poseidon_empty_hash() {
        let empty: Vec<FieldElement> = vec![];
        let hash = PoseidonHash::hash_many(&empty);

        // Empty hash should be deterministic
        assert_eq!(hash, PoseidonHash::hash_many(&[]));
    }
}
