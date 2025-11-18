//! Goldilocks field arithmetic for Polygon zkEVM
//!
//! This module provides a thin wrapper around the battle-tested winterfell
//! implementation of the Goldilocks field.
//!
//! # Field Definition
//!
//! The Goldilocks field is defined by the prime:
//! ```text
//! p = 2^64 - 2^32 + 1
//!   = 18446744069414584321
//!   = 0xffffffff00000001
//! ```
//!
//! # Why We Use winterfell
//!
//! Instead of implementing field arithmetic from scratch, we use Facebook/Meta's
//! battle-tested winterfell crypto library:
//!
//! - ✅ Production-grade (used by Facebook/Meta)
//! - ✅ Works on stable Rust (no nightly features)
//! - ✅ Extensively tested
//! - ✅ Optimized for performance
//! - ✅ Goldilocks field is a core primitive
//!
//! # References
//!
//! - [Winterfell](https://github.com/facebook/winterfell)
//! - [Goldilocks Field Paper](https://eprint.iacr.org/2022/1542.pdf)

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use winterfell::math::FieldElement as WinterfellFieldElement;

// Re-export the battle-tested Goldilocks field from winterfell
pub use winterfell::math::fields::f64::BaseElement as WinterfellGoldilocksField;

/// Goldilocks field element (wrapper around winterfell's implementation)
///
/// # Examples
///
/// ```
/// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
///
/// let a = GoldilocksFieldElement::from(123u64);
/// let b = GoldilocksFieldElement::from(456u64);
/// let sum = a + b;
/// assert_eq!(sum.to_u64(), 579);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GoldilocksFieldElement(pub WinterfellGoldilocksField);

impl GoldilocksFieldElement {
    /// Zero element
    pub const ZERO: Self = Self(WinterfellGoldilocksField::ZERO);

    /// One element
    pub const ONE: Self = Self(WinterfellGoldilocksField::ONE);

    /// Create from u64 (automatically reduced modulo p)
    pub fn from_u64(value: u64) -> Self {
        Self(WinterfellGoldilocksField::new(value))
    }

    /// Create from canonical u64 (assumes value < p)
    pub fn from_canonical_u64(value: u64) -> Self {
        Self::from_u64(value)
    }

    /// Convert to u64
    pub fn to_u64(&self) -> u64 {
        self.0.as_int()
    }

    /// Convert to canonical u64 (alias)
    pub fn to_canonical_u64(&self) -> u64 {
        self.to_u64()
    }

    /// Convert to bytes (little-endian)
    pub fn to_bytes_le(&self) -> [u8; 8] {
        self.to_u64().to_le_bytes()
    }

    /// Convert to bytes (big-endian)
    pub fn to_bytes_be(&self) -> [u8; 8] {
        self.to_u64().to_be_bytes()
    }

    /// Create from bytes (little-endian)
    pub fn from_bytes_le(bytes: &[u8; 8]) -> Self {
        Self::from_u64(u64::from_le_bytes(*bytes))
    }

    /// Create from bytes (big-endian)
    pub fn from_bytes_be(bytes: &[u8; 8]) -> Self {
        Self::from_u64(u64::from_be_bytes(*bytes))
    }

    /// Create from hexadecimal string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches("0x");
        u64::from_str_radix(hex, 16)
            .map_err(|_| crate::error::CryptoError::HexError("Invalid hex string".to_string()))
            .map(Self::from_u64)
    }

    /// Multiplicative inverse
    pub fn inverse(&self) -> Result<Self> {
        if self.0 == WinterfellGoldilocksField::ZERO {
            return Err(crate::error::CryptoError::DivisionByZero);
        }
        Ok(Self(self.0.inv()))
    }

    /// Square this element
    pub fn square(&self) -> Self {
        Self(self.0.square())
    }

    /// Double this element
    pub fn double(&self) -> Self {
        Self(self.0.double())
    }

    /// Negate this element
    pub fn neg(&self) -> Self {
        Self(-self.0)
    }
}

// Arithmetic operations

impl std::ops::Add for GoldilocksFieldElement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for GoldilocksFieldElement {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for GoldilocksFieldElement {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for GoldilocksFieldElement {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Mul for GoldilocksFieldElement {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl std::ops::MulAssign for GoldilocksFieldElement {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl std::ops::Neg for GoldilocksFieldElement {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

// Conversions

impl From<u64> for GoldilocksFieldElement {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<u32> for GoldilocksFieldElement {
    fn from(value: u32) -> Self {
        Self::from_u64(value as u64)
    }
}

impl From<u16> for GoldilocksFieldElement {
    fn from(value: u16) -> Self {
        Self::from_u64(value as u64)
    }
}

impl From<u8> for GoldilocksFieldElement {
    fn from(value: u8) -> Self {
        Self::from_u64(value as u64)
    }
}

// Display

impl fmt::Display for GoldilocksFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_u64())
    }
}

// Serialization

impl Serialize for GoldilocksFieldElement {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{:016x}", self.to_u64()))
    }
}

impl<'de> Deserialize<'de> for GoldilocksFieldElement {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(GoldilocksFieldElement::ZERO.to_u64(), 0);
        assert_eq!(GoldilocksFieldElement::ONE.to_u64(), 1);
        assert_eq!(GoldilocksFieldElement::from_u64(2).to_u64(), 2);
    }

    #[test]
    fn test_arithmetic() {
        let a = GoldilocksFieldElement::from(123u64);
        let b = GoldilocksFieldElement::from(456u64);

        assert_eq!((a + b).to_u64(), 579);
        assert_eq!((b - a).to_u64(), 333);
        assert_eq!((a * b).to_u64(), 123 * 456);
    }

    #[test]
    fn test_inverse() {
        let a = GoldilocksFieldElement::from(123u64);
        let inv = a.inverse().unwrap();
        assert_eq!((a * inv).to_u64(), 1);
    }

    #[test]
    fn test_inverse_of_zero_fails() {
        let zero = GoldilocksFieldElement::ZERO;
        assert!(zero.inverse().is_err());
    }

    #[test]
    fn test_bytes_roundtrip() {
        let a = GoldilocksFieldElement::from(123456789u64);

        let bytes_le = a.to_bytes_le();
        let b = GoldilocksFieldElement::from_bytes_le(&bytes_le);
        assert_eq!(a, b);

        let bytes_be = a.to_bytes_be();
        let c = GoldilocksFieldElement::from_bytes_be(&bytes_be);
        assert_eq!(a, c);
    }

    #[test]
    fn test_hex_roundtrip() {
        let a = GoldilocksFieldElement::from(123u64);
        let hex = format!("0x{:x}", a.to_u64());
        let b = GoldilocksFieldElement::from_hex(&hex).unwrap();
        assert_eq!(a, b);
    }
}
