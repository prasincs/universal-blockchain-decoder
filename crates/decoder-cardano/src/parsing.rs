//! CBOR parsing functions for Cardano transactions
//!
//! This module provides pure Rust CBOR parsing without external dependencies.

use crate::types::*;
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Read a single byte from the cursor
pub fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| DecoderError::invalid_structure("Unexpected end of input reading u8"))?;
    Ok(buf[0])
}

/// Read a u16 in big-endian format
pub fn read_u16_be(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| DecoderError::invalid_structure("Unexpected end of input reading u16"))?;
    Ok(u16::from_be_bytes(buf))
}

/// Read a u32 in big-endian format
pub fn read_u32_be(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| DecoderError::invalid_structure("Unexpected end of input reading u32"))?;
    Ok(u32::from_be_bytes(buf))
}

/// Read a u64 in big-endian format
pub fn read_u64_be(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let mut buf = [0u8; 8];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| DecoderError::invalid_structure("Unexpected end of input reading u64"))?;
    Ok(u64::from_be_bytes(buf))
}

/// Read exact number of bytes
pub fn read_bytes(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf).map_err(|_| {
        DecoderError::invalid_structure(format!("Unexpected end of input reading {} bytes", len))
    })?;
    Ok(buf)
}

/// Read CBOR array header and return the number of elements
pub fn read_cbor_array_header(cursor: &mut Cursor<&[u8]>) -> Result<usize> {
    let first = read_u8(cursor)?;

    // CBOR major type 4 (array)
    let major_type = (first & 0xE0) >> 5;
    if major_type != 4 {
        return Err(DecoderError::invalid_structure(format!(
            "Expected CBOR array (major type 4), got major type {}",
            major_type
        )));
    }

    let additional = first & 0x1F;

    match additional {
        0..=23 => Ok(additional as usize),
        24 => Ok(read_u8(cursor)? as usize),
        25 => Ok(read_u16_be(cursor)? as usize),
        26 => Ok(read_u32_be(cursor)? as usize),
        27 => Ok(read_u64_be(cursor)? as usize),
        31 => Err(DecoderError::invalid_structure(
            "Indefinite-length arrays not supported",
        )),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid CBOR additional info: {}",
            additional
        ))),
    }
}

/// Read CBOR map header and return the number of key-value pairs
pub fn read_cbor_map_header(cursor: &mut Cursor<&[u8]>) -> Result<usize> {
    let first = read_u8(cursor)?;

    // CBOR major type 5 (map)
    let major_type = (first & 0xE0) >> 5;
    if major_type != 5 {
        return Err(DecoderError::invalid_structure(format!(
            "Expected CBOR map (major type 5), got major type {}",
            major_type
        )));
    }

    let additional = first & 0x1F;

    match additional {
        0..=23 => Ok(additional as usize),
        24 => Ok(read_u8(cursor)? as usize),
        25 => Ok(read_u16_be(cursor)? as usize),
        26 => Ok(read_u32_be(cursor)? as usize),
        27 => Ok(read_u64_be(cursor)? as usize),
        31 => Err(DecoderError::invalid_structure(
            "Indefinite-length maps not supported",
        )),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid CBOR additional info: {}",
            additional
        ))),
    }
}

/// Read CBOR unsigned integer
pub fn read_cbor_uint(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let first = read_u8(cursor)?;

    // CBOR major type 0 (unsigned integer)
    let major_type = (first & 0xE0) >> 5;
    if major_type != 0 {
        return Err(DecoderError::invalid_structure(format!(
            "Expected CBOR unsigned int (major type 0), got major type {}",
            major_type
        )));
    }

    let additional = first & 0x1F;

    match additional {
        0..=23 => Ok(additional as u64),
        24 => Ok(read_u8(cursor)? as u64),
        25 => Ok(read_u16_be(cursor)? as u64),
        26 => Ok(read_u32_be(cursor)? as u64),
        27 => Ok(read_u64_be(cursor)?),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid CBOR unsigned int additional info: {}",
            additional
        ))),
    }
}

