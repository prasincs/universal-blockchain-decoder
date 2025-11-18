//! Bag of Cells (BoC) parsing for TON blockchain
//!
//! This module implements parsing of TON's Bag of Cells serialization format.
//! BoC is a binary format for encoding cell trees used throughout TON.
//!
//! ## BoC Format
//!
//! The standard BoC format (magic 0xb5ee9c72) has the following structure:
//!
//! ```text
//! +-------------------+------------+----------------------------------+
//! | Field             | Size       | Description                      |
//! +-------------------+------------+----------------------------------+
//! | magic             | 4 bytes    | 0xb5ee9c72 (standard)           |
//! | flags_and_s_bytes | 1 byte     | Packed: has_idx|has_crc|...|size|
//! | off_bytes         | 1 byte     | Offset size (1-8 bytes)         |
//! | cells             | off_bytes  | Number of cells                 |
//! | roots             | off_bytes  | Number of root cells            |
//! | absent            | off_bytes  | Number of absent cells          |
//! | tot_cells_size    | off_bytes  | Total cells data size           |
//! | root_list         | variable   | Root cell indices               |
//! | index             | optional   | Cell offset index (if has_idx)  |
//! | cell_data         | variable   | All cell data                   |
//! | crc32c            | 4 bytes    | CRC32-C checksum (if has_crc)   |
//! +-------------------+------------+----------------------------------+
//! ```

use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Standard BoC magic number: 0xb5ee9c72
pub const BOC_MAGIC_STANDARD: u32 = 0xb5ee9c72;

/// BoC with index magic: 0x68ff65f3
pub const BOC_MAGIC_IDX: u32 = 0x68ff65f3;

/// BoC with CRC32C magic: 0xacc3a728
pub const BOC_MAGIC_CRC32C: u32 = 0xacc3a728;

/// Maximum cell size in bytes (1023 bits = 128 bytes rounded up)
const MAX_CELL_SIZE: usize = 128;

/// Maximum number of cell references (4 per cell)
const MAX_CELL_REFS: usize = 4;

/// Represents a parsed TON cell
#[derive(Debug, Clone)]
pub struct Cell {
    /// Cell data (up to 1023 bits)
    pub data: Vec<u8>,

    /// Number of bits in data (0-1023)
    pub bit_len: u16,

    /// References to other cells (by index)
    pub refs: Vec<usize>,
}

impl Cell {
    /// Create a new empty cell
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            bit_len: 0,
            refs: Vec::new(),
        }
    }

    /// Read bits from the cell as bytes
    pub fn read_bits(&self, offset: usize, count: usize) -> Result<Vec<u8>> {
        if offset + count > self.bit_len as usize {
            return Err(DecoderError::invalid_structure(format!(
                "Cannot read {} bits at offset {} (cell has {} bits)",
                count, offset, self.bit_len
            )));
        }

        // For simplicity, only support byte-aligned reads for now
        if !offset.is_multiple_of(8) || !count.is_multiple_of(8) {
            return Err(DecoderError::invalid_structure(
                "Non-byte-aligned bit reads not yet implemented",
            ));
        }

        let byte_offset = offset / 8;
        let byte_count = count / 8;

        if byte_offset + byte_count > self.data.len() {
            return Err(DecoderError::invalid_structure("Read beyond cell data"));
        }

        Ok(self.data[byte_offset..byte_offset + byte_count].to_vec())
    }

    /// Get a reference to another cell
    pub fn get_ref(&self, index: usize) -> Result<usize> {
        self.refs.get(index).copied().ok_or_else(|| {
            DecoderError::invalid_structure(format!(
                "Cell reference {} not found (has {} refs)",
                index,
                self.refs.len()
            ))
        })
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a Bag of Cells from bytes
pub fn parse_boc(bytes: &[u8]) -> Result<Vec<Cell>> {
    let mut cursor = Cursor::new(bytes);

    // Read magic number (4 bytes, big-endian)
    let magic = read_u32_be(&mut cursor)?;

    // Verify magic number
    let (has_idx, has_crc32c) = match magic {
        BOC_MAGIC_STANDARD => (false, false),
        BOC_MAGIC_IDX => (true, false),
        BOC_MAGIC_CRC32C => (false, true),
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid BoC magic: 0x{:08x}",
                magic
            )))
        }
    };

    // Read flags byte
    let flags_byte = read_u8(&mut cursor)?;

    // Parse flags: has_idx(1) | has_crc32c(1) | has_cache_bits(1) | flags(2) | size(3)
    let _size = (flags_byte & 0x07) as usize; // Last 3 bits

    // Read offset size (1 byte)
    let off_bytes = read_u8(&mut cursor)? as usize;

    if off_bytes == 0 || off_bytes > 8 {
        return Err(DecoderError::invalid_structure(format!(
            "Invalid off_bytes: {} (must be 1-8)",
            off_bytes
        )));
    }

    // Read counts
    let cells_count = read_var_uint(&mut cursor, off_bytes)? as usize;
    let roots_count = read_var_uint(&mut cursor, off_bytes)? as usize;
    let _absent_count = read_var_uint(&mut cursor, off_bytes)? as usize;
    let _tot_cells_size = read_var_uint(&mut cursor, off_bytes)? as usize;

    if cells_count == 0 {
        return Err(DecoderError::invalid_structure("BoC has zero cells"));
    }

    if roots_count == 0 {
        return Err(DecoderError::invalid_structure("BoC has zero root cells"));
    }

    // Read root list
    let mut _root_list = Vec::with_capacity(roots_count);
    for _ in 0..roots_count {
        _root_list.push(read_var_uint(&mut cursor, off_bytes)? as usize);
    }

    // Skip index if present
    if has_idx {
        let index_size = cells_count * off_bytes;
        let mut _index = vec![0u8; index_size];
        cursor
            .read_exact(&mut _index)
            .map_err(|e| DecoderError::invalid_structure(format!("Failed to read index: {}", e)))?;
    }

    // Parse cells
    let mut cells = Vec::with_capacity(cells_count);
    for _ in 0..cells_count {
        let cell = parse_cell(&mut cursor)?;
        cells.push(cell);
    }

    // Verify CRC32C if present
    if has_crc32c {
        let _crc = read_u32_le(&mut cursor)?;
        // TODO: Implement CRC32C verification
    }

    Ok(cells)
}

