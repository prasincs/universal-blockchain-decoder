//! VarInt encoding utilities

/// Encode a variable-length integer (VarInt) used in Bitcoin protocol
///
/// Format:
/// - < 0xFD: 1 byte
/// - 0xFD - 0xFFFF: 0xFD + 2 bytes (little-endian)
/// - 0x10000 - 0xFFFFFFFF: 0xFE + 4 bytes (little-endian)
/// - > 0xFFFFFFFF: 0xFF + 8 bytes (little-endian)
pub fn encode_varint(buf: &mut Vec<u8>, value: u64) {
    if value < 0xFD {
        buf.push(value as u8);
    } else if value <= 0xFFFF {
        buf.push(0xFD);
        buf.extend_from_slice(&(value as u16).to_le_bytes());
    } else if value <= 0xFFFFFFFF {
        buf.push(0xFE);
        buf.extend_from_slice(&(value as u32).to_le_bytes());
    } else {
        buf.push(0xFF);
        buf.extend_from_slice(&value.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_varint_small() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, 10);
        assert_eq!(buf, vec![0x0a]);
    }

    #[test]
    fn test_encode_varint_medium() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, 0xFC);
        assert_eq!(buf, vec![0xFC]);

        buf.clear();
        encode_varint(&mut buf, 0xFD);
        assert_eq!(buf, vec![0xFD, 0xFD, 0x00]);

        buf.clear();
        encode_varint(&mut buf, 0xFFFF);
        assert_eq!(buf, vec![0xFD, 0xFF, 0xFF]);
    }

    #[test]
    fn test_encode_varint_large() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, 0x10000);
        assert_eq!(buf, vec![0xFE, 0x00, 0x00, 0x01, 0x00]);

        buf.clear();
        encode_varint(&mut buf, 0xFFFFFFFF);
        assert_eq!(buf, vec![0xFE, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_encode_varint_very_large() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, 0x100000000);
        assert_eq!(
            buf,
            vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
        );
    }
}
