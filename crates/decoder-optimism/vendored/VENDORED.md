# Vendored Dependencies

This directory contains vendored external data sources for the OP Stack decoder.

## superchain-registry/ - Optimism Superchain Registry

**Source**: https://github.com/ethereum-optimism/superchain-registry
**Upstream Commit**: `main` (vendored via git subtree)
**Vendored Date**: 2025-11-12
**Method**: Git subtree (squashed)

### Verification

To verify the vendored data matches upstream:

```bash
# Clone upstream repository
git clone https://github.com/ethereum-optimism/superchain-registry.git /tmp/superchain-registry
cd /tmp/superchain-registry

# Compare
diff -r /tmp/superchain-registry/ crates/decoder-optimism/vendored/superchain-registry/
```

### What's Included

- **35+ OP Stack chain definitions** - Chains built on Optimism's OP Stack:
  - Optimism Mainnet
  - Base
  - Zora
  - Mode
  - Public Goods Network
  - Orderly
  - And more...
- `chainList.json` - Complete list of all superchain networks
- `chainList.toml` - TOML format registry
- `superchain/` - Per-chain configuration files
- `validation/` - Schema validation rules

### What's Excluded

None currently. Full registry is vendored (7.1MB).

**Future optimization**: After creating unified `registry-generator` tool, transform to Borsh binary format (~200KB) and remove raw JSON.

### Updates

To update the superchain registry:

```bash
# Git subtree pull (recommended)
git subtree pull \
    --prefix crates/decoder-optimism/vendored/superchain-registry \
    https://github.com/ethereum-optimism/superchain-registry.git \
    main \
    --squash
```

### License

The ethereum-optimism/superchain-registry data is licensed under MIT License.
See `superchain-registry/LICENSE` for details.

### Chain Count

As of 2025-11-12:
- **Total OP Stack chains**: 35+
- **Total size**: 7.1MB (JSON + config files)

### Chain Schema

Each chain in the superchain follows OP Stack specifications with:
- L1 contract addresses (OptimismPortal, L1CrossDomainMessenger, etc.)
- L2 genesis configuration
- Deposit transaction support (0x7E type)
- Sequencer and batch inbox addresses

### Build Process (Future)

**Planned**: Unified `registry-generator` tool with subcommands:
```bash
cargo run -p registry-generator -- superchain \
    --input crates/decoder-optimism/vendored/superchain-registry \
    --output crates/decoder-optimism/data/op-chains.borsh
```

This approach ensures:
- ✅ Airgapped operation (works completely offline)
- ✅ Verifiable supply chain (git history + this doc)
- ✅ Reproducible builds (pinned commit)
- ✅ Compact size (~200KB Borsh vs 7.1MB JSON)

### Related Documentation

- `../../../docs/GIT_SUBTREE_VENDORING.md` - Git subtree vendoring strategy
- `../../../ROADMAP.md` - Phase 1.5.1: Chain registry vendoring
- `../../../CLAUDE.md` - Airgapped operation requirements
