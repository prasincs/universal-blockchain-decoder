# Starknet Research: Action Plan

**Date**: 2025-11-14
**Status**: ✅ Research Complete - Ready for Implementation
**Branch**: `claude/research-starknet-chains-01TKiKpvAsFe1KsmwoKxGtCQ`

---

## Research Completed ✅

Three comprehensive documents created:

1. **`docs/STARKNET_RESEARCH.md`** (844 lines)
   - Complete Starknet architecture analysis
   - 3 transaction types (INVOKE, DECLARE, DEPLOY_ACCOUNT)
   - Cryptographic primitives (Poseidon, Pedersen, STARK curve)
   - 230+ chains in ecosystem (mainnet + testnet + appchains)
   - Implementation feasibility: 2-3 weeks
   - ROI: VERY HIGH

2. **`docs/STARKNET_REUSABLE_COMPONENTS.md`** (893 lines)
   - 60-80% of infrastructure already exists
   - Core traits: 100% reusable
   - Byte readers: 95% reusable
   - Testing framework: 95% reusable
   - Effort savings: 70-75% (15 days → 4-5 days with dependencies)

3. **`docs/CRYPTO_VENDORING_LEVERAGE.md`** (663 lines)
   - **CRITICAL INSIGHT**: Vendoring crypto unlocks 300+ chains
   - Poseidon hash → 265+ chains
   - Pedersen hash → 235+ chains
   - Strategic approach: 56% faster (7 weeks vs 16 weeks for 5 ZK chains)
   - Recommendation: Create Phase 3.6a (ZK Crypto Infrastructure)

---

## Key Decision Made ✅

**Approach**: Strategic (Vendor Crypto First)

**Rationale**:
- 1 week extra upfront → 9 weeks saved overall
- Unlocks 300+ blockchain chains
- Achieves minimal TCB (project goal)
- Airgapped operation ready
- Single audit point

---

## Recommended Next Steps

### Immediate (This Week)

1. **Merge Research Branch**
   ```bash
   # Create PR for research documents
   gh pr create \
     --title "Research: Starknet family and ZK crypto leverage analysis" \
     --body "See docs/STARKNET_RESEARCH.md, docs/STARKNET_REUSABLE_COMPONENTS.md, and docs/CRYPTO_VENDORING_LEVERAGE.md"
   ```

2. **Update ROADMAP.md**
   - Insert Phase 3.6a: ZK Cryptography Infrastructure (2-3 weeks)
   - Update Phase 3.6 → Phase 3.6b: Starknet Decoder (1 week)
   - Add phases 3.7-3.10 for other ZK chains

### Phase 3.6a: ZK Cryptography Infrastructure (2-3 weeks)

**Week 1: Setup & STARK Field + Poseidon**
```bash
# Day 1-2: Setup
git checkout -b phase-3.6a/zk-crypto-infrastructure
mkdir -p crates/decoder-crypto-zk/src/{field,hash,curve,signature}

# Vendor starknet-crypto
git subtree add \
    --prefix crates/decoder-crypto-zk/vendored/starknet-crypto \
    https://github.com/xJonathanLEI/starknet-rs.git \
    starknet-crypto/v0.7.0 --squash

# Day 3-5: Extract and implement
- STARK field (252-bit modular arithmetic) - 200 LOC
- Poseidon hash (Hades permutation) - 500 LOC
- Property tests for field operations
```

**Week 2: Pedersen + STARK Curve + ECDSA**
```bash
# Day 6-7: Pedersen hash
- Extract Pedersen implementation - 400 LOC
- Elliptic curve point operations
- Property tests

# Day 8-10: STARK curve + ECDSA
- Curve primitives - 300 LOC
- ECDSA verification - 300 LOC
- Signature validation tests
```

**Week 3: Testing & Validation**
```bash
# Day 11-12: Comprehensive testing
- 100+ test vectors from Starknet docs
- Cross-validate with starknet-crypto (dev-dependency)
- Property tests (determinism, correctness)

# Day 13-14: Documentation & benchmarks
- Document all algorithms
- Add VENDORED.md with audit trail
- Performance benchmarks
- Update ROADMAP.md

# Day 15: Final validation
- All tests passing
- CI/CD integration
- Ready for Phase 3.6b
```

