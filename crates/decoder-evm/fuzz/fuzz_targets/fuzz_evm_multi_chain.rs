#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_evm::EvmDecoder;
use universal_decoder_core::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: Cross-chain consistency
    //
    // This fuzzer tests that the same transaction bytes produce consistent
    // results across different EVM chains (except for chain-specific validation).

    if data.len() < 10 {
        return;
    }

    // Extract multiple chain IDs from the input
    let chain_ids = [
        u64::from(u16::from_le_bytes([data[0], data[1]])),
        u64::from(u16::from_le_bytes([data[2], data[3]])),
        u64::from(u16::from_le_bytes([data[4], data[5]])),
    ];

    let tx_data = &data[6..];

    // Test 1: Decode on multiple chains
    let mut decoded_txs = Vec::new();
    for &chain_id in &chain_ids {
        let decoder = EvmDecoder::new(chain_id);
        if let Ok(tx) = decoder.decode(tx_data) {
            decoded_txs.push((chain_id, tx));
        }
    }

    // Test 2: If multiple chains successfully decode, verify consistency
    if decoded_txs.len() >= 2 {
        let (chain_id1, tx1) = &decoded_txs[0];
        let (chain_id2, tx2) = &decoded_txs[1];

        // Structural fields should match
        assert_eq!(
            tx1.nonce(),
            tx2.nonce(),
            "Nonce mismatch between chain {} and {}",
            chain_id1,
            chain_id2
        );

        assert_eq!(
            tx1.gas_limit(),
            tx2.gas_limit(),
            "Gas limit mismatch between chain {} and {}",
            chain_id1,
            chain_id2
        );

        assert_eq!(
            tx1.value(),
            tx2.value(),
            "Value mismatch between chain {} and {}",
            chain_id1,
            chain_id2
        );

        assert_eq!(
            tx1.data(),
            tx2.data(),
            "Data mismatch between chain {} and {}",
            chain_id1,
            chain_id2
        );

        // Transaction hash should be identical (based on bytes, not chain)
        let hash1 = tx1.hash().ok();
        let hash2 = tx2.hash().ok();
        assert_eq!(
            hash1, hash2,
            "Transaction hash differs between chain {} and {}",
            chain_id1, chain_id2
        );

        // Signature components should match
        assert_eq!(
            tx1.r(),
            tx2.r(),
            "Signature r differs between chains"
        );
        assert_eq!(
            tx1.s(),
            tx2.s(),
            "Signature s differs between chains"
        );
    }

    // Test 3: Canonicalization should work on all chains
    for (chain_id, tx) in decoded_txs {
        if let Ok(tx_ir) = tx.canonicalize() {
            // Verify chain ID is correct
            assert_eq!(tx_ir.metadata().chain_id, chain_id);

            // Canonical operations should be deterministic
            let hash1 = tx_ir.canonical_hash();
            let hash2 = tx_ir.canonical_hash();

            if let (Ok(h1), Ok(h2)) = (hash1, hash2) {
                assert_eq!(h1, h2, "Canonical hash non-deterministic on chain {}", chain_id);
            }
        }
    }
});
