//! Superchain registry generator (ethereum-optimism/superchain-registry)

use anyhow::{Context, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::utils;

/// OP Stack chain information (minimal subset for decoder use)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SuperchainInfo {
    pub name: String,
    pub chain_id: u64,
    pub rpc: Vec<String>,
    pub explorers: Vec<String>,
    pub superchain_level: u8,
    pub data_availability_type: String, // "eth-da" | "alt-da"
}

/// Upstream chainList.json schema (full schema, we only parse what we need)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuperchainJson {
    name: String,
    chain_id: u64,
    rpc: Vec<String>,
    explorers: Vec<String>,
    superchain_level: u8,
    data_availability_type: String,
    // ... other fields we don't need (parent, gasPayingToken, faultProofs, etc.)
}

impl From<SuperchainJson> for SuperchainInfo {
    fn from(json: SuperchainJson) -> Self {
        SuperchainInfo {
            name: json.name,
            chain_id: json.chain_id,
            rpc: json.rpc,
            explorers: json.explorers,
            superchain_level: json.superchain_level,
            data_availability_type: json.data_availability_type,
        }
    }
}

/// Wrapper for serialization
#[derive(BorshSerialize)]
struct SuperchainRegistry {
    chains: Vec<SuperchainInfo>,
}

pub fn generate_superchain_registry(
    input: PathBuf,
    output: PathBuf,
    metadata: PathBuf,
    commit: Option<String>,
) -> Result<()> {
    println!("🔧 Superchain Registry Generator");
    println!("================================");
    println!();

    // Verify input file exists
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    println!("📂 Input: {}", input.display());
    println!("💾 Output: {}", output.display());
    println!();

    // Parse chainList.json
    println!("📖 Parsing chainList.json...");
    let chains = parse_chain_list(&input)?;
    println!("   ✓ Parsed {} chains", chains.len());

    // Create output directory if needed
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    // Serialize to Borsh
    println!();
    println!("💾 Serializing to Borsh...");
    let registry = SuperchainRegistry {
        chains: chains.clone(),
    };
    let serialized =
        borsh::to_vec(&registry).context("Failed to serialize Superchain registry to Borsh")?;

    println!("   ✓ Serialized {} bytes", serialized.len());

    // Write binary file
    fs::write(&output, &serialized)
        .with_context(|| format!("Failed to write output file: {}", output.display()))?;

    println!("   ✓ Wrote {}", output.display());

    // Get or detect upstream commit
    let commit = commit
        .or_else(|| utils::detect_upstream_commit("crates/decoder-optimism/vendored/VENDORED.md"))
        .unwrap_or_else(|| "unknown".to_string());

    // Write metadata
    println!();
    println!("📄 Writing metadata...");
    utils::write_metadata(
        &metadata,
        "ethereum-optimism/superchain-registry",
        "https://github.com/ethereum-optimism/superchain-registry",
        chains.len(),
        serialized.len(),
        &commit,
        "crates/decoder-optimism/data/op-chains.borsh",
        &[
            "Clone upstream: git clone https://github.com/ethereum-optimism/superchain-registry.git /tmp/superchain-registry".to_string(),
            format!("Checkout: cd /tmp/superchain-registry && git checkout {}", commit),
            "Regenerate: cargo run -p registry-generator -- superchain".to_string(),
            "Compare: diff crates/decoder-optimism/data/op-chains.borsh (should be identical)".to_string(),
        ],
    )?;
    println!("   ✓ Wrote {}", metadata.display());

    // Summary
    utils::print_summary(
        "OP Stack Chains",
        chains.len(),
        serialized.len(),
        &commit,
        &output,
        "cargo test -p decoder-optimism",
    );

    Ok(())
}

fn parse_chain_list(path: &PathBuf) -> Result<Vec<SuperchainInfo>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let chains: Vec<SuperchainJson> = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

    Ok(chains.into_iter().map(|c| c.into()).collect())
}
