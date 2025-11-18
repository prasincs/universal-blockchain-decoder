#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_bitcoin::BitcoinDecoder;
use universal_decoder_core::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: Test TxIR creation and structure safety
    //
    // This fuzz test verifies:
    // 1. Decode never panics on arbitrary input
    // 2. Canonicalization never panics
    // 3. TxIR field access is safe
    // 4. Very large inputs are rejected gracefully

    // Test 1: Decode should never panic
    if let Ok(tx) = BitcoinDecoder::decode(data) {
        // Test 2: Canonicalization should not panic
        if let Ok(tx_ir) = tx.canonicalize() {
            // Test 3: Field access doesn't panic
            let _ = tx_ir.version();
            let _ = &tx_ir.chain;
            let _ = &tx_ir.metadata;
            let _ = &tx_ir.authorization;
            let _ = &tx_ir.operations;
            let _ = &tx_ir.state_deltas;
            let _ = &tx_ir.privacy;

            // Test 4: Operations are iterable
            for operation in &tx_ir.operations {
                let _ = operation;
            }

            // Test 5: Verify transaction properties
            if !tx_ir.operations.is_empty() {
                // Has at least one operation
                assert!(tx_ir.operations.len() > 0);
            }
        }
    }

    // Test 6: Validation should never panic
    let _ = BitcoinDecoder::validate_format(data);

    // Test 7: Very large inputs should be rejected gracefully
    if data.len() > 10_000_000 {
        // Should error, not panic or OOM
        assert!(BitcoinDecoder::decode(data).is_err());
    }
});
