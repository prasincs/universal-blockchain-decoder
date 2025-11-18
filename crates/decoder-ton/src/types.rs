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

/// TON Address types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MsgAddress {
    /// No address (addr_none$00)
    None,
    /// External address (addr_extern$01)
    External {
        /// External address bits
        address: Vec<u8>,
        /// Length in bits
        bit_len: usize,
    },
    /// Standard internal address (addr_std$10)
    Standard {
        /// Anycast info (if present)
        anycast: Option<Vec<u8>>,
        /// Workchain ID (-128 to 127)
        workchain_id: i8,
        /// Account address (256 bits)
        address: Vec<u8>,
    },
    /// Variable-length internal address (addr_var$11)
    Variable {
        /// Anycast info (if present)
        anycast: Option<Vec<u8>>,
        /// Workchain ID (32-bit)
        workchain_id: i32,
        /// Account address (variable length)
        address: Vec<u8>,
        /// Address length in bits
        bit_len: usize,
    },
}

/// Message header types
#[derive(Debug, Clone)]
pub enum CommonMsgInfo {
    /// Internal message (int_msg_info$0)
    Internal {
        ihr_disabled: bool,
        bounce: bool,
        bounced: bool,
        src: MsgAddress,
        dest: MsgAddress,
        value: CurrencyCollection,
        ihr_fee: u128,
        fwd_fee: u128,
        created_lt: u64,
        created_at: u32,
    },
    /// External inbound message (ext_in_msg_info$10)
    ExternalIn {
        src: MsgAddress,
        dest: MsgAddress,
        import_fee: u128,
    },
    /// External outbound message (ext_out_msg_info$11)
    ExternalOut {
        src: MsgAddress,
        dest: MsgAddress,
        created_lt: u64,
        created_at: u32,
    },
}

/// TON Message
#[derive(Debug, Clone)]
pub struct Message {
    /// Message header
    pub info: CommonMsgInfo,
    /// StateInit (if present)
    pub init: Option<Vec<u8>>,
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

/// Parse Anycast (optional)
/// Format: Maybe Anycast = Maybe (anycast_info$_ depth:(#<= 30) ...)
fn parse_anycast(reader: &mut BitReader) -> Result<Option<Vec<u8>>> {
    let has_anycast = reader.read_bit()?;
    if has_anycast {
        // Read anycast depth (5 bits for value <= 30)
        let depth = reader.read_bits_u8(5)?;
        if depth > 30 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid anycast depth: {}",
                depth
            )));
        }
        // Read rewrite prefix (depth bits)
        let prefix = reader.read_bits(depth as usize)?;
        Ok(Some(prefix))
    } else {
        Ok(None)
    }
}

/// Parse MsgAddress
///
/// TL-B schemas:
/// ```text
/// addr_none$00 = MsgAddressExt;
/// addr_extern$01 len:(## 9) external_address:(bits len) = MsgAddressExt;
/// addr_std$10 anycast:(Maybe Anycast) workchain_id:int8 address:bits256 = MsgAddressInt;
/// addr_var$11 anycast:(Maybe Anycast) addr_len:(## 9) workchain_id:int32 address:(bits addr_len) = MsgAddressInt;
/// ```
fn parse_msg_address(reader: &mut BitReader) -> Result<MsgAddress> {
    // Read 2-bit tag
    let tag = reader.read_bits_u8(2)?;

    match tag {
        0b00 => {
            // addr_none$00
            Ok(MsgAddress::None)
        }
        0b01 => {
            // addr_extern$01 len:(## 9) external_address:(bits len)
            // ## 9 means 9-bit length field
            let len = reader.read_bits_u16(9)? as usize;
            if len > 0 {
                let address = reader.read_bits(len)?;
                Ok(MsgAddress::External {
                    address,
                    bit_len: len,
                })
            } else {
                Ok(MsgAddress::External {
                    address: vec![],
                    bit_len: 0,
                })
            }
        }
        0b10 => {
            // addr_std$10 anycast:(Maybe Anycast) workchain_id:int8 address:bits256
            let anycast = parse_anycast(reader)?;

            // Read workchain_id as signed 8-bit integer
            let wc_bits = reader.read_bits_u8(8)?;
            let workchain_id = wc_bits as i8;

            // Read 256-bit address
            let address = reader.read_bits(256)?;

            Ok(MsgAddress::Standard {
                anycast,
                workchain_id,
                address,
            })
        }
        0b11 => {
            // addr_var$11 anycast:(Maybe Anycast) addr_len:(## 9) workchain_id:int32 address:(bits addr_len)
            let anycast = parse_anycast(reader)?;

            // Read address length (9 bits)
            let addr_len = reader.read_bits_u16(9)? as usize;

            // Read workchain_id as signed 32-bit integer
            let wc_bits = reader.read_bits_u32(32)?;
            let workchain_id = wc_bits as i32;

            // Read variable-length address
            let address = if addr_len > 0 {
                reader.read_bits(addr_len)?
            } else {
                vec![]
            };

            Ok(MsgAddress::Variable {
                anycast,
                workchain_id,
                address,
                bit_len: addr_len,
            })
        }
        _ => unreachable!(),
    }
}

