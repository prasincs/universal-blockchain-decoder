//! SCALE (Simple Concatenated Aggregate Little-Endian) encoding parser
//!
//! SCALE is the encoding format used by Polkadot and all Substrate-based chains.
//! It's designed for efficient encoding/decoding of blockchain data.

use decoder_primitives::prelude::*;

use crate::types::*;

/// Read a SCALE-encoded compact integer
///
/// Compact integers are variable-length:
/// - 0b00: Single-byte mode (0-63)
/// - 0b01: Two-byte mode (64-16383)
/// - 0b10: Four-byte mode (16384-1073741823)
/// - 0b11: Big-integer mode (4+ bytes, length prefix)
pub fn read_compact_u32(bytes: &[u8], offset: &mut usize) -> Result<u32> {
    if *offset >= bytes.len() {
        return Err(DecoderError::invalid_structure(
            "Not enough bytes for compact integer",
        ));
    }

    let first_byte = bytes[*offset];
    *offset += 1;

    match first_byte & 0b11 {
        // Single-byte mode: 0b00
        0b00 => Ok((first_byte >> 2) as u32),

        // Two-byte mode: 0b01
        0b01 => {
            if *offset >= bytes.len() {
                return Err(DecoderError::invalid_structure(
                    "Not enough bytes for two-byte compact",
                ));
            }
            let second_byte = bytes[*offset];
            *offset += 1;
            Ok(((first_byte as u32) >> 2) | ((second_byte as u32) << 6))
        }

        // Four-byte mode: 0b10
        0b10 => {
            if *offset + 3 > bytes.len() {
                return Err(DecoderError::invalid_structure(
                    "Not enough bytes for four-byte compact",
                ));
            }
            let mut value = (first_byte as u32) >> 2;
            for i in 0..3 {
                value |= (bytes[*offset + i] as u32) << (8 + i * 8);
            }
            *offset += 3;
            Ok(value)
        }

        // Big-integer mode: 0b11
        0b11 => {
            let length = ((first_byte >> 2) + 4) as usize;
            if *offset + length > bytes.len() {
                return Err(DecoderError::invalid_structure(
                    "Not enough bytes for big-integer compact",
                ));
            }
            let mut value = 0u32;
            for i in 0..length.min(4) {
                value |= (bytes[*offset + i] as u32) << (i * 8);
            }
            *offset += length;
            Ok(value)
        }

        _ => unreachable!(),
    }
}

/// Read a SCALE-encoded compact u64
pub fn read_compact_u64(bytes: &[u8], offset: &mut usize) -> Result<u64> {
    if *offset >= bytes.len() {
        return Err(DecoderError::invalid_structure(
            "Not enough bytes for compact integer",
        ));
    }

    let first_byte = bytes[*offset];

    match first_byte & 0b11 {
        // Single-byte and two-byte modes
        0b00 | 0b01 => read_compact_u32(bytes, offset).map(|v| v as u64),

        // Four-byte mode
        0b10 => {
            *offset += 1;
            if *offset + 3 > bytes.len() {
                return Err(DecoderError::invalid_structure(
                    "Not enough bytes for four-byte compact",
                ));
            }
            let mut value = (first_byte as u64) >> 2;
            for i in 0..3 {
                value |= (bytes[*offset + i] as u64) << (8 + i * 8);
            }
            *offset += 3;
            Ok(value)
        }

        // Big-integer mode
        0b11 => {
            *offset += 1;
            let length = ((first_byte >> 2) + 4) as usize;
            if *offset + length > bytes.len() {
                return Err(DecoderError::invalid_structure(
                    "Not enough bytes for big-integer compact",
                ));
            }
            let mut value = 0u64;
            for i in 0..length.min(8) {
                value |= (bytes[*offset + i] as u64) << (i * 8);
            }
            *offset += length;
            Ok(value)
        }

        _ => unreachable!(),
    }
}

/// Read a fixed-size byte array
pub fn read_bytes(bytes: &[u8], offset: &mut usize, length: usize) -> Result<Vec<u8>> {
    if *offset + length > bytes.len() {
        return Err(DecoderError::invalid_structure(format!(
            "Not enough bytes: need {}, have {}",
            length,
            bytes.len() - *offset
        )));
    }
    let data = bytes[*offset..*offset + length].to_vec();
    *offset += length;
    Ok(data)
}

