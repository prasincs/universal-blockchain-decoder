#![no_main]

use libfuzzer_sys::fuzz_target;
use universal_decoder_core::ir::*;
use borsh::{BorshSerialize, BorshDeserialize};

fuzz_target!(|data: &[u8]| {
    // Try to deserialize arbitrary bytes as TxMetadata
    if let Ok(metadata) = borsh::from_slice::<TxMetadata>(data) {
        // If deserialization succeeds, canonical serialization must not panic
        let _ = borsh::to_vec(&metadata);

        // And should be deterministic
        let bytes1 = borsh::to_vec(&metadata).unwrap();
        let bytes2 = borsh::to_vec(&metadata).unwrap();
        assert_eq!(bytes1, bytes2, "Canonical serialization must be deterministic");
    }

    // Try to deserialize as Amount
    if let Ok(amount) = borsh::from_slice::<Amount>(data) {
        let _ = borsh::to_vec(&amount);
    }

    // Try to deserialize as Address
    if let Ok(address) = borsh::from_slice::<Address>(data) {
        let _ = borsh::to_vec(&address);
    }
});
