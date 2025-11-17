//! Pallas field arithmetic for Mina Protocol
//!
//! The Pallas curve is one half of the Pasta curves (Pallas/Vesta cycle)
//! used extensively in Mina Protocol's zkSNARK system.
//!
//! # Field Definition
//!
//! The Pallas base field is defined by the prime:
//! ```text
//! p = 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001
//!   = 28948022309329048855892746252171976963363056481941560715954676764349967630337
//! ```
//!
//! This is a 255-bit prime (slightly larger than 2^254).
//!
//! # References
//!
//! - [Pasta Curves](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/)
//! - [Mina Book - Cryptography](https://o1-labs.github.io/proof-systems/specs/kimchi.html)

use crate::error::{CryptoError, Result};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// The Pallas field prime modulus
///
/// p = 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001
const PALLAS_MODULUS_HEX: &str = "40000000000000000000000000000000224698fc094cf91b992d30ed00000001";

/// Pallas field element
///
/// Represents an element in the Pallas base field F_p where
/// p = 28948022309329048855892746252171976963363056481941560715954676764349967630337
///
/// All arithmetic operations are performed modulo p.
///
/// # Examples
///
/// ```
/// use decoder_crypto_zk::field::pallas::PallasFieldElement;
///
/// let a = PallasFieldElement::from(123u64);
/// let b = PallasFieldElement::from(456u64);
/// let sum = a + b;
/// assert_eq!(sum, PallasFieldElement::from(579u64));
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct PallasFieldElement {
    value: BigUint,
}

impl PallasFieldElement {
    /// Get the Pallas field modulus
    fn modulus() -> BigUint {
        BigUint::parse_bytes(PALLAS_MODULUS_HEX.as_bytes(), 16).expect("Invalid Pallas modulus")
    }

    /// Create a field element from a BigUint (automatically reduced modulo p)
    fn from_biguint(value: BigUint) -> Self {
        let modulus = Self::modulus();
        Self {
            value: value % modulus,
        }
    }

    /// Zero element
    pub fn zero() -> Self {
        Self {
            value: BigUint::zero(),
        }
    }

    /// One element
    pub fn one() -> Self {
        Self {
            value: BigUint::one(),
        }
    }

    /// Two element (useful for Poseidon)
    pub fn two() -> Self {
        Self {
            value: BigUint::from(2u32),
        }
    }

    /// Create from a u64 value
    pub fn from_u64(value: u64) -> Self {
        Self {
            value: BigUint::from(value),
        }
    }

    /// Create from hexadecimal string
    ///
    /// # Examples
    ///
    /// ```
    /// use decoder_crypto_zk::field::pallas::PallasFieldElement;
    ///
    /// let fe = PallasFieldElement::from_hex("0x123").unwrap();
    /// ```
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let value = BigUint::parse_bytes(s.as_bytes(), 16)
            .ok_or_else(|| CryptoError::HexError(format!("Invalid hex string: {}", s)))?;
        Ok(Self::from_biguint(value))
    }

    /// Convert to hexadecimal string
    pub fn to_hex(&self) -> String {
        format!("0x{:x}", self.value)
    }

    /// Create from big-endian bytes
    pub fn from_bytes_be(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 32 {
            return Err(CryptoError::InvalidInputLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let value = BigUint::from_bytes_be(bytes);
        Ok(Self::from_biguint(value))
    }

    /// Convert to big-endian bytes (32 bytes)
    pub fn to_bytes_be(&self) -> [u8; 32] {
        let bytes = self.value.to_bytes_be();
        let mut result = [0u8; 32];
        let start = 32 - bytes.len();
        result[start..].copy_from_slice(&bytes);
        result
    }

    /// Multiplicative inverse (for field division)
    ///
    /// Computes a^(-1) mod p using Fermat's Little Theorem:
    /// a^(-1) = a^(p-2) mod p
    pub fn inverse(&self) -> Result<Self> {
        if self.value.is_zero() {
            return Err(CryptoError::DivisionByZero);
        }

        let modulus = Self::modulus();
        let exponent = &modulus - 2u32; // p - 2
        let result = self.value.modpow(&exponent, &modulus);
        Ok(Self { value: result })
    }

    /// Power operation (for Poseidon S-box)
    ///
    /// Computes self^exp mod p
    pub fn pow(&self, exp: u64) -> Self {
        let modulus = Self::modulus();
        let exponent = BigUint::from(exp);
        let result = self.value.modpow(&exponent, &modulus);
        Self { value: result }
    }
}

// Implement From<u64> for convenience
impl From<u64> for PallasFieldElement {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

// Implement From<u32> for convenience
impl From<u32> for PallasFieldElement {
    fn from(value: u32) -> Self {
        Self::from_u64(value as u64)
    }
}

// Addition
impl Add for PallasFieldElement {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let modulus = Self::modulus();
        let result = (self.value + other.value) % &modulus;
        Self { value: result }
    }
}

impl Add for &PallasFieldElement {
    type Output = PallasFieldElement;

    fn add(self, other: &PallasFieldElement) -> PallasFieldElement {
        let modulus = PallasFieldElement::modulus();
        let result = (&self.value + &other.value) % &modulus;
        PallasFieldElement { value: result }
    }
}

impl AddAssign for PallasFieldElement {
    fn add_assign(&mut self, other: Self) {
        let modulus = Self::modulus();
        self.value = (&self.value + &other.value) % &modulus;
    }
}

impl AddAssign<&PallasFieldElement> for PallasFieldElement {
    fn add_assign(&mut self, other: &PallasFieldElement) {
        let modulus = Self::modulus();
        self.value = (&self.value + &other.value) % &modulus;
    }
}

// Subtraction
impl Sub for PallasFieldElement {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let modulus = Self::modulus();
        let result = if self.value >= other.value {
            &self.value - &other.value
        } else {
            &modulus - (&other.value - &self.value)
        };
        Self { value: result }
    }
}

