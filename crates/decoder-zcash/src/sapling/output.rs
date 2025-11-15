//! Sapling OutputDescription parsing
//!
//! An OutputDescription represents the creation of a new shielded note in a Sapling transaction.
//! It contains:
//! - Note commitment (commitment to recipient and amount)
//! - Encrypted ciphertext (note details, encrypted to recipient)
//! - Ephemeral key (for ECDH key agreement)
//! - zk-SNARK proof (proves output correctness)

use decoder_primitives::prelude::*;
use std::io::Cursor;

use super::read_fixed_bytes;

/// Sapling output description (creates a shielded note)
///
/// Binary format (total: 948 bytes):
/// ```text
/// [cv]              32 bytes  - Value commitment (compressed point on Jubjub)
/// [cmu]             32 bytes  - Note commitment (to recipient)
/// [ephemeral_key]   32 bytes  - Ephemeral public key (for ECDH)
/// [enc_ciphertext] 580 bytes  - Encrypted note (ChaCha20-Poly1305)
/// [out_ciphertext]  80 bytes  - Outgoing cipher (for sender recovery)
/// [zkproof]        192 bytes  - Groth16 zk-SNARK proof
/// ```
///
/// ## Privacy Features
///
/// - **Note Commitment (cmu)**: Binding commitment to (recipient, value, randomness)
/// - **Encrypted Ciphertext**: Only recipient with viewing key can decrypt
/// - **Ephemeral Key**: One-time ECDH key for encryption (not linkable)
/// - **zk-SNARK**: Proves output correctness without revealing details
///
/// ## Encryption Scheme
///
/// The note is encrypted using **ChaCha20-Poly1305 AEAD**:
/// 1. Sender derives shared secret via ECDH with ephemeral key
/// 2. ChaCha20 encrypts note plaintext (recipient, value, memo)
/// 3. Poly1305 authenticates ciphertext
/// 4. Only recipient with correct viewing key can decrypt
///
/// ## Security Notes
///
/// This parser **does not decrypt** or **verify proofs**. It only extracts binary data.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputDescription {
    /// Value commitment (32 bytes, compressed Jubjub point)
    ///
    /// Homomorphic commitment to the note value:
    /// `cv = v * G_value + rcv * G_randomness`
    ///
    /// Allows proving value balance without revealing amounts.
    pub cv: [u8; 32],

    /// Note commitment (32 bytes)
    ///
    /// Commitment to the note being created:
    /// `cm = COMM_rcm(recipient || value)`
    ///
    /// Published on-chain but does not reveal recipient or amount.
    pub cmu: [u8; 32],

    /// Ephemeral public key (32 bytes)
    ///
    /// One-time ECDH public key for encryption:
    /// `epk = esk * G`
    ///
    /// Used for key agreement: recipient derives shared secret as `esk * ivk * G`.
    pub ephemeral_key: [u8; 32],

    /// Encrypted note ciphertext (580 bytes)
    ///
    /// ChaCha20-Poly1305 encrypted note containing:
    /// - Recipient payment address (32 bytes)
    /// - Value (8 bytes)
    /// - Randomness (32 bytes)
    /// - Memo field (512 bytes) - arbitrary data
    /// - Poly1305 MAC (16 bytes)
    ///
    /// **Decryption requires**: Sapling incoming viewing key (ivk)
    pub enc_ciphertext: [u8; 580],

    /// Outgoing ciphertext (80 bytes)
    ///
    /// Encrypted copy for sender recovery:
    /// - Contains (recipient, value, memo key)
    /// - Encrypted to sender's outgoing viewing key (ovk)
    /// - Allows sender to see sent transactions in wallet
    pub out_ciphertext: [u8; 80],

    /// zk-SNARK proof (192 bytes, Groth16)
    ///
    /// Proves:
    /// 1. Correct note commitment derivation
    /// 2. Correct value commitment
    /// 3. Correct ciphertext encryption
    ///
    /// **Not verified by this parser** - verification requires BLS12-381 pairing.
    pub zkproof: [u8; 192],
}

impl OutputDescription {
    /// Total size of an OutputDescription in bytes
    pub const SIZE: usize = 32 + 32 + 32 + 580 + 80 + 192; // 948 bytes

    /// Size of value commitment (cv)
    pub const CV_SIZE: usize = 32;

    /// Size of note commitment (cmu)
    pub const CMU_SIZE: usize = 32;

    /// Size of ephemeral key
    pub const EPHEMERAL_KEY_SIZE: usize = 32;

    /// Size of encrypted note ciphertext
    pub const ENC_CIPHERTEXT_SIZE: usize = 580;

    /// Size of outgoing ciphertext
    pub const OUT_CIPHERTEXT_SIZE: usize = 80;

    /// Size of zk-SNARK proof (Groth16)
    pub const ZKPROOF_SIZE: usize = 192;

