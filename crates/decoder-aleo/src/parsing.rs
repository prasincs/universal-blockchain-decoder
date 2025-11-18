//! Parsing logic for Aleo transactions

use crate::error::{AleoDecoderError, Result};
use crate::types::*;
use std::io::{Cursor, Read};

/// Parse an Aleo transaction from bytes
pub fn parse_transaction(cursor: &mut Cursor<&[u8]>) -> Result<AleoTransaction> {
    // Read transaction version/type marker
    let mut type_byte = [0u8; 1];
    cursor
        .read_exact(&mut type_byte)
        .map_err(|e| AleoDecoderError::ParsingError(format!("Failed to read type byte: {}", e)))?;

    // Determine transaction type
    // In Aleo, transactions are typically JSON or binary-encoded
    // For this implementation, we'll use a simplified binary format:
    // Type byte: 0x00 = Fee, 0x01 = Deploy, 0x02 = Execute
    let transaction_type = match type_byte[0] {
        0x00 => {
            let fee = parse_fee(cursor)?;
            TransactionType::Fee(fee)
        }
        0x01 => {
            let deployment = parse_deployment(cursor)?;
            TransactionType::Deploy(deployment)
        }
        0x02 => {
            let execution = parse_execution(cursor)?;
            TransactionType::Execute(execution)
        }
        _ => {
            return Err(AleoDecoderError::UnsupportedTransactionType(format!(
                "Unknown transaction type: 0x{:02x}",
                type_byte[0]
            )));
        }
    };

    // Parse optional fee (for Deploy and Execute transactions)
    let fee = if type_byte[0] != 0x00 {
        // Check if there's a fee attached
        let mut has_fee = [0u8; 1];
        if cursor.read_exact(&mut has_fee).is_ok() && has_fee[0] == 0x01 {
            Some(parse_fee(cursor)?)
        } else {
            None
        }
    } else {
        None
    };

    // Calculate transaction ID (hash of the full transaction)
    let position = cursor.position() as usize;
    let all_bytes = cursor.get_ref();
    let tx_bytes = &all_bytes[..position];
    let id = compute_transaction_id(tx_bytes);

    Ok(AleoTransaction {
        id,
        transaction_type,
        fee,
        raw_bytes: tx_bytes.to_vec(),
    })
}

/// Parse a deployment transaction
fn parse_deployment(cursor: &mut Cursor<&[u8]>) -> Result<Deployment> {
    // Edition (2 bytes)
    let edition = read_u16(cursor)?;

    // Program ID (variable length string)
    let program_id = read_string(cursor)?;

    // Program source (variable length string)
    let program = read_string(cursor)?;

    // Verifying keys count
    let vk_count = read_u16(cursor)? as usize;

    // Verifying keys
    let mut verifying_keys = Vec::with_capacity(vk_count);
    for _ in 0..vk_count {
        let function_name = read_string(cursor)?;
        let key_len = read_u32(cursor)? as usize;
        let key = read_bytes(cursor, key_len)?;

        verifying_keys.push(VerifyingKey { function_name, key });
    }

    Ok(Deployment {
        edition,
        program_id,
        program,
        verifying_keys,
    })
}

/// Parse an execution transaction
fn parse_execution(cursor: &mut Cursor<&[u8]>) -> Result<Execution> {
    // Global state root (32 bytes)
    let global_state_root = read_bytes(cursor, 32)?;

    // Transitions count
    let transition_count = read_u16(cursor)? as usize;

    // Transitions
    let mut transitions = Vec::with_capacity(transition_count);
    for _ in 0..transition_count {
        transitions.push(parse_transition(cursor)?);
    }

    // Optional proof
    let proof = if read_u8(cursor)? == 0x01 {
        let proof_len = read_u32(cursor)? as usize;
        Some(read_bytes(cursor, proof_len)?)
    } else {
        None
    };

    Ok(Execution {
        transitions,
        global_state_root,
        proof,
    })
}

/// Parse a fee transaction
fn parse_fee(cursor: &mut Cursor<&[u8]>) -> Result<Fee> {
    // Global state root (32 bytes)
    let global_state_root = read_bytes(cursor, 32)?;

    // Amount (8 bytes)
    let amount = read_u64(cursor)?;

    // Priority fee (8 bytes)
    let priority_fee = read_u64(cursor)?;

    // Optional transition
    let transition = if read_u8(cursor)? == 0x01 {
        Some(parse_transition(cursor)?)
    } else {
        None
    };

    Ok(Fee {
        global_state_root,
        amount,
        priority_fee,
        transition,
    })
}

/// Parse a transition
fn parse_transition(cursor: &mut Cursor<&[u8]>) -> Result<Transition> {
    // Transition ID (32 bytes)
    let id = read_bytes(cursor, 32)?;

    // Program ID
    let program_id = read_string(cursor)?;

    // Function name
    let function_name = read_string(cursor)?;

    // Inputs count
    let input_count = read_u8(cursor)? as usize;

    // Inputs
    let mut inputs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
        inputs.push(parse_transition_input(cursor)?);
    }

    // Outputs count
    let output_count = read_u8(cursor)? as usize;

    // Outputs
    let mut outputs = Vec::with_capacity(output_count);
    for _ in 0..output_count {
        outputs.push(parse_transition_output(cursor)?);
    }

    // Optional proof
    let proof = if read_u8(cursor)? == 0x01 {
        let proof_len = read_u32(cursor)? as usize;
        Some(read_bytes(cursor, proof_len)?)
    } else {
        None
    };

    // Finalize operations count
    let finalize_count = read_u8(cursor)? as usize;

    // Finalize operations
    let mut finalize = Vec::with_capacity(finalize_count);
    for _ in 0..finalize_count {
        finalize.push(parse_finalize_operation(cursor)?);
    }

    Ok(Transition {
        id,
        program_id,
        function_name,
        inputs,
        outputs,
        proof,
        finalize,
    })
}

