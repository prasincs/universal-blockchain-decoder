//! TON (The Open Network) transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for TON blockchain transactions, transforming them
//! from their native Bag of Cells (BoC) format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. The `tonlib` crate is used only in `dev-dependencies`
//! for validation testing.
//!
//! ## TON Transaction Format
//!
//! TON uses a unique cell-based data structure called Bag of Cells (BoC):
//! - **Cell**: Fundamental data structure, can store up to 1023 bits and 4 references
//! - **BoC**: Serialization format for cells, with magic number 0xb5ee9c72
//! - **Transaction**: Encoded as a cell with TL-B schema
//!
//! ### BoC Header Format
//!
//! ```text
//! Magic:        4 bytes (0xb5ee9c72)
//! Flags:        1 byte  (has_idx | has_crc32c | has_cache_bits | flags:2 | size:3)
//! OffBytes:     1 byte  (offset size in bytes)
//! Cells:        OffBytes (number of cells)
//! Roots:        OffBytes (number of root cells)
//! Absent:       OffBytes (number of absent cells)
//! TotCellsSize: OffBytes (total cells size)
//! RootList:     Roots * OffBytes (root cell indices)
//! Index:        optional (if has_idx)
//! CellData:     variable (all cell data)
//! CRC32C:       optional 4 bytes (if has_crc32c)
//! ```
//!
//! ### Transaction TL-B Schema
//!
//! ```text
//! transaction$0111
//!   account_addr:bits256
//!   lt:uint64
//!   prev_trans_hash:bits256
//!   prev_trans_lt:uint64
//!   now:uint32
//!   outmsg_cnt:uint15
//!   orig_status:AccountStatus
//!   end_status:AccountStatus
//!   ^[in_msg:(Maybe ^(Message Any))]
//!   ^[out_msgs:(HashmapE 15 ^(Message Any))]
//!   total_fees:CurrencyCollection
//!   ^[state_update:^(HASH_UPDATE Account)]
//!   description:^TransactionDescr
//!   = Transaction;
//! ```
//!
//! ## Chain Family
//!
//! TON uses a **message-passing actor model**:
//! - Accounts send and receive messages
//! - Transactions represent state changes from message processing
//! - Similar to Actor Model systems (ICP, AO)
//!
//! Currently classified as `ChainFamily::Account` (will update to `ChainFamily::Actor` when available).
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_ton::*;
//! use decoder_primitives::prelude::*;
//!
//! let boc_bytes = &[...]; // TON transaction in BoC format
//!
//! let decoded = TonDecoder::decode(boc_bytes)?;
//!
//! // Access parsed transaction data
//! println!("Account: {}", decoded.account_addr);
//! println!("Logical time: {}", decoded.lt);
//! println!("Timestamp: {}", decoded.now);
//! ```

use decoder_primitives::prelude::*;

mod bitreader;
pub mod boc;
pub mod types;

pub use types::{
    AccountStatus, CommonMsgInfo, CurrencyCollection, Message, MsgAddress, TonTransaction,
};

/// TON chain identity
///
/// This type implements `ChainIdentity` for The Open Network (TON).
/// TON uses a unique architecture with workchains:
/// - Masterchain (workchain_id = -1): Coordination and validation
/// - Basechain (workchain_id = 0): General-purpose smart contracts
#[derive(Debug, Clone, Copy)]
pub struct TonChain;

impl ChainIdentity for TonChain {
    fn chain_id(&self) -> u64 {
        // TON mainnet chain ID (using SLIP-44 coin type)
        607
    }

    fn chain_name(&self) -> &str {
        "TON"
    }

    fn chain_family(&self) -> ChainFamily {
        // TON uses message-passing actor model
        // Using Account for now; will update to Actor when available
        ChainFamily::Account
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet")
    }
}

/// TON decoder implementing the ChainDecoder trait
///
/// This decoder uses a pure Rust implementation to parse TON transactions
/// from Bag of Cells (BoC) format without depending on external blockchain
/// libraries in production.
pub struct TonDecoder;

impl ChainDecoder for TonDecoder {
    type TxSpecific = TonTransaction;
    type Chain = TonChain;

