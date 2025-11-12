//! Chain Registry Generator
//!
//! Utility to generate compact Borsh-serialized chain registry from
//! ethereum-lists/chains JSON files.
//!
//! Usage:
//!   cargo run -p chain-registry-generator -- \
//!     --input crates/decoder-evm/vendored/chainlist/_data/chains \
//!     --output crates/decoder-evm/data/chains.borsh

use anyhow::{Context, Result};
use borsh::BorshSerialize;
use clap::Parser;
use decoder_evm::types::{ChainInfo, CurrencyInfo, ExplorerInfo};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing chain JSON files
    #[arg(
        short,
        long,
        default_value = "crates/decoder-evm/vendored/chainlist/_data/chains"
    )]
    input: PathBuf,

    /// Output file for Borsh binary
    #[arg(short, long, default_value = "crates/decoder-evm/data/chains.borsh")]
    output: PathBuf,

    /// Output metadata file
    #[arg(
        short,
        long,
        default_value = "crates/decoder-evm/data/chains.metadata.txt"
    )]
    metadata: PathBuf,

    /// Upstream commit hash (optional, read from VENDORED.md if not provided)
    #[arg(long)]
    commit: Option<String>,
}

/// Upstream chain JSON schema
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

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔧 Chain Registry Generator");
    println!("================================");
    println!();

    // Verify input directory exists
    if !args.input.exists() {
        anyhow::bail!("Input directory not found: {}", args.input.display());
    }

    println!("📂 Input: {}", args.input.display());
    println!("💾 Output: {}", args.output.display());
    println!();

    // Parse all chain JSON files
    println!("📖 Parsing chain files...");
    let chains = parse_chain_files(&args.input)?;
    println!("   ✓ Parsed {} chains", chains.len());

    // Create output directory if needed
    if let Some(parent) = args.output.parent() {
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
    fs::write(&args.output, &serialized)
        .with_context(|| format!("Failed to write output file: {}", args.output.display()))?;

    println!("   ✓ Wrote {}", args.output.display());

    // Get or detect upstream commit
    let commit = args
        .commit
        .or_else(|| detect_upstream_commit())
        .unwrap_or_else(|| "unknown".to_string());

    // Write metadata
    println!();
    println!("📄 Writing metadata...");
    write_metadata(&args.metadata, chains.len(), serialized.len(), &commit)?;
    println!("   ✓ Wrote {}", args.metadata.display());

    // Summary
    println!();
    println!("✨ Generation complete!");
    println!();
    println!("Summary:");
    println!("  Chains: {}", chains.len());
    println!(
        "  Size: {} bytes ({:.2} MB)",
        serialized.len(),
        serialized.len() as f64 / 1_048_576.0
    );
    println!("  Commit: {}", commit);
    println!();
    println!("Next steps:");
    println!(
        "  1. Review: ls -lh {}",
        args.output.parent().unwrap().display()
    );
    println!("  2. Test: cargo test -p decoder-evm");
    println!("  3. Commit: git add {}", args.output.display());
    println!();

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

fn detect_upstream_commit() -> Option<String> {
    // Try to read from VENDORED.md
    let vendored_md = "crates/decoder-evm/vendored/VENDORED.md";
    if let Ok(contents) = fs::read_to_string(vendored_md) {
        for line in contents.lines() {
            if line.contains("Upstream Commit") {
                if let Some(commit) = line.split('`').nth(1) {
                    return Some(commit.to_string());
                }
            }
        }
    }
    None
}

fn write_metadata(path: &Path, chain_count: usize, byte_size: usize, commit: &str) -> Result<()> {
    let generated_date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let content = format!(
        r#"Chain Registry Binary Data
=========================

Format: Borsh (Binary Object Representation Serializer for Hashing)
Source: ethereum-lists/chains
Repository: https://github.com/ethereum-lists/chains
Upstream Commit: {commit}
Chain Count: {chain_count}
Generated: {generated_date}

File Size: {size_mb:.2} MB ({byte_size} bytes)

Verification:
1. Clone upstream: git clone https://github.com/ethereum-lists/chains.git /tmp/chains
2. Checkout: cd /tmp/chains && git checkout {commit}
3. Regenerate: cargo run -p chain-registry-generator
4. Compare: diff {output} (should be identical)

This binary file is embedded at compile time via include_bytes!() and
deserialized on first use using borsh::BorshDeserialize.

Benefits:
- ✅ Compact: ~1-2MB vs ~46MB of JSON files (95%+ reduction)
- ✅ Fast builds: No JSON parsing at build time
- ✅ Fast runtime: Deserialize once, cache forever
- ✅ Verifiable: Git commit hash provides full audit trail
"#,
        commit = commit,
        chain_count = chain_count,
        generated_date = generated_date,
        size_mb = byte_size as f64 / 1_048_576.0,
        byte_size = byte_size,
        output = "crates/decoder-evm/data/chains.borsh",
    );

    fs::write(path, content)
        .with_context(|| format!("Failed to write metadata file: {}", path.display()))?;

    Ok(())
}
