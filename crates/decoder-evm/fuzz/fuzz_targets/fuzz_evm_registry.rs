#![no_main]

use libfuzzer_sys::fuzz_target;
use decoder_evm::registry::EvmChainRegistry;

fuzz_target!(|data: &[u8]| {
    // Fuzz target: EVM chain registry should never panic
    //
    // The registry contains information about 500+ EVM chains.
    // This fuzzer ensures that querying the registry is always safe.

    if data.len() < 8 {
        return;
    }

    // Extract chain ID from input (u64)
    let chain_id = u64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ]);

    let registry = EvmChainRegistry::new();

    // Test 1: Registry creation should never panic
    let _new_registry = EvmChainRegistry::new();

    // Test 2: Querying chain info should never panic
    let chain_info = registry.get_chain_info(chain_id);

    // Test 3: If chain exists, verify info is valid
    if let Some(info) = chain_info {
        // Chain ID should match
        assert_eq!(info.chain_id, chain_id, "Chain ID mismatch in registry");

        // Name should be non-empty
        assert!(!info.name.is_empty(), "Chain name should not be empty");

        // Native currency should be valid
        assert!(!info.native_currency.name.is_empty(), "Currency name empty");
        assert!(!info.native_currency.symbol.is_empty(), "Currency symbol empty");
        assert!(
            info.native_currency.decimals <= 18,
            "Currency decimals should be <= 18, got {}",
            info.native_currency.decimals
        );

        // RPC URLs (if present) should be non-empty
        for rpc in &info.rpc_urls {
            assert!(!rpc.is_empty(), "RPC URL should not be empty");
        }

        // Explorers (if present) should have valid URLs
        for explorer in &info.explorers {
            assert!(!explorer.url.is_empty(), "Explorer URL should not be empty");
        }
    }

    // Test 4: Multiple queries for same chain should return same result
    let info1 = registry.get_chain_info(chain_id);
    let info2 = registry.get_chain_info(chain_id);

    match (info1, info2) {
        (Some(i1), Some(i2)) => {
            assert_eq!(i1.chain_id, i2.chain_id, "Chain ID should be consistent");
            assert_eq!(i1.name, i2.name, "Chain name should be consistent");
            assert_eq!(
                i1.native_currency.symbol, i2.native_currency.symbol,
                "Currency symbol should be consistent"
            );
        }
        (None, None) => {
            // Both None is consistent
        }
        _ => {
            panic!("Registry returned inconsistent results for chain {}", chain_id);
        }
    }

    // Test 5: Well-known chains should always be present
    let well_known_chains = [
        1u64,      // Ethereum Mainnet
        56u64,     // BNB Smart Chain
        137u64,    // Polygon
        42161u64,  // Arbitrum One
        10u64,     // Optimism
    ];

    for &well_known_chain_id in &well_known_chains {
        let info = registry.get_chain_info(well_known_chain_id);
        assert!(
            info.is_some(),
            "Well-known chain {} should be in registry",
            well_known_chain_id
        );
    }

    // Test 6: Query all chains (if fuzzer provides enough data)
    if data.len() >= 16 {
        let start_chain = u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]);
        let end_chain = u64::from_le_bytes([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15],
        ]);

        let (min_chain, max_chain) = if start_chain <= end_chain {
            (start_chain, end_chain)
        } else {
            (end_chain, start_chain)
        };

        // Limit range to prevent timeout
        let range = (max_chain - min_chain).min(1000);

        for chain_id in min_chain..min_chain + range {
            let _ = registry.get_chain_info(chain_id);
        }
    }
});
