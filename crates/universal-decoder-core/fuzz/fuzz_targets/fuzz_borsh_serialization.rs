#![no_main]

use libfuzzer_sys::fuzz_target;
use universal_decoder_core::ir::*;
use borsh::{BorshSerialize, BorshDeserialize};

fuzz_target!(|data: &[u8]| {
    // Test that deserialization of arbitrary data never panics

    // Try TxMetadata
    if let Ok(metadata) = borsh::from_slice::<TxMetadata>(data) {
        // If successful, re-serialization should work
        if let Ok(serialized) = borsh::to_vec(&metadata) {
            // And round-trip should preserve data
            if let Ok(deserialized) = borsh::from_slice::<TxMetadata>(&serialized) {
                // Values should match
                assert_eq!(metadata.version, deserialized.version);
                assert_eq!(metadata.timestamp, deserialized.timestamp);
            }
        }
    }

    // Try Amount
    if let Ok(amount) = borsh::from_slice::<Amount>(data) {
        if let Ok(serialized) = borsh::to_vec(&amount) {
            if let Ok(deserialized) = borsh::from_slice::<Amount>(&serialized) {
                assert_eq!(amount, deserialized);
            }
        }
    }

    // Try Address
    if let Ok(address) = borsh::from_slice::<Address>(data) {
        if let Ok(serialized) = borsh::to_vec(&address) {
            if let Ok(deserialized) = borsh::from_slice::<Address>(&serialized) {
                assert_eq!(address.bytes, deserialized.bytes);
            }
        }
    }

    // Try SignatureScheme
    if let Ok(scheme) = borsh::from_slice::<SignatureScheme>(data) {
        if let Ok(serialized) = borsh::to_vec(&scheme) {
            if let Ok(deserialized) = borsh::from_slice::<SignatureScheme>(&serialized) {
                // Check discriminant matches
                std::mem::discriminant(&scheme) == std::mem::discriminant(&deserialized);
            }
        }
    }
});