/// Parse a transition input
fn parse_transition_input(cursor: &mut Cursor<&[u8]>) -> Result<TransitionInput> {
    let input_type = read_u8(cursor)?;

    match input_type {
        0x00 => {
            // Constant
            let len = read_u16(cursor)? as usize;
            let value = read_bytes(cursor, len)?;
            Ok(TransitionInput::Constant { value })
        }
        0x01 => {
            // Public
            let len = read_u16(cursor)? as usize;
            let value = read_bytes(cursor, len)?;
            Ok(TransitionInput::Public { value })
        }
        0x02 => {
            // Private
            let len = read_u16(cursor)? as usize;
            let ciphertext = read_bytes(cursor, len)?;
            Ok(TransitionInput::Private { ciphertext })
        }
        0x03 => {
            // Record
            let serial_number = read_bytes(cursor, 32)?;
            let tag = read_bytes(cursor, 32)?;
            Ok(TransitionInput::Record { serial_number, tag })
        }
        0x04 => {
            // External record
            let commitment = read_bytes(cursor, 32)?;
            Ok(TransitionInput::ExternalRecord { commitment })
        }
        _ => Err(AleoDecoderError::InvalidTransition(format!(
            "Unknown input type: 0x{:02x}",
            input_type
        ))),
    }
}

/// Parse a transition output
fn parse_transition_output(cursor: &mut Cursor<&[u8]>) -> Result<TransitionOutput> {
    let output_type = read_u8(cursor)?;

    match output_type {
        0x00 => {
            // Constant
            let len = read_u16(cursor)? as usize;
            let value = read_bytes(cursor, len)?;
            Ok(TransitionOutput::Constant { value })
        }
        0x01 => {
            // Public
            let len = read_u16(cursor)? as usize;
            let value = read_bytes(cursor, len)?;
            Ok(TransitionOutput::Public { value })
        }
        0x02 => {
            // Private
            let ciphertext_len = read_u16(cursor)? as usize;
            let ciphertext = read_bytes(cursor, ciphertext_len)?;
            let commitment = read_bytes(cursor, 32)?;
            Ok(TransitionOutput::Private {
                ciphertext,
                commitment,
            })
        }
        0x03 => {
            // Record
            let commitment = read_bytes(cursor, 32)?;
            let nonce = read_bytes(cursor, 32)?;
            let checksum = read_bytes(cursor, 16)?;
            let ciphertext_len = read_u16(cursor)? as usize;
            let ciphertext = read_bytes(cursor, ciphertext_len)?;
            Ok(TransitionOutput::Record {
                commitment,
                nonce,
                checksum,
                ciphertext,
            })
        }
        0x04 => {
            // External record
            let commitment = read_bytes(cursor, 32)?;
            Ok(TransitionOutput::ExternalRecord { commitment })
        }
        _ => Err(AleoDecoderError::InvalidTransition(format!(
            "Unknown output type: 0x{:02x}",
            output_type
        ))),
    }
}

/// Parse a finalize operation
fn parse_finalize_operation(cursor: &mut Cursor<&[u8]>) -> Result<FinalizeOperation> {
    let op_type = read_u8(cursor)?;

    match op_type {
        0x00 => {
            // Initialize mapping
            let name = read_string(cursor)?;
            Ok(FinalizeOperation::InitializeMapping { name })
        }
        0x01 => {
            // Insert mapping
            let name = read_string(cursor)?;
            let key_len = read_u16(cursor)? as usize;
            let key = read_bytes(cursor, key_len)?;
            let value_len = read_u16(cursor)? as usize;
            let value = read_bytes(cursor, value_len)?;
            Ok(FinalizeOperation::InsertMapping { name, key, value })
        }
        0x02 => {
            // Update mapping
            let name = read_string(cursor)?;
            let key_len = read_u16(cursor)? as usize;
            let key = read_bytes(cursor, key_len)?;
            let value_len = read_u16(cursor)? as usize;
            let value = read_bytes(cursor, value_len)?;
            Ok(FinalizeOperation::UpdateMapping { name, key, value })
        }
        0x03 => {
            // Remove mapping
            let name = read_string(cursor)?;
            let key_len = read_u16(cursor)? as usize;
            let key = read_bytes(cursor, key_len)?;
            Ok(FinalizeOperation::RemoveMapping { name, key })
        }
        _ => Err(AleoDecoderError::InvalidTransition(format!(
            "Unknown finalize operation type: 0x{:02x}",
            op_type
        ))),
    }
}

/// Compute transaction ID (SHA-256 hash)
fn compute_transaction_id(bytes: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).to_vec()
}

// Helper functions for reading primitive types

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| AleoDecoderError::ParsingError(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| AleoDecoderError::ParsingError(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| AleoDecoderError::ParsingError(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let mut buf = [0u8; 8];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| AleoDecoderError::ParsingError(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_le_bytes(buf))
}

fn read_bytes(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf).map_err(|e| {
        AleoDecoderError::ParsingError(format!("Failed to read {} bytes: {}", len, e))
    })?;
    Ok(buf)
}

fn read_string(cursor: &mut Cursor<&[u8]>) -> Result<String> {
    let len = read_u16(cursor)? as usize;
    let bytes = read_bytes(cursor, len)?;
    String::from_utf8(bytes)
        .map_err(|e| AleoDecoderError::ParsingError(format!("Invalid UTF-8 string: {}", e)))
}
