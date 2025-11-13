//! BCS (Binary Canonical Serialization) encoding utilities
//!
//! BCS is the canonical serialization format used by Move-based blockchains
//! including Aptos and Sui. It provides deterministic encoding with:
//! - Little-endian byte order
//! - ULEB128 variable-length integers
//! - Length-prefixed sequences
//! - Fixed-size primitives
//!
//! Reference: <https://docs.rs/bcs/latest/bcs/>

use std::io::Read;
use universal_decoder_core::prelude::*;

/// Maximum bytes in a ULEB128-encoded value (10 bytes for u64)
const MAX_ULEB128_BYTES: usize = 10;

/// Maximum sequence length (防止 DoS attacks)
const MAX_SEQUENCE_LENGTH: u64 = 1_000_000;

/// Read a ULEB128 (unsigned LEB128) encoded integer
///
/// ULEB128 is a variable-length encoding where:
/// - Each byte has 7 data bits and 1 continuation bit (MSB)
/// - If MSB is 1, more bytes follow
/// - Bytes are in little-endian order (LSB first)
///
/// # Example
///
/// ```ignore
/// use std::io::Cursor;
/// use decoder_encodings::bcs::read_uleb128;
///
/// let data = vec![0xe5, 0x8e, 0x26]; // 624485 in ULEB128
/// let mut cursor = Cursor::new(data.as_slice());
/// let value = read_uleb128(&mut cursor).unwrap();
/// assert_eq!(value, 624485);
/// ```
pub fn read_uleb128<R: Read>(reader: &mut R) -> Result<u64> {
    let mut result = 0u64;
    let mut shift = 0u32;

    for byte_index in 0..MAX_ULEB128_BYTES {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf).map_err(|e| {
            DecoderError::invalid_structure(format!(
                "Failed to read ULEB128 byte {}: {}",
                byte_index, e
            ))
        })?;

        let byte = buf[0];
        let value_bits = (byte & 0x7F) as u64;

        // Check for overflow before shifting
        if shift >= 64 {
            return Err(DecoderError::invalid_structure(
                "ULEB128 value exceeds u64 range",
            ));
        }

        // Add this byte's contribution to the result
        result |= value_bits << shift;

        // If MSB is 0, this is the last byte
        if byte & 0x80 == 0 {
            return Ok(result);
        }

        shift += 7;
    }

    Err(DecoderError::invalid_structure(
        "ULEB128 value exceeds maximum length",
    ))
}

/// Read a BCS-encoded u8
#[inline]
pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

/// Read a BCS-encoded u16 (little-endian)
#[inline]
pub fn read_u16<R: Read>(reader: &mut R) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_le_bytes(buf))
}

/// Read a BCS-encoded u32 (little-endian)
#[inline]
pub fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(buf))
}

/// Read a BCS-encoded u64 (little-endian)
#[inline]
pub fn read_u64<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_le_bytes(buf))
}

/// Read a BCS-encoded u128 (little-endian)
#[inline]
pub fn read_u128<R: Read>(reader: &mut R) -> Result<u128> {
    let mut buf = [0u8; 16];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u128: {}", e)))?;
    Ok(u128::from_le_bytes(buf))
}

/// Read a BCS-encoded boolean
///
/// BCS encodes booleans as a single byte: 0x00 for false, 0x01 for true
#[inline]
pub fn read_bool<R: Read>(reader: &mut R) -> Result<bool> {
    match read_u8(reader)? {
        0x00 => Ok(false),
        0x01 => Ok(true),
        byte => Err(DecoderError::invalid_structure(format!(
            "Invalid boolean value: 0x{:02x} (expected 0x00 or 0x01)",
            byte
        ))),
    }
}

/// Read a BCS-encoded byte vector (length-prefixed)
///
/// Format: ULEB128(length) || bytes
///
/// # Example
///
/// ```ignore
/// use std::io::Cursor;
/// use decoder_encodings::bcs::read_bytes;
///
/// let data = vec![0x03, 0x01, 0x02, 0x03]; // length=3, bytes=[1,2,3]
/// let mut cursor = Cursor::new(data.as_slice());
/// let bytes = read_bytes(&mut cursor).unwrap();
/// assert_eq!(bytes, vec![1, 2, 3]);
/// ```
pub fn read_bytes<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let length = read_uleb128(reader)?;

    if length > MAX_SEQUENCE_LENGTH {
        return Err(DecoderError::invalid_structure(format!(
            "Byte vector too long: {} (max {})",
            length, MAX_SEQUENCE_LENGTH
        )));
    }

    let mut buf = vec![0u8; length as usize];
    reader.read_exact(&mut buf).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to read {} bytes: {}", length, e))
    })?;

    Ok(buf)
}

/// Read exactly N bytes from reader
///
/// This is a helper for reading fixed-size byte arrays (e.g., addresses, hashes)
///
/// # Example
///
/// ```ignore
/// use std::io::Cursor;
/// use decoder_encodings::bcs::read_fixed_bytes;
///
/// let data = vec![0x01, 0x02, 0x03, 0x04];
/// let mut cursor = Cursor::new(data.as_slice());
/// let bytes: [u8; 4] = read_fixed_bytes(&mut cursor).unwrap();
/// assert_eq!(bytes, [0x01, 0x02, 0x03, 0x04]);
/// ```
pub fn read_fixed_bytes<R: Read, const N: usize>(reader: &mut R) -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to read {} bytes: {}", N, e))
    })?;
    Ok(buf)
}

