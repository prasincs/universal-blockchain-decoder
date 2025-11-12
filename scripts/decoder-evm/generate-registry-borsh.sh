#!/usr/bin/env bash
# Generate compact Borsh-serialized chain registry
#
# This generates a binary file instead of Rust code, dramatically reducing size.
# The binary is embedded via include_bytes! and deserialized on first use.
#
# Usage:
#   ./generate-registry-borsh.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VENDOR_DIR="$REPO_ROOT/crates/decoder-evm/vendored/chainlist"
OUTPUT_DIR="$REPO_ROOT/crates/decoder-evm/data"
BINARY_FILE="$OUTPUT_DIR/chains.borsh"
METADATA_FILE="$OUTPUT_DIR/chains.metadata.txt"

echo "=== Generating Borsh-serialized chain registry ==="

if [ ! -d "$VENDOR_DIR/_data/chains" ]; then
    echo "ERROR: Vendored chain data not found at $VENDOR_DIR/_data/chains"
    echo "Run ./update-chains.sh first"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Create a small Rust program to serialize the data
cd "$REPO_ROOT"

cat > /tmp/serialize_chains.rs <<'EOF'
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, borsh::BorshSerialize, borsh::BorshDeserialize)]
struct ChainRegistry {
    chains: HashMap<u64, ChainInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, borsh::BorshSerialize, borsh::BorshDeserialize)]
struct ChainInfo {
    chain_id: u64,
    name: String,
    short_name: String,
    chain: String,
    network_id: u64,
    is_testnet: bool,
    has_custom_tx_types: bool,
    native_currency: CurrencyInfo,
    info_url: String,
    rpc: Vec<String>,
    explorers: Vec<ExplorerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, borsh::BorshSerialize, borsh::BorshDeserialize)]
struct CurrencyInfo {
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, borsh::BorshSerialize, borsh::BorshDeserialize)]
struct ExplorerInfo {
    name: String,
    url: String,
    standard: String,
}

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let chains_dir = Path::new(&args[1]);
    let output_file = Path::new(&args[2]);

    let mut chains: HashMap<u64, ChainInfo> = HashMap::new();

    // Parse all chain files
    for entry in fs::read_dir(chains_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with("eip155-") {
                    if let Ok(contents) = fs::read_to_string(&path) {
                        if let Ok(chain) = serde_json::from_str::<ChainlistChain>(&contents) {
                            let name = chain.name.clone();
                            let is_testnet = name.to_lowercase().contains("test") ||
                                           name.to_lowercase().contains("sepolia") ||
                                           name.to_lowercase().contains("goerli") ||
                                           name.to_lowercase().contains("holesky");

                            let has_custom_tx_types = chain.chain_id == 10 ||
                                                     chain.chain_id == 42161 ||
                                                     chain.chain_id == 324;

                            chains.insert(chain.chain_id, ChainInfo {
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
                                explorers: chain.explorers.into_iter().take(2).map(|e| ExplorerInfo {
                                    name: e.name,
                                    url: e.url,
                                    standard: e.standard.unwrap_or_default(),
                                }).collect(),
                            });
                        }
                    }
                }
            }
        }
    }

    let registry = ChainRegistry { chains };

    // Serialize to Borsh
    let serialized = borsh::to_vec(&registry).unwrap();
    fs::write(output_file, &serialized).unwrap();

    println!("Serialized {} chains to {} bytes", registry.chains.len(), serialized.len());
}
EOF

# Compile and run the serializer
echo "==> Compiling serializer..."
rustc --edition 2021 \
    --extern serde=/tmp/libserde.rlib \
    --extern serde_json=/tmp/libserde_json.rlib \
    --extern borsh=/tmp/libborsh.rlib \
    /tmp/serialize_chains.rs -o /tmp/serialize_chains 2>/dev/null || {

    # Fallback: use cargo script
    echo "==> Using cargo to compile..."
    cat > /tmp/Cargo.toml <<EOF
[package]
name = "serialize_chains"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
borsh = { version = "1.3", features = ["derive"] }
EOF

    mkdir -p /tmp/serialize_chains/src
    mv /tmp/serialize_chains.rs /tmp/serialize_chains/src/main.rs
    mv /tmp/Cargo.toml /tmp/serialize_chains/

    cd /tmp/serialize_chains
    cargo build --release 2>&1 | grep -E "(Compiling|Finished)"
    SERIALIZER="/tmp/serialize_chains/target/release/serialize_chains"
}

# Run the serializer
echo "==> Serializing chains..."
if [ -z "${SERIALIZER:-}" ]; then
    SERIALIZER="/tmp/serialize_chains"
fi

$SERIALIZER "$VENDOR_DIR/_data/chains" "$BINARY_FILE"

# Get metadata
CHAIN_COUNT=$(find "$VENDOR_DIR/_data/chains" -name "eip155-*.json" | wc -l)
UPSTREAM_COMMIT=$(grep "Upstream Commit" "$REPO_ROOT/crates/decoder-evm/vendored/VENDORED.md" | cut -d'`' -f2 || echo "unknown")
GENERATED_DATE=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
FILE_SIZE=$(du -h "$BINARY_FILE" | cut -f1)
BYTE_SIZE=$(stat -f%z "$BINARY_FILE" 2>/dev/null || stat -c%s "$BINARY_FILE")

# Write metadata
cat > "$METADATA_FILE" <<EOF
Chain Registry Binary Data
=========================

Format: Borsh (Binary Object Representation Serializer for Hashing)
Source: ethereum-lists/chains
Upstream Commit: $UPSTREAM_COMMIT
Chain Count: $CHAIN_COUNT
Generated: $GENERATED_DATE

File Size: $FILE_SIZE ($BYTE_SIZE bytes)

Verification:
1. Clone upstream: git clone https://github.com/ethereum-lists/chains.git
2. Checkout: git checkout $UPSTREAM_COMMIT
3. Regenerate: ./scripts/decoder-evm/generate-registry-borsh.sh
4. Compare: diff $BINARY_FILE (should be identical)

This binary file is embedded at compile time and deserialized on first use.
EOF

echo ""
echo "==> Generated files:"
echo "  Binary: $BINARY_FILE ($FILE_SIZE, $BYTE_SIZE bytes)"
echo "  Metadata: $METADATA_FILE"
echo "  Chains: $CHAIN_COUNT"

# Compare sizes
JSON_SIZE=$(du -sh "$VENDOR_DIR/_data/chains" | cut -f1)
echo ""
echo "==> Size comparison:"
echo "  JSON files: $JSON_SIZE (~46MB)"
echo "  Borsh binary: $FILE_SIZE"
echo "  Reduction: ~95%+"

echo ""
echo "=== Generation complete! ==="
echo ""
echo "Next steps:"
echo "  1. Review: ls -lh $OUTPUT_DIR"
echo "  2. Update src/registry.rs to use embedded binary"
echo "  3. Test: cargo test -p decoder-evm"
echo "  4. Commit: git add crates/decoder-evm/data/"
echo ""
