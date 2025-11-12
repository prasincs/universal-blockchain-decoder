#!/usr/bin/env bash
# Update vendored ethereum-lists/chains registry
#
# This script updates the vendored chain registry to the latest upstream version
# and cleans up unnecessary files to minimize repository size.
#
# Usage:
#   ./update-chains.sh              # Update to latest master
#   ./update-chains.sh <commit>     # Update to specific commit

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VENDOR_DIR="$REPO_ROOT/crates/decoder-evm/vendored/chainlist"
UPSTREAM_REPO="https://github.com/ethereum-lists/chains.git"
UPSTREAM_BRANCH="${1:-master}"

echo "=== Updating vendored chainlist ==="
echo "Repository: $UPSTREAM_REPO"
echo "Target: $UPSTREAM_BRANCH"
echo ""

# Navigate to repo root
cd "$REPO_ROOT"

# Check if we're on the right branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != *"decoder-evm"* ]]; then
    echo "Warning: Not on a decoder-evm branch. Current branch: $CURRENT_BRANCH"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Pull latest from upstream using git subtree
echo "==> Pulling from upstream..."
if [ -d "$VENDOR_DIR" ]; then
    # Existing vendored directory - update it
    git subtree pull \
        --prefix crates/decoder-evm/vendored/chainlist \
        "$UPSTREAM_REPO" \
        "$UPSTREAM_BRANCH" \
        --squash
else
    # First time - add it
    git subtree add \
        --prefix crates/decoder-evm/vendored/chainlist \
        "$UPSTREAM_REPO" \
        "$UPSTREAM_BRANCH" \
        --squash
fi

# Get the upstream commit hash
echo ""
echo "==> Getting upstream commit hash..."
cd "$VENDOR_DIR"
UPSTREAM_COMMIT=$(git log --format="%H" -n 1 | head -c 40)
UPSTREAM_DATE=$(date -u +"%Y-%m-%d")

echo "Upstream commit: $UPSTREAM_COMMIT"
echo "Date: $UPSTREAM_DATE"

# Clean up unnecessary files
echo ""
echo "==> Cleaning up unnecessary files..."

# Files and directories to remove (not needed for our use case)
CLEANUP_ITEMS=(
    ".ci"
    ".github"
    ".gitignore"
    ".gitmodules"
    ".jitpack.yml"
    ".prettierignore"
    ".prettierrc.json"
    "README.md"
    "build.gradle"
    "gradle"
    "gradlew"
    "gradlew.bat"
    "httpsloader"
    "maintainer_checklist.md"
    "model"
    "package.json"
    "package-lock.json"
    "processor"
    "settings.gradle.kts"
    "tools"
    "website"
)

for item in "${CLEANUP_ITEMS[@]}"; do
    if [ -e "$item" ]; then
        echo "  Removing: $item"
        rm -rf "$item"
    fi
done

# Verify essential files are still present
echo ""
echo "==> Verifying essential files..."
if [ ! -d "_data/chains" ]; then
    echo "ERROR: _data/chains directory missing!"
    exit 1
fi

CHAIN_COUNT=$(find _data/chains -name "eip155-*.json" | wc -l)
echo "  Chain files found: $CHAIN_COUNT"

if [ ! -f "LICENSE" ]; then
    echo "ERROR: LICENSE file missing!"
    exit 1
fi

# Update VENDORED.md with new commit hash
echo ""
echo "==> Updating VENDORED.md..."
cd "$REPO_ROOT/crates/decoder-evm/vendored"

cat > VENDORED.md <<EOF
# Vendored Dependencies

This directory contains vendored external data sources for the EVM decoder.

## chainlist/ - Ethereum Chain Registry

