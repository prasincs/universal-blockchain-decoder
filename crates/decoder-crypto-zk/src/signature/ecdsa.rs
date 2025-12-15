//! ECDSA signature verification on STARK curve
//!
//! This module provides ECDSA signature verification for Starknet transactions.
//! The implementation is extracted from the vendored `starknet-crypto` library
//! (<https://github.com/xJonathanLEI/starknet-rs/tree/master/starknet-crypto>).
//!
//! # For Decoding Only
//!
//! This module only implements verification (not signing) as we're building a decoder.
//! Transaction decoders need to verify existing signatures, not create new ones.
//!
//! # Usage
//!
//! ```
//! use decoder_crypto_zk::signature::verify;
//! use decoder_crypto_zk::field::FieldElement;
//! use decoder_crypto_zk::VerifyError;
//!
//! let public_key = FieldElement::from(123u64);
//! let message = FieldElement::from(456u64);
//! let r = FieldElement::from(789u64);
//! let s = FieldElement::from(101112u64);
//!
//! // Verify signature
//! let result = verify(&public_key, &message, &r, &s)?;
//! # Ok::<(), VerifyError>(())
//! ```

use crate::curve::{ALPHA, BETA, EC_ORDER, GENERATOR};
use crate::field::FieldElement;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Zero};
use starknet_types_core::curve::{AffinePoint, ProjectivePoint};

// ============================================================================
// Public API
// ============================================================================

/// Stark ECDSA signature
#[derive(Debug, Clone)]
pub struct Signature {
    /// The `r` value of a signature
    pub r: FieldElement,
    /// The `s` value of a signature
    pub s: FieldElement,
}

/// Errors when performing ECDSA verification
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VerifyError {
    /// The public key is not a valid point on the STARK curve
    #[error("Invalid public key")]
    InvalidPublicKey,
    /// The message hash is not in the range of [0, 2^251)
    #[error("Invalid message hash")]
    InvalidMessageHash,
    /// The `r` value is not in the range of [0, 2^251)
    #[error("Invalid r value")]
    InvalidR,
    /// The `s` value is not in the range of [0, 2^251)
    #[error("Invalid s value")]
    InvalidS,
}

/// The (exclusive) upper bound on many ECDSA-related elements.
///
/// Value: `0x0800000000000000000000000000000000000000000000000000000000000000`
///
/// Based on the original C++ implementation from crypto-cpp.
const ELEMENT_UPPER_BOUND: FieldElement = FieldElement::from_raw([
    576459263475450960,
    18446744073709255680,
    160989183,
    18446743986131435553,
]);

/// Verifies if a signature is valid over a message hash given a public key.
///
/// Returns `Ok(true)` if the signature is valid, `Ok(false)` if invalid but parameters
/// are well-formed, or `Err` if the parameters are malformed.
///
/// # Parameters
///
/// - `public_key`: The public key (x-coordinate of the point)
/// - `message`: The message hash
/// - `r`: The `r` value of the signature
/// - `s`: The `s` value of the signature
///
/// # Algorithm
///
/// The verification checks if the signature (r, s) is valid for the given message and public key:
/// 1. Validate all inputs are in valid ranges
/// 2. Reconstruct the full public key point from x-coordinate
/// 3. Compute w = s^-1 mod EC_ORDER
/// 4. Compute zw = message * w mod EC_ORDER
/// 5. Compute rw = r * w mod EC_ORDER
/// 6. Check if (zw*G + rw*Q).x == r or (zw*G - rw*Q).x == r
///
/// This follows the standard ECDSA verification algorithm adapted for the STARK curve.
///
/// # Example
///
/// ```
/// use decoder_crypto_zk::signature::verify;
/// use decoder_crypto_zk::field::FieldElement;
///
/// let public_key = FieldElement::from(123u64);
/// let message = FieldElement::from(456u64);
/// let r = FieldElement::from(789u64);
/// let s = FieldElement::from(101112u64);
///
/// let result = verify(&public_key, &message, &r, &s);
/// assert!(result.is_ok());
/// ```
///
/// # References
///
/// - Vendored from: <https://github.com/xJonathanLEI/starknet-rs/blob/master/starknet-crypto/src/ecdsa.rs>
/// - Original C++ implementation: <https://github.com/starkware-libs/crypto-cpp>
pub fn verify(
    public_key: &FieldElement,
    message: &FieldElement,
    r: &FieldElement,
    s: &FieldElement,
) -> Result<bool, VerifyError> {
    // Validate inputs are in valid ranges
    if message >= &ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidMessageHash);
    }
    if r == &FieldElement::ZERO || r >= &ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidR);
    }
    if s == &FieldElement::ZERO || s >= &ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidS);
    }

    // Reconstruct the full public key point from x-coordinate
    // For the STARK curve: y^2 = x^3 + alpha * x + beta
    let y_squared = public_key.square() * public_key + ALPHA * public_key + BETA;
    let y = y_squared.sqrt().ok_or(VerifyError::InvalidPublicKey)?;

    let full_public_key = AffinePoint::new(*public_key, y).unwrap();

    // Compute w = s^-1 mod EC_ORDER
    let w = mod_inverse(s, &EC_ORDER);
    if w == FieldElement::ZERO || w >= ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidS);
    }

    // Compute zw = message * w mod EC_ORDER
    let zw = mul_mod_floor(message, &w, &EC_ORDER);
    let zw_g = mul_by_scalar(&GENERATOR, &zw);

    // Compute rw = r * w mod EC_ORDER
    let rw = mul_mod_floor(r, &w, &EC_ORDER);
    let rw_q = mul_by_scalar(&full_public_key, &rw);

    // Check if (zw*G + rw*Q).x == r or (zw*G - rw*Q).x == r
    // The second check handles the case where y might be negated
    let sum_point = (&zw_g + &rw_q).to_affine().unwrap();
    let diff_point = (&zw_g - &rw_q).to_affine().unwrap();

    Ok(sum_point.x() == *r || diff_point.x() == *r)
}

