//! ZIP-243 Signature Hash (SIGHASH) Computation
//!
//! This module implements the ZIP-243 transaction signature hash algorithm
//! for Sapling transactions, using BLAKE2b-256 instead of Bitcoin's SHA-256d.
//!
//! ## References
//!
//! - **ZIP-243**: https://zips.z.cash/zip-0243
//! - **Zcash Protocol Specification**: Section 4.9 "Transaction Encoding and Consensus"
//!
//! ## SIGHASH Algorithm
//!
//! The signature hash is computed as:
//!
//! ```text
//! BLAKE2b-256 hash with personalization "ZcashSigHash" || consensus_branch_id:
//!   1. header (8 bytes: version || version_group_id)
//!   2. hashPrevouts (32 bytes) - BLAKE2b of all transparent input outpoints
//!   3. hashSequence (32 bytes) - BLAKE2b of all sequence numbers
//!   4. hashOutputs (32 bytes) - BLAKE2b of all transparent outputs
//!   5. hashShieldedSpends (32 bytes) - BLAKE2b of all Spend Descriptions
//!   6. hashShieldedOutputs (32 bytes) - BLAKE2b of all Output Descriptions
//!   7. hashJoinSplits (32 bytes) - BLAKE2b of all JoinSplits (Sprout, usually empty)
//!   8. locktime (4 bytes)
//!   9. expiryHeight (4 bytes)
//!   10. valueBalance (8 bytes) - net shielded value
//!   11. nHashType (4 bytes) - signature hash type (usually SIGHASH_ALL = 0x01)
//! ```
//!
//! ## Hash Type Constants
//!
//! - `SIGHASH_ALL` (0x01): Sign all inputs and outputs
//! - `SIGHASH_NONE` (0x02): Sign all inputs, no outputs
//! - `SIGHASH_SINGLE` (0x03): Sign all inputs, one output
//! - `SIGHASH_ANYONECANPAY` (0x80): Can be combined with above
//!
//! ## Personalization Strings
//!
//! Each hash component uses BLAKE2b with a specific personalization:
//! - `ZcashPrevoutHash` - for prevout outpoints
//! - `ZcashSequencHash` - for sequence numbers
//! - `ZcashOutputsHash` - for transparent outputs
//! - `ZcashSSpendsHash` - for Sapling spends
//! - `ZcashSOutputHash` - for Sapling outputs (note: 15 chars, not 16)
//! - `ZcashJSplitsHash` - for JoinSplits (Sprout)

use crate::sapling::{OutputDescription, SpendDescription};
use crate::types::{SaplingTransaction, TransparentTransaction};
use blake2b_simd::Params as Blake2bParams;
use decoder_encodings::varint::encode_varint;
use decoder_primitives::prelude::*;

/// Consensus branch IDs for different Zcash network upgrades
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
#[allow(clippy::enum_clike_unportable_variant)]
pub enum ConsensusBranchId {
    /// Overwinter (network upgrade 1)
    Overwinter = 0x5ba81b19,
    /// Sapling (network upgrade 2)
    Sapling = 0x76b809bb,
    /// Blossom (network upgrade 3)
    Blossom = 0x2bb40e60,
    /// Heartwood (network upgrade 4)
    Heartwood = 0xf5b9230b,
    /// Canopy (network upgrade 5)
    Canopy = 0xe9ff75a6,
    /// NU5 (network upgrade 6, includes Orchard)
    Nu5 = 0xc2d6d0b4,
}

impl ConsensusBranchId {
    /// Get the branch ID as a 4-byte little-endian array
    pub fn as_bytes(self) -> [u8; 4] {
        (self as u32).to_le_bytes()
    }
}

/// Signature hash type flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigHashType {
    /// Sign all inputs and outputs (default)
    All = 0x01,
    /// Sign all inputs, no outputs
    None = 0x02,
    /// Sign all inputs, one corresponding output
    Single = 0x03,
    /// Modifier: allow others to add inputs
    AnyoneCanPay = 0x80,
}

impl SigHashType {
    /// Get the hash type as a 4-byte little-endian value
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Get the hash type as 4 bytes (little-endian)
    pub fn as_bytes(self) -> [u8; 4] {
        self.as_u32().to_le_bytes()
    }
}

