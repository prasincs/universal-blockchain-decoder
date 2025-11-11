//! Universal blockchain decoder primitives
//!
//! This crate provides low-level parsing primitives used across all blockchain decoders.
//! It includes:
//! - Byte readers (little-endian and big-endian)
//! - Bounds-checked byte operations
//! - VarInt parsing (coming soon)
//!
//! # Design Goals
//!
//! 1. **Zero external dependencies** (except universal-decoder-core)
//! 2. **Reusable across all chains** (Bitcoin, Ethereum, Solana, etc.)
//! 3. **Security-first** (bounds checking, overflow protection)
//! 4. **Well-tested** (unit tests for all functions)
//!
//! # Usage
//!
//! ```rust,ignore
//! use decoder_primitives::prelude::*;
//! use std::io::Cursor;
//!
//! let data = vec![0x01, 0x02, 0x03, 0x04];
//! let mut cursor = Cursor::new(data);
//!
//! // Read little-endian u32 (Bitcoin, Solana)
//! let value = read_u32_le(&mut cursor)?;
//! assert_eq!(value, 0x04030201);
//!
//! // Read big-endian u32 (Ethereum, Cosmos)
//! let mut cursor2 = Cursor::new(vec![0x01, 0x02, 0x03, 0x04]);
//! let value2 = read_u32_be(&mut cursor2)?;
//! assert_eq!(value2, 0x01020304);
//! ```

pub mod bytes;
pub mod readers;

/// Re-export core types
pub use universal_decoder_core::prelude::*;

/// Commonly used imports
pub mod prelude {
    pub use crate::bytes::*;
    pub use crate::readers::big_endian::*;
    pub use crate::readers::little_endian::*;
    pub use universal_decoder_core::prelude::*;
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use std::io::Cursor;

    #[test]
    fn test_little_endian_round_trip() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(data);

        let value = read_u32_le(&mut cursor).unwrap();
        assert_eq!(value, 0x04030201);
    }

    #[test]
    fn test_big_endian_round_trip() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let mut cursor = Cursor::new(data);

        let value = read_u32_be(&mut cursor).unwrap();
        assert_eq!(value, 0x01020304);
    }

    #[test]
    fn test_bytes_bounded() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);

        // Read 3 bytes with max 10
        let bytes = read_bytes_bounded(&mut cursor, 3, 10).unwrap();
        assert_eq!(bytes, vec![1, 2, 3]);

        // Try to read more than available
        let result = read_bytes_bounded(&mut cursor, 10, 20);
        assert!(result.is_err());
    }

    #[test]
    fn test_mixed_endianness() {
        // Simulate a hybrid transaction format
        let data = vec![
            0x01, 0x02, 0x03, 0x04, // LE u32
            0x05, 0x06, 0x07, 0x08, // BE u32
        ];
        let mut cursor = Cursor::new(data);

        let le_value = read_u32_le(&mut cursor).unwrap();
        let be_value = read_u32_be(&mut cursor).unwrap();

        assert_eq!(le_value, 0x04030201);
        assert_eq!(be_value, 0x05060708);
    }
}
