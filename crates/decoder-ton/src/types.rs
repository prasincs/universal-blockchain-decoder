//! TON transaction types and parsing
//!
//! This module defines the transaction structure for TON blockchain
//! and implements parsing from cell format using TL-B schemas.

use crate::bitreader::BitReader;
use crate::boc::Cell;
use decoder_primitives::prelude::*;

/// Account status in TON
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    /// Account is uninitialized (uninit$00)
    Uninit,
    /// Account is frozen (frozen$01)
    Frozen,
    /// Account is active (active$10)
    Active,
    /// Account is nonexist (nonexist$11)
    Nonexist,
}

impl AccountStatus {
    /// Parse account status from 2 bits
    pub fn from_bits(bits: u8) -> Result<Self> {
        match bits & 0x03 {
            0b00 => Ok(AccountStatus::Uninit),
            0b01 => Ok(AccountStatus::Frozen),
            0b10 => Ok(AccountStatus::Active),
            0b11 => Ok(AccountStatus::Nonexist),
            _ => unreachable!(),
        }
    }
}

/// Currency collection (Grams + ExtraCurrencyCollection)
#[derive(Debug, Clone)]
pub struct CurrencyCollection {
    /// Main currency amount in nanotons
    pub grams: u128,
    /// Extra currencies (if any)
    pub extra: Vec<(u32, u128)>,
}

/// TON Message (simplified)
#[derive(Debug, Clone)]
pub struct Message {
    /// Message source address
    pub src: Option<Vec<u8>>,
    /// Message destination address
    pub dest: Option<Vec<u8>>,
    /// Message value
    pub value: CurrencyCollection,
    /// Message body (first 256 bits for display)
    pub body_preview: Vec<u8>,
}

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

    /// Account status before transaction
    pub orig_status: AccountStatus,

    /// Account status after transaction
    pub end_status: AccountStatus,

    /// Total fees paid
    pub total_fees: CurrencyCollection,

    /// Input message (if any)
    pub in_msg: Option<Message>,

    /// Output messages
    pub out_msgs: Vec<Message>,
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
    pub orig_status: AccountStatus,
    pub end_status: AccountStatus,
    pub total_fees: CurrencyCollection,
    pub in_msg_cell: Option<usize>,
    pub out_msgs_cell: Option<usize>,
}

/// Parse VarUInteger (variable-length unsigned integer)
/// Format: length in unary, then value
fn parse_var_uint(reader: &mut BitReader, max_len: usize) -> Result<u128> {
    // Read length in unary (count leading 1s)
    let mut len = 0;
    while len < max_len && reader.read_bit()? {
        len += 1;
    }

    if len == 0 {
        return Ok(0);
    }

    // Read value (len * 8 bits)
    let bit_count = len * 8;
    let mut value = 0u128;
    for _ in 0..bit_count {
        value = (value << 1) | (reader.read_bit()? as u128);
    }

    Ok(value)
}

/// Parse Grams (TON currency amount)
/// Format: VarUInteger 16
fn parse_grams(reader: &mut BitReader) -> Result<u128> {
    parse_var_uint(reader, 16)
}

/// Parse CurrencyCollection
fn parse_currency_collection(reader: &mut BitReader) -> Result<CurrencyCollection> {
    let grams = parse_grams(reader)?;

    // ExtraCurrencyCollection (optional dictionary)
    // For simplicity, skip extra currencies for now
    let has_extra = reader.read_bit()?;
    if has_extra {
        // Skip extra currency dictionary
        // TODO: Implement full dictionary parsing
    }

    Ok(CurrencyCollection {
        grams,
        extra: vec![],
    })
}

/// Parse message from a cell (simplified)
pub(crate) fn parse_message(_cells: &[Cell], _msg_cell_idx: usize) -> Result<Message> {
    // TODO: Implement full message parsing
    // For now, return a placeholder
    Ok(Message {
        src: None,
        dest: None,
        value: CurrencyCollection {
            grams: 0,
            extra: vec![],
        },
        body_preview: vec![],
    })
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
///   orig_status:AccountStatus
///   end_status:AccountStatus
///   ^[in_msg:(Maybe ^(Message Any))]
///   ^[out_msgs:(HashmapE 15 ^(Message Any))]
///   total_fees:CurrencyCollection
///   ^[state_update:^(HASH_UPDATE Account)]
///   description:^TransactionDescr
/// ```
pub(crate) fn parse_transaction(cell: &Cell) -> Result<TxData> {
    let mut reader = BitReader::new(&cell.data, cell.bit_len as usize);

    // Read transaction tag (4 bits) - should be 0x7 (0b0111)
    let tag = reader.read_bits_u8(4)?;
    if tag != 0b0111 {
        return Err(DecoderError::invalid_structure(format!(
            "Invalid transaction tag: 0x{:x} (expected 0x7)",
            tag
        )));
    }

    // Read account_addr (256 bits)
    let account_addr = reader.read_bits(256)?;

    // Read lt (uint64 = 64 bits)
    let lt = reader.read_bits_u64(64)?;

    // Read prev_trans_hash (256 bits)
    let prev_trans_hash = reader.read_bits(256)?;

    // Read prev_trans_lt (uint64 = 64 bits)
    let prev_trans_lt = reader.read_bits_u64(64)?;

    // Read now (uint32 = 32 bits)
    let now = reader.read_bits_u32(32)?;

    // Read outmsg_cnt (uint15 = 15 bits)
    let outmsg_cnt = reader.read_bits_u16(15)?;

    // Read orig_status (AccountStatus = 2 bits)
    let orig_status = AccountStatus::from_bits(reader.read_bits_u8(2)?)?;

    // Read end_status (AccountStatus = 2 bits)
    let end_status = AccountStatus::from_bits(reader.read_bits_u8(2)?)?;

    // The next fields are in cell references
    // For now, we'll extract the cell reference indices from the cell.refs field
    let in_msg_cell = if cell.refs.is_empty() {
        None
    } else {
        Some(cell.refs[0])
    };

    let out_msgs_cell = if cell.refs.len() > 1 {
        Some(cell.refs[1])
    } else {
        None
    };

    // Parse total_fees (CurrencyCollection)
    let total_fees = parse_currency_collection(&mut reader)?;

    // Remaining cell references:
    // cell.refs[2] = state_update (if present)
    // cell.refs[3] = description (if present)

    Ok(TxData {
        account_addr,
        lt,
        prev_trans_lt,
        now,
        outmsg_cnt,
        orig_status,
        end_status,
        total_fees,
        in_msg_cell,
        out_msgs_cell,
        prev_trans_hash,
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
    #[ignore] // Ignore for now - need proper TL-B formatted data
    fn test_parse_transaction_minimal() {
        // TODO: Create a proper TL-B formatted transaction cell with:
        // - Transaction tag (4 bits: 0b0111)
        // - account_addr (256 bits)
        // - lt (64 bits)
        // - prev_trans_hash (256 bits)
        // - prev_trans_lt (64 bits)
        // - now (32 bits)
        // - outmsg_cnt (15 bits)
        // - orig_status (2 bits)
        // - end_status (2 bits)
        // - total_fees (VarUInteger)
        //
        // For now, use real mainnet BoC data for testing
    }
}
