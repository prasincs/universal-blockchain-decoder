//! ANS-104 DataItem binary format parser
//!
//! Parses the binary format specified in ANS-104:
//! - signature_type: 2 bytes
//! - signature: variable length (depends on signature type)
//! - owner: variable length (depends on signature type)
//! - target_present: 1 byte (0 or 1)
//! - target: 32 bytes (if present)
//! - anchor_present: 1 byte (0 or 1)
//! - anchor: 32 bytes (if present)
//! - number_of_tags: 8 bytes (u64, big-endian)
//! - number_of_tag_bytes: 8 bytes (u64, big-endian)
//! - tags: variable (Avro encoded)
//! - data: remaining bytes

use crate::types::{AOMessage, SignatureType, Tag};
use universal_decoder_core::error::{DecoderError, Result};

/// Simple byte reader for parsing ANS-104 messages
pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.data.len() {
            return Err(DecoderError::invalid_structure("Unexpected end of data"));
        }
        let val = self.data[self.pos];
        self.pos += 1;
        Ok(val)
    }

    pub fn read_u16_be(&mut self) -> Result<u16> {
        if self.pos + 2 > self.data.len() {
            return Err(DecoderError::invalid_structure("Unexpected end of data"));
        }
        let val = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(val)
    }

    pub fn read_u64_be(&mut self) -> Result<u64> {
        if self.pos + 8 > self.data.len() {
            return Err(DecoderError::invalid_structure("Unexpected end of data"));
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(u64::from_be_bytes(bytes))
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.pos + len > self.data.len() {
            return Err(DecoderError::invalid_structure(format!(
                "Cannot read {} bytes, only {} remaining",
                len,
                self.data.len() - self.pos
            )));
        }
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }
}

/// Parse an ANS-104 DataItem from bytes
pub fn parse_ans104(bytes: &[u8]) -> Result<AOMessage> {
    let mut reader = ByteReader::new(bytes);

    // 1. Parse signature type (2 bytes, big-endian)
    let signature_type_raw = reader.read_u16_be()?;
    let signature_type = SignatureType::from(signature_type_raw);

    // 2. Parse signature (variable length based on signature type)
    let signature_len = signature_length(&signature_type)?;
    let signature = reader.read_bytes(signature_len)?.to_vec();

    // 3. Parse owner (public key, variable length based on signature type)
    let owner_len = owner_length(&signature_type)?;
    let owner = reader.read_bytes(owner_len)?.to_vec();

    // 4. Parse target (optional, 32 bytes)
    let target_present = reader.read_u8()? != 0;
    let target = if target_present {
        Some(reader.read_bytes(32)?.to_vec())
    } else {
        None
    };

    // 5. Parse anchor (optional, 32 bytes)
    let anchor_present = reader.read_u8()? != 0;
    let anchor = if anchor_present {
        Some(reader.read_bytes(32)?.to_vec())
    } else {
        None
    };

    // 6. Parse number of tags (8 bytes, big-endian)
    let number_of_tags = reader.read_u64_be()?;

    // 7. Parse number of tag bytes (8 bytes, big-endian)
    let number_of_tag_bytes = reader.read_u64_be()?;

    // 8. Parse tags (Avro encoded)
    let tags = if number_of_tag_bytes > 0 {
        let tags_bytes = reader.read_bytes(number_of_tag_bytes as usize)?;
        parse_tags_avro(tags_bytes, number_of_tags)?
    } else {
        vec![]
    };

    // 9. Parse data (remaining bytes)
    let data = reader.remaining().to_vec();

    Ok(AOMessage {
        signature_type,
        signature,
        owner,
        target,
        anchor,
        tags,
        data,
        epoch: None, // Populated by Scheduler Unit after submission
        nonce: None, // Populated by Scheduler Unit after submission
    })
}