/// Parse CommonMsgInfo (message header)
///
/// TL-B schemas:
/// ```text
/// int_msg_info$0 ihr_disabled:Bool bounce:Bool bounced:Bool
///   src:MsgAddressInt dest:MsgAddressInt
///   value:CurrencyCollection ihr_fee:Grams fwd_fee:Grams
///   created_lt:uint64 created_at:uint32 = CommonMsgInfo;
///
/// ext_in_msg_info$10 src:MsgAddressExt dest:MsgAddressInt
///   import_fee:Grams = CommonMsgInfo;
///
/// ext_out_msg_info$11 src:MsgAddressInt dest:MsgAddressExt
///   created_lt:uint64 created_at:uint32 = CommonMsgInfo;
/// ```
fn parse_common_msg_info(reader: &mut BitReader) -> Result<CommonMsgInfo> {
    // Read first bit to distinguish int_msg_info from ext_*_msg_info
    let first_bit = reader.read_bit()?;

    if !first_bit {
        // int_msg_info$0
        let ihr_disabled = reader.read_bit()?;
        let bounce = reader.read_bit()?;
        let bounced = reader.read_bit()?;

        let src = parse_msg_address(reader)?;
        let dest = parse_msg_address(reader)?;

        let value = parse_currency_collection(reader)?;
        let ihr_fee = parse_grams(reader)?;
        let fwd_fee = parse_grams(reader)?;
        let created_lt = reader.read_bits_u64(64)?;
        let created_at = reader.read_bits_u32(32)?;

        Ok(CommonMsgInfo::Internal {
            ihr_disabled,
            bounce,
            bounced,
            src,
            dest,
            value,
            ihr_fee,
            fwd_fee,
            created_lt,
            created_at,
        })
    } else {
        // Read second bit to distinguish ext_in from ext_out
        let second_bit = reader.read_bit()?;

        if !second_bit {
            // ext_in_msg_info$10
            let src = parse_msg_address(reader)?;
            let dest = parse_msg_address(reader)?;
            let import_fee = parse_grams(reader)?;

            Ok(CommonMsgInfo::ExternalIn {
                src,
                dest,
                import_fee,
            })
        } else {
            // ext_out_msg_info$11
            let src = parse_msg_address(reader)?;
            let dest = parse_msg_address(reader)?;
            let created_lt = reader.read_bits_u64(64)?;
            let created_at = reader.read_bits_u32(32)?;

            Ok(CommonMsgInfo::ExternalOut {
                src,
                dest,
                created_lt,
                created_at,
            })
        }
    }
}

/// Parse message from a cell
///
/// TL-B schema:
/// ```text
/// message$_ {X:Type} info:CommonMsgInfo
///   init:(Maybe (Either StateInit ^StateInit))
///   body:(Either X ^X) = Message X;
/// ```
pub(crate) fn parse_message(cells: &[Cell], msg_cell_idx: usize) -> Result<Message> {
    if msg_cell_idx >= cells.len() {
        return Err(DecoderError::invalid_structure(format!(
            "Message cell index {} out of bounds (total cells: {})",
            msg_cell_idx,
            cells.len()
        )));
    }

    let cell = &cells[msg_cell_idx];
    let mut reader = BitReader::new(&cell.data, cell.bit_len as usize);

    // Parse CommonMsgInfo header
    let info = parse_common_msg_info(&mut reader)?;

    // Parse Maybe (Either StateInit ^StateInit)
    let has_init = reader.read_bit()?;
    let init = if has_init {
        // Either StateInit ^StateInit
        let init_inline = reader.read_bit()?;
        if !init_inline {
            // StateInit is inline - for simplicity, skip parsing full StateInit
            // Just read remaining bits as raw data
            // TODO: Implement full StateInit parsing
            let remaining = reader.remaining();
            if remaining > 0 {
                Some(reader.read_bits(remaining.min(256))?)
            } else {
                None
            }
        } else {
            // StateInit is in reference - would need to follow cell ref
            // For now, return placeholder
            None
        }
    } else {
        None
    };

    // Parse (Either X ^X) for body
    let body_inline = reader.read_bit()?;
    let body_preview = if !body_inline {
        // Body is inline
        let remaining = reader.remaining();
        if remaining > 0 {
            // Read first 256 bits max for preview
            reader.read_bits(remaining.min(256))?
        } else {
            vec![]
        }
    } else {
        // Body is in reference - follow cell ref if available
        if !cell.refs.is_empty() {
            let body_cell_idx = cell.refs[0];
            if body_cell_idx < cells.len() {
                let body_cell = &cells[body_cell_idx];
                let body_len = (body_cell.bit_len as usize).min(256);
                let mut body_reader = BitReader::new(&body_cell.data, body_len);
                if body_len > 0 {
                    body_reader.read_bits(body_len)?
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    };

    Ok(Message {
        info,
        init,
        body_preview,
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
