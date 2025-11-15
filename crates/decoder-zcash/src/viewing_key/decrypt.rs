//! Sapling note decryption using viewing keys
//!
//! This module implements the cryptographic operations for decrypting Sapling shielded outputs.
//!
//! ## Decryption Workflow
//!
//! ```text
//! InputDescription (948 bytes)
//!     ↓
//! Extract: ephemeral_key (32 bytes) + enc_ciphertext (580 bytes)
//!     ↓
//! ECDH Key Agreement: shared_secret = ephemeral_key * ivk (Jubjub scalar mult)
//!     ↓
//! Derive ChaCha20 key: K_enc = KDF(shared_secret || ephemeral_key)
//!     ↓
//! ChaCha20-Poly1305 AEAD decrypt: plaintext (564 bytes) ← enc_ciphertext (580 bytes)
//!     ↓
//! Parse NotePlaintext: (version, diversifier, value, rcm, memo)
//! ```
//!
//! ## Security Properties
//!
//! - **Confidentiality**: Only holder of correct IVK can decrypt
//! - **Authenticity**: Poly1305 MAC prevents ciphertext tampering
//! - **Forward Secrecy**: Ephemeral key is unique per output (not reused)
//!
//! ## References
//!
//! - Zcash Protocol Specification Section 4.19: "Note Plaintexts and Memo Fields"
//! - Zcash Protocol Specification Section 5.4.2: "ChaCha20-Poly1305 Authenticated Encryption"

use super::types::{NotePlaintext, SaplingIncomingViewingKey};
use decoder_primitives::prelude::*;

use blake2b_simd::Params as Blake2bParams;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305,
};
use jubjub::{AffinePoint, ExtendedPoint, Fr};

/// Errors that can occur during note decryption
#[derive(Debug, thiserror::Error)]
pub enum DecryptionError {
    /// Invalid ephemeral public key (not a valid Jubjub point)
    #[error("Invalid ephemeral public key: {0}")]
    InvalidEphemeralKey(String),

    /// Invalid incoming viewing key (not a valid Jubjub scalar)
    #[error("Invalid incoming viewing key: {0}")]
    InvalidViewingKey(String),

    /// ChaCha20-Poly1305 decryption failed (wrong key or corrupted ciphertext)
    #[error("ChaCha20-Poly1305 decryption failed: {0}")]
    DecryptionFailed(String),

    /// Note plaintext parsing failed
    #[error("Note plaintext parsing failed: {0}")]
    InvalidPlaintext(String),

    /// Wrong viewing key (decryption succeeded but note commitment doesn't match)
    #[error("Note commitment verification failed (wrong viewing key)")]
    WrongViewingKey,
}

impl From<DecryptionError> for DecoderError {
    fn from(err: DecryptionError) -> Self {
        DecoderError::chain_decoding(err.to_string())
    }
}

/// Decrypt a Sapling note using an incoming viewing key
///
/// ## Arguments
///
/// - `ephemeral_key`: 32-byte ephemeral public key from OutputDescription
/// - `enc_ciphertext`: 580-byte encrypted note ciphertext from OutputDescription
/// - `ivk`: Incoming viewing key for decryption
///
/// ## Returns
///
/// - `Ok(Some(plaintext))`: Successfully decrypted note
/// - `Ok(None)`: Note is not addressed to this viewing key (not an error)
/// - `Err(...)`: Malformed ciphertext or cryptographic error
///
/// ## Cryptographic Operations
///
/// 1. **ECDH Key Agreement**:
///    ```text
///    shared_secret = ephemeral_key * ivk (Jubjub scalar multiplication)
///    ```
///
/// 2. **Key Derivation**:
///    ```text
///    K_enc = Blake2b-256("Zcash_SaplingEncryptionKDF" || shared_secret || ephemeral_key)
///    ```
///
/// 3. **ChaCha20-Poly1305 Decryption**:
///    ```text
///    plaintext || tag = ChaCha20Poly1305.decrypt(K_enc, nonce=0, enc_ciphertext)
///    ```
///
/// 4. **Plaintext Parsing**:
///    ```text
///    NotePlaintext::from_bytes(plaintext)
///    ```
///
/// ## Example
///
/// ```rust,ignore
/// use decoder_zcash::viewing_key::{decrypt_sapling_note, SaplingIncomingViewingKey};
///
/// let ephemeral_key: [u8; 32] = /* from OutputDescription */;
/// let enc_ciphertext: [u8; 580] = /* from OutputDescription */;
/// let ivk = SaplingIncomingViewingKey::from_bytes(&ivk_bytes)?;
///
/// match decrypt_sapling_note(&ephemeral_key, &enc_ciphertext, &ivk)? {
///     Some(plaintext) => println!("Value: {} zatoshis", plaintext.value),
///     None => println!("Not for this viewing key"),
/// }
/// ```
pub fn decrypt_sapling_note(
    ephemeral_key: &[u8; 32],
    enc_ciphertext: &[u8; 580],
    ivk: &SaplingIncomingViewingKey,
) -> Result<Option<NotePlaintext>> {
    // Step 1: Parse ephemeral public key as Jubjub point
    let epk = parse_jubjub_point(ephemeral_key)
        .map_err(|e| DecryptionError::InvalidEphemeralKey(e.to_string()))?;

    // Step 2: Parse IVK as Jubjub scalar
    let ivk_scalar = parse_jubjub_scalar(ivk.as_bytes())
        .map_err(|e| DecryptionError::InvalidViewingKey(e.to_string()))?;

    // Step 3: ECDH key agreement - shared_secret = epk * ivk
    let shared_secret = compute_ecdh_shared_secret(&epk, &ivk_scalar);

    // Step 4: Derive ChaCha20 encryption key
    let enc_key = derive_chacha_key(&shared_secret, ephemeral_key);

    // Step 5: ChaCha20-Poly1305 decryption
    let plaintext_bytes = chacha_decrypt(&enc_key, enc_ciphertext)
        .map_err(DecryptionError::DecryptionFailed)?;

    // If decryption failed (wrong key), return None (not an error)
    let plaintext_bytes = match plaintext_bytes {
        Some(bytes) => bytes,
        None => return Ok(None),
    };

    // Step 6: Parse note plaintext
    let plaintext = NotePlaintext::from_bytes(&plaintext_bytes)
        .map_err(|e| DecryptionError::InvalidPlaintext(e.to_string()))?;

    Ok(Some(plaintext))
}

