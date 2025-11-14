//! Example: Decoding SVM transactions across different chains
//!
//! This example demonstrates how to use the SVM decoder to decode
//! transactions from different SVM-based chains (Solana, Eclipse, etc.)
//!
//! Run with: cargo run --example decode_svm_transaction

use decoder_svm::registry::{SvmChainId, SvmChainRegistry};
use decoder_svm::SvmDecoder;

fn main() {
    println!("🔗 SVM (Solana Virtual Machine) Ecosystem Decoder");
    println!("{}", "=".repeat(60));
    println!();

    // Show all supported SVM chains
    show_supported_chains();
    println!();

    // Demonstrate multi-chain decoding
    demonstrate_chain_selection();
    println!();

    // Show chain properties
    show_chain_properties();
}

fn show_supported_chains() {
    println!("📋 Supported SVM Chains:");
    println!("{}", "-".repeat(60));

    let registry = SvmChainRegistry::new();

    for chain in registry.all_chains() {
        let chain_type = if chain.is_mainnet {
            "Mainnet"
        } else {
            "Testnet"
        };

        println!(
            "  • {} (ID: {}) - {}",
            chain.name,
            chain.chain_id.to_u64(),
            chain_type
        );

        if let Some(ref rpc) = chain.rpc_endpoint {
            println!("    RPC: {}", rpc);
        }

        if let Some(ref explorer) = chain.explorer_url {
            println!("    Explorer: {}", explorer);
        }
    }

    println!("\nTotal: {} chains", registry.chain_count());
}

fn demonstrate_chain_selection() {
    println!("🌐 Chain Selection Examples:");
    println!("{}", "-".repeat(60));

    // Example 1: Decode for Solana Mainnet (default)
    println!("\n1️⃣  Solana Mainnet (default):");
    let solana_decoder = SvmDecoder::new(SvmChainId::SolanaMainnet);
    println!(
        "   Decoder configured for: {}",
        solana_decoder.chain_id().name()
    );

    // Example 2: Decode for Solana Devnet
    println!("\n2️⃣  Solana Devnet:");
    let devnet_decoder = SvmDecoder::new(SvmChainId::SolanaDevnet);
    println!(
        "   Decoder configured for: {}",
        devnet_decoder.chain_id().name()
    );

    // Example 3: Decode for Eclipse Mainnet
    println!("\n3️⃣  Eclipse Mainnet (Ethereum-Solana hybrid):");
    let eclipse_decoder = SvmDecoder::new(SvmChainId::EclipseMainnet);
    println!(
        "   Decoder configured for: {}",
        eclipse_decoder.chain_id().name()
    );

    // Example 4: Decode for Pyth Network
    println!("\n4️⃣  Pyth Network (Oracle network):");
    let pyth_decoder = SvmDecoder::new(SvmChainId::PythNetwork);
    println!(
        "   Decoder configured for: {}",
        pyth_decoder.chain_id().name()
    );
}

fn show_chain_properties() {
    println!("🔍 Chain Properties:");
    println!("{}", "-".repeat(60));

    let chains = [
        SvmChainId::SolanaMainnet,
        SvmChainId::SolanaDevnet,
        SvmChainId::EclipseMainnet,
        SvmChainId::PythNetwork,
        SvmChainId::DriftProtocol,
        SvmChainId::Jito,
    ];

    for chain_id in &chains {
        println!("\n  {}", chain_id.name());
        println!("  {}", "─".repeat(chain_id.name().len()));
        println!("  Chain ID: {}", chain_id.to_u64());
        println!("  Is Solana: {}", chain_id.is_solana());
        println!("  Is Mainnet: {}", chain_id.is_mainnet());

        if let Some(rpc) = chain_id.rpc_endpoint() {
            println!("  RPC: {}", rpc);
        }

        if let Some(explorer) = chain_id.explorer_url() {
            println!("  Explorer: {}", explorer);
        }
    }
}
