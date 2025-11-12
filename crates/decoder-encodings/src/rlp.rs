//! Pure Rust RLP (Recursive Length Prefix) decoder
//!
//! Implements the RLP specification from the Ethereum Yellow Paper.
//! This is a minimal, security-focused implementation without external dependencies.

use universal_decoder_core::prelude::*;

/// RLP item representation
#[derive(Debug, Clone, PartialEq)]
pub enum RlpItem {
    /// Raw byte data
    Data(Vec<u8>),
    /// List of RLP items
    List(Vec<RlpItem>),
}

impl RlpItem {
    /// Decode RLP-encoded bytes into an RlpItem
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let (item, consumed) = Self::decode_with_consumed(bytes)?;

        if consumed != bytes.len() {
            return Err(DecoderError::invalid_structure(
                "RLP decoding did not consume all bytes",
            ));
        }

        Ok(item)
    }

    /// Decode RLP and return the item plus number of bytes consumed
    fn decode_with_consumed(bytes: &[u8]) -> Result<(Self, usize)> {
        if bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Cannot decode empty RLP"));
        }

        let prefix = bytes[0];

        match prefix {
            // Single byte in [0x00, 0x7f]
            0x00..=0x7f => Ok((RlpItem::Data(vec![prefix]), 1)),

            // String 0-55 bytes: [0x80, 0xb7]
            0x80..=0xb7 => {
                let length = (prefix - 0x80) as usize;

                if length == 0 {
                    return Ok((RlpItem::Data(vec![]), 1));
                }

                if bytes.len() < 1 + length {
                    return Err(DecoderError::invalid_structure(
                        "RLP string length exceeds available data",
                    ));
                }

                let data = bytes[1..1 + length].to_vec();
                Ok((RlpItem::Data(data), 1 + length))
            }

            // String > 55 bytes: [0xb8, 0xbf]
            0xb8..=0xbf => {
                let length_of_length = (prefix - 0xb7) as usize;

                if bytes.len() < 1 + length_of_length {
                    return Err(DecoderError::invalid_structure(
                        "RLP long string length encoding incomplete",
                    ));
                }

                let length = decode_length(&bytes[1..1 + length_of_length])?;
                let data_start = 1 + length_of_length;
                let data_end = data_start
                    .checked_add(length)
                    .ok_or_else(|| DecoderError::invalid_structure("RLP length overflow"))?;

                if bytes.len() < data_end {
                    return Err(DecoderError::invalid_structure(
                        "RLP long string data incomplete",
                    ));
                }

                let data = bytes[data_start..data_end].to_vec();
                Ok((RlpItem::Data(data), data_end))
            }

            // List 0-55 bytes payload: [0xc0, 0xf7]
            0xc0..=0xf7 => {
                let length = (prefix - 0xc0) as usize;

                if length == 0 {
                    return Ok((RlpItem::List(vec![]), 1));
                }

                if bytes.len() < 1 + length {
                    return Err(DecoderError::invalid_structure(
                        "RLP list length exceeds available data",
                    ));
                }

                let list = decode_list(&bytes[1..1 + length])?;
                Ok((RlpItem::List(list), 1 + length))
            }

            // List > 55 bytes payload: [0xf8, 0xff]
            0xf8..=0xff => {
                let length_of_length = (prefix - 0xf7) as usize;

                if bytes.len() < 1 + length_of_length {
                    return Err(DecoderError::invalid_structure(
                        "RLP long list length encoding incomplete",
                    ));
                }

                let length = decode_length(&bytes[1..1 + length_of_length])?;
                let data_start = 1 + length_of_length;
                let data_end = data_start
                    .checked_add(length)
                    .ok_or_else(|| DecoderError::invalid_structure("RLP length overflow"))?;

                if bytes.len() < data_end {
                    return Err(DecoderError::invalid_structure(
                        "RLP long list data incomplete",
                    ));
                }

                let list = decode_list(&bytes[data_start..data_end])?;
                Ok((RlpItem::List(list), data_end))
            }
        }
    }

    /// Extract as data bytes, error if this is a list
    pub fn as_data(&self) -> Result<&[u8]> {
        match self {
            RlpItem::Data(data) => Ok(data),
            RlpItem::List(_) => Err(DecoderError::invalid_structure(
                "Expected RLP data, found list",
            )),
        }
    }

    /// Extract as list, error if this is data
    pub fn as_list(&self) -> Result<&[RlpItem]> {
        match self {
            RlpItem::List(list) => Ok(list),
            RlpItem::Data(_) => Err(DecoderError::invalid_structure(
                "Expected RLP list, found data",
            )),
        }
    }

    /// Extract as u64 from data bytes (big-endian)
    pub fn as_u64(&self) -> Result<u64> {
        let data = self.as_data()?;

        if data.is_empty() {
            return Ok(0);
        }

        if data.len() > 8 {
            return Err(DecoderError::invalid_structure(
                "RLP data too large for u64",
            ));
        }

        // Check for leading zeros (non-canonical encoding)
        if data.len() > 1 && data[0] == 0 {
            return Err(DecoderError::invalid_structure(
                "RLP integer has leading zeros",
            ));
        }

        let mut result = 0u64;
        for &byte in data {
            result = result
                .checked_shl(8)
                .ok_or_else(|| DecoderError::invalid_structure("Integer overflow in RLP"))?;
            result |= byte as u64;
        }

        Ok(result)
    }

    /// Extract as u128 from data bytes (big-endian)
    pub fn as_u128(&self) -> Result<u128> {
        let data = self.as_data()?;

        if data.is_empty() {
            return Ok(0);
        }

        if data.len() > 16 {
            return Err(DecoderError::invalid_structure(
                "RLP data too large for u128",
            ));
        }

        // Check for leading zeros (non-canonical encoding)
        if data.len() > 1 && data[0] == 0 {
            return Err(DecoderError::invalid_structure(
                "RLP integer has leading zeros",
            ));
        }

        let mut result = 0u128;
        for &byte in data {
            result = result
                .checked_shl(8)
                .ok_or_else(|| DecoderError::invalid_structure("Integer overflow in RLP"))?;
            result |= byte as u128;
        }

        Ok(result)
    }
}

