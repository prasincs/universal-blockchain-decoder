# Technical Roadmap - Universal Blockchain Decoder

> **Pure software engineering**: decoders, verification, performance, security

**See also**:
- [Product Roadmap](./ROADMAP_PRODUCT.md) - Use cases, adoption, user validation
- [Marketing Roadmap](./ROADMAP_MARKETING.md) - Content, conferences, community
- [FAQ](./ROADMAP_FAQ.md) - Critical analysis and honest answers

---

## Current Status

**Version**: v0.1.0-alpha
**Phase**: 3.5 Complete (Cosmos SDK Decoder)
**Next**: Phase 5.2.1 (Benchmarks & Evaluation) ⭐ **CRITICAL**

**Completed**:
- ✅ Core architecture (traits, TxIR, canonical serialization)
- ✅ Bitcoin decoder (pure Rust, 186 tests)
- ✅ Ethereum decoder (pure Rust, RLP + EIP-2718, 6 tests)
- ✅ Solana decoder (pure Rust, compact-u16, 13 tests)
- ✅ Cosmos SDK decoder (Protobuf, 31 tests)
- ✅ OP Stack decoder (90% complete, deposit transactions)
- ✅ Verus integration (infrastructure, basic annotations)
- ✅ Type system visualization (blog post, interactive demo)

---

## Q1 2025 (Jan-Mar): Credibility & Evidence

**Goal**: Make project **credible** through proofs and benchmarks

### Critical Path

1. **Phase 4.1.1: Complete ONE Formal Proof** ⭐ **HIGHEST PRIORITY**
   - **Timeline**: 4 weeks
   - **Target**: Canonicalization determinism OR panic-freedom
   - **Deliverable**: Complete Verus proof + documentation
   - **Why critical**: Can't claim "formally verifiable" without actual proofs
   - **Blocker for**: Academic papers, conference submissions

2. **Phase 5.2.1: Benchmark Suite** ⭐ **CRITICAL**
   - **Timeline**: 4 weeks
   - **Deliverable**: Performance comparison vs native libraries
   - **Metrics**:
     - Decode time: P50, P95, P99 (Bitcoin, Ethereum, Solana)
     - Memory usage: Peak, average
     - Overhead vs direct parsing
   - **Comparison**:
     ```rust
     Bitcoin:  universal-decoder vs bitcoin crate
     Ethereum: universal-decoder vs alloy
     Solana:   universal-decoder vs solana-transaction-status
     ```
   - **Why critical**: Can't claim "fast" without data
   - **Blocker for**: Papers, production use

3. **Phase 3.2: Complete OP Stack** (4 hours)
   - Fix trait implementations
   - Integration tests with real deposit transactions
   - Connect to superchain registry

### Secondary (Can Defer)

4. **Phase 3.5 Enhancement: IBC & CosmWasm** (optional, 1-2 days)
5. **Phase 1.5.2: More Property Tests** (ongoing, 34 more needed)

---

## Q2 2025 (Apr-Jun): Security & Robustness

**Goal**: Make project **trustworthy** through security hardening

### Phase 5.1: Security Hardening

#### 5.1.1: Differential Fuzzing ⭐ **CRITICAL** (4 weeks)

**Goal**: Verify decoder correctness against reference implementations

```rust
// Example: Bitcoin differential fuzzing
fuzz_target!(|data: &[u8]| {
    let our_result = BitcoinDecoder::decode(data);
    let ref_result = bitcoin::Transaction::deserialize(data);

    // Must agree on parse success/failure
    assert_eq!(our_result.is_ok(), ref_result.is_ok());

    // If both succeed, validate equivalence
    if let (Ok(our_tx), Ok(ref_tx)) = (our_result, ref_result) {
        assert_tx_equivalence(our_tx, ref_tx);
    }
});
```

**Chains to fuzz**:
- Bitcoin (vs `bitcoin` crate)
- Ethereum (vs `alloy`)
- Solana (vs `solana-transaction-status`)
- Cosmos (vs `cosmos-sdk-proto`)

**Infrastructure**:
- `cargo-fuzz` with libFuzzer
- CI integration (run 1M iterations per PR)
- Corpus collection (save interesting inputs)

**Success criteria**:
- 10M+ fuzz iterations per chain
- 0 panics
- 0 disagreements with reference implementation

#### 5.1.2: Corpus Evaluation (4 weeks)

**Goal**: Validate on real-world transactions

