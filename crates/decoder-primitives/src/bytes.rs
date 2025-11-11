//! Byte-level operations with bounds checking
//!
//! This module provides safe byte reading operations with configurable limits
//! to prevent memory exhaustion attacks and ensure safe parsing.

use std::io::Read;
use universal_decoder_core::prelude::{DecoderError, Result};

/// Read exactly N bytes with bounds checking
///
/// # Arguments
/// * `reader` - Input reader
/// * `len` - Number of bytes to read
/// * `max_len` - Maximum allowed length (for safety)
///
/// # Errors
/// Returns error if:
/// - `len > max_len` (prevents excessive memory allocation)
/// - Not enough bytes available in reader
///
/// # Examples
///
/// ```rust,ignore
/// use decoder_primitives::bytes::read_bytes_bounded;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let mut cursor = Cursor::new(data);
///
/// // Read 3 bytes with max limit of 10
/// let bytes = read_bytes_bounded(&mut cursor, 3, 10).unwrap();
/// assert_eq!(bytes, vec![1, 2, 3]);
///
/// // Try to read more than max - will fail
/// assert!(read_bytes_bounded(&mut cursor, 20, 10).is_err());
/// ```
pub fn read_bytes_bounded<R: Read>(
    reader: &mut R,
    len: usize,
    max_len: usize,
) -> Result<Vec<u8>> {
    if len > max_len {
        return Err(DecoderError::invalid_structure(format!(
            "Requested {} bytes, but maximum is {} (possible attack)",
            len, max_len
        )));
    }

    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read {} bytes: {}", len, e)))?;

    Ok(buf)
}

/// Read exactly N bytes without explicit max (uses default safety limit)
///
/// Default max: 10 MB (prevents most attacks while allowing legitimate data)
///
/// For custom limits, use `read_bytes_bounded` instead.
pub fn read_bytes<R: Read>(reader: &mut R, len: usize) -> Result<Vec<u8>> {
    const DEFAULT_MAX: usize = 10 * 1024 * 1024; // 10 MB
    read_bytes_bounded(reader, len, DEFAULT_MAX)
}

/// Read exactly N bytes into a fixed-size array
///
/// This is more efficient than reading into a Vec when the size is known at compile time.
///
/// # Examples
///
/// ```rust,ignore
/// // Read a 32-byte hash
/// let hash: [u8; 32] = read_array(&mut reader)?;
///
/// // Read a 20-byte Ethereum address
/// let address: [u8; 20] = read_array(&mut reader)?;
/// ```
pub fn read_array<R: Read, const N: usize>(reader: &mut R) -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read {} bytes: {}", N, e)))?;
    Ok(buf)
}

/// Read all remaining bytes from reader
///
/// **Warning**: Use with caution! Only use when you trust the data source.
/// For untrusted input, use `read_bytes_bounded` instead.
pub fn read_remaining<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read remaining bytes: {}", e)))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_bytes_bounded_success() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);

        let bytes = read_bytes_bounded(&mut cursor, 3, 10).unwrap();
        assert_eq!(bytes, vec![1, 2, 3]);
    }

    #[test]
    fn test_read_bytes_bounded_exceeds_max() {
        let data = vec![1, 2, 3];
        let mut cursor = Cursor::new(data);

        // Try to read more than max
        let result = read_bytes_bounded(&mut cursor, 20, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_bytes_bounded_eof() {
        let data = vec![1, 2, 3];
        let mut cursor = Cursor::new(data);

        // Try to read more than available
        let result = read_bytes_bounded(&mut cursor, 5, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_bytes_zero_length() {
        let data = vec![1, 2, 3];
        let mut cursor = Cursor::new(data);

        let bytes = read_bytes_bounded(&mut cursor, 0, 10).unwrap();
        assert_eq!(bytes, Vec::<u8>::new());
    }

    #[test]
    fn test_read_array() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut cursor = Cursor::new(data);

        let arr: [u8; 4] = read_array(&mut cursor).unwrap();
        assert_eq!(arr, [1, 2, 3, 4]);

        let arr2: [u8; 3] = read_array(&mut cursor).unwrap();
        assert_eq!(arr2, [5, 6, 7]);
    }

    #[test]
    fn test_read_array_eof() {
        let data = vec![1, 2, 3];
        let mut cursor = Cursor::new(data);

        let result: Result<[u8; 5]> = read_array(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_remaining() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);

        // Read first 2 bytes
        let _first = read_bytes_bounded(&mut cursor, 2, 10).unwrap();

        // Read remaining
        let remaining = read_remaining(&mut cursor).unwrap();
        assert_eq!(remaining, vec![3, 4, 5]);
    }

    #[test]
    fn test_read_bytes_default_max() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);

        let bytes = read_bytes(&mut cursor, 3).unwrap();
        assert_eq!(bytes, vec![1, 2, 3]);
    }
}
