//! Common cryptographic hash functions used across blockchain decoders.
//!
//! This module provides standardized hash operations to ensure consistency
//! and reduce code duplication across decoder implementations.

use sha2::{Digest, Sha256};
use sha3::Keccak256;

/// Computes a single SHA-256 hash.
///
/// # Arguments
///
/// * `data` - The data to hash
///
/// # Returns
///
/// A 32-byte SHA-256 hash
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::hashing;
///
/// let data = b"hello world";
/// let hash = hashing::sha256(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Computes a double SHA-256 hash (Bitcoin-style).
///
/// This is commonly used in Bitcoin and Bitcoin-like chains for transaction
/// and block hashing: `SHA256(SHA256(data))`.
///
/// # Arguments
///
/// * `data` - The data to hash
///
/// # Returns
///
/// A 32-byte double-SHA-256 hash
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::hashing;
///
/// let data = b"hello world";
/// let hash = hashing::sha256_double(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn sha256_double(data: &[u8]) -> Vec<u8> {
    let first_hash = sha256(data);
    sha256(&first_hash)
}

/// Computes a Keccak-256 hash (Ethereum-style).
///
/// This is used in Ethereum and EVM-compatible chains for transaction
/// and address hashing.
///
/// # Arguments
///
/// * `data` - The data to hash
///
/// # Returns
///
/// A 32-byte Keccak-256 hash
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::hashing;
///
/// let data = b"hello world";
/// let hash = hashing::keccak256(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Computes a SHA-256 hash and returns it as a fixed-size array.
///
/// This is useful when you need a `[u8; 32]` instead of a `Vec<u8>`.
///
/// # Arguments
///
/// * `data` - The data to hash
///
/// # Returns
///
/// A 32-byte SHA-256 hash as a fixed-size array
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::hashing;
///
/// let data = b"hello world";
/// let hash: [u8; 32] = hashing::sha256_array(data);
/// ```
pub fn sha256_array(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes a double SHA-256 hash and returns it as a fixed-size array.
///
/// # Arguments
///
/// * `data` - The data to hash
///
/// # Returns
///
/// A 32-byte double-SHA-256 hash as a fixed-size array
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::hashing;
///
/// let data = b"hello world";
/// let hash: [u8; 32] = hashing::sha256_double_array(data);
/// ```
pub fn sha256_double_array(data: &[u8]) -> [u8; 32] {
    let first_hash = sha256_array(data);
    sha256_array(&first_hash)
}

/// Computes a Keccak-256 hash and returns it as a fixed-size array.
///
/// # Arguments
///
/// * `data` - The data to hash
///
/// # Returns
///
/// A 32-byte Keccak-256 hash as a fixed-size array
///
/// # Example
///
/// ```rust
/// use decoder_chains_common::hashing;
///
/// let data = b"hello world";
/// let hash: [u8; 32] = hashing::keccak256_array(data);
/// ```
pub fn keccak256_array(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = sha256(data);

        assert_eq!(hash.len(), 32);

        // Test determinism
        let hash2 = sha256(data);
        assert_eq!(hash, hash2);

        // Test different inputs produce different hashes
        let hash3 = sha256(b"different data");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_sha256_double() {
        let data = b"hello world";
        let hash = sha256_double(data);

        assert_eq!(hash.len(), 32);

        // Verify it's actually double-hashed
        let manual_double = sha256(&sha256(data));
        assert_eq!(hash, manual_double);

        // Test determinism
        let hash2 = sha256_double(data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_keccak256() {
        let data = b"hello world";
        let hash = keccak256(data);

        assert_eq!(hash.len(), 32);

        // Test determinism
        let hash2 = keccak256(data);
        assert_eq!(hash, hash2);

        // Keccak should produce different hash than SHA256
        let sha_hash = sha256(data);
        assert_ne!(hash, sha_hash);
    }

    #[test]
    fn test_sha256_array() {
        let data = b"hello world";
        let hash = sha256_array(data);

        assert_eq!(hash.len(), 32);

        // Should match vector version
        let vec_hash = sha256(data);
        assert_eq!(&hash[..], &vec_hash[..]);
    }

    #[test]
    fn test_sha256_double_array() {
        let data = b"hello world";
        let hash = sha256_double_array(data);

        assert_eq!(hash.len(), 32);

        // Should match vector version
        let vec_hash = sha256_double(data);
        assert_eq!(&hash[..], &vec_hash[..]);
    }

    #[test]
    fn test_keccak256_array() {
        let data = b"hello world";
        let hash = keccak256_array(data);

        assert_eq!(hash.len(), 32);

        // Should match vector version
        let vec_hash = keccak256(data);
        assert_eq!(&hash[..], &vec_hash[..]);
    }

    #[test]
    fn test_empty_input() {
        let empty: &[u8] = &[];

        // All functions should handle empty input
        assert_eq!(sha256(empty).len(), 32);
        assert_eq!(sha256_double(empty).len(), 32);
        assert_eq!(keccak256(empty).len(), 32);
        assert_eq!(sha256_array(empty).len(), 32);
        assert_eq!(sha256_double_array(empty).len(), 32);
        assert_eq!(keccak256_array(empty).len(), 32);
    }
}