/// Read a SCALE-encoded vector (compact length prefix + data)
pub fn read_vec(bytes: &[u8], offset: &mut usize) -> Result<Vec<u8>> {
    let length = read_compact_u32(bytes, offset)? as usize;
    read_bytes(bytes, offset, length)
}

/// Read a u8
pub fn read_u8(bytes: &[u8], offset: &mut usize) -> Result<u8> {
    if *offset >= bytes.len() {
        return Err(DecoderError::invalid_structure("Not enough bytes for u8"));
    }
    let value = bytes[*offset];
    *offset += 1;
    Ok(value)
}

/// Read a u16 (little-endian)
pub fn read_u16_le(bytes: &[u8], offset: &mut usize) -> Result<u16> {
    if *offset + 2 > bytes.len() {
        return Err(DecoderError::invalid_structure("Not enough bytes for u16"));
    }
    let value = u16::from_le_bytes([bytes[*offset], bytes[*offset + 1]]);
    *offset += 2;
    Ok(value)
}

/// Read a u32 (little-endian)
pub fn read_u32_le(bytes: &[u8], offset: &mut usize) -> Result<u32> {
    if *offset + 4 > bytes.len() {
        return Err(DecoderError::invalid_structure("Not enough bytes for u32"));
    }
    let value = u32::from_le_bytes([
        bytes[*offset],
        bytes[*offset + 1],
        bytes[*offset + 2],
        bytes[*offset + 3],
    ]);
    *offset += 4;
    Ok(value)
}

/// Read a u64 (little-endian)
pub fn read_u64_le(bytes: &[u8], offset: &mut usize) -> Result<u64> {
    if *offset + 8 > bytes.len() {
        return Err(DecoderError::invalid_structure("Not enough bytes for u64"));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[*offset..*offset + 8]);
    *offset += 8;
    Ok(u64::from_le_bytes(buf))
}

/// Read a u128 (little-endian)
pub fn read_u128_le(bytes: &[u8], offset: &mut usize) -> Result<u128> {
    if *offset + 16 > bytes.len() {
        return Err(DecoderError::invalid_structure("Not enough bytes for u128"));
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&bytes[*offset..*offset + 16]);
    *offset += 16;
    Ok(u128::from_le_bytes(buf))
}

/// Read an `Option<T>` encoded as SCALE
/// 0x00 = None, 0x01 + value = Some(value)
pub fn read_option_bool(bytes: &[u8], offset: &mut usize) -> Result<Option<bool>> {
    let tag = read_u8(bytes, offset)?;
    match tag {
        0x00 => Ok(None),
        0x01 => Ok(Some(true)),
        0x02 => Ok(Some(false)),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid option tag: {:#x}",
            tag
        ))),
    }
}

/// Read a SCALE-encoded compact u128
pub fn read_compact_u128(bytes: &[u8], offset: &mut usize) -> Result<u128> {
    if *offset >= bytes.len() {
        return Err(DecoderError::invalid_structure(
            "Not enough bytes for compact integer",
        ));
    }

    let first_byte = bytes[*offset];

    match first_byte & 0b11 {
        // Single-byte, two-byte, or four-byte modes
        0b00..=0b10 => read_compact_u64(bytes, offset).map(|v| v as u128),

        // Big-integer mode
        0b11 => {
            *offset += 1;
            let length = ((first_byte >> 2) + 4) as usize;
            if *offset + length > bytes.len() {
                return Err(DecoderError::invalid_structure(
                    "Not enough bytes for big-integer compact",
                ));
            }
            let mut value = 0u128;
            for i in 0..length.min(16) {
                value |= (bytes[*offset + i] as u128) << (i * 8);
            }
            *offset += length;
            Ok(value)
        }

        _ => unreachable!(),
    }
}

/// Parse a call (pallet + function + parameters)
pub fn parse_call(call_data: &[u8]) -> Result<Call> {
    if call_data.len() < 2 {
        return Err(DecoderError::invalid_structure(
            "Call data too short (need at least 2 bytes)",
        ));
    }

    let pallet_index = call_data[0];
    let call_index = call_data[1];
    let parameters = call_data[2..].to_vec();

    Ok(Call {
        pallet_index,
        call_index,
        parameters,
    })
}

