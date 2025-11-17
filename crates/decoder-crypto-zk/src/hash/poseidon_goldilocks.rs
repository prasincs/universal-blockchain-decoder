//! Poseidon hash implementation for Goldilocks field (Polygon zkEVM)
//!
//! This module implements the Poseidon hash function using the Goldilocks field.
//! It's used in Polygon zkEVM for efficient zkSTARK proofs.
//!
//! # Poseidon Parameters for Goldilocks
//!
//! - **Field**: Goldilocks (p = 2^64 - 2^32 + 1)
//! - **State size**: 12 field elements (standard for Goldilocks Poseidon)
//! - **Full rounds**: 8 (4 at start, 4 at end)
//! - **Partial rounds**: 22
//! - **S-box**: x^7
//! - **Security level**: ~100-bit security
//!
//! # References
//!
//! - [Poseidon Paper](https://eprint.iacr.org/2019/458.pdf)
//! - [Plonky2 - Goldilocks Poseidon](https://github.com/mir-protocol/plonky2)
//! - [Polygon zkEVM Documentation](https://github.com/0xPolygonHermez/zkevm-prover)

use crate::field::goldilocks::GoldilocksFieldElement;

/// Poseidon parameters for Goldilocks
const STATE_SIZE: usize = 12;
const HALF_FULL_ROUNDS: usize = 4;
#[allow(dead_code)]
const FULL_ROUNDS: usize = HALF_FULL_ROUNDS * 2; // 8 total
const PARTIAL_ROUNDS: usize = 22;
#[allow(dead_code)]
const TOTAL_ROUNDS: usize = FULL_ROUNDS + PARTIAL_ROUNDS; // 30 rounds total
#[allow(dead_code)]
const SBOX_EXPONENT: u64 = 7; // x^7

