//! TON transaction types and parsing
//!
//! This module defines the transaction structure for TON blockchain
//! and implements parsing from cell format.

use crate::boc::Cell;
use decoder_primitives::prelude::*;

/// Parsed TON transaction
#[derive(Debug, Clone)]
pub struct TonTransaction {
    /// Raw BoC bytes
    pub raw_bytes: Vec<u8>,

    /// Parsed cells from BoC
    pub cells: Vec<Cell>,

    /// Account address (256 bits = 32 bytes)
    pub account_addr: Vec<u8>,

    /// Logical time
    pub lt: u64,

    /// Previous transaction hash (256 bits = 32 bytes)
    pub prev_trans_hash: Vec<u8>,

    /// Previous transaction logical time
    pub prev_trans_lt: u64,

    /// Unix timestamp (seconds)
    pub now: u32,

    /// Output messages count
    pub outmsg_cnt: u16,
}

/// Intermediate transaction data parsed from cell
#[derive(Debug)]
pub(crate) struct TxData {
    pub account_addr: Vec<u8>,
    pub lt: u64,
    pub prev_trans_hash: Vec<u8>,
    pub prev_trans_lt: u64,
    pub now: u32,
    pub outmsg_cnt: u16,
}

/// Parse transaction from the root cell
///
/// Transaction TL-B schema:
/// ```text
/// transaction$0111
///   account_addr:bits256
///   lt:uint64
///   prev_trans_hash:bits256
///   prev_trans_lt:uint64
///   now:uint32
///   outmsg_cnt:uint15
///   ...
/// ```
pub(crate) fn parse_transaction(cell: &Cell) -> Result<TxData> {
    // Transaction tag should be 0111 (binary) = 0x7
    let tag_bits = 4;

    // For now, implement a simplified parser that extracts key fields
    // Full TL-B parsing would require a complete bit-level reader

    // Verify minimum cell size
    if cell.bit_len < 256 + 64 + 256 + 64 + 32 + 15 + 4 {
        return Err(DecoderError::invalid_structure(format!(
            "Transaction cell too small: {} bits (expected at least {} bits)",
            cell.bit_len,
            256 + 64 + 256 + 64 + 32 + 15 + 4
        )));
    }

    let mut _bit_offset = 0;

    // Skip tag (4 bits) - for byte-aligned reading, we'll skip this for now
    // In production, would need proper bit-level reading
    _bit_offset += tag_bits;

    // For simplified parsing, assume byte-aligned start after tag
    // In real implementation, would use bit-level reader

    // Read account address (256 bits = 32 bytes)
    let account_addr = if cell.data.len() >= 32 {
        cell.data[0..32].to_vec()
    } else {
        return Err(DecoderError::invalid_structure(
            "Insufficient data for account address",
        ));
    };

    // Read logical time (64 bits = 8 bytes, big-endian in TON)
    let lt = if cell.data.len() >= 40 {
        u64::from_be_bytes([
            cell.data[32],
            cell.data[33],
            cell.data[34],
            cell.data[35],
            cell.data[36],
            cell.data[37],
            cell.data[38],
            cell.data[39],
        ])
    } else {
        return Err(DecoderError::invalid_structure(
            "Insufficient data for logical time",
        ));
    };

    // Read prev_trans_hash (256 bits = 32 bytes)
    let prev_trans_hash = if cell.data.len() >= 72 {
        cell.data[40..72].to_vec()
    } else {
        return Err(DecoderError::invalid_structure(
            "Insufficient data for prev_trans_hash",
        ));
    };

    // Read prev_trans_lt (64 bits = 8 bytes)
    let prev_trans_lt = if cell.data.len() >= 80 {
        u64::from_be_bytes([
            cell.data[72],
            cell.data[73],
            cell.data[74],
            cell.data[75],
            cell.data[76],
            cell.data[77],
            cell.data[78],
            cell.data[79],
        ])
    } else {
        return Err(DecoderError::invalid_structure(
            "Insufficient data for prev_trans_lt",
        ));
    };

    // Read now (32 bits = 4 bytes)
    let now = if cell.data.len() >= 84 {
        u32::from_be_bytes([cell.data[80], cell.data[81], cell.data[82], cell.data[83]])
    } else {
        return Err(DecoderError::invalid_structure(
            "Insufficient data for timestamp",
        ));
    };

    // Read outmsg_cnt (15 bits, we'll read as 16 bits for simplicity)
    let outmsg_cnt = if cell.data.len() >= 86 {
        u16::from_be_bytes([cell.data[84], cell.data[85]]) & 0x7FFF // Mask to 15 bits
    } else {
        return Err(DecoderError::invalid_structure(
            "Insufficient data for outmsg_cnt",
        ));
    };

    Ok(TxData {
        account_addr,
        lt,
        prev_trans_hash,
        prev_trans_lt,
        now,
        outmsg_cnt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transaction_insufficient_data() {
        let cell = Cell {
            data: vec![0u8; 10], // Too small
            bit_len: 80,
            refs: vec![],
        };

        let result = parse_transaction(&cell);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_transaction_minimal() {
        // Create a cell with minimal transaction data
        let mut data = Vec::new();

        // Account address (32 bytes)
        data.extend_from_slice(&[1u8; 32]);

        // Logical time (8 bytes)
        data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 42]);

        // Prev trans hash (32 bytes)
        data.extend_from_slice(&[2u8; 32]);

        // Prev trans lt (8 bytes)
        data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 41]);

        // Timestamp (4 bytes)
        data.extend_from_slice(&[0x65, 0x00, 0x00, 0x00]); // ~2024 timestamp

        // Outmsg count (2 bytes, 15 bits)
        data.extend_from_slice(&[0x00, 0x05]);

        // Add extra bytes to satisfy minimum bit requirement (691 bits minimum)
        // We have 86 bytes so far = 688 bits, need at least 691 bits
        // Add 1 more byte to exceed minimum
        data.push(0x00);

        let cell = Cell {
            data,
            bit_len: (32 + 8 + 32 + 8 + 4 + 2 + 1) * 8, // Updated to include extra byte
            refs: vec![],
        };

        let tx = parse_transaction(&cell).expect("Failed to parse transaction");

        assert_eq!(tx.account_addr.len(), 32);
        assert_eq!(tx.lt, 42);
        assert_eq!(tx.prev_trans_hash.len(), 32);
        assert_eq!(tx.prev_trans_lt, 41);
        assert_eq!(tx.outmsg_cnt, 5);
    }
}
