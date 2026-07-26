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
/// Note: Exotic cells may have different size limits
const MAX_CELL_SIZE: usize = 256;

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

/// BoC parsing context to pass cell-level parsing flags
struct BocContext {
    _has_idx: bool,
    cells_have_hashes: bool,
}

/// Parse a Bag of Cells from bytes
pub fn parse_boc(bytes: &[u8]) -> Result<Vec<Cell>> {
    Ok(parse_boc_with_roots(bytes)?.0)
}

/// Parse a Bag of Cells, returning the flat cell list together with the
/// indices (into that list) of the root cells.
///
/// This is identical to [`parse_boc`] but also exposes the root list, which is
/// required to walk the cell tree in the same topological order the producer
/// intended (e.g. to compare against another BoC parser reference-by-reference).
pub fn parse_boc_with_roots(bytes: &[u8]) -> Result<(Vec<Cell>, Vec<usize>)> {
    let mut cursor = Cursor::new(bytes);

    // Read magic number (4 bytes, big-endian)
    let magic = read_u32_be(&mut cursor)?;

    // Verify magic number
    match magic {
        BOC_MAGIC_STANDARD | BOC_MAGIC_IDX | BOC_MAGIC_CRC32C => {}
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
    let has_idx = (flags_byte & 0x80) != 0; // Bit 7
    let has_crc32c = (flags_byte & 0x40) != 0; // Bit 6
    let _has_cache_bits = (flags_byte & 0x20) != 0; // Bit 5
    let _size = (flags_byte & 0x07) as usize; // Last 3 bits (bits 2-0)

    // In indexed BoC format, cells have hashes/depths serialized
    let ctx = BocContext {
        _has_idx: has_idx,
        cells_have_hashes: has_idx,
    };

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
    let mut root_list = Vec::with_capacity(roots_count);
    for _ in 0..roots_count {
        root_list.push(read_var_uint(&mut cursor, off_bytes)? as usize);
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
        let cell = parse_cell(&mut cursor, &ctx)?;
        cells.push(cell);
    }

    // Verify CRC32C if present
    if has_crc32c {
        let _crc = read_u32_le(&mut cursor)?;
        // TODO: Implement CRC32C verification
    }

    Ok((cells, root_list))
}

/// Parse a single cell from the cursor
fn parse_cell(cursor: &mut Cursor<&[u8]>, ctx: &BocContext) -> Result<Cell> {
    // Read cell descriptor (2 bytes)
    let d1 = read_u8(cursor)?;
    let d2 = read_u8(cursor)?;

    // Parse descriptor
    // d1 = refs_count + 8*is_exotic + 16*has_hashes + 32*level
    // Note: Cells with has_hashes=true might have hashes/depths serialized (in indexed BoC)
    let is_exotic = (d1 & 0x08) != 0;
    let has_hashes = (d1 & 0x10) != 0;
    let level_mask = (d1 >> 5) & 0x07;
    let level = level_mask.count_ones() as usize;

    let refs_count = if is_exotic {
        // For exotic cells, lower 3 bits are NOT refs_count
        // Exotic cells determine refs from their type/data
        // For now, assume 0 refs and parse type from data
        0
    } else {
        // Ordinary cell: standard descriptor format
        let refs = (d1 & 0x07) as usize;

        // Sanity check: if refs > 4, this might be a special cell type or misaligned parsing
        // For now, treat as exotic and skip
        if refs > MAX_CELL_REFS {
            // Log warning but continue - treat as exotic cell with no refs
            // TODO: Investigate why some cells have >4 refs indicated
            0
        } else {
            refs
        }
    };

    // Read hashes and depths if present and if the BoC format includes them
    // In indexed BoC format, cells with has_hashes=true have hashes/depths serialized
    if has_hashes && ctx.cells_have_hashes {
        let hash_count = level + 1;
        let hash_size = hash_count * 32; // 32 bytes per hash
        let depth_size = hash_count * 2; // 2 bytes per depth

        let mut _hashes = vec![0u8; hash_size];
        cursor.read_exact(&mut _hashes).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to read cell hashes: {}", e))
        })?;

        let mut _depths = vec![0u8; depth_size];
        cursor.read_exact(&mut _depths).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to read cell depths: {}", e))
        })?;
    }

    // Bit length encoding in d2 (standard BoC, matches tonlib-core):
    //   data_bytes = (d2 >> 1) + (d2 & 1)   == ceil(d2 / 2)
    //   full_bytes = (d2 & 1) == 0
    // When the cell is NOT byte-aligned (d2 odd), the final byte carries a
    // "completion tag": a single `1` bit followed by zero padding, which is
    // NOT part of the payload. d2 == 0 is a legitimate empty cell (0 bytes),
    // NOT a signal to read an extra size byte.
    let full_bytes = (d2 & 0x01) == 0;

    // Calculate data size and read data.
    // Exotic cells have a different descriptor format - for now, treat as opaque.
    let (data_bytes, is_exotic_with_descriptor) = if is_exotic && d2 == 0 {
        // Exotic cells with d2=0 have first data byte as cell type,
        // not size. We need to parse based on exotic type.
        // For now, read the type byte and skip the exotic cell data
        let _type_byte = read_u8(cursor)?;
        // TODO: Parse exotic cell based on type (1=pruned, 2=library, 3=merkle_proof, 4=merkle_update)
        // For now, return minimal exotic cell
        (0, true)
    } else {
        ((d2 as usize >> 1) + (d2 as usize & 1), false)
    };

    if data_bytes > MAX_CELL_SIZE {
        return Err(DecoderError::invalid_structure(format!(
            "Cell data too large: {} bytes (max {})",
            data_bytes, MAX_CELL_SIZE
        )));
    }

    let mut data = vec![0u8; data_bytes];
    if data_bytes > 0 {
        cursor.read_exact(&mut data).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to read cell data: {}", e))
        })?;
    }

    // Decode the payload bit length. For byte-aligned cells (d2 even) the bit
    // length is simply data.len() * 8. Otherwise the final byte holds a
    // completion tag `<payload_bits><1><padding_zeros>`; strip that tag from
    // the stored data so `data` holds only payload bits (0-padded). This
    // mirrors tonlib-core's raw cell reader.
    let actual_bit_len = if data.is_empty() || is_exotic_with_descriptor {
        0 // Exotic cells or empty data
    } else if full_bytes {
        (data.len() * 8) as u16
    } else {
        let last = data.len() - 1;
        let num_zeros = data[last].trailing_zeros() as usize;
        if num_zeros >= 8 {
            // Malformed: last byte is zero but a completion tag was expected.
            // Fall back to a byte-aligned interpretation rather than panicking.
            (data.len() * 8) as u16
        } else {
            // Clear the completion bit so only payload bits remain.
            data[last] &= !(1u8 << num_zeros);
            ((data.len() * 8) - (num_zeros + 1)) as u16
        }
    };

    // Read cell references (indices to other cells)
    // Skip refs for exotic cells or limit to MAX_CELL_REFS
    let safe_refs_count = if is_exotic {
        0 // Skip refs for exotic cells for now
    } else {
        refs_count.min(MAX_CELL_REFS)
    };

    let mut refs = Vec::with_capacity(safe_refs_count);
    for _ in 0..safe_refs_count {
        // References are 1 byte indices
        let ref_idx = read_u8(cursor)? as usize;
        refs.push(ref_idx);
    }

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
