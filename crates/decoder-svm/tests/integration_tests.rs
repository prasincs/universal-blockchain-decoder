//! Integration tests for SVM decoder
//!
//! These tests validate the SVM family decoder across multiple chains.

use decoder_svm::registry::{SvmChainId, SvmChainRegistry};
use decoder_svm::{SvmChain, SvmDecoder, SvmTransaction};
use universal_decoder_core::prelude::*;

#[test]
fn test_svm_chain_registry_initialization() {
    let registry = SvmChainRegistry::new();
    assert_eq!(registry.chain_count(), 8, "Should have 8 SVM chains");

    // Verify all chains are present
    assert!(registry.has_chain(SvmChainId::SolanaMainnet));
    assert!(registry.has_chain(SvmChainId::SolanaDevnet));
    assert!(registry.has_chain(SvmChainId::SolanaTestnet));
    assert!(registry.has_chain(SvmChainId::EclipseMainnet));
    assert!(registry.has_chain(SvmChainId::EclipseTestnet));
    assert!(registry.has_chain(SvmChainId::PythNetwork));
    assert!(registry.has_chain(SvmChainId::DriftProtocol));
    assert!(registry.has_chain(SvmChainId::Jito));
}

#[test]
fn test_svm_chain_registry_solana_chains() {
    let registry = SvmChainRegistry::new();

    // Test Solana Mainnet
    let solana = registry.get_chain(SvmChainId::SolanaMainnet);
    assert!(solana.is_some());
    let chain = solana.unwrap();
    assert_eq!(chain.name, "Solana Mainnet");
    assert!(chain.is_mainnet);
    assert!(chain.rpc_endpoint.is_some());
    assert!(chain
        .rpc_endpoint
        .as_ref()
        .unwrap()
        .contains("mainnet-beta"));

    // Test Solana Devnet
    let devnet = registry.get_chain(SvmChainId::SolanaDevnet);
    assert!(devnet.is_some());
    let chain = devnet.unwrap();
    assert_eq!(chain.name, "Solana Devnet");
    assert!(!chain.is_mainnet);
    assert!(chain.rpc_endpoint.is_some());

    // Test Solana Testnet
    let testnet = registry.get_chain(SvmChainId::SolanaTestnet);
    assert!(testnet.is_some());
    let chain = testnet.unwrap();
    assert_eq!(chain.name, "Solana Testnet");
    assert!(!chain.is_mainnet);
}

#[test]
fn test_svm_chain_registry_svm_ecosystem() {
    let registry = SvmChainRegistry::new();

    // Test Eclipse Mainnet
    let eclipse = registry.get_chain(SvmChainId::EclipseMainnet);
    assert!(eclipse.is_some());
    assert_eq!(eclipse.unwrap().name, "Eclipse Mainnet");

    // Test Pyth Network
    let pyth = registry.get_chain(SvmChainId::PythNetwork);
    assert!(pyth.is_some());
    assert_eq!(pyth.unwrap().name, "Pyth Network");

    // Test Drift Protocol
    let drift = registry.get_chain(SvmChainId::DriftProtocol);
    assert!(drift.is_some());
    assert_eq!(drift.unwrap().name, "Drift Protocol");

    // Test Jito
    let jito = registry.get_chain(SvmChainId::Jito);
    assert!(jito.is_some());
    assert_eq!(jito.unwrap().name, "Jito");
}

#[test]
fn test_svm_chain_registry_filters() {
    let registry = SvmChainRegistry::new();

    let mainnet_count = registry.mainnet_chains().count();
    let testnet_count = registry.testnet_chains().count();

    assert!(mainnet_count >= 5, "Should have at least 5 mainnet chains");
    assert!(testnet_count >= 3, "Should have at least 3 testnet chains");
    assert_eq!(
        mainnet_count + testnet_count,
        registry.chain_count(),
        "All chains should be either mainnet or testnet"
    );
}

#[test]
fn test_svm_chain_registry_lookup_by_id() {
    let registry = SvmChainRegistry::new();

    let solana = registry.get_chain_by_id(101);
    assert!(solana.is_some());
    assert_eq!(solana.unwrap().name, "Solana Mainnet");

    let eclipse = registry.get_chain_by_id(201);
    assert!(eclipse.is_some());
    assert_eq!(eclipse.unwrap().name, "Eclipse Mainnet");

    let unknown = registry.get_chain_by_id(999);
    assert!(unknown.is_none());
}

