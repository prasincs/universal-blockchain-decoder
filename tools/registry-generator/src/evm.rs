//! EVM chain registry generator (ethereum-lists/chains)

use anyhow::{Context, Result};
use borsh::BorshSerialize;
use decoder_evm::types::{ChainInfo, CurrencyInfo, ExplorerInfo};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::utils;

/// Upstream chain JSON schema (from ethereum-lists/chains)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChainlistChain {
    name: String,
    chain: String,
    rpc: Vec<String>,
    #[serde(default)]
    explorers: Vec<ChainlistExplorer>,
    native_currency: ChainlistCurrency,
    #[serde(rename = "infoURL")]
    info_url: String,
    short_name: String,
    chain_id: u64,
    network_id: u64,
}

#[derive(Debug, Deserialize)]
struct ChainlistCurrency {
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Deserialize)]
struct ChainlistExplorer {
    name: String,
    url: String,
    #[serde(default)]
    standard: Option<String>,
}

/// Wrapper for serialization
#[derive(BorshSerialize)]
struct ChainRegistry {
    chains: HashMap<u64, ChainInfo>,
}

pub fn generate_evm_registry(
    input: PathBuf,
    output: PathBuf,
    metadata: PathBuf,
    commit: Option<String>,
) -> Result<()> {
    println!("🔧 EVM Chain Registry Generator");
    println!("================================");
    println!();

    // Verify input directory exists
    if !input.exists() {
        anyhow::bail!("Input directory not found: {}", input.display());
    }

    println!("📂 Input: {}", input.display());
    println!("💾 Output: {}", output.display());
    println!();

    // Parse all chain JSON files
    println!("📖 Parsing chain files...");
    let chains = parse_chain_files(&input)?;
    println!("   ✓ Parsed {} chains", chains.len());

    // Create output directory if needed
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    // Serialize to Borsh
    println!();
    println!("💾 Serializing to Borsh...");
    let registry = ChainRegistry {
        chains: chains.clone(),
    };
    let serialized =
        borsh::to_vec(&registry).context("Failed to serialize chain registry to Borsh")?;

    println!("   ✓ Serialized {} bytes", serialized.len());

    // Write binary file
    fs::write(&output, &serialized)
        .with_context(|| format!("Failed to write output file: {}", output.display()))?;

    println!("   ✓ Wrote {}", output.display());

    // Get or detect upstream commit
    let commit = commit
        .or_else(|| utils::detect_upstream_commit("crates/decoder-evm/vendored/VENDORED.md"))
        .unwrap_or_else(|| "unknown".to_string());

    // Write metadata
    println!();
    println!("📄 Writing metadata...");
    utils::write_metadata(
        &metadata,
        "ethereum-lists/chains",
        "https://github.com/ethereum-lists/chains",
        chains.len(),
        serialized.len(),
        &commit,
        "crates/decoder-evm/data/chains.borsh",
        &[
            format!("Clone upstream: git clone https://github.com/ethereum-lists/chains.git /tmp/chains"),
            format!("Checkout: cd /tmp/chains && git checkout {}", commit),
            "Regenerate: cargo run -p registry-generator -- evm".to_string(),
            "Compare: diff crates/decoder-evm/data/chains.borsh (should be identical)".to_string(),
        ],
    )?;
    println!("   ✓ Wrote {}", metadata.display());

    // Summary
    utils::print_summary(
        "EVM Chains",
        chains.len(),
        serialized.len(),
        &commit,
        &output,
        "cargo test -p decoder-evm",
    );

    Ok(())
}

fn parse_chain_files(dir: &Path) -> Result<HashMap<u64, ChainInfo>> {
    let mut chains = HashMap::new();
    let mut parse_errors = 0;
    let mut _total_files = 0;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
    {
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if filename.starts_with("eip155-") {
                _total_files += 1;

                match parse_chain_file(path) {
                    Ok(chain) => {
                        chains.insert(chain.chain_id, chain);
                    }
                    Err(e) => {
                        if parse_errors < 5 {
                            eprintln!("   ⚠️  Failed to parse {}: {}", filename, e);
                        }
                        parse_errors += 1;
                    }
                }
            }
        }
    }

    if parse_errors > 0 {
        println!("   ⚠️  {} parse errors (showing first 5)", parse_errors);
    }

    Ok(chains)
}

fn parse_chain_file(path: &Path) -> Result<ChainInfo> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let chain: ChainlistChain = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

    // Detect testnet (heuristic)
    let name_lower = chain.name.to_lowercase();
    let is_testnet = name_lower.contains("test")
        || name_lower.contains("sepolia")
        || name_lower.contains("goerli")
        || name_lower.contains("holesky");

    // Detect special chains
    let has_custom_tx_types = chain.chain_id == 10 // Optimism
        || chain.chain_id == 42161 // Arbitrum
        || chain.chain_id == 324; // zkSync Era

    Ok(ChainInfo {
        chain_id: chain.chain_id,
        name: chain.name,
        short_name: chain.short_name,
        chain: chain.chain,
        network_id: chain.network_id,
        is_testnet,
        has_custom_tx_types,
        native_currency: CurrencyInfo {
            name: chain.native_currency.name,
            symbol: chain.native_currency.symbol,
            decimals: chain.native_currency.decimals,
        },
        info_url: chain.info_url,
        rpc: chain.rpc.into_iter().take(3).collect(),
        explorers: chain
            .explorers
            .into_iter()
            .take(2)
            .map(|e| ExplorerInfo {
                name: e.name,
                url: e.url,
                standard: e.standard.unwrap_or_default(),
            })
            .collect(),
    })
}
