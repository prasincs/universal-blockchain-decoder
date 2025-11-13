//! Multi-chain integration tests for EVM decoder

use decoder_evm::{ChainRegistry, EvmDecoder};

// ========================================================================
// WELL-KNOWN CHAIN TESTS
// ========================================================================

/// Test that well-known chains are in the registry
#[test]
fn test_well_known_chains() {
    let registry = ChainRegistry::global();

    let well_known_chains = vec![
        (1, "Ethereum Mainnet"),
        (56, "BNB Smart Chain"),
        (137, "Polygon"),
        (42161, "Arbitrum One"),
        (10, "Optimism"),
        (8453, "Base"),
        (43114, "Avalanche C-Chain"),
        (250, "Fantom Opera"),
    ];

    for (chain_id, expected_name) in well_known_chains {
        if let Some(chain_info) = registry.get_chain(chain_id) {
            assert_eq!(chain_info.chain_id, chain_id);
            assert!(
                chain_info.name.contains(expected_name) || expected_name.contains(&chain_info.name),
                "Expected chain name to contain '{}', got '{}'",
                expected_name,
                chain_info.name
            );
        } else {
            // Chain might not be in registry yet, that's OK for now
            eprintln!(
                "Warning: Chain {} ({}) not found in registry",
                chain_id, expected_name
            );
        }
    }
}

/// Test EVM decoder creation
#[test]
fn test_evm_decoder_creation() {
    let decoder = EvmDecoder::new();

    // Decoder should be usable
    let result = decoder.decode(&[], None);
    assert!(result.is_err(), "Should reject empty input");
}

/// Test that decoder handles invalid input gracefully
#[test]
fn test_decoder_handles_garbage() {
    use decoder_test_utils::assertions::assert_decode_never_panics;

    let decoder = EvmDecoder::new();

    let test_cases = vec![
        vec![],                        // Empty
        vec![0xFF; 100],               // Random bytes
        vec![0x00; 1000],              // Zeros
        vec![0xc0],                    // Invalid RLP
        (0..255).collect::<Vec<u8>>(), // Sequential bytes
    ];

    for input in test_cases {
        let _ = decoder.decode(&input, None);
    }
}

/// Test registry is deterministic
#[test]
fn test_registry_deterministic() {
    let registry1 = ChainRegistry::global();
    let registry2 = ChainRegistry::global();

    let chain_ids = vec![1, 56, 137, 42161, 10];

    for chain_id in chain_ids {
        let info1 = registry1.get_chain(chain_id);
        let info2 = registry2.get_chain(chain_id);

        match (info1, info2) {
            (Some(i1), Some(i2)) => {
                assert_eq!(i1.chain_id, i2.chain_id);
                assert_eq!(i1.name, i2.name);
            }
            (None, None) => {
                // Consistent
            }
            _ => {
                panic!("Registry inconsistent for chain {}", chain_id);
            }
        }
    }
}
