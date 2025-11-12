//! Solana transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for Solana transactions, transforming them
//! from their native format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. The `solana-sdk` and `solana-parser` crates are
//! used only in `dev-dependencies` for validation testing.
//!
//! ## Transaction Format Support
//!
//! - ✅ Legacy transactions (v0)
//! - ✅ Message parsing (header, accounts, blockhash, instructions)
//! - ✅ Instruction extraction
//! - 🚧 Versioned transactions (v1 - future work)
//! - 🚧 Address lookup tables (future work)
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_solana::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_bytes = &[...]; // Solana transaction bytes
//!
//! let decoded = SolanaDecoder::decode(tx_bytes)?;
//!
//! // Access the parsed message and instructions
//! println!("Instructions: {}", decoded.message.num_instructions());
//! for instruction in decoded.instructions() {
//!     println!("Program ID index: {}", instruction.program_id_index);
//!     println!("Accounts: {:?}", instruction.accounts);
//!     println!("Data: {:?}", instruction.data);
//! }
//! ```

use decoder_primitives::prelude::*;
use std::io::Cursor;

pub mod parsing;
pub mod types;

use parsing::*;
pub use types::{CompiledInstruction, Message, MessageHeader, SolanaTransaction};

/// Solana chain identity
///
/// This type implements `ChainIdentity` and is used to identify Solana transactions
/// in the universal decoder system.
#[derive(Debug, Clone, Copy)]
pub struct SolanaChain;

impl ChainIdentity for SolanaChain {
    fn chain_id(&self) -> u64 {
        // Solana's chain ID (not officially standardized, using common convention)
        // Mainnet-beta: could use 101 or similar
        101
    }

    fn chain_name(&self) -> &str {
        "Solana"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account // Solana uses account-based model
    }
}

/// Solana decoder implementing the ChainDecoder trait
///
/// This decoder uses a pure Rust implementation to parse Solana transactions
/// without depending on external blockchain libraries in production.
pub struct SolanaDecoder;

impl ChainDecoder for SolanaDecoder {
    type TxSpecific = SolanaTransaction;
    type Chain = SolanaChain;

    fn chain() -> Self::Chain {
        SolanaChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        let mut cursor = Cursor::new(raw_bytes);

        // Parse signatures (compact-u16 length + array of 64-byte signatures)
        let signatures = parse_signatures(&mut cursor)?;

        // Parse message (header + accounts + blockhash + instructions)
        let message = parse_message(&mut cursor)?;

        // Verify we consumed all bytes
        let consumed = cursor.position() as usize;
        if consumed != raw_bytes.len() {
            return Err(DecoderError::invalid_structure(format!(
                "Transaction has {} trailing bytes (consumed {}, total {})",
                raw_bytes.len() - consumed,
                consumed,
                raw_bytes.len()
            )));
        }

        let tx = SolanaTransaction {
            signatures,
            message,
            raw_bytes: raw_bytes.to_vec(),
        };

        // Final validation
        if !tx.is_valid() {
            return Err(DecoderError::invalid_structure(
                "Transaction validation failed: signature count mismatch or invalid message",
            ));
        }

        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Solana transaction cannot be empty",
            ));
        }

        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(format!(
                "Solana transaction too small: {} bytes (minimum ~10 bytes)",
                raw_bytes.len()
            )));
        }

        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Solana transaction too large: {} bytes (maximum {} bytes)",
                raw_bytes.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain() {
        let chain = SolanaDecoder::chain();
        assert_eq!(chain.chain_id(), 101);
        assert_eq!(chain.chain_name(), "Solana");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(SolanaDecoder::validate_format(&[]).is_err());

        // Too small transaction should fail
        assert!(SolanaDecoder::validate_format(&[0x01]).is_err());

        // Too large transaction should fail
        let huge_tx = vec![0u8; MAX_TRANSACTION_SIZE + 1];
        assert!(SolanaDecoder::validate_format(&huge_tx).is_err());

        // Reasonable size should pass basic validation
        let dummy_tx = vec![0u8; 100];
        assert!(SolanaDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_decode_minimal_transaction() {
        // Create a minimal valid Solana transaction
        let mut tx_bytes = vec![];

        // Signatures: 1 signature (compact-u16 = 0x01)
        tx_bytes.push(0x01);
        // Signature (64 bytes of zeros)
        tx_bytes.extend_from_slice(&[0u8; 64]);

        // Message header:
        // - num_required_signatures: 1
        tx_bytes.push(0x01);
        // - num_readonly_signed_accounts: 0
        tx_bytes.push(0x00);
        // - num_readonly_unsigned_accounts: 1 (for the program)
        tx_bytes.push(0x01);

        // Account keys: 2 accounts (compact-u16 = 0x02)
        tx_bytes.push(0x02);
        // Account 0: signer (32 bytes)
        tx_bytes.extend_from_slice(&[1u8; 32]);
        // Account 1: program (32 bytes)
        tx_bytes.extend_from_slice(&[2u8; 32]);

        // Recent blockhash (32 bytes)
        tx_bytes.extend_from_slice(&[0u8; 32]);

        // Instructions: 1 instruction (compact-u16 = 0x01)
        tx_bytes.push(0x01);
        // Instruction 0:
        // - program_id_index: 1
        tx_bytes.push(0x01);
        // - accounts: 1 account (compact-u16 = 0x01)
        tx_bytes.push(0x01);
        // - account index: 0
        tx_bytes.push(0x00);
        // - data: empty (compact-u16 = 0x00)
        tx_bytes.push(0x00);

        let decoded =
            SolanaDecoder::decode(&tx_bytes).expect("Failed to decode minimal transaction");

        assert_eq!(decoded.num_signatures(), 1);
        assert_eq!(decoded.message.num_account_keys(), 2);
        assert_eq!(decoded.message.num_instructions(), 1);
        assert_eq!(decoded.message.header.num_required_signatures, 1);
        assert!(decoded.is_valid());
    }

    #[test]
    fn test_decode_invalid_empty() {
        let empty = vec![];
        assert!(SolanaDecoder::decode(&empty).is_err());
    }

    #[test]
    fn test_decode_invalid_truncated() {
        let truncated = vec![0x01, 0x00, 0x00]; // Only 3 bytes
        assert!(SolanaDecoder::decode(&truncated).is_err());
    }
}