**Process**:
1. Collect 100K real transactions per chain from mainnet
2. Decode all with our decoder
3. Compare to reference implementation
4. Document success rate, failure modes

**Chains**:
- Bitcoin: Blocks 0-800,000 (sample every 8 blocks = 100K txs)
- Ethereum: Blocks 1-18,000,000 (sample every 180 blocks = 100K txs)
- Solana: Recent 100K transactions from mainnet-beta

**Metrics**:
- Parse success rate (target: >99.9%)
- Agreement with reference (target: 100%)
- Edge cases discovered
- Performance on real data

**Deliverable**: `docs/CORPUS_EVALUATION_REPORT.md`

#### 5.1.3: External Security Audit (8 weeks, if funded)

**Scope**:
- Core library (~2700 LOC)
- Bitcoin decoder
- Ethereum decoder
- Solana decoder

**Focus areas**:
- Parser correctness (consensus bugs)
- Panic-freedom (DoS vulnerabilities)
- Signature verification correctness
- Information leakage

**Auditors** (shortlist):
- Trail of Bits ($80-120K)
- OpenZeppelin ($60-100K)
- Least Authority ($50-80K)

**Funding**: Apply to Ethereum Foundation, Web3 Foundation

**Timeline**: Q3 2025 (if funded in Q1)

#### 5.1.4: Bug Bounty Program (ongoing)

**Launch**: Q2 2025
**Platform**: HackerOne or direct GitHub issues
**Rewards**:
- Critical (consensus bug, panic): $500-$2,000
- High (incorrect parse, signature bypass): $200-$500
- Medium (performance issue, edge case): $50-$200

**Funding**: $10K initial pool (from grants or personal)

---

### Phase 5.2: Performance Optimization

#### 5.2.2: Profiling & Optimization (4 weeks)

**After benchmarks show bottlenecks**:

1. **Profile hot paths** (1 week):
   - Use `cargo flamegraph`, `perf`, `valgrind`
   - Identify allocation-heavy code
   - Find unnecessary copies

2. **Optimize allocations** (1 week):
   - Zero-copy parsing where possible
   - Arena allocation for temporary data
   - Reuse buffers

3. **Micro-optimizations** (1 week):
   - SIMD for hash functions (if bottleneck)
   - Lookup tables for encoding/decoding
   - Inline hot functions

4. **Re-benchmark** (1 week):
   - Measure improvement
   - Document trade-offs
   - Publish results

**Target**: Within 10% of specialized decoders (aggressive but achievable)

#### 5.2.3: Performance Regression Tests (1 week)

**CI integration**:
```bash
# Run on every PR
cargo bench --bench decode_benchmark

# Fail if >10% regression
```

**Metrics to track**:
- Decode time for standard transactions
- Memory usage
- Binary size

---

## Q3 2025 (Jul-Sep): Completeness

**Goal**: Complete remaining decoders

### Phase 3.3: Arbitrum Orbit Family Decoder (1-2 weeks)

**Custom transaction types**:
- Retryable tickets
- Batch transactions
- Cross-chain messages

**Chains covered**: Arbitrum One, Arbitrum Nova, all Orbit chains

### Phase 3.4: SVM Family Decoder (1 week)

**Chains**: Solana forks (Eclipse, Neon, etc.)

**Implementation**: Reuse Solana decoder with minor modifications

### Phase 3.6: Move VM Family Decoder (1 week)

**Chains**: Aptos, Sui

**Transaction structure**:
- Module publishing
- Entry function calls
- Move script execution

### Phase 3.7: Bitcoin Forks Family Decoder (3 days)

**Chains**: Litecoin, Dogecoin, Bitcoin Cash, Bitcoin SV, Dash

**Implementation**: Reuse Bitcoin decoder with parameter changes

### Phase 3.8: Privacy Chains Family Decoder (2 weeks)

**Zcash** (1 week):
- Transparent transactions (reuse UTXO decoder)
- Shielded transactions (Sapling, Orchard)
- Hybrid t→z, z→t transactions

**Monero** (1 week, research required):
- RingCT transactions
- Stealth addresses
- Bulletproofs

---

## Q4 2025 (Oct-Dec): Verification & Stability

### Phase 4.1: Complete Formal Verification

**Goal**: Verify core properties in Verus

#### 4.1.2: Core Properties (8 weeks)

1. **Canonicalization properties** (2 weeks):
   - Determinism: `encode(x) = encode(x)`
   - Injectivity: `encode(x) = encode(y) ⟹ x = y`

