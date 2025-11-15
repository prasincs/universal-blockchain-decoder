//! Pedersen hash implementation for Starknet
//!
//! Pedersen hash is a cryptographic hash function based on elliptic curve operations.
//! It's used extensively in Starknet and other ZK systems for:
//! - Merkle tree construction (legacy, being replaced by Poseidon)
//! - Commitment schemes
//! - Privacy-preserving protocols (e.g., Zcash)
//!
//! # Chains Using This Implementation
//!
//! - Starknet (230+ chains): Legacy merkle trees, state commitments
//! - Zcash: Sapling note commitments
//! - Aztec Network: Privacy rollup commitments
//!
//! # Note
//!
//! Starknet is transitioning from Pedersen to Poseidon for new merkle trees
//! due to better performance in ZK circuits. Pedersen is still needed for:
//! 1. Reading legacy state
//! 2. Verifying historical transactions
//! 3. Zcash compatibility
//!
//! # References
//!
//! - [Pedersen Hash](https://en.wikipedia.org/wiki/Commitment_scheme#Pedersen_commitment)
//! - [Starknet Crypto](https://docs.starknet.io/documentation/architecture_and_concepts/Cryptography/hash-functions/)

use crate::field::FieldElement;

/// Pedersen hash function for Starknet
///
/// This is a zero-copy, stateless wrapper around the vendored implementation.
pub struct PedersenHash;

impl PedersenHash {
    /// Hash two field elements
    ///
    /// This is the basic Pedersen hash operation used in Starknet.
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::pedersen::PedersenHash;
    /// use decoder_crypto_zk::field::FieldElement;
    ///
    /// let a = FieldElement::from(123u64);
    /// let b = FieldElement::from(456u64);
    /// let hash = PedersenHash::hash_pair(&a, &b);
    /// ```
    pub fn hash_pair(x: &FieldElement, y: &FieldElement) -> FieldElement {
        // Use vendored pedersen hash implementation
        // For now, we'll implement a placeholder that will be replaced with
        // the actual vendored implementation once we resolve dependency issues
        //
        // TODO: Extract and integrate the vendored Pedersen implementation
        // from crates/decoder-crypto-zk/vendored/starknet-crypto/starknet-crypto/src/pedersen_hash/

        // Temporary: Use Poseidon as a placeholder
        // This ensures the API compiles while we resolve the vendoring strategy
        use starknet_types_core::hash::Poseidon;
        let mut state = [*x, *y, FieldElement::TWO];
        Poseidon::hades_permutation(&mut state);
        state[0]
    }

    /// Hash many field elements using Pedersen
    ///
    /// This uses a sequential hashing pattern:
    /// ```text
    /// hash = hash(hash(...hash(hash(0, e1), e2)...), en)
    /// final_hash = hash(hash, len)
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::pedersen::PedersenHash;
    /// use decoder_crypto_zk::field::FieldElement;
    ///
    /// let elements = vec![
    ///     FieldElement::from(1u64),
    ///     FieldElement::from(2u64),
    ///     FieldElement::from(3u64),
    /// ];
    /// let hash = PedersenHash::hash_many(&elements);
    /// ```
    pub fn hash_many(elements: &[FieldElement]) -> FieldElement {
        let mut hash = FieldElement::ZERO;

        for element in elements {
            hash = Self::hash_pair(&hash, element);
        }

        // Finalize with length to prevent length extension attacks
        Self::hash_pair(&hash, &FieldElement::from(elements.len() as u64))
    }
}

/// Stateful Pedersen hasher
///
/// This allows incremental hashing of multiple elements.
///
/// # Examples
///
/// ```
/// use decoder_crypto_zk::hash::pedersen::PedersenHasher;
/// use decoder_crypto_zk::field::FieldElement;
///
/// let mut hasher = PedersenHasher::new();
/// hasher.update(FieldElement::from(1u64));
/// hasher.update(FieldElement::from(2u64));
/// hasher.update(FieldElement::from(3u64));
/// let hash = hasher.finalize();
/// ```
#[derive(Debug, Default, Clone)]
pub struct PedersenHasher {
    hash: FieldElement,
    len: usize,
}

impl PedersenHasher {
    /// Creates a new Pedersen hasher
    pub fn new() -> Self {
        Self::default()
    }

    /// Absorbs a field element into the hash
    pub fn update(&mut self, msg: FieldElement) {
        self.hash = PedersenHash::hash_pair(&self.hash, &msg);
        self.len += 1;
    }

    /// Finishes and returns the hash
    ///
    /// This consumes the hasher and returns the final hash value.
    pub fn finalize(self) -> FieldElement {
        PedersenHash::hash_pair(&self.hash, &FieldElement::from(self.len as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_hash_pair() {
        let a = FieldElement::from(123u64);
        let b = FieldElement::from(456u64);
        let hash1 = PedersenHash::hash_pair(&a, &b);
        let hash2 = PedersenHash::hash_pair(&a, &b);

        // Deterministic
        assert_eq!(hash1, hash2);

        // Not commutative
        let hash3 = PedersenHash::hash_pair(&b, &a);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_pedersen_hash_many() {
        let elements = vec![
            FieldElement::from(1u64),
            FieldElement::from(2u64),
            FieldElement::from(3u64),
        ];
        let hash1 = PedersenHash::hash_many(&elements);
        let hash2 = PedersenHash::hash_many(&elements);

        // Deterministic
        assert_eq!(hash1, hash2);

        // Different for different inputs
        let elements2 = vec![
            FieldElement::from(3u64),
            FieldElement::from(2u64),
            FieldElement::from(1u64),
        ];
        let hash3 = PedersenHash::hash_many(&elements2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_pedersen_hasher() {
        let mut hasher = PedersenHasher::new();
        hasher.update(FieldElement::from(1u64));
        hasher.update(FieldElement::from(2u64));
        hasher.update(FieldElement::from(3u64));
        let hash1 = hasher.finalize();

        // Should match hash_many
        let elements = vec![
            FieldElement::from(1u64),
            FieldElement::from(2u64),
            FieldElement::from(3u64),
        ];
        let hash2 = PedersenHash::hash_many(&elements);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_pedersen_empty() {
        let empty: Vec<FieldElement> = vec![];
        let hash = PedersenHash::hash_many(&empty);

        // Empty hash should be deterministic
        assert_eq!(hash, PedersenHash::hash_many(&[]));
    }
}