impl Sub for &PallasFieldElement {
    type Output = PallasFieldElement;

    fn sub(self, other: &PallasFieldElement) -> PallasFieldElement {
        let modulus = PallasFieldElement::modulus();
        let result = if self.value >= other.value {
            &self.value - &other.value
        } else {
            &modulus - (&other.value - &self.value)
        };
        PallasFieldElement { value: result }
    }
}

impl SubAssign for PallasFieldElement {
    fn sub_assign(&mut self, other: Self) {
        let modulus = Self::modulus();
        self.value = if self.value >= other.value {
            &self.value - &other.value
        } else {
            &modulus - (&other.value - &self.value)
        };
    }
}

// Multiplication
impl Mul for PallasFieldElement {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let modulus = Self::modulus();
        let result = (self.value * other.value) % &modulus;
        Self { value: result }
    }
}

impl Mul for &PallasFieldElement {
    type Output = PallasFieldElement;

    fn mul(self, other: &PallasFieldElement) -> PallasFieldElement {
        let modulus = PallasFieldElement::modulus();
        let result = (&self.value * &other.value) % &modulus;
        PallasFieldElement { value: result }
    }
}

impl MulAssign for PallasFieldElement {
    fn mul_assign(&mut self, other: Self) {
        let modulus = Self::modulus();
        self.value = (&self.value * &other.value) % &modulus;
    }
}

// Negation
impl Neg for PallasFieldElement {
    type Output = Self;

    fn neg(self) -> Self {
        if self.value.is_zero() {
            self
        } else {
            let modulus = Self::modulus();
            Self {
                value: modulus - self.value,
            }
        }
    }
}

impl Neg for &PallasFieldElement {
    type Output = PallasFieldElement;

    fn neg(self) -> PallasFieldElement {
        if self.value.is_zero() {
            PallasFieldElement {
                value: BigUint::zero(),
            }
        } else {
            let modulus = PallasFieldElement::modulus();
            PallasFieldElement {
                value: modulus - &self.value,
            }
        }
    }
}

// Debug and Display
impl fmt::Debug for PallasFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PallasFieldElement({})", self.to_hex())
    }
}

