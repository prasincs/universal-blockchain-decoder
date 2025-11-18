//! RLP Encoder
//!
//! Implements RLP (Recursive Length Prefix) encoding for Ethereum transactions.
//! This is the inverse of the RLP decoder and enables true round-trip encoding.

use universal_decoder_core::prelude::DecoderError;

type Result<T> = std::result::Result<T, DecoderError>;

/// RLP Encoder for Ethereum data structures
#[derive(Default)]
pub struct RlpEncoder {
    buffer: Vec<u8>,
}

impl RlpEncoder {
    /// Create a new RLP encoder
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Encode a single byte value
    ///
    /// RLP encoding for single bytes:
    /// - If byte is 0x00-0x7f: encode as itself
    /// - If byte is 0x80-0xff: encode as 0x81 followed by the byte
    pub fn encode_byte(&mut self, byte: u8) -> Result<&mut Self> {
        if byte < 0x80 {
            self.buffer.push(byte);
        } else {
            self.buffer.push(0x81);
            self.buffer.push(byte);
        }
        Ok(self)
    }

    /// Encode bytes (string)
    ///
    /// RLP encoding for byte arrays:
    /// - 0-55 bytes: 0x80 + length, then data
    /// - 56+ bytes: 0xb7 + length_of_length, then length, then data
    pub fn encode_bytes(&mut self, data: &[u8]) -> Result<&mut Self> {
        if data.is_empty() {
            self.buffer.push(0x80);
        } else if data.len() == 1 && data[0] < 0x80 {
            self.buffer.push(data[0]);
        } else if data.len() <= 55 {
            self.buffer.push(0x80 + data.len() as u8);
            self.buffer.extend_from_slice(data);
        } else {
            let len_bytes = encode_length(data.len());
            self.buffer.push(0xb7 + len_bytes.len() as u8);
            self.buffer.extend_from_slice(&len_bytes);
            self.buffer.extend_from_slice(data);
        }
        Ok(self)
    }

    /// Encode u64 as RLP bytes
    pub fn encode_u64(&mut self, value: u64) -> Result<&mut Self> {
        if value == 0 {
            self.encode_bytes(&[])?;
        } else {
            let bytes = value.to_be_bytes();
            let start = bytes.iter().position(|&b| b != 0).unwrap_or(0);
            self.encode_bytes(&bytes[start..])?;
        }
        Ok(self)
    }

    /// Encode u128 as RLP bytes
    pub fn encode_u128(&mut self, value: u128) -> Result<&mut Self> {
        if value == 0 {
            self.encode_bytes(&[])?;
        } else {
            let bytes = value.to_be_bytes();
            let start = bytes.iter().position(|&b| b != 0).unwrap_or(0);
            self.encode_bytes(&bytes[start..])?;
        }
        Ok(self)
    }

    /// Encode optional u128
    pub fn encode_optional_u128(&mut self, value: Option<u128>) -> Result<&mut Self> {
        match value {
            Some(v) => self.encode_u128(v),
            None => self.encode_bytes(&[]),
        }
    }

    /// Encode 20-byte address (or empty)
    pub fn encode_address(&mut self, addr: Option<[u8; 20]>) -> Result<&mut Self> {
        match addr {
            Some(a) => self.encode_bytes(&a),
            None => self.encode_bytes(&[]),
        }
    }

    /// Begin a list - returns a new encoder for the list contents
    pub fn begin_list(&mut self) -> ListEncoder<'_> {
        ListEncoder {
            parent: self,
            items: Vec::new(),
        }
    }

    /// Finalize and return the encoded bytes
    pub fn finalize(self) -> Vec<u8> {
        self.buffer
    }

    /// Encode a list prefix for given payload length
    fn encode_list_header(&mut self, payload_len: usize) -> Result<&mut Self> {
        if payload_len <= 55 {
            self.buffer.push(0xc0 + payload_len as u8);
        } else {
            let len_bytes = encode_length(payload_len);
            self.buffer.push(0xf7 + len_bytes.len() as u8);
            self.buffer.extend_from_slice(&len_bytes);
        }
        Ok(self)
    }
}

/// List encoder - accumulates list items then encodes with proper header
pub struct ListEncoder<'a> {
    parent: &'a mut RlpEncoder,
    items: Vec<Vec<u8>>,
}

impl<'a> ListEncoder<'a> {
    /// Add an RLP-encoded item to the list
    pub fn append_bytes(&mut self, data: &[u8]) -> Result<&mut Self> {
        let mut encoder = RlpEncoder::new();
        encoder.encode_bytes(data)?;
        self.items.push(encoder.finalize());
        Ok(self)
    }

