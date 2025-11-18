//! XRP binary codec parser
//!
//! XRP Ledger uses a custom binary serialization format with:
//! - Field headers (field type + field ID)
//! - Canonical field ordering (sorted by field ID)
//! - Special amount encoding (XRP drops vs IOU format)

use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Field types in XRP binary format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FieldType {
    UInt16 = 1,
    UInt32 = 2,
    UInt64 = 3,
    Hash256 = 5,
    Amount = 6,
    Blob = 7,
    AccountId = 8,
    Object = 14,
    Array = 15,
    UInt8 = 16,
    Hash160 = 17,
    PathSet = 18,
    Vector256 = 19,
}

impl FieldType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(FieldType::UInt16),
            2 => Ok(FieldType::UInt32),
            3 => Ok(FieldType::UInt64),
            5 => Ok(FieldType::Hash256),
            6 => Ok(FieldType::Amount),
            7 => Ok(FieldType::Blob),
            8 => Ok(FieldType::AccountId),
            14 => Ok(FieldType::Object),
            15 => Ok(FieldType::Array),
            16 => Ok(FieldType::UInt8),
            17 => Ok(FieldType::Hash160),
            18 => Ok(FieldType::PathSet),
            19 => Ok(FieldType::Vector256),
            _ => Err(DecoderError::invalid_structure(format!(
                "Unknown field type: {}",
                value
            ))),
        }
    }
}

/// XRP amount representation
#[derive(Debug, Clone, PartialEq)]
pub enum XrpAmount {
    /// XRP drops (1 XRP = 1,000,000 drops)
    Drops(u64),
    /// Issued currency (IOU)
    Iou {
        value: String,      // Decimal string representation
        currency: [u8; 20], // Currency code (20 bytes)
        issuer: [u8; 20],   // Issuer account ID (20 bytes)
    },
}

