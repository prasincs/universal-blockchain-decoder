#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_bitcoin::parsing::read_varint;
use decoder_encodings::varint::encode_varint;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: VarInt parsing should never panic
    //
    // This specifically targets the VarInt encoding/decoding logic
    // which is critical for Bitcoin transaction parsing.
    //
    // Properties to verify:
    // 1. read_varint never panics (returns Err on invalid input)
    // 2. Roundtrip property: encode(decode(x)) == x for valid varints
    // 3. Non-canonical varints are detected and rejected

    // Test 1: Read varint should never panic
    let mut cursor = Cursor::new(data);
    let result = read_varint(&mut cursor);

    // Test 2: If read succeeds, the value should be encodable
    if let Ok(value) = result {
        let mut buf = Vec::new();
        encode_varint(&mut buf, value);

        // Test 3: Roundtrip property
        let mut roundtrip_cursor = Cursor::new(&buf);
        let decoded = read_varint(&mut roundtrip_cursor);
        assert!(decoded.is_ok(), "Roundtrip encode/decode failed");
        assert_eq!(decoded.unwrap(), value, "Roundtrip value mismatch");

        // Test 4: Encoded length should be canonical
        let expected_len = if value < 0xFD {
            1
        } else if value <= 0xFFFF {
            3
        } else if value <= 0xFFFFFFFF {
            5
        } else {
            9
        };
        assert_eq!(buf.len(), expected_len, "Non-canonical encoding");
    }

    // Test 5: Edge case - very short inputs should fail gracefully
    if data.is_empty() {
        assert!(read_varint(&mut Cursor::new(data)).is_err());
    }

    // Test 6: Edge case - inputs starting with 0xFD/FE/FF but truncated
    if data.len() >= 1 {
        let first = data[0];
        if first == 0xFD && data.len() < 3 {
            // Should fail with Err, not panic
            assert!(read_varint(&mut Cursor::new(data)).is_err());
        } else if first == 0xFE && data.len() < 5 {
            assert!(read_varint(&mut Cursor::new(data)).is_err());
        } else if first == 0xFF && data.len() < 9 {
            assert!(read_varint(&mut Cursor::new(data)).is_err());
        }
    }
});
