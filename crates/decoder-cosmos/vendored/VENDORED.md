# Vendored Dependencies

This directory contains vendored external data sources for the Cosmos SDK decoder.

## chain-registry/ - Cosmos Chain Registry

**Source**: https://github.com/cosmos/chain-registry
**Upstream Commit**: `2a630faa` (vendored via git subtree)
**Vendored Date**: 2025-11-12
**Method**: Git subtree (squashed)

### Verification

To verify the vendored data matches upstream:

```bash
# Clone upstream repository
git clone https://github.com/cosmos/chain-registry.git /tmp/chain-registry
cd /tmp/chain-registry
git checkout 2a630faa

# Compare chain data
diff -r /tmp/chain-registry/ crates/decoder-cosmos/vendored/chain-registry/
```

### What's Included

- **406 Cosmos SDK chain definitions** - Each directory contains:
  - `chain.json` - Chain metadata (RPC, network info, genesis)
  - `assetlist.json` - Native and IBC token information
  - `versions.json` - Software versions and upgrade history
  - `images/` - Chain and token logos (PNG/SVG)
- `_IBC/` - IBC connection data between chains
- `_non-cosmos/` - Reference data for non-Cosmos chains (Bitcoin, Ethereum, etc.)
- `testnets/` - Testnet chain definitions

### What's Excluded

To minimize repository size, we exclude non-essential files:
- **Images removed**: All `/images/` directories (~66MB of PNG/SVG files)
- **CI/CD removed**: `.github/` workflows and configuration
- **Build scripts removed**: `_scripts/` Python utilities
- **Templates removed**: `_template/` example files
- **Testnets removed**: `testnets/` directory (169 test networks)

**Size reduction**: 74MB → 7.4MB (90% reduction!)

These files are not needed for transaction decoding. Images and testnets can be accessed from upstream if needed.

### Updates

To update the chain registry:

```bash
# Method 1: Git subtree pull (recommended)
git subtree pull \
    --prefix crates/decoder-cosmos/vendored/chain-registry \
    https://github.com/cosmos/chain-registry.git \
    master \
    --squash

# Method 2: Manual update with specific commit
cd /tmp
git clone https://github.com/cosmos/chain-registry.git
cd chain-registry
NEW_COMMIT=$(git rev-parse HEAD)

# Copy files
rsync -av --delete \
    /tmp/chain-registry/ \
    /path/to/crates/decoder-cosmos/vendored/chain-registry/

# Update this file with new commit hash
# Commit with message including $NEW_COMMIT
```

### License

The cosmos/chain-registry data is licensed under MIT License.
See `chain-registry/LICENSE` for details.

### Chain Count

As of commit `2a630faa`:
- **Mainnet chains**: 406 Cosmos SDK chains
- **Testnets**: 169 test networks
- **IBC connections**: 3,600+ channel mappings in `_IBC/`
- **Total size**: 74MB (raw JSON + images)

### Chain Schema

Each chain directory follows this structure:

```
cosmoshub/
├── chain.json          # Network metadata
├── assetlist.json      # Token information
├── versions.json       # Software versions
└── images/
    ├── atom.png
    └── atom.svg
```

Example `chain.json`:
```json
{
  "chain_name": "cosmoshub",
  "chain_id": "cosmoshub-4",
  "bech32_prefix": "cosmos",
  "daemon_name": "gaiad",
  "genesis": {
    "genesis_url": "https://..."
  },
  "apis": {
    "rpc": [{"address": "https://rpc.cosmos.network"}]
  }
}
```

### Build Process (Future)

**Planned**: Create `cosmos-registry-generator` tool (similar to `chain-registry-generator` for EVM):
1. Parse all `chain.json` and `assetlist.json` files
2. Extract essential metadata (chain_id, bech32_prefix, RPCs)
3. Serialize to Borsh binary format → `data/cosmos-chains.borsh`
4. No runtime file I/O or network calls

This approach ensures:
- ✅ Airgapped operation (works completely offline)
- ✅ Verifiable supply chain (git history + this doc)
- ✅ Reproducible builds (pinned commit hash)
- ✅ No TOCTOU attacks (data cannot change at runtime)
- ✅ Compact size (~5MB Borsh vs 74MB raw JSON)

### Related Documentation

- `../../../docs/GIT_SUBTREE_VENDORING.md` - Git subtree vendoring strategy
- `../../../ROADMAP.md` - Phase 1.5.1: Chain registry vendoring
- `../../../CLAUDE.md` - Airgapped operation requirements