/// Parse a SCALE-encoded Polkadot extrinsic
pub fn parse_extrinsic(bytes: &[u8]) -> Result<Extrinsic> {
    let mut offset = 0;

    // First read the length prefix (compact-encoded)
    let _length = read_compact_u32(bytes, &mut offset)?;

    // Read version byte
    let version_byte = read_u8(bytes, &mut offset)?;
    let version = ExtrinsicVersion::from_byte(version_byte);

    if version.is_signed {
        parse_signed_extrinsic(bytes, &mut offset)
    } else {
        parse_unsigned_extrinsic(bytes, &mut offset)
    }
}

/// Parse a signed extrinsic
fn parse_signed_extrinsic(bytes: &[u8], offset: &mut usize) -> Result<Extrinsic> {
    // Parse address
    let from = parse_address(bytes, offset)?;

    // Parse signature
    let signature = parse_signature(bytes, offset)?;

    // Parse era
    let era = parse_era(bytes, offset)?;

    // Parse nonce (compact-encoded)
    let nonce = read_compact_u64(bytes, offset)?;

    // Parse tip (compact-encoded u128)
    let tip = read_compact_u128(bytes, offset)?;

    // Remaining bytes are the call data
    let call = bytes[*offset..].to_vec();
    *offset = bytes.len();

    Ok(Extrinsic::Signed(SignedExtrinsic {
        from,
        signature,
        extension: SignedExtension { era, nonce, tip },
        call,
    }))
}

/// Parse an unsigned extrinsic
fn parse_unsigned_extrinsic(bytes: &[u8], offset: &mut usize) -> Result<Extrinsic> {
    // Remaining bytes are the call data
    let call = bytes[*offset..].to_vec();
    *offset = bytes.len();

    Ok(Extrinsic::Unsigned(UnsignedExtrinsic { call }))
}

/// Parse a SCALE-encoded address
fn parse_address(bytes: &[u8], offset: &mut usize) -> Result<PolkadotAddress> {
    let address_type = read_u8(bytes, offset)?;

    match address_type {
        0x00 => {
            // Id: 32-byte account ID
            let id = read_bytes(bytes, offset, 32)?;
            Ok(PolkadotAddress::Id(id))
        }
        0x01 => {
            // Index: compact-encoded u32
            let index = read_compact_u32(bytes, offset)?;
            Ok(PolkadotAddress::Index(index))
        }
        0x02 => {
            // Raw: length-prefixed bytes
            let raw = read_vec(bytes, offset)?;
            Ok(PolkadotAddress::Raw(raw))
        }
        0x03 => {
            // Address32: 32-byte address
            let addr = read_bytes(bytes, offset, 32)?;
            Ok(PolkadotAddress::Address32(addr))
        }
        0x04 => {
            // Address20: 20-byte address
            let addr = read_bytes(bytes, offset, 20)?;
            Ok(PolkadotAddress::Address20(addr))
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Unknown address type: {:#x}",
            address_type
        ))),
    }
}

/// Parse a SCALE-encoded signature
fn parse_signature(bytes: &[u8], offset: &mut usize) -> Result<PolkadotSignature> {
    let sig_type = read_u8(bytes, offset)?;

    match sig_type {
        0x00 => {
            // Ed25519: 64 bytes
            let sig = read_bytes(bytes, offset, 64)?;
            Ok(PolkadotSignature::Ed25519(sig))
        }
        0x01 => {
            // Sr25519: 64 bytes
            let sig = read_bytes(bytes, offset, 64)?;
            Ok(PolkadotSignature::Sr25519(sig))
        }
        0x02 => {
            // ECDSA: 65 bytes
            let sig = read_bytes(bytes, offset, 65)?;
            Ok(PolkadotSignature::Ecdsa(sig))
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Unknown signature type: {:#x}",
            sig_type
        ))),
    }
}

/// Parse era (transaction mortality)
fn parse_era(bytes: &[u8], offset: &mut usize) -> Result<Era> {
    let first_byte = read_u8(bytes, offset)?;

    if first_byte == 0x00 {
        // Immortal
        Ok(Era::Immortal)
    } else {
        // Mortal: encoded as (period, phase)
        let second_byte = read_u8(bytes, offset)?;
        let encoded = u16::from_le_bytes([first_byte, second_byte]);
        let period = 2u64 << (encoded % (1 << 4));
        let quantize_factor = period.max(4) >> 12;
        let phase = (encoded >> 4) as u64 * quantize_factor.max(1);
        Ok(Era::Mortal(period, phase))
    }
}