/// Get signature length based on signature type
fn signature_length(sig_type: &SignatureType) -> Result<usize> {
    match sig_type {
        SignatureType::Arweave => Ok(512), // RSA-PSS 4096-bit = 512 bytes
        SignatureType::Ethereum => Ok(65), // ECDSA signature + recovery byte
        SignatureType::Solana => Ok(64),   // Ed25519 signature
        SignatureType::Unknown(val) => Err(DecoderError::invalid_structure(format!(
            "Unknown signature type: {}",
            val
        ))),
    }
}

/// Get owner (public key) length based on signature type
fn owner_length(sig_type: &SignatureType) -> Result<usize> {
    match sig_type {
        SignatureType::Arweave => Ok(512), // RSA public key 4096-bit = 512 bytes
        SignatureType::Ethereum => Ok(65), // Uncompressed ECDSA public key
        SignatureType::Solana => Ok(32),   // Ed25519 public key
        SignatureType::Unknown(val) => Err(DecoderError::invalid_structure(format!(
            "Unknown signature type: {}",
            val
        ))),
    }
}

/// Parse Avro-encoded tags
///
/// ANS-104 uses Apache Avro with ZigZag + VInt encoding for tags.
/// Each tag is encoded as: {name: string, value: string}
fn parse_tags_avro(bytes: &[u8], expected_count: u64) -> Result<Vec<Tag>> {
    let mut tags = Vec::new();
    let mut reader = ByteReader::new(bytes);

    for _ in 0..expected_count {
        // Parse name (Avro string: VInt length + UTF-8 bytes)
        let name_len = read_varint(&mut reader)?;
        let name_bytes = reader.read_bytes(name_len)?;
        let name = String::from_utf8(name_bytes.to_vec()).map_err(|e| {
            DecoderError::invalid_structure(format!("Invalid UTF-8 in tag name: {}", e))
        })?;

        // Parse value (Avro string: VInt length + UTF-8 bytes)
        let value_len = read_varint(&mut reader)?;
        let value_bytes = reader.read_bytes(value_len)?;
        let value = String::from_utf8(value_bytes.to_vec()).map_err(|e| {
            DecoderError::invalid_structure(format!("Invalid UTF-8 in tag value: {}", e))
        })?;

        tags.push(Tag { name, value });
    }

    Ok(tags)
}

