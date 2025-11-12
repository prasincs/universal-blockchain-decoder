//! Pure Rust parsing utilities for Solana transactions
//!
//! This module implements the binary format parsing for Solana transactions
//! without depending on external blockchain libraries.
//!
//! ## Binary Format
//!
//! Solana uses a compact binary format with variable-length integers (compact-u16)
//! for array lengths. The format is:
//!
//! ```text
//! Transaction:
//!   - signatures: compact-u16 length + array of 64-byte Ed25519 signatures
//!   - message: Message struct (see below)
//!
//! Message:
//!   - header: MessageHeader (3 bytes)
//!   - account_keys: compact-u16 length + array of 32-byte pubkeys
//!   - recent_blockhash: 32 bytes
//!   - instructions: compact-u16 length + array of CompiledInstruction
//!
//! MessageHeader:
//!   - num_required_signatures: u8
//!   - num_readonly_signed_accounts: u8
//!   - num_readonly_unsigned_accounts: u8
//!
//! CompiledInstruction:
//!   - program_id_index: u8
//!   - accounts: compact-u16 length + array of u8 indices
//!   - data: compact-u16 length + bytes
//! ```

use std::io::{Cursor, Read};
use universal_decoder_core::prelude::DecoderError;
use decoder_encodings::compact_u16::read_compact_u16;

use crate::types::{CompiledInstruction, Message, MessageHeader, SolanaBlockhash, SolanaPubkey, SolanaSignature};

/// Maximum transaction size (Solana's limit based on MTU)
pub const MAX_TRANSACTION_SIZE: usize = 1232;

/// Solana public key size (Ed25519)
pub const PUBKEY_SIZE: usize = 32;

/// Solana signature size (Ed25519)
pub const SIGNATURE_SIZE: usize = 64;

/// Blockhash size
pub const BLOCKHASH_SIZE: usize = 32;

/// Maximum number of signatures
pub const MAX_SIGNATURES: usize = 16;

/// Maximum number of account keys
pub const MAX_ACCOUNT_KEYS: usize = 256;

/// Maximum number of instructions
pub const MAX_INSTRUCTIONS: usize = 256;

type Result<T> = std::result::Result<T, DecoderError>;

/// Read a fixed-size byte array
pub fn read_bytes<const N: usize>(cursor: &mut Cursor<&[u8]>) -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read {} bytes: {}", N, e)))?;
    Ok(buf)
}

/// Read a single u8
pub fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

/// Read a variable-length byte vector (compact-u16 length prefix)
pub fn read_compact_array(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let len = read_compact_u16(cursor)?;
    let mut buf = vec![0u8; len as usize];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read compact array of {} bytes: {}", len, e)))?;
    Ok(buf)
}

/// Parse an array of signatures
pub fn parse_signatures(cursor: &mut Cursor<&[u8]>) -> Result<Vec<SolanaSignature>> {
    let num_signatures = read_compact_u16(cursor)?;

    if num_signatures as usize > MAX_SIGNATURES {
        return Err(DecoderError::invalid_structure(format!(
            "Too many signatures: {} (max {})",
            num_signatures, MAX_SIGNATURES
        )));
    }

    let mut signatures = Vec::with_capacity(num_signatures as usize);
    for _ in 0..num_signatures {
        signatures.push(read_bytes::<SIGNATURE_SIZE>(cursor)?.to_vec());
    }

    Ok(signatures)
}

/// Parse the message header (3 bytes)
pub fn parse_message_header(cursor: &mut Cursor<&[u8]>) -> Result<MessageHeader> {
    Ok(MessageHeader {
        num_required_signatures: read_u8(cursor)?,
        num_readonly_signed_accounts: read_u8(cursor)?,
        num_readonly_unsigned_accounts: read_u8(cursor)?,
    })
}

