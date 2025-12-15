//! zkTrie - Poseidon-based Merkle tree for Polygon zkEVM
//!
//! zkTrie is a modified Merkle Patricia Trie that uses Poseidon hash instead of Keccak256.
//! This makes it more efficient to prove in zero-knowledge proof systems.
//!
//! ## Key Differences from Ethereum MPT
//!
//! | Feature | Ethereum MPT | zkTrie |
//! |---------|--------------|--------|
//! | Hash function | Keccak256 (256-bit) | Poseidon/Rescue Prime (Goldilocks field) |
//! | Field | - | Goldilocks (p = 2^64 - 2^32 + 1) |
//! | Optimization | For EVM execution | For zk-STARK proof generation |
//! | Proof size | Large (Keccak not ZK-friendly) | Small (algebraic hash) |
//!
//! ## Usage
//!
//! This module provides utilities for analyzing zkTrie structures in Polygon zkEVM:
//! - Parsing zkTrie nodes
//! - Computing Poseidon hashes of trie nodes
//! - Verifying state commitments
//!
//! ## References
//!
//! - [Polygon zkEVM Prover](https://github.com/0xPolygonHermez/zkevm-prover)
//! - [zkTrie Specification](https://docs.polygon.technology/zkevm/zkprover/)

use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
use decoder_crypto_zk::hash::poseidon_goldilocks::PoseidonGoldilocksHash;

/// zkTrie node types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkTrieNode {
    /// Branch node with up to 2 children (binary trie)
    Branch {
        /// Left child hash
        left: Option<ZkTrieHash>,
        /// Right child hash
        right: Option<ZkTrieHash>,
    },
    /// Leaf node containing account data or storage slot
    Leaf {
        /// Key hash
        key: ZkTrieHash,
        /// Value hash
        value: ZkTrieHash,
    },
    /// Empty node
    Empty,
}

/// zkTrie hash (Poseidon hash output in Goldilocks field)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZkTrieHash(pub GoldilocksFieldElement);

impl ZkTrieHash {
    /// Create from field element
    pub fn from_field(element: GoldilocksFieldElement) -> Self {
        Self(element)
    }

    /// Create from u64 value
    pub fn from_u64(value: u64) -> Self {
        Self(GoldilocksFieldElement::from_u64(value))
    }

    /// Get the underlying field element
    pub fn as_field(&self) -> GoldilocksFieldElement {
        self.0
    }

    /// Get as u64 (canonical representation)
    pub fn to_u64(&self) -> u64 {
        self.0.to_u64()
    }

    /// Zero hash (empty node)
    pub const ZERO: Self = Self(GoldilocksFieldElement::ZERO);
}

impl ZkTrieNode {
    /// Compute the hash of this trie node using Poseidon
    ///
    /// # Hashing Rules
    ///
    /// - **Branch**: H(left || right) using Poseidon hash_pair
    /// - **Leaf**: H(key || value) using Poseidon hash_pair
    /// - **Empty**: Zero field element
    ///
    /// # Example
    ///
    /// ```
    /// use decoder_polygon_zkevm::zktrie::{ZkTrieNode, ZkTrieHash};
    /// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
    ///
    /// // Create a leaf node
    /// let key = ZkTrieHash::from_u64(123);
    /// let value = ZkTrieHash::from_u64(456);
    /// let leaf = ZkTrieNode::Leaf { key, value };
    ///
    /// // Compute hash
    /// let hash = leaf.compute_hash();
    /// ```
    pub fn compute_hash(&self) -> ZkTrieHash {
        match self {
            ZkTrieNode::Branch { left, right } => {
                let left_hash = left.unwrap_or(ZkTrieHash::ZERO).as_field();
                let right_hash = right.unwrap_or(ZkTrieHash::ZERO).as_field();
                ZkTrieHash(PoseidonGoldilocksHash::hash_pair(left_hash, right_hash))
            }
            ZkTrieNode::Leaf { key, value } => ZkTrieHash(PoseidonGoldilocksHash::hash_pair(
                key.as_field(),
                value.as_field(),
            )),
            ZkTrieNode::Empty => ZkTrieHash::ZERO,
        }
    }

    /// Check if node is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, ZkTrieNode::Empty)
    }
}

