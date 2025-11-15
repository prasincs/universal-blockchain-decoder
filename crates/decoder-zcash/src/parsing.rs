//! Zcash transaction parsing utilities
//!
//! This module provides low-level parsing functions for Zcash transactions.

use decoder_bitcoin::parsing::read_varint;
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

use crate::transparent::parse_transparent_transaction;
use crate::types::*;

/// Parse Zcash transaction header (version + version_group_id)
///
/// Zcash transactions have a 4-byte header with:
/// - Bit 31: Overwinter flag (1 for Sapling+)
/// - Bits 0-30: Version number
///
/// If Overwinter flag is set, next 4 bytes are `version_group_id`.
///
/// Returns: (version, version_group_id)
pub fn parse_zcash_header(cursor: &mut Cursor<&[u8]>) -> Result<(u32, u32)> {
    // Read 4-byte header
    let header = read_u32_le(cursor)?;

    // Extract Overwinter bit (bit 31)
    let is_overwinter = (header & 0x8000_0000) != 0;

    // Extract version (bits 0-30)
    let version = header & 0x7FFF_FFFF;

    // Read version_group_id if Overwinter
    let version_group_id = if is_overwinter {
        read_u32_le(cursor)?
    } else {
        0x0000_0000 // Pre-Overwinter has no version_group_id
    };

    Ok((version, version_group_id))
}

/// Parse Zcash v4 transaction (Sapling)
///
/// Phase 1: Only transparent component
/// Phase 2+: Full Sapling support (shielded spends/outputs)
pub fn parse_zcash_v4_transaction(
    cursor: &mut Cursor<&[u8]>,
    version: u32,
    version_group_id: u32,
) -> Result<ZcashTransaction> {
    // Validate version_group_id for Sapling
    match version_group_id {
        0x892F2085 => {} // Sapling
        0x26A7270A => {} // Blossom
        0xF919A198 => {} // Heartwood
        0xC2D6D0B4 => {} // Canopy
        0x00000000 => {
            // Pre-Overwinter (should not happen with v4)
            return Err(DecoderError::invalid_structure(
                "Version 4 transaction with no version_group_id (invalid)".to_string(),
            ));
        }
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Unknown version_group_id: 0x{:08X}",
                version_group_id
            )));
        }
    }

    // Parse transparent component (reuses Bitcoin decoder logic)
    let transparent = parse_transparent_transaction(cursor, version, version_group_id)?;

    // Phase 1: Check for Sapling components
    // In Phase 2+, we will parse:
    // - sapling_spends (varint count + spend descriptions)
    // - sapling_outputs (varint count + output descriptions)
    // - value_balance (i64)
    // - binding_sig (64 bytes)
    //
    // For now, we only support pure transparent transactions.
    // If there are Sapling components, return an error.

    let sapling_spends_count = read_varint(cursor)?;
    if sapling_spends_count > 0 {
        return Err(DecoderError::chain_specific(format!(
            "Sapling shielded transactions not yet supported (Phase 2 feature). Found {} spends.",
            sapling_spends_count
        )));
    }

    let sapling_outputs_count = read_varint(cursor)?;
    if sapling_outputs_count > 0 {
        return Err(DecoderError::chain_specific(format!(
            "Sapling shielded transactions not yet supported (Phase 2 feature). Found {} outputs.",
            sapling_outputs_count
        )));
    }

    // Phase 1: No Sapling components, pure transparent transaction
    Ok(ZcashTransaction::Transparent(transparent))
}

/// Read u8 from cursor
pub fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

/// Read u32 (little-endian) from cursor
pub fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zcash_header_sapling() {
        // Version 4 with Overwinter bit + Sapling version_group_id
        let bytes = [
            0x04, 0x00, 0x00, 0x80, // Version 4 with Overwinter bit (0x80000004)
            0x85, 0x20, 0x2F, 0x89, // version_group_id: Sapling (0x892F2085)
        ];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_zcash_header(&mut cursor);
        assert!(result.is_ok());

        let (version, vgi) = result.unwrap();
        assert_eq!(version, 4);
        assert_eq!(vgi, 0x892F2085);
    }

    #[test]
    fn test_parse_zcash_header_pre_overwinter() {
        // Version 1 without Overwinter bit
        let bytes = [0x01, 0x00, 0x00, 0x00]; // Version 1
        let mut cursor = Cursor::new(&bytes[..]);

        let result = parse_zcash_header(&mut cursor);
        assert!(result.is_ok());

        let (version, vgi) = result.unwrap();
        assert_eq!(version, 1);
        assert_eq!(vgi, 0);
    }

    #[test]
    fn test_read_u32_le() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = read_u32_le(&mut cursor).unwrap();
        assert_eq!(result, 0x04030201); // Little-endian
    }
}
