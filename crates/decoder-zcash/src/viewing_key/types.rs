//! Viewing key and note plaintext types for Zcash Sapling

use decoder_primitives::prelude::*;
use std::fmt;

/// Sapling Incoming Viewing Key (IVK)
///
/// Allows decrypting incoming shielded transactions (notes received).
/// Derived from the Full Viewing Key (FVK).
///
/// ## Security Properties
///
/// - **Reveals**: All incoming transactions, amounts, and memos
/// - **Conceals**: Spending authority, sender information
/// - **Derivation**: `ivk = CRH_ivk(ak, nk)` from full viewing key
///
/// ## Binary Format
///
/// - **Size**: 32 bytes (scalar in Jubjub base field)
/// - **Encoding**: Little-endian byte array
///
/// ## Example
///
/// ```rust,ignore
/// use decoder_zcash::viewing_key::SaplingIncomingViewingKey;
///
/// let ivk_bytes: [u8; 32] = /* from wallet or key derivation */;
/// let ivk = SaplingIncomingViewingKey::from_bytes(&ivk_bytes)?;
/// ```
#[derive(Clone)]
pub struct SaplingIncomingViewingKey {
    /// 32-byte incoming viewing key (scalar in Jubjub base field)
    pub(crate) ivk: [u8; 32],
}

impl SaplingIncomingViewingKey {
    /// Size of IVK in bytes
    pub const SIZE: usize = 32;

    /// Create an IVK from 32 bytes
    ///
    /// ## Arguments
    ///
    /// - `bytes`: 32-byte incoming viewing key
    ///
    /// ## Errors
    ///
    /// Returns `DecoderError` if the byte array is not exactly 32 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "IVK must be {} bytes, got {}",
                Self::SIZE,
                bytes.len()
            )));
        }

        let mut ivk = [0u8; 32];
        ivk.copy_from_slice(bytes);

        Ok(Self { ivk })
    }

    /// Get the raw IVK bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.ivk
    }
}

// Don't print IVK bytes (security sensitive)
impl fmt::Debug for SaplingIncomingViewingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SaplingIncomingViewingKey([REDACTED])")
    }
}

/// Sapling Full Viewing Key (FVK)
///
/// Contains complete viewing authority:
/// - Incoming viewing key (decrypt received notes)
/// - Outgoing viewing key (decrypt sent notes)
/// - Nullifier deriving key (link spends to notes)
///
/// ## Components
///
/// ```text
/// FVK = (ak, nk, ovk)
///   ak  - Authorizing key (32 bytes, Jubjub point)
///   nk  - Nullifier deriving key (32 bytes, Jubjub point)
///   ovk - Outgoing viewing key (32 bytes, arbitrary)
/// ```
///
/// ## Derivation
///
/// - **From Spending Key**: `FVK = derive(ask, nsk)`
/// - **To IVK**: `ivk = CRH_ivk(ak, nk)`
#[derive(Clone)]
pub struct SaplingFullViewingKey {
    /// Authorizing key (32 bytes)
    pub ak: [u8; 32],

    /// Nullifier deriving key (32 bytes)
    pub nk: [u8; 32],

    /// Outgoing viewing key (32 bytes)
    pub ovk: [u8; 32],
}

impl SaplingFullViewingKey {
    /// Total size of FVK (ak + nk + ovk)
    pub const SIZE: usize = 96;

    /// Create an FVK from 96 bytes
    ///
    /// ## Format
    ///
    /// ```text
    /// [0..32]   - ak (authorizing key)
    /// [32..64]  - nk (nullifier deriving key)
    /// [64..96]  - ovk (outgoing viewing key)
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "FVK must be {} bytes, got {}",
                Self::SIZE,
                bytes.len()
            )));
        }

        let mut ak = [0u8; 32];
        let mut nk = [0u8; 32];
        let mut ovk = [0u8; 32];

        ak.copy_from_slice(&bytes[0..32]);
        nk.copy_from_slice(&bytes[32..64]);
        ovk.copy_from_slice(&bytes[64..96]);

        Ok(Self { ak, nk, ovk })
    }

    /// Derive the Incoming Viewing Key from this FVK
    ///
    /// Uses `CRH_ivk(ak, nk)` hash function as specified in the Zcash protocol.
    ///
    /// ## Returns
    ///
    /// The derived IVK, which can decrypt incoming notes.
    pub fn derive_ivk(&self) -> Result<SaplingIncomingViewingKey> {
        // TODO: Implement CRH_ivk derivation (requires jubjub point operations + blake2b)
        // For now, return an error - this will be implemented in the decryption module
        Err(DecoderError::chain_specific(
            "FVK to IVK derivation not yet implemented (requires jubjub point compression)"
                .to_string(),
        ))
    }
}

