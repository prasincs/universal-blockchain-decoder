//! Big-endian primitive readers
//!
//! These readers are used by blockchains that use big-endian encoding:
//! - Ethereum
//! - Polkadot
//! - Cosmos
//! - Most EVM-compatible chains

use std::io::Read;
use universal_decoder_core::prelude::{DecoderError, Result};

/// Read u16 (2 bytes, big-endian)
///
/// # Examples
///
/// ```rust,ignore
/// let data = vec![0x12, 0x34]; // 0x1234 in big-endian
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_u16_be(&mut cursor).unwrap(), 0x1234);
/// ```
#[inline]
pub fn read_u16_be<R: Read>(reader: &mut R) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_be_bytes(buf))
}

/// Read u32 (4 bytes, big-endian)
///
/// # Examples
///
/// ```rust,ignore
/// let data = vec![0x12, 0x34, 0x56, 0x78]; // 0x12345678 in big-endian
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_u32_be(&mut cursor).unwrap(), 0x12345678);
/// ```
#[inline]
pub fn read_u32_be<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_be_bytes(buf))
}

/// Read u64 (8 bytes, big-endian)
///
/// # Examples
///
/// ```rust,ignore
/// let data = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_u64_be(&mut cursor).unwrap(), 0x1122334455667788);
/// ```
#[inline]
pub fn read_u64_be<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_be_bytes(buf))
}

/// Read u128 (16 bytes, big-endian)
#[inline]
pub fn read_u128_be<R: Read>(reader: &mut R) -> Result<u128> {
    let mut buf = [0u8; 16];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u128: {}", e)))?;
    Ok(u128::from_be_bytes(buf))
}

/// Read u256 (32 bytes, big-endian)
///
/// Returns as byte array. Useful for Ethereum addresses, hashes, and large integers.
///
/// # Examples
///
/// ```rust,ignore
/// let data = vec![0xFF; 32];
/// let mut cursor = Cursor::new(data);
/// let u256 = read_u256_be(&mut cursor).unwrap();
/// assert_eq!(u256.len(), 32);
/// ```
pub fn read_u256_be<R: Read>(reader: &mut R) -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u256: {}", e)))?;
    Ok(buf)
}

/// Read Ethereum address (20 bytes, big-endian)
pub fn read_address<R: Read>(reader: &mut R) -> Result<[u8; 20]> {
    let mut buf = [0u8; 20];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read address: {}", e)))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_u16_be() {
        let data = vec![0x12, 0x34]; // 0x1234 in big-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u16_be(&mut cursor).unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u32_be() {
        let data = vec![0x12, 0x34, 0x56, 0x78]; // 0x12345678 in big-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u32_be(&mut cursor).unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_u64_be() {
        let data = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_u64_be(&mut cursor).unwrap(), 0x1122334455667788);
    }

    #[test]
    fn test_read_u128_be() {
        let data = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        let mut cursor = Cursor::new(data);
        assert_eq!(
            read_u128_be(&mut cursor).unwrap(),
            0x0102030405060708090A0B0C0D0E0F10
        );
    }

    #[test]
    fn test_read_u256_be() {
        let data = vec![0xAB; 32];
        let mut cursor = Cursor::new(data);
        let result = read_u256_be(&mut cursor).unwrap();
        assert_eq!(result, [0xAB; 32]);
    }

    #[test]
    fn test_read_address() {
        let data = vec![0x42; 20];
        let mut cursor = Cursor::new(data);
        let result = read_address(&mut cursor).unwrap();
        assert_eq!(result, [0x42; 20]);
    }

    #[test]
    fn test_read_address_eof() {
        let data = vec![0x42; 10]; // Only 10 bytes
        let mut cursor = Cursor::new(data);
        assert!(read_address(&mut cursor).is_err());
    }
}