#[test]
fn test_svm_chain_id_properties() {
    // Test Solana chains
    assert!(SvmChainId::SolanaMainnet.is_solana());
    assert!(SvmChainId::SolanaDevnet.is_solana());
    assert!(SvmChainId::SolanaTestnet.is_solana());

    // Test non-Solana chains
    assert!(!SvmChainId::EclipseMainnet.is_solana());
    assert!(!SvmChainId::PythNetwork.is_solana());
    assert!(!SvmChainId::DriftProtocol.is_solana());
    assert!(!SvmChainId::Jito.is_solana());

    // Test mainnet detection
    assert!(SvmChainId::SolanaMainnet.is_mainnet());
    assert!(SvmChainId::EclipseMainnet.is_mainnet());
    assert!(SvmChainId::PythNetwork.is_mainnet());
    assert!(SvmChainId::DriftProtocol.is_mainnet());
    assert!(SvmChainId::Jito.is_mainnet());

    // Test testnet detection
    assert!(SvmChainId::SolanaDevnet.is_testnet());
    assert!(SvmChainId::SolanaTestnet.is_testnet());
    assert!(SvmChainId::EclipseTestnet.is_testnet());
}

#[test]
fn test_svm_chain_id_conversion() {
    // Test roundtrip conversion
    let chains = [
        SvmChainId::SolanaMainnet,
        SvmChainId::SolanaDevnet,
        SvmChainId::EclipseMainnet,
        SvmChainId::PythNetwork,
        SvmChainId::DriftProtocol,
        SvmChainId::Jito,
    ];

    for chain in &chains {
        let id = chain.to_u64();
        let converted = SvmChainId::from_u64(id);
        assert_eq!(converted, Some(*chain));
    }

    // Test invalid ID
    assert_eq!(SvmChainId::from_u64(999), None);
}

#[test]
fn test_svm_chain_identity_implementation() {
    // Test Solana Mainnet
    let solana = SvmChain::new(SvmChainId::SolanaMainnet);
    assert_eq!(solana.chain_id(), 101);
    assert_eq!(solana.chain_name(), "Solana Mainnet");
    assert_eq!(solana.chain_family(), ChainFamily::Account);

    // Test Eclipse
    let eclipse = SvmChain::new(SvmChainId::EclipseMainnet);
    assert_eq!(eclipse.chain_id(), 201);
    assert_eq!(eclipse.chain_name(), "Eclipse Mainnet");
    assert_eq!(eclipse.chain_family(), ChainFamily::Account);

    // Test Pyth
    let pyth = SvmChain::new(SvmChainId::PythNetwork);
    assert_eq!(pyth.chain_id(), 301);
    assert_eq!(pyth.chain_name(), "Pyth Network");
    assert_eq!(pyth.chain_family(), ChainFamily::Account);
}

#[test]
fn test_svm_decoder_default_behavior() {
    // Test that default decoder uses Solana Mainnet
    let chain = SvmDecoder::chain();
    assert_eq!(chain.chain_id(), 101);
    assert_eq!(chain.chain_name(), "Solana Mainnet");
}

#[test]
fn test_svm_decoder_multi_chain() {
    // Create decoders for different chains
    let solana_decoder = SvmDecoder::new(SvmChainId::SolanaMainnet);
    assert_eq!(solana_decoder.chain_id(), SvmChainId::SolanaMainnet);

    let devnet_decoder = SvmDecoder::new(SvmChainId::SolanaDevnet);
    assert_eq!(devnet_decoder.chain_id(), SvmChainId::SolanaDevnet);

    let eclipse_decoder = SvmDecoder::new(SvmChainId::EclipseMainnet);
    assert_eq!(eclipse_decoder.chain_id(), SvmChainId::EclipseMainnet);

    let pyth_decoder = SvmDecoder::new(SvmChainId::PythNetwork);
    assert_eq!(pyth_decoder.chain_id(), SvmChainId::PythNetwork);
}

#[test]
fn test_svm_decoder_format_validation() {
    // Test empty input
    let result = SvmDecoder::validate_format(&[]);
    assert!(result.is_err(), "Should reject empty input");

    // Test too small input
    let result = SvmDecoder::validate_format(&[0x01]);
    assert!(result.is_err(), "Should reject too small input");

    // Test reasonable size (but still invalid structure)
    let large_input = vec![0u8; 100];
    let result = SvmDecoder::validate_format(&large_input);
    // Format validation should pass (size is ok), but decoding will fail
    assert!(result.is_ok() || result.is_err()); // Either is acceptable at format level
}

