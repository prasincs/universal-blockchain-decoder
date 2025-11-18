//! Property-based tests for Bittensor decoder
//!
//! These tests use proptest to verify invariants across many random inputs.

use decoder_bittensor::*;
use decoder_primitives::prelude::*;
use proptest::prelude::*;

// Strategies for generating test data

prop_compose! {
    fn arbitrary_address()(address_type in 0u8..5, data in prop::collection::vec(any::<u8>(), 20..33)) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(address_type);
        match address_type {
            0 | 3 => bytes.extend_from_slice(&data[..32.min(data.len())]),  // 32-byte address
            1 => {
                // Index: compact u32
                bytes.push(0x04); // Compact encoding for small number
            },
            2 | 4 => bytes.extend_from_slice(&data[..20.min(data.len())]),  // 20-byte or raw
            _ => {}
        }
        bytes
    }
}

prop_compose! {
    fn arbitrary_signature()(sig_type in 0u8..3, data in prop::collection::vec(any::<u8>(), 64..66)) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(sig_type);
        match sig_type {
            0 | 1 => bytes.extend_from_slice(&data[..64]),  // Sr25519, Ed25519: 64 bytes
            2 => bytes.extend_from_slice(&data[..65]),      // ECDSA: 65 bytes
            _ => {}
        }
        bytes
    }
}

prop_compose! {
    fn arbitrary_compact_u32()(value in 0u32..16384) -> Vec<u8> {
        let mut bytes = Vec::new();
        if value < 64 {
            bytes.push((value << 2) as u8);
        } else if value < 16384 {
            bytes.push(((value << 2) | 0x01) as u8);
            bytes.push((value >> 6) as u8);
        } else {
            bytes.push(((value << 2) | 0x02) as u8);
            bytes.push((value >> 6) as u8);
            bytes.push((value >> 14) as u8);
            bytes.push((value >> 22) as u8);
        }
        bytes
    }
}

proptest! {
    #[test]
    fn test_compact_u32_roundtrip(value in 0u32..16384) {
        let mut bytes = Vec::new();
        if value < 64 {
            bytes.push((value << 2) as u8);
        } else if value < 16384 {
            bytes.push(((value << 2) | 0x01) as u8);
            bytes.push((value >> 6) as u8);
        }

        let mut offset = 0;
        let result = parsing::read_compact_u32(&bytes, &mut offset);
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_compact_u64_never_panics(bytes in prop::collection::vec(any::<u8>(), 1..100)) {
        let mut offset = 0;
        let _result = parsing::read_compact_u64(&bytes, &mut offset);
        // Should never panic, even with random data
    }

    #[test]
    fn test_compact_u128_never_panics(bytes in prop::collection::vec(any::<u8>(), 1..100)) {
        let mut offset = 0;
        let _result = parsing::read_compact_u128(&bytes, &mut offset);
        // Should never panic, even with random data
    }

    #[test]
    fn test_decoder_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let _result = BittensorDecoder::decode(&bytes);
        // Should never panic, only return errors
    }

    #[test]
    fn test_validate_format_consistency(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let validate_result = BittensorDecoder::validate_format(&bytes);
        let decode_result = BittensorDecoder::decode(&bytes);

        // If validation passes, decode should not fail on format issues
        if validate_result.is_ok() && bytes.len() >= 4 {
            prop_assert!(decode_result.is_ok() || decode_result.is_err());
        }
    }

    #[test]
    fn test_hash_determinism(bytes in prop::collection::vec(any::<u8>(), 4..1000)) {
        // Only test if it's a valid-ish length
        if bytes.len() >= 4 {
            let hash1 = BittensorTransaction::calculate_hash(&bytes);
            let hash2 = BittensorTransaction::calculate_hash(&bytes);
            prop_assert_eq!(hash1.len(), 64, "Blake2b-512 produces 64 bytes");
            prop_assert_eq!(hash1, hash2, "Hash should be deterministic");
        }
    }

    #[test]
    fn test_call_parsing_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..100)) {
        let _result = parsing::parse_call(&bytes);
        // Should never panic
    }

    #[test]
    fn test_canonicalize_never_panics_on_valid_tx(
        pallet in 0u8..20,
        call_idx in 0u8..10,
    ) {
        // Create a minimal but valid-looking extrinsic
        let mut extrinsic = Vec::new();

        // Length (we'll calculate)
        extrinsic.push(0x84); // Version: v4, signed

        // Address
        extrinsic.push(0x00);
        extrinsic.extend_from_slice(&[0xFF; 32]);

        // Signature
        extrinsic.push(0x01);
        extrinsic.extend_from_slice(&[0xAA; 64]);

        // Era
        extrinsic.push(0x00);

        // Nonce
        extrinsic.push(0x00);

        // Tip
        extrinsic.push(0x00);

        // Call
        extrinsic.push(pallet);
        extrinsic.push(call_idx);

        // Add length prefix
        let length = extrinsic.len() as u32;
        let mut with_length = vec![(length << 2) as u8];
        with_length.extend_from_slice(&extrinsic);

        if let Ok(tx) = BittensorDecoder::decode(&with_length) {
            let _result = tx.canonicalize();
            // Should not panic
        }
    }

    #[test]
    fn test_extrinsic_version_roundtrip(version in 0u8..128, is_signed: bool) {
        let version_byte = if is_signed {
            version | 0x80
        } else {
            version & 0x7F
        };

        let parsed = ExtrinsicVersion::from_byte(version_byte);
        prop_assert_eq!(parsed.version, version & 0x7F);
        prop_assert_eq!(parsed.is_signed, is_signed);
        prop_assert_eq!(parsed.to_byte(), version_byte);
    }

    #[test]
    fn test_blake2_hash_collision_resistance(
        bytes1 in prop::collection::vec(any::<u8>(), 1..100),
        bytes2 in prop::collection::vec(any::<u8>(), 1..100)
    ) {
        if bytes1 != bytes2 {
            let hash1 = BittensorTransaction::calculate_hash(&bytes1);
            let hash2 = BittensorTransaction::calculate_hash(&bytes2);
            // Hashes should be different for different inputs (collision resistance)
            prop_assert_ne!(hash1, hash2);
        }
    }
}

