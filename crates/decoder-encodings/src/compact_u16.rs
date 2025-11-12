//! Compact-u16 encoding utilities
//!
//! Solana-style variable-length encoding for 16-bit integers.

use std::io::{Cursor, Read};
use universal_decoder_core::prelude::DecoderError;

type Result<T> = std::result::Result<T, DecoderError>;

/// Decode a compact-u16 value
///
/// Compact-u16 is Solana's variable-length encoding for 16-bit integers:
/// - If first byte is 0-127 (high bit = 0): value is that byte
/// - If first byte is 128-255 (high bit = 1):
///   - Remove high bit from first byte
///   - Second byte provides upper 8 bits
///   - Result = (second_byte << 7) | (first_byte & 0x7F)
///
/// Examples:
/// - 0x00 -> 0
/// - 0x7F -> 127
/// - 0x80 0x01 -> 128 (0x80 & 0x7F = 0, (1 << 7) | 0 = 128)
/// - 0xFF 0x7F -> 16383 (0xFF & 0x7F = 127, (127 << 7) | 127 = 16383)
pub fn read_compact_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read compact-u16: {}", e)))?;

    let first_byte = buf[0];

    // If high bit is not set, this is a single-byte value
    if first_byte & 0x80 == 0 {
        return Ok(first_byte as u16);
    }

    // High bit is set, need to read second byte
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read second byte of compact-u16: {}", e)))?;

    let second_byte = buf[0];

    // Combine: (second_byte << 7) | (first_byte & 0x7F)
    let value = ((second_byte as u16) << 7) | ((first_byte & 0x7F) as u16);

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_u16_single_byte() {
        // Test values 0-127 (single byte)
        for i in 0..=127u8 {
            let data = vec![i];
            let mut cursor = Cursor::new(data.as_slice());
            let result = read_compact_u16(&mut cursor).unwrap();
            assert_eq!(result, i as u16, "Failed for value {}", i);
            assert_eq!(cursor.position(), 1, "Should consume 1 byte for {}", i);
        }
    }

    #[test]
    fn test_compact_u16_two_bytes() {
        // Test 128: 0x80 0x01
        let data = vec![0x80, 0x01];
        let mut cursor = Cursor::new(data.as_slice());
        let result = read_compact_u16(&mut cursor).unwrap();
        assert_eq!(result, 128);
        assert_eq!(cursor.position(), 2);

        // Test 255: 0xFF 0x01
        let data = vec![0xFF, 0x01];
        let mut cursor = Cursor::new(data.as_slice());
        let result = read_compact_u16(&mut cursor).unwrap();
        assert_eq!(result, 255); // (1 << 7) | 127 = 128 + 127 = 255

        // Test 256: 0x80 0x02
        let data = vec![0x80, 0x02];
        let mut cursor = Cursor::new(data.as_slice());
        let result = read_compact_u16(&mut cursor).unwrap();
        assert_eq!(result, 256); // (2 << 7) | 0 = 256

        // Test maximum value: 0xFF 0x7F = 16383
        let data = vec![0xFF, 0x7F];
        let mut cursor = Cursor::new(data.as_slice());
        let result = read_compact_u16(&mut cursor).unwrap();
        assert_eq!(result, 16383); // (127 << 7) | 127 = 16256 + 127 = 16383
    }

    #[test]
    fn test_compact_u16_empty() {
        let data = vec![];
        let mut cursor = Cursor::new(data.as_slice());
        assert!(read_compact_u16(&mut cursor).is_err());
    }

    #[test]
    fn test_compact_u16_truncated() {
        // High bit set but missing second byte
        let data = vec![0x80];
        let mut cursor = Cursor::new(data.as_slice());
        assert!(read_compact_u16(&mut cursor).is_err());
    }
}