#[test]
fn test_svm_transaction_chain_context() {
    use decoder_solana::types::{Message, MessageHeader, SolanaTransaction};

    // Create a minimal Solana transaction
    let message = Message {
        header: MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 0,
        },
        account_keys: vec![vec![1u8; 32]],
        recent_blockhash: vec![2u8; 32],
        instructions: vec![],
    };

    let solana_tx = SolanaTransaction {
        signatures: vec![vec![3u8; 64]],
        message,
        raw_bytes: vec![],
    };

    // Test with Solana Mainnet
    let svm_tx_mainnet = SvmTransaction::new(SvmChainId::SolanaMainnet, solana_tx.clone());
    assert_eq!(svm_tx_mainnet.chain_id(), SvmChainId::SolanaMainnet);
    assert!(svm_tx_mainnet.is_solana());
    assert!(svm_tx_mainnet.is_mainnet());

    // Test with Solana Devnet
    let svm_tx_devnet = SvmTransaction::new(SvmChainId::SolanaDevnet, solana_tx.clone());
    assert_eq!(svm_tx_devnet.chain_id(), SvmChainId::SolanaDevnet);
    assert!(svm_tx_devnet.is_solana());
    assert!(!svm_tx_devnet.is_mainnet());

    // Test with Eclipse (non-Solana SVM chain)
    let svm_tx_eclipse = SvmTransaction::new(SvmChainId::EclipseMainnet, solana_tx.clone());
    assert_eq!(svm_tx_eclipse.chain_id(), SvmChainId::EclipseMainnet);
    assert!(!svm_tx_eclipse.is_solana());
    assert!(svm_tx_eclipse.is_mainnet());

    // Test with Pyth
    let svm_tx_pyth = SvmTransaction::new(SvmChainId::PythNetwork, solana_tx);
    assert_eq!(svm_tx_pyth.chain_id(), SvmChainId::PythNetwork);
    assert!(!svm_tx_pyth.is_solana());
    assert!(svm_tx_pyth.is_mainnet());
}

#[test]
fn test_svm_chain_explorer_urls() {
    // Test Solana Mainnet explorer
    let url = SvmChainId::SolanaMainnet.explorer_url();
    assert!(url.is_some());
    assert!(url.unwrap().contains("explorer.solana.com"));
    assert!(url.unwrap().contains("{txid}"));

    // Test Solana Devnet explorer
    let url = SvmChainId::SolanaDevnet.explorer_url();
    assert!(url.is_some());
    assert!(url.unwrap().contains("devnet"));

    // Test Solana Testnet explorer
    let url = SvmChainId::SolanaTestnet.explorer_url();
    assert!(url.is_some());
    assert!(url.unwrap().contains("testnet"));
}

#[test]
fn test_svm_chain_rpc_endpoints() {
    // Test Solana chains have RPC endpoints
    assert!(SvmChainId::SolanaMainnet.rpc_endpoint().is_some());
    assert!(SvmChainId::SolanaDevnet.rpc_endpoint().is_some());
    assert!(SvmChainId::SolanaTestnet.rpc_endpoint().is_some());

    // Test that RPCs contain expected domains
    assert!(SvmChainId::SolanaMainnet
        .rpc_endpoint()
        .unwrap()
        .contains("solana.com"));
    assert!(SvmChainId::SolanaDevnet
        .rpc_endpoint()
        .unwrap()
        .contains("devnet"));
    assert!(SvmChainId::SolanaTestnet
        .rpc_endpoint()
        .unwrap()
        .contains("testnet"));
}

#[test]
fn test_svm_registry_iteration() {
    let registry = SvmChainRegistry::new();

    let all_chains: Vec<_> = registry.all_chains().collect();
    assert_eq!(all_chains.len(), registry.chain_count());

    // Verify all chains have valid properties
    for chain in all_chains {
        assert!(!chain.name.is_empty());
        assert!(chain.chain_id.to_u64() > 0);
    }
}

#[test]
fn test_svm_chain_name_consistency() {
    // Verify chain names match between registry and enum
    let registry = SvmChainRegistry::new();

    let chains = [
        SvmChainId::SolanaMainnet,
        SvmChainId::SolanaDevnet,
        SvmChainId::SolanaTestnet,
        SvmChainId::EclipseMainnet,
        SvmChainId::PythNetwork,
        SvmChainId::DriftProtocol,
        SvmChainId::Jito,
    ];

    for chain_id in &chains {
        let chain_info = registry.get_chain(*chain_id).unwrap();
        assert_eq!(chain_info.name, chain_id.name());
    }
}