/// Read CBOR byte string
pub fn read_cbor_bytes(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let first = read_u8(cursor)?;

    // CBOR major type 2 (byte string)
    let major_type = (first & 0xE0) >> 5;
    if major_type != 2 {
        return Err(DecoderError::invalid_structure(format!(
            "Expected CBOR byte string (major type 2), got major type {}",
            major_type
        )));
    }

    let additional = first & 0x1F;

    let len = match additional {
        0..=23 => additional as usize,
        24 => read_u8(cursor)? as usize,
        25 => read_u16_be(cursor)? as usize,
        26 => read_u32_be(cursor)? as usize,
        27 => read_u64_be(cursor)? as usize,
        31 => {
            return Err(DecoderError::invalid_structure(
                "Indefinite-length byte strings not supported",
            ))
        }
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid CBOR byte string additional info: {}",
                additional
            )))
        }
    };

    read_bytes(cursor, len)
}

/// Read CBOR boolean
pub fn read_cbor_bool(cursor: &mut Cursor<&[u8]>) -> Result<bool> {
    let first = read_u8(cursor)?;

    match first {
        0xF4 => Ok(false), // CBOR false
        0xF5 => Ok(true),  // CBOR true
        _ => Err(DecoderError::invalid_structure(format!(
            "Expected CBOR boolean (0xF4 or 0xF5), got 0x{:02x}",
            first
        ))),
    }
}

/// Skip a CBOR value (for fields we don't parse yet)
pub fn skip_cbor_value(cursor: &mut Cursor<&[u8]>) -> Result<()> {
    let first = read_u8(cursor)?;
    let major_type = (first & 0xE0) >> 5;
    let additional = first & 0x1F;

    match major_type {
        // Unsigned int, negative int
        0 | 1 => match additional {
            0..=23 => {}
            24 => {
                read_u8(cursor)?;
            }
            25 => {
                read_u16_be(cursor)?;
            }
            26 => {
                read_u32_be(cursor)?;
            }
            27 => {
                read_u64_be(cursor)?;
            }
            _ => {
                return Err(DecoderError::invalid_structure(
                    "Invalid CBOR integer encoding",
                ))
            }
        },
        // Byte string, text string
        2 | 3 => {
            let len = match additional {
                0..=23 => additional as usize,
                24 => read_u8(cursor)? as usize,
                25 => read_u16_be(cursor)? as usize,
                26 => read_u32_be(cursor)? as usize,
                27 => read_u64_be(cursor)? as usize,
                _ => {
                    return Err(DecoderError::invalid_structure(
                        "Invalid CBOR string encoding",
                    ))
                }
            };
            read_bytes(cursor, len)?;
        }
        // Array
        4 => {
            let len = match additional {
                0..=23 => additional as usize,
                24 => read_u8(cursor)? as usize,
                25 => read_u16_be(cursor)? as usize,
                26 => read_u32_be(cursor)? as usize,
                27 => read_u64_be(cursor)? as usize,
                _ => {
                    return Err(DecoderError::invalid_structure(
                        "Invalid CBOR array encoding",
                    ))
                }
            };
            for _ in 0..len {
                skip_cbor_value(cursor)?;
            }
        }
        // Map
        5 => {
            let len = match additional {
                0..=23 => additional as usize,
                24 => read_u8(cursor)? as usize,
                25 => read_u16_be(cursor)? as usize,
                26 => read_u32_be(cursor)? as usize,
                27 => read_u64_be(cursor)? as usize,
                _ => return Err(DecoderError::invalid_structure("Invalid CBOR map encoding")),
            };
            for _ in 0..len {
                skip_cbor_value(cursor)?; // key
                skip_cbor_value(cursor)?; // value
            }
        }
        // Simple values and floats
        7 => {
            match additional {
                0..=23 => {} // Simple value
                24 => {
                    read_u8(cursor)?;
                } // One-byte simple value
                25 => {
                    read_u16_be(cursor)?;
                } // Float16
                26 => {
                    read_u32_be(cursor)?;
                } // Float32
                27 => {
                    read_u64_be(cursor)?;
                } // Float64
                _ => return Err(DecoderError::invalid_structure("Invalid CBOR simple value")),
            }
        }
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Unsupported CBOR major type: {}",
                major_type
            )))
        }
    }

    Ok(())
}

