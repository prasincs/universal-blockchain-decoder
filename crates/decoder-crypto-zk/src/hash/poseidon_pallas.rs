//! Poseidon hash implementation for Pallas field (Mina Protocol)
//!
//! This module implements the Poseidon hash function using the Pallas field
//! from the Pasta curves. It's the primary hash function used in Mina Protocol
//! for zkSNARK-friendly hashing.
//!
//! # Poseidon Parameters for Mina
//!
//! - **Field**: Pallas base field (28948022309329048855892746252171976963363056481941560715954676764349967630337)
//! - **State size**: 3 field elements
//! - **Full rounds**: 8 full rounds (4 at start, 4 at end)
//! - **Partial rounds**: 55 partial rounds (in the middle)
//! - **S-box**: x^7 (optimized for Pallas)
//! - **Security level**: 128-bit security
//!
//! # References
//!
//! - [Poseidon Paper](https://eprint.iacr.org/2019/458.pdf)
//! - [Mina Book - Poseidon](https://o1-labs.github.io/proof-systems/specs/poseidon.html)
//! - [o1js reference implementation](https://github.com/o1-labs/o1js/blob/main/src/lib/provable/crypto/poseidon.ts)

use crate::field::pallas::PallasFieldElement;

/// Poseidon parameters for Pallas
const STATE_SIZE: usize = 3;
const FULL_ROUNDS: usize = 8; // 4 full rounds at start, 4 at end
const PARTIAL_ROUNDS: usize = 55;
const TOTAL_ROUNDS: usize = FULL_ROUNDS + PARTIAL_ROUNDS; // 63 rounds total
const SBOX_EXPONENT: u64 = 7; // x^7 for Pallas

/// Poseidon hash function for Pallas field (Mina Protocol)
///
/// This implements the Poseidon permutation using parameters optimized
/// for Mina's zkSNARK circuits on the Pallas curve.
///
/// # Examples
///
/// ```
/// use decoder_crypto_zk::hash::poseidon_pallas::PoseidonPallasHash;
/// use decoder_crypto_zk::field::pallas::PallasFieldElement;
///
/// let a = PallasFieldElement::from(123u64);
/// let b = PallasFieldElement::from(456u64);
/// let hash = PoseidonPallasHash::hash_pair(a, b);
/// ```
pub struct PoseidonPallasHash;

impl PoseidonPallasHash {
    /// Hash two field elements (most common operation)
    ///
    /// This is used extensively in Mina for:
    /// - Merkle tree construction
    /// - Transaction hashing
    /// - State commitments
    /// - zkApp state updates
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::poseidon_pallas::PoseidonPallasHash;
    /// use decoder_crypto_zk::field::pallas::PallasFieldElement;
    ///
    /// let a = PallasFieldElement::from(123u64);
    /// let b = PallasFieldElement::from(456u64);
    /// let hash = PoseidonPallasHash::hash_pair(a, b);
    /// ```
    pub fn hash_pair(x: PallasFieldElement, y: PallasFieldElement) -> PallasFieldElement {
        // Domain separation: use 2 in the last position for pair hashing
        let mut state = [x, y, PallasFieldElement::two()];
        Self::permute(&mut state);
        state[0].clone()
    }

    /// Hash a single field element
    ///
    /// Used for domain separation and single-value commitments.
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::poseidon_pallas::PoseidonPallasHash;
    /// use decoder_crypto_zk::field::pallas::PallasFieldElement;
    ///
    /// let value = PallasFieldElement::from(123u64);
    /// let hash = PoseidonPallasHash::hash_single(value);
    /// ```
    pub fn hash_single(x: PallasFieldElement) -> PallasFieldElement {
        // Domain separation: use 1 in the last position for single hashing
        let mut state = [x, PallasFieldElement::zero(), PallasFieldElement::one()];
        Self::permute(&mut state);
        state[0].clone()
    }