2. **Panic-freedom** (3 weeks):
   - All decode paths return Result
   - No array access panics
   - No arithmetic overflow

3. **Information preservation** (3 weeks):
   - All semantic information from original transaction preserved in TxIR
   - No data loss in canonicalization

#### 4.1.3: Reference Decoder Verification (8 weeks)

**Target**: Bitcoin decoder (simplest)

**Properties to prove**:
- Parse correctness (if decode succeeds, structure is valid)
- Signature verification correctness
- Canonical round-trip (decode → canonicalize deterministic)

---

### Phase 5.3: API Stabilization (4 weeks)

**Goal**: Commit to stable API for v1.0

**Tasks**:
1. **API review** (1 week):
   - Identify breaking changes
   - Simplify where possible
   - Add deprecation warnings

2. **Version policy** (1 week):
   - Document SemVer commitment
   - Deprecation timeline (6 months notice)
   - Migration guides

3. **Breaking changes** (2 weeks):
   - Fix naming issues (e.g., `canonical_hash` → `txir_content_hash`)
   - Unify error types
   - Simplify trait bounds

---

## 2026: Post-1.0 Research

### Advanced Formal Verification

- Verify all decoder implementations (not just core)
- Cross-chain property verification
- Integration with EVM/WASM verifiers

### Zero-Knowledge Proofs

- Generate ZK proofs of transaction validity
- Privacy-preserving transaction analysis (verify properties without revealing transaction)

### Streaming & Scalability

- Streaming decoder for large blocks
- Parallel batch decoding
- Memory-mapped file support

---

## Success Metrics (Technical)

### v0.2.0 (Q1 2025)
- ✅ 1 formal proof complete
- ✅ Benchmark suite published
- ✅ Performance evaluation complete
- ✅ 10 decoders implemented

### v0.3.0 (Q2 2025)
- ✅ Differential fuzzing: 10M+ iterations, 0 bugs
- ✅ Corpus evaluation: >99.9% success rate
- ✅ Performance within 20% of specialized decoders

### v0.4.0 (Q3 2025)
- ✅ External security audit passed (if funded)
- ✅ Bug bounty program active
- ✅ 20+ decoders implemented
- ✅ Performance within 10% of specialized decoders

### v1.0.0 (Q4 2025)
- ✅ Core fully verified in Verus
- ✅ 1 reference decoder verified (Bitcoin)
- ✅ Stable API commitment
- ✅ All major chain families supported

---

## Current Blockers

**For academic papers**:
1. ❌ No formal proofs (Phase 4.1.1 - 4 weeks)
2. ❌ No benchmarks (Phase 5.2.1 - 4 weeks)
3. ❌ No evaluation (Phase 5.1.2 - 4 weeks)

**For production use**:
1. ❌ No security audit (Phase 5.1.3 - 8 weeks + funding)
2. ❌ No extensive fuzzing (Phase 5.1.1 - 4 weeks)
3. ❌ API not stable (Phase 5.3 - 4 weeks)

**For v1.0**:
1. ❌ Formal verification incomplete (Phase 4.1 - 16 weeks)
2. ❌ Not all decoders implemented (Phase 3 - 8 weeks)

---

## Dependencies & Risks

### Technical Risks

**Risk**: Differential fuzzing finds consensus bugs
- **Likelihood**: Medium
- **Impact**: High (need to fix before production)
- **Mitigation**: Use reference implementations in dev-deps, extensive testing

**Risk**: Performance significantly slower than specialized decoders
- **Likelihood**: Low (architecture designed for zero-cost)
- **Impact**: Medium (limits adoption)
- **Mitigation**: Profile and optimize (Phase 5.2.2)

**Risk**: Verus proofs too difficult/time-consuming
- **Likelihood**: Medium
- **Impact**: High (credibility depends on verification)
- **Mitigation**: Start with simplest properties, get Verus community help

### Resource Risks

**Risk**: No funding for security audit
- **Likelihood**: Medium
- **Impact**: High (can't claim production-ready)
- **Mitigation**: Apply to multiple foundations, consider self-funded audit

**Risk**: No time for comprehensive verification
- **Likelihood**: Medium
- **Impact**: Medium (can still be useful without full verification)
- **Mitigation**: Prioritize one complete proof over many partial proofs

---

**Last Updated**: 2025-11-16
**Next Review**: 2026-01-16 (quarterly)
**Status**: Focused on credibility (proofs + benchmarks)
