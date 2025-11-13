#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_ethereum::EthereumDecoder;
use universal_decoder_core::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: Ethereum decoder should never panic on arbitrary input
    //
    // This fuzz test explores the input space to find:
    // 1. Inputs that cause panics (bugs)
    // 2. Inputs that cause unexpected behavior
    // 3. Edge cases in RLP parsing logic
    // 4. Edge cases in transaction type detection
    //
    // The decoder should always return Result::Ok or Result::Err,
    // never panic.

    // Test 1: Decode should never panic
    let _ = EthereumDecoder::decode(data);

    // Test 2: Validate format should never panic
    let _ = EthereumDecoder::validate_format(data);

    // Test 3: If decode succeeds, all methods should not panic
    if let Ok(tx) = EthereumDecoder::decode(data) {
        // Basic property access
        let _ = tx.nonce();
        let _ = tx.gas_limit();
        let _ = tx.gas_price();
        let _ = tx.value();
        let _ = tx.data();
        let _ = tx.chain_id();

        // Transaction type detection
        let _ = tx.is_legacy();
        let _ = tx.is_eip1559();
        let _ = tx.is_contract_creation();

        // Hashing operations
        let _ = tx.hash();

        // Validation
        let _ = tx.validate();

        // Canonicalization may fail but should never panic
        if let Ok(tx_ir) = tx.canonicalize() {
            let _ = tx_ir.canonical_hash();
            let _ = tx_ir.to_canonical_bytes();
        }

        // Signature access
        let _ = tx.v();
        let _ = tx.r();
        let _ = tx.s();

        // Address handling
        let _ = tx.to();
        let _ = tx.from();
    }

    // Test 4: Very large inputs should be rejected gracefully
    // (DoS protection - no panic even with huge inputs)
    if data.len() > 1_000_000 {
        // Decoder should reject with error, not panic
        assert!(EthereumDecoder::decode(data).is_err());
    }

    // Test 5: Transaction type prefix handling
    if !data.is_empty() {
        let first_byte = data[0];

        // Legacy transactions (RLP-encoded)
        if first_byte >= 0xc0 {
            let _ = EthereumDecoder::decode(data);
        }

        // Typed transactions
        if first_byte <= 0x03 {
            let _ = EthereumDecoder::decode(data);
        }
    }
});