/// Parse a single cell from the cursor
fn parse_cell(cursor: &mut Cursor<&[u8]>) -> Result<Cell> {
    // Read cell descriptor (2 bytes)
    let d1 = read_u8(cursor)?;
    let d2 = read_u8(cursor)?;

    // Parse descriptor
    let refs_count = (d1 & 0x07) as usize;
    let _is_exotic = (d1 & 0x08) != 0;
    let has_hashes = (d1 & 0x10) != 0;
    let level_mask = (d1 >> 5) & 0x07;

    // Bit length is in d2 (and possibly overflow in d1)
    let bit_len = if d2 == 0 {
        // Full bytes, calculate from next field
        0 // Will be determined from data size
    } else {
        d2 as u16
    };

    if refs_count > MAX_CELL_REFS {
        return Err(DecoderError::invalid_structure(format!(
            "Cell has {} refs (max {})",
            refs_count, MAX_CELL_REFS
        )));
    }

    // Skip hashes if present (32 bytes per level + 1)
    if has_hashes {
        let hash_count = level_mask.count_ones() as usize + 1;
        let hash_size = hash_count * 32;
        let mut _hashes = vec![0u8; hash_size];
        cursor.read_exact(&mut _hashes).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to read cell hashes: {}", e))
        })?;

        // Skip depths (2 bytes per level + 1)
        let depth_size = hash_count * 2;
        let mut _depths = vec![0u8; depth_size];
        cursor.read_exact(&mut _depths).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to read cell depths: {}", e))
        })?;
    }

    // Calculate data size
    let data_size = if bit_len == 0 {
        // Read size byte
        read_u8(cursor)? as usize
    } else {
        bit_len.div_ceil(8) as usize
    };

    if data_size > MAX_CELL_SIZE {
        return Err(DecoderError::invalid_structure(format!(
            "Cell data too large: {} bytes (max {})",
            data_size, MAX_CELL_SIZE
        )));
    }

    // Read cell data
    let mut data = vec![0u8; data_size];
    cursor
        .read_exact(&mut data)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read cell data: {}", e)))?;

    // Read cell references (indices to other cells)
    let mut refs = Vec::with_capacity(refs_count);
    for _ in 0..refs_count {
        // References are typically 1-4 bytes depending on cell count
        let ref_idx = read_u8(cursor)? as usize;
        refs.push(ref_idx);
    }

    let actual_bit_len = if bit_len == 0 {
        data_size as u16 * 8
    } else {
        bit_len
    };

    Ok(Cell {
        data,
        bit_len: actual_bit_len,
        refs,
    })
}

/// Read variable-length unsigned integer
fn read_var_uint(cursor: &mut Cursor<&[u8]>, size: usize) -> Result<u64> {
    let mut bytes = vec![0u8; size];
    cursor
        .read_exact(&mut bytes)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read var uint: {}", e)))?;

    let mut result = 0u64;
    for &byte in &bytes {
        result = (result << 8) | (byte as u64);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boc_magic() {
        assert_eq!(BOC_MAGIC_STANDARD, 0xb5ee9c72);
    }

    #[test]
    fn test_cell_new() {
        let cell = Cell::new();
        assert_eq!(cell.data.len(), 0);
        assert_eq!(cell.bit_len, 0);
        assert_eq!(cell.refs.len(), 0);
    }

    #[test]
    fn test_parse_boc_empty() {
        let result = parse_boc(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_boc_invalid_magic() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00];
        let result = parse_boc(&bytes);
        assert!(result.is_err());
    }
}