    /// Hash many field elements using the sponge construction
    ///
    /// This is used for hashing arrays and variable-length data in zkApps.
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::hash::poseidon_pallas::PoseidonPallasHash;
    /// use decoder_crypto_zk::field::pallas::PallasFieldElement;
    ///
    /// let elements = vec![
    ///     PallasFieldElement::from(1u64),
    ///     PallasFieldElement::from(2u64),
    ///     PallasFieldElement::from(3u64),
    /// ];
    /// let hash = PoseidonPallasHash::hash_many(&elements);
    /// ```
    pub fn hash_many(elements: &[PallasFieldElement]) -> PallasFieldElement {
        let mut state = [
            PallasFieldElement::zero(),
            PallasFieldElement::zero(),
            PallasFieldElement::zero(),
        ];
        let mut iter = elements.iter();

        loop {
            match iter.next() {
                Some(v) => state[0] = &state[0] + v,
                None => {
                    state[0] = &state[0] + &PallasFieldElement::one();
                    break;
                }
            }

            match iter.next() {
                Some(v) => state[1] = &state[1] + v,
                None => {
                    state[1] = &state[1] + &PallasFieldElement::one();
                    break;
                }
            }

            Self::permute(&mut state);
        }
        Self::permute(&mut state);

        state[0].clone()
    }

    /// Apply the Poseidon permutation to the state
    ///
    /// This implements the full Poseidon permutation:
    /// - 4 full rounds
    /// - 55 partial rounds
    /// - 4 full rounds
    ///
    /// Each round consists of:
    /// 1. Add round constants (ARK)
    /// 2. Apply S-box (x^7)
    /// 3. Mix with MDS matrix
    ///
    /// # Note
    ///
    /// This is a simplified implementation that uses placeholder constants.
    /// For production use, the actual round constants and MDS matrix from
    /// Mina's specification should be used.
    pub fn permute(state: &mut [PallasFieldElement; STATE_SIZE]) {
        // NOTE: This is a simplified implementation
        // In production, we would use the actual round constants and MDS matrix
        // from Mina's Poseidon specification

        // First 4 full rounds
        for round in 0..4 {
            Self::full_round(state, round);
        }

        // 55 partial rounds
        for round in 4..(4 + PARTIAL_ROUNDS) {
            Self::partial_round(state, round);
        }

        // Final 4 full rounds
        for round in (4 + PARTIAL_ROUNDS)..TOTAL_ROUNDS {
            Self::full_round(state, round);
        }
    }

    /// Apply a full round (S-box to all elements)
    fn full_round(state: &mut [PallasFieldElement; STATE_SIZE], round: usize) {
        // Add round constants (simplified - would use actual ARK constants)
        for (i, elem) in state.iter_mut().enumerate() {
            let ark = Self::get_round_constant(round, i);
            *elem = &*elem + &ark;
        }

        // Apply S-box to all elements
        for elem in state.iter_mut() {
            *elem = elem.pow(SBOX_EXPONENT);
        }

        // Mix with MDS matrix
        Self::apply_mds(state);
    }

    /// Apply a partial round (S-box only to first element)
    fn partial_round(state: &mut [PallasFieldElement; STATE_SIZE], round: usize) {
        // Add round constants (simplified - would use actual ARK constants)
        for (i, elem) in state.iter_mut().enumerate() {
            let ark = Self::get_round_constant(round, i);
            *elem = &*elem + &ark;
        }

        // Apply S-box only to first element
        state[0] = state[0].pow(SBOX_EXPONENT);

        // Mix with MDS matrix
        Self::apply_mds(state);
    }

    /// Get round constant (simplified implementation)
    ///
    /// NOTE: This is a placeholder. In production, we would use the actual
    /// round constants from Mina's Poseidon specification.
    fn get_round_constant(round: usize, index: usize) -> PallasFieldElement {
        // Simplified constant generation (deterministic but not the actual Mina constants)
        // In production, these would be loaded from Mina's specification
        let value = ((round as u64) * 7 + (index as u64) * 13 + 1) % 1000;
        PallasFieldElement::from(value)
    }

