//! X-Chain transaction parsing

use crate::common::*;
use crate::xchain::types::*;
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Parse X-Chain transaction from raw bytes
pub fn parse_xchain_transaction(raw_bytes: &[u8]) -> Result<XChainTransaction> {
    let mut cursor = Cursor::new(raw_bytes);

    // Parse codec ID (2 bytes, big-endian)
    let codec_id = read_u16_be(&mut cursor)?;
    if codec_id != CODEC_ID {
        return Err(DecoderError::invalid_structure(format!(
            "Invalid codec ID: 0x{:04x}",
            codec_id
        )));
    }

    // Parse type ID (4 bytes, big-endian)
    let type_id = read_u32_be(&mut cursor)?;

    // Parse transaction based on type
    let tx_type = match type_id {
        BASE_TX => {
            let base = parse_base_tx(&mut cursor)?;
            XChainTxType::Base(base)
        }
        CREATE_ASSET_TX => parse_create_asset_tx(&mut cursor)?,
        OPERATION_TX => parse_operation_tx(&mut cursor)?,
        IMPORT_TX => parse_import_tx(&mut cursor)?,
        EXPORT_TX => parse_export_tx(&mut cursor)?,
        _ => {
            // Unknown transaction type
            let mut data = Vec::new();
            cursor.read_to_end(&mut data)?;
            XChainTxType::Unknown { type_id, data }
        }
    };

    Ok(XChainTransaction { codec_id, tx_type })
}

/// Parse base transaction fields
pub(crate) fn parse_base_tx(cursor: &mut Cursor<&[u8]>) -> Result<BaseTx> {
    // Parse network ID (4 bytes)
    let network_id = read_u32_be(cursor)?;

    // Parse blockchain ID (32 bytes)
    let blockchain_id = read_bytes_32(cursor)?;

    // Parse outputs
    let num_outputs = read_u32_be(cursor)?;
    let mut outputs = Vec::with_capacity(num_outputs.min(1000) as usize);
    for _ in 0..num_outputs {
        outputs.push(parse_transferable_output(cursor)?);
    }

    // Parse inputs
    let num_inputs = read_u32_be(cursor)?;
    let mut inputs = Vec::with_capacity(num_inputs.min(1000) as usize);
    for _ in 0..num_inputs {
        inputs.push(parse_transferable_input(cursor)?);
    }

    // Parse memo
    let memo_len = read_u32_be(cursor)?;
    let memo = read_bytes(cursor, memo_len as usize)?;

    Ok(BaseTx {
        network_id,
        blockchain_id,
        outputs,
        inputs,
        memo,
    })
}

/// Parse transferable output
pub(crate) fn parse_transferable_output(cursor: &mut Cursor<&[u8]>) -> Result<TransferableOutput> {
    let asset_id = read_bytes_32(cursor)?;
    let output = parse_output(cursor)?;

    Ok(TransferableOutput { asset_id, output })
}

/// Parse output
fn parse_output(cursor: &mut Cursor<&[u8]>) -> Result<Output> {
    let type_id = read_u32_be(cursor)?;

    match type_id {
        SECP256K1_TRANSFER_OUTPUT => {
            let amount = read_u64_be(cursor)?;
            let locktime = read_u64_be(cursor)?;
            let threshold = read_u32_be(cursor)?;

            let num_addresses = read_u32_be(cursor)?;
            let mut addresses = Vec::with_capacity(num_addresses.min(100) as usize);
            for _ in 0..num_addresses {
                addresses.push(read_bytes_20(cursor)?);
            }

            Ok(Output::Secp256k1Transfer {
                amount,
                locktime,
                threshold,
                addresses,
            })
        }
        SECP256K1_MINT_OUTPUT => {
            let locktime = read_u64_be(cursor)?;
            let threshold = read_u32_be(cursor)?;

            let num_addresses = read_u32_be(cursor)?;
            let mut addresses = Vec::with_capacity(num_addresses.min(100) as usize);
            for _ in 0..num_addresses {
                addresses.push(read_bytes_20(cursor)?);
            }

            Ok(Output::Secp256k1Mint {
                locktime,
                threshold,
                addresses,
            })
        }
        _ => {
            // Unknown output type - read remaining as data
            let mut data = Vec::new();
            cursor.read_to_end(&mut data)?;
            Ok(Output::Unknown { type_id, data })
        }
    }
}

/// Parse transferable input
pub(crate) fn parse_transferable_input(cursor: &mut Cursor<&[u8]>) -> Result<TransferableInput> {
    let tx_id = read_bytes_32(cursor)?;
    let utxo_index = read_u32_be(cursor)?;
    let asset_id = read_bytes_32(cursor)?;
    let input = parse_input(cursor)?;

    Ok(TransferableInput {
        tx_id,
        utxo_index,
        asset_id,
        input,
    })
}

/// Parse input
fn parse_input(cursor: &mut Cursor<&[u8]>) -> Result<Input> {
    let type_id = read_u32_be(cursor)?;

    match type_id {
        SECP256K1_TRANSFER_INPUT => {
            let amount = read_u64_be(cursor)?;

            let num_indices = read_u32_be(cursor)?;
            let mut address_indices = Vec::with_capacity(num_indices.min(100) as usize);
            for _ in 0..num_indices {
                address_indices.push(read_u32_be(cursor)?);
            }

            Ok(Input::Secp256k1Transfer {
                amount,
                address_indices,
            })
        }
        _ => {
            // Unknown input type - read remaining as data
            let mut data = Vec::new();
            cursor.read_to_end(&mut data)?;
            Ok(Input::Unknown { type_id, data })
        }
    }
}

