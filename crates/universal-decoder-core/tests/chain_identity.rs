//! Tests for chain identity and multi-chain support
//!
//! Verifies that the trait-based chain identity system works correctly
//! and supports multiple chains without core modifications.

use universal_decoder_core::chain::{ChainFamily, ChainIdentity, ChainRef};

#[derive(Debug)]
struct BitcoinMainnet;

impl ChainIdentity for BitcoinMainnet {
    fn chain_id(&self) -> u64 {
        0
    }

    fn chain_name(&self) -> &str {
        "Bitcoin"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet")
    }
}

#[derive(Debug)]
struct EthereumMainnet;

impl ChainIdentity for EthereumMainnet {
    fn chain_id(&self) -> u64 {
        1
    }

    fn chain_name(&self) -> &str {
        "Ethereum"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet")
    }
}

#[derive(Debug)]
struct SolanaMainnet;

impl ChainIdentity for SolanaMainnet {
    fn chain_id(&self) -> u64 {
        501
    }

    fn chain_name(&self) -> &str {
        "Solana"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Instruction
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet-beta")
    }
}

#[test]
fn test_bitcoin_chain_identity() {
    let chain = BitcoinMainnet;
    assert_eq!(chain.chain_id(), 0);
    assert_eq!(chain.chain_name(), "Bitcoin");
    assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    assert_eq!(chain.network(), Some("mainnet"));
}

#[test]
fn test_ethereum_chain_identity() {
    let chain = EthereumMainnet;
    assert_eq!(chain.chain_id(), 1);
    assert_eq!(chain.chain_name(), "Ethereum");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
    assert_eq!(chain.network(), Some("mainnet"));
}

#[test]
fn test_solana_chain_identity() {
    let chain = SolanaMainnet;
    assert_eq!(chain.chain_id(), 501);
    assert_eq!(chain.chain_name(), "Solana");
    assert_eq!(chain.chain_family(), ChainFamily::Instruction);
    assert_eq!(chain.network(), Some("mainnet-beta"));
}

#[test]
fn test_chain_ref_from_identity() {
    let chain = BitcoinMainnet;
    let chain_ref = ChainRef::from(&chain);

    assert_eq!(chain_ref.id, 0);
    assert_eq!(chain_ref.name, "Bitcoin");
    assert_eq!(chain_ref.network, Some("mainnet".to_string()));
}

#[test]
fn test_multiple_chains_different_families() {
    let chains: Vec<Box<dyn ChainIdentity>> = vec![
        Box::new(BitcoinMainnet),
        Box::new(EthereumMainnet),
        Box::new(SolanaMainnet),
    ];

    let families: Vec<ChainFamily> = chains.iter().map(|c| c.chain_family()).collect();

    assert_eq!(families[0], ChainFamily::Utxo);
    assert_eq!(families[1], ChainFamily::Account);
    assert_eq!(families[2], ChainFamily::Instruction);
}

#[test]
fn test_chain_family_equality() {
    assert_eq!(ChainFamily::Utxo, ChainFamily::Utxo);
    assert_ne!(ChainFamily::Utxo, ChainFamily::Account);
    assert_ne!(ChainFamily::Account, ChainFamily::Instruction);
    assert_ne!(ChainFamily::Instruction, ChainFamily::Other);
}

#[test]
fn test_custom_chain_without_network() {
    #[derive(Debug)]
    struct CustomChain;

    impl ChainIdentity for CustomChain {
        fn chain_id(&self) -> u64 {
            9999
        }
        fn chain_name(&self) -> &str {
            "Custom"
        }
        fn chain_family(&self) -> ChainFamily {
            ChainFamily::Other
        }
        // network() uses default implementation (None)
    }

    let chain = CustomChain;
    assert_eq!(chain.network(), None);

    let chain_ref = ChainRef::from(&chain);
    assert_eq!(chain_ref.network, None);
}
