//! Goldilocks field arithmetic for Polygon zkEVM
//!
//! The Goldilocks field is a 64-bit field optimized for efficient STARK proofs.
//! It's used extensively in Polygon zkEVM and other zkSTARK systems.
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
//! This prime has special properties that make arithmetic very efficient:
//! - Fits in a 64-bit word
//! - Has special form allowing fast modular reduction
//! - Named "Goldilocks" because it's "just right" - not too big, not too small
//!
//! # References
//!
//! - [Goldilocks Field](https://eprint.iacr.org/2022/1542.pdf) - "Plonky2: Fast Recursive Arguments with Plonk and FRI"
//! - [Polygon zkEVM](https://github.com/0xPolygonHermez/zkevm-prover)

use crate::error::{CryptoError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// The Goldilocks field prime modulus
///
/// p = 2^64 - 2^32 + 1 = 0xffffffff00000001
pub const GOLDILOCKS_MODULUS: u64 = 0xffffffff00000001;

/// Goldilocks field element
///
/// Represents an element in the Goldilocks field F_p where
/// p = 2^64 - 2^32 + 1 = 18446744069414584321
///
/// All arithmetic operations are performed modulo p using efficient
/// algorithms that take advantage of the special form of the modulus.
///
/// # Examples
///
/// ```
/// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
///
/// let a = GoldilocksFieldElement::from(123u64);
/// let b = GoldilocksFieldElement::from(456u64);
/// let sum = a + b;
/// assert_eq!(sum, GoldilocksFieldElement::from(579u64));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoldilocksFieldElement {
    /// Value stored in canonical form: 0 <= value < p
    value: u64,
}

impl GoldilocksFieldElement {
    /// Zero element
    pub const ZERO: Self = Self { value: 0 };

    /// One element
    pub const ONE: Self = Self { value: 1 };

    /// Two element (useful for Poseidon)
    pub const TWO: Self = Self { value: 2 };

    /// The field modulus
    pub const MODULUS: u64 = GOLDILOCKS_MODULUS;

    /// Create a field element from a u64 value (automatically reduced modulo p)
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
    ///
    /// let x = GoldilocksFieldElement::from(123u64);
    /// assert_eq!(x.to_u64(), 123);
    /// ```
    pub fn from_u64(value: u64) -> Self {
        Self {
            value: Self::reduce(value),
        }
    }

    /// Create from canonical u64 (assumes value < p, panics otherwise)
    ///
    /// Use this when you know the value is already in canonical form.
    /// This is faster than `from_u64` because it skips reduction.
    pub fn from_canonical_u64(value: u64) -> Self {
        assert!(
            value < GOLDILOCKS_MODULUS,
            "Value must be less than modulus"
        );
        Self { value }
    }

    /// Create from hexadecimal string
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::field::goldilocks::GoldilocksFieldElement;
    ///
    /// let x = GoldilocksFieldElement::from_hex("7b").unwrap();
    /// assert_eq!(x.to_u64(), 123);
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches("0x");
        u64::from_str_radix(hex, 16)
            .map_err(|_| CryptoError::HexError("Invalid hexadecimal string".to_string()))
            .map(Self::from_u64)
    }

    /// Convert to u64
    pub fn to_u64(&self) -> u64 {
        self.value
    }

    /// Convert to bytes (little-endian)
    pub fn to_bytes_le(&self) -> [u8; 8] {
        self.value.to_le_bytes()
    }

    /// Convert to bytes (big-endian)
    pub fn to_bytes_be(&self) -> [u8; 8] {
        self.value.to_be_bytes()
    }

    /// Create from bytes (little-endian)
    pub fn from_bytes_le(bytes: &[u8; 8]) -> Self {
        Self::from_u64(u64::from_le_bytes(*bytes))
    }

    /// Create from bytes (big-endian)
    pub fn from_bytes_be(bytes: &[u8; 8]) -> Self {
        Self::from_u64(u64::from_be_bytes(*bytes))
    }

    /// Efficient modular reduction for Goldilocks field
    ///
    /// Uses the special form of the modulus p = 2^64 - 2^32 + 1
    /// to perform reduction without division.
    ///
    /// Algorithm:
    /// If x >= p, then x - p = x - (2^64 - 2^32 + 1) = x + 2^32 - 1 (mod 2^64)
    fn reduce(value: u64) -> u64 {
        if value >= GOLDILOCKS_MODULUS {
            // Wrapping arithmetic: (value + 2^32 - 1) mod 2^64
            value.wrapping_add(0xFFFFFFFF)
        } else {
            value
        }
    }

    /// Efficient modular reduction for 128-bit intermediate results
    ///
    /// This is used after multiplication to reduce the 128-bit result
    /// back to the field.
    ///
    /// Based on the algorithm from Plonky2:
    /// https://github.com/mir-protocol/plonky2/blob/main/field/src/goldilocks_field.rs
    fn reduce_u128(value: u128) -> u64 {
        // For now, use a simple but correct approach
        // TODO: Optimize with clever reduction later
        let modulus = GOLDILOCKS_MODULUS as u128;
        let reduced = value % modulus;
        reduced as u64
    }

    /// Compute multiplicative inverse using Fermat's Little Theorem
    ///
    /// For prime p, a^(p-1) = 1 (mod p), so a^(p-2) = a^(-1) (mod p)
    ///
    /// This uses binary exponentiation for efficiency.
    pub fn inverse(&self) -> Result<Self> {
        if self.value == 0 {
            return Err(CryptoError::DivisionByZero);
        }

        // Compute self^(p-2) mod p using binary exponentiation
        let exponent = GOLDILOCKS_MODULUS - 2;
        Ok(self.pow(exponent))
    }

    /// Exponentiation using binary exponentiation
    ///
    /// Computes self^exponent (mod p) efficiently
    pub fn pow(&self, mut exponent: u64) -> Self {
        let mut result = Self::ONE;
        let mut base = *self;

        while exponent > 0 {
            if exponent & 1 == 1 {
                result *= base;
            }
            base *= base;
            exponent >>= 1;
        }

        result
    }

    /// Square this element
    pub fn square(&self) -> Self {
        *self * *self
    }

    /// Double this element
    pub fn double(&self) -> Self {
        *self + *self
    }
}