// Don't print FVK components (security sensitive)
impl fmt::Debug for SaplingFullViewingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SaplingFullViewingKey([REDACTED])")
    }
}

/// Decrypted Sapling note plaintext
///
/// Contains the actual payment information encrypted in a shielded output.
///
/// ## Structure (580 bytes ciphertext → 564 bytes plaintext)
///
/// ```text
/// [0..1]     - Version byte (0x01 for Sapling)
/// [1..12]    - Diversifier (11 bytes, address component)
/// [12..20]   - Value (8 bytes, little-endian u64 zatoshis)
/// [20..52]   - rcm (32 bytes, note randomness)
/// [52..564]  - Memo (512 bytes, arbitrary data)
/// ```
///
/// ## Privacy Model
///
/// - **Diversifier + IVK → Payment Address**: Reveals where funds were sent
/// - **Value**: Amount in zatoshis (1 ZEC = 10^8 zatoshis)
/// - **Memo**: Arbitrary data (often UTF-8 text, but can be binary)
#[derive(Debug, Clone, PartialEq)]
pub struct NotePlaintext {
    /// Protocol version (always 0x01 for Sapling)
    pub version: u8,

    /// Diversifier (11 bytes) - used to derive payment address
    pub diversifier: [u8; 11],

    /// Note value in zatoshis (1 ZEC = 10^8 zatoshis)
    pub value: u64,

    /// Note randomness commitment (32 bytes)
    pub rcm: [u8; 32],

    /// Memo field (512 bytes) - arbitrary data
    ///
    /// By convention:
    /// - UTF-8 text messages (null-padded)
    /// - Empty: All zeros
    /// - Binary protocols: Custom encoding
    pub memo: [u8; 512],
}

impl NotePlaintext {
    /// Expected version byte for Sapling
    pub const SAPLING_VERSION: u8 = 0x01;

    /// Size of diversifier
    pub const DIVERSIFIER_SIZE: usize = 11;

    /// Size of value field (u64)
    pub const VALUE_SIZE: usize = 8;

    /// Size of note randomness
    pub const RCM_SIZE: usize = 32;

    /// Size of memo field
    pub const MEMO_SIZE: usize = 512;

    /// Total plaintext size (before encryption)
    pub const PLAINTEXT_SIZE: usize = 1 + 11 + 8 + 32 + 512; // 564 bytes