/// Read a BCS-encoded string (UTF-8, length-prefixed)
///
/// Format: ULEB128(length) || UTF-8 bytes
pub fn read_string<R: Read>(reader: &mut R) -> Result<String> {
    let bytes = read_bytes(reader)?;
    String::from_utf8(bytes)
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid UTF-8 string: {}", e)))
}

/// Read a BCS-encoded option<T>
///
/// Format:
/// - 0x00 => None
/// - 0x01 || T => Some(T)
pub fn read_option<R: Read, T, F>(reader: &mut R, read_fn: F) -> Result<Option<T>>
where
    F: FnOnce(&mut R) -> Result<T>,
{
    match read_u8(reader)? {
        0x00 => Ok(None),
        0x01 => Ok(Some(read_fn(reader)?)),
        byte => Err(DecoderError::invalid_structure(format!(
            "Invalid option tag: 0x{:02x} (expected 0x00 or 0x01)",
            byte
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_uleb128_single_byte() {
        let data = vec![0x00]; // 0
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_uleb128(&mut cursor).unwrap(), 0);

        let data = vec![0x7F]; // 127
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_uleb128(&mut cursor).unwrap(), 127);
    }

    #[test]
    fn test_uleb128_multi_byte() {
        // 128 = 0x80, 0x01
        let data = vec![0x80, 0x01];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_uleb128(&mut cursor).unwrap(), 128);

        // 624485 = 0xe5, 0x8e, 0x26
        let data = vec![0xe5, 0x8e, 0x26];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_uleb128(&mut cursor).unwrap(), 624485);

        // Maximum value: u64::MAX
        // Encoding: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_uleb128(&mut cursor).unwrap(), u64::MAX);
    }

    #[test]
    fn test_uleb128_invalid_too_long() {
        // 11 bytes with continuation bits (exceeds MAX_ULEB128_BYTES)
        let data = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
        ];
        let mut cursor = Cursor::new(data.as_slice());
        assert!(read_uleb128(&mut cursor).is_err());
    }

    #[test]
    fn test_read_primitives() {
        let data = vec![
            0x2A, // u8: 42
            0x39, 0x30, // u16: 12345 (LE)
            0x15, 0xCD, 0x5B, 0x07, // u32: 123456789 (LE)
        ];
        let mut cursor = Cursor::new(data.as_slice());

        assert_eq!(read_u8(&mut cursor).unwrap(), 42);
        assert_eq!(read_u16(&mut cursor).unwrap(), 12345);
        assert_eq!(read_u32(&mut cursor).unwrap(), 123456789);
    }

    #[test]
    fn test_read_bool() {
        let data = vec![0x00, 0x01];
        let mut cursor = Cursor::new(data.as_slice());

        assert!(!read_bool(&mut cursor).unwrap());
        assert!(read_bool(&mut cursor).unwrap());

        // Invalid boolean value
        let data = vec![0x02];
        let mut cursor = Cursor::new(data.as_slice());
        assert!(read_bool(&mut cursor).is_err());
    }

    #[test]
    fn test_read_bytes() {
        // Length 3, bytes [1, 2, 3]
        let data = vec![0x03, 0x01, 0x02, 0x03];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_bytes(&mut cursor).unwrap(), vec![1, 2, 3]);

        // Empty byte vector
        let data = vec![0x00];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_bytes(&mut cursor).unwrap(), vec![]);
    }

    #[test]
    fn test_read_fixed_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(data.as_slice());
        let bytes: [u8; 4] = read_fixed_bytes(&mut cursor).unwrap();
        assert_eq!(bytes, [0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_read_string() {
        // "hello" = length 5, UTF-8 bytes
        let data = vec![0x05, b'h', b'e', b'l', b'l', b'o'];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_string(&mut cursor).unwrap(), "hello");

        // Empty string
        let data = vec![0x00];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_string(&mut cursor).unwrap(), "");
    }

    #[test]
    fn test_read_option() {
        // Some(42)
        let data = vec![0x01, 0x2A];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_option(&mut cursor, read_u8).unwrap(), Some(42));

        // None
        let data = vec![0x00];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_option(&mut cursor, read_u8).unwrap(), None);

        // Invalid tag
        let data = vec![0x02];
        let mut cursor = Cursor::new(data.as_slice());
        assert!(read_option(&mut cursor, read_u8).is_err());
    }

    #[test]
    fn test_bytes_too_long() {
        // Try to read a byte vector with length > MAX_SEQUENCE_LENGTH
        // ULEB128 encoding of (MAX_SEQUENCE_LENGTH + 1)
        let length = MAX_SEQUENCE_LENGTH + 1;
        let mut data = vec![];
        let mut value = length;
        while value >= 0x80 {
            data.push(((value & 0x7F) | 0x80) as u8);
            value >>= 7;
        }
        data.push(value as u8);

        let mut cursor = Cursor::new(data.as_slice());
        assert!(read_bytes(&mut cursor).is_err());
    }
}