/// Compute the ZIP-243 signature hash for a Sapling transaction
///
/// ## Arguments
///
/// - `tx`: The Sapling transaction to hash
/// - `branch_id`: Consensus branch ID for the network upgrade
/// - `hash_type`: Signature hash type (usually SIGHASH_ALL)
///
/// ## Returns
///
/// 32-byte BLAKE2b-256 signature hash
///
/// ## Example
///
/// ```rust,ignore
/// use decoder_zcash::sighash::{compute_sighash, ConsensusBranchId, SigHashType};
///
/// let sighash = compute_sighash(
///     &sapling_tx,
///     ConsensusBranchId::Sapling,
///     SigHashType::All,
/// )?;
/// ```
pub fn compute_sighash(
    tx: &SaplingTransaction,
    branch_id: ConsensusBranchId,
    hash_type: SigHashType,
) -> Result<[u8; 32]> {
    // Build personalization: "ZcashSigHash" (13 bytes) + branch_id (4 bytes) = 17 bytes
    // Note: BLAKE2b personalization is limited to 16 bytes, so we truncate to 12 + 4
    let mut personalization = [0u8; 16];
    personalization[..12].copy_from_slice(b"ZcashSigHash");
    personalization[12..16].copy_from_slice(&branch_id.as_bytes());

    let mut hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(&personalization)
        .to_state();

    // 1. Header (8 bytes: version || version_group_id)
    hasher.update(&tx.transparent.version.to_le_bytes());
    hasher.update(&tx.transparent.version_group_id.to_le_bytes());

    // 2. hashPrevouts (32 bytes)
    let hash_prevouts = compute_hash_prevouts(&tx.transparent)?;
    hasher.update(&hash_prevouts);

    // 3. hashSequence (32 bytes)
    let hash_sequence = compute_hash_sequence(&tx.transparent)?;
    hasher.update(&hash_sequence);

    // 4. hashOutputs (32 bytes)
    let hash_outputs = compute_hash_outputs(&tx.transparent)?;
    hasher.update(&hash_outputs);

    // 5. hashShieldedSpends (32 bytes)
    let hash_shielded_spends = compute_hash_shielded_spends(&tx.spends)?;
    hasher.update(&hash_shielded_spends);

    // 6. hashShieldedOutputs (32 bytes)
    let hash_shielded_outputs = compute_hash_shielded_outputs(&tx.outputs)?;
    hasher.update(&hash_shielded_outputs);

    // 7. hashJoinSplits (32 bytes) - usually empty for Sapling
    let hash_joinsplits = compute_hash_joinsplits()?;
    hasher.update(&hash_joinsplits);

    // 8. locktime (4 bytes)
    hasher.update(&tx.transparent.locktime.to_le_bytes());

    // 9. expiryHeight (4 bytes)
    hasher.update(&tx.transparent.expiry_height.to_le_bytes());

    // 10. valueBalance (8 bytes)
    hasher.update(&tx.value_balance.to_le_bytes());

    // 11. nHashType (4 bytes)
    hasher.update(&hash_type.as_bytes());

    // Finalize and return
    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    Ok(result)
}