// ============================================================================
// Helper functions (extracted from vendored fe_utils.rs)
// ============================================================================

/// Multiply two field elements modulo a modulus
///
/// Computes: (multiplicand * multiplier) mod modulus
fn mul_mod_floor(
    multiplicand: &FieldElement,
    multiplier: &FieldElement,
    modulus: &FieldElement,
) -> FieldElement {
    let multiplicand = BigInt::from_bytes_be(num_bigint::Sign::Plus, &multiplicand.to_bytes_be());
    let multiplier = BigInt::from_bytes_be(num_bigint::Sign::Plus, &multiplier.to_bytes_be());
    let modulus = BigInt::from_bytes_be(num_bigint::Sign::Plus, &modulus.to_bytes_be());

    let result = (multiplicand * multiplier).mod_floor(&modulus);

    bigint_to_felt(result)
}

/// Compute modular inverse using extended Euclidean algorithm
///
/// Computes: operand^-1 mod modulus
fn mod_inverse(operand: &FieldElement, modulus: &FieldElement) -> FieldElement {
    let operand = BigInt::from_bytes_be(num_bigint::Sign::Plus, &operand.to_bytes_be());
    let modulus = BigInt::from_bytes_be(num_bigint::Sign::Plus, &modulus.to_bytes_be());

    let extended_gcd = operand.extended_gcd(&modulus);
    if extended_gcd.gcd != BigInt::one() {
        // This should never happen for valid inputs on the STARK curve
        return FieldElement::ZERO;
    }

    let result = if extended_gcd.x < BigInt::zero() {
        extended_gcd.x + modulus
    } else {
        extended_gcd.x
    };

    bigint_to_felt(result)
}

/// Scalar multiplication on elliptic curve
///
/// Computes: point * scalar
fn mul_by_scalar(point: &AffinePoint, scalar: &FieldElement) -> ProjectivePoint {
    &ProjectivePoint::from_affine(point.x(), point.y()).unwrap() * *scalar
}

/// Convert BigInt to FieldElement
#[inline]
fn bigint_to_felt(value: BigInt) -> FieldElement {
    let (_, buffer) = value.to_bytes_be();
    let mut result_bytes = [0u8; 32];
    result_bytes[(32 - buffer.len())..].copy_from_slice(&buffer[..]);
    FieldElement::from_bytes_be(&result_bytes)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_runs_without_panic() {
        // Basic smoke test - ensure it doesn't panic on invalid inputs
        let public_key = FieldElement::from(1u64);
        let message = FieldElement::from(2u64);
        let r = FieldElement::from(3u64);
        let s = FieldElement::from(4u64);

        let result = verify(&public_key, &message, &r, &s);
        // Should return Ok (either true or false), not panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_rejects_invalid_message() {
        let public_key = FieldElement::from(1u64);
        let message = ELEMENT_UPPER_BOUND; // Invalid: >= upper bound
        let r = FieldElement::from(3u64);
        let s = FieldElement::from(4u64);

        assert!(matches!(
            verify(&public_key, &message, &r, &s),
            Err(VerifyError::InvalidMessageHash)
        ));
    }

    #[test]
    fn test_verify_rejects_zero_r() {
        let public_key = FieldElement::from(1u64);
        let message = FieldElement::from(2u64);
        let r = FieldElement::ZERO; // Invalid
        let s = FieldElement::from(4u64);

        assert!(matches!(
            verify(&public_key, &message, &r, &s),
            Err(VerifyError::InvalidR)
        ));
    }

    #[test]
    fn test_verify_rejects_zero_s() {
        let public_key = FieldElement::from(1u64);
        let message = FieldElement::from(2u64);
        let r = FieldElement::from(3u64);
        let s = FieldElement::ZERO; // Invalid

        assert!(matches!(
            verify(&public_key, &message, &r, &s),
            Err(VerifyError::InvalidS)
        ));
    }

    #[test]
    fn test_signature_struct() {
        let sig = Signature {
            r: FieldElement::from(123u64),
            s: FieldElement::from(456u64),
        };

        assert_eq!(sig.r, FieldElement::from(123u64));
        assert_eq!(sig.s, FieldElement::from(456u64));
    }
}