/// zkTrie path utilities
pub mod path {
    /// Convert a key to a binary path in the trie
    ///
    /// In zkTrie, keys are hashed and then traversed bit-by-bit:
    /// - 0 = go left
    /// - 1 = go right
    pub fn key_to_path(key_hash: u64) -> Vec<bool> {
        (0..64).map(|i| (key_hash >> i) & 1 == 1).collect()
    }

    /// Get the bit at position in the path
    pub fn get_bit(key_hash: u64, position: usize) -> bool {
        if position >= 64 {
            false
        } else {
            (key_hash >> position) & 1 == 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zktrie_hash_creation() {
        let hash1 = ZkTrieHash::from_u64(12345);
        assert_eq!(hash1.to_u64(), 12345);

        let hash2 = ZkTrieHash::from_field(GoldilocksFieldElement::from_u64(67890));
        assert_eq!(hash2.to_u64(), 67890);
    }

    #[test]
    fn test_empty_node_hash() {
        let empty = ZkTrieNode::Empty;
        let hash = empty.compute_hash();
        assert_eq!(hash, ZkTrieHash::ZERO);
        assert_eq!(hash.to_u64(), 0);
    }

    #[test]
    fn test_leaf_node_hash() {
        let key = ZkTrieHash::from_u64(123);
        let value = ZkTrieHash::from_u64(456);
        let leaf = ZkTrieNode::Leaf { key, value };

        let hash = leaf.compute_hash();

        // Hash should be deterministic
        assert_eq!(leaf.compute_hash(), hash);

        // Hash should not be zero (extremely unlikely)
        assert_ne!(hash, ZkTrieHash::ZERO);
    }

    #[test]
    fn test_branch_node_hash() {
        let left = Some(ZkTrieHash::from_u64(111));
        let right = Some(ZkTrieHash::from_u64(222));
        let branch = ZkTrieNode::Branch { left, right };

        let hash = branch.compute_hash();

        // Hash should be deterministic
        assert_eq!(branch.compute_hash(), hash);

        // Hash should not be zero
        assert_ne!(hash, ZkTrieHash::ZERO);
    }

    #[test]
    fn test_branch_node_with_empty_children() {
        let branch1 = ZkTrieNode::Branch {
            left: None,
            right: None,
        };
        let hash1 = branch1.compute_hash();

        // Branch with no children should hash to same as H(0, 0)
        let left = ZkTrieHash::ZERO;
        let right = ZkTrieHash::ZERO;
        let expected = ZkTrieHash(PoseidonGoldilocksHash::hash_pair(
            left.as_field(),
            right.as_field(),
        ));

        assert_eq!(hash1, expected);
    }

    #[test]
    fn test_branch_node_asymmetric() {
        let left_only = ZkTrieNode::Branch {
            left: Some(ZkTrieHash::from_u64(100)),
            right: None,
        };

        let right_only = ZkTrieNode::Branch {
            left: None,
            right: Some(ZkTrieHash::from_u64(100)),
        };

        // Different structures should produce different hashes
        assert_ne!(left_only.compute_hash(), right_only.compute_hash());
    }

    #[test]
    fn test_path_conversion() {
        use super::path::*;

        // Test specific bit patterns
        assert!(!get_bit(0b0, 0));
        assert!(get_bit(0b1, 0));
        assert!(get_bit(0b10, 1));
        assert!(!get_bit(0b10, 0));

        // Test path length
        let path = key_to_path(12345);
        assert_eq!(path.len(), 64); // 64-bit path

        // Test out of bounds
        assert!(!get_bit(0xFF, 100));
    }

    #[test]
    fn test_is_empty() {
        let empty = ZkTrieNode::Empty;
        assert!(empty.is_empty());

        let leaf = ZkTrieNode::Leaf {
            key: ZkTrieHash::ZERO,
            value: ZkTrieHash::ZERO,
        };
        assert!(!leaf.is_empty());

        let branch = ZkTrieNode::Branch {
            left: None,
            right: None,
        };
        assert!(!branch.is_empty());
    }

    #[test]
    fn test_hash_collision_resistance() {
        // Different inputs should produce different hashes
        let leaf1 = ZkTrieNode::Leaf {
            key: ZkTrieHash::from_u64(1),
            value: ZkTrieHash::from_u64(2),
        };

        let leaf2 = ZkTrieNode::Leaf {
            key: ZkTrieHash::from_u64(2),
            value: ZkTrieHash::from_u64(1),
        };

        // Swapping key and value should produce different hash
        assert_ne!(leaf1.compute_hash(), leaf2.compute_hash());
    }
}
