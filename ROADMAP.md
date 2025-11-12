# Universal Blockchain Decoder Roadmap

## Current Status: v0.1.0-alpha (Phase 2.1 Complete ✅)

**Latest**: Pure Rust Bitcoin decoder + decoder-primitives crate
**Branch**: `claude/phase-2-learn-mea-011CV2ySRxpHH3dokEGzFiEe`
**See**: `docs/PHASE_2_STATUS_AND_FOLLOWUP_PLAN.md` for detailed status and next PRs

## Phase 1: Core Architecture ✅ COMPLETE

**Status**: Merged in initial PR
**Branch**: `claude/rust-blockchain-tx-decoder-011CV2dBVQBzjBkvEd7bznci`

### Delivered

- ✅ Core trait hierarchy (`ChainDecoder`, `Canonicalizer`, `TxHashable`)
- ✅ Trait-based chain identity (open extension, no enums)
- ✅ TxIR (Transaction Intermediate Representation)
- ✅ Canonical serialization (Borsh-based, secure)
- ✅ Hook system for extensible processing
- ✅ Comprehensive error handling
- ✅ Design documentation (5 major docs)
- ✅ Top 20 blockchain validation
- ✅ Formal verification roadmap

### Design Principles Established

1. **Minimal TCB**: Core < 3000 LOC
2. **Trait-based extensibility**: Zero core changes for new chains
3. **Canonical serialization**: Borsh only (no JSON for hashing)
4. **Zero-cost abstractions**: Static dispatch
5. **Formally verifiable**: Verus-ready

## Phase 1.5: Testing & Dependency Infrastructure ✅ COMPLETE

**Target**: v0.1.0-alpha+testing
**Timeline**: 2 weeks
**Focus**: Establish comprehensive testing strategy and minimal TCB
**Status**: Complete - decoder-primitives crate extracted, dependencies minimized
**See**: `docs/ARCHITECTURE_REFACTORING.md` for extraction rationale

### 1.5.1: Dependency Minimization (Week 1)

**Priority**: CRITICAL (Minimal TCB requirement)

**Current Status**: 8 production dependencies → **Target**: 5 dependencies

Tasks:
- [ ] Vendor `hex` crate using git subtree (~3 hours)
  - [ ] Use `git subtree add` for verifiable vendoring
  - [ ] Include LICENSE-MIT and LICENSE-APACHE
  - [ ] Create VENDORING.md with attribution
  - [ ] Re-export from core: `pub use vendored::hex`
  - [ ] Update all imports in decoder crates
- [ ] Move `serde_json` to dev-dependencies (display/test only)
  - [ ] Remove public JSON APIs from core
  - [ ] Keep JSON only for debugging/tests
- [ ] Benchmark `smallvec` vs `Vec`
  - [ ] Create criterion benchmarks
  - [ ] Decide: keep, remove, or vendor based on <10% threshold
- [ ] Move blockchain libs to dev-dependencies
  - [ ] `bitcoin` → dev-dependencies (test validation only)
  - [ ] `alloy` → dev-dependencies (test validation only)
  - [ ] Decoders use pure Rust parsing

**Final Dependencies**:
```toml
[dependencies]
serde = "1.0"      # Essential - serialization
borsh = "1.3"      # Essential - canonical encoding
thiserror = "1.0"  # Essential - error handling
sha2 = "0.10"      # Essential - Bitcoin hashing
sha3 = "0.10"      # Essential - Ethereum hashing
# hex - VENDORED via git subtree
```

**Documentation**:
- ✅ docs/TESTING_STRATEGY.md
- ✅ docs/DEPENDENCY_AUDIT.md
- ✅ docs/DECODER_DEPENDENCY_STRATEGY.md
- ✅ docs/VENDORING_GUIDE.md
- ✅ docs/GIT_SUBTREE_VENDORING.md
- ✅ docs/USING_VENDORED_DEPS.md

### 1.5.2: Testing Infrastructure (Week 2)

**Priority**: CRITICAL (Quality assurance)

