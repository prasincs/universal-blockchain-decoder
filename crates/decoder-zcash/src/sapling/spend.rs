//! Sapling SpendDescription parsing
//!
//! A SpendDescription represents the consumption of a shielded note in a Sapling transaction.
//! It contains:
//! - Nullifier (prevents double-spending)
//! - Value commitment (homomorphic commitment to amount)
//! - zk-SNARK proof (proves spending authorization)
//! - Signature (authorizes the spend)

use decoder_primitives::prelude::*;
use std::io::Cursor;

use super::read_fixed_bytes;

/// Sapling spend description (consumes a shielded note)
///
/// Binary format (total: 384 bytes):
/// ```text
/// [cv]               32 bytes  - Value commitment (compressed point on Jubjub)
/// [anchor]           32 bytes  - Merkle tree root (note commitment tree)
/// [nullifier]        32 bytes  - Unique nullifier (prevents double-spend)
/// [rk]               32 bytes  - Randomized verification key
/// [zkproof]         192 bytes  - Groth16 zk-SNARK proof
/// [spend_auth_sig]   64 bytes  - Signature over transaction sighash
/// ```
///
/// ## Privacy Features
///
/// - **Nullifier**: Unique per note, reveals when spent but not who spent it
/// - **Value Commitment**: Homomorphic, allows sum verification without revealing amount
/// - **zk-SNARK**: Proves spending authorization without revealing private key
///
/// ## Security Notes
///
/// This parser **does not verify** zk-SNARK proofs. It only extracts the binary data.
/// For consensus validation, use full node software with proof verification.
#[derive(Debug, Clone, PartialEq)]
pub struct SpendDescription {
    /// Value commitment (32 bytes, compressed Jubjub point)
    ///
    /// Homomorphic commitment to the note value:
    /// `cv = v * G_value + rcv * G_randomness`
    ///
    /// Allows proving value balance without revealing amounts.
    pub cv: [u8; 32],

    /// Merkle tree anchor (32 bytes)
    ///
    /// Root hash of the note commitment tree at the time of spending.
    /// Proves that the note being spent exists in a valid state.
    pub anchor: [u8; 32],

    /// Nullifier (32 bytes)
    ///
    /// Unique identifier that prevents double-spending:
    /// `nf = PRF_nf(nk, rho)`
    ///
    /// Once revealed, this note cannot be spent again.
    /// **Critical for privacy**: Cannot be linked to the original note commitment.
    pub nullifier: [u8; 32],

    /// Randomized verification key (32 bytes)
    ///
    /// Re-randomized public key for this spend:
    /// `rk = ak + alpha * G`
    ///
    /// Prevents key linkability across transactions.
    pub rk: [u8; 32],

    /// zk-SNARK proof (192 bytes, Groth16)
    ///
    /// Proves:
    /// 1. Knowledge of note (value, recipient, randomness)
    /// 2. Merkle path from note to anchor
    /// 3. Correct nullifier derivation
    /// 4. Correct value commitment
    ///
    /// **Not verified by this parser** - verification requires BLS12-381 pairing.
    pub zkproof: [u8; 192],

    /// Spend authorization signature (64 bytes)
    ///
    /// Signature over the transaction sighash using spend authorization key:
    /// `sig = Sign(ask, sighash)`
    ///
    /// Authorizes this specific spend in this specific transaction.
    pub spend_auth_sig: [u8; 64],
}

impl SpendDescription {
    /// Total size of a SpendDescription in bytes
    pub const SIZE: usize = 32 + 32 + 32 + 32 + 192 + 64; // 384 bytes

    /// Size of value commitment (cv)
    pub const CV_SIZE: usize = 32;

    /// Size of Merkle anchor
    pub const ANCHOR_SIZE: usize = 32;

    /// Size of nullifier
    pub const NULLIFIER_SIZE: usize = 32;

    /// Size of randomized key (rk)
    pub const RK_SIZE: usize = 32;

    /// Size of zk-SNARK proof (Groth16)
    pub const ZKPROOF_SIZE: usize = 192;

    /// Size of spend authorization signature
    pub const SPEND_AUTH_SIG_SIZE: usize = 64;
}