// Arithmetic operations

impl Add for GoldilocksFieldElement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let (sum, overflow) = self.value.overflowing_add(rhs.value);
        let sum = if overflow || sum >= GOLDILOCKS_MODULUS {
            // If overflow or >= p, subtract p
            // p = 2^64 - 2^32 + 1, so subtracting p is equivalent to adding 2^32 - 1
            sum.wrapping_add(0xFFFFFFFF)
        } else {
            sum
        };
        Self { value: sum }
    }
}

impl AddAssign for GoldilocksFieldElement {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for GoldilocksFieldElement {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        if self.value >= rhs.value {
            Self {
                value: self.value - rhs.value,
            }
        } else {
            // Underflow: add p and then subtract
            Self {
                value: GOLDILOCKS_MODULUS - (rhs.value - self.value),
            }
        }
    }
}

impl SubAssign for GoldilocksFieldElement {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for GoldilocksFieldElement {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let product = (self.value as u128) * (rhs.value as u128);
        Self {
            value: Self::reduce_u128(product),
        }
    }
}

impl MulAssign for GoldilocksFieldElement {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Neg for GoldilocksFieldElement {
    type Output = Self;

    fn neg(self) -> Self {
        if self.value == 0 {
            Self::ZERO
        } else {
            Self {
                value: GOLDILOCKS_MODULUS - self.value,
            }
        }
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

// Display and Debug

impl fmt::Debug for GoldilocksFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GoldilocksFieldElement({})", self.value)
    }
}

impl fmt::Display for GoldilocksFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// Serialization

impl Serialize for GoldilocksFieldElement {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{:016x}", self.value))
    }
}

impl<'de> Deserialize<'de> for GoldilocksFieldElement {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulus_value() {
        // Verify p = 2^64 - 2^32 + 1
        let expected = (1u128 << 64) - (1u128 << 32) + 1;
        assert_eq!(GOLDILOCKS_MODULUS as u128, expected);
        assert_eq!(GOLDILOCKS_MODULUS, 0xffffffff00000001);
    }

    #[test]
    fn test_constants() {
        assert_eq!(GoldilocksFieldElement::ZERO.value, 0);
        assert_eq!(GoldilocksFieldElement::ONE.value, 1);
        assert_eq!(GoldilocksFieldElement::TWO.value, 2);
    }

    #[test]
    fn test_addition() {
        let a = GoldilocksFieldElement::from(123u64);
        let b = GoldilocksFieldElement::from(456u64);
        let sum = a + b;
        assert_eq!(sum.to_u64(), 579);
    }

    #[test]
    fn test_addition_with_reduction() {
        let a = GoldilocksFieldElement::from(GOLDILOCKS_MODULUS - 1);
        let b = GoldilocksFieldElement::from(2u64);
        let sum = a + b;
        // (p-1) + 2 = p+1 ≡ 1 (mod p)
        assert_eq!(sum.to_u64(), 1);
    }

    #[test]
    fn test_subtraction() {
        let a = GoldilocksFieldElement::from(456u64);
        let b = GoldilocksFieldElement::from(123u64);
        let diff = a - b;
        assert_eq!(diff.to_u64(), 333);
    }

    #[test]
    fn test_subtraction_with_underflow() {
        let a = GoldilocksFieldElement::from(123u64);
        let b = GoldilocksFieldElement::from(456u64);
        let diff = a - b;
        // 123 - 456 = -333 ≡ p - 333 (mod p)
        assert_eq!(diff.to_u64(), GOLDILOCKS_MODULUS - 333);
    }

