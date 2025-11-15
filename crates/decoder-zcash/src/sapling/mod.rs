//! Sapling shielded transaction parsing (Phase 2)
//!
//! This module implements parsing for Zcash Sapling shielded transactions,
//! which use zk-SNARK proofs to enable private transfers.
//!
//! ## Sapling Features
//!
//! - **SpendDescription**: Consumes a shielded note (nullifier-based)
//! - **OutputDescription**: Creates a shielded note (encrypted to recipient)
//! - **Value Balance**: Net transparent ↔ shielded value transfer
//! - **Binding Signature**: Proves value conservation across all components
//!
//! ## Privacy Guarantees
//!
//! Sapling transactions hide:
//! - Sender identity (via nullifiers, not linkable to address)
//! - Recipient identity (via encrypted note commitments)
//! - Transaction amounts (via homomorphic commitments)
//!
//! ## Parsing Strategy
//!
//! This module **parses** Sapling components (binary extraction) but does **not verify**
//! zk-SNARK proofs. Proof verification is out of scope for the decoder
//! (decoders are for analysis, not consensus validation).
//!
//! ## Module Structure
//!
//! - `spend.rs`: SpendDescription parsing (nullifiers, commitments, proofs)
//! - `output.rs`: OutputDescription parsing (encrypted notes, ephemeral keys)

pub mod output;
pub mod spend;

pub use output::{parse_output_description, OutputDescription};
pub use spend::{parse_spend_description, SpendDescription};

use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Parse i64 (little-endian) from cursor
///
/// Used for parsing value_balance (net transparent ↔ shielded)
pub fn read_i64_le(cursor: &mut Cursor<&[u8]>) -> Result<i64> {
    let mut buf = [0u8; 8];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read i64: {}", e)))?;
    Ok(i64::from_le_bytes(buf))
}

/// Read fixed-size byte array from cursor
///
/// Used for reading commitments, nullifiers, proofs, etc.
pub fn read_fixed_bytes<const N: usize>(cursor: &mut Cursor<&[u8]>) -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read {} bytes: {}", N, e)))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_i64_le_positive() {
        let bytes = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = read_i64_le(&mut cursor).unwrap();
        assert_eq!(result, 0x0807060504030201_i64);
    }

    #[test]
    fn test_read_i64_le_negative() {
        let bytes = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let mut cursor = Cursor::new(&bytes[..]);

        let result = read_i64_le(&mut cursor).unwrap();
        assert_eq!(result, -1_i64);
    }

    #[test]
    fn test_read_fixed_bytes_32() {
        let bytes = [0x42; 32];
        let mut cursor = Cursor::new(&bytes[..]);

        let result: [u8; 32] = read_fixed_bytes(&mut cursor).unwrap();
        assert_eq!(result, [0x42; 32]);
    }

    #[test]
    fn test_read_fixed_bytes_insufficient() {
        let bytes = [0x42; 10]; // Only 10 bytes
        let mut cursor = Cursor::new(&bytes[..]);

        let result: Result<[u8; 32]> = read_fixed_bytes(&mut cursor);
        assert!(result.is_err());
    }
}