/// Parse transaction body
pub fn parse_transaction_body(cursor: &mut Cursor<&[u8]>) -> Result<TransactionBody> {
    let map_len = read_cbor_map_header(cursor)?;

    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut fee = 0u64;
    let mut ttl = None;
    let mut certificates = vec![];
    let mut withdrawals = vec![];
    let mut script_data_hash = None;
    let mut required_signers = vec![];
    let mut network_id = None;
    let mut collateral = vec![];
    let mut mint = vec![];

    for _ in 0..map_len {
        let key = read_cbor_uint(cursor)?;

        match key {
            0 => inputs = parse_transaction_inputs(cursor)?,
            1 => outputs = parse_transaction_outputs(cursor)?,
            2 => fee = read_cbor_uint(cursor)?,
            3 => ttl = Some(read_cbor_uint(cursor)?),
            4 => certificates = parse_certificates(cursor)?,
            5 => withdrawals = parse_withdrawals(cursor)?,
            11 => script_data_hash = Some(read_cbor_bytes(cursor)?),
            14 => required_signers = parse_required_signers(cursor)?,
            15 => network_id = Some(read_cbor_uint(cursor)? as u8),
            13 => collateral = parse_transaction_inputs(cursor)?,
            9 => mint = parse_multi_assets(cursor)?,
            _ => skip_cbor_value(cursor)?, // Skip unknown fields
        }
    }

    Ok(TransactionBody {
        inputs,
        outputs,
        fee,
        ttl,
        certificates,
        withdrawals,
        script_data_hash,
        required_signers,
        network_id,
        collateral,
        mint,
    })
}

/// Parse transaction inputs
fn parse_transaction_inputs(cursor: &mut Cursor<&[u8]>) -> Result<Vec<TransactionInput>> {
    let array_len = read_cbor_array_header(cursor)?;
    let mut inputs = Vec::with_capacity(array_len);

    for _ in 0..array_len {
        let input_array_len = read_cbor_array_header(cursor)?;
        if input_array_len != 2 {
            return Err(DecoderError::invalid_structure(format!(
                "Expected input array with 2 elements, got {}",
                input_array_len
            )));
        }

        let transaction_id = read_cbor_bytes(cursor)?;
        let index = read_cbor_uint(cursor)?;

        inputs.push(TransactionInput {
            transaction_id,
            index,
        });
    }

    Ok(inputs)
}

/// Parse transaction outputs
fn parse_transaction_outputs(cursor: &mut Cursor<&[u8]>) -> Result<Vec<TransactionOutput>> {
    let array_len = read_cbor_array_header(cursor)?;
    let mut outputs = Vec::with_capacity(array_len);

    for _ in 0..array_len {
        let output_array_len = read_cbor_array_header(cursor)?;
        if output_array_len < 2 {
            return Err(DecoderError::invalid_structure(format!(
                "Expected output array with at least 2 elements, got {}",
                output_array_len
            )));
        }

        let address = read_cbor_bytes(cursor)?;
        let amount = read_cbor_uint(cursor)?;

        // For now, skip additional fields (datum hash, etc.)
        for _ in 2..output_array_len {
            skip_cbor_value(cursor)?;
        }

        outputs.push(TransactionOutput {
            address,
            amount,
            assets: vec![],
            datum_hash: None,
            inline_datum: None,
        });
    }

    Ok(outputs)
}

/// Parse certificates (simplified - just skip for now)
fn parse_certificates(_cursor: &mut Cursor<&[u8]>) -> Result<Vec<Certificate>> {
    // Simplified: just skip the certificates
    Ok(vec![])
}

