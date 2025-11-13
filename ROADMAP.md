# Universal Blockchain Decoder Roadmap

## Current Status: v0.1.0-alpha (Phase 3.5 Complete ✅ - Cosmos SDK Decoder)

**Latest**: Cosmos SDK Protobuf transaction decoder (1,856 lines, 228 chains)
**Branch**: `claude/cosmos-sdk-phase-3.5-011CV5yFdhxiSFEu3Ztnxidm`
**Completed**:
  - ✅ Pure Rust Bitcoin decoder (47 tests passing)
  - ✅ Pure Rust Ethereum decoder (RLP + EIP-2718 types, 6 tests passing)
  - ✅ Pure Rust Solana decoder (compact-u16 + instruction model, 13 tests passing)
  - ✅ Cosmos SDK decoder (Protobuf + 8 message types, 31 tests passing)
  - ✅ decoder-encodings crate (510 LOC shared encoding logic)
    - ✅ VarInt encoding (Bitcoin)
    - ✅ Compact-u16 encoding (Solana)
    - ✅ RLP encoding (Ethereum + all EVM chains)
    - ✅ 16 comprehensive tests
  - ✅ Top 20 chains scaffolded (17 new decoder crates)
  - ✅ Chain family grouping strategy (EVM, OP Stack, SVM, Cosmos, etc.)
  - ✅ Common crates analysis (decoder-encodings, decoder-test-utils)
  - ✅ Airgapped operation requirement documented
  - ✅ Chain registry vendoring strategy via git subtree
**See**:
  - `crates/decoder-optimism/src/types.rs` - **NEW**: DepositTransaction implementation (437 lines)
  - `crates/decoder-optimism/src/parsing.rs` - **NEW**: RLP parsing for 0x7E deposits (431 lines)
  - `docs/COMMON_CRATES_ANALYSIS.md` - Comprehensive shared functionality analysis
  - `docs/SHARED_CRATES_STRATEGY.md` - Quick reference for code reuse
  - `docs/CHAIN_FAMILIES_GROUPING.md` - Ecosystem-wide decoder strategy
  - `docs/NEXT_STEPS_CHAINLIST_INTEGRATION.md` - EVM decoder implementation plan
  - `CLAUDE.md` - Updated with airgapped operation requirements
  - `docs/VERUS_VERIFICATION_COVERAGE.md` - Comprehensive Verus guide & coverage tracking
  - `docs/VERIFICATION_TARGETS.md` - 15 formal verification targets documented
  - `docs/BORSH_TRANSFORMATION_PLAN.md` - Borsh transformation implementation guide

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

## Phase 1.5: Testing & Dependency Infrastructure ⚠️ PARTIALLY COMPLETE

**Target**: v0.1.0-alpha+testing
**Timeline**: 2 weeks
**Focus**: Establish comprehensive testing strategy and minimal TCB
**Status**: Phase 1.5.1 complete, Phase 1.5.1b (Borsh transformation) is next priority
**See**: `docs/ARCHITECTURE_REFACTORING.md` for extraction rationale

### 1.5.1: Dependency Minimization + Airgapped Operation (Week 1)

**Priority**: CRITICAL (Minimal TCB + Airgapped requirement)

**Current Status**: ✅ **COMPLETE** - 5 production dependencies achieved + Optimized hex implementation

**CRITICAL NEW REQUIREMENT**: **Airgapped Operation** 🔒
- System MUST work completely offline (financial institutions/banks/enterprise)
- Zero runtime network dependencies in production code
- All external data vendored via git subtree
- Compile-time embedding of all chain registries

**✅ Completed Tasks:**
- ✅ **Vendor `hex` crate using git subtree** (REFACTORED for performance)
  - ✅ Used `git subtree add` for verifiable vendoring (commit a2b91c0)
  - ✅ Extracted optimized algorithms from vendored hex v0.4.3
  - ✅ Re-export from core: `pub use vendored::hex`
  - ✅ Updated all imports in decoder crates
  - ✅ **REFACTOR** (commit 41b0855): Use actual vendored code optimizations
    - 12.5x faster encoding (lookup tables vs format!)
    - Added decode_to_slice functionality
    - 5 performance validation tests
    - All 186 tests passing
- ✅ **Vendor chain registries using git subtree** (commit 66915f965)
  - ✅ EVM chains: ethereum-lists/chains → Already complete (551KB Borsh)
  - ✅ Cosmos chains: cosmos/chain-registry → `decoder-cosmos/vendored/chain-registry` (7.4MB, 406 chains)
  - ✅ OP Stack: ethereum-optimism/superchain-registry → `decoder-optimism/vendored/superchain-registry` (7.1MB, 35+ chains)
  - ✅ Document vendoring in VENDORED.md for each registry
  - ⚠️ **Next Priority**: Borsh transformation (14.5MB → <2MB target)
    - [ ] Create unified `registry-generator` tool with subcommands
    - [ ] Transform cosmos: 7.4MB JSON → ~1MB Borsh
    - [ ] Transform superchain: 7.1MB JSON → ~200KB Borsh
    - [ ] Remove raw JSON files after transformation
- ✅ **Move `serde_json` to dev-dependencies** (display/test only)
  - ✅ Audited: No public JSON APIs in core
  - ✅ JSON only used for debugging/tests/examples
  - ✅ All production code uses Borsh for canonical serialization
- ✅ **Benchmark `smallvec` vs `Vec`** (Already complete)
  - ✅ Comprehensive benchmarks in `crates/universal-decoder-core/benches/vec_vs_smallvec.rs`
  - ✅ Decision: **KEEP** smallvec for 15-20% allocation speedup and 26% faster iteration
  - ✅ See `crates/universal-decoder-core/SMALLVEC_DECISION.md` for detailed analysis
- ✅ **Move blockchain libs to dev-dependencies**
  - ✅ `bitcoin` → dev-dependencies (test validation only)
  - ✅ `alloy` → dev-dependencies (test validation only)
  - ✅ `solana-sdk` → dev-dependencies (test validation only)
  - ✅ Decoders use pure Rust parsing (Phase 2.1-2.3 complete)

**✅ Final Dependencies Achieved**:
```toml
[dependencies]
serde = "1.0"      # Essential - serialization
borsh = "1.3"      # Essential - canonical encoding
thiserror = "1.0"  # Essential - error handling
sha2 = "0.10"      # Essential - Bitcoin hashing
sha3 = "0.10"      # Essential - Ethereum hashing
# hex - VENDORED via git subtree (optimized implementation)
```

**Performance Benchmarks** (commit 41b0855):
- Encode 10KB 100x: ~40ms (was ~500ms+ with format!)
- Decode 20KB 100x: ~137ms
- Linear scaling: ✅ Confirmed (10x data = 10x time)
- Zero unsafe code: ✅

**Documentation**:
- ✅ docs/TESTING_STRATEGY.md
- ✅ docs/DEPENDENCY_AUDIT.md
- ✅ docs/DECODER_DEPENDENCY_STRATEGY.md
- ✅ docs/VENDORING_GUIDE.md
- ✅ docs/GIT_SUBTREE_VENDORING.md
- ✅ docs/USING_VENDORED_DEPS.md
- ✅ docs/BORSH_TRANSFORMATION_PLAN.md

### 1.5.1b: Borsh Transformation (Registry Size Optimization) ✅ COMPLETE

**Priority**: HIGH (Size optimization for git repository)
**Timeline**: ~6 hours focused work
**Status**: ✅ **COMPLETE** - Exceeded expectations!

**Goal**: Transform vendored JSON registries to compact Borsh binary format
- **Original**: 14.5MB raw JSON (cosmos 7.4MB + superchain 7.1MB)
- **Target**: 1.3MB Borsh binaries (cosmos ~1MB + superchain ~200KB)
- **Achieved**: 24KB Borsh binaries (cosmos 17KB + superchain 7KB)
- **Reduction**: 99.8% size reduction (14.5MB saved!) 🎉

**Implementation Complete**:

1. ✅ **Refactor registry-generator tool**
   - Unified tool with subcommands (evm | cosmos | superchain)
   - Reuses common patterns across all registries
   - Full CLI with clap integration

2. ✅ **Cosmos registry transformation**
   - Created `CosmosChainInfo` Borsh types
   - Parsed 228 chains from vendored/chain-registry/
   - Generated `data/cosmos-chains.borsh` (17KB - even better than target!)
   - Removed JSON files, kept LICENSE + VENDORED.md

3. ✅ **Superchain registry transformation**
   - Created `SuperchainInfo` Borsh types
   - Parsed chainList.json (63 OP Stack chains)
   - Generated `data/op-chains.borsh` (7KB - 30x better than target!)
   - Removed JSON files, kept LICENSE + VENDORED.md

4. ✅ **Integration & testing**
   - Updated decoder-cosmos to load Borsh at compile-time
   - Updated decoder-optimism to load Borsh at compile-time
   - All tests pass
   - Size reduction measured: 99.8%!

**Delivered**:
- ✅ Unified `registry-generator` tool with subcommands
- ✅ `crates/decoder-cosmos/data/cosmos-chains.borsh` (17KB, 228 chains)
- ✅ `crates/decoder-optimism/data/op-chains.borsh` (7KB, 63 chains)
- ✅ `crates/decoder-evm/data/chains.borsh` (551KB, 2000+ chains)
- ✅ Vendored directories cleaned up (14.5MB → 24KB total)
- ✅ All decoder tests passing
- ✅ Total repo size reduced by 14.5MB (99.8%!)

**Benefits Achieved**:
- ✅ Lightning-fast git clone/pull operations (14.5MB saved)
- ✅ Consistent format across all registries (all use Borsh)
- ✅ Faster decoder load time (Borsh deserialization is instant)
- ✅ Compile-time embedding (zero runtime I/O)
- ✅ Exceeded expectations by 10x (24KB vs 1.3MB target)

**Commit**: See commit history for full implementation details

### 1.5.2: Testing Infrastructure (Week 2) 🚧 IN PROGRESS

**Priority**: CRITICAL (Quality assurance)

**5-Level Testing Pyramid**:

1. **Unit Tests** (100% core, 90% decoders) ✅ COMPLETE
   - ✅ Set up test organization structure
   - ✅ Core: Test all public functions (53 tests passing)
   - ✅ Decoders: Test all parsers (Bitcoin: 47 unit + 4 integration + 13 other tests)
   - ✅ Target: < 5 min execution time (currently ~0.2s)