/// Decode length from big-endian bytes
fn decode_length(bytes: &[u8]) -> Result<usize> {
    if bytes.is_empty() {
        return Err(DecoderError::invalid_structure("Empty length encoding"));
    }

    // Check for leading zeros (non-canonical)
    if bytes.len() > 1 && bytes[0] == 0 {
        return Err(DecoderError::invalid_structure(
            "Length encoding has leading zeros",
        ));
    }

    let mut length = 0usize;
    for &byte in bytes {
        length = length
            .checked_shl(8)
            .ok_or_else(|| DecoderError::invalid_structure("Length overflow"))?;
        length = length
            .checked_add(byte as usize)
            .ok_or_else(|| DecoderError::invalid_structure("Length overflow"))?;
    }

    // Check that the length encoding was necessary (canonical form)
    if length < 56 {
        return Err(DecoderError::invalid_structure(
            "Length should have used short form encoding",
        ));
    }

    Ok(length)
}

/// Decode a list payload into individual RLP items
fn decode_list(bytes: &[u8]) -> Result<Vec<RlpItem>> {
    let mut items = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        let (item, consumed) = RlpItem::decode_with_consumed(&bytes[offset..])?;
        items.push(item);
        offset += consumed;
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_single_byte() {
        // Single byte values [0x00, 0x7f] are their own RLP encoding
        let result = RlpItem::decode(&[0x42]).unwrap();
        assert_eq!(result, RlpItem::Data(vec![0x42]));
    }

    #[test]
    fn test_decode_empty_string() {
        // Empty string is 0x80
        let result = RlpItem::decode(&[0x80]).unwrap();
        assert_eq!(result, RlpItem::Data(vec![]));
    }

    #[test]
    fn test_decode_short_string() {
        // "dog" = [0x83, 'd', 'o', 'g']
        let result = RlpItem::decode(&[0x83, b'd', b'o', b'g']).unwrap();
        assert_eq!(result, RlpItem::Data(b"dog".to_vec()));
    }

    #[test]
    fn test_decode_empty_list() {
        // Empty list is 0xc0
        let result = RlpItem::decode(&[0xc0]).unwrap();
        assert_eq!(result, RlpItem::List(vec![]));
    }

    #[test]
    fn test_decode_list() {
        // List containing ["cat", "dog"]
        // 0xc8 (list with 8 byte payload)
        // 0x83 'c' 'a' 't' (string "cat")
        // 0x83 'd' 'o' 'g' (string "dog")
        let bytes = &[0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
        let result = RlpItem::decode(bytes).unwrap();

        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].as_data().unwrap(), b"cat");
        assert_eq!(list[1].as_data().unwrap(), b"dog");
    }

    #[test]
    fn test_as_u64() {
        // Integer 15 is encoded as 0x0f (single byte)
        let item = RlpItem::decode(&[0x0f]).unwrap();
        assert_eq!(item.as_u64().unwrap(), 15);

        // Integer 1024 is encoded as 0x82 0x04 0x00 (2-byte string)
        let item = RlpItem::decode(&[0x82, 0x04, 0x00]).unwrap();
        assert_eq!(item.as_u64().unwrap(), 1024);
    }

    #[test]
    fn test_as_u128() {
        // Large value
        let item =
            RlpItem::decode(&[0x88, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]).unwrap();
        assert_eq!(item.as_u128().unwrap(), 0x0102030405060708u128);
    }

    #[test]
    fn test_invalid_leading_zeros() {
        // Non-canonical: integer with leading zero
        // RLP decoding succeeds (it's valid RLP for a string)
        let result = RlpItem::decode(&[0x82, 0x00, 0x01]);
        assert!(result.is_ok());

        // But converting to integer should fail due to leading zero
        let item = result.unwrap();
        assert!(
            item.as_u64().is_err(),
            "Should reject integer with leading zeros"
        );
        assert!(
            item.as_u128().is_err(),
            "Should reject integer with leading zeros"
        );
    }
}