/// Parse an array of account keys (pubkeys)
pub fn parse_account_keys(cursor: &mut Cursor<&[u8]>) -> Result<Vec<SolanaPubkey>> {
    let num_accounts = read_compact_u16(cursor)?;

    if num_accounts as usize > MAX_ACCOUNT_KEYS {
        return Err(DecoderError::invalid_structure(format!(
            "Too many account keys: {} (max {})",
            num_accounts, MAX_ACCOUNT_KEYS
        )));
    }

    let mut account_keys = Vec::with_capacity(num_accounts as usize);
    for _ in 0..num_accounts {
        account_keys.push(read_bytes::<PUBKEY_SIZE>(cursor)?.to_vec());
    }

    Ok(account_keys)
}

/// Parse the recent blockhash
pub fn parse_blockhash(cursor: &mut Cursor<&[u8]>) -> Result<SolanaBlockhash> {
    Ok(read_bytes::<BLOCKHASH_SIZE>(cursor)?.to_vec())
}

/// Parse a single compiled instruction
pub fn parse_instruction(cursor: &mut Cursor<&[u8]>) -> Result<CompiledInstruction> {
    let program_id_index = read_u8(cursor)?;

    // Parse account indices
    let accounts_len = read_compact_u16(cursor)?;
    let mut accounts = Vec::with_capacity(accounts_len as usize);
    for _ in 0..accounts_len {
        accounts.push(read_u8(cursor)?);
    }

    // Parse instruction data
    let data = read_compact_array(cursor)?;

    Ok(CompiledInstruction {
        program_id_index,
        accounts,
        data,
    })
}

/// Parse an array of instructions
pub fn parse_instructions(cursor: &mut Cursor<&[u8]>) -> Result<Vec<CompiledInstruction>> {
    let num_instructions = read_compact_u16(cursor)?;

    if num_instructions as usize > MAX_INSTRUCTIONS {
        return Err(DecoderError::invalid_structure(format!(
            "Too many instructions: {} (max {})",
            num_instructions, MAX_INSTRUCTIONS
        )));
    }

    let mut instructions = Vec::with_capacity(num_instructions as usize);
    for i in 0..num_instructions {
        instructions.push(parse_instruction(cursor).map_err(|e| {
            DecoderError::chain_decoding(format!("Failed to parse instruction {}: {}", i, e))
        })?);
    }

    Ok(instructions)
}

/// Parse a complete Solana message
pub fn parse_message(cursor: &mut Cursor<&[u8]>) -> Result<Message> {
    let header = parse_message_header(cursor)?;
    let account_keys = parse_account_keys(cursor)?;
    let recent_blockhash = parse_blockhash(cursor)?;
    let instructions = parse_instructions(cursor)?;

    let message = Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    };

    // Validate the message structure
    if !message.is_valid() {
        return Err(DecoderError::invalid_structure(
            "Invalid message: instruction references invalid account indices",
        ));
    }

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: Tests for read_compact_u16 are now in decoder-encodings crate

    #[test]
    fn test_read_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data.as_slice());
        let result: [u8; 3] = read_bytes(&mut cursor).unwrap();
        assert_eq!(result, [1, 2, 3]);
        assert_eq!(cursor.position(), 3);
    }

    #[test]
    fn test_read_u8() {
        let data = vec![42, 100];
        let mut cursor = Cursor::new(data.as_slice());
        assert_eq!(read_u8(&mut cursor).unwrap(), 42);
        assert_eq!(read_u8(&mut cursor).unwrap(), 100);
    }

    #[test]
    fn test_read_compact_array() {
        // Array with 3 elements: [0x03, 10, 20, 30]
        let data = vec![0x03, 10, 20, 30];
        let mut cursor = Cursor::new(data.as_slice());
        let result = read_compact_array(&mut cursor).unwrap();
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn test_read_compact_array_empty() {
        // Empty array: [0x00]
        let data = vec![0x00];
        let mut cursor = Cursor::new(data.as_slice());
        let result = read_compact_array(&mut cursor).unwrap();
        assert_eq!(result, Vec::<u8>::new());
    }
}