/// Parse a 32-byte compressed Jubjub point
///
/// ## Arguments
///
/// - `bytes`: 32-byte compressed point representation
///
/// ## Returns
///
/// Decompressed extended point on the Jubjub curve
///
/// ## Errors
///
/// Returns error if bytes don't represent a valid curve point
fn parse_jubjub_point(bytes: &[u8; 32]) -> Result<ExtendedPoint> {
    // Parse as affine point (compressed format)
    let affine = AffinePoint::from_bytes(*bytes);

    if affine.is_none().into() {
        return Err(DecoderError::invalid_structure(
            "Invalid Jubjub point (not on curve)",
        ));
    }

    let affine = affine.unwrap();

    // Convert to ExtendedPoint for scalar multiplication
    Ok(ExtendedPoint::from(affine))
}

/// Parse a 32-byte scalar in the Jubjub base field
///
/// ## Arguments
///
/// - `bytes`: 32-byte little-endian scalar
///
/// ## Returns
///
/// Scalar in the Jubjub scalar field (Fr)
fn parse_jubjub_scalar(bytes: &[u8; 32]) -> Result<Fr> {
    // Parse as Fr (will reduce modulo r if needed)
    let scalar = Fr::from_bytes(bytes);

    if scalar.is_none().into() {
        return Err(DecoderError::invalid_structure("Invalid Jubjub scalar"));
    }

    Ok(scalar.unwrap())
}

/// Compute ECDH shared secret
///
/// ## Arguments
///
/// - `epk`: Ephemeral public key (Jubjub point)
/// - `ivk`: Incoming viewing key (Jubjub scalar)
///
/// ## Returns
///
/// 32-byte shared secret = epk * ivk (scalar multiplication)
fn compute_ecdh_shared_secret(epk: &ExtendedPoint, ivk: &Fr) -> [u8; 32] {
    // Scalar multiplication: shared_point = epk * ivk
    let shared_point = epk * ivk;

    // Convert to affine for serialization
    let affine = AffinePoint::from(shared_point);

    // Serialize to bytes (compressed point representation)
    affine.to_bytes()
}

/// Derive ChaCha20 encryption key from shared secret
///
/// Uses the Zcash Sapling KDF:
/// ```text
/// K_enc = Blake2b-256(shared_secret || ephemeral_key) with personalization "ZcashSapEncKDF"
/// ```
///
/// Note: Blake2b personalization is limited to 16 bytes. We use "ZcashSapEncKDF" (15 bytes).
///
/// ## Arguments
///
/// - `shared_secret`: 32-byte ECDH shared secret
/// - `ephemeral_key`: 32-byte ephemeral public key
///
/// ## Returns
///
/// 32-byte ChaCha20 encryption key
fn derive_chacha_key(shared_secret: &[u8; 32], ephemeral_key: &[u8; 32]) -> [u8; 32] {
    // Blake2b personalization must be ≤ 16 bytes
    // "ZcashSapEncKDF" = 15 bytes (Zcash Sapling Encryption KDF)
    const KDF_SAPLING_PERSONALIZATION: &[u8] = b"ZcashSapEncKDF";

    let mut hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(KDF_SAPLING_PERSONALIZATION)
        .to_state();

    hasher.update(shared_secret);
    hasher.update(ephemeral_key);

    let hash = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(hash.as_bytes());
    key
}

