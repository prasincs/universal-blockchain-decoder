#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_bitcoin::BitcoinDecoder;
use universal_decoder_core::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: Bitcoin decoder should never panic on arbitrary input
    //
    // This fuzz test explores the input space to find:
    // 1. Inputs that cause panics (bugs)
    // 2. Inputs that cause unexpected behavior
    // 3. Edge cases in parsing logic
    //
    // The decoder should always return Result::Ok or Result::Err,
    // never panic.

    // Test 1: Decode should never panic
    let _ = BitcoinDecoder::decode(data);

    // Test 2: Validate format should never panic
    let _ = BitcoinDecoder::validate_format(data);

    // Test 3: If decode succeeds, canonicalization should not panic
    if let Ok(tx) = BitcoinDecoder::decode(data) {
        // Canonicalization may fail (Err) but should never panic
        let _ = tx.canonicalize();

        // Test basic properties without panicking
        let _ = tx.version();
        let _ = tx.is_coinbase();
        let _ = tx.is_segwit();
        let _ = tx.txid();
        let _ = tx.validate();

        // Note: calculate_fee() requires input_values which we don't have
        // from arbitrary bytes, so we skip it in fuzzing

        // If canonicalization succeeds, hashing should not panic
        if let Ok(tx_ir) = tx.canonicalize() {
            let _ = tx_ir.canonical_hash();
            let _ = tx_ir.to_canonical_bytes();
        }
    }

    // Test 4: Very large inputs should be rejected gracefully
    // (DoS protection - no panic even with huge inputs)
    if data.len() > 1_000_000 {
        // Decoder should reject with error, not panic
        assert!(BitcoinDecoder::decode(data).is_err());
    }
});
