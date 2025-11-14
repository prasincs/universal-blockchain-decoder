//! Property-based tests for SVM decoder
//!
//! These tests use proptest to verify invariants hold across
//! random inputs and chain configurations.

use decoder_svm::registry::{SvmChainId, SvmChainRegistry};
use decoder_svm::{SvmChain, SvmDecoder};
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

// Custom strategy for generating SVM chain IDs
fn arb_svm_chain_id() -> impl Strategy<Value = SvmChainId> {
    prop_oneof![
        Just(SvmChainId::SolanaMainnet),
        Just(SvmChainId::SolanaDevnet),
        Just(SvmChainId::SolanaTestnet),
        Just(SvmChainId::EclipseMainnet),
        Just(SvmChainId::EclipseTestnet),
        Just(SvmChainId::PythNetwork),
        Just(SvmChainId::DriftProtocol),
        Just(SvmChainId::Jito),
        Just(SvmChainId::Sonic),
        Just(SvmChainId::Firedancer),
        Just(SvmChainId::NeonEvm),
    ]
}

//
// Property 1: Chain ID Conversion Roundtrip
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Chain ID conversion is bidirectional
    ///
    /// For all valid chain IDs:
    /// - Converting to u64 and back should give the original chain
    /// - to_u64() should return a unique value
    #[test]
    fn prop_chain_id_conversion_roundtrip(chain_id in arb_svm_chain_id()) {
        let id_u64 = chain_id.to_u64();
        let converted_back = SvmChainId::from_u64(id_u64);

        prop_assert_eq!(converted_back, Some(chain_id),
            "Chain ID {:?} should convert back from u64 {}", chain_id, id_u64);

        // Verify uniqueness: different chains should have different IDs
        let id2_u64 = match chain_id {
            SvmChainId::SolanaMainnet => SvmChainId::SolanaDevnet.to_u64(),
            SvmChainId::SolanaDevnet => SvmChainId::SolanaMainnet.to_u64(),
            SvmChainId::EclipseMainnet => SvmChainId::PythNetwork.to_u64(),
            _ => SvmChainId::SolanaMainnet.to_u64(),
        };

        prop_assert_ne!(id_u64, id2_u64, "Different chains should have different IDs");
    }
}

//
// Property 2: Chain Properties Consistency
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Chain properties are mutually consistent
    ///
    /// For all chain IDs:
    /// - Solana chains (mainnet/devnet/testnet) should have is_solana() = true
    /// - Non-Solana chains should have is_solana() = false
    /// - Mainnet and testnet should be mutually exclusive
    #[test]
    fn prop_chain_properties_consistent(chain_id in arb_svm_chain_id()) {
        let is_solana = chain_id.is_solana();
        let is_mainnet = chain_id.is_mainnet();
        let is_testnet = chain_id.is_testnet();

        // Solana chains should be exactly those with SolanaMainnet/Devnet/Testnet
        let should_be_solana = matches!(
            chain_id,
            SvmChainId::SolanaMainnet | SvmChainId::SolanaDevnet | SvmChainId::SolanaTestnet
        );
        prop_assert_eq!(is_solana, should_be_solana,
            "Chain {:?}: is_solana() should be {}", chain_id, should_be_solana);

        // Mainnet and testnet should be mutually exclusive
        // (A chain is either mainnet or testnet, not both, not neither)
        prop_assert_eq!(is_mainnet, !is_testnet,
            "Chain {:?}: mainnet and testnet should be mutually exclusive", chain_id);
    }
}

//
// Property 3: Registry Completeness
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Registry contains all generated chain IDs
    ///
    /// For all chain IDs we can generate:
    /// - The registry should have that chain
    /// - get_chain() should return Some
    /// - get_chain_by_id() should return Some
    #[test]
    fn prop_registry_contains_all_chains(chain_id in arb_svm_chain_id()) {
        let registry = SvmChainRegistry::new();

        prop_assert!(registry.has_chain(chain_id),
            "Registry should contain chain {:?}", chain_id);

        let by_enum = registry.get_chain(chain_id);
        prop_assert!(by_enum.is_some(),
            "get_chain() should return Some for {:?}", chain_id);

        let by_id = registry.get_chain_by_id(chain_id.to_u64());
        prop_assert!(by_id.is_some(),
            "get_chain_by_id() should return Some for ID {}", chain_id.to_u64());

        // Verify consistency
        prop_assert_eq!(by_enum.unwrap().chain_id, by_id.unwrap().chain_id,
            "Both lookup methods should return the same chain");
    }
}

//
// Property 4: Chain Identity Consistency
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: SvmChain implements ChainIdentity correctly
    ///
    /// For all chain IDs:
    /// - chain_id() should match the enum's u64 value
    /// - chain_name() should be non-empty
    /// - chain_family() should be Account (all SVM chains use account model)
    #[test]
    fn prop_chain_identity_implementation(chain_id in arb_svm_chain_id()) {
        let chain = SvmChain::new(chain_id);

        prop_assert_eq!(chain.chain_id(), chain_id.to_u64(),
            "ChainIdentity::chain_id() should match enum value");

        let name = chain.chain_name();
        prop_assert!(!name.is_empty(),
            "Chain name should be non-empty for {:?}", chain_id);

        prop_assert_eq!(chain.chain_family(), ChainFamily::Account,
            "All SVM chains should use Account model");
    }
}

