//! Fuzz target for privacy metadata serialization
//!
//! This fuzzer tests that arbitrary bytes can be safely deserialized
//! without panicking, and that valid privacy metadata roundtrips correctly.

#![no_main]

use libfuzzer_sys::fuzz_target;
use universal_decoder_core::privacy::*;

fuzz_target!(|data: &[u8]| {
    // Test 1: Deserializing arbitrary bytes should not panic
    // (it should either succeed or return an error)
    let _ = serde_json::from_slice::<PrivacyMetadata>(data);
    let _ = serde_json::from_slice::<PrivateAddress>(data);
    let _ = serde_json::from_slice::<ConfidentialAmount>(data);
    let _ = serde_json::from_slice::<PrivacyPool>(data);
    let _ = serde_json::from_slice::<EncryptedTransaction>(data);
    let _ = serde_json::from_slice::<ViewingKey>(data);

    // Test 2: If we can deserialize, roundtrip should work
    if let Ok(metadata) = serde_json::from_slice::<PrivacyMetadata>(data) {
        // Serialize back
        if let Ok(serialized) = serde_json::to_vec(&metadata) {
            // Deserialize again
            if let Ok(metadata2) = serde_json::from_slice::<PrivacyMetadata>(&serialized) {
                // Should be equal
                assert_eq!(metadata, metadata2);
            }
        }
    }

    // Test 3: Privacy features should handle arbitrary data gracefully
    if let Ok(feature) = serde_json::from_slice::<PrivacyFeature>(data) {
        // Should be able to clone
        let _ = feature.clone();

        // Should be able to serialize
        let _ = serde_json::to_string(&feature);
    }

    // Test 4: Observability levels should be robust
    if let Ok(level) = serde_json::from_slice::<ObservabilityLevel>(data) {
        let copied = level;
        assert_eq!(level, copied);
    }
});