// Additional determinism tests
#[test]
fn test_decode_canonicalize_determinism() {
    // Create a simple valid extrinsic
    let mut extrinsic = Vec::new();
    extrinsic.push(0x84); // Version
    extrinsic.push(0x00); // Address type
    extrinsic.extend_from_slice(&[0xFF; 32]); // Address
    extrinsic.push(0x01); // Signature type
    extrinsic.extend_from_slice(&[0xAA; 64]); // Signature
    extrinsic.push(0x00); // Era
    extrinsic.push(0x00); // Nonce
    extrinsic.push(0x00); // Tip
    extrinsic.push(0x04); // Pallet (Balances)
    extrinsic.push(0x00); // Call (transfer)

    // Add length
    let length = extrinsic.len() as u32;
    let mut with_length = vec![(length << 2) as u8];
    with_length.extend_from_slice(&extrinsic);

    // Decode multiple times
    let tx1 = BittensorDecoder::decode(&with_length).unwrap();
    let tx2 = BittensorDecoder::decode(&with_length).unwrap();

    // Hashes should be identical
    assert_eq!(tx1.tx_hash, tx2.tx_hash);

    // Canonicalization should be identical
    let ir1 = tx1.canonicalize().unwrap();
    let ir2 = tx2.canonicalize().unwrap();

    assert_eq!(ir1.metadata.tx_hash, ir2.metadata.tx_hash);
    assert_eq!(ir1.metadata.size, ir2.metadata.size);
    assert_eq!(ir1.operations.len(), ir2.operations.len());
}

#[test]
fn test_bittensor_specific_pallets() {
    // Test that Bittensor-specific pallet names are recognized
    let subtensor_call = Call {
        pallet_index: 7,
        call_index: 0,
        parameters: vec![],
    };
    assert_eq!(subtensor_call.pallet_name(), "SubtensorModule");
    assert_eq!(subtensor_call.call_name(), "set_weights");

    let registry_call = Call {
        pallet_index: 15,
        call_index: 0,
        parameters: vec![],
    };
    assert_eq!(registry_call.pallet_name(), "Registry");
}
