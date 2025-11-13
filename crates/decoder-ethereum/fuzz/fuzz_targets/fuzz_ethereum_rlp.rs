#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_encodings::rlp::RlpItem;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: RLP decoder should never panic on arbitrary input
    //
    // RLP (Recursive Length Prefix) encoding is used by Ethereum for
    // transaction serialization. This fuzzer ensures our RLP parser
    // handles all edge cases correctly:
    // 1. Malformed length prefixes
    // 2. Nested list structures
    // 3. Very large lengths
    // 4. Truncated data
    // 5. Invalid structure

    // Test 1: RLP decode should never panic
    let _ = RlpItem::decode(data);

    // Test 2: If decode succeeds, accessing items should not panic
    if let Ok(item) = RlpItem::decode(data) {
        // Try to access as different types
        let _ = item.as_data();
        let _ = item.as_list();

        // If it's a list, traverse it
        if let Some(list) = item.as_list() {
            for sub_item in list {
                let _ = sub_item.as_data();
                let _ = sub_item.as_list();

                // Recursively check nested lists
                if let Some(nested_list) = sub_item.as_list() {
                    for nested_item in nested_list {
                        let _ = nested_item.as_data();
                        let _ = nested_item.as_list();
                    }
                }
            }
        }
    }

    // Test 3: RLP encoding roundtrip (if we can decode)
    if let Ok(item) = RlpItem::decode(data) {
        if let Some(bytes) = item.as_data() {
            // Re-encode and verify it doesn't panic
            use decoder_encodings::rlp::encode_bytes;
            let _ = encode_bytes(bytes);
        }
    }

    // Test 4: Very large RLP structures should be rejected gracefully
    if data.len() > 10_000_000 {
        // Should reject, not panic
        assert!(RlpItem::decode(data).is_err());
    }

    // Test 5: Specific RLP edge cases
    if !data.is_empty() {
        let first_byte = data[0];

        match first_byte {
            // Single byte (0x00-0x7f): direct value
            0x00..=0x7f => {
                let _ = RlpItem::decode(data);
            }
            // Short string (0x80-0xb7): length in first byte
            0x80..=0xb7 => {
                let _ = RlpItem::decode(data);
            }
            // Long string (0xb8-0xbf): length of length in first byte
            0xb8..=0xbf => {
                let _ = RlpItem::decode(data);
            }
            // Short list (0xc0-0xf7): length in first byte
            0xc0..=0xf7 => {
                let _ = RlpItem::decode(data);
            }
            // Long list (0xf8-0xff): length of length in first byte
            0xf8..=0xff => {
                let _ = RlpItem::decode(data);
            }
        }
    }
});
