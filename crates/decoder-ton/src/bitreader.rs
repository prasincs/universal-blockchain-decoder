//! Bit-level reader for TL-B parsing
//!
//! TON uses Type Language Binary (TL-B) which requires bit-level precision
//! for reading non-byte-aligned fields.

use decoder_primitives::prelude::*;

/// Bit-level reader for cell data
pub struct BitReader<'a> {
    data: &'a [u8],
    bit_offset: usize,
    bit_len: usize,
}

impl<'a> BitReader<'a> {
    /// Create a new bit reader from cell data
    pub fn new(data: &'a [u8], bit_len: usize) -> Self {
        Self {
            data,
            bit_offset: 0,
            bit_len,
        }
    }

    /// Get current bit position
    #[allow(dead_code)]
    pub fn position(&self) -> usize {
        self.bit_offset
    }

    /// Get remaining bits
    #[allow(dead_code)]
    pub fn remaining(&self) -> usize {
        self.bit_len.saturating_sub(self.bit_offset)
    }

    /// Read a single bit as bool
    pub fn read_bit(&mut self) -> Result<bool> {
        if self.bit_offset >= self.bit_len {
            return Err(DecoderError::invalid_structure(
                "Bit reader: attempt to read beyond cell data",
            ));
        }

        let byte_index = self.bit_offset / 8;
        let bit_index = 7 - (self.bit_offset % 8); // MSB first

        if byte_index >= self.data.len() {
            return Err(DecoderError::invalid_structure(
                "Bit reader: byte index out of bounds",
            ));
        }

        let bit = (self.data[byte_index] >> bit_index) & 1;
        self.bit_offset += 1;

        Ok(bit == 1)
    }

    /// Read multiple bits as u8 (max 8 bits)
    pub fn read_bits_u8(&mut self, count: usize) -> Result<u8> {
        if count > 8 {
            return Err(DecoderError::invalid_structure(
                "Cannot read more than 8 bits into u8",
            ));
        }

        let mut result = 0u8;
        for _ in 0..count {
            result = (result << 1) | (self.read_bit()? as u8);
        }

        Ok(result)
    }

    /// Read multiple bits as u16 (max 16 bits)
    pub fn read_bits_u16(&mut self, count: usize) -> Result<u16> {
        if count > 16 {
            return Err(DecoderError::invalid_structure(
                "Cannot read more than 16 bits into u16",
            ));
        }

        let mut result = 0u16;
        for _ in 0..count {
            result = (result << 1) | (self.read_bit()? as u16);
        }

        Ok(result)
    }

    /// Read multiple bits as u32 (max 32 bits)
    pub fn read_bits_u32(&mut self, count: usize) -> Result<u32> {
        if count > 32 {
            return Err(DecoderError::invalid_structure(
                "Cannot read more than 32 bits into u32",
            ));
        }

        let mut result = 0u32;
        for _ in 0..count {
            result = (result << 1) | (self.read_bit()? as u32);
        }

        Ok(result)
    }

    /// Read multiple bits as u64 (max 64 bits)
    pub fn read_bits_u64(&mut self, count: usize) -> Result<u64> {
        if count > 64 {
            return Err(DecoderError::invalid_structure(
                "Cannot read more than 64 bits into u64",
            ));
        }

        let mut result = 0u64;
        for _ in 0..count {
            result = (result << 1) | (self.read_bit()? as u64);
        }

        Ok(result)
    }

    /// Read exact number of bits into a byte vector
    pub fn read_bits(&mut self, count: usize) -> Result<Vec<u8>> {
        let byte_count = count.div_ceil(8);
        let mut result = vec![0u8; byte_count];

        for i in 0..count {
            let bit = self.read_bit()?;
            if bit {
                let byte_index = i / 8;
                let bit_index = 7 - (i % 8);
                result[byte_index] |= 1 << bit_index;
            }
        }

        Ok(result)
    }

    /// Read bytes (must be byte-aligned and count in bytes)
    #[allow(dead_code)]
    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        if !self.bit_offset.is_multiple_of(8) {
            return Err(DecoderError::invalid_structure(
                "Bit reader: read_bytes requires byte alignment",
            ));
        }

        let byte_offset = self.bit_offset / 8;
        if byte_offset + count > self.data.len() {
            return Err(DecoderError::invalid_structure(
                "Bit reader: not enough bytes to read",
            ));
        }

        let result = self.data[byte_offset..byte_offset + count].to_vec();
        self.bit_offset += count * 8;

        Ok(result)
    }

    /// Skip bits
    #[allow(dead_code)]
    pub fn skip_bits(&mut self, count: usize) -> Result<()> {
        if self.bit_offset + count > self.bit_len {
            return Err(DecoderError::invalid_structure(
                "Bit reader: cannot skip beyond cell data",
            ));
        }

        self.bit_offset += count;
        Ok(())
    }

    /// Read Maybe type (1 bit indicating presence, then value)
    #[allow(dead_code)]
    pub fn read_maybe<T, F>(&mut self, read_fn: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let has_value = self.read_bit()?;
        if has_value {
            Ok(Some(read_fn(self)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_single_bit() {
        let data = vec![0b10110000]; // First 4 bits: 1011
        let mut reader = BitReader::new(&data, 8);

        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
    }

    #[test]
    fn test_read_bits_u8() {
        let data = vec![0b10110100]; // 1011 0100
        let mut reader = BitReader::new(&data, 8);

        assert_eq!(reader.read_bits_u8(4).unwrap(), 0b1011);
        assert_eq!(reader.read_bits_u8(4).unwrap(), 0b0100);
    }

    #[test]
    fn test_read_bits_u16() {
        let data = vec![0b10110100, 0b11001010]; // 1011010011001010
        let mut reader = BitReader::new(&data, 16);

        assert_eq!(reader.read_bits_u16(12).unwrap(), 0b101101001100);
        assert_eq!(reader.read_bits_u16(4).unwrap(), 0b1010);
    }

    #[test]
    fn test_read_bits_u32() {
        let data = vec![0xff, 0x00, 0xaa, 0x55];
        let mut reader = BitReader::new(&data, 32);

        assert_eq!(reader.read_bits_u32(32).unwrap(), 0xff00aa55);
    }

    #[test]
    fn test_read_bytes() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let mut reader = BitReader::new(&data, 32);

        let bytes = reader.read_bytes(4).unwrap();
        assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_read_maybe() {
        // Test with value: 1 followed by 4 bits (0110) = 6
        let data = vec![0b10110100];
        let mut reader = BitReader::new(&data, 8);

        let result = reader.read_maybe(|r| r.read_bits_u8(4)).unwrap();
        assert_eq!(result, Some(0b0110)); // = 6

        // Test without value: 0
        let data = vec![0b00000000];
        let mut reader = BitReader::new(&data, 8);

        let result: Option<u8> = reader.read_maybe(|r| r.read_bits_u8(4)).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_remaining() {
        let data = vec![0xff, 0xff];
        let mut reader = BitReader::new(&data, 16);

        assert_eq!(reader.remaining(), 16);
        reader.read_bits_u8(4).unwrap();
        assert_eq!(reader.remaining(), 12);
        reader.read_bits_u8(8).unwrap();
        assert_eq!(reader.remaining(), 4);
    }
}
