# Vendored Dependencies

This directory contains vendored external data sources for the EVM decoder.

## chainlist/ - Ethereum Chain Registry

**Source**: https://github.com/ethereum-lists/chains
**Upstream Commit**: `1b3d9505216a9fac17a67e4c0c6922f9f4e560ae`
**Vendored Date**: 2025-11-12
**Method**: Git subtree (squashed)

### Verification

To verify the vendored data matches upstream:

```bash
# Clone upstream repository
git clone https://github.com/ethereum-lists/chains.git /tmp/chains
cd /tmp/chains
git checkout 1b3d9505216a9fac17a67e4c0c6922f9f4e560ae

# Compare chain data
diff -r /tmp/chains/_data/chains/ crates/decoder-evm/vendored/chainlist/_data/chains/
```

### What's Included

- `_data/chains/` - 2,397 EVM chain definitions (JSON files)
- `LICENSE` - Upstream license file

### What's Excluded

To minimize repository bloat, we exclude non-essential files:
- CI/CD configurations (`.ci/`, `.github/`)
- Build tools (`gradle/`, `build.gradle`, etc.)
- Development tools (`tools/`, `processor/`)
- Website assets (`website/`)
- Node.js configs (`package.json`, etc.)

These files are not needed for our use case (parsing chain metadata at build time).

### Updates

To update the chain registry:

```bash
# Method 1: Git subtree pull (recommended)
git subtree pull \
    --prefix crates/decoder-evm/vendored/chainlist \
    https://github.com/ethereum-lists/chains.git \
    master \
    --squash

# Then clean up unnecessary files (see below)

# Method 2: Manual update
cd /tmp
git clone https://github.com/ethereum-lists/chains.git
cd chains
NEW_COMMIT=$(git rev-parse HEAD)

# Copy only needed files
rm -rf /path/to/crates/decoder-evm/vendored/chainlist/_data
cp -r _data /path/to/crates/decoder-evm/vendored/chainlist/
cp LICENSE /path/to/crates/decoder-evm/vendored/chainlist/

# Update this file with new commit hash
# Commit with message including $NEW_COMMIT
```

### License

The ethereum-lists/chains data is licensed under MIT License.
See `chainlist/LICENSE` for details.

### Chain Count

As of commit `1b3d9505`:
- **Total chains**: 2,397 EVM-compatible chains
- **File format**: JSON (one file per chain, named `eip155-{chainId}.json`)
- **Total size**: ~12MB (chain data only)

### Chain Schema

Each chain file follows this structure:

```json
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
```

### Build Process

The `build.rs` script:
1. Reads all JSON files from `_data/chains/`
2. Parses chain metadata
3. Generates Rust code with embedded data
4. No runtime file I/O or network calls

This approach ensures:
- ✅ Airgapped operation (works completely offline)
- ✅ Verifiable supply chain (git history + this doc)
- ✅ Reproducible builds (pinned commit hash)
- ✅ No TOCTOU attacks (data cannot change at runtime)
