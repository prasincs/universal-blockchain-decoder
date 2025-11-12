//! Cosmos chain registry generator (cosmos/chain-registry)

use anyhow::{Context, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils;

/// Cosmos chain information (minimal subset for decoder use)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct CosmosChainInfo {
    pub chain_name: String,
    pub chain_id: String,
    pub pretty_name: String,
    pub bech32_prefix: String,
    pub slip44: u32,
    pub network_type: String, // "mainnet" | "testnet"
}

/// Upstream chain.json schema (full schema, we only parse what we need)
#[derive(Debug, Deserialize)]
struct CosmosChainJson {
    chain_name: String,
    chain_id: String,
    #[serde(default)]
    pretty_name: Option<String>,
    bech32_prefix: String,
    slip44: u32,
    #[serde(default)]
    network_type: Option<String>,
    // ... other fields we don't need
}

impl From<CosmosChainJson> for CosmosChainInfo {
    fn from(json: CosmosChainJson) -> Self {
        CosmosChainInfo {
            chain_name: json.chain_name.clone(),
            chain_id: json.chain_id,
            pretty_name: json.pretty_name.unwrap_or(json.chain_name),
            bech32_prefix: json.bech32_prefix,
            slip44: json.slip44,
            network_type: json.network_type.unwrap_or_else(|| "mainnet".to_string()),
        }
    }
}

/// Wrapper for serialization
#[derive(BorshSerialize)]
struct CosmosRegistry {
    chains: HashMap<String, CosmosChainInfo>,
}

pub fn generate_cosmos_registry(
    input: PathBuf,
    output: PathBuf,
    metadata: PathBuf,
    commit: Option<String>,
) -> Result<()> {
    println!("🔧 Cosmos Chain Registry Generator");
    println!("================================");
    println!();

    // Verify input directory exists
    if !input.exists() {
        anyhow::bail!("Input directory not found: {}", input.display());
    }

    println!("📂 Input: {}", input.display());
    println!("💾 Output: {}", output.display());
    println!();

    // Parse all chain directories
    println!("📖 Parsing chain directories...");
    let chains = parse_chain_directories(&input)?;
    println!("   ✓ Parsed {} chains", chains.len());

    // Create output directory if needed
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    // Serialize to Borsh
    println!();
    println!("💾 Serializing to Borsh...");
    let registry = CosmosRegistry {
        chains: chains.clone(),
    };
    let serialized =
        borsh::to_vec(&registry).context("Failed to serialize Cosmos registry to Borsh")?;

    println!("   ✓ Serialized {} bytes", serialized.len());

    // Write binary file
    fs::write(&output, &serialized)
        .with_context(|| format!("Failed to write output file: {}", output.display()))?;

    println!("   ✓ Wrote {}", output.display());

    // Get or detect upstream commit
    let commit = commit
        .or_else(|| utils::detect_upstream_commit("crates/decoder-cosmos/vendored/VENDORED.md"))
        .unwrap_or_else(|| "unknown".to_string());

    // Write metadata
    println!();
    println!("📄 Writing metadata...");
    utils::write_metadata(
        &metadata,
        "cosmos/chain-registry",
        "https://github.com/cosmos/chain-registry",
        chains.len(),
        serialized.len(),
        &commit,
        "crates/decoder-cosmos/data/cosmos-chains.borsh",
        &[
            "Clone upstream: git clone https://github.com/cosmos/chain-registry.git /tmp/chain-registry".to_string(),
            format!("Checkout: cd /tmp/chain-registry && git checkout {}", commit),
            "Regenerate: cargo run -p registry-generator -- cosmos".to_string(),
            "Compare: diff crates/decoder-cosmos/data/cosmos-chains.borsh (should be identical)".to_string(),
        ],
    )?;
    println!("   ✓ Wrote {}", metadata.display());

    // Summary
    utils::print_summary(
        "Cosmos Chains",
        chains.len(),
        serialized.len(),
        &commit,
        &output,
        "cargo test -p decoder-cosmos",
    );

    Ok(())
}

fn parse_chain_directories(dir: &Path) -> Result<HashMap<String, CosmosChainInfo>> {
    let mut chains = HashMap::new();
    let mut parse_errors = 0;
    let mut _total_dirs = 0;

    // Read all subdirectories
    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // Skip non-directories and directories starting with _ (like _IBC, _non-cosmos)
        if !path.is_dir() {
            continue;
        }

        if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
            if dir_name.starts_with('_') {
                continue;
            }

            _total_dirs += 1;

            // Look for chain.json in this directory
            let chain_json_path = path.join("chain.json");
            if chain_json_path.exists() {
                match parse_chain_file(&chain_json_path) {
                    Ok(chain) => {
                        // Use chain_id as key (e.g., "cosmoshub-4")
                        chains.insert(chain.chain_id.clone(), chain);
                    }
                    Err(e) => {
                        if parse_errors < 5 {
                            eprintln!("   ⚠️  Failed to parse {}: {}", dir_name, e);
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

fn parse_chain_file(path: &Path) -> Result<CosmosChainInfo> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let chain: CosmosChainJson = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

    Ok(chain.into())
}