/// Parse CreateAsset transaction
fn parse_create_asset_tx(cursor: &mut Cursor<&[u8]>) -> Result<XChainTxType> {
    let base = parse_base_tx(cursor)?;

    // Parse name
    let name_len = read_u16_be(cursor)? as usize;
    let name = read_string(cursor, name_len)?;

    // Parse symbol
    let symbol_len = read_u16_be(cursor)? as usize;
    let symbol = read_string(cursor, symbol_len)?;

    // Parse denomination
    let denomination = read_u8(cursor)?;

    // Parse initial states
    let num_states = read_u32_be(cursor)?;
    let mut initial_states = Vec::with_capacity(num_states.min(100) as usize);
    for _ in 0..num_states {
        initial_states.push(parse_initial_state(cursor)?);
    }

    Ok(XChainTxType::CreateAsset {
        base,
        name,
        symbol,
        denomination,
        initial_states,
    })
}

/// Parse initial state
fn parse_initial_state(cursor: &mut Cursor<&[u8]>) -> Result<InitialState> {
    let fx_id = read_u32_be(cursor)?;

    let num_outputs = read_u32_be(cursor)?;
    let mut outputs = Vec::with_capacity(num_outputs.min(100) as usize);
    for _ in 0..num_outputs {
        outputs.push(parse_output(cursor)?);
    }

    Ok(InitialState { fx_id, outputs })
}

/// Parse Operation transaction
fn parse_operation_tx(cursor: &mut Cursor<&[u8]>) -> Result<XChainTxType> {
    let base = parse_base_tx(cursor)?;

    let num_ops = read_u32_be(cursor)?;
    let mut operations = Vec::with_capacity(num_ops.min(100) as usize);
    for _ in 0..num_ops {
        operations.push(parse_operation(cursor)?);
    }

    Ok(XChainTxType::Operation { base, operations })
}

/// Parse operation
fn parse_operation(cursor: &mut Cursor<&[u8]>) -> Result<XChainOperation> {
    let asset_id = read_bytes_32(cursor)?;

    let num_utxos = read_u32_be(cursor)?;
    let mut utxo_ids = Vec::with_capacity(num_utxos.min(100) as usize);
    for _ in 0..num_utxos {
        let tx_id = read_bytes_32(cursor)?;
        let output_index = read_u32_be(cursor)?;
        utxo_ids.push(UtxoId {
            tx_id,
            output_index,
        });
    }

    // Read operation data (simplified - actual parsing depends on operation type)
    let data_len = read_u32_be(cursor)?;
    let operation_data = read_bytes(cursor, data_len as usize)?;

    Ok(XChainOperation {
        asset_id,
        utxo_ids,
        operation_data,
    })
}

/// Parse Import transaction
fn parse_import_tx(cursor: &mut Cursor<&[u8]>) -> Result<XChainTxType> {
    let base = parse_base_tx(cursor)?;
    let source_chain = read_bytes_32(cursor)?;

    let num_inputs = read_u32_be(cursor)?;
    let mut imported_inputs = Vec::with_capacity(num_inputs.min(1000) as usize);
    for _ in 0..num_inputs {
        imported_inputs.push(parse_transferable_input(cursor)?);
    }

    Ok(XChainTxType::Import {
        base,
        source_chain,
        imported_inputs,
    })
}

/// Parse Export transaction
fn parse_export_tx(cursor: &mut Cursor<&[u8]>) -> Result<XChainTxType> {
    let base = parse_base_tx(cursor)?;
    let destination_chain = read_bytes_32(cursor)?;

    let num_outputs = read_u32_be(cursor)?;
    let mut exported_outputs = Vec::with_capacity(num_outputs.min(1000) as usize);
    for _ in 0..num_outputs {
        exported_outputs.push(parse_transferable_output(cursor)?);
    }

    Ok(XChainTxType::Export {
        base,
        destination_chain,
        exported_outputs,
    })
}

// Helper functions for reading big-endian values

pub(crate) fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

pub(crate) fn read_u16_be(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_be_bytes(buf))
}

pub(crate) fn read_u32_be(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_be_bytes(buf))
}

pub(crate) fn read_u64_be(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let mut buf = [0u8; 8];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_be_bytes(buf))
}

pub(crate) fn read_bytes_20(cursor: &mut Cursor<&[u8]>) -> Result<[u8; 20]> {
    let mut buf = [0u8; 20];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read 20 bytes: {}", e)))?;
    Ok(buf)
}

pub(crate) fn read_bytes_32(cursor: &mut Cursor<&[u8]>) -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read 32 bytes: {}", e)))?;
    Ok(buf)
}

pub(crate) fn read_bytes(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<Vec<u8>> {
    if len > 1_000_000 {
        return Err(DecoderError::invalid_structure(format!(
            "Byte length too large: {}",
            len
        )));
    }

    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf).map_err(|e| {
        DecoderError::chain_decoding(format!("Failed to read {} bytes: {}", len, e))
    })?;
    Ok(buf)
}

fn read_string(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<String> {
    let bytes = read_bytes(cursor, len)?;
    String::from_utf8(bytes)
        .map_err(|e| DecoderError::chain_decoding(format!("Invalid UTF-8 string: {}", e)))
}
