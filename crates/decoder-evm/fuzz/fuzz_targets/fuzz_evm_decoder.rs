#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_evm::EvmDecoder;
use universal_decoder_core::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: EVM decoder should never panic on arbitrary input
    //
    // This fuzzer tests the generic EVM decoder which supports 500+ chains.
    // It ensures that regardless of the chain, the decoder handles all inputs safely.

    if data.len() < 2 {
        return;
    }

    // Use first two bytes to determine chain ID (0-65535)
    let chain_id = u64::from(u16::from_le_bytes([data[0], data[1]]));
    let tx_data = &data[2..];

    // Create decoder for this chain
    let decoder = EvmDecoder::new(chain_id);

    // Test 1: Decode should never panic
    let _ = decoder.decode(tx_data);

    // Test 2: Chain identity operations should never panic
    let _ = decoder.chain_id();
    let _ = decoder.chain_name();
    let _ = decoder.chain_family();

    // Test 3: If decode succeeds, all operations should work
    if let Ok(tx) = decoder.decode(tx_data) {
        // Transaction properties
        let _ = tx.nonce();
        let _ = tx.gas_limit();
        let _ = tx.gas_price();
        let _ = tx.value();
        let _ = tx.data();
        let _ = tx.chain_id();

        // Transaction type
        let _ = tx.is_legacy();
        let _ = tx.is_eip1559();
        let _ = tx.is_contract_creation();

        // Hashing
        let _ = tx.hash();

        // Validation
        let _ = tx.validate();

        // Canonicalization
        if let Ok(tx_ir) = tx.canonicalize() {
            let _ = tx_ir.canonical_hash();
            let _ = tx_ir.to_canonical_bytes();

            // Verify chain ID is preserved
            assert_eq!(tx_ir.metadata().chain_id, chain_id);
        }

        // Signature
        let _ = tx.v();
        let _ = tx.r();
        let _ = tx.s();

        // Addresses
        let _ = tx.to();
        let _ = tx.from();
    }

    // Test 4: Very large inputs should be rejected
    if tx_data.len() > 1_000_000 {
        assert!(decoder.decode(tx_data).is_err());
    }
});