    fn chain() -> Self::Chain {
        TonChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        // Parse BoC to extract transaction cell
        let cells = boc::parse_boc(raw_bytes)?;

        if cells.is_empty() {
            return Err(DecoderError::invalid_structure("BoC contains no cells"));
        }

        // The root cell should be the transaction
        let tx_cell = &cells[0];

        // Parse transaction from cell
        let tx = types::parse_transaction(tx_cell)?;

        // Parse messages from cell references
        let in_msg = if let Some(msg_idx) = tx.in_msg_cell {
            if msg_idx < cells.len() {
                types::parse_message(&cells, msg_idx).ok()
            } else {
                None
            }
        } else {
            None
        };

        let out_msgs = if let Some(msgs_idx) = tx.out_msgs_cell {
            if msgs_idx < cells.len() {
                // For now, return empty vec - full hashmap parsing TODO
                vec![]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Ok(TonTransaction {
            raw_bytes: raw_bytes.to_vec(),
            cells,
            account_addr: tx.account_addr,
            lt: tx.lt,
            prev_trans_hash: tx.prev_trans_hash,
            prev_trans_lt: tx.prev_trans_lt,
            now: tx.now,
            outmsg_cnt: tx.outmsg_cnt,
            orig_status: tx.orig_status,
            end_status: tx.end_status,
            total_fees: tx.total_fees,
            in_msg,
            out_msgs,
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "TON transaction cannot be empty",
            ));
        }

        if raw_bytes.len() < 4 {
            return Err(DecoderError::invalid_structure(format!(
                "TON BoC too small: {} bytes (minimum 4 bytes for magic)",
                raw_bytes.len()
            )));
        }

        // Check magic number (0xb5ee9c72 for standard BoC)
        let magic = u32::from_be_bytes([raw_bytes[0], raw_bytes[1], raw_bytes[2], raw_bytes[3]]);

        if magic != boc::BOC_MAGIC_STANDARD
            && magic != boc::BOC_MAGIC_IDX
            && magic != boc::BOC_MAGIC_CRC32C
        {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid BoC magic number: 0x{:08x} (expected 0xb5ee9c72, 0x68ff65f3, or 0xacc3a728)",
                magic
            )));
        }

        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for TonTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let metadata = TxMetadata {
            tx_hash: self.prev_trans_hash.to_vec(),
            block_height: None,
            timestamp: Some(self.now as u64),
            size: self.raw_bytes.len(),
            extra: format!("lt:{},outmsg_cnt:{}", self.lt, self.outmsg_cnt),
        };

        let authorization = AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::EdDsa,
        };

        let state_deltas = StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![AccountChange {
                address: Address {
                    bytes: self.account_addr.to_vec(),
                    human_readable: None,
                },
                nonce: Some(self.lt),
                balance_change: 0,
                storage_changes: vec![],
            }],
        };

        Ok(TxIR::new(
            &TonChain,
            metadata,
            authorization,
            vec![], // operations - will populate with message parsing
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Basic validation
        if self.account_addr.len() != 32 {
            return Err(DecoderError::invalid_structure(
                "Account address must be 32 bytes",
            ));
        }

        if self.prev_trans_hash.len() != 32 {
            return Err(DecoderError::invalid_structure(
                "Previous transaction hash must be 32 bytes",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain() {
        let chain = TonDecoder::chain();
        assert_eq!(chain.chain_id(), 607);
        assert_eq!(chain.chain_name(), "TON");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format_empty() {
        assert!(TonDecoder::validate_format(&[]).is_err());
    }

    #[test]
    fn test_validate_format_too_small() {
        assert!(TonDecoder::validate_format(&[0x01]).is_err());
    }

    #[test]
    fn test_validate_format_invalid_magic() {
        let invalid = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x02];
        assert!(TonDecoder::validate_format(&invalid).is_err());
    }

    #[test]
    fn test_validate_format_valid_magic() {
        // Standard BoC magic: 0xb5ee9c72
        let valid = vec![0xb5, 0xee, 0x9c, 0x72, 0x01, 0x02];
        assert!(TonDecoder::validate_format(&valid).is_ok());
    }
}