/// Read Avro VarInt (Variable-length integer)
///
/// Avro uses a variant of VarInt encoding where each byte uses 7 bits for data
/// and 1 bit as a continuation flag.
fn read_varint(reader: &mut ByteReader) -> Result<usize> {
    let mut result: usize = 0;
    let mut shift = 0;

    loop {
        let byte = reader.read_u8()?;
        result |= ((byte & 0x7F) as usize) << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;

        if shift >= 64 {
            return Err(DecoderError::invalid_structure("VarInt too large"));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_length() {
        assert_eq!(signature_length(&SignatureType::Arweave).unwrap(), 512);
        assert_eq!(signature_length(&SignatureType::Ethereum).unwrap(), 65);
        assert_eq!(signature_length(&SignatureType::Solana).unwrap(), 64);
    }

    #[test]
    fn test_owner_length() {
        assert_eq!(owner_length(&SignatureType::Arweave).unwrap(), 512);
        assert_eq!(owner_length(&SignatureType::Ethereum).unwrap(), 65);
        assert_eq!(owner_length(&SignatureType::Solana).unwrap(), 32);
    }

    #[test]
    fn test_read_varint() {
        // Single byte VarInt
        let bytes = vec![0x05]; // 5
        let mut reader = ByteReader::new(&bytes);
        assert_eq!(read_varint(&mut reader).unwrap(), 5);

        // Multi-byte VarInt
        let bytes = vec![0xAC, 0x02]; // 300 = 0b100101100
        let mut reader = ByteReader::new(&bytes);
        assert_eq!(read_varint(&mut reader).unwrap(), 300);

        // Three-byte VarInt
        let bytes = vec![0x80, 0x80, 0x01]; // 16384
        let mut reader = ByteReader::new(&bytes);
        assert_eq!(read_varint(&mut reader).unwrap(), 16384);
    }

    #[test]
    fn test_parse_tags_avro() {
        // Minimal Avro encoding: length (VarInt) + string bytes
        // Tag 1: name="Action", value="Transfer"
        // Tag 2: name="From", value="sender"

        let mut tag_bytes = Vec::new();

        // Tag 1: "Action"
        tag_bytes.push(6); // length of "Action"
        tag_bytes.extend_from_slice(b"Action");
        tag_bytes.push(8); // length of "Transfer"
        tag_bytes.extend_from_slice(b"Transfer");

        // Tag 2: "From"
        tag_bytes.push(4); // length of "From"
        tag_bytes.extend_from_slice(b"From");
        tag_bytes.push(6); // length of "sender"
        tag_bytes.extend_from_slice(b"sender");

        let tags = parse_tags_avro(&tag_bytes, 2).unwrap();

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "Action");
        assert_eq!(tags[0].value, "Transfer");
        assert_eq!(tags[1].name, "From");
        assert_eq!(tags[1].value, "sender");
    }

    #[test]
    fn test_parse_ans104_minimal() {
        // Construct a minimal ANS-104 DataItem with Solana signature
        let mut bytes = Vec::new();

        // 1. Signature type (Solana = 4)
        bytes.extend_from_slice(&4u16.to_be_bytes());

        // 2. Signature (64 bytes for Solana)
        bytes.extend_from_slice(&[0x01; 64]);

        // 3. Owner (32 bytes for Solana)
        bytes.extend_from_slice(&[0x02; 32]);

        // 4. Target present = 0 (no target)
        bytes.push(0);

        // 5. Anchor present = 0 (no anchor)
        bytes.push(0);

        // 6. Number of tags = 0
        bytes.extend_from_slice(&0u64.to_be_bytes());

        // 7. Number of tag bytes = 0
        bytes.extend_from_slice(&0u64.to_be_bytes());

        // 8. Data = "Hello, AO!"
        bytes.extend_from_slice(b"Hello, AO!");

        let msg = parse_ans104(&bytes).unwrap();

        assert_eq!(msg.signature_type, SignatureType::Solana);
        assert_eq!(msg.signature.len(), 64);
        assert_eq!(msg.owner.len(), 32);
        assert_eq!(msg.target, None);
        assert_eq!(msg.anchor, None);
        assert_eq!(msg.tags.len(), 0);
        assert_eq!(msg.data, b"Hello, AO!");
    }

    #[test]
    fn test_parse_ans104_with_tags_and_target() {
        let mut bytes = Vec::new();

        // 1. Signature type (Ethereum = 3)
        bytes.extend_from_slice(&3u16.to_be_bytes());

        // 2. Signature (65 bytes for Ethereum)
        bytes.extend_from_slice(&[0xAA; 65]);

        // 3. Owner (65 bytes for Ethereum)
        bytes.extend_from_slice(&[0xBB; 65]);

        // 4. Target present = 1
        bytes.push(1);
        bytes.extend_from_slice(&[0xCC; 32]); // 32-byte target

        // 5. Anchor present = 0
        bytes.push(0);

        // 6. Number of tags = 1
        bytes.extend_from_slice(&1u64.to_be_bytes());

        // 7. Tag bytes
        let mut tag_bytes = Vec::new();
        tag_bytes.push(6); // "Action"
        tag_bytes.extend_from_slice(b"Action");
        tag_bytes.push(4); // "Eval"
        tag_bytes.extend_from_slice(b"Eval");

        bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
        bytes.extend_from_slice(&tag_bytes);

        // 8. Data
        bytes.extend_from_slice(b"print('hello')");

        let msg = parse_ans104(&bytes).unwrap();

        assert_eq!(msg.signature_type, SignatureType::Ethereum);
        assert!(msg.target.is_some());
        assert_eq!(msg.tags.len(), 1);
        assert_eq!(msg.tags[0].name, "Action");
        assert_eq!(msg.tags[0].value, "Eval");
        assert_eq!(msg.data, b"print('hello')");
    }
}
