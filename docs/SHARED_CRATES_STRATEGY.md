# Shared Crates Strategy - Quick Reference

## Current Architecture ✅

```
┌─────────────────────────────────────────┐
│     Universal Decoder Core              │
│  (traits, types, canonical encoding)    │
│  Dependencies: serde, borsh, thiserror  │
└────────────────┬────────────────────────┘
                 │
                 ├─────────────────────────┐
                 │                         │
    ┌────────────▼──────────┐   ┌─────────▼──────────┐
    │  decoder-primitives   │   │   decoder-bitcoin  │
    │  - Byte readers       │   │   decoder-ethereum │
    │  - Little/big endian  │   │   decoder-solana   │
    │  - Bounds checking    │   │   + 18 more...     │
    │  LOC: ~500            │   └────────────────────┘
    │  Dependencies: 0      │
    └───────────────────────┘
```

## Proposed Architecture 🆕

```
┌─────────────────────────────────────────┐
│     Universal Decoder Core              │
└────────────────┬────────────────────────┘
                 │
        ┌────────┼─────────────────┐
        │        │                 │
┌───────▼─────┐  │  ┌─────────────▼──────────┐
│  primitives │  │  │  decoder-encodings 🆕  │
│   (bytes)   │  │  │  - VarInt (Bitcoin)    │
│             │  │  │  - Compact-u16 (Solana)│
└─────────────┘  │  │  - RLP (Ethereum)      │
                 │  │  - Base58, Bech32      │
                 │  │  LOC: ~800             │
                 │  │  Dependencies: 0       │
                 │  └────────────────────────┘
                 │
    ┌────────────▼─────────────┐
    │  decoder-test-utils 🆕   │
    │  (dev-dependencies only) │
    │  - Test assertions       │
    │  - Property tests        │
    │  - Fixture loading       │
    │  LOC: ~300               │
    └──────────────────────────┘
```

## Key Extracted Functionalities

### 1. Variable-Length Encodings → decoder-encodings

| Encoding | Current Location | Used By | LOC |
|----------|-----------------|---------|-----|
| **VarInt** | decoder-bitcoin/src/varint.rs | Bitcoin, Litecoin, Dogecoin | 70 |
| **Compact-u16** | decoder-solana/src/parsing.rs | Solana | 100 |
| **RLP** | decoder-ethereum/src/rlp.rs | Ethereum, Polygon, Arbitrum, Optimism, BNB, Avalanche, Base | 340 |
| **Total** | - | **10+ chains** | **510** |

**Impact**: RLP alone shared by 7+ EVM chains → massive code reuse

### 2. Address Encodings → decoder-encodings/address

| Encoding | Used By | Implementation |
|----------|---------|----------------|
| **Base58** | Bitcoin, Solana, Stellar, Cardano | Vendor `bs58` crate |
| **Base58Check** | Bitcoin, Bitcoin forks | Build on base58 |
| **Bech32** | Bitcoin SegWit, Cosmos, Osmosis | Vendor `bech32` crate |
| **SS58** | Polkadot, Kusama, Substrate chains | Custom or vendor |
| **EIP-55** | Ethereum (checksummed hex) | Pure Rust impl |

**Impact**: 15+ chains need address formatting

### 3. Test Utilities → decoder-test-utils

```rust
// Before: Every decoder writes this
#[test]
fn test_decode_never_panics() {
    let result = std::panic::catch_unwind(|| {
        BitcoinDecoder::decode(&random_bytes)
    });
    assert!(result.is_ok());
}

// After: One-liner
#[test]
fn test_decode_never_panics() {
    assert_decode_never_panics::<BitcoinDecoder>(random_bytes);
}
```

**Impact**: Reduce test boilerplate by ~30%, standardize testing

## Migration Priority

### Week 1: decoder-encodings Core
- [x] Bitcoin decoder uses VarInt ✅ (already implemented)
- [ ] Move VarInt → decoder-encodings
- [ ] Update Bitcoin decoder to import from decoder-encodings
- [x] Solana decoder uses compact-u16 ✅ (already implemented)
- [ ] Move compact-u16 → decoder-encodings
- [x] Ethereum decoder uses RLP ✅ (already implemented)
- [ ] Move RLP → decoder-encodings

**Benefit**: EVM chains can now share RLP implementation

### Week 2: Address Formatting + Test Utils
- [ ] Vendor `bs58` crate using git subtree
- [ ] Add Base58/Base58Check APIs
- [ ] Vendor `bech32` crate
- [ ] Create decoder-test-utils crate
- [ ] Extract common test patterns

## Code Reduction

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| **Decoder-specific LOC** | 3,200 | 2,770 | **-430 LOC** (-13%) |
| **Shared encoding LOC** | 0 | 510 | **+510 LOC** (reusable) |
| **Future EVM decoder** | ~800 LOC | ~460 LOC | **-340 LOC** (42% reduction!) |
| **Production dependencies** | 5 | 5 | **0 change** ✅ |

## Why This Matters

### For Current Chains
- **Ethereum family** (7+ chains): Share RLP implementation
- **Bitcoin family** (3+ chains): Share VarInt implementation
- **All chains**: Share address formatting utilities

### For Future Chains
- **New EVM chain**: Import RLP, done
- **New Bitcoin fork**: Import VarInt, done
- **New chain with custom encoding**: Add to decoder-encodings, share with ecosystem

### For Security
- **Single source of truth**: Bug fixes benefit all chains
- **Better testing**: Focus testing effort on shared code
- **Easier auditing**: Review 500 LOC of encoding once vs 10x across decoders

## Decision Matrix

| Functionality | Extract? | Rationale |
|--------------|----------|-----------|
| Variable-length encodings (VarInt, RLP) | ✅ **YES** | Used by 10+ chains, 510 LOC saved |
| Address formatting (Base58, Bech32) | ✅ **YES** | Used by 15+ chains, airgapped requirement |
| Test utilities | ✅ **YES** | Reduces boilerplate, standardizes testing |
| Byte readers (primitives) | ✅ **Already done** | Working well, keep as-is |
| Hash functions (sha2, sha3) | ❌ **NO** | Different per chain, direct deps are fine |
| Chain-specific helpers | ⏸️ **WAIT** | Extract after 5+ implementations per family |

## Next Steps

1. **Review this analysis** with team
2. **Create decoder-encodings** (Week 1)
3. **Migrate existing decoders** to use shared encodings (Week 1)
4. **Vendor address encoding crates** (Week 2)
5. **Create decoder-test-utils** (Week 2)
6. **Update all decoder tests** to use shared utilities (Week 2)

## Success Criteria

- ✅ 10+ chains use shared encodings
- ✅ 430 LOC reduction across existing decoders
- ✅ Future EVM chains need ~50% less code
- ✅ No increase in production dependencies
- ✅ All existing tests pass
- ✅ Core TCB unchanged (< 3000 LOC)

---

**Status**: Ready for implementation
**Effort**: 2 weeks (Phase 2.2)
**Impact**: High - enables rapid addition of new chains
**Risk**: Low - moving existing code, not writing new logic