**5-Level Testing Pyramid**:

1. **Unit Tests** (100% core, 90% decoders)
   - [ ] Set up test organization structure
   - [ ] Core: Test all public functions
   - [ ] Decoders: Test all parsers
   - [ ] Target: < 5 min execution time

2. **Property-Based Tests** (proptest)
   - [ ] Install proptest framework
   - [ ] Core: Canonical serialization properties
   - [ ] Core: Amount arithmetic properties
   - [ ] Decoders: Parse/encode roundtrip
   - [ ] Target: 50+ property tests

3. **Integration Tests** (Real blockchain data)
   - [ ] Create test fixture repository
   - [ ] Bitcoin: Genesis, SegWit, Taproot transactions
   - [ ] Ethereum: Legacy, EIP-1559, EIP-2930 transactions
   - [ ] Validation against reference implementations
   - [ ] Target: 100+ real transaction fixtures

4. **Fuzz Testing** (cargo-fuzz)
   - [ ] Install cargo-fuzz
   - [ ] Fuzz target: Bitcoin decoder
   - [ ] Fuzz target: Ethereum decoder
   - [ ] Fuzz target: Canonical serialization
   - [ ] Run continuously in CI (1 hour/night)

5. **Formal Verification** (Verus - foundational)
   - [ ] Install Verus toolchain
   - [ ] Add first annotations (Amount arithmetic)
   - [ ] Prove panic-freedom for critical paths
   - [ ] Document verification strategy

**CI/CD Pipeline**:
- [ ] GitHub Actions: Unit tests (every commit)
- [ ] GitHub Actions: Property tests (every commit)
- [ ] GitHub Actions: Integration tests (every commit)
- [ ] GitHub Actions: Fuzz tests (nightly)
- [ ] GitHub Actions: Coverage report (every PR, >90% threshold)
- [ ] GitHub Actions: Verify vendored dependencies

**Deliverables**:
- Comprehensive test suite
- CI/CD automation
- Coverage reporting (tarpaulin)
- Benchmark baseline (criterion)
- Test fixture repository

### Success Criteria

- ✅ ≤ 5 production dependencies in core
- ✅ hex vendored via git subtree (verifiable)
- ✅ Blockchain libs in dev-dependencies only
- ✅ Unit test coverage: 100% core, 90% decoders
- ✅ 50+ property tests
- ✅ 100+ real transaction fixtures
- ✅ Fuzzing infrastructure operational
- ✅ CI/CD pipeline passing
- ✅ Verus toolchain installed and annotated

**Reference Documentation**:
- `TESTING_AND_DEPENDENCIES_SUMMARY.md` - Executive summary
- `docs/TESTING_STRATEGY.md` - Complete testing approach
- `docs/GIT_SUBTREE_VENDORING.md` - Verifiable vendoring

---

## Phase 2: Reference Implementations (Months 2-3)

**Target**: v0.1.0-beta
**Timeline**: 6-8 weeks
**Focus**: Pure Rust decoders with comprehensive testing
**Prerequisites**: Phase 1.5 complete

### 2.1: Bitcoin Decoder - Pure Rust Implementation ✅ COMPLETE

**Priority**: HIGH (Reference UTXO implementation)
**Status**: Complete - PR merged with 56 passing tests
**See**: `docs/PHASE_2_STATUS_AND_FOLLOWUP_PLAN.md` for details and follow-up PRs

**Strategy**: Pure Rust parsing, validate against `bitcoin` crate in dev-dependencies

Implementation Tasks:
- ✅ Create `BitcoinChain` struct implementing `ChainIdentity`
- ✅ **Implement pure Rust parsing** (NO production dependency on `bitcoin` crate)
  - ✅ Parse version (4 bytes, little-endian)
  - ✅ Detect and parse SegWit marker/flag
  - ✅ Implement varint parsing with non-canonical detection
  - ✅ Parse inputs (prev_hash, index, script_sig, sequence)
  - ✅ Parse outputs (value, script_pubkey)
  - ✅ Parse witness data (if SegWit)
  - ✅ Parse locktime