**Deliverables**:
- ✅ `crates/decoder-crypto-zk/` (~1,800 LOC)
- ✅ Poseidon hash (Starknet variant)
- ✅ Pedersen hash (Starknet variant)
- ✅ STARK field arithmetic
- ✅ ECDSA on STARK curve
- ✅ 100+ tests
- ✅ Comprehensive documentation

### Phase 3.6b: Starknet Decoder (1 week)

**Now Fast! Uses vendored crypto**

```bash
# Day 1-3: Transaction parsing
- Implement 3 transaction types (INVOKE, DECLARE, DEPLOY_ACCOUNT)
- Support v1 and v3 versions
- Use decoder-crypto-zk for all crypto

# Day 4-5: TxIR conversion & testing
- Convert Starknet → TxIR
- 50+ unit tests
- 20+ real transaction fixtures

# Day 6-7: Integration & documentation
- CI/CD validation
- Update docs
- Commit and push
```

**Deliverables**:
- ✅ `crates/decoder-starknet/` (~1,000 LOC)
- ✅ 230+ chains supported (mainnet + testnet + appchains)
- ✅ 50+ tests passing
- ✅ Full TxIR integration

### Phase 3.7+: Other ZK Chains (1 week each)

**Zcash** (1 week):
- Add Jubjub Pedersen to decoder-crypto-zk (3 days)
- Implement Sapling/Orchard decoder (4 days)

**Polygon zkEVM** (1 week):
- Add Goldilocks Poseidon to decoder-crypto-zk (3 days)
- Implement zkEVM decoder (4 days)

**Mina Protocol** (1 week):
- Add Pallas Poseidon to decoder-crypto-zk (2 days)
- Implement Mina decoder (5 days)

**Aleo** (1 week):
- Add BLS12-377 Poseidon to decoder-crypto-zk (3 days)
- Implement Aleo decoder (4 days)

---

## Timeline Summary

| Phase | Duration | Chains Unlocked | Status |
|-------|----------|-----------------|--------|
| 3.5 Cosmos SDK | - | 228 | ✅ Complete |
| **3.6a ZK Crypto** | **2-3 weeks** | **Infrastructure** | **📋 Next** |
| 3.6b Starknet | 1 week | 230+ | Pending |
| 3.7 Zcash | 1 week | 1 | Pending |
| 3.8 Polygon zkEVM | 1 week | 10+ | Pending |
| 3.9 Mina | 1 week | 1 | Pending |
| 3.10 Aleo | 1 week | 1 | Pending |
| **Total** | **7-8 weeks** | **~470 chains** | |

**vs Original Plan** (dependencies): 16 weeks
**Savings**: 8-9 weeks (56% faster)

---

## Success Criteria

**Phase 3.6a Complete When**:
- ✅ decoder-crypto-zk crate created
- ✅ All crypto primitives implemented and tested
- ✅ 100+ test vectors passing
- ✅ Cross-validated with starknet-crypto
- ✅ Documentation complete
- ✅ CI/CD passing

**Phase 3.6b Complete When**:
- ✅ All 3 transaction types decoded
- ✅ v1 and v3 versions supported
- ✅ Hash verification working (Poseidon + Pedersen)
- ✅ Signature verification working (ECDSA)
- ✅ TxIR conversion complete
- ✅ 50+ tests passing
- ✅ 20+ real transaction fixtures

---

## References

- `docs/STARKNET_RESEARCH.md` - Full technical analysis
- `docs/STARKNET_REUSABLE_COMPONENTS.md` - Reuse opportunities
- `docs/CRYPTO_VENDORING_LEVERAGE.md` - Strategic leverage analysis
- `ROADMAP.md` - Updated with new phases (to be updated)

---

## Questions & Decisions

**Resolved**:
- ✅ Vendor crypto first (Phase 3.6a) vs use dependencies - **VENDOR FIRST**
- ✅ Monolithic crypto crate vs modular - **MONOLITHIC (decoder-crypto-zk)**
- ✅ Starknet priority - **HIGH (after crypto infrastructure)**

**Open**:
- When to start Phase 3.6a? (Awaiting stakeholder approval)
- Which ZK chain after Starknet? (Recommend: Zcash or Polygon zkEVM)

---

**Status**: ✅ Ready to Begin Implementation
**Next Action**: Create Phase 3.6a branch and start vendoring starknet-crypto

---

**Document Version**: 1.0
**Date**: 2025-11-14
**Author**: Claude (Anthropic)
