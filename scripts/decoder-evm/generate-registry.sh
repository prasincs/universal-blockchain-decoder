#!/usr/bin/env bash
# Generate chain registry Rust code from vendored JSON files
#
# This script generates the chain registry code once and checks it in,
# eliminating the need to parse 2,397 JSON files at every build.
#
# Usage:
#   ./generate-registry.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VENDOR_DIR="$REPO_ROOT/crates/decoder-evm/vendored/chainlist"
OUTPUT_FILE="$REPO_ROOT/crates/decoder-evm/src/generated_registry.rs"

echo "=== Generating chain registry Rust code ==="

if [ ! -d "$VENDOR_DIR/_data/chains" ]; then
    echo "ERROR: Vendored chain data not found at $VENDOR_DIR/_data/chains"
    echo "Run ./update-chains.sh first"
    exit 1
fi

# Build the generator (uses build.rs logic)
cd "$REPO_ROOT"
echo "==> Building decoder-evm..."
cargo build -p decoder-evm 2>&1 | grep -E "(Compiling|Finished|warning.*Parsed)"

# Find the generated registry file
GENERATED_FILE=$(find target/debug/build/decoder-evm-*/out -name "chain_registry.rs" 2>/dev/null | head -1)

if [ -z "$GENERATED_FILE" ]; then
    echo "ERROR: Generated registry file not found"
    exit 1
fi

echo "==> Found generated file: $GENERATED_FILE"

# Get metadata
CHAIN_COUNT=$(find "$VENDOR_DIR/_data/chains" -name "eip155-*.json" | wc -l)
UPSTREAM_COMMIT=$(grep "Upstream Commit" "$REPO_ROOT/crates/decoder-evm/vendored/VENDORED.md" | cut -d'`' -f2 || echo "unknown")
GENERATED_DATE=$(date -u +"%Y-%m-%d %H:%M:%S UTC")

# Copy with header
cat > "$OUTPUT_FILE" <<EOF
// Auto-generated chain registry
// DO NOT EDIT MANUALLY - regenerate with scripts/decoder-evm/generate-registry.sh
//
// Source: ethereum-lists/chains
// Upstream Commit: $UPSTREAM_COMMIT
// Chain Count: $CHAIN_COUNT
// Generated: $GENERATED_DATE

EOF

cat "$GENERATED_FILE" >> "$OUTPUT_FILE"

# Get file size
FILE_SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)
LINE_COUNT=$(wc -l < "$OUTPUT_FILE")

echo ""
echo "==> Generated registry:"
echo "  File: $OUTPUT_FILE"
echo "  Size: $FILE_SIZE"
echo "  Lines: $LINE_COUNT"
echo "  Chains: $CHAIN_COUNT"

# Format the code
echo ""
echo "==> Formatting..."
cargo fmt --package decoder-evm

echo ""
echo "=== Generation complete! ==="
echo ""
echo "Next steps:"
echo "  1. Review: head -100 $OUTPUT_FILE"
echo "  2. Test: cargo test -p decoder-evm"
echo "  3. Commit: git add crates/decoder-evm/src/generated_registry.rs"
echo ""
