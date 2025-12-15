//! Transparent transaction parsing (Bitcoin-compatible)
//!
//! This module handles parsing of Zcash transparent transactions,
//! which are structurally identical to Bitcoin transactions with
//! additional Zcash-specific fields.

use decoder_bitcoin::parsing::{parse_input, parse_output, read_varint, MAX_INPUTS_OUTPUTS};
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

use crate::parsing::{read_u32_le, read_u8};
use crate::types::TransparentTransaction;

/// Parse transparent Zcash transaction
///
/// This function parses the transparent component of a Zcash transaction,
/// which is Bitcoin-compatible but includes additional fields:
/// - `version_group_id` (already parsed in header)
/// - `expiry_height` (4 bytes after locktime)
///
/// Input format (after header):
/// - \[transparent inputs\] (varint count + inputs)
/// - \[transparent outputs\] (varint count + outputs)
/// - \[locktime\] (4 bytes)
/// - \[expiry_height\] (4 bytes, Zcash-specific)
/// - \[optional: joinsplit data\] (Phase 2+)
pub fn parse_transparent_transaction(
    cursor: &mut Cursor<&[u8]>,
    version: u32,
    version_group_id: u32,
    raw_bytes: &[u8],
) -> Result<TransparentTransaction> {
    // Detect SegWit (marker 0x00, flag 0x01)
    let is_segwit = detect_segwit_zcash(cursor)?;

    // Skip marker and flag if SegWit
    if is_segwit {
        let _marker = read_u8(cursor)?;
        let _flag = read_u8(cursor)?;
    }

    // Parse input count
    let input_count = read_varint(cursor)?;
    if input_count > MAX_INPUTS_OUTPUTS as u64 {
        return Err(DecoderError::invalid_structure(format!(
            "Too many inputs: {}",
            input_count
        )));
    }

    // Parse inputs (reuse Bitcoin parser)
    let mut inputs = Vec::with_capacity(input_count as usize);
    for i in 0..input_count {
        inputs.push(parse_input(cursor).map_err(|e| {
            DecoderError::chain_decoding(format!("Failed to parse input {}: {}", i, e))
        })?);
    }

    // Parse output count
    let output_count = read_varint(cursor)?;
    if output_count > MAX_INPUTS_OUTPUTS as u64 {
        return Err(DecoderError::invalid_structure(format!(
            "Too many outputs: {}",
            output_count
        )));
    }

    // Parse outputs (reuse Bitcoin parser)
    let mut outputs = Vec::with_capacity(output_count as usize);
    for i in 0..output_count {
        outputs.push(parse_output(cursor).map_err(|e| {
            DecoderError::chain_decoding(format!("Failed to parse output {}: {}", i, e))
        })?);
    }

    // Parse witnesses if SegWit
    let witnesses = if is_segwit {
        let mut witness_data = Vec::with_capacity(inputs.len());
        for i in 0..inputs.len() {
            let witness_count = read_varint(cursor)?;
            let mut witness_items = Vec::with_capacity(witness_count as usize);
            for _ in 0..witness_count {
                let item_len = read_varint(cursor)?;
                let mut item = vec![0u8; item_len as usize];
                cursor.read_exact(&mut item).map_err(|e| {
                    DecoderError::chain_decoding(format!(
                        "Failed to read witness item for input {}: {}",
                        i, e
                    ))
                })?;
                witness_items.push(item);
            }
            witness_data.push(witness_items);
        }
        Some(witness_data)
    } else {
        None
    };

    // Parse locktime (4 bytes)
    let locktime = read_u32_le(cursor)?;

    // Parse expiry_height (4 bytes, Zcash-specific)
    let expiry_height = read_u32_le(cursor)?;

    Ok(TransparentTransaction {
        version,
        version_group_id,
        inputs,
        outputs,
        locktime,
        expiry_height,
        is_segwit,
        witnesses,
        raw_bytes: raw_bytes.to_vec(),
    })
}

/// Detect SegWit in Zcash transaction
///
/// SegWit detection in Zcash is the same as Bitcoin:
/// - Marker: 0x00 (at position after version)
/// - Flag: 0x01 (immediately after marker)
fn detect_segwit_zcash(cursor: &mut Cursor<&[u8]>) -> Result<bool> {
    let pos = cursor.position() as usize;
    let data = cursor.get_ref();

    // Check if there are at least 2 bytes for marker + flag
    if pos + 2 > data.len() {
        return Ok(false);
    }

    // SegWit: marker=0x00, flag=0x01
    let is_segwit = data[pos] == 0x00 && data[pos + 1] == 0x01;

    Ok(is_segwit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_segwit_zcash_true() {
        let bytes = [0x00, 0x01, 0x02, 0x03];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = detect_segwit_zcash(&mut cursor).unwrap();
        assert!(result);
    }

    #[test]
    fn test_detect_segwit_zcash_false() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = detect_segwit_zcash(&mut cursor).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_detect_segwit_zcash_insufficient_bytes() {
        let bytes = [0x00]; // Only 1 byte
        let mut cursor = Cursor::new(&bytes[..]);

        let result = detect_segwit_zcash(&mut cursor).unwrap();
        assert!(!result); // Not enough bytes for SegWit
    }
}