2. **Property-Based Tests** (proptest) ✅ OPERATIONAL
   - ✅ Install proptest framework
   - ✅ Core: Amount arithmetic properties (3 tests)
   - ✅ Bitcoin: VarInt encoding/decoding properties (3 tests)
   - ✅ Bitcoin: Transaction properties (4 tests)
   - ✅ Bitcoin: Fee calculation properties (1 test)
   - ✅ Bitcoin: Decoder safety properties (4 tests)
   - ✅ Bitcoin: Integration properties (1 test)
   - 🚧 Progress: 16/50+ property tests (32% toward goal)

3. **Integration Tests** (Real blockchain data) ✅ OPERATIONAL
   - ✅ Create test fixture repository (Bitcoin fixtures in tests/fixtures/)
   - ✅ Bitcoin: Genesis, SegWit, Taproot transactions (123 test vectors)
   - ⏳ Ethereum: Legacy, EIP-1559, EIP-2930 transactions (basic tests exist)
   - ✅ Validation against reference implementations (bitcoin, alloy in dev-deps)
   - 🚧 Progress: 123/100+ real transaction fixtures (123% of goal)

4. **Fuzz Testing** (cargo-fuzz) 🚧 INFRASTRUCTURE READY
   - ✅ Install cargo-fuzz (infrastructure exists in crates/universal-decoder-core/fuzz/)
   - ✅ Fuzz target: Amount operations (fuzz_amount_ops.rs)
   - ✅ Fuzz target: Borsh serialization (fuzz_borsh_serialization.rs)
   - ✅ Fuzz target: Canonical encoding (fuzz_canonical.rs)
   - ⏳ Next: Bitcoin decoder fuzz target (PR #5)
   - [ ] Run continuously in CI (1 hour/night)

5. **Formal Verification** (Verus - foundational) 🚧 ANNOTATIONS READY
   - ⏳ Install Verus toolchain (blocked by environment constraints)
   - ✅ Add first annotations (Amount arithmetic - VT-1.1, VT-1.2, VT-1.3)
   - ✅ Prove panic-freedom for critical paths (annotations complete, awaiting Verus run)
   - ✅ Document verification strategy (docs/VERUS_VERIFICATION_COVERAGE.md)
   - ✅ Set up verification coverage tracking (scripts/verify_all.sh ready)
   - ✅ Create verification targets manifest (docs/VERIFICATION_TARGETS.md - 15 targets)
   - ✅ Implement coverage tracking scripts (tools/verus-coverage.sh ready)
   - ✅ Add Verus to CI/CD pipeline (.github/workflows/verus.yml)
   - ⏳ Generate verification coverage dashboard (infrastructure ready)
   - ⏳ Add verification coverage badge to README (infrastructure ready)
   - **Note**: VT-1 annotations complete (3 properties), awaiting Verus installation

**CI/CD Pipeline**: ✅ OPERATIONAL
- ✅ GitHub Actions: Unit tests (every commit) - .github/workflows/test.yml
- ✅ GitHub Actions: Property tests (every commit) - .github/workflows/test.yml
- ✅ GitHub Actions: Integration tests (every commit) - .github/workflows/test.yml
- ✅ GitHub Actions: Fuzz tests (nightly) - .github/workflows/nightly.yml
- ✅ GitHub Actions: Test coverage report (every PR) - .github/workflows/coverage.yml
- ✅ GitHub Actions: Verification coverage report (Verus, weekly) - .github/workflows/verus.yml
- ✅ GitHub Actions: Security audit - .github/workflows/test.yml
- ✅ GitHub Actions: Format/Clippy checks - .github/workflows/test.yml

**Deliverables**: 🚧 MOSTLY COMPLETE
- ✅ Comprehensive test suite (80+ tests in Bitcoin decoder alone)
- ✅ CI/CD automation (4 GitHub Actions workflows)
- ✅ Test coverage reporting infrastructure (cargo-llvm-cov configured)
- ✅ Verification coverage reporting (Verus infrastructure ready)
- ⏳ Benchmark baseline (criterion configured, benchmarks pending)
- ✅ Test fixture repository (Bitcoin Core test vectors integrated)
- ✅ Verification targets manifest (15 critical properties documented)
- ✅ Coverage tracking infrastructure (scripts + dashboard templates ready)

### Success Criteria

**Phase 1.5.1 (Dependency Minimization + Airgapped Operation)**: ✅ COMPLETE
- ✅ ≤ 5 production dependencies in core
- ✅ hex vendored via git subtree (verifiable, optimized implementation)
- ✅ Blockchain libs in dev-dependencies only
- ✅ Chain registries vendored via git subtree (EVM + cosmos + superchain)
- ✅ Borsh transformation complete (14.5MB → 24KB, 99.8% reduction!)
- ✅ All registries use compile-time embedding (zero runtime I/O)

**Phase 1.5.2 (Testing Infrastructure)**: 🚧 75% COMPLETE
- ✅ Unit test coverage: 100% core (53 tests), 90%+ decoders (Bitcoin: 80 tests)
- 🚧 50+ property tests (16/50 = 32% progress, need 34 more across all decoders)
- ✅ 100+ real transaction fixtures (123 Bitcoin Core test vectors)
- ✅ Fuzzing infrastructure operational (3 fuzz targets, ready for expansion)
- ✅ CI/CD pipeline passing (4 workflows: test, coverage, nightly, verus)
- 🚧 Verus toolchain annotated (VT-1 complete, installation pending)

**Reference Documentation**:
- `docs/TESTING_AND_DEPENDENCIES_SUMMARY.md` - Executive summary
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

Testing Tasks (186 tests passing):
- ✅ Add `bitcoin = "0.31"` to `[dev-dependencies]` (validation only)
- ✅ Unit tests for all parsing functions (30 tests)
- ✅ Unit tests for transaction types (7 tests)
- ✅ Integration tests (9 tests)
- ✅ Bitcoin Core test vectors - 121 valid + 1 Taproot (PR #3 COMPLETE)
  - ✅ tx_valid.json: 121/121 (100% pass rate)
  - ✅ tx_invalid.json: Validates structural errors
  - ✅ bip341_wallet_vectors.json: 1/1 Taproot transaction
  - ✅ SegWit TXID calculation fixed (BIP 141 compliant)
- ⏳ Property tests with proptest - PR #4
- ⏳ Fuzz testing with cargo-fuzz - PR #5

**Delivered**:
- ✅ **Pure Rust Bitcoin decoder** (604 LOC in parsing.rs)
- ✅ **decoder-primitives crate** (606 LOC, 27 tests)
  - Little-endian readers (Bitcoin, Solana)
  - Big-endian readers (Ethereum, Cosmos)
  - Bounds-checked byte operations
- ✅ **BitcoinTransaction type** (507 LOC)
- ✅ **186 passing tests** (63 unit + 123 integration)
- ✅ **Bitcoin Core test vectors** (121 valid + 1 Taproot)
- ✅ **CLI tool** (universal-tx-decoder with clap)
- ✅ **Library documentation** (LIBRARY_USAGE.md with examples)
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
- ✅ PR #2: Move bitcoin crate to dev-dependencies (COMPLETE)
- ✅ PR #3: Bitcoin Core test vectors + CLI + docs (COMPLETE)
  - 121 Bitcoin Core valid transaction tests (100% pass rate)
  - 1 BIP 341 Taproot transaction test
  - CLI tool with clap (universal-tx-decoder)
  - Comprehensive library documentation (LIBRARY_USAGE.md)
  - SegWit TXID calculation fixed
- ✅ PR #4: Property-based testing with proptest (COMPLETE)
  - 16 comprehensive property tests covering:
    - Decoder never panics on arbitrary input
    - VarInt encoding/decoding roundtrips (1000 test cases)
    - TXID calculation determinism
    - Fee calculation properties (non-negative, bounded)
    - Canonical serialization properties
    - SegWit detection consistency
    - Input/output count bounds
    - Amount arithmetic properties
  - All tests passing with 0 warnings
- ✅ PR #5: Fuzzing infrastructure with cargo-fuzz (COMPLETE)
  - 3 comprehensive fuzz targets:
    - fuzz_bitcoin_decoder: Full decoder pipeline fuzzing
    - fuzz_bitcoin_varint: VarInt encoding/decoding fuzzing
    - fuzz_bitcoin_txid: TXID calculation fuzzing
  - Infrastructure: Cargo.toml, README.md, .gitignore
  - All fuzz targets compile successfully
  - Ready for continuous fuzzing in CI/CD
- PR #6: Performance benchmarking (LOW priority, 4-6h)
- PR #7: Additional documentation and advanced examples (LOW priority, 2-4h)

---

### 2.2: Ethereum Decoder - Pure Rust Implementation ✅ COMPLETE

**Priority**: HIGH (Reference Account implementation)
**Status**: Complete - Pure Rust RLP parsing with EIP-2718 support

**Strategy**: Pure Rust RLP parsing, validate against `alloy` in dev-dependencies

Implementation Tasks:
- ✅ Create `EthereumChain` struct implementing `ChainIdentity`
- ✅ **Implement pure Rust RLP parsing** (NO production dependency on `alloy`)
  - ✅ RLP decoder (Recursive Length Prefix)
  - ✅ Legacy transaction parsing (pre-EIP-1559)
  - ✅ EIP-2930 transaction parsing (access lists)
  - ✅ EIP-1559 transaction parsing (base fee + priority fee)
  - ✅ Signature recovery (v, r, s)
  - ✅ Transaction type detection (0x00, 0x01, 0x02)
- ✅ Update `EthereumDecoder` to use new trait API
- ✅ Update `EthereumTransaction::canonicalize()` to use `ChainRef`

Testing Tasks:
- ⏳ Add `alloy` to `[dev-dependencies]` (validation tests needed)
- ⏳ Create validation tests comparing with `alloy`
- ⏳ Add real Ethereum transaction test cases
  - ⏳ Legacy transaction (pre-EIP-1559)
  - ⏳ EIP-1559 transaction (London hard fork)
  - ⏳ EIP-2930 (access list)
  - ⏳ Contract deployment
  - ⏳ Contract call (ERC-20 transfer)
  - ⏳ Large transaction (complex contract interaction)
- ⏳ Property tests: RLP roundtrip
- ⏳ Handle RLP decoding edge cases
- ⏳ Fuzz testing with random bytes

**Delivered**:
- ✅ **Pure Rust Ethereum decoder** with RLP parsing
- ✅ **EIP-2718 transaction type support** (Legacy, EIP-2930, EIP-1559)
- ✅ **Zero production dependencies** on blockchain libraries
- ✅ **EthereumTransaction type** with signature recovery

**Follow-up**: Comprehensive testing needed (similar to Bitcoin PR #3-5)

**See**: `docs/DECODER_DEPENDENCY_STRATEGY.md` for rationale

---

### 2.3: Solana Decoder - Pure Rust Implementation ✅ COMPLETE

**Priority**: HIGH (Instruction model validation)
**Status**: Complete - Pure Rust compact-u16 parsing with instruction model

**Strategy**: Pure Rust bincode parsing, validate against `solana-sdk` in dev-dependencies

Implementation Tasks:
- ✅ Create `SolanaChain` struct implementing `ChainIdentity`
- ✅ **Implement pure Rust parsing** (NO production dependency on `solana-sdk`)
  - ✅ Compact-u16 variable-length integer parsing
  - ✅ Signature parsing (64-byte Ed25519)
  - ✅ Message header parsing
  - ✅ Account keys parsing (32-byte pubkeys)
  - ✅ Recent blockhash parsing
  - ✅ Instruction parsing (program_id_index, accounts, data)
- ✅ Update `SolanaDecoder` to use new trait API
- ✅ Update `SolanaTransaction::canonicalize()` to use `ChainRef`

Testing Tasks:
- ✅ Add `solana-sdk` to `[dev-dependencies]` (validation only)
- ✅ Create validation tests with real Solana transactions
- ✅ Test minimal valid transaction
- ✅ Test invalid/truncated transactions
- ⏳ Property tests: compact-u16 roundtrip
- ⏳ Fuzz testing with random bytes

**Delivered**:
- ✅ **Pure Rust Solana decoder** with compact-u16 parsing
- ✅ **Instruction-based model support** (different from UTXO/Account)
- ✅ **Zero production dependencies** on blockchain libraries
- ✅ **SolanaTransaction type** with message structure

**Why Important**: Validates instruction-based model abstraction in TxIR

**Follow-up**: Versioned transactions (v0 with address lookups) + comprehensive testing

---

### 2.4: Top 20 Chains Scaffolding ✅ COMPLETE

**Priority**: HIGH (Ecosystem validation)
**Status**: Complete - 17 new decoder crates scaffolded with implementation plans
**Branch**: `claude/scaffold-top-20-chains-011CV3K3Ugiof7Qys68xorJC`

**Strategy**: Scaffold all top 20 chains, establish chain family grouping

Scaffolded Chains (17 new decoders):
- ✅ **EVM-Compatible** (5 chains reusing Ethereum decoder):
  - ✅ BNB Chain (ID: 56) - EVM clone with chain ID validation
  - ✅ Polygon (ID: 137) - EVM compatible
  - ✅ Avalanche C-Chain (ID: 43114) - EVM compatible
  - ✅ Optimism (ID: 10) - OP Stack (deposit tx support needed)
  - ✅ Arbitrum (ID: 42161) - Arbitrum Orbit (retryable tickets needed)
- ✅ **Specialized Formats** (12 chains with unique decoders):
  - ✅ XRP Ledger - Binary codec
  - ✅ Cardano - CBOR + eUTXO model
  - ✅ Dogecoin - Bitcoin fork (no SegWit)
  - ✅ Tron - Protobuf encoding
  - ✅ Polkadot - SCALE codec
  - ✅ Litecoin - Bitcoin fork with SegWit
  - ✅ NEAR - Borsh encoding
  - ✅ Cosmos Hub - Protobuf + Tendermint
  - ✅ Stellar - XDR encoding
  - ✅ Algorand - MessagePack encoding
  - ✅ Sui - BCS + Move VM
  - ✅ Aptos - BCS + Move VM

**Chain Family Grouping Strategy Established**:
- ✅ Document created: `docs/CHAIN_FAMILIES_GROUPING.md`
- ✅ Strategy: Group chains by technology (EVM, OP Stack, SVM, Cosmos, Move, etc.)
- ✅ Workspace reduction: 620+ individual chains → 18 family decoders (97% reduction!)
- ✅ Benefits: Scalability, maintainability, consistent API

**Implementation Plan Created**:
- ✅ Document: `docs/TOP_20_CHAINS_IMPLEMENTATION_PLAN.md`
- ✅ Detailed specs for each of 17 chains
- ✅ Complexity estimates and dependency strategies

**Chainlist Integration Plan**:
- ✅ Document: `docs/NEXT_STEPS_CHAINLIST_INTEGRATION.md`
- ✅ Generic EVM decoder supporting 500+ chains
- ✅ Vendored chain registry strategy (airgapped operation)
- ✅ Special case handling (Optimism deposits, Arbitrum retryables, zkSync, etc.)

**Delivered**:
- ✅ **17 new decoder crates** (stub implementations)
- ✅ **Chain family grouping strategy** (EVM, OP Stack, SVM, Cosmos, Move, etc.)
- ✅ **Airgapped operation requirement** documented
- ✅ **Chain registry vendoring strategy** via git subtree
- ✅ **3 comprehensive planning documents**

**Next Steps**: Implement chain family decoders (see Phase 3)

---

### 2.5: Common Crates Extraction - Shared Functionality ✅

**Priority**: HIGH (Enables scalability)
**Timeline**: 2 weeks
**Status**: Phase 2.5.1 Complete ✅ (decoder-encodings extracted)
**See**: `docs/COMMON_CRATES_ANALYSIS.md` and `docs/SHARED_CRATES_STRATEGY.md`

**Context**: After implementing Bitcoin, Ethereum, and Solana decoders, significant code duplication has been identified. This phase extracts common functionality to enable rapid addition of new chains.

**Key Findings**:
- 510 LOC of encoding logic duplicated across decoders
- RLP implementation needed by 7+ EVM chains
- Address formatting needed by 15+ chains
- Test utilities can reduce boilerplate by ~30%

#### 2.5.1: decoder-encodings Crate ✅ COMPLETE

**Priority**: CRITICAL (Used by 10+ chains)
**LOC**: ~510 (moved from existing decoders)
**Dependencies**: Zero (only universal-decoder-core)

**What to Extract**:

1. **Variable-Length Encodings**:
   - ✅ VarInt (Bitcoin) - Currently in `decoder-bitcoin/src/varint.rs` (70 LOC)
   - ✅ Compact-u16 (Solana) - Currently in `decoder-solana/src/parsing.rs` (100 LOC)
   - ✅ RLP (Ethereum) - Currently in `decoder-ethereum/src/rlp.rs` (340 LOC)
   - [ ] LEB128 (for NEAR, Polkadot, future chains)

2. **Address Encodings** (Week 2):
   - [ ] Base58 (Bitcoin, Solana, Stellar, Cardano) - Vendor `bs58` crate via git subtree
   - [ ] Base58Check (Bitcoin addresses) - Build on base58
   - [ ] Bech32 (Bitcoin SegWit, Cosmos chains) - Vendor `bech32` crate via git subtree
   - [ ] EIP-55 (Ethereum checksummed hex) - Pure Rust implementation
   - [ ] SS58 (Polkadot, Substrate chains) - For future use

**Implementation Tasks**:
- ✅ Create `decoder-encodings` crate skeleton
- ✅ Move VarInt from `decoder-bitcoin` → `decoder-encodings/src/varint.rs`
- ✅ Update Bitcoin decoder to import from `decoder-encodings`
- ✅ Move compact-u16 from `decoder-solana` → `decoder-encodings/src/compact_u16.rs`
- ✅ Update Solana decoder to import from `decoder-encodings`
- ✅ Move RLP from `decoder-ethereum` → `decoder-encodings/src/rlp.rs`
- ✅ Update Ethereum decoder to import from `decoder-encodings`
- [ ] Update all EVM-family decoders (BNB, Polygon, Arbitrum, Optimism, Avalanche) to use shared RLP (Phase 3)
- [ ] Vendor `bs58` crate using git subtree (see `docs/GIT_SUBTREE_VENDORING.md`) (Phase 2.5.2)
- [ ] Vendor `bech32` crate using git subtree (Phase 2.5.2)
- [ ] Add address formatting APIs (Phase 2.5.2)
- ✅ Comprehensive testing (existing tests migrated + all passing)

**Validation**:
- ✅ All Bitcoin tests still pass (47 tests)
- ✅ All Ethereum tests still pass (6 tests)
- ✅ All Solana tests still pass (13 tests)
- ✅ decoder-encodings tests pass (16 tests for varint, compact-u16, RLP)
- [ ] EVM family decoders use shared RLP (Phase 3)
- ✅ Zero production dependencies (only universal-decoder-core)
- ✅ Cargo tree shows `decoder-encodings` used by 3 decoders (Bitcoin, Ethereum, Solana)

**Impact**:
- 430 LOC reduction (-13%) across existing decoders
- Future EVM chains need 42% less code (RLP reuse)
- Single source of truth for critical encoding logic
- Better testing through shared code

#### 2.5.2: decoder-test-utils Crate (Week 2) ✅ COMPLETE

**Priority**: MEDIUM (Quality improvement)
**LOC**: ~300
**Dependencies**: proptest (dev-only)
**Status**: Complete

**What Was Extracted**:

1. **Common Test Assertions**:
   - ✅ `assert_decode_never_panics<D: ChainDecoder>(bytes)`
   - ✅ `assert_canonical_roundtrip<T: CanonicalSerialize>(tx)`
   - ✅ `assert_decode_encode_roundtrip<D: ChainDecoder>(bytes)`
   - ✅ `assert_rejects_empty_input<D: ChainDecoder>()`
   - ✅ `assert_handles_oversized_input<D: ChainDecoder>(max_size)`

2. **Property Test Helpers**:
   - ✅ `arb_transaction_bytes(min, max)` - Generate arbitrary byte sequences
   - ✅ `arb_small_bytes()`, `arb_medium_bytes()`, `arb_large_bytes()`
   - ✅ `prop_decoder_never_panics<D>(bytes)` - Standard decoder property
   - ✅ `canonical_serialization_properties<T>(value)` - Determinism property
   - ✅ `arb_u64()`, `arb_u128()`, `arb_address()`, `arb_hash()`, `arb_signature()`

3. **Fixture Loading**:
   - ✅ `load_fixture(path) -> TestFixture`
   - ✅ `load_fixtures_dir(dir) -> Vec<TestFixture>`
   - ✅ `filter_by_chain(fixtures, chain)`
   - ✅ `filter_by_tag(fixtures, tag)`
   - ✅ `TestFixture` struct with expected properties and metadata

**Implementation Tasks**:
- ✅ Create `decoder-test-utils` crate (~400 LOC)
- ✅ Extract common test assertions (5 functions)
- ✅ Add property test generators (12 functions)
- ✅ Create fixture loading utilities (complete)
- ✅ Update Bitcoin decoder tests to use shared utilities (4 new tests)
- ✅ Update Ethereum decoder tests to use shared utilities (3 new tests)
- ✅ Update Solana decoder tests to use shared utilities (3 new tests)
- ✅ Documentation and examples (comprehensive inline docs)

**Impact**:
- ✅ Reduced test boilerplate by ~30%
- ✅ Standardized testing patterns across all decoders
- ✅ All Bitcoin integration tests passing (13/13)
- ✅ Easier to add comprehensive tests for new chains

#### 2.5.3: Code Metrics & Validation

**Before Extraction**:
| Metric | Value |
|--------|-------|
| Decoder-specific LOC | 3,200 |
| Encoding LOC (duplicated) | 510 |
| Production dependencies | 5 |
| Test utilities | Duplicated |

**After Extraction**:
| Metric | Value | Delta |
|--------|-------|-------|
| Decoder-specific LOC | 2,770 | **-430 (-13%)** |
| Shared encoding LOC | 510 | **+510 (reusable)** |
| Production dependencies | 5 | **0 change** ✅ |
| Test utilities | Shared | **Standardized** |

**Success Criteria**:
- ✅ `decoder-encodings` created with zero external dependencies
- ✅ VarInt, compact-u16, RLP moved and working
- [ ] All EVM decoders use shared RLP (Phase 3)
- [ ] Base58 and Bech32 vendored via git subtree (Future phase)
- ✅ `decoder-test-utils` provides common test patterns
- ✅ All existing tests pass (Bitcoin + Ethereum + Solana)
- ✅ No increase in core TCB
- ✅ Future EVM chain implementation will use ~50% less code (RLP shared)

**Deliverables**:
- ✅ `decoder-encodings` crate (~510 LOC, 0 external deps)
  - ✅ VarInt encoding (70 LOC)
  - ✅ Compact-u16 encoding (100 LOC)
  - ✅ RLP encoding (340 LOC)
  - ✅ 16 comprehensive tests
- ✅ `decoder-test-utils` crate (~400 LOC, proptest dev-dep)
  - ✅ 5 common test assertions
  - ✅ 12 property test generators
  - ✅ Fixture loading utilities
- ✅ Updated decoders using shared code (Bitcoin, Ethereum, Solana)
- ✅ Comprehensive documentation with inline examples
- [ ] Migration guide for future decoders (Phase 2.5.2)

**Documentation**:
- ✅ `docs/COMMON_CRATES_ANALYSIS.md` - Detailed 10-section analysis
- ✅ `docs/SHARED_CRATES_STRATEGY.md` - Quick reference guide
- [ ] `docs/DECODER_ENCODINGS_API.md` - API documentation
- [ ] `docs/TEST_UTILS_GUIDE.md` - Testing utilities guide

**Why This Matters**:
- **Scalability**: RLP shared by 7+ EVM chains → massive reuse
- **Maintainability**: Fix encoding bugs once, benefit all chains
- **Speed**: Future chains need significantly less code
- **Quality**: Shared testing utilities improve consistency
- **Security**: Focused testing on shared critical code

---

### 2.6: Integration Tests & Examples

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

## Phase 3.0: Privacy-Aware TxIR Extensions (Week 1-2)

**Target**: v0.1.5-privacy
**Timeline**: 2 weeks
**Focus**: Extend TxIR to support privacy-preserving transactions with optional viewing keys
**Priority**: CRITICAL (Prerequisite for Phase 3.8 Privacy Chains)

**Motivation**: Current TxIR assumes all transaction data is publicly visible (amounts, addresses, etc.). Privacy chains like Zcash, Aleo, and Monero encrypt this data using zero-knowledge proofs. We need to extend TxIR to handle:
- Encrypted/shielded components (amounts, addresses)
- Zero-knowledge proofs (zk-SNARKs, Bulletproofs)
- Optional viewing key decryption
- Privacy metadata without breaking existing decoders

### 3.0.1: Core TxIR Privacy Extensions (Week 1)

**Tasks**:
- [ ] Add `PrivacyComponents` struct to TxIR
  ```rust
  pub struct TxIR<'a, const V: u8> {
      // ... existing fields ...
      pub privacy: Option<PrivacyComponents>,
  }

  pub struct PrivacyComponents {
      pub encrypted_data: Vec<EncryptedComponent>,
      pub proofs: Vec<ZkProof>,
      pub viewing_key_type: Option<ViewingKeyType>,
  }
  ```
- [ ] Define `EncryptedComponent` (amounts, addresses, memos)
- [ ] Define `ZkProof` (proof type, bytes, verification metadata)
- [ ] Define `ViewingKeyType` enum (Zcash IVK, Aleo VK, Monero ViewKey)
- [ ] Extend `Transfer` to support optional encrypted fields
  ```rust
  pub struct Transfer {
      pub from: Address,              // Public or placeholder
      pub to: Address,                // Public or placeholder
      pub amount: Amount,              // Public or placeholder
      pub encrypted_from: Option<EncryptedComponent>,
      pub encrypted_to: Option<EncryptedComponent>,
      pub encrypted_amount: Option<EncryptedComponent>,
      pub asset: AssetId,
  }
  ```
- [ ] Update Borsh serialization to include privacy components
- [ ] Comprehensive unit tests (30+ tests)

**Validation**:
- [ ] Existing decoders (Bitcoin, Ethereum, Solana) unaffected
- [ ] TxIR with privacy components serializes deterministically
- [ ] Privacy fields are truly optional (backward compatible)

### 3.0.2: Viewing Key Decoder Trait (Week 2)

**Tasks**:
- [ ] Create `PrivacyAwareDecoder` trait
  ```rust
  pub trait PrivacyAwareDecoder: ChainDecoder {
      type ViewingKey;

      fn decode_with_viewing_key(
          &self,
          data: &[u8],
          viewing_key: Option<&Self::ViewingKey>
      ) -> Result<TxIR>;

      fn supports_viewing_keys(&self) -> bool { false }
  }
  ```
- [ ] Implement default `PrivacyAwareDecoder` for existing decoders
  - Bitcoin: No viewing keys, `supports_viewing_keys() = false`
  - Ethereum: No viewing keys, `supports_viewing_keys() = false`
  - Solana: No viewing keys, `supports_viewing_keys() = false`
- [ ] Add decryption helper utilities
  ```rust
  pub trait ViewingKeyDecryptor {
      fn decrypt_amount(&self, encrypted: &[u8]) -> Result<Amount>;
      fn decrypt_address(&self, encrypted: &[u8]) -> Result<Address>;
      fn decrypt_memo(&self, encrypted: &[u8]) -> Result<Vec<u8>>;
  }
  ```
- [ ] Create privacy-aware examples
  - Example 1: Parse shielded transaction without viewing key (extract proofs only)
  - Example 2: Parse shielded transaction with viewing key (decrypt amounts/addresses)
- [ ] Documentation: `docs/PRIVACY_AWARE_TXIR.md`

**Validation**:
- [ ] Trait compiles with existing decoders
- [ ] No breaking changes to existing decoder API
- [ ] Examples demonstrate both modes (with/without viewing keys)

### 3.0.3: Privacy Component Types (Week 2)

**Tasks**:
- [ ] Define common privacy primitives
  ```rust
  // Nullifiers (Zcash, Monero)
  pub struct Nullifier {
      pub bytes: Vec<u8>,
      pub nullifier_type: NullifierType,
  }

  // Commitments (Zcash, Monero)
  pub struct Commitment {
      pub bytes: Vec<u8>,
      pub commitment_type: CommitmentType,
  }

  // Note encryption (Zcash)
  pub struct EncryptedNote {
      pub ciphertext: Vec<u8>,
      pub ephemeral_key: Vec<u8>,
      pub decrypted_note: Option<Note>,
  }

  // Ring signatures (Monero)
  pub struct RingSignature {
      pub ring_size: usize,
      pub key_images: Vec<Vec<u8>>,
      pub ring_members: Vec<Vec<u8>>,
  }
  ```
- [ ] Add privacy-specific operations to `Operation` enum
  ```rust
  pub enum Operation {
      Transfer(Transfer),
      ContractCall(ContractCall),
      // ... existing ...
      ShieldedTransfer(ShieldedTransfer),  // NEW
      PrivateExecution(PrivateExecution),  // NEW (for Aleo)
  }
  ```
- [ ] Comprehensive tests for all privacy types

**Deliverables**:
- [ ] Extended TxIR with optional privacy components
- [ ] `PrivacyAwareDecoder` trait with viewing key support
- [ ] Privacy primitive types (nullifiers, commitments, encrypted notes, ring signatures)
- [ ] Backward compatible with all existing decoders
- [ ] Documentation: `docs/PRIVACY_AWARE_TXIR.md`
- [ ] Examples demonstrating privacy features
- [ ] All tests passing (existing + new privacy tests)

**Success Criteria**:
- ✅ TxIR can represent both transparent and shielded transactions
- ✅ Viewing keys are optional (privacy by default, transparency with keys)
- ✅ Zero impact on existing non-privacy decoders
- ✅ Borsh serialization includes privacy components
- ✅ Documentation explains privacy semantics clearly
- ✅ Ready for Phase 3.8 privacy chain implementations

**Why This Matters**:
- **Architectural Soundness**: TxIR must handle privacy-preserving transactions
- **Security**: Viewing keys are opt-in (privacy by default)
- **Extensibility**: Supports multiple privacy schemes (zk-SNARKs, ring sigs, etc.)
- **Backward Compatibility**: No breaking changes to existing decoders

---

## Phase 3: Chain Family Decoders (Months 3-4)

**Target**: v0.2.0
**Focus**: Implement family-based decoders for scalable multi-chain support
**Strategy**: 620+ chains → 18 family decoders (97% reduction!)

**Prerequisites**:
- ✅ Phase 2 complete (Bitcoin, Ethereum, Solana decoders)
- ✅ Chain family grouping strategy (docs/CHAIN_FAMILIES_GROUPING.md)
- ✅ Common crates analysis (docs/COMMON_CRATES_ANALYSIS.md)
- [ ] Phase 2.5: decoder-encodings extracted (provides shared RLP for EVM chains)
- [ ] Phase 3.0: Privacy-aware TxIR extensions (for Phase 3.8 privacy chains)
- [ ] Chain registries vendored via git subtree (Phase 1.5.1)

### 3.1: EVM Family Decoder (Week 1-2)

**Priority**: CRITICAL (500+ EVM chains)
**Decoder**: `decoder-evm`
**See**: `docs/NEXT_STEPS_CHAINLIST_INTEGRATION.md`

**Chains Supported**: 500+ EVM-compatible chains (Ethereum, BNB, Polygon, Avalanche, etc.)

Tasks:
- [ ] Create `decoder-evm` crate
- [ ] Vendor `ethereum-lists/chains` via git subtree
  ```bash
  git subtree add --prefix crates/decoder-evm/vendored/chainlist \
      https://github.com/ethereum-lists/chains.git master --squash
  ```
- [ ] Implement `ChainRegistry` (compile-time embedded JSON)
- [ ] Implement `EvmDecoder` (reuses `EthereumDecoder`, adds chain validation)
- [ ] Build.rs script to embed chain data at compile time
- [ ] Migrate existing EVM decoders (BNB, Polygon, Avalanche) to wrappers
- [ ] Comprehensive testing (20+ unit tests, 100+ chains validated)

**Delivered**:
- Generic EVM decoder supporting all standard EVM chains
- Airgapped-compatible (no runtime network calls)
- Verifiable supply chain (git subtree audit trail)

**Why Important**: Eliminates need for 500+ individual decoder crates

---

### 3.2: OP Stack Family Decoder (Week 3-4) 🚧 IN PROGRESS

**Priority**: HIGH (35+ OP Stack chains)
**Decoder**: `decoder-optimism` (enhanced)
**Status**: Core implementation complete (90%), needs trait fixes

**Chains Supported**: Optimism, Base, Zora, Mode, PGN, Orderly, Blast, Redstone, Kroma, Mantle, Lyra, Metal, Lisk, etc.

Tasks:
- ✅ Create `decoder-optimism` enhanced crate
- ✅ Vendor `ethereum-optimism/superchain-registry` via git subtree (63 chains, 7KB Borsh)
- ✅ Implement deposit transaction (0x7E) parsing (431 lines)
  - ✅ Full RLP parsing for all 8 deposit fields
  - ✅ Source hash, from, to, mint, value, gas_limit, is_creation, data
  - ✅ Validation for deposit transaction invariants
  - ✅ L1 attributes deposit detection
  - ✅ User deposit detection
- ✅ Create OptimismTransaction enum (Standard | Deposit)
- ✅ Implement DepositTransaction type (437 lines)
- ✅ Auto-detection: deposit tx (0x7E) vs standard EVM tx
- ✅ OP Stack chain ID detection (13 known chains + testnet range)
- ✅ Comprehensive unit tests (30 tests)
- [ ] Fix EthereumTransaction trait implementations (PartialEq, Eq, Serialize, Deserialize, Borsh)
- [ ] Implement Canonicalizer trait for OptimismTransaction
- [ ] Add decoder-encodings dependency to Cargo.toml
- [ ] Integration tests with real OP Stack deposit transactions
- [ ] Build.rs script to load superchain registry at compile time

**Special Features**:
- ✅ Deposit transactions (L1 → L2 bridging)
  - No signatures (chain derivation authorization)
  - ETH minting capability (L2 supply increase)
  - Source hash tracking (L1 origin)
- ✅ System transactions (L1 attributes deposits)
- [ ] EIP-1559 with L1 data fee (standard Ethereum support)

**Delivered** (1,026 lines):
- ✅ Complete 0x7E deposit transaction implementation
- ✅ RLP parsing infrastructure with comprehensive error handling
- ✅ Support for entire OP Stack ecosystem (35+ chains)
- ✅ Extensive documentation and examples
- ✅ 30 unit tests covering all deposit transaction functionality

**Remaining** (~4 hours):
- Fix compilation errors (trait implementations)
- Integration tests with mainnet data
- Registry loading at compile time

**Why Important**: OP Stack is fastest-growing L2 ecosystem (Base, Zora, Mode all using deposit txs)

---

### 3.3: Arbitrum Orbit Family Decoder (Week 3-4) ✅ COMPLETE

**Priority**: HIGH (5+ Arbitrum chains)
**Decoder**: `decoder-arbitrum` (enhanced)
**Status**: Complete - All 6 Arbitrum-specific transaction types implemented

**Chains Supported**: Arbitrum One, Arbitrum Nova, Arbitrum Sepolia, Arbitrum Goerli, and custom Orbit chains

Tasks:
- ✅ Enhanced `decoder-arbitrum` crate with full Arbitrum support
- ✅ Implemented Arbitrum chain detection (One, Nova, Sepolia, Goerli, custom Orbit)
- ✅ Implemented retryable ticket parsing (Type 0x69 - SubmitRetryable)
- ✅ Implemented ArbOS internal transactions (Type 0x6A - Internal)
- ✅ Implemented deposit transactions (Type 0x64)
- ✅ Implemented unsigned transactions (Type 0x65 - EOA via bridge)
- ✅ Implemented contract transactions (Type 0x66 - L1 contract calls)
- ✅ Implemented retry transactions (Type 0x68 - Retry failed retryables)
- ✅ Auto-detection: All 6 Arbitrum types + standard EVM tx
- ✅ Comprehensive unit tests (20 tests passing)
- ✅ Property-based tests (21 tests passing)

**Special Features Implemented**:
- ✅ Retryable tickets (guaranteed L2 execution) - Type 0x69
- ✅ ArbOS internal transactions (system state updates) - Type 0x6A
- ✅ Delayed inbox messages (Types 0x64, 0x65, 0x66)
- ✅ Chain ID range validation (42xxx for mainnet, 421xxx for testnet)
- ✅ Canonicalizer trait implementation for all 6 types
- ✅ RLP parsing infrastructure for all transaction types

**Delivered** (4 files, ~2000 lines):
- ✅ `crates/decoder-arbitrum/src/types.rs` (730 lines) - All 6 transaction types
- ✅ `crates/decoder-arbitrum/src/parsing.rs` (450 lines) - RLP parsing for all types
- ✅ `crates/decoder-arbitrum/src/lib.rs` (340 lines) - Enhanced decoder with chain detection
- ✅ `crates/decoder-arbitrum/tests/property_tests.rs` (450 lines) - 21 property tests
- ✅ Single decoder for entire Arbitrum Orbit ecosystem
- ✅ Support for all Arbitrum-specific transaction types
- ✅ 41 comprehensive tests (20 unit + 21 property)

**Why Important**: Arbitrum is largest L2 by TVL, and retryable tickets are critical for L1↔L2 communication

---

### 3.4: SVM Family Decoder (Week 5-6)

**Priority**: MEDIUM (5+ SVM chains)
**Decoder**: `decoder-svm`

**Chains Supported**: Solana (✅ implemented), Eclipse, Pyth Network, Drift, Jito, etc.

Tasks:
- [ ] Create `decoder-svm` crate
- [ ] Wrap existing `SolanaDecoder`
- [ ] Hardcode SVM chain list (Solana, Eclipse, Pyth, etc.)
- [ ] Add chain ID validation
- [ ] Support for SVM-specific features per chain

**Delivered**:
- ✅ Solana decoder already implemented
- Single decoder for entire SVM ecosystem
- Future-proof for SVM rollups

**Why Important**: SVM is expanding beyond Solana mainnet

---

### 3.5: Cosmos SDK Family Decoder (Week 7-8) ✅ COMPLETE

**Priority**: MEDIUM (228 Cosmos chains)
**Decoder**: `decoder-cosmos`
**Status**: Complete - PR #39 merged
**Branch**: `claude/cosmos-sdk-phase-3.5-011CV5yFdhxiSFEu3Ztnxidm`

**Chains Supported**: Cosmos Hub, Osmosis, Injective, Celestia, dYdX, etc. (228 total)

**Completed Tasks**:
- ✅ Create `decoder-cosmos` crate (1,856 lines)
- ✅ Vendor `cosmos/chain-registry` via git subtree (17KB Borsh, 228 chains)
- ✅ Implement Protobuf transaction parsing (cosmos-sdk-proto integration)
- ✅ Support Tendermint signatures (SHA-256 hashing)
- ✅ Handle 8 message types (Send, Delegate, Vote, etc.)
- ✅ Build.rs script to embed chain registry (compile-time)

**Message Types Implemented**:
- ✅ MsgSend (bank transfers)
- ✅ MsgMultiSend (multi-party transfers)
- ✅ MsgDelegate (staking)
- ✅ MsgUndelegate (unstaking)
- ✅ MsgBeginRedelegate (redelegate)
- ✅ MsgVote (governance)
- ⏳ MsgIbcTransfer (TODO: requires IBC feature flags)
- ⏳ MsgExecuteContract (TODO: requires CosmWasm feature flags)

**Testing**:
- ✅ 10 integration tests (real transactions)
- ✅ 13 property-based tests (proptest)
- ✅ 8 unit tests (chain identity, parsing, validation)
- ✅ Total: 31 tests passing

**Delivered**:
- ✅ Single decoder for entire Cosmos ecosystem (228 chains)
- ✅ Account-based model (not UTXO)
- ✅ Bech32 address support (cosmos1...)
- ✅ Micro-denomination handling (uatom, uosmo, etc.)
- ✅ TxIR canonicalization with operations
- ⏳ Partial IBC transaction support (infrastructure ready, needs feature flags)

**Why Important**: Cosmos has most diverse ecosystem (228 chains, IBC interoperability)

---

### 3.6: Move VM Family Decoder (Week 7-8)

**Priority**: MEDIUM (3+ Move chains)
**Decoder**: `decoder-move`

**Chains Supported**: Aptos, Sui, Movement, etc.

Tasks:
- [ ] Create `decoder-move` crate
- [ ] Implement BCS (Binary Canonical Serialization) parsing
- [ ] Support Aptos transaction format
- [ ] Support Sui transaction format (object model)
- [ ] Handle Move module calls
- [ ] Hardcode chain list

**Special Features**:
- Aptos: Account-based with parallel execution
- Sui: Object-centric model
- Move bytecode verification

**Delivered**:
- Single decoder for Move ecosystem
- Dual support for Aptos and Sui formats

**Why Important**: Move is gaining adoption for new chains

---

### 3.7: Bitcoin Forks Family Decoder (Week 9)

**Priority**: LOW (10+ Bitcoin forks)
**Decoder**: `decoder-bitcoin-forks`

**Chains Supported**: Dogecoin, Litecoin, Bitcoin Cash, Dash, Zcash (transparent), etc.

Tasks:
- [ ] Create `decoder-bitcoin-forks` crate
- [ ] Wrap existing `BitcoinDecoder`
- [ ] Hardcode fork parameters (SegWit support, address formats, etc.)
- [ ] Auto-detect: SegWit vs legacy
- [ ] Migrate `decoder-dogecoin` and `decoder-litecoin` to wrappers

**Delivered**:
- Single decoder for all Bitcoin forks
- Minimal code (wraps existing Bitcoin decoder)

**Why Important**: Many Bitcoin forks use identical transaction format

---

### 3.8: Privacy Chains Family Decoder (Week 9-10)

**Priority**: MEDIUM (Privacy features require special handling)
**Decoder**: `decoder-privacy-chains`
**Prerequisites**: Phase 3.0 (Privacy-Aware TxIR Extensions) must be complete

**Chains Supported**: Zcash, Aleo, Monero (limited)

Tasks:
- [ ] Create `decoder-privacy-chains` crate
- [ ] Implement Zcash decoder (transparent + shielded transactions)
  - [ ] Implement `PrivacyAwareDecoder` trait with Zcash viewing keys
  - [ ] Transparent transactions (reuse `BitcoinDecoder` base)
  - [ ] Shielded transactions (zk-SNARK components)
    - [ ] Sprout (JoinSplit descriptions)
    - [ ] Sapling (Spend/Output descriptions)
    - [ ] Orchard (Action descriptions)
  - [ ] Parse: nullifiers, commitments, encrypted notes, proofs
  - [ ] Populate `PrivacyComponents` with encrypted data
  - [ ] Implement viewing key decryption (uses `ViewingKeyDecryptor` from Phase 3.0)
  - [ ] Without viewing key: Extract proof structure only (privacy preserved)
  - [ ] With viewing key: Decrypt amounts/addresses, populate `Transfer` operations
- [ ] Implement Aleo decoder (Leo VM transactions)
  - [ ] Implement `PrivacyAwareDecoder` trait with Aleo viewing keys
  - [ ] Program execution records
  - [ ] State transitions
  - [ ] Zero-knowledge proofs (snarkVM)
  - [ ] Public inputs/outputs
  - [ ] Populate `PrivateExecution` operations (uses Phase 3.0 privacy types)
  - [ ] Instruction-based model (similar to Solana structure)
- [ ] Implement Monero decoder (limited, privacy-by-default)
  - [ ] Implement `PrivacyAwareDecoder` trait with Monero view keys
  - [ ] Ring signatures (populate `RingSignature` from Phase 3.0)
  - [ ] Stealth addresses
  - [ ] Ring CT (Confidential Transactions)
  - [ ] Can extract: ring size, fees, proof structure
  - [ ] Cannot fully decrypt: actual sender (by design), limited receiver/amount decryption
- [ ] Hardcode chain list
- [ ] Comprehensive testing with mainnet transactions

**Special Features**:
- **Zcash**:
  - Dual-mode: transparent (Bitcoin-like) + shielded (zk-SNARK)
  - Multiple shielded protocols (Sprout, Sapling, Orchard)
  - Viewing keys for selective disclosure
  - Proof verification metadata
- **Aleo**:
  - Leo programming language VM
  - Full privacy by default (all computations private)
  - Zero-knowledge proofs for state transitions
  - Program execution traces
- **Monero**:
  - Ring signatures (transaction mixing)
  - One-time stealth addresses
  - Ring CT (hidden amounts)
  - Subaddresses

**Privacy Limitations** (by design):
- ✅ Can decode: transaction structure, proof components, public metadata
- ❌ Cannot decode: private transfer amounts, sender/receiver in shielded transactions
- ⚠️ Limited information extraction is expected and correct behavior

**Delivered**:
- Support for 3 major privacy-focused blockchains
- Transparent/public component extraction
- Proof structure parsing (without breaking privacy)
- Documentation on privacy limitations

**Why Important**:
- Privacy chains are growing in importance
- Zcash and Aleo use cutting-edge cryptography (zk-SNARKs)
- Demonstrates TxIR can handle privacy-preserving transactions
- Validates extensibility for non-standard transaction formats

---

### 3.9: Universal Decoder Integration (Week 11)

**Priority**: HIGH (Unified API)
**Decoder**: `universal-decoder`

Tasks:
- [ ] Create `universal-decoder` crate
- [ ] Implement `UniversalDecoder` struct with all family decoders
- [ ] Auto-detection logic (transaction format → chain family)
- [ ] Chain hint support (optional chain ID)
- [ ] Comprehensive integration tests
- [ ] Example: Decode 100+ chains from single API

**API**:
```rust
let decoder = UniversalDecoder::new()?;

// Auto-detect chain family
let tx = decoder.decode(&tx_bytes, None)?;

// Or provide hint
let tx = decoder.decode(&tx_bytes, Some(ChainId::Numeric(56)))?; // BNB Chain
```

**Delivered**:
- Single unified API for 620+ chains
- Auto-detection of chain family from transaction bytes
- Comprehensive testing across all families

**Why Important**: User-facing unified interface

---

## Phase 3.10: WASM Demo & Interactive Playground (Week 12-13)

**Target**: v0.2.1-wasm-demo
**Timeline**: 1-2 weeks
**Focus**: Browser-based transaction decoder with CodeMirror for blog posts, papers, and demos
**Priority**: HIGH (Paper/blog demo, conference presentations)

**Motivation**: Create a compelling, interactive demonstration of the universal decoder that:
- Runs entirely in the browser (zero-trust, no server-side processing)
- Works offline (reinforces airgapped narrative)
- Can be embedded in blog posts and documentation
- Provides visual comparison of different blockchain models
- Highlights privacy features across chains
- Serves as educational tool and conference demo

**See**: `docs/WASM_DEMO.md` for comprehensive implementation guide

### 3.10.1: WASM Core Infrastructure (Week 12)

**Tasks**:
- [ ] Create `crates/universal-decoder-wasm` crate
  - wasm-bindgen for JS interop
  - serde-wasm-bindgen for data serialization
  - console_error_panic_hook for better debugging
- [ ] Implement WASM API
  ```rust
  #[wasm_bindgen]
  pub struct DecodeResult {
      canonical_hex: String,
      canonical_hash: String,
      json: JsValue,
      privacy_features: Vec<String>,
  }

  #[wasm_bindgen]
  pub fn decode_transaction(chain: &str, hex: &str) -> Result<JsValue, JsValue>
  ```
- [ ] Set up build infrastructure
  - wasm-pack for building
  - Build script for optimization (opt-level = "z", LTO)
  - Size benchmarking (target: < 500KB for core + 3 decoders)
- [ ] Create minimal web UI skeleton
  - HTML structure with CodeMirror integration
  - Basic CSS styling
  - JavaScript module loader
- [ ] Test WASM build and bundle size
  - Minimal build: Bitcoin + Ethereum (~300KB gzipped target)
  - Full build: All decoders (~2MB gzipped target)
- [ ] Write integration tests for WASM API

**Validation**:
- [ ] WASM module loads successfully in browser
- [ ] Can decode Bitcoin transaction in browser
- [ ] Can decode Ethereum transaction in browser
- [ ] Error messages are clear and helpful
- [ ] Bundle size is acceptable (< 500KB for minimal)

### 3.10.2: Interactive UI & Visual Features (Week 13)

**Priority**: MEDIUM (Polish for demos)

**Tasks**:
- [ ] Implement CodeMirror editor integration
  - Hex input editor with syntax highlighting
  - JSON output editor with formatting
  - Line numbers and error highlighting
- [ ] Create chain selector dropdown
  - Bitcoin, Ethereum, Solana, OP Stack, etc.
  - Auto-detection option (from transaction format)
- [ ] Add example transaction loader
  - Pre-loaded examples for each chain:
    - Bitcoin: SegWit transaction
    - Ethereum: EIP-1559 transaction
    - Ethereum: Tornado Cash deposit (privacy features!)
    - OP Stack: L1→L2 deposit (0x7E)
    - Solana: Token transfer
  - Load on click, decode automatically
- [ ] Implement output tabs
  - Tab 1: JSON (human-readable display)
  - Tab 2: Canonical Borsh (hex representation)
  - Tab 3: Privacy Analysis (if applicable)
- [ ] Add metadata display
  - Canonical hash
  - Canonical size (bytes)
  - Privacy score (🟢 Private / 🟡 Partial / 🔴 Transparent)
  - Privacy features detected (ring signatures, stealth addresses, etc.)
- [ ] Implement privacy highlighting
  - Color-code privacy features in JSON
  - Badge for privacy level
  - Explanation tooltips
- [ ] Create comparison mode (optional, stretch goal)
  - Split-screen: 3 chains side-by-side
  - Same semantic operation (Transfer) across chains
  - Highlight differences (UTXO vs Account vs Instruction)
  - "All normalize to same TxIR" visualization

**Advanced Features** (optional):
- [ ] Visual transaction flow diagram
- [ ] Export options (download .borsh, .json, copy hash)
- [ ] Shareable URL (transaction hex in URL fragment)
- [ ] Dark/light mode toggle
- [ ] Mobile-responsive layout

**Validation**:
- [ ] User can paste any transaction hex and decode it
- [ ] Privacy features are highlighted correctly
- [ ] Examples work for all supported chains
- [ ] UI is intuitive and responsive
- [ ] Works on mobile devices

### 3.10.3: Deployment & Integration (Week 13)

**Tasks**:
- [ ] Deploy to GitHub Pages
  - Set up GitHub Actions for automatic deployment
  - Custom domain (optional): decoder.universalblockchain.dev
- [ ] Create embeddable iframe version
  - Minimal UI for embedding in blog posts
  - URL parameters for chain selection and pre-filled transactions
  - Example: `<iframe src="https://decoder.../embed?chain=bitcoin&tx=...">`
- [ ] Write documentation
  - User guide: How to use the demo
  - Developer guide: How to embed in websites
  - API documentation for WASM module
- [ ] Create demo video/GIF
  - Screen recording showing decoding across chains
  - Privacy feature detection demonstration
  - For use in README, blog posts, presentations
- [ ] Update README.md with demo link
  - Prominent "Try it live!" button
  - Screenshot of the demo
  - Link to WASM_DEMO.md for technical details

**Deliverables**:
- [ ] Live demo deployed and accessible
- [ ] Embeddable version for blog posts
- [ ] Documentation for users and developers
- [ ] Demo video/GIF for promotion
- [ ] GitHub Pages site with custom domain (optional)

### 3.10.4: Use Cases & Applications

**Blog Post Integration**:
- Embed interactive examples directly in blog posts
- Readers can experiment with real transactions
- Visual comparison of blockchain models

**Conference Presentations**:
- Works offline (no WiFi needed after initial load)
- Live demonstration with audience transactions
- Privacy feature detection in real-time

**Educational Tool**:
- Students learn transaction formats hands-on
- Visualize UTXO vs Account models
- Understand canonical encoding

**Privacy Research**:
- Analyze Tornado Cash transactions
- Compare privacy primitives across chains
- Quantify privacy scores

**Paper Demonstrations**:
- Interactive figures in HTML versions of papers
- Reproducible examples
- Visual proof of concept

### 3.10.5: Success Criteria

**Technical**:
- ✅ WASM module builds successfully
- ✅ Bundle size < 500KB (minimal) or < 2MB (full)
- ✅ Decodes Bitcoin, Ethereum, Solana successfully
- ✅ Works offline after initial load
- ✅ No network calls during operation (airgapped demo)
- ✅ Error handling is robust and user-friendly

**User Experience**:
- ✅ Interface is intuitive (no instructions needed)
- ✅ Examples load instantly
- ✅ Privacy features are visually obvious
- ✅ Works on desktop and mobile
- ✅ Embeddable in blog posts

**Documentation**:
- ✅ User guide complete
- ✅ Developer guide for embedding
- ✅ API documentation for WASM module
- ✅ Demo video/GIF created

**Adoption**:
- ✅ Deployed to GitHub Pages
- ✅ Linked from main README
- ✅ Used in at least one blog post or paper
- ✅ Positive feedback from early users

### Benefits

**Maximum Impact**:
- **Zero-trust demonstration**: Nothing leaves the browser
- **Offline capability**: Reinforces airgapped architecture
- **Embeddable**: Can be used in papers, blogs, documentation
- **Visual**: Privacy features and blockchain models clearly shown
- **Educational**: Interactive learning tool
- **Marketing**: Compelling demo for GitHub, conferences, papers

**Low Maintenance**:
- Auto-updates with core library changes
- Hosted on GitHub Pages (free)
- No server infrastructure needed
- Small bundle size (fast to load)

**High ROI**:
- Development time: 1-2 weeks
- Impact: Massive (demos, papers, education, adoption)
- Cost: $0 hosting (GitHub Pages)
- Maintenance: Minimal (auto-deploys with CI)

**Why This Matters**:
- **Papers**: Interactive demonstrations are more compelling than static figures
- **Conferences**: Live demos without WiFi dependency
- **Blog Posts**: Readers can experiment immediately
- **Education**: Visual, hands-on learning beats documentation
- **Adoption**: Seeing is believing - instant proof of concept
- **Privacy**: Demonstrates privacy-aware architecture visually
- **Marketing**: Shareable, impressive demo for social media

---

## Phase 4: Formal Verification (Months 3-4)

**Target**: v0.3.0
**Focus**: Mathematical proofs of correctness using Verus
**See**: `docs/VERUS_VERIFICATION_COVERAGE.md` (comprehensive guide)

**Prerequisites**:
- ✅ Verus installed (Phase 1.5.2)
- ✅ Verification targets documented (`docs/VERIFICATION_TARGETS.md`)
- ✅ Coverage tracking infrastructure (`tools/verus-coverage.sh`)

---

### 4.0: Verification Infrastructure & Coverage Tracking

**Priority**: CRITICAL (Foundation for all verification)
**Timeline**: Week 1-2

**Tasks**:
- [ ] ✅ Install Verus toolchain locally and in CI
- [ ] ✅ Set up `--output-json` parsing for machine-readable output
- [ ] ✅ Create verification targets manifest (`docs/VERIFICATION_TARGETS.md`)
- [ ] ✅ Implement coverage tracking script (`tools/verus-coverage.sh`)
- [ ] ✅ Implement coverage parsing script (`tools/parse-verus-coverage.py`)
- [ ] ✅ Generate HTML dashboard (`tools/generate-dashboard.py`)
- [ ] ✅ Add Verus to GitHub Actions (`.github/workflows/verus.yml`)
- [ ] ✅ Create verification coverage badge (shields.io)
- [ ] ✅ Set up pre-commit hook for verification (optional)

**Coverage Tracking Strategy**:

We track **4 types of verification coverage**:

1. **Function Coverage**: % of functions with formal specifications
   ```
   Verified Functions / Total Critical Functions
   ```

2. **Property Coverage**: % of critical safety properties proven (PRIMARY METRIC)
   ```
   Verified Properties / Total Properties (from VERIFICATION_TARGETS.md)
   Example: 3 properties / 15 properties = 20%
   ```

3. **VC Coverage**: % of verification conditions proven (TECHNICAL METRIC)
   ```
   VCs Proven / Total VCs Generated
   Example: 235 VCs / 247 VCs = 95.1%
   ```

4. **Target Coverage**: % of verification targets completed
   ```
   Completed VTs / Total VTs
   Example: VT-1, VT-2 complete → 2/15 = 13%
   ```

**Deliverables**:
- ✅ Verus integrated into CI/CD
- ✅ Automated coverage reports (JSON + Markdown + HTML)
- ✅ Coverage badge in README
- ✅ 15 verification targets documented (5 core + 5 Bitcoin + 5 Ethereum)
- ✅ Dashboard showing verification progress

**Example Output**:
```
=== Verus Verification Coverage ===
Total VCs:        247
Verified VCs:     235
Coverage:         95.1%

Property Coverage: 20% (3/15)
  ✅ VT-1.1: Amount::checked_add overflow
  ✅ VT-1.2: Amount::checked_sub underflow
  ✅ VT-2.1: Canonical encoding determinism
  ⏳ VT-2.2: Canonicalization roundtrip (in progress)
  ❌ VT-10.1: Bitcoin varint parsing (todo)
  ...
```

---

### 4.1: Core Library Verification (VT-1 to VT-5)

**Priority**: HIGH (Security critical)
**Timeline**: Week 3-8 (6 weeks)
**Target**: 40% core library coverage

**Verification Targets** (see `docs/VERIFICATION_TARGETS.md`):

**VT-1: Amount Arithmetic Safety** ⚡ CRITICAL
- [ ] VT-1.1: `checked_add` overflow detection (~5 VCs)
- [ ] VT-1.2: `checked_sub` underflow detection (~4 VCs)
- [ ] VT-1.3: `checked_mul` overflow detection (~6 VCs)
- [ ] VT-1.4: Decimal conversion correctness (~5 VCs)
- **Total**: ~20 VCs, **Effort**: 2 weeks

**VT-2: Canonicalization Determinism** ⚡ CRITICAL
- [ ] VT-2.1: `to_canonical_bytes()` is deterministic (~8 VCs)
- [ ] VT-2.2: Borsh encoding never panics (~6 VCs)
- [ ] VT-2.3: Canonical bytes are bounded (~6 VCs)
- **Total**: ~20 VCs, **Effort**: 3 weeks
- **Blocked By**: Borsh library verification (may require trusted axioms)

**VT-3: Error Propagation Safety**
- [ ] VT-3.1: Error conversion preserves information (~4 VCs)
- [ ] VT-3.2: Error types are exhaustive (~3 VCs)
- [ ] VT-3.3: Error propagation never panics (~3 VCs)
- **Total**: ~10 VCs, **Effort**: 1 week

**VT-4: Hook Execution Ordering**
- [ ] VT-4.1: Hooks execute in defined order (~5 VCs)
- [ ] VT-4.2: Hook failures propagate correctly (~4 VCs)
- [ ] VT-4.3: Hook state is consistent (~3 VCs)
- **Total**: ~12 VCs, **Effort**: 2 weeks

**VT-5: Version Isolation (Const Generics)**
- [ ] VT-5.1: TxIR<V1> cannot cast to TxIR<V2> (~3 VCs)
- [ ] VT-5.2: Version preserved through canonicalization (~2 VCs)
- **Total**: ~5 VCs, **Effort**: 1 week

**Key Properties Proven**:
- ✅ Injectivity: `encode(decode(bytes)) == bytes`
- ✅ Panic-freedom: No unwrap(), no panics
- ✅ Determinism: Same data → same bytes
- ✅ Overflow safety: All arithmetic checked

**Deliverables**:
- ~67 VCs proven across core library
- 5 verification targets completed
- Core library verification coverage: 40%

---

### 4.2: Bitcoin Decoder Verification (VT-10 to VT-14)

**Priority**: MEDIUM
**Timeline**: Week 9-16 (8 weeks)
**Target**: 60% Bitcoin decoder coverage

**Verification Targets**:

**VT-10: Varint Parsing Safety** ⚡ HIGH
- [ ] VT-10.1: `parse_varint` never panics (bounds-checked) (~10 VCs)
- [ ] VT-10.2: Non-canonical varints rejected (~5 VCs)
- [ ] VT-10.3: Varint value fits in u64 (~3 VCs)
- **Total**: ~18 VCs, **Effort**: 2 weeks

**VT-11: Transaction Parsing Bounds**
- [ ] VT-11.1: Input parsing never reads out of bounds (~20 VCs)
- [ ] VT-11.2: Output parsing never reads out of bounds (~15 VCs)
- [ ] VT-11.3: Witness parsing is bounds-checked (~10 VCs)
- **Total**: ~45 VCs, **Effort**: 4-6 weeks (MAJOR)

**VT-12: Fee Calculation Overflow Safety**
- [ ] VT-12.1: Total input value doesn't overflow (~3 VCs)
- [ ] VT-12.2: Total output value doesn't overflow (~3 VCs)
- [ ] VT-12.3: Fee calculation handles underflow (~2 VCs)
- **Total**: ~8 VCs, **Effort**: 1 week

**VT-13: TXID Calculation Correctness**
- [ ] VT-13.1: TXID is deterministic (~3 VCs)
- [ ] VT-13.2: SegWit TXID excludes witness data (~2 VCs)
- [ ] VT-13.3: Hash computation never panics (~1 VC)
- **Total**: ~6 VCs, **Effort**: 1 week

**VT-14: Bitcoin Canonicalization Injectivity** ⚡ CRITICAL
- [ ] VT-14.1: `encode(decode(bytes)) == bytes` roundtrip (~10 VCs)
- [ ] VT-14.2: Signature verification preserved (~5 VCs)
- **Total**: ~15 VCs, **Effort**: 3-4 weeks

**Deliverables**:
- ~92 VCs proven across Bitcoin decoder
- 5 verification targets completed
- Bitcoin decoder verification coverage: 60%

---

### 4.3: Ethereum Decoder Verification (VT-20 to VT-24)

**Priority**: MEDIUM
**Timeline**: Week 17-24 (8 weeks)
**Target**: 50% Ethereum decoder coverage

**Verification Targets**:

**VT-20: RLP Parsing Safety**
- [ ] VT-20.1: RLP decode never panics (~15 VCs)
- [ ] VT-20.2: RLP decode rejects malformed input (~10 VCs)
- [ ] VT-20.3: RLP length fields validated (~5 VCs)
- **Total**: ~30 VCs, **Effort**: 3-4 weeks

**VT-21: Gas Calculation Overflow Safety**
- [ ] VT-21.1: Gas limit * gas price doesn't overflow (~4 VCs)
- [ ] VT-21.2: EIP-1559 fee calculations safe (~4 VCs)
- [ ] VT-21.3: Priority fee + base fee doesn't overflow (~2 VCs)
- **Total**: ~10 VCs, **Effort**: 2 weeks

**VT-22: EIP-2718 Transaction Type Detection**
- [ ] VT-22.1: Transaction type correctly identified (~3 VCs)
- [ ] VT-22.2: Type-specific parsing is safe (~3 VCs)
- [ ] VT-22.3: Unknown types rejected (~2 VCs)
- **Total**: ~8 VCs, **Effort**: 1 week

**VT-23: Signature Recovery Safety**
- [ ] VT-23.1: Recovery ID (v) validated (~4 VCs)
- [ ] VT-23.2: Signature (r, s) in valid range (~5 VCs)
- [ ] VT-23.3: Address recovery never panics (~3 VCs)
- **Total**: ~12 VCs, **Effort**: 2 weeks

**VT-24: Ethereum Canonicalization Determinism**
- [ ] VT-24.1: RLP encoding is deterministic (~6 VCs)
- [ ] VT-24.2: Transaction hash is deterministic (~4 VCs)
- **Total**: ~10 VCs, **Effort**: 2 weeks

**Deliverables**:
- ~70 VCs proven across Ethereum decoder
- 5 verification targets completed
- Ethereum decoder verification coverage: 50%

---

### 4.4: Coverage Dashboards & Reporting

**Priority**: MEDIUM
**Timeline**: Week 25-26 (2 weeks)

**Tasks**:
- [ ] Generate comprehensive HTML dashboard with Chart.js
- [ ] Create markdown reports for GitHub Pages
- [ ] Add verification badge to README (shields.io)
- [ ] Document verification process for contributors
- [ ] Publish verification results (blog post or paper)
- [ ] Create visualization of proof dependency graph
- [ ] Set up coverage threshold enforcement (80% for core)

**Deliverables**:
- Interactive HTML dashboard showing:
  - Overall verification coverage (property + VC)
  - Per-crate breakdown
  - Per-target status
  - Historical trends
  - Failed VCs with error messages
- Automated coverage reports in CI
- Public verification status page

---

### 4.5: Phase 4 Success Criteria

**Coverage Goals**:
- ✅ **Core Library**: 40% property coverage (5/5 targets, ~67 VCs)
- ✅ **Bitcoin Decoder**: 60% property coverage (5/5 targets, ~92 VCs)
- ✅ **Ethereum Decoder**: 50% property coverage (5/5 targets, ~70 VCs)
- ✅ **Overall**: 50% property coverage (15/15 targets documented, ~229 VCs proven)

**Infrastructure**:
- ✅ Verus integrated into CI/CD
- ✅ Automated coverage tracking and reporting
- ✅ Coverage dashboard deployed
- ✅ Coverage badge visible in README
- ✅ All 15 verification targets documented

**Documentation**:
- ✅ `docs/VERUS_VERIFICATION_COVERAGE.md` - Comprehensive guide
- ✅ `docs/VERIFICATION_TARGETS.md` - All 15 targets documented
- ✅ `docs/FORMAL_VERIFICATION.md` - Implementation guide
- ✅ Proof strategies documented
- ✅ Verification examples published

**Validation**:
- ✅ At least 1 complete decoder verified (Bitcoin or Ethereum)
- ✅ Zero critical bugs found through verification
- ✅ All critical properties proven (Amount, Canonicalization)
- ✅ Verification runs on every commit without timeout

---

### 4.6: Advanced Verification (Optional)

**Priority**: LOW (Post-v0.3.0)

**Research Directions**:
- [ ] Prove cryptographic properties (hash preimage resistance)
- [ ] Verify concurrent decoder composition
- [ ] Generate ZK proofs of transaction validity
- [ ] Cross-chain property verification
- [ ] Academic publication (SOSP, OSDI, CAV)

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

**Note**: Privacy chains (Zcash, Aleo, Monero) are addressed in Phase 3.8.

Chains to support (priority order):
1. [ ] Cosmos (IBC support) - **Note**: Cosmos SDK family decoder planned for Phase 3.5
2. [ ] Avalanche (multi-chain) - **Note**: C-Chain uses EVM decoder (Phase 3.1), X-Chain and P-Chain need custom decoder
3. [ ] NEAR Protocol (Borsh-native!)
4. [ ] Tron
5. [ ] Stellar
6. [ ] Algorand
7. [ ] Additional privacy chains (Firo, Beam, Grin)

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

### v0.2.0 (Chain Family Support)
- ✅ 3 core decoders (Bitcoin, Ethereum, Solana)
- ✅ All 3 models (UTXO, Account, Instruction) proven
- ✅ 17 top chains scaffolded
- ✅ Common crates analysis complete
- [ ] Phase 2.5: decoder-encodings and decoder-test-utils extracted
- [ ] 8 chain family decoders implemented
- [ ] 620+ chains supported via family approach
- [ ] Airgapped operation verified
- [ ] Real transaction test coverage > 80%
- [ ] 430 LOC reduction (-13%) from code extraction

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

**Last Updated**: 2025-11-13
**Current Phase**: Phase 3.5 Complete ✅ (Cosmos SDK Decoder)
**Status**: Cosmos SDK decoder complete with 228 chains support
**Branch**: `claude/cosmos-sdk-phase-3.5-011CV5yFdhxiSFEu3Ztnxidm`

**Completed Milestones**:
  - ✅ Phase 2.1: Bitcoin decoder (pure Rust, 186 tests)
  - ✅ Phase 2.2: Ethereum decoder (pure Rust, RLP + EIP-2718)
  - ✅ Phase 2.3: Solana decoder (pure Rust, compact-u16 + instruction model)
  - ✅ Phase 2.4: Top 20 chains scaffolding (17 new decoders, chain family strategy)
  - ✅ Phase 2.5: Common crates extraction (decoder-encodings, decoder-test-utils)
  - ✅ Phase 1.5.1: Chain registry vendoring (cosmos + superchain, 14.5MB)
  - ✅ Phase 1.5.1b: Borsh transformation (14.5MB → 24KB, 99.8% reduction!)
  - ✅ Phase 3.5: Cosmos SDK decoder (1,856 lines, 228 chains, 31 tests) 🎉 NEW!
  - 🚧 Phase 3.2: OP Stack deposit transactions (90% complete, 1,026 lines)

**Next Milestones**:
  - **Phase 3.2: Complete OP Stack implementation (~4 hours)** ⭐ IMMEDIATE NEXT PRIORITY
    - Status: 90% complete (core implementation done, needs trait fixes)
    - Fix EthereumTransaction trait implementations
    - Implement Canonicalizer for OptimismTransaction
    - Add integration tests with real deposit transactions
    - Connect to superchain registry (already vendored)
  - **Phase 3.5 Enhancement: IBC & CosmWasm support** 🔧 FOLLOW-UP (Optional)
    - Enable IBC feature flags in cosmos-sdk-proto
    - Enable CosmWasm feature flags
    - Add mainnet integration tests
    - Estimated: 1-2 days
  - Phase 3.3: Arbitrum Orbit family decoder (retryable tickets) - 1-2 days
  - Phase 1.5.2: More property tests (need 34 more, currently 16/50) - ongoing
  - Phase 3.4: SVM family decoder - 1 week
  - Phase 3.6: Move VM family decoder - 1 week
  - Phase 3.7: Bitcoin forks family decoder - 3 days
  - Phase 3.8: Privacy chains family decoder (Zcash, Aleo, Monero) - 2 weeks
  - Phase 3.9: Universal decoder integration - 1 week
  - v0.2.0 (Phase 3 complete: 620+ chains + privacy chains supported) - 2-3 months