/// Compute hashPrevouts - BLAKE2b hash of all transparent input outpoints
///
/// Personalization: "ZcashPrevoutHash" (16 bytes)
fn compute_hash_prevouts(transparent: &TransparentTransaction) -> Result<[u8; 32]> {
    let personalization = b"ZcashPrevoutHash";

    let mut hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(personalization)
        .to_state();

    // Hash all prevout outpoints (txid || vout)
    for input in &transparent.inputs {
        hasher.update(&input.prev_hash);
        hasher.update(&input.prev_index.to_le_bytes());
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    Ok(result)
}

/// Compute hashSequence - BLAKE2b hash of all sequence numbers
///
/// Personalization: "ZcashSequencHash" (16 bytes)
fn compute_hash_sequence(transparent: &TransparentTransaction) -> Result<[u8; 32]> {
    let personalization = b"ZcashSequencHash";

    let mut hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(personalization)
        .to_state();

    // Hash all sequence numbers (4 bytes each)
    for input in &transparent.inputs {
        hasher.update(&input.sequence.to_le_bytes());
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    Ok(result)
}

/// Compute hashOutputs - BLAKE2b hash of all transparent outputs
///
/// Personalization: "ZcashOutputsHash" (16 bytes)
fn compute_hash_outputs(transparent: &TransparentTransaction) -> Result<[u8; 32]> {
    let personalization = b"ZcashOutputsHash";

    let mut hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(personalization)
        .to_state();

    // Hash all outputs (value || script_pubkey)
    for output in &transparent.outputs {
        hasher.update(&output.value.to_le_bytes());

        // Script is var_int(len) || bytes
        let mut script_len_bytes = Vec::new();
        encode_varint(&mut script_len_bytes, output.script_pubkey.len() as u64);
        hasher.update(&script_len_bytes);
        hasher.update(&output.script_pubkey);
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    Ok(result)
}

/// Compute hashShieldedSpends - BLAKE2b hash of all Sapling Spend Descriptions
///
/// Personalization: "ZcashSSpendsHash" (16 bytes)
fn compute_hash_shielded_spends(spends: &[SpendDescription]) -> Result<[u8; 32]> {
    let personalization = b"ZcashSSpendsHash";

    let mut hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(personalization)
        .to_state();

    // Hash all spend descriptions (cv || anchor || nullifier || rk || zkproof)
    // Note: spend_auth_sig is NOT included in SIGHASH
    for spend in spends {
        hasher.update(&spend.cv);
        hasher.update(&spend.anchor);
        hasher.update(&spend.nullifier);
        hasher.update(&spend.rk);
        hasher.update(&spend.zkproof);
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    Ok(result)
}

/// Compute hashShieldedOutputs - BLAKE2b hash of all Sapling Output Descriptions
///
/// Personalization: "ZcashSOutputHash" (15 bytes, padded to 16 with \0)
///
/// **Important**: The Zcash specification uses "ZcashSOutputHash" (15 bytes),
/// not 16 bytes like other personalizations. BLAKE2b will pad with zeros.
fn compute_hash_shielded_outputs(outputs: &[OutputDescription]) -> Result<[u8; 32]> {
    // Note: "ZcashSOutputHash" is 15 bytes, BLAKE2b will pad to 16
    let personalization = b"ZcashSOutputHash";

    let mut hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(personalization)
        .to_state();

    // Hash all output descriptions (cv || cmu || ephemeral_key || enc_ciphertext || out_ciphertext || zkproof)
    for output in outputs {
        hasher.update(&output.cv);
        hasher.update(&output.cmu);
        hasher.update(&output.ephemeral_key);
        hasher.update(&output.enc_ciphertext);
        hasher.update(&output.out_ciphertext);
        hasher.update(&output.zkproof);
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    Ok(result)
}

/// Compute hashJoinSplits - BLAKE2b hash of all JoinSplits (Sprout)
///
/// Personalization: "ZcashJSplitsHash" (16 bytes)
///
/// For Sapling-only transactions, this is always the hash of empty input.
fn compute_hash_joinsplits() -> Result<[u8; 32]> {
    let personalization = b"ZcashJSplitsHash";

    let hasher = Blake2bParams::new()
        .hash_length(32)
        .personal(personalization)
        .to_state();

    // No JoinSplits in pure Sapling transactions
    // Hash of empty input
    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sighash_type_values() {
        assert_eq!(SigHashType::All.as_u32(), 0x01);
        assert_eq!(SigHashType::None.as_u32(), 0x02);
        assert_eq!(SigHashType::Single.as_u32(), 0x03);
        assert_eq!(SigHashType::AnyoneCanPay.as_u32(), 0x80);
    }

    #[test]
    fn test_consensus_branch_ids() {
        assert_eq!(
            ConsensusBranchId::Sapling.as_bytes(),
            [0xbb, 0x09, 0xb8, 0x76]
        );
    }

    #[test]
    fn test_hash_prevouts_empty() {
        // Test with empty transparent transaction
        let transparent = TransparentTransaction {
            version: 0x80000004,
            version_group_id: 0x892f2085,
            inputs: vec![],
            outputs: vec![],
            locktime: 0,
            expiry_height: 0,
            is_segwit: false,
            witnesses: None,
            raw_bytes: Vec::new(),
        };

        let hash = compute_hash_prevouts(&transparent).unwrap();

        // Expected hash from ZIP-243 for empty prevouts
        let expected_hex = "d53a633bbecf82fe9e9484d8a0e727c73bb9e68c96e72dec30144f6a84afa136";
        let expected: [u8; 32] = universal_decoder_core::hex::decode(expected_hex)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(
            hash, expected,
            "hashPrevouts for empty inputs should match ZIP-243 test vector"
        );
    }

    #[test]
    fn test_personalization_lengths() {
        // Verify all personalizations are ≤ 16 bytes (BLAKE2b limit)
        assert!(b"ZcashPrevoutHash".len() == 16);
        assert!(b"ZcashSequencHash".len() == 16);
        assert!(b"ZcashOutputsHash".len() == 16);
        assert!(b"ZcashSSpendsHash".len() == 16);
        assert!(b"ZcashSOutputHash".len() == 16); // Note: 16 bytes, but spec says 15
        assert!(b"ZcashJSplitsHash".len() == 16);
    }
}
