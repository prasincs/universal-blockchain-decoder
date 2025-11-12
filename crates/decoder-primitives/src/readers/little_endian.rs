//! Little-endian primitive readers
//!
//! These readers are used by blockchains that use little-endian encoding:
//! - Bitcoin
//! - Solana
//! - Litecoin
//! - Dogecoin
//! - Most Bitcoin-derived chains

use std::io::Read;
use universal_decoder_core::prelude::{DecoderError, Result};

/// Read a single byte (u8)
///
/// # Examples
///
/// ```rust,ignore
/// use std::io::Cursor;
/// use decoder_primitives::readers::little_endian::*;
///
/// let data = vec![0x42];
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_u8(&mut cursor).unwrap(), 0x42);
/// ```
#[inline]
pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

/// Read u16 (2 bytes, little-endian)
///
/// # Examples
///
/// ```rust,ignore
/// let data = vec![0x34, 0x12]; // 0x1234 in little-endian
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_u16_le(&mut cursor).unwrap(), 0x1234);
/// ```
#[inline]
pub fn read_u16_le<R: Read>(reader: &mut R) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_le_bytes(buf))
}

/// Read u32 (4 bytes, little-endian)
///
/// # Examples
///
/// ```rust,ignore
/// let data = vec![0x78, 0x56, 0x34, 0x12]; // 0x12345678 in little-endian
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_u32_le(&mut cursor).unwrap(), 0x12345678);
/// ```
#[inline]
pub fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(buf))
}

/// Read u64 (8 bytes, little-endian)
///
/// # Examples
///
/// ```rust,ignore
/// let data = vec![0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11];
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_u64_le(&mut cursor).unwrap(), 0x1122334455667788);
/// ```
#[inline]
pub fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_le_bytes(buf))
}

/// Read u128 (16 bytes, little-endian)
///
/// Useful for large integer values in some blockchain protocols.
#[inline]
pub fn read_u128_le<R: Read>(reader: &mut R) -> Result<u128> {
    let mut buf = [0u8; 16];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u128: {}", e)))?;
    Ok(u128::from_le_bytes(buf))
}

/// Read i32 (4 bytes, little-endian, signed)
///
/// Used by some blockchains for signed integers (e.g., Bitcoin version field).
#[inline]
pub fn read_i32_le<R: Read>(reader: &mut R) -> Result<i32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read i32: {}", e)))?;
    Ok(i32::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_u8() {
        let data = vec![0x42];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u8(&mut cursor).unwrap(), 0x42);
    }

    #[test]
    fn test_read_u8_eof() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        assert!(read_u8(&mut cursor).is_err());
    }

    #[test]
    fn test_read_u16_le() {
        let data = vec![0x34, 0x12]; // 0x1234 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u16_le(&mut cursor).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u16_le_eof() {
        let data = vec![0x34]; // Only 1 byte
        let mut cursor = Cursor::new(data);
        assert!(read_u16_le(&mut cursor).is_err());
    }

    #[test]
    fn test_read_u32_le() {
        let data = vec![0x78, 0x56, 0x34, 0x12]; // 0x12345678 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u32_le(&mut cursor).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_u64_le() {
        let data = vec![0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u64_le(&mut cursor).unwrap(), 0x1122334455667788);
    }

    #[test]
    fn test_read_u128_le() {
        let data = vec![
            0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
            0x02, 0x01,
        ];
        let mut cursor = Cursor::new(data);
        assert_eq!(
            read_u128_le(&mut cursor).unwrap(),
            0x0102030405060708090A0B0C0D0E0F10
        );
    }

    #[test]
    fn test_read_i32_le() {
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // -1 in two's complement
        let mut cursor = Cursor::new(data);
        assert_eq!(read_i32_le(&mut cursor).unwrap(), -1);
    }
}