    /// Attempt to decrypt this output with an incoming viewing key
    ///
    /// ## Arguments
    ///
    /// - `ivk`: Incoming viewing key to try decryption with
    ///
    /// ## Returns
    ///
    /// - `Ok(Some(plaintext))`: Successfully decrypted (note is addressed to this IVK)
    /// - `Ok(None)`: Decryption failed (note is NOT addressed to this IVK, not an error)
    /// - `Err(...)`: Malformed ciphertext or cryptographic error
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// use decoder_zcash::sapling::OutputDescription;
    /// use decoder_zcash::viewing_key::SaplingIncomingViewingKey;
    ///
    /// let output: OutputDescription = /* parsed from transaction */;
    /// let ivk = SaplingIncomingViewingKey::from_bytes(&ivk_bytes)?;
    ///
    /// match output.try_decrypt(&ivk)? {
    ///     Some(plaintext) => {
    ///         println!("Received {} zatoshis", plaintext.value);
    ///         if let Some(memo) = plaintext.memo_as_str() {
    ///             println!("Memo: {}", memo);
    ///         }
    ///     }
    ///     None => println!("Not addressed to this viewing key"),
    /// }
    /// ```
    pub fn try_decrypt(
        &self,
        ivk: &crate::viewing_key::SaplingIncomingViewingKey,
    ) -> Result<Option<crate::viewing_key::NotePlaintext>> {
        crate::viewing_key::decrypt_sapling_note(&self.ephemeral_key, &self.enc_ciphertext, ivk)
    }
}

/// Parse a Sapling OutputDescription from binary data
///
/// ## Format
///
/// ```text
/// Offset  | Size  | Field
/// --------|-------|------------------
/// 0       | 32    | cv (value commitment)
/// 32      | 32    | cmu (note commitment)
/// 64      | 32    | ephemeral_key
/// 96      | 580   | enc_ciphertext
/// 676     | 80    | out_ciphertext
/// 756     | 192   | zkproof (Groth16)
/// ```
///
/// ## Example
///
/// ```rust,ignore
/// use decoder_zcash::sapling::parse_output_description;
/// use std::io::Cursor;
///
/// let output_bytes: &[u8] = &[/* 948 bytes */];
/// let mut cursor = Cursor::new(output_bytes);
///
/// let output = parse_output_description(&mut cursor)?;
/// assert_eq!(output.enc_ciphertext.len(), 580);
/// assert_eq!(output.zkproof.len(), 192);
/// ```
///
/// ## Errors
///
/// Returns `DecoderError` if:
/// - Insufficient bytes (< 948 bytes)
/// - I/O error during read
pub fn parse_output_description(cursor: &mut Cursor<&[u8]>) -> Result<OutputDescription> {
    // Read value commitment (32 bytes)
    let cv = read_fixed_bytes::<32>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read cv: {}", e)))?;

    // Read note commitment (32 bytes) - CRITICAL: binds value to recipient
    let cmu = read_fixed_bytes::<32>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read cmu: {}", e)))?;

    // Read ephemeral public key (32 bytes) - CRITICAL: for ECDH decryption
    let ephemeral_key = read_fixed_bytes::<32>(cursor).map_err(|e| {
        DecoderError::chain_decoding(format!("Failed to read ephemeral_key: {}", e))
    })?;

    // Read encrypted note ciphertext (580 bytes)
    let enc_ciphertext = read_fixed_bytes::<580>(cursor).map_err(|e| {
        DecoderError::chain_decoding(format!("Failed to read enc_ciphertext: {}", e))
    })?;

    // Read outgoing ciphertext (80 bytes) - for sender recovery
    let out_ciphertext = read_fixed_bytes::<80>(cursor).map_err(|e| {
        DecoderError::chain_decoding(format!("Failed to read out_ciphertext: {}", e))
    })?;

    // Read zk-SNARK proof (192 bytes, Groth16)
    let zkproof = read_fixed_bytes::<192>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read zkproof: {}", e)))?;

    Ok(OutputDescription {
        cv,
        cmu,
        ephemeral_key,
        enc_ciphertext,
        out_ciphertext,
        zkproof,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_description_size() {
        assert_eq!(OutputDescription::SIZE, 948);
        assert_eq!(
            OutputDescription::SIZE,
            OutputDescription::CV_SIZE
                + OutputDescription::CMU_SIZE
                + OutputDescription::EPHEMERAL_KEY_SIZE
                + OutputDescription::ENC_CIPHERTEXT_SIZE
                + OutputDescription::OUT_CIPHERTEXT_SIZE
                + OutputDescription::ZKPROOF_SIZE
        );
    }

    #[test]
    fn test_parse_output_description_valid() {
        // Create 948 bytes of test data
        let mut bytes = Vec::with_capacity(948);
        bytes.extend_from_slice(&[0x01; 32]); // cv
        bytes.extend_from_slice(&[0x02; 32]); // cmu
        bytes.extend_from_slice(&[0x03; 32]); // ephemeral_key
        bytes.extend_from_slice(&[0x04; 580]); // enc_ciphertext
        bytes.extend_from_slice(&[0x05; 80]); // out_ciphertext
        bytes.extend_from_slice(&[0x06; 192]); // zkproof

        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_output_description(&mut cursor);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.cv, [0x01; 32]);
        assert_eq!(output.cmu, [0x02; 32]);
        assert_eq!(output.ephemeral_key, [0x03; 32]);
        assert_eq!(output.enc_ciphertext, [0x04; 580]);
        assert_eq!(output.out_ciphertext, [0x05; 80]);
        assert_eq!(output.zkproof, [0x06; 192]);
    }

    #[test]
    fn test_parse_output_description_insufficient_bytes() {
        // Only 100 bytes (need 948)
        let bytes = [0x42; 100];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_output_description(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_output_description_partial_read() {
        // 800 bytes (missing zkproof)
        let bytes = [0x42; 800];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_output_description(&mut cursor);
        assert!(result.is_err());
    }
}