    /// Add a u64 to the list
    pub fn append_u64(&mut self, value: u64) -> Result<&mut Self> {
        let mut encoder = RlpEncoder::new();
        encoder.encode_u64(value)?;
        self.items.push(encoder.finalize());
        Ok(self)
    }

    /// Add a u128 to the list
    pub fn append_u128(&mut self, value: u128) -> Result<&mut Self> {
        let mut encoder = RlpEncoder::new();
        encoder.encode_u128(value)?;
        self.items.push(encoder.finalize());
        Ok(self)
    }

    /// Add an optional u128 to the list
    pub fn append_optional_u128(&mut self, value: Option<u128>) -> Result<&mut Self> {
        let mut encoder = RlpEncoder::new();
        encoder.encode_optional_u128(value)?;
        self.items.push(encoder.finalize());
        Ok(self)
    }

    /// Add an address to the list
    pub fn append_address(&mut self, addr: Option<[u8; 20]>) -> Result<&mut Self> {
        let mut encoder = RlpEncoder::new();
        encoder.encode_address(addr)?;
        self.items.push(encoder.finalize());
        Ok(self)
    }

    /// Add a nested list
    pub fn append_list<F>(&mut self, f: F) -> Result<&mut Self>
    where
        F: FnOnce(&mut ListEncoder) -> Result<()>,
    {
        let mut nested_encoder = RlpEncoder::new();
        let mut nested_list = nested_encoder.begin_list();
        f(&mut nested_list)?;
        nested_list.finalize()?;
        self.items.push(nested_encoder.finalize());
        Ok(self)
    }

    /// Finalize the list and write to parent encoder
    pub fn finalize(self) -> Result<()> {
        // Calculate total payload length
        let payload_len: usize = self.items.iter().map(|item| item.len()).sum();

        // Encode list header
        self.parent.encode_list_header(payload_len)?;

        // Append all items
        for item in &self.items {
            self.parent.buffer.extend_from_slice(item);
        }

        Ok(())
    }
}

/// Encode a length as big-endian bytes (minimal representation)
fn encode_length(len: usize) -> Vec<u8> {
    let bytes = len.to_be_bytes();
    let start = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len() - 1);
    bytes[start..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_single_byte() {
        let mut encoder = RlpEncoder::new();
        encoder.encode_byte(0x00).unwrap();
        assert_eq!(encoder.finalize(), vec![0x00]);

        let mut encoder = RlpEncoder::new();
        encoder.encode_byte(0x7f).unwrap();
        assert_eq!(encoder.finalize(), vec![0x7f]);

        let mut encoder = RlpEncoder::new();
        encoder.encode_byte(0x80).unwrap();
        assert_eq!(encoder.finalize(), vec![0x81, 0x80]);
    }

    #[test]
    fn test_encode_empty_bytes() {
        let mut encoder = RlpEncoder::new();
        encoder.encode_bytes(&[]).unwrap();
        assert_eq!(encoder.finalize(), vec![0x80]);
    }

    #[test]
    fn test_encode_short_string() {
        let mut encoder = RlpEncoder::new();
        encoder.encode_bytes(b"dog").unwrap();
        assert_eq!(encoder.finalize(), vec![0x83, b'd', b'o', b'g']);
    }

    #[test]
    fn test_encode_u64_zero() {
        let mut encoder = RlpEncoder::new();
        encoder.encode_u64(0).unwrap();
        assert_eq!(encoder.finalize(), vec![0x80]); // Empty bytes
    }

    #[test]
    fn test_encode_u64() {
        let mut encoder = RlpEncoder::new();
        encoder.encode_u64(1024).unwrap();
        assert_eq!(encoder.finalize(), vec![0x82, 0x04, 0x00]);
    }

    #[test]
    fn test_encode_list() {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();
        list.append_u64(1).unwrap();
        list.append_u64(2).unwrap();
        list.append_u64(3).unwrap();
        list.finalize().unwrap();

        // List of [1, 2, 3]
        // 0xc3 (list header for 3 bytes), 0x01, 0x02, 0x03
        assert_eq!(encoder.finalize(), vec![0xc3, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_encode_nested_list() {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        list.append_list(|inner| {
            inner.append_u64(1)?;
            inner.append_u64(2)?;
            Ok(())
        })
        .unwrap();

        list.append_u64(3).unwrap();
        list.finalize().unwrap();

        // List of [[1, 2], 3]
        assert_eq!(encoder.finalize(), vec![0xc4, 0xc2, 0x01, 0x02, 0x03]);
    }
}
