# ETL Datasets: Quick Reference Guide

## TL;DR

The Universal Blockchain Decoder is **airgapped-first** - all data must be vendored via git subtree and embedded at build time. No runtime network calls allowed.

---

## Dataset Viability Matrix

| Dataset | Can Use? | How | ROI | Effort |
|---------|----------|-----|-----|--------|
| **Official test vectors** (Bitcoin Core, EIP specs) | ✅ YES | Git subtree | ★★★★★ | 1-2 days |
| **Blockchain explorer APIs** (Etherscan, etc.) | ⚠️ DEV-ONLY | Fetch script | ★★★★☆ | 3-5 days |
| **Google BigQuery** | ⚠️ OFFLINE | Export + transform | ★★★☆☆ | 1 week |
| **Allium analytics** | ⚠️ OFFLINE | Manual export | ★★☆☆☆ | 1 week |
| **Runtime API calls** | ❌ NO | N/A | ☆☆☆☆☆ | N/A |

---

## Quick Decision Guide

### "I want to add real blockchain transaction data..."

**Step 1: Is it official?** (Bitcoin Core, Ethereum specs, BIPs, EIPs)
→ YES: Use git subtree. Highest value, takes 1 day.

**Step 2: Is it from GitHub?**
→ YES: Use git subtree (if JSON/data files). Otherwise fetch and commit.

**Step 3: Can I fetch it once and commit?**
→ YES: Create a `fetch_fixtures.rs` dev tool. Commit results. Works great.

**Step 4: Do I need real-time data?**
→ YES: Document that it's not possible in production. Use in tests only with dev-dependencies.

### "I want to integrate [specific dataset]..."

1. **Google BigQuery** → One-time export workflow (see Part 1)
2. **Allium** → Manual export of transaction samples
3. **Etherscan** → Dev tool for fixture generation (not production!)
4. **Blockchain.com** → Same as Etherscan
5. **Custom chain data** → Vendor via git subtree if on GitHub

---

## Current Test Fixture Coverage

```
Bitcoin:     47 tests (Genesis, SegWit, Taproot, Multisig)
Ethereum:     6 tests (Legacy, EIP-1559, Contracts)
Solana:      13 tests (Transfers, Instructions, Token)
Cosmos:      31 tests (Multiple message types)
Zcash:       16 tests (Mainnet transactions)
─────────────────────────────────────────────────
TOTAL:     113 transaction tests
TARGET:    500+ by Phase completion
```

---

## Recommended Next Steps (Priority Order)

### Week 1: Official Test Vectors
- [ ] Bitcoin Core test vectors (git subtree)
- [ ] Ethereum JSON tests (git subtree)
- [ ] Cosmos SDK vectors (git subtree)
- Effort: ~3 days
- Payoff: Cover 80% of edge cases

### Week 2: Build Fixture Generator Tool
- [ ] Create `tools/fixture-generator/`
- [ ] Add support for explorer APIs (dev-only)
- [ ] Add validation against reference implementations
- Effort: ~3 days
- Payoff: Automate fixture generation

### Week 3-4: Expand Fixtures
- [ ] Bitcoin: 50+ fixtures
- [ ] Ethereum: 50+ fixtures
- [ ] Solana: 30+ fixtures
- Effort: ~1 week
- Payoff: Reach 500+ fixtures

### Month 2: Automate Maintenance
- [ ] Weekly fixture update CI job
- [ ] Privacy audit
- [ ] Documentation
- Effort: ~1 week
- Payoff: Self-updating test suite

---

## Architecture Patterns to Follow

### Pattern 1: Git Subtree Vendoring
```bash
# Official test vectors
git subtree add \
    --prefix tools/fixture-generator/vendored/bitcoin-core \
    https://github.com/bitcoin/bitcoin.git \
    master --squash
```

### Pattern 2: Build-Time Data Embedding
```rust
// crates/*/build.rs
fn main() {
    // Verify vendored data exists
    if !Path::new("data/transactions.borsh").exists() {
        panic!("Missing test fixtures. Run: cargo run -p fixture-generator");
    }
    println!("cargo:rerun-if-changed=data/transactions.borsh");
}
```

### Pattern 3: Dev-Only Network Tools
```rust
// tools/fetch_fixtures.rs (NOT in production)
#[cfg(feature = "dev-tools")]
mod fetch_fixtures {
    // Requires API keys, only for developers
    // Results committed to git, not runtime-fetched
}
```

---

## Privacy Checklist

Before committing test fixtures:

- [ ] Remove sender wallet addresses (use test vectors)
- [ ] Remove receiver addresses (replace with test accounts)
- [ ] Keep transaction structure/complexity
- [ ] Keep cryptographic validity
- [ ] Document source of transaction

Example:
```json
{
  "description": "SegWit transaction with multiple inputs",
  "from_address": "1A1z7agoat0test0000000000000000000",  // Test vector
  "to_address": "1test00000000000000000000000000000",      // Test vector
  "source": "Bitcoin Core test suite (tx_valid.json)"
}
```

---

## Common Mistakes to Avoid

❌ **DON'T**: Add runtime network calls for data
- Violates airgapped requirement
- Fails in offline deployments
- Security liability

❌ **DON'T**: Fetch data at runtime from APIs
- Even with API keys
- Breaks airgapped deployments
- Rate-limited and unreliable

❌ **DON'T**: Use BigQuery/Allium directly in tests
- Requires credentials
- Too slow for CI
- Doesn't work offline

✅ **DO**: Export once, commit to git
✅ **DO**: Use official test vectors (git subtree)
✅ **DO**: Keep dev tools separate from production code
✅ **DO**: Document all fixture sources

---

## Resources

- **Full Details**: See `docs/ETL_DATASETS_INTEGRATION.md`
- **Current Testing**: See `docs/TESTING_STRATEGY.md`
- **Airgapped Requirements**: See `CLAUDE.md` (Section 8: Supply Chain Security)
- **Vendoring Guide**: See `docs/GIT_SUBTREE_VENDORING.md`

---

## Quick Links

| Resource | Location |
|----------|----------|
| Fixture format spec | `tests/fixtures/TEMPLATE.json` |
| Fixture utilities | `crates/decoder-test-utils/src/fixtures.rs` |
| Test config | `.github/workflows/test.yml` |
| Build verification | `crates/*/build.rs` |

---

**Last Updated**: 2025-11-15
**Status**: Ready for Phase 1 (Official Test Vectors Integration)
**Next Action**: Create `fixture-generator` tool