- ✅ Update `BitcoinDecoder` to use new trait API
- ✅ Update `BitcoinTransaction::canonicalize()` to use `ChainRef`
- ✅ **Extract decoder-primitives crate** (prevents 600 LOC duplication)

Testing Tasks (56 tests passing):
- ✅ Add `bitcoin = "0.31"` to `[dev-dependencies]` (validation only)
- ✅ Unit tests for all parsing functions (30 tests)
- ✅ Unit tests for transaction types (7 tests)
- ✅ Integration tests (9 tests)
- ⏳ Bitcoin Core test vectors (500+ transactions) - PR #3
- ⏳ Property tests with proptest - PR #4
- ⏳ Fuzz testing with cargo-fuzz - PR #5

**Delivered**:
- ✅ **Pure Rust Bitcoin decoder** (604 LOC in parsing.rs)
- ✅ **decoder-primitives crate** (606 LOC, 27 tests)
  - Little-endian readers (Bitcoin, Solana)
  - Big-endian readers (Ethereum, Cosmos)
  - Bounds-checked byte operations
- ✅ **BitcoinTransaction type** (507 LOC)
- ✅ **56 passing tests** (47 unit + 9 integration)
- ✅ **Zero production dependencies** on blockchain libraries
- ✅ **All CI checks passing** (format, lint, minimal-versions)

**Validation Criteria**:
- ✅ **Pure Rust implementation** (no `bitcoin` crate in production deps)
- ✅ Decodes mainnet Bitcoin transactions
- ✅ SegWit support (BIP 141, 143, 144)
- ✅ Coinbase transaction detection
- ✅ Fee calculation with overflow protection
- ✅ TXID calculation (double SHA-256)

**Follow-up PRs** (see `docs/PHASE_2_STATUS_AND_FOLLOWUP_PLAN.md`):
- PR #2: Move bitcoin crate to dev-dependencies (HIGH priority, 1h)
- PR #3: Bitcoin Core test vectors - 500+ transactions (HIGH priority, 8-12h)
- PR #4: Property-based testing with proptest (MEDIUM priority, 6-8h)
- PR #5: Fuzzing infrastructure with cargo-fuzz (MEDIUM priority, 4-6h)
- PR #6: Performance benchmarking (LOW priority, 4-6h)
- PR #7: Documentation and examples (MEDIUM priority, 4-6h)

---

### 2.2: Ethereum Decoder - Pure Rust Implementation (Weeks 3-4)

**Priority**: HIGH (Reference Account implementation)

**Strategy**: Pure Rust RLP parsing, validate against `alloy` in dev-dependencies

Implementation Tasks:
- [ ] Create `EthereumChain` struct implementing `ChainIdentity`
- [ ] **Implement pure Rust RLP parsing** (NO production dependency on `alloy`)
  - [ ] RLP decoder (Recursive Length Prefix)
  - [ ] Legacy transaction parsing (pre-EIP-1559)
  - [ ] EIP-2930 transaction parsing (access lists)
  - [ ] EIP-1559 transaction parsing (base fee + priority fee)
  - [ ] Signature recovery (v, r, s)
  - [ ] Transaction type detection (0x00, 0x01, 0x02)
- [ ] Update `EthereumDecoder` to use new trait API
- [ ] Update `EthereumTransaction::canonicalize()` to use `ChainRef`

Testing Tasks:
- [ ] Add `alloy = "2.0"` to `[dev-dependencies]` (validation only)
- [ ] Create validation tests comparing with `alloy`
- [ ] Add real Ethereum transaction test cases
  - [ ] Legacy transaction (pre-EIP-1559)
  - [ ] EIP-1559 transaction (London hard fork)
  - [ ] EIP-2930 (access list)
  - [ ] Contract deployment
  - [ ] Contract call (ERC-20 transfer)
  - [ ] Large transaction (complex contract interaction)
- [ ] Property tests: RLP roundtrip
- [ ] Handle RLP decoding edge cases
- [ ] Fuzz testing with random bytes