impl fmt::Display for PallasFieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// Serde implementation
impl Serialize for PallasFieldElement {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as hex string
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for PallasFieldElement {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        PallasFieldElement::from_hex(&hex_str).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pallas_modulus() {
        let modulus = PallasFieldElement::modulus();
        assert_eq!(modulus.to_str_radix(16), PALLAS_MODULUS_HEX.to_lowercase());
    }

    #[test]
    fn test_zero_one_two() {
        let zero = PallasFieldElement::zero();
        let one = PallasFieldElement::one();
        let two = PallasFieldElement::two();

        assert_eq!(zero.value, BigUint::zero());
        assert_eq!(one.value, BigUint::one());
        assert_eq!(two.value, BigUint::from(2u32));
    }

    #[test]
    fn test_from_u64() {
        let fe = PallasFieldElement::from_u64(12345);
        assert_eq!(fe.value, BigUint::from(12345u64));
    }

    #[test]
    fn test_from_hex() {
        let fe = PallasFieldElement::from_hex("0x123").unwrap();
        assert_eq!(fe.value, BigUint::from(0x123u64));

        // Test without 0x prefix
        let fe2 = PallasFieldElement::from_hex("456").unwrap();
        assert_eq!(fe2.value, BigUint::from(0x456u64));
    }

    #[test]
    fn test_to_hex() {
        let fe = PallasFieldElement::from_u64(0x123);
        let hex = fe.to_hex();
        assert_eq!(hex, "0x123");
    }

    #[test]
    fn test_addition() {
        let a = PallasFieldElement::from(100u64);
        let b = PallasFieldElement::from(200u64);
        let sum = a + b;
        assert_eq!(sum, PallasFieldElement::from(300u64));
    }

    #[test]
    fn test_addition_with_modulo() {
        // Test that addition wraps around modulo p
        let modulus = PallasFieldElement::modulus();
        let a = PallasFieldElement::from_biguint(modulus.clone() - 10u32);
        let b = PallasFieldElement::from(20u64);
        let sum = a + b;

        // (p - 10) + 20 = p + 10 ≡ 10 (mod p)
        assert_eq!(sum, PallasFieldElement::from(10u64));
    }

    #[test]
    fn test_subtraction() {
        let a = PallasFieldElement::from(300u64);
        let b = PallasFieldElement::from(100u64);
        let diff = a - b;
        assert_eq!(diff, PallasFieldElement::from(200u64));
    }

    #[test]
    fn test_subtraction_wrapping() {
        // Test that subtraction wraps around correctly
        let a = PallasFieldElement::from(10u64);
        let b = PallasFieldElement::from(20u64);
        let diff = a - b;

        // 10 - 20 = -10 ≡ p - 10 (mod p)
        let expected = PallasFieldElement::from_biguint(PallasFieldElement::modulus() - 10u32);
        assert_eq!(diff, expected);
    }

    #[test]
    fn test_multiplication() {
        let a = PallasFieldElement::from(12u64);
        let b = PallasFieldElement::from(34u64);
        let product = a * b;
        assert_eq!(product, PallasFieldElement::from(408u64));
    }

    #[test]
    fn test_negation() {
        let a = PallasFieldElement::from(123u64);
        let neg_a = -a.clone();

        // a + (-a) should equal 0
        let sum = a + neg_a;
        assert_eq!(sum, PallasFieldElement::zero());
    }

    #[test]
    fn test_negation_of_zero() {
        let zero = PallasFieldElement::zero();
        let neg_zero = -zero;
        assert_eq!(neg_zero, PallasFieldElement::zero());
    }

    #[test]
    fn test_inverse() {
        let a = PallasFieldElement::from(7u64);
        let inv_a = a.inverse().unwrap();

        // a * a^(-1) should equal 1
        let product = a * inv_a;
        assert_eq!(product, PallasFieldElement::one());
    }

    #[test]
    fn test_inverse_of_zero_fails() {
        let zero = PallasFieldElement::zero();
        assert!(zero.inverse().is_err());
    }

    #[test]
    fn test_pow() {
        let a = PallasFieldElement::from(2u64);
        let result = a.pow(10);

        // 2^10 = 1024
        assert_eq!(result, PallasFieldElement::from(1024u64));
    }

    #[test]
    fn test_bytes_roundtrip() {
        let original = PallasFieldElement::from(0x123456789abcdef0u64);
        let bytes = original.to_bytes_be();
        let restored = PallasFieldElement::from_bytes_be(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_add_assign() {
        let mut a = PallasFieldElement::from(100u64);
        a += PallasFieldElement::from(200u64);
        assert_eq!(a, PallasFieldElement::from(300u64));
    }

    #[test]
    fn test_sub_assign() {
        let mut a = PallasFieldElement::from(300u64);
        a -= PallasFieldElement::from(100u64);
        assert_eq!(a, PallasFieldElement::from(200u64));
    }

    #[test]
    fn test_mul_assign() {
        let mut a = PallasFieldElement::from(12u64);
        a *= PallasFieldElement::from(34u64);
        assert_eq!(a, PallasFieldElement::from(408u64));
    }
}