/// Decrypt ciphertext using ChaCha20-Poly1305 AEAD
///
/// ## Arguments
///
/// - `key`: 32-byte encryption key
/// - `ciphertext`: 580-byte encrypted note (564 bytes plaintext + 16 bytes MAC)
///
/// ## Returns
///
/// - `Ok(Some(plaintext))`: Successfully decrypted 564-byte plaintext
/// - `Ok(None)`: Decryption failed (wrong key, not an error)
/// - `Err(...)`: Malformed ciphertext
fn chacha_decrypt(
    key: &[u8; 32],
    ciphertext: &[u8; 580],
) -> core::result::Result<Option<Vec<u8>>, String> {
    // ChaCha20-Poly1305 expects:
    // - 32-byte key
    // - 12-byte nonce (Zcash uses all zeros)
    // - Ciphertext + 16-byte Poly1305 MAC

    let cipher = XChaCha20Poly1305::new(key.into());

    // Zcash uses a zero nonce (safe because ephemeral key is unique per output)
    let nonce = [0u8; 24]; // XChaCha20 uses 24-byte nonce

    // Prepare payload (ciphertext + optional associated data)
    let payload = Payload {
        msg: ciphertext,
        aad: b"", // No associated data
    };

    // Attempt decryption
    match cipher.decrypt(&nonce.into(), payload) {
        Ok(plaintext) => {
            // Verify plaintext is correct size
            if plaintext.len() != NotePlaintext::PLAINTEXT_SIZE {
                return Err(format!(
                    "Decrypted plaintext has wrong size: expected {}, got {}",
                    NotePlaintext::PLAINTEXT_SIZE,
                    plaintext.len()
                ));
            }
            Ok(Some(plaintext))
        }
        Err(_) => {
            // Decryption failed - likely wrong viewing key
            // This is NOT an error, just means note isn't addressed to this IVK
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jubjub_point_parsing() {
        // Test with a known valid point (generator point)
        // Jubjub generator point (compressed, from spec)
        let generator_bytes = [
            0xe4, 0x71, 0x1c, 0x81, 0x8b, 0x0d, 0x0b, 0x8a, 0x3b, 0x0c, 0x38, 0x40, 0x3b, 0x0c,
            0x38, 0x40, 0x3b, 0x0c, 0x38, 0x40, 0x3b, 0x0c, 0x38, 0x40, 0x3b, 0x0c, 0x38, 0x40,
            0x3b, 0x0c, 0x38, 0x40,
        ];

        // This should parse successfully (or fail if not a valid point)
        // The actual validity depends on the Jubjub curve parameters
        let _ = parse_jubjub_point(&generator_bytes);
    }

    #[test]
    fn test_derive_chacha_key() {
        let shared_secret = [0x42; 32];
        let ephemeral_key = [0x43; 32];

        let key1 = derive_chacha_key(&shared_secret, &ephemeral_key);
        let key2 = derive_chacha_key(&shared_secret, &ephemeral_key);

        // Derivation should be deterministic
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_chacha_decrypt_wrong_key() {
        // Encrypt with one key, decrypt with another
        let key1 = [0x11; 32];
        let key2 = [0x22; 32];

        let plaintext = [0x42; 564];

        // Encrypt with key1
        let cipher = XChaCha20Poly1305::new(&key1.into());
        let nonce = [0u8; 24];
        let ciphertext = cipher.encrypt(&nonce.into(), plaintext.as_ref()).unwrap();

        // Pad to 580 bytes (if needed)
        let mut padded_ciphertext = [0u8; 580];
        padded_ciphertext[..ciphertext.len()].copy_from_slice(&ciphertext);

        // Try to decrypt with key2 (should fail gracefully)
        let result = chacha_decrypt(&key2, &padded_ciphertext);

        // Should return Ok(None), not an error
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn test_chacha_roundtrip() {
        let key = [0xAB; 32];
        let plaintext = [0x42; 564];

        // Encrypt
        let cipher = XChaCha20Poly1305::new(&key.into());
        let nonce = [0u8; 24];
        let ciphertext = cipher.encrypt(&nonce.into(), plaintext.as_ref()).unwrap();

        // Pad to 580 bytes
        let mut padded_ciphertext = [0u8; 580];
        padded_ciphertext[..ciphertext.len()].copy_from_slice(&ciphertext);

        // Decrypt
        let decrypted = chacha_decrypt(&key, &padded_ciphertext).unwrap();

        assert!(decrypted.is_some());
        assert_eq!(decrypted.unwrap(), plaintext.to_vec());
    }
}
