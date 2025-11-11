//! Pure Rust Bitcoin transaction parsing utilities
//!
//! This module provides low-level parsing functions for Bitcoin transaction
//! components, including VarInt encoding, inputs, outputs, and witness data.
//!
//! All parsing is done without external blockchain library dependencies.

use std::io::Read;
use universal_decoder_core::prelude::*;
use decoder_primitives::prelude::*;

/// Maximum script size (conservative limit for safety)
pub const MAX_SCRIPT_SIZE: usize = 10_000;

/// Maximum transaction size (100 KB for standard transactions)
pub const MAX_TRANSACTION_SIZE: usize = 100_000;

/// Maximum number of inputs/outputs (sanity check)
pub const MAX_INPUTS_OUTPUTS: usize = 10_000;

/// Read exactly N bytes with script size limit
pub fn read_bytes<R: Read>(reader: &mut R, len: usize) -> Result<Vec<u8>> {
    read_bytes_bounded(reader, len, MAX_SCRIPT_SIZE)
}

/// Parse a Bitcoin VarInt (variable-length integer)
///
/// Bitcoin uses a variable-length integer encoding called VarInt:
/// - 0x00-0xFC: 1 byte (value 0-252)
/// - 0xFD: 3 bytes (0xFD + u16 little-endian, value 253-65535)
/// - 0xFE: 5 bytes (0xFE + u32 little-endian, value 65536-4294967295)
/// - 0xFF: 9 bytes (0xFF + u64 little-endian, value 4294967296+)
///
/// # Examples
///
/// ```rust,ignore
/// use std::io::Cursor;
/// let data = vec![0x12]; // 18 in single byte
/// let mut cursor = Cursor::new(data);
/// assert_eq!(read_varint(&mut cursor).unwrap(), 18);
/// ```
pub fn read_varint<R: Read>(reader: &mut R) -> Result<u64> {
    let first_byte = read_u8(reader)?;

    match first_byte {
        // Single byte: 0-252
        0x00..=0xFC => Ok(first_byte as u64),

        // 0xFD: Next 2 bytes are u16 (little-endian)
        0xFD => {
            let value = read_u16_le(reader)?;
            if value < 0xFD {
                return Err(DecoderError::invalid_structure(format!(
                    "Non-canonical VarInt: 0xFD prefix for value {}",
                    value
                )));
            }
            Ok(value as u64)
        }

        // 0xFE: Next 4 bytes are u32 (little-endian)
        0xFE => {
            let value = read_u32_le(reader)?;
            if value < 0x10000 {
                return Err(DecoderError::invalid_structure(format!(
                    "Non-canonical VarInt: 0xFE prefix for value {}",
                    value
                )));
            }
            Ok(value as u64)
        }

        // 0xFF: Next 8 bytes are u64 (little-endian)
        0xFF => {
            let value = read_u64_le(reader)?;
            if value < 0x100000000 {
                return Err(DecoderError::invalid_structure(format!(
                    "Non-canonical VarInt: 0xFF prefix for value {}",
                    value
                )));
            }
            Ok(value)
        }
    }
}

/// Bitcoin transaction input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInput {
    /// Previous transaction hash (32 bytes, little-endian)
    pub prev_hash: [u8; 32],
    /// Previous output index
    pub prev_index: u32,
    /// Unlocking script (scriptSig)
    pub script_sig: Vec<u8>,
    /// Sequence number
    pub sequence: u32,
}

/// Parse a transaction input
///
/// Input structure:
/// - Previous hash: 32 bytes
/// - Previous index: 4 bytes (u32 little-endian)
/// - Script length: VarInt
/// - Script sig: Variable bytes
/// - Sequence: 4 bytes (u32 little-endian)
pub fn parse_input<R: Read>(reader: &mut R) -> Result<TxInput> {
    // Read previous transaction hash (32 bytes)
    let mut prev_hash = [0u8; 32];
    reader
        .read_exact(&mut prev_hash)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read prev_hash: {}", e)))?;

    // Read previous output index (4 bytes, little-endian)
    let prev_index = read_u32_le(reader)?;

    // Read script length (varint)
    let script_len = read_varint(reader)?;
    if script_len > MAX_SCRIPT_SIZE as u64 {
        return Err(DecoderError::invalid_structure(format!(
            "Script too large: {} bytes",
            script_len
        )));
    }

    // Read script bytes
    let script_sig = read_bytes(reader, script_len as usize)?;

    // Read sequence (4 bytes, little-endian)
    let sequence = read_u32_le(reader)?;

    Ok(TxInput {
        prev_hash,
        prev_index,
        script_sig,
        sequence,
    })
}

/// Bitcoin transaction output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    /// Value in satoshis
    pub value: u64,
    /// Locking script (scriptPubKey)
    pub script_pubkey: Vec<u8>,
}