/// Parse a Sapling SpendDescription from binary data
///
/// ## Format
///
/// ```text
/// Offset  | Size  | Field
/// --------|-------|------------------
/// 0       | 32    | cv (value commitment)
/// 32      | 32    | anchor (Merkle root)
/// 64      | 32    | nullifier
/// 96      | 32    | rk (randomized key)
/// 128     | 192   | zkproof (Groth16)
/// 320     | 64    | spend_auth_sig
/// ```
///
/// ## Example
///
/// ```rust,ignore
/// use decoder_zcash::sapling::parse_spend_description;
/// use std::io::Cursor;
///
/// let spend_bytes: &[u8] = &[/* 384 bytes */];
/// let mut cursor = Cursor::new(spend_bytes);
///
/// let spend = parse_spend_description(&mut cursor)?;
/// assert_eq!(spend.nullifier.len(), 32);
/// assert_eq!(spend.zkproof.len(), 192);
/// ```
///
/// ## Errors
///
/// Returns `DecoderError` if:
/// - Insufficient bytes (< 384 bytes)
/// - I/O error during read
pub fn parse_spend_description(cursor: &mut Cursor<&[u8]>) -> Result<SpendDescription> {
    // Read value commitment (32 bytes)
    let cv = read_fixed_bytes::<32>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read cv: {}", e)))?;

    // Read Merkle anchor (32 bytes)
    let anchor = read_fixed_bytes::<32>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read anchor: {}", e)))?;

    // Read nullifier (32 bytes) - CRITICAL for double-spend prevention
    let nullifier = read_fixed_bytes::<32>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read nullifier: {}", e)))?;

    // Read randomized key (32 bytes)
    let rk = read_fixed_bytes::<32>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read rk: {}", e)))?;

    // Read zk-SNARK proof (192 bytes, Groth16)
    let zkproof = read_fixed_bytes::<192>(cursor)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read zkproof: {}", e)))?;

    // Read spend authorization signature (64 bytes)
    let spend_auth_sig = read_fixed_bytes::<64>(cursor).map_err(|e| {
        DecoderError::chain_decoding(format!("Failed to read spend_auth_sig: {}", e))
    })?;

    Ok(SpendDescription {
        cv,
        anchor,
        nullifier,
        rk,
        zkproof,
        spend_auth_sig,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spend_description_size() {
        assert_eq!(SpendDescription::SIZE, 384);
        assert_eq!(
            SpendDescription::SIZE,
            SpendDescription::CV_SIZE
                + SpendDescription::ANCHOR_SIZE
                + SpendDescription::NULLIFIER_SIZE
                + SpendDescription::RK_SIZE
                + SpendDescription::ZKPROOF_SIZE
                + SpendDescription::SPEND_AUTH_SIG_SIZE
        );
    }

    #[test]
    fn test_parse_spend_description_valid() {
        // Create 384 bytes of test data
        let mut bytes = Vec::with_capacity(384);
        bytes.extend_from_slice(&[0x01; 32]); // cv
        bytes.extend_from_slice(&[0x02; 32]); // anchor
        bytes.extend_from_slice(&[0x03; 32]); // nullifier
        bytes.extend_from_slice(&[0x04; 32]); // rk
        bytes.extend_from_slice(&[0x05; 192]); // zkproof
        bytes.extend_from_slice(&[0x06; 64]); // spend_auth_sig

        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_spend_description(&mut cursor);
        assert!(result.is_ok());

        let spend = result.unwrap();
        assert_eq!(spend.cv, [0x01; 32]);
        assert_eq!(spend.anchor, [0x02; 32]);
        assert_eq!(spend.nullifier, [0x03; 32]);
        assert_eq!(spend.rk, [0x04; 32]);
        assert_eq!(spend.zkproof, [0x05; 192]);
        assert_eq!(spend.spend_auth_sig, [0x06; 64]);
    }

    #[test]
    fn test_parse_spend_description_insufficient_bytes() {
        // Only 100 bytes (need 384)
        let bytes = [0x42; 100];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_spend_description(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_spend_description_partial_read() {
        // 300 bytes (missing spend_auth_sig)
        let bytes = [0x42; 300];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_spend_description(&mut cursor);
        assert!(result.is_err());
    }
}