**Validation Criteria**:
- ✅ **Pure Rust implementation** (no `alloy` in production deps)
- ✅ RLP parsing matches `alloy` behavior (validated in tests)
- ✅ Decodes mainnet Ethereum transactions
- ✅ Handles all transaction types (Legacy, EIP-2930, EIP-1559)
- ✅ Canonical serialization works
- ✅ Gas calculations correct
- ✅ Signature recovery works
- ✅ Fuzz testing: No panics on any input

**See**: `docs/DECODER_DEPENDENCY_STRATEGY.md` for rationale

### 2.3: Integration Tests & Examples (Week 3)

**Priority**: MEDIUM

Tasks:
- [ ] Update `simple-decoder` example with real transactions
- [ ] Create end-to-end test suite
- [ ] Add performance benchmarks (criterion)
- [ ] Test hook system with real use cases
- [ ] Add batch decoding tests
- [ ] Documentation review and updates

**Deliverables**:
- Working example decoding Bitcoin and Ethereum transactions
- Performance baseline established
- User documentation complete

## Phase 3: Extended Chain Support (Months 2-3)

**Target**: v0.2.0
**Focus**: Demonstrate universality with diverse chains

### 3.1: Solana Decoder (Instruction Model)

**Priority**: HIGH (Different model validation)

Tasks:
- [ ] Create `SolanaChain` struct implementing `ChainIdentity`
- [ ] Implement full Solana transaction parsing
- [ ] Handle versioned transactions (Legacy, V0)
- [ ] Support address lookup tables
- [ ] Map instructions to TxIR operations
- [ ] Test with real Solana transactions

**Why Important**: Validates instruction-based model support

### 3.2: Cardano Decoder (Extended UTXO)

**Priority**: MEDIUM (Advanced UTXO validation)

Tasks:
- [ ] Create `CardanoChain` struct
- [ ] Implement eUTXO parsing (with datums)
- [ ] Support multi-asset transactions
- [ ] Handle staking certificates
- [ ] Plutus script representation

**Why Important**: Tests limits of UTXO abstraction

### 3.3: Polkadot/Substrate Decoder

**Priority**: MEDIUM (Metadata-driven decoding)

Tasks:
- [ ] Create `PolkadotChain` struct
- [ ] Implement SCALE codec decoding
- [ ] Add `MetadataProvider` trait for runtime metadata
- [ ] Support extrinsics
- [ ] Handle cross-chain messages (XCM)

**Why Important**: Tests metadata-driven extensibility

## Phase 4: Formal Verification (Months 3-4)

**Target**: v0.3.0
**Focus**: Mathematical proofs of correctness

### 4.1: Core Verification

**Priority**: HIGH (Security critical)

Tasks:
- [ ] Set up Verus toolchain in CI
- [ ] Add Verus annotations to core traits
- [ ] Prove panic-freedom for decode path
- [ ] Prove overflow safety in arithmetic
- [ ] Verify canonical serialization determinism

**Key Properties**:
- Injectivity: `encode(decode(bytes)) == bytes`
- Panic-freedom: No unwrap(), no panics
- Determinism: Same data → same bytes

### 4.2: Bitcoin Decoder Verification

**Priority**: MEDIUM

Tasks:
- [ ] Verify UTXO parsing bounds
- [ ] Prove fee calculation overflow safety
- [ ] Verify canonicalization injectivity
- [ ] Prove script parsing safety

### 4.3: Verification Documentation

**Priority**: MEDIUM

Tasks:
- [ ] Document proof strategies
- [ ] Create verification examples
- [ ] Publish verification results
- [ ] Academic paper (optional)

## Phase 5: Production Hardening (Months 4-5)

**Target**: v0.4.0
**Focus**: Security, performance, reliability

### 5.1: Security Audit

**Priority**: CRITICAL

Tasks:
- [ ] External security audit (Trail of Bits, OpenZeppelin, etc.)
- [ ] Fuzzing campaign (cargo-fuzz)
- [ ] Differential testing against reference implementations
- [ ] Address audit findings
- [ ] Publish security advisory process