**Source**: $UPSTREAM_REPO
**Upstream Commit**: \`$UPSTREAM_COMMIT\`
**Vendored Date**: $UPSTREAM_DATE
**Method**: Git subtree (squashed)
**Chain Count**: $CHAIN_COUNT

### Verification

To verify the vendored data matches upstream:

\`\`\`bash
# Clone upstream repository
git clone $UPSTREAM_REPO /tmp/chains
cd /tmp/chains
git checkout $UPSTREAM_COMMIT

# Compare chain data (should show no differences)
diff -r _data/chains/ $VENDOR_DIR/_data/chains/
\`\`\`

### What's Included

- \`_data/chains/\` - $CHAIN_COUNT EVM chain definitions (JSON files)
- \`_data/icons/\` - Chain icons
- \`LICENSE\` - Upstream MIT license

### What's Excluded

To minimize repository bloat, we exclude non-essential files:
- CI/CD configurations (\`.ci/\`, \`.github/\`)
- Build tools (\`gradle/\`, \`build.gradle\`, etc.)
- Development tools (\`tools/\`, \`processor/\`)
- Website assets (\`website/\`)
- Node.js configs (\`package.json\`, etc.)

These files are not needed for parsing chain metadata at build time.

### Chain Schema

Each chain file (\`eip155-{chainId}.json\`) contains:

\`\`\`json
{
  "name": "Ethereum Mainnet",
  "chain": "ETH",
  "chainId": 1,
  "networkId": 1,
  "shortName": "eth",
  "nativeCurrency": {
    "name": "Ether",
    "symbol": "ETH",
    "decimals": 18
  },
  "rpc": ["https://..."],
  "explorers": [{"name": "etherscan", "url": "https://etherscan.io"}],
  "infoURL": "https://ethereum.org"
}
\`\`\`

### Build Process

The \`build.rs\` script:
1. Reads all JSON files from \`_data/chains/\`
2. Parses chain metadata using serde
3. Generates Rust code with embedded data
4. No runtime file I/O or network calls

This ensures:
- ✅ Airgapped operation (works completely offline)
- ✅ Verifiable supply chain (git history + this doc)
- ✅ Reproducible builds (pinned commit hash)
- ✅ No TOCTOU attacks (data cannot change at runtime)

### License

The ethereum-lists/chains data is licensed under MIT License.
See \`chainlist/LICENSE\` for details.

---

**Last Updated**: $UPSTREAM_DATE
**Script**: \`scripts/decoder-evm/update-chains.sh\`
EOF

echo "  Updated VENDORED.md with commit $UPSTREAM_COMMIT"

# Stage the changes
cd "$REPO_ROOT"
git add crates/decoder-evm/vendored/

# Show status
echo ""
echo "==> Changes staged:"
git status --short crates/decoder-evm/vendored/ | head -20
TOTAL_CHANGES=$(git status --short crates/decoder-evm/vendored/ | wc -l)
if [ $TOTAL_CHANGES -gt 20 ]; then
    echo "... and $((TOTAL_CHANGES - 20)) more files"
fi

# Test that the build still works
echo ""
echo "==> Testing build..."
if ! cargo build -p decoder-evm 2>&1 | grep -q "Finished"; then
    echo "ERROR: Build failed after update!"
    echo "Run: cargo build -p decoder-evm"
    exit 1
fi

echo ""
echo "==> Testing..."
if ! RUST_MIN_STACK=8388608 cargo test -p decoder-evm --lib 2>&1 | grep -q "test result: ok"; then
    echo "ERROR: Tests failed after update!"
    echo "Run: RUST_MIN_STACK=8388608 cargo test -p decoder-evm --lib"
    exit 1
fi

# Generate commit message
echo ""
echo "==> Suggested commit message:"
echo ""
cat <<EOF
Update vendored chainlist to $UPSTREAM_COMMIT

Updates ethereum-lists/chains to latest version.

Upstream: $UPSTREAM_REPO
Commit: $UPSTREAM_COMMIT
Date: $UPSTREAM_DATE
Chain count: $CHAIN_COUNT

Changes:
- Updated chain registry data
- Cleaned up unnecessary build/CI files
- Updated VENDORED.md documentation

Verification:
  git clone $UPSTREAM_REPO /tmp/chains
  cd /tmp/chains && git checkout $UPSTREAM_COMMIT
  diff -r _data/ $VENDOR_DIR/_data/

All tests passing: cargo test -p decoder-evm
EOF

echo ""
echo "=== Update complete! ==="
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff --stat"
echo "  2. Commit: git commit -m 'Update vendored chainlist to $UPSTREAM_COMMIT'"
echo "  3. Push: git push"
echo ""