/// Parse a transaction output
///
/// Output structure:
/// - Value: 8 bytes (u64 little-endian)
/// - Script length: VarInt
/// - Script pubkey: Variable bytes
pub fn parse_output<R: Read>(reader: &mut R) -> Result<TxOutput> {
    // Read value (8 bytes, little-endian)
    let value = read_u64_le(reader)?;

    // Read script length (varint)
    let script_len = read_varint(reader)?;
    if script_len > MAX_SCRIPT_SIZE as u64 {
        return Err(DecoderError::invalid_structure(format!(
            "Script too large: {} bytes",
            script_len
        )));
    }

    // Read script bytes
    let script_pubkey = read_bytes(reader, script_len as usize)?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

/// Witness data for a single input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    /// Stack items (witness elements)
    pub items: Vec<Vec<u8>>,
}

impl Witness {
    /// Create an empty witness
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    /// Check if witness is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of witness items
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Parse witness data for a single input
///
/// Witness structure:
/// - Stack item count: VarInt
/// - For each item:
///   - Item length: VarInt
///   - Item data: Variable bytes
pub fn parse_witness<R: Read>(reader: &mut R) -> Result<Witness> {
    // Read number of stack items (varint)
    let stack_count = read_varint(reader)?;

    if stack_count > MAX_INPUTS_OUTPUTS as u64 {
        return Err(DecoderError::invalid_structure(format!(
            "Too many witness items: {}",
            stack_count
        )));
    }

    // Read each stack item
    let mut items = Vec::with_capacity(stack_count as usize);
    for i in 0..stack_count {
        let item_len = read_varint(reader)?;
        if item_len > MAX_SCRIPT_SIZE as u64 {
            return Err(DecoderError::invalid_structure(format!(
                "Witness item {} too large: {} bytes",
                i, item_len
            )));
        }

        let item = read_bytes(reader, item_len as usize)?;
        items.push(item);
    }

    Ok(Witness { items })
}

/// Parse witness data for all inputs
pub fn parse_witnesses<R: Read>(reader: &mut R, input_count: usize) -> Result<Vec<Witness>> {
    let mut witnesses = Vec::with_capacity(input_count);

    for i in 0..input_count {
        witnesses.push(
            parse_witness(reader)
                .map_err(|e| DecoderError::chain_decoding(format!("Failed to parse witness {}: {}", i, e)))?,
        );
    }

    Ok(witnesses)
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
    fn test_read_varint_single_byte() {
        let data = vec![0x12];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 18);
    }