/// Binary codec reader for XRP transactions
pub struct BinaryCodec<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> BinaryCodec<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    /// Read a single byte
    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u8: {}", e)))?;
        Ok(buf[0])
    }

    /// Read a u16 (big-endian)
    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u16: {}", e)))?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Read a u32 (big-endian)
    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u32: {}", e)))?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Read a u64 (big-endian)
    #[allow(dead_code)]
    pub fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u64: {}", e)))?;
        Ok(u64::from_be_bytes(buf))
    }

    /// Read exact number of bytes
    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to read {} bytes: {}", len, e))
        })?;
        Ok(buf)
    }

    /// Read variable-length field (with length prefix)
    pub fn read_var_length(&mut self) -> Result<Vec<u8>> {
        let len1 = self.read_u8()?;
        let len = if len1 <= 192 {
            len1 as usize
        } else if len1 <= 240 {
            let len2 = self.read_u8()?;
            193 + ((len1 - 193) as usize) * 256 + (len2 as usize)
        } else if len1 <= 254 {
            let len2 = self.read_u8()?;
            let len3 = self.read_u8()?;
            12481 + ((len1 - 241) as usize) * 65536 + (len2 as usize) * 256 + (len3 as usize)
        } else {
            return Err(DecoderError::invalid_structure(
                "Invalid variable length encoding",
            ));
        };

        self.read_bytes(len)
    }

    /// Read field header (returns field type and field ID)
    pub fn read_field_header(&mut self) -> Result<Option<(FieldType, u16)>> {
        let position = self.cursor.position();
        let data_len = self.cursor.get_ref().len() as u64;

        if position >= data_len {
            return Ok(None); // End of data
        }

        let first_byte = self.read_u8()?;

        // Extract type code (upper 4 bits) and field code (lower 4 bits)
        let type_code = (first_byte >> 4) & 0x0F;
        let field_code = first_byte & 0x0F;

        // Read extended type if needed
        let field_type = if type_code == 0 {
            let extended_type = self.read_u8()?;
            FieldType::from_u8(extended_type)?
        } else {
            FieldType::from_u8(type_code)?
        };

        // Read extended field ID if needed
        let field_id = if field_code == 0 {
            self.read_u8()? as u16
        } else {
            field_code as u16
        };

        Ok(Some((field_type, field_id)))
    }

    /// Read an amount field (XRP drops or IOU)
    pub fn read_amount(&mut self) -> Result<XrpAmount> {
        let first_byte = self.read_u8()?;

        // Check bit 7 (0x80): 0 = XRP, 1 = IOU
        if first_byte & 0x80 == 0 {
            // XRP amount (positive)
            let mut amount_bytes = [0u8; 8];
            amount_bytes[0] = first_byte & 0x3F; // Clear bit 7 and 6
            self.cursor
                .read_exact(&mut amount_bytes[1..])
                .map_err(|e| {
                    DecoderError::invalid_structure(format!("Failed to read XRP amount: {}", e))
                })?;
            let drops = u64::from_be_bytes(amount_bytes);
            Ok(XrpAmount::Drops(drops))
        } else {
            // IOU amount (48 bytes total)
            // First 8 bytes: amount with exponent
            let mut amount_bytes = [0u8; 8];
            amount_bytes[0] = first_byte;
            self.cursor
                .read_exact(&mut amount_bytes[1..])
                .map_err(|e| {
                    DecoderError::invalid_structure(format!("Failed to read IOU amount: {}", e))
                })?;

            // Decode the mantissa and exponent
            let bits = u64::from_be_bytes(amount_bytes);
            let is_positive = (bits & 0x4000_0000_0000_0000) != 0;
            let exponent = ((bits >> 54) & 0xFF) as i32 - 97;
            let mantissa = bits & 0x3F_FFFF_FFFF_FFFF;

            // Convert to decimal string
            let value = if mantissa == 0 {
                "0".to_string()
            } else {
                let mut val = mantissa as f64;
                val *= 10_f64.powi(exponent - 16);
                if !is_positive {
                    val = -val;
                }
                format!("{}", val)
            };

            // Read currency code (20 bytes)
            let mut currency = [0u8; 20];
            self.cursor.read_exact(&mut currency).map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to read currency: {}", e))
            })?;

            // Read issuer account ID (20 bytes)
            let mut issuer = [0u8; 20];
            self.cursor.read_exact(&mut issuer).map_err(|e| {
                DecoderError::invalid_structure(format!("Failed to read issuer: {}", e))
            })?;

            Ok(XrpAmount::Iou {
                value,
                currency,
                issuer,
            })
        }
    }

    /// Read account ID (20 bytes with variable-length prefix)
    pub fn read_account_id(&mut self) -> Result<[u8; 20]> {
        let data = self.read_var_length()?;
        if data.len() != 20 {
            return Err(DecoderError::invalid_structure(format!(
                "Account ID must be 20 bytes, got {}",
                data.len()
            )));
        }
        let mut account_id = [0u8; 20];
        account_id.copy_from_slice(&data);
        Ok(account_id)
    }

    /// Read hash256 (32 bytes)
    pub fn read_hash256(&mut self) -> Result<[u8; 32]> {
        let data = self.read_bytes(32)?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&data);
        Ok(hash)
    }

    /// Get current position
    #[allow(dead_code)]
    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    /// Check if we're at the end
    #[allow(dead_code)]
    pub fn is_at_end(&self) -> bool {
        self.cursor.position() >= self.cursor.get_ref().len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x12, 0x34];
        let mut codec = BinaryCodec::new(&data);
        assert_eq!(codec.read_u8().unwrap(), 0x12);
        assert_eq!(codec.read_u8().unwrap(), 0x34);
    }

    #[test]
    fn test_read_u16() {
        let data = [0x12, 0x34];
        let mut codec = BinaryCodec::new(&data);
        assert_eq!(codec.read_u16().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u32() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let mut codec = BinaryCodec::new(&data);
        assert_eq!(codec.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_xrp_amount() {
        // XRP amount: 1000000 drops (1 XRP)
        // Bit 7 = 0 (XRP), bit 6 = 0 (positive)
        let data = [0x40, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x42, 0x40];
        let mut codec = BinaryCodec::new(&data);
        match codec.read_amount().unwrap() {
            XrpAmount::Drops(drops) => assert_eq!(drops, 1000000),
            _ => panic!("Expected XRP drops"),
        }
    }

    #[test]
    fn test_read_var_length_short() {
        // Short length: 5 bytes
        let data = [5, 0x01, 0x02, 0x03, 0x04, 0x05];
        let mut codec = BinaryCodec::new(&data);
        let result = codec.read_var_length().unwrap();
        assert_eq!(result, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_field_type_from_u8() {
        assert_eq!(FieldType::from_u8(1).unwrap(), FieldType::UInt16);
        assert_eq!(FieldType::from_u8(6).unwrap(), FieldType::Amount);
        assert_eq!(FieldType::from_u8(8).unwrap(), FieldType::AccountId);
        assert!(FieldType::from_u8(99).is_err());
    }
}
