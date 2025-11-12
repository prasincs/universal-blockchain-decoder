#!/usr/bin/env bash
# Update Bitcoin Core test vectors from upstream
#
# This script fetches the latest test vectors from Bitcoin Core's master branch
# and updates our local fixtures. This ensures we're always testing against
# the most current Bitcoin Core test suite.
#
# Usage:
#   ./scripts/update-bitcoin-test-vectors.sh
#
# Or run in CI to validate against latest:
#   ./scripts/update-bitcoin-test-vectors.sh && cargo test -p decoder-bitcoin

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FIXTURES_DIR="$PROJECT_ROOT/crates/decoder-bitcoin/tests/fixtures/bitcoin-core"

# Bitcoin Core repository
BITCOIN_REPO="https://raw.githubusercontent.com/bitcoin/bitcoin"
BITCOIN_BRANCH="${BITCOIN_BRANCH:-master}"  # Can override with env var

echo "=== Updating Bitcoin Core Test Vectors ==="
echo "Source: $BITCOIN_REPO/$BITCOIN_BRANCH"
echo "Target: $FIXTURES_DIR"
echo

# Create fixtures directory if it doesn't exist
mkdir -p "$FIXTURES_DIR"

# Download tx_valid.json
echo "Downloading tx_valid.json..."
curl -fsSL "$BITCOIN_REPO/$BITCOIN_BRANCH/src/test/data/tx_valid.json" \
    -o "$FIXTURES_DIR/tx_valid.json"

# Download tx_invalid.json
echo "Downloading tx_invalid.json..."
curl -fsSL "$BITCOIN_REPO/$BITCOIN_BRANCH/src/test/data/tx_invalid.json" \
    -o "$FIXTURES_DIR/tx_invalid.json"

# Count test cases
valid_count=$(jq 'length' "$FIXTURES_DIR/tx_valid.json" 2>/dev/null || echo "unknown")
invalid_count=$(jq 'length' "$FIXTURES_DIR/tx_invalid.json" 2>/dev/null || echo "unknown")

echo
echo "=== Download Complete ==="
echo "tx_valid.json:   $valid_count test cases"
echo "tx_invalid.json: $invalid_count test cases"
echo
echo "Run tests with:"
echo "  cargo test -p decoder-bitcoin --test bitcoin_core_vectors"