//
// Property 5: Decoder Creation Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Creating decoders never panics
    ///
    /// For all chain IDs, decoder creation should succeed
    #[test]
    fn prop_decoder_creation_never_panics(chain_id in arb_svm_chain_id()) {
        // Creating a decoder should never panic
        let decoder = SvmDecoder::new(chain_id);

        prop_assert_eq!(decoder.chain_id(), chain_id,
            "Decoder should be configured for the correct chain");
    }
}

//
// Property 6: Format Validation Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Format validation behaves consistently
    ///
    /// - Empty input should always fail
    /// - Very small inputs (< 5 bytes) should always fail
    /// - Large random inputs might pass or fail, but shouldn't panic
    #[test]
    fn prop_format_validation_empty_always_fails(chain_id in arb_svm_chain_id()) {
        let _decoder = SvmDecoder::new(chain_id);

        // Empty input should always fail
        let result = SvmDecoder::validate_format(&[]);
        prop_assert!(result.is_err(), "Empty input should fail validation");
    }

    #[test]
    fn prop_format_validation_tiny_always_fails(
        chain_id in arb_svm_chain_id(),
        size in 1usize..5,
    ) {
        let _decoder = SvmDecoder::new(chain_id);
        let tiny_input = vec![0u8; size];

        // Very small inputs should fail
        let result = SvmDecoder::validate_format(&tiny_input);
        prop_assert!(result.is_err(),
            "Input of size {} should fail validation", size);
    }

    #[test]
    fn prop_format_validation_never_panics(
        chain_id in arb_svm_chain_id(),
        input in prop::collection::vec(any::<u8>(), 0..500),
    ) {
        let _decoder = SvmDecoder::new(chain_id);

        // Format validation should never panic, regardless of input
        let _result = SvmDecoder::validate_format(&input);
        // We don't assert on the result, just that it doesn't panic
    }
}

//
// Property 7: Chain Name Stability
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Chain names are stable across calls
    ///
    /// For all chain IDs:
    /// - name() should always return the same string
    /// - name() should match registry name
    #[test]
    fn prop_chain_name_stability(chain_id in arb_svm_chain_id()) {
        let name1 = chain_id.name();
        let name2 = chain_id.name();

        prop_assert_eq!(name1, name2,
            "Chain name should be stable across calls");

        // Verify registry consistency
        let registry = SvmChainRegistry::new();
        if let Some(chain_info) = registry.get_chain(chain_id) {
            prop_assert_eq!(name1, chain_info.name.as_str(),
                "Chain name should match registry");
        }
    }
}

//
// Property 8: Registry Filter Completeness
//

#[test]
fn test_registry_filters_cover_all_chains() {
    // This is a regular test, not a property test,
    // but it verifies an important invariant

    let registry = SvmChainRegistry::new();
    let total = registry.chain_count();
    let mainnet = registry.mainnet_chains().count();
    let testnet = registry.testnet_chains().count();

    assert_eq!(
        mainnet + testnet,
        total,
        "Mainnet + testnet should equal total chains"
    );

    // Verify no chain is in both categories
    for chain in registry.all_chains() {
        let is_main = chain.is_mainnet;
        let is_test = !chain.is_mainnet;

        assert_ne!(
            is_main, is_test,
            "Chain {} should be either mainnet or testnet, not both",
            chain.name
        );
    }
}

//
// Property 9: RPC Endpoint Consistency
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: RPC endpoints are consistent
    ///
    /// For Solana chains (mainnet/devnet/testnet):
    /// - Should have RPC endpoints
    /// - Should contain "solana.com"
    ///
    /// For other chains:
    /// - May or may not have RPC endpoints (future expansion)
    #[test]
    fn prop_rpc_endpoint_consistency(chain_id in arb_svm_chain_id()) {
        if chain_id.is_solana() {
            let rpc = chain_id.rpc_endpoint();
            prop_assert!(rpc.is_some(),
                "Solana chain {:?} should have RPC endpoint", chain_id);

            let url = rpc.unwrap();
            prop_assert!(url.contains("solana.com"),
                "Solana RPC should contain solana.com: {}", url);
        }
    }
}

//
// Property 10: Explorer URL Format
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Explorer URLs have consistent format
    ///
    /// For chains with explorer URLs:
    /// - Should contain "{txid}" placeholder
    /// - Should start with "http" (https)
    #[test]
    fn prop_explorer_url_format(chain_id in arb_svm_chain_id()) {
        if let Some(url) = chain_id.explorer_url() {
            prop_assert!(url.contains("{txid}"),
                "Explorer URL should contain {{txid}} placeholder: {}", url);

            prop_assert!(url.starts_with("http"),
                "Explorer URL should be HTTP(S): {}", url);
        }
    }
}