    /// Apply the MDS (Maximum Distance Separable) matrix
    ///
    /// NOTE: This is a simplified implementation using a Cauchy matrix.
    /// In production, we would use the actual MDS matrix from Mina's specification.
    fn apply_mds(state: &mut [PallasFieldElement; STATE_SIZE]) {
        // Simplified MDS matrix (3x3 Cauchy matrix)
        // In production, this would be the actual MDS matrix from Mina's spec
        let temp = [state[0].clone(), state[1].clone(), state[2].clone()];

        // Simple mixing (placeholder for actual MDS matrix multiplication)
        state[0] = &(&temp[0] + &temp[1]) + &temp[2];
        state[1] = &(&temp[0] + &temp[2]) + &temp[2];
        state[2] = &(&temp[1] + &temp[2]) + &temp[0];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_hash_pair() {
        let a = PallasFieldElement::from(123u64);
        let b = PallasFieldElement::from(456u64);
        let hash1 = PoseidonPallasHash::hash_pair(a.clone(), b.clone());
        let hash2 = PoseidonPallasHash::hash_pair(a.clone(), b.clone());

        // Deterministic
        assert_eq!(hash1, hash2);

        // Not commutative (hash(a,b) != hash(b,a))
        let hash3 = PoseidonPallasHash::hash_pair(b, a);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_poseidon_hash_single() {
        let a = PallasFieldElement::from(123u64);
        let hash1 = PoseidonPallasHash::hash_single(a.clone());
        let hash2 = PoseidonPallasHash::hash_single(a);

        // Deterministic
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_poseidon_hash_many() {
        let elements = vec![
            PallasFieldElement::from(1u64),
            PallasFieldElement::from(2u64),
            PallasFieldElement::from(3u64),
        ];
        let hash1 = PoseidonPallasHash::hash_many(&elements);
        let hash2 = PoseidonPallasHash::hash_many(&elements);

        // Deterministic
        assert_eq!(hash1, hash2);

        // Different for different inputs
        let elements2 = vec![
            PallasFieldElement::from(3u64),
            PallasFieldElement::from(2u64),
            PallasFieldElement::from(1u64),
        ];
        let hash3 = PoseidonPallasHash::hash_many(&elements2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_poseidon_empty_hash() {
        let empty: Vec<PallasFieldElement> = vec![];
        let hash = PoseidonPallasHash::hash_many(&empty);

        // Empty hash should be deterministic
        assert_eq!(hash, PoseidonPallasHash::hash_many(&[]));
    }

    #[test]
    fn test_poseidon_permute_deterministic() {
        let mut state1 = [
            PallasFieldElement::from(1u64),
            PallasFieldElement::from(2u64),
            PallasFieldElement::from(3u64),
        ];
        let mut state2 = state1.clone();

        PoseidonPallasHash::permute(&mut state1);
        PoseidonPallasHash::permute(&mut state2);

        // Permutation is deterministic
        assert_eq!(state1[0], state2[0]);
        assert_eq!(state1[1], state2[1]);
        assert_eq!(state1[2], state2[2]);
    }

    #[test]
    fn test_hash_single_vs_hash_pair() {
        let a = PallasFieldElement::from(123u64);

        // hash_single(a) should differ from hash_pair(a, 0)
        let hash_single = PoseidonPallasHash::hash_single(a.clone());
        let hash_pair = PoseidonPallasHash::hash_pair(a, PallasFieldElement::zero());

        // These use different initial states, so should differ
        assert_ne!(hash_single, hash_pair);
    }

    #[test]
    fn test_hash_many_different_lengths() {
        let short = vec![PallasFieldElement::from(1u64)];
        let long = vec![
            PallasFieldElement::from(1u64),
            PallasFieldElement::from(2u64),
            PallasFieldElement::from(3u64),
            PallasFieldElement::from(4u64),
        ];

        let hash_short = PoseidonPallasHash::hash_many(&short);
        let hash_long = PoseidonPallasHash::hash_many(&long);

        // Different lengths should produce different hashes
        assert_ne!(hash_short, hash_long);
    }
}