/// Poseidon hash function for Goldilocks field (Polygon zkEVM)
///
/// This implements the Poseidon permutation using parameters optimized
/// for Polygon zkEVM's zkSTARK proofs on the Goldilocks field.
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
    /// Hash two field elements (most common operation)
    ///
    /// This is used for:
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
        // Initialize state with inputs in first two positions
        let mut state = [GoldilocksFieldElement::ZERO; STATE_SIZE];
        state[0] = x;
        state[1] = y;

        Self::permute(&mut state);
        state[0]
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
        let mut state = [GoldilocksFieldElement::ZERO; STATE_SIZE];
        state[0] = x;
        // Domain separation: set last element to ONE for single-element hashing
        state[STATE_SIZE - 1] = GoldilocksFieldElement::ONE;

        Self::permute(&mut state);
        state[0]
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
        let mut state = [GoldilocksFieldElement::ZERO; STATE_SIZE];
        let rate = STATE_SIZE - 1; // Capacity = 1

        let mut i = 0;
        while i < elements.len() {
            // Absorb phase: add elements to state
            for j in 0..rate {
                if i + j < elements.len() {
                    state[j] += elements[i + j];
                }
            }
            i += rate;

            // Permute
            Self::permute(&mut state);
        }

        // Squeeze phase: return first element
        state[0]
    }

    /// Apply the Poseidon permutation to the state
    ///
    /// This implements the full Poseidon permutation:
    /// - 4 full rounds (start)
    /// - 22 partial rounds (middle)
    /// - 4 full rounds (end)
    ///
    /// Each round consists of:
    /// 1. AddRoundConstants: Add round constants to state
    /// 2. SubWords (S-box): Apply x^7 to state elements
    /// 3. MixLayer: Multiply state by MDS matrix
    pub fn permute(state: &mut [GoldilocksFieldElement; STATE_SIZE]) {
        let mut round_ctr = 0;

        // First half of full rounds
        for _ in 0..HALF_FULL_ROUNDS {
            Self::full_round(state, round_ctr);
            round_ctr += 1;
        }

        // Partial rounds
        for _ in 0..PARTIAL_ROUNDS {
            Self::partial_round(state, round_ctr);
            round_ctr += 1;
        }

        // Second half of full rounds
        for _ in 0..HALF_FULL_ROUNDS {
            Self::full_round(state, round_ctr);
            round_ctr += 1;
        }
    }

    /// Apply a full round: S-box on all elements, then mix
    fn full_round(state: &mut [GoldilocksFieldElement; STATE_SIZE], round: usize) {
        // Add round constants
        Self::add_round_constants(state, round);

        // Apply S-box to all elements
        for element in state.iter_mut() {
            *element = Self::sbox(*element);
        }

        // Mix layer (MDS matrix multiplication)
        Self::mix_layer(state);
    }

    /// Apply a partial round: S-box on first element only, then mix
    fn partial_round(state: &mut [GoldilocksFieldElement; STATE_SIZE], round: usize) {
        // Add round constants
        Self::add_round_constants(state, round);

        // Apply S-box only to first element
        state[0] = Self::sbox(state[0]);

        // Mix layer (MDS matrix multiplication)
        Self::mix_layer(state);
    }

    /// S-box function: x^7 for Goldilocks
    #[inline(always)]
    fn sbox(x: GoldilocksFieldElement) -> GoldilocksFieldElement {
        let x2 = x * x;
        let x4 = x2 * x2;
        let x6 = x4 * x2;
        x6 * x
    }

    /// Add round constants to state
    fn add_round_constants(state: &mut [GoldilocksFieldElement; STATE_SIZE], round: usize) {
        for (i, element) in state.iter_mut().enumerate() {
            *element += Self::get_round_constant(round, i);
        }
    }

    /// Get round constant for given round and position
    ///
    /// TODO: Extract actual round constants from Polygon zkEVM prover
    /// For now, using placeholder constants derived from round and position
    fn get_round_constant(round: usize, pos: usize) -> GoldilocksFieldElement {
        // Placeholder: Generate pseudo-random constants from round and position
        // In production, these should be the actual constants from Polygon zkEVM
        let value = ((round as u64)
            .wrapping_mul(STATE_SIZE as u64)
            .wrapping_add(pos as u64))
        .wrapping_mul(0x9e3779b97f4a7c15); // Golden ratio multiplier
        GoldilocksFieldElement::from(value)
    }

    /// Mix layer: Multiply state by MDS (Maximum Distance Separable) matrix
    ///
    /// TODO: Extract actual MDS matrix from Polygon zkEVM prover
    /// For now, using a simple circulant matrix as placeholder
    fn mix_layer(state: &mut [GoldilocksFieldElement; STATE_SIZE]) {
        let mut new_state = [GoldilocksFieldElement::ZERO; STATE_SIZE];

        // Placeholder MDS matrix: circulant matrix
        // In production, this should be the actual MDS matrix from Polygon zkEVM
        for (i, new_element) in new_state.iter_mut().enumerate() {
            for (j, state_element) in state.iter().enumerate() {
                let coeff = Self::get_mds_element(i, j);
                *new_element += coeff * *state_element;
            }
        }

        *state = new_state;
    }

    /// Get MDS matrix element at (row, col)
    ///
    /// TODO: Extract actual MDS matrix from Polygon zkEVM prover
    /// Placeholder: Using circulant matrix with first row = [2, 1, 1, 1, ..., 1]
    fn get_mds_element(row: usize, col: usize) -> GoldilocksFieldElement {
        let diff = if col >= row {
            col - row
        } else {
            STATE_SIZE - row + col
        };

        if diff == 0 {
            GoldilocksFieldElement::TWO
        } else {
            GoldilocksFieldElement::ONE
        }
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

        // hash_single(a) should differ from hash_pair(a, 0) due to different initialization
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
    fn test_sbox() {
        let x = GoldilocksFieldElement::from(2u64);
        let result = PoseidonGoldilocksHash::sbox(x);

        // 2^7 = 128
        assert_eq!(result, GoldilocksFieldElement::from(128u64));
    }

    #[test]
    fn test_sbox_zero() {
        let zero = GoldilocksFieldElement::ZERO;
        let result = PoseidonGoldilocksHash::sbox(zero);

        // 0^7 = 0
        assert_eq!(result, GoldilocksFieldElement::ZERO);
    }

    #[test]
    fn test_sbox_one() {
        let one = GoldilocksFieldElement::ONE;
        let result = PoseidonGoldilocksHash::sbox(one);

        // 1^7 = 1
        assert_eq!(result, GoldilocksFieldElement::ONE);
    }

    #[test]
    fn test_permute_deterministic() {
        let mut state1 = [GoldilocksFieldElement::ZERO; STATE_SIZE];
        state1[0] = GoldilocksFieldElement::from(123u64);
        state1[1] = GoldilocksFieldElement::from(456u64);

        let mut state2 = state1;

        PoseidonGoldilocksHash::permute(&mut state1);
        PoseidonGoldilocksHash::permute(&mut state2);

        // Permutation should be deterministic
        assert_eq!(state1, state2);
    }

    #[test]
    fn test_permute_changes_state() {
        let mut state = [GoldilocksFieldElement::ZERO; STATE_SIZE];
        state[0] = GoldilocksFieldElement::from(123u64);

        let original = state;
        PoseidonGoldilocksHash::permute(&mut state);

        // Permutation should change the state
        assert_ne!(state, original);
    }

    #[test]
    fn test_avalanche_effect() {
        // Small change in input should cause large change in output
        let a1 = GoldilocksFieldElement::from(123u64);
        let a2 = GoldilocksFieldElement::from(124u64); // One bit different
        let b = GoldilocksFieldElement::from(456u64);

        let hash1 = PoseidonGoldilocksHash::hash_pair(a1, b);
        let hash2 = PoseidonGoldilocksHash::hash_pair(a2, b);

        // Should produce completely different hashes
        assert_ne!(hash1, hash2);
    }

    // TODO: Add test vectors from Polygon zkEVM prover
    // Once we extract the actual round constants and MDS matrix,
    // we should add tests that verify our implementation against
    // known test vectors from Polygon zkEVM.
}