/// Parse withdrawals (simplified - just skip for now)
fn parse_withdrawals(_cursor: &mut Cursor<&[u8]>) -> Result<Vec<Withdrawal>> {
    // Simplified: just skip the withdrawals
    Ok(vec![])
}

/// Parse required signers
fn parse_required_signers(cursor: &mut Cursor<&[u8]>) -> Result<Vec<Vec<u8>>> {
    let array_len = read_cbor_array_header(cursor)?;
    let mut signers = Vec::with_capacity(array_len);

    for _ in 0..array_len {
        signers.push(read_cbor_bytes(cursor)?);
    }

    Ok(signers)
}

/// Parse multi-assets (simplified)
fn parse_multi_assets(_cursor: &mut Cursor<&[u8]>) -> Result<Vec<MultiAsset>> {
    // Simplified: just skip for now
    Ok(vec![])
}

/// Parse witness set
pub fn parse_witness_set(cursor: &mut Cursor<&[u8]>) -> Result<WitnessSet> {
    let map_len = read_cbor_map_header(cursor)?;

    let mut vkey_witnesses = vec![];
    let mut native_scripts = vec![];
    let mut plutus_v1_scripts = vec![];
    let mut plutus_v2_scripts = vec![];
    let mut redeemers = vec![];
    let mut plutus_data = vec![];

    for _ in 0..map_len {
        let key = read_cbor_uint(cursor)?;

        match key {
            0 => vkey_witnesses = parse_vkey_witnesses(cursor)?,
            1 => native_scripts = parse_scripts(cursor)?,
            3 => plutus_v1_scripts = parse_scripts(cursor)?,
            6 => plutus_v2_scripts = parse_scripts(cursor)?,
            5 => redeemers = parse_redeemers(cursor)?,
            4 => plutus_data = parse_plutus_data(cursor)?,
            _ => skip_cbor_value(cursor)?,
        }
    }

    Ok(WitnessSet {
        vkey_witnesses,
        native_scripts,
        plutus_v1_scripts,
        plutus_v2_scripts,
        redeemers,
        plutus_data,
    })
}

/// Parse verification key witnesses
fn parse_vkey_witnesses(cursor: &mut Cursor<&[u8]>) -> Result<Vec<VKeyWitness>> {
    let array_len = read_cbor_array_header(cursor)?;
    let mut witnesses = Vec::with_capacity(array_len);

    for _ in 0..array_len {
        let witness_array_len = read_cbor_array_header(cursor)?;
        if witness_array_len != 2 {
            return Err(DecoderError::invalid_structure(format!(
                "Expected witness array with 2 elements, got {}",
                witness_array_len
            )));
        }

        let vkey = read_cbor_bytes(cursor)?;
        let signature = read_cbor_bytes(cursor)?;

        witnesses.push(VKeyWitness { vkey, signature });
    }

    Ok(witnesses)
}

/// Parse scripts (generic)
fn parse_scripts(cursor: &mut Cursor<&[u8]>) -> Result<Vec<Vec<u8>>> {
    let array_len = read_cbor_array_header(cursor)?;
    let mut scripts = Vec::with_capacity(array_len);

    for _ in 0..array_len {
        scripts.push(read_cbor_bytes(cursor)?);
    }

    Ok(scripts)
}

/// Parse redeemers (simplified)
fn parse_redeemers(_cursor: &mut Cursor<&[u8]>) -> Result<Vec<Redeemer>> {
    // Simplified: just skip for now
    Ok(vec![])
}

/// Parse Plutus data (simplified)
fn parse_plutus_data(_cursor: &mut Cursor<&[u8]>) -> Result<Vec<Vec<u8>>> {
    // Simplified: just skip for now
    Ok(vec![])
}

/// Parse auxiliary data (metadata)
pub fn parse_auxiliary_data(_cursor: &mut Cursor<&[u8]>) -> Result<Option<AuxiliaryData>> {
    // Simplified: just skip for now
    Ok(None)
}