    /// Parse note plaintext from decrypted bytes
    ///
    /// ## Arguments
    ///
    /// - `plaintext`: Decrypted bytes (must be 564 bytes)
    ///
    /// ## Format
    ///
    /// ```text
    /// Offset | Size | Field
    /// -------|------|-------------
    /// 0      | 1    | version
    /// 1      | 11   | diversifier
    /// 12     | 8    | value (LE u64)
    /// 20     | 32   | rcm
    /// 52     | 512  | memo
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns `DecoderError` if:
    /// - Incorrect plaintext size
    /// - Invalid version byte
    pub fn from_bytes(plaintext: &[u8]) -> Result<Self> {
        if plaintext.len() != Self::PLAINTEXT_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Note plaintext must be {} bytes, got {}",
                Self::PLAINTEXT_SIZE,
                plaintext.len()
            )));
        }

        // Parse version
        let version = plaintext[0];
        if version != Self::SAPLING_VERSION {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid Sapling version: expected {}, got {}",
                Self::SAPLING_VERSION,
                version
            )));
        }

        // Parse diversifier (11 bytes)
        let mut diversifier = [0u8; 11];
        diversifier.copy_from_slice(&plaintext[1..12]);

        // Parse value (8 bytes, little-endian)
        let value = u64::from_le_bytes([
            plaintext[12],
            plaintext[13],
            plaintext[14],
            plaintext[15],
            plaintext[16],
            plaintext[17],
            plaintext[18],
            plaintext[19],
        ]);

        // Parse rcm (32 bytes)
        let mut rcm = [0u8; 32];
        rcm.copy_from_slice(&plaintext[20..52]);

        // Parse memo (512 bytes)
        let mut memo = [0u8; 512];
        memo.copy_from_slice(&plaintext[52..564]);

        Ok(Self {
            version,
            diversifier,
            value,
            rcm,
            memo,
        })
    }

    /// Get memo as UTF-8 string (if valid)
    ///
    /// Returns `None` if memo is not valid UTF-8.
    /// Strips trailing null bytes.
    pub fn memo_as_str(&self) -> Option<&str> {
        // Find the first null byte
        let end = self.memo.iter().position(|&b| b == 0).unwrap_or(512);

        // Try to parse as UTF-8
        std::str::from_utf8(&self.memo[..end]).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivk_from_bytes_valid() {
        let ivk_bytes = [0x42; 32];
        let ivk = SaplingIncomingViewingKey::from_bytes(&ivk_bytes).unwrap();
        assert_eq!(ivk.as_bytes(), &ivk_bytes);
    }

    #[test]
    fn test_ivk_from_bytes_invalid_length() {
        let ivk_bytes = [0x42; 31]; // Wrong length
        let result = SaplingIncomingViewingKey::from_bytes(&ivk_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_fvk_from_bytes_valid() {
        let mut fvk_bytes = [0u8; 96];
        fvk_bytes[0..32].copy_from_slice(&[0x01; 32]); // ak
        fvk_bytes[32..64].copy_from_slice(&[0x02; 32]); // nk
        fvk_bytes[64..96].copy_from_slice(&[0x03; 32]); // ovk

        let fvk = SaplingFullViewingKey::from_bytes(&fvk_bytes).unwrap();
        assert_eq!(fvk.ak, [0x01; 32]);
        assert_eq!(fvk.nk, [0x02; 32]);
        assert_eq!(fvk.ovk, [0x03; 32]);
    }

    #[test]
    fn test_note_plaintext_parsing() {
        let mut plaintext = [0u8; 564];
        plaintext[0] = 0x01; // version
        plaintext[1..12].copy_from_slice(&[0xAA; 11]); // diversifier
        plaintext[12..20].copy_from_slice(&123456789u64.to_le_bytes()); // value
        plaintext[20..52].copy_from_slice(&[0xBB; 32]); // rcm
        plaintext[52..564].copy_from_slice(&[0xCC; 512]); // memo

        let note = NotePlaintext::from_bytes(&plaintext).unwrap();
        assert_eq!(note.version, 0x01);
        assert_eq!(note.diversifier, [0xAA; 11]);
        assert_eq!(note.value, 123456789);
        assert_eq!(note.rcm, [0xBB; 32]);
        assert_eq!(note.memo, [0xCC; 512]);
    }

    #[test]
    fn test_note_plaintext_invalid_version() {
        let mut plaintext = [0u8; 564];
        plaintext[0] = 0x99; // Invalid version

        let result = NotePlaintext::from_bytes(&plaintext);
        assert!(result.is_err());
    }

    #[test]
    fn test_note_memo_as_str() {
        let mut plaintext = [0u8; 564];
        plaintext[0] = 0x01;

        // Set memo to "Hello, Zcash!" followed by null bytes
        let memo_text = b"Hello, Zcash!";
        plaintext[52..52 + memo_text.len()].copy_from_slice(memo_text);

        let note = NotePlaintext::from_bytes(&plaintext).unwrap();
        assert_eq!(note.memo_as_str(), Some("Hello, Zcash!"));
    }
}