    #[test]
    fn test_multiplication() {
        let a = GoldilocksFieldElement::from(123u64);
        let b = GoldilocksFieldElement::from(456u64);
        let product = a * b;
        assert_eq!(product.to_u64(), 123 * 456);
    }

    #[test]
    fn test_multiplication_large() {
        let a = GoldilocksFieldElement::from(1u64 << 32);
        let b = GoldilocksFieldElement::from(1u64 << 32);
        let product = a * b;
        // (2^32)^2 = 2^64 ≡ 2^32 - 1 (mod p)
        assert_eq!(product.to_u64(), (1u64 << 32) - 1);
    }

    #[test]
    fn test_negation() {
        let a = GoldilocksFieldElement::from(123u64);
        let neg_a = -a;
        assert_eq!(neg_a.to_u64(), GOLDILOCKS_MODULUS - 123);

        // -0 = 0
        let zero = GoldilocksFieldElement::ZERO;
        assert_eq!((-zero).to_u64(), 0);
    }

    #[test]
    fn test_additive_inverse() {
        let a = GoldilocksFieldElement::from(123u64);
        let sum = a + (-a);
        assert_eq!(sum, GoldilocksFieldElement::ZERO);
    }

    #[test]
    fn test_inverse() {
        let a = GoldilocksFieldElement::from(123u64);
        let inv = a.inverse().unwrap();
        let product = a * inv;
        assert_eq!(product, GoldilocksFieldElement::ONE);
    }

    #[test]
    fn test_inverse_of_zero_fails() {
        let zero = GoldilocksFieldElement::ZERO;
        assert!(zero.inverse().is_err());
    }

    #[test]
    fn test_pow() {
        let a = GoldilocksFieldElement::from(2u64);

        // 2^0 = 1
        assert_eq!(a.pow(0), GoldilocksFieldElement::ONE);

        // 2^1 = 2
        assert_eq!(a.pow(1), a);

        // 2^10 = 1024
        assert_eq!(a.pow(10).to_u64(), 1024);
    }

    #[test]
    fn test_square() {
        let a = GoldilocksFieldElement::from(123u64);
        assert_eq!(a.square(), a * a);
    }

    #[test]
    fn test_double() {
        let a = GoldilocksFieldElement::from(123u64);
        assert_eq!(a.double(), a + a);
    }

    #[test]
    fn test_bytes_roundtrip_le() {
        let a = GoldilocksFieldElement::from(123456789u64);
        let bytes = a.to_bytes_le();
        let b = GoldilocksFieldElement::from_bytes_le(&bytes);
        assert_eq!(a, b);
    }

    #[test]
    fn test_bytes_roundtrip_be() {
        let a = GoldilocksFieldElement::from(123456789u64);
        let bytes = a.to_bytes_be();
        let b = GoldilocksFieldElement::from_bytes_be(&bytes);
        assert_eq!(a, b);
    }

    #[test]
    fn test_hex_roundtrip() {
        let a = GoldilocksFieldElement::from(123u64);
        let hex = format!("0x{:x}", a.to_u64());
        let b = GoldilocksFieldElement::from_hex(&hex).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_field_axioms() {
        let a = GoldilocksFieldElement::from(123u64);
        let b = GoldilocksFieldElement::from(456u64);
        let c = GoldilocksFieldElement::from(789u64);

        // Associativity of addition
        assert_eq!((a + b) + c, a + (b + c));

        // Associativity of multiplication
        assert_eq!((a * b) * c, a * (b * c));

        // Commutativity of addition
        assert_eq!(a + b, b + a);

        // Commutativity of multiplication
        assert_eq!(a * b, b * a);

        // Distributivity
        assert_eq!(a * (b + c), a * b + a * c);

        // Additive identity
        assert_eq!(a + GoldilocksFieldElement::ZERO, a);

        // Multiplicative identity
        assert_eq!(a * GoldilocksFieldElement::ONE, a);

        // Additive inverse
        assert_eq!(a + (-a), GoldilocksFieldElement::ZERO);
    }

    #[test]
    fn test_from_canonical_u64_panics() {
        let result = std::panic::catch_unwind(|| {
            GoldilocksFieldElement::from_canonical_u64(GOLDILOCKS_MODULUS)
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_reduce() {
        // Value already in range
        assert_eq!(GoldilocksFieldElement::reduce(123), 123);

        // Value equal to modulus
        assert_eq!(GoldilocksFieldElement::reduce(GOLDILOCKS_MODULUS), 0);

        // Value greater than modulus
        let value = GOLDILOCKS_MODULUS + 123;
        assert_eq!(GoldilocksFieldElement::reduce(value), 123);
    }
}