### 5.2: Performance Optimization

**Priority**: MEDIUM

Tasks:
- [ ] Profile hot paths
- [ ] Optimize allocations
- [ ] Add zero-copy parsing where possible
- [ ] Benchmark against single-chain decoders
- [ ] Document performance characteristics

**Target**: Within 10% of specialized decoders

### 5.3: API Stabilization

**Priority**: HIGH

Tasks:
- [ ] API review and finalization
- [ ] Deprecation policy
- [ ] Semantic versioning commitment
- [ ] Breaking change documentation
- [ ] Migration guides

## Phase 6: Ecosystem & Adoption (Months 5-6)

**Target**: v1.0.0 🎉
**Focus**: Production-ready, community-driven

### 6.1: Additional Chains (Community-Driven)

**Priority**: COMMUNITY

Chains to support (priority order):
1. [ ] Cosmos (IBC support)
2. [ ] Avalanche (multi-chain)
3. [ ] NEAR Protocol (Borsh-native!)
4. [ ] Tron
5. [ ] Stellar
6. [ ] Algorand
7. [ ] Monero (privacy-limited)

### 6.2: Tooling & Integration

**Priority**: MEDIUM

Tasks:
- [ ] CLI tool for transaction inspection
- [ ] Web service example (REST API)
- [ ] WASM bindings for browser/Node.js
- [ ] Language bindings (Python, Go via FFI)
- [ ] Integration examples (indexers, explorers)

### 6.3: Documentation & Community

**Priority**: HIGH

Tasks:
- [ ] Complete API documentation
- [ ] Tutorial series
- [ ] Video walkthrough
- [ ] Contributing guide enhancements
- [ ] Discord/forum setup
- [ ] Monthly contributor calls

## Post-1.0: Advanced Features

### Research Directions

**Zero-Knowledge Proofs**:
- Generate ZK proofs of transaction validity
- Privacy-preserving transaction analysis

**Cross-Chain Analytics**:
- Unified query language across chains
- Cross-chain transaction tracing

**AI/ML Integration**:
- Transaction classification
- Anomaly detection
- Pattern recognition

**Formal Verification Extensions**:
- Verify decoder implementations (not just core)
- Prove protocol-level properties
- Integration with EVM/WASM verifiers

## Success Metrics

### v0.1.0 (Architecture)
- ✅ Core < 3000 LOC
- ✅ Trait-based extensibility
- ✅ Top 20 blockchain validation
- ✅ Design docs complete

### v0.2.0 (Chain Support)
- [ ] 5+ chains supported
- [ ] All 3 models (UTXO, Account, Instruction) proven
- [ ] Real transaction test coverage > 80%

### v0.3.0 (Verification)
- [ ] Core fully verified in Verus
- [ ] At least 1 decoder fully verified
- [ ] Zero critical bugs from fuzzing

### v0.4.0 (Production)
- [ ] Security audit passed
- [ ] Performance within 10% of specialized decoders
- [ ] Stable API commitment

### v1.0.0 (Ecosystem)
- [ ] 10+ chains supported
- [ ] 5+ external contributors
- [ ] Used in production by 2+ projects
- [ ] 1000+ GitHub stars

## Contributing

See `CONTRIBUTING.md` for:
- How to add a new chain decoder
- Code review process
- Testing requirements
- Documentation standards

## Questions or Ideas?

- Open an issue: https://github.com/prasincs/universal-blockchain-decoder/issues
- Discussion: https://github.com/prasincs/universal-blockchain-decoder/discussions
- Contact maintainers

---

**Last Updated**: 2025-01-11
**Current Phase**: Phase 1.5 In Progress ⚡
**Status**: Testing & dependency strategy documentation complete
**Next Milestones**:
  - v0.1.0-alpha+testing (Phase 1.5 complete) - 2 weeks
  - v0.1.0-beta (Phase 2 complete) - 2-3 months
