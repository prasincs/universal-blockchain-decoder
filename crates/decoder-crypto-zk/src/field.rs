//! STARK field arithmetic
//!
//! This module provides 252-bit field operations for Starknet and related ZK systems.
//!
//! The STARK field is defined by the prime:
//! ```text
//! p = 2^251 + 17 * 2^192 + 1
//! ```
//!
//! This is the same field used in the Cairo VM and Starknet.

use crate::error::{CryptoError, Result};

// Re-export Felt from starknet-types-core
// This is battle-tested field arithmetic used by the entire Starknet ecosystem
pub use starknet_types_core::felt::Felt;

/// Field element for STARK operations
///
/// This is a 252-bit field element used in Starknet cryptography.
/// All operations are performed modulo the STARK prime.
///
/// # Examples
///
/// ```
/// use decoder_crypto_zk::field::FieldElement;
///
/// let a = FieldElement::from_hex("0x123").unwrap();
/// let b = FieldElement::from_hex("0x456").unwrap();
/// let sum = a + b;
/// ```
pub type FieldElement = Felt;

/// Extension trait for field operations
pub trait FieldExt {
    /// Create a field element from a hexadecimal string
    fn from_hex(s: &str) -> Result<Self>
    where
        Self: Sized;

    /// Convert to hexadecimal string
    fn to_hex(&self) -> String;
}

impl FieldExt for FieldElement {
    fn from_hex(s: &str) -> Result<Self> {
        // Remove 0x prefix if present
        let s = s.strip_prefix("0x").unwrap_or(s);

        Felt::from_hex(s).map_err(|_| CryptoError::HexError(format!("Invalid hex string: {}", s)))
    }

    fn to_hex(&self) -> String {
        // Convert to hex bytes and format with 0x prefix
        let bytes = self.to_bytes_be();
        format!("0x{}", hex::encode(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_from_hex() {
        let fe = FieldElement::from_hex("0x123").unwrap();
        assert_eq!(fe, Felt::from_hex("123").unwrap());
    }

    #[test]
    fn test_field_to_hex() {
        let fe = Felt::from_hex("123").unwrap();
        let hex = fe.to_hex();
        assert!(hex.starts_with("0x"));
    }

    #[test]
    fn test_field_arithmetic() {
        let a = Felt::from(123u64);
        let b = Felt::from(456u64);
        let sum = a + b;
        assert_eq!(sum, Felt::from(579u64));
    }

    #[test]
    fn test_field_constants() {
        assert_eq!(Felt::ZERO, Felt::from(0u64));
        assert_eq!(Felt::ONE, Felt::from(1u64));
        assert_eq!(Felt::TWO, Felt::from(2u64));
    }
}
