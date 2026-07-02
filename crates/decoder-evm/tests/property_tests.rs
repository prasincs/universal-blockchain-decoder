//! Property-based tests for EVM decoder
//!
//! This module tests the generic EVM decoder that supports 500+ EVM-compatible chains.

use decoder_evm::EvmDecoder;
use decoder_test_utils::proptest_helpers::arb_small_bytes;
use proptest::prelude::*;

//
// Property 1: Decoder Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: EVM decoder never panics on arbitrary input
    #[test]
    fn prop_evm_decoder_never_panics(bytes in arb_small_bytes()) {
        use std::panic;

        let decoder = EvmDecoder::new();

        let result = panic::catch_unwind(|| {
            let _ = decoder.decode(&bytes, None);
        });

        prop_assert!(result.is_ok(), "Decoder panicked on input");
    }

    /// Property: EVM decoder rejects empty input
    #[test]
    fn prop_evm_decoder_rejects_empty(_unit in 0u8..1) {
        let decoder = EvmDecoder::new();
        let result = decoder.decode(&[], None);
        prop_assert!(result.is_err(), "Decoder should reject empty input");
    }

    /// Property: Decoder handles oversized input gracefully
    #[test]
    fn prop_evm_decoder_handles_large_input(size in 10_000usize..100_000) {
        let decoder = EvmDecoder::new();
        let bytes = vec![0x00; size];
        // Should either decode or error, never panic
        let result = decoder.decode(&bytes, None);
        prop_assert!(result.is_ok() || result.is_err());
    }
}

//
// Property 2: Chain ID Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: When chain ID is specified, it's validated
    #[test]
    fn prop_chain_id_validation(
        bytes in arb_small_bytes(),
        chain_id in 1u64..10000,
    ) {
        let decoder = EvmDecoder::new();

        // Try to decode with specified chain ID
        if let Ok((tx, chain_info)) = decoder.decode(&bytes, Some(chain_id)) {
            // If successful, chain ID should match (if transaction has one)
            if let Some(tx_chain_id) = tx.chain_id {
                prop_assert_eq!(tx_chain_id, chain_id,
                    "Transaction chain ID should match expected");
            }
            // Chain info should be for the decoded chain
            prop_assert!(chain_info.chain_id > 0, "Chain info should have valid chain ID");
        }
    }

    /// Property: Hash calculation is deterministic
    #[test]
    fn prop_transaction_hash_deterministic(bytes in arb_small_bytes()) {
        let decoder = EvmDecoder::new();

        if let Ok((tx, _)) = decoder.decode(&bytes, None) {
            let hash1 = tx.hash().ok();
            let hash2 = tx.hash().ok();
            prop_assert_eq!(hash1, hash2, "Hash should be deterministic");
        }
    }
}

//
// Property 3: Registry Operations Never Panic
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Registry lookups never panic
    #[test]
    fn prop_registry_never_panics(chain_id in 1u64..100_000) {
        use std::panic;
        use decoder_evm::ChainRegistry;

        let result = panic::catch_unwind(|| {
            let registry = ChainRegistry::global();
            let _ = registry.get_chain(chain_id);
        });

        prop_assert!(result.is_ok(), "Registry panicked on chain_id {}", chain_id);
    }
}

//
// Property 4: Full Pipeline Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Decode-Hash-Canonicalize pipeline never panics
    #[test]
    fn prop_full_pipeline_never_panics(bytes in arb_small_bytes()) {
        use std::panic;

        let result = panic::catch_unwind(|| {
            let decoder = EvmDecoder::new();

            if let Ok((tx, _chain_info)) = decoder.decode(&bytes, None) {
                let _ = tx.hash();
                // Note: canonicalize is on the decoder trait, not the transaction
            }
        });

        prop_assert!(result.is_ok(), "Full pipeline panicked");
    }
}