    #[test]
    fn test_read_varint_max_single_byte() {
        let data = vec![0xFC];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 252);
    }

    #[test]
    fn test_read_varint_fd() {
        let data = vec![0xFD, 0xFD, 0x00]; // 253 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 253);
    }

    #[test]
    fn test_read_varint_fd_max() {
        let data = vec![0xFD, 0xFF, 0xFF]; // 65535 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 65535);
    }

    #[test]
    fn test_read_varint_fe() {
        let data = vec![0xFE, 0x00, 0x00, 0x01, 0x00]; // 65536 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 65536);
    }

    #[test]
    fn test_read_varint_fe_max() {
        let data = vec![0xFE, 0xFF, 0xFF, 0xFF, 0xFF]; // 4294967295 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 4294967295);
    }

    #[test]
    fn test_read_varint_ff() {
        let data = vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]; // 4294967296
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 4294967296);
    }

    #[test]
    fn test_read_varint_ff_max() {
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), u64::MAX);
    }

    #[test]
    fn test_read_varint_truncated_fd() {
        let data = vec![0xFD, 0x00]; // Incomplete
        let mut cursor = Cursor::new(data);
        assert!(read_varint(&mut cursor).is_err());
    }

    #[test]
    fn test_read_varint_truncated_fe() {
        let data = vec![0xFE, 0x00, 0x00]; // Incomplete
        let mut cursor = Cursor::new(data);
        assert!(read_varint(&mut cursor).is_err());
    }

    #[test]
    fn test_read_varint_truncated_ff() {
        let data = vec![0xFF, 0x00, 0x00, 0x00, 0x00]; // Incomplete
        let mut cursor = Cursor::new(data);
        assert!(read_varint(&mut cursor).is_err());
    }

    #[test]
    fn test_read_varint_non_canonical_fd() {
        // 0xFD prefix but value < 253 (non-canonical)
        let data = vec![0xFD, 0xFC, 0x00]; // 252 with FD prefix
        let mut cursor = Cursor::new(data);
        assert!(read_varint(&mut cursor).is_err());
    }

    #[test]
    fn test_read_varint_non_canonical_fe() {
        // 0xFE prefix but value < 65536 (non-canonical)
        let data = vec![0xFE, 0xFF, 0xFF, 0x00, 0x00]; // 65535 with FE prefix
        let mut cursor = Cursor::new(data);
        assert!(read_varint(&mut cursor).is_err());
    }

    #[test]
    fn test_parse_input_simple() {
        let mut data = vec![];
        // prev_hash (32 bytes, all zeros)
        data.extend_from_slice(&[0u8; 32]);
        // prev_index (4 bytes)
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        // script_len (varint: 0)
        data.push(0x00);
        // sequence (4 bytes)
        data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

        let mut cursor = Cursor::new(data);
        let input = parse_input(&mut cursor).unwrap();

        assert_eq!(input.prev_hash, [0u8; 32]);
        assert_eq!(input.prev_index, 1);
        assert_eq!(input.script_sig, Vec::<u8>::new());
        assert_eq!(input.sequence, 0xFFFFFFFF);
    }

    #[test]
    fn test_parse_input_with_script() {
        let mut data = vec![];
        // prev_hash (32 bytes)
        data.extend_from_slice(&[0xAA; 32]);
        // prev_index
        data.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]);
        // script_len (varint: 3)
        data.push(0x03);
        // script_sig (3 bytes)
        data.extend_from_slice(&[0x11, 0x22, 0x33]);
        // sequence
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let mut cursor = Cursor::new(data);
        let input = parse_input(&mut cursor).unwrap();

        assert_eq!(input.prev_hash, [0xAA; 32]);
        assert_eq!(input.prev_index, 2);
        assert_eq!(input.script_sig, vec![0x11, 0x22, 0x33]);
        assert_eq!(input.sequence, 0);
    }

    #[test]
    fn test_parse_input_truncated() {
        let data = vec![0x00; 16]; // Only 16 bytes (need at least 32 for prev_hash)
        let mut cursor = Cursor::new(data);
        assert!(parse_input(&mut cursor).is_err());
    }

    #[test]
    fn test_parse_output_simple() {
        let mut data = vec![];
        // value (8 bytes: 50 BTC in satoshis = 5,000,000,000)
        data.extend_from_slice(&5_000_000_000u64.to_le_bytes());
        // script_len (varint: 0)
        data.push(0x00);

        let mut cursor = Cursor::new(data);
        let output = parse_output(&mut cursor).unwrap();

        assert_eq!(output.value, 5_000_000_000);
        assert_eq!(output.script_pubkey, Vec::<u8>::new());
    }

    #[test]
    fn test_parse_output_with_script() {
        let mut data = vec![];
        // value
        data.extend_from_slice(&1_000_000u64.to_le_bytes());
        // script_len (varint: 5)
        data.push(0x05);
        // script_pubkey (5 bytes)
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);

        let mut cursor = Cursor::new(data);
        let output = parse_output(&mut cursor).unwrap();

        assert_eq!(output.value, 1_000_000);
        assert_eq!(output.script_pubkey, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
    }

    #[test]
    fn test_parse_output_truncated() {
        let data = vec![0x00; 4]; // Only 4 bytes (need at least 8 for value)
        let mut cursor = Cursor::new(data);
        assert!(parse_output(&mut cursor).is_err());
    }

    #[test]
    fn test_parse_witness_empty() {
        let data = vec![0x00]; // 0 items
        let mut cursor = Cursor::new(data);
        let witness = parse_witness(&mut cursor).unwrap();

        assert!(witness.is_empty());
        assert_eq!(witness.len(), 0);
    }

    #[test]
    fn test_parse_witness_single_item() {
        let mut data = vec![];
        // item count: 1
        data.push(0x01);
        // item 0 length: 3
        data.push(0x03);
        // item 0 data
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC]);

        let mut cursor = Cursor::new(data);
        let witness = parse_witness(&mut cursor).unwrap();

        assert_eq!(witness.len(), 1);
        assert_eq!(witness.items[0], vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_parse_witness_multiple_items() {
        let mut data = vec![];
        // item count: 2
        data.push(0x02);
        // item 0: length 2
        data.push(0x02);
        data.extend_from_slice(&[0x11, 0x22]);
        // item 1: length 3
        data.push(0x03);
        data.extend_from_slice(&[0x33, 0x44, 0x55]);

        let mut cursor = Cursor::new(data);
        let witness = parse_witness(&mut cursor).unwrap();

        assert_eq!(witness.len(), 2);
        assert_eq!(witness.items[0], vec![0x11, 0x22]);
        assert_eq!(witness.items[1], vec![0x33, 0x44, 0x55]);
    }

    #[test]
    fn test_parse_witnesses_multiple_inputs() {
        let mut data = vec![];
        // witness 0: 1 item
        data.push(0x01);
        data.push(0x02);
        data.extend_from_slice(&[0xAA, 0xBB]);
        // witness 1: 0 items
        data.push(0x00);

        let mut cursor = Cursor::new(data);
        let witnesses = parse_witnesses(&mut cursor, 2).unwrap();

        assert_eq!(witnesses.len(), 2);
        assert_eq!(witnesses[0].len(), 1);
        assert_eq!(witnesses[1].len(), 0);
    }

    #[test]
    fn test_read_bytes_normal() {
        let data = vec![0x11, 0x22, 0x33, 0x44];
        let mut cursor = Cursor::new(data);
        let bytes = read_bytes(&mut cursor, 4).unwrap();
        assert_eq!(bytes, vec![0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn test_read_bytes_oversized() {
        let data = vec![0x00; 100];
        let mut cursor = Cursor::new(data);
        assert!(read_bytes(&mut cursor, MAX_SCRIPT_SIZE + 1).is_err());
    }
}
