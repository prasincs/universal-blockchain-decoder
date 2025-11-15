# CLAUDE: Universal Blockchain Decoder Guide

**Version**: 0.3.0 (Streamlined & Action-Oriented)
**Last Updated**: 2025-11-15
**Status**: Proposed Improvements

---

## 📋 Quick Navigation

**For Day-to-Day Work**:
- [Add New Chain (5 min)](#add-new-chain-in-5-minutes) ⚡
- [Add Tests (10 min)](#testing-quick-start) 🧪
- [Pre-Commit Checklist](#pre-commit-checklist) ✅
- [Common Commands](#common-commands) 🔧

**For Deep Dives**:
- [Design Philosophy](#design-philosophy) 🎯
- [Architecture Patterns](#architecture-patterns) 🏗️
- [Full Documentation Map](#documentation-map) 📚

**Current Phase**: Phase 2 - Pure Rust Decoders (6 decoders production-ready, 12 in progress)

---

## 🚀 Add New Chain in 5 Minutes

### Option A: Automated (Recommended)

```bash
# 1. Generate decoder from template
cargo run -p decoder-generator -- new \
    --chain "Avalanche" \
    --chain-id 43114 \
    --family Account \
    --template evm-compatible

# 2. Run generated tests
cargo test -p decoder-avalanche

# 3. Pre-commit checks
cargo fmt --all && cargo clippy --all -- -D warnings

# 4. Done! Commit and iterate
git add crates/decoder-avalanche
git commit -m "Add Avalanche decoder scaffold"
```

**Generated Files**:
```
crates/decoder-avalanche/
├── Cargo.toml              # Dependencies configured
├── src/
│   ├── lib.rs              # Chain traits + basic decoder
│   ├── types.rs            # Transaction structure (template)
│   └── registry.rs         # Chain metadata (if registry-based)
├── tests/
│   ├── unit_tests.rs       # Chain identity, basic validation
│   ├── property_tests.rs   # Proptest templates (ready to customize)
│   └── fixtures/           # Empty directory (add .hex files here)
└── README.md               # Pre-filled with chain info
```

### Option B: Manual (Full Control)

<details>
<summary>Click to expand manual steps</summary>

```bash
# 1. Copy template
cp -r crates/decoder-_template crates/decoder-avalanche

# 2. Find & replace
cd crates/decoder-avalanche
find . -type f -exec sed -i 's/TEMPLATE/Avalanche/g' {} \;
find . -type f -exec sed -i 's/template/avalanche/g' {} \;

# 3. Update chain metadata
# Edit src/lib.rs:
#   - chain_id: 43114
#   - chain_name: "Avalanche"
#   - chain_family: ChainFamily::Account

# 4. Add to workspace
# Edit /Cargo.toml workspace members:
#   + "crates/decoder-avalanche"

# 5. Build to verify
cargo build -p decoder-avalanche

# 6. Add minimal tests
# See templates/MINIMAL_TESTS.md for copy-paste tests
```

</details>

### Option C: Registry-Based (For EVM/Cosmos/OP chains)

```bash
# For chains that are variations of existing families:

# EVM-compatible chains (2000+ supported automatically)
# → Just add chain ID to crates/decoder-evm/vendored/chainlist
# → No new decoder needed!

# Cosmos SDK chains (228 supported automatically)
# → Add to crates/decoder-cosmos-sdk/vendored/chain-registry
# → No new decoder needed!

# OP Stack chains (Optimism, Base, Zora, etc.)
# → Add to crates/decoder-op-stack/vendored/superchain-registry
# → No new decoder needed!
```

**Next Steps After Scaffold**:
1. Add real transaction parsing logic (replace `raw_bytes` stub)
2. Add test fixtures (see [Testing Quick Start](#testing-quick-start))
3. Implement `canonicalize()` method for TxIR conversion
4. Add property-based tests
5. (Optional) Add fuzzing targets

---

## 🧪 Testing Quick Start

### Minimal Testing Requirements by Maturity

| Decoder Maturity | Required Tests | Files | Example |
|------------------|----------------|-------|---------|
| **Scaffold** (Phase 1) | Chain identity test | 1 inline test | `decoder-algorand` |
| **Core** (Phase 2.1) | + Format validation, + 1 fixture | 2 files (unit + integration) | `decoder-litecoin` |
| **Advanced** (Phase 2.2) | + Property tests (10+ cases), + 3 fixtures | 3 files + fixtures/ | `decoder-sui` |
| **Production** (Phase 2.3) | + Validation vs reference impl, + Fuzzing | 4 files + fuzz/ | `decoder-bitcoin` |

### Add Tests in 10 Minutes

```bash
# 1. Copy test templates
cp docs/templates/PROPERTY_TEST_TEMPLATE.rs \
   crates/decoder-avalanche/tests/property_tests.rs

cp docs/templates/INTEGRATION_TEST_TEMPLATE.rs \
   crates/decoder-avalanche/tests/integration_tests.rs

# 2. Find & replace (automated)
find crates/decoder-avalanche/tests -type f \
    -exec sed -i 's/{{CHAIN}}/Avalanche/g' {} \;

# 3. Add a test fixture
# Get real transaction from chain explorer
# Save hex-encoded bytes to:
mkdir -p crates/decoder-avalanche/tests/fixtures
echo "YOUR_HEX_TX_HERE" > crates/decoder-avalanche/tests/fixtures/tx_001.hex

# 4. Update integration test to use fixture
# (Template already includes fixture loading pattern)

# 5. Run tests
cargo test -p decoder-avalanche

# 6. Check coverage (optional)
cargo tarpaulin -p decoder-avalanche --out Html
open tarpaulin-report.html
```

### Property Test Patterns (Copy-Paste Ready)

<details>
<summary>Never Panics Test</summary>

```rust
use proptest::prelude::*;
use decoder_test_utils::arb_small_bytes;

proptest! {
    #[test]
    fn prop_decoder_never_panics(bytes in arb_small_bytes()) {
        let result = AvalancheDecoder::decode(&bytes);
        // Should return Ok or Err, never panic
        prop_assert!(result.is_ok() || result.is_err());
    }
}
```

</details>

<details>
<summary>Canonical Bytes Deterministic Test</summary>

```rust
proptest! {
    #[test]
    fn prop_canonical_bytes_deterministic(bytes in arb_valid_avalanche_tx()) {
        let tx = AvalancheDecoder::decode(&bytes).unwrap();
        let canonical1 = tx.to_canonical_bytes().unwrap();
        let canonical2 = tx.to_canonical_bytes().unwrap();
        prop_assert_eq!(canonical1, canonical2);
    }
}
```

</details>

<details>
<summary>Roundtrip Test (Decode → Canonical → Decode)</summary>

```rust
proptest! {
    #[test]
    fn prop_roundtrip_decode_canonical(bytes in arb_valid_avalanche_tx()) {
        let tx1 = AvalancheDecoder::decode(&bytes).unwrap();
        let canonical = tx1.to_canonical_bytes().unwrap();
        let ir = tx1.canonicalize().unwrap();
        let canonical2 = ir.to_canonical_bytes().unwrap();
        prop_assert_eq!(canonical, canonical2);
    }
}
```

</details>

### Fixture Management

```bash
# Fixture directory structure (standardized):
tests/fixtures/
├── README.md                    # Source URLs for all fixtures
├── mainnet/
│   ├── tx_001_simple.hex        # Basic transfer
│   ├── tx_002_complex.hex       # Complex transaction
│   └── tx_003_edge_case.hex     # Edge case (max size, etc.)
├── testnet/
│   └── tx_testnet_001.hex
└── invalid/                     # Malformed transactions
    ├── invalid_empty.hex
    ├── invalid_truncated.hex
    └── invalid_wrong_version.hex

# README.md template:
# Transaction Fixtures
#
# - tx_001_simple.hex: Simple transfer
#   Source: https://explorer.avax.network/tx/0xABC123
#   Block: 12345678
#   Date: 2024-01-15
```

### Automated Test Generation

```bash
# Generate comprehensive test suite
cargo run -p decoder-generator -- add-tests \
    --decoder avalanche \
    --fixtures 3 \
    --property-tests 10 \
    --fuzz

# This creates:
# - tests/property_tests.rs (10 property tests)
# - tests/integration_tests.rs (3 fixture tests)
# - fuzz/fuzz_targets/fuzz_decode.rs (fuzzing target)
```

---

## ✅ Pre-Commit Checklist

**MANDATORY before every commit**:

```bash
# Run this ONE command (sequential checks):
cargo fmt --all && \
cargo clippy --all --all-targets --all-features -- -D warnings && \
cargo test --lib --all

# Expected output:
# ✓ cargo fmt: no changes needed
# ✓ cargo clippy: no warnings
# ✓ cargo test: all tests passed
```

**Optional but recommended**:
```bash
# Run property tests (slower, 1000 iterations)
cargo test --test property_tests --all -- --test-threads=1

# Run integration tests (requires fixtures)
cargo test --tests --all

# Security audit (checks for known vulnerabilities)
cargo audit

# Documentation build
cargo doc --no-deps
```

**Git Workflow**:
```bash
# 1. Make changes
# 2. Pre-commit checks (see above)
# 3. Commit with descriptive message
git add .
git commit -m "feat(avalanche): Add basic transaction parsing"

# 4. Push to feature branch
git push -u origin claude/add-avalanche-decoder-SESSIONID
```

**If Clippy Fails**:
```bash
# Common fixes:

# 1. Borrowed expression implements required traits
- Err(DecoderError::invalid(&format!("...", x)))
+ Err(DecoderError::invalid(format!("...", x)))

# 2. Length comparison to zero
- if vec.len() > 0 { }
+ if !vec.is_empty() { }

# 3. Useless vec! for single element
- let items = vec!["single"];
+ let items = ["single"];

# 4. Unnecessary lifetimes
- fn decode<'a>(bytes: &'a [u8]) -> Result<Tx<'a>>
+ fn decode(bytes: &[u8]) -> Result<Tx>
```

---

## 🔧 Common Commands

### Development

```bash
# Build specific decoder
cargo build -p decoder-avalanche

# Test specific decoder
cargo test -p decoder-avalanche

# Test everything
cargo test --all

# Build all decoders
cargo build --workspace

# Check without building (fast)
cargo check --workspace
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all --all-targets --all-features -- -D warnings

# Security audit
cargo audit

# Dependency tree
cargo tree -p decoder-avalanche --depth 2

# Check for outdated dependencies
cargo outdated
```

### Testing

```bash
# Unit tests only (fast)
cargo test --lib -p decoder-avalanche

# Integration tests only
cargo test --tests -p decoder-avalanche

# Property tests only
cargo test --test property_tests -p decoder-avalanche

# Run with verbose output
cargo test -p decoder-avalanche -- --nocapture

# Run specific test
cargo test -p decoder-avalanche test_chain_identity

# Coverage report
cargo tarpaulin -p decoder-avalanche --out Html
```

### Fuzzing

```bash
# List fuzz targets
cargo fuzz list

# Run fuzzing (bitcoin example)
cargo fuzz run fuzz_bitcoin_decode -- -max_len=1000000 -max_total_time=300

# Run with coverage
cargo fuzz coverage fuzz_bitcoin_decode

# View corpus
ls fuzz/corpus/fuzz_bitcoin_decode/
```

### Documentation

```bash
# Build docs
cargo doc --no-deps --open

# Build specific decoder docs
cargo doc -p decoder-avalanche --open

# Check doc links
cargo doc --no-deps 2>&1 | grep warning
```

### Benchmarking

```bash
# Run benchmarks
cargo bench -p decoder-avalanche

# Compare benchmarks (after changes)
cargo bench -- --save-baseline before
# ... make changes ...
cargo bench -- --baseline before
```

---

## 🎯 Design Philosophy

> **Core Principle**: Minimal Trusted Computing Base (TCB) < 3000 LOC

### The 10 Design Criteria

1. **Minimal Core** ⚡ - Core < 3000 LOC, traits not implementations
2. **Formally Verifiable** 🔬 - Verus annotations, panic-free, deterministic
3. **Reviewable** 📖 - Audit core in one sitting, comprehensive docs
4. **Trait-Based Extensibility** 🔌 - Zero core changes for new chains
5. **Canonical Serialization** 🔐 - Borsh (never JSON) for determinism
6. **Zero-Cost Abstractions** ⚡ - Static dispatch, monomorphization
7. **Layered Security** 🛡️ - Core (trusted) + Decoders (untrusted)
8. **Supply Chain Security** 🔗 - Vendor dependencies, airgapped operation
9. **Testing Strategy** 🧪 - 5-level pyramid (unit → property → integration → validation → fuzz)
10. **Documentation as Code** 📝 - Examples, formal properties, security notes

**See**: [Full design philosophy](#detailed-design-philosophy) (scroll down)

### Scope Boundaries

**In Scope** ✅:
- Decoding blockchain transactions (chain bytes → TxIR)
- Canonical serialization (TxIR → Borsh bytes)
- Transaction validation (structural correctness)
- Signature verification (checking existing signatures)

**Out of Scope** ❌:
- Transaction encoding (TxIR → chain bytes)
- Transaction construction (building new transactions)
- Transaction signing (creating signatures)
- Fee estimation, UTXO selection, nonce management

**Why Decode-Only?** See [Decision Log: Why Decoding Only?](#decision-log)

---

## 🏗️ Architecture Patterns

### Trait-Based Chain Addition

```rust
// Core defines traits (NO chain-specific logic)
pub trait ChainIdentity {
    fn chain_id(&self) -> u64;
    fn chain_name(&self) -> &str;
    fn chain_family(&self) -> ChainFamily;
}

pub trait ChainDecoder {
    type TxSpecific;
    type Chain: ChainIdentity;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific>;
    fn validate_format(raw_bytes: &[u8]) -> Result<()>;
}

// Decoders implement traits (external crates)
impl ChainDecoder for AvalancheDecoder {
    type TxSpecific = AvalancheTx;
    type Chain = AvalancheChain;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Pure Rust parsing, no external libs
    }
}
```

### Canonical Serialization Pattern

```rust
// ALWAYS use Borsh for canonicalization
impl Canonicalizer for AvalancheTx {
    const VERSION: u8 = 1;

    fn canonicalize(&self) -> Result<TxIR<1>> {
        // Convert chain-specific → chain-agnostic TxIR
        Ok(TxIR::new(
            &AvalancheChain,
            TxMetadata { /* ... */ },
            AuthorizationPackage { /* ... */ },
            operations,
            StateDeltas { /* ... */ },
        ))
    }

    fn to_canonical_bytes(&self) -> Result<Vec<u8>> {
        let ir = self.canonicalize()?;
        borsh::to_vec(&ir) // ✅ DETERMINISTIC
    }

    fn canonical_hash(&self) -> Result<[u8; 32]> {
        let bytes = self.to_canonical_bytes()?;
        Ok(sha256(&bytes))
    }
}

// ❌ NEVER use JSON (non-deterministic key ordering)
// let json = serde_json::to_string(&tx)?;  // DANGEROUS
```

### Chain Family Grouping

**Strategy**: Share parsing logic for similar chains

```
EVM Family (2000+ chains)
├── decoder-evm (base RLP parsing)
├── decoder-optimism (OP Stack additions)
├── decoder-arbitrum (ArbOS additions)
└── [Other EVM chains use decoder-evm directly]

Cosmos Family (228 chains)
├── decoder-cosmos-sdk (Protobuf parsing)
└── [Chain-specific via registry]

Bitcoin Family (10+ chains)
├── decoder-bitcoin (base)
├── decoder-litecoin (extends bitcoin)
├── decoder-dogecoin (extends bitcoin)
└── decoder-zcash (adds shielded transactions)

Solana Family
├── decoder-solana (base)
└── decoder-svm (abstraction)
```

**See**: `docs/CHAIN_FAMILIES_GROUPING.md`

---

## 📚 Documentation Map

**Quick Guides** (Start Here):
- `CLAUDE.md` - This file (day-to-day workflows)
- `ROADMAP.md` - Project phases and timeline
- `CONTRIBUTING.md` - Contribution guidelines
- `docs/TESTING_AND_DEPENDENCIES_SUMMARY.md` - Testing overview

**Architecture**:
- `docs/TRAIT_BASED_ARCHITECTURE.md` - Extensibility patterns
- `docs/CHAIN_FAMILIES_GROUPING.md` - Shared decoder strategy
- `docs/SHARED_CRATES_STRATEGY.md` - Code reuse
- `docs/CANONICAL_SERIALIZATION.md` - Borsh requirements

**Implementation**:
- `docs/TESTING_STRATEGY.md` - 5-level testing pyramid
- `docs/GIT_SUBTREE_VENDORING.md` - Supply chain verification
- `docs/DECODER_DEPENDENCY_STRATEGY.md` - Pure Rust parsing
- `docs/CHAIN_ADDITION_STRATEGY.md` - How to add chains (comprehensive)

**Verification**:
- `docs/FORMAL_VERIFICATION.md` - Verus integration
- `docs/VERUS_VERIFICATION_COVERAGE.md` - Verification targets
- `docs/VERIFICATION_TARGETS.md` - 15 formal properties

**Research**:
- `docs/CHAIN_COVERAGE_ANALYSIS.md` - All 9000+ chains analyzed
- `docs/TOP_20_CHAINS_IMPLEMENTATION_PLAN.md` - Priority roadmap
- `docs/STARKNET_RESEARCH.md` - ZK architecture (844 LOC)

---

## 🤖 Claude CLI Workflows

> **Moved to separate file**: See `docs/CLAUDE_CLI_WORKFLOWS.md` for detailed tool usage patterns

**Quick Tips**:
- Use `Task(Explore)` for open-ended codebase questions
- Use `TodoWrite` for all multi-step tasks (3+ steps)
- Run pre-commit checks: `cargo fmt && cargo clippy -- -D warnings`
- Parallel tool calls for independent operations
- Always read files before editing

**Example Workflow: Add New Chain**:
```bash
# 1. Create todo list
TodoWrite([...chain addition steps...])

# 2. Run generator
Bash("cargo run -p decoder-generator -- new --chain Avalanche ...")

# 3. Verify generated files
Read("crates/decoder-avalanche/src/lib.rs")
Read("crates/decoder-avalanche/Cargo.toml")

# 4. Run tests
Bash("cargo test -p decoder-avalanche")

# 5. Mark todo complete
TodoWrite([...mark completed...])
```

---

## 🚦 Current Status & Roadmap

### Decoder Implementation Status (34 total)

| Maturity | Count | Status | Examples |
|----------|-------|--------|----------|
| **Production** | 6 | ✅ Complete | Bitcoin, Ethereum, Solana, Cosmos, EVM, Optimism |
| **Advanced** | 4 | 🚧 90% | Sui, Aptos, SVM, Arbitrum |
| **Core** | 8 | 🚧 60% | XRP, Litecoin, Dogecoin, Zcash |
| **Scaffold** | 12 | 📋 10% | Algorand, TRON, Stellar, Polkadot |
| **Infrastructure** | 4 | ✅ Complete | Core, Primitives, Encodings, Test-Utils |

### Roadmap Phases

**Phase 1.5** - Testing & Dependency Infrastructure (✅ 80% Complete)
- [x] Vendor hex using git subtree
- [x] Move serde_json to dev-dependencies
- [x] Install Verus formal verification
- [ ] Complete property test coverage (16/50 current)
- [ ] Add fuzzing for all production decoders (3/6 current)

**Phase 2** - Pure Rust Decoders (🚧 In Progress)
- [x] Bitcoin decoder (production)
- [x] Ethereum decoder (production)
- [x] Solana decoder (production)
- [ ] Complete 8 Core decoders (5/8 done)
- [ ] Complete 12 Scaffold decoders (2/12 done)

**Phase 3** - Chain Family Extensions (📋 Planned)
- **Phase 3.2**: OP Stack (90% done, ~4 hours) - Optimism, Base, Zora
- **Phase 3.5**: Cosmos SDK (registry vendored, high ROI)
- **Phase 3.6a**: ZK Cryptography (Starknet foundation)
- **Phase 3.10**: WASM Demo (1-2 weeks, perfect for papers/blogs/conferences)

**Phase 4** - Formal Verification (📋 Planned)
- Verify core traits with Verus
- Verify canonical serialization
- Verify reference implementations

**See**: `ROADMAP.md` for detailed timeline

---

## 📊 Metrics & Goals

### Current Metrics (as of 2025-11-15)

| Metric | Current | Goal | Status |
|--------|---------|------|--------|
| Core LOC | ~2700 | < 3000 | ✅ Good |
| Core dependencies | 5 | ≤ 5 | ✅ Good |
| Decoder dependencies | 0 production | 0 | ✅ Good |
| Test coverage | ~45% | 100% core, 90% decoders | 🚧 In Progress |
| Property tests | 16 | 50+ | 🚧 Week 2 |
| Fuzzing decoders | 3/30 | 30/30 | 📋 Planned |
| Production decoders | 6/30 | 30/30 | 🚧 Weeks 3-8 |
| Chains supported | ~2500 | 9000+ | 🚧 Registry-based |

### Quality Gates (Required for v1.0.0)

- [ ] Core library < 3000 LOC
- [ ] Zero production dependencies (except serde, borsh, thiserror, crypto)
- [ ] 100% test coverage in core
- [ ] All decoders have property tests (50+ cases each)
- [ ] All production decoders have fuzzing
- [ ] Formal verification annotations complete
- [ ] Security audit passed
- [ ] All 30 decoders at "Advanced" maturity or higher

---

## 🔍 Troubleshooting

### Common Issues

<details>
<summary>Clippy warnings about borrowed expressions</summary>

```rust
// Problem:
Err(DecoderError::invalid(&format!("error: {}", x)))
//                         ^ borrowed expression implements required traits

// Solution: Remove &
Err(DecoderError::invalid(format!("error: {}", x)))
```

</details>

<details>
<summary>Tests not running in CI</summary>

```bash
# Problem: Property tests not discovered
# tests/my_property_tests.rs not running

# Solution: Check filename pattern in .github/workflows/test.yml
# Must match: cargo test --test property_tests
# Or rename: tests/property_tests.rs
```

</details>

<details>
<summary>Vendored hex not found</summary>

```rust
// Problem:
use hex::decode;  // Can't find hex in dependencies

// Solution: Use vendored version
use universal_decoder_core::hex::decode;
```

</details>

<details>
<summary>Cargo.toml workspace errors</summary>

```bash
# Problem: New decoder not building
# Error: package `decoder-avalanche` is listed in workspace's members but is not found

# Solution: Check path in /Cargo.toml
[workspace]
members = [
    "crates/decoder-avalanche",  # ← Must match actual directory
]
```

</details>

---

## 📖 Detailed Design Philosophy

<details>
<summary>Click to expand full design criteria</summary>

### 1. Minimal Core ⚡

**Goal**: Core library should be < 3000 LOC

**Why**: Smaller core = easier audit, faster verification, fewer bugs

**How**:
- Core defines **traits**, not implementations
- Core provides **types** and **guarantees**, not algorithms
- Chain-specific logic lives in **separate crates**

### 2. Formally Verifiable 🔬

**Goal**: Core library amenable to formal verification with Verus

**Requirements**:
- No `unsafe` code in core
- Explicit preconditions and postconditions
- Provable panic-freedom

**Critical Properties**:
1. Injectivity: `encode(canonicalize(decode(tx_bytes))) = tx_bytes`
2. Panic-Freedom: `decode(input)` never panics
3. Determinism: `to_canonical_bytes(tx) = to_canonical_bytes(tx)`

### 3. Reviewable & Auditable 📖

**Audit Checklist**:
- [ ] Core library < 3000 LOC
- [ ] No `unsafe` blocks
- [ ] All panics documented and justified
- [ ] All arithmetic operations checked for overflow

### 4. Trait-Based Extensibility 🔌

**Goal**: Zero core changes to add new blockchains

**See**: [Architecture Patterns](#architecture-patterns)

### 5. Canonical Serialization 🔐

**Non-Negotiable Rules**:
1. **NEVER** use JSON for hashing
2. **ALWAYS** use Borsh for canonical representation
3. JSON is **ONLY** for human display

### 6-10. [See original CLAUDE.md sections]

</details>

---

## 📝 Decision Log

### Why Decoding Only (No Encoding)?

**Decision**: Project scope limited to transaction decoding

**Rationale**:
- Different problem domains (defensive vs constructive)
- TCB preservation (encoding would add 2500+ LOC)
- Dependency explosion (chain state, fee oracles, etc.)
- Clear use case focus (explorers, forensics, indexers)

**Future Path**: Separate `universal-blockchain-encoder` project if needed

### Why Borsh over Protobuf?

**Decision**: Use Borsh for canonical serialization

**Rationale**:
- Designed specifically for deterministic encoding
- Simpler than Protobuf (no schema management)
- Battle-tested in Solana, NEAR

### Why Traits over Enums?

**Decision**: Trait-based chains (v0.2.0+)

**Rationale**:
- Enums violate open-closed principle
- Traits enable ecosystem growth
- Core stays minimal

---

## 🙏 Contributing

All contributions must adhere to:
1. ✅ No core changes for new chains (use traits)
2. ✅ Maintain formal verifiability (no unsafe)
3. ✅ Preserve minimal TCB (< 3000 LOC core)
4. ✅ Use canonical serialization (Borsh, not JSON)
5. ✅ Comprehensive tests (unit + property + integration)
6. ✅ Decoding only (no encoding/construction/signing)

**See**: `CONTRIBUTING.md`

---

## 📞 Getting Help

- **Documentation**: See [Documentation Map](#documentation-map)
- **Issues**: https://github.com/anthropics/claude-code/issues
- **Discussions**: GitHub Discussions (for architecture questions)
- **Security**: See SECURITY.md for reporting vulnerabilities

---

**Last Updated**: 2025-11-15
**Version**: 0.3.0 (Streamlined & Action-Oriented)
**Status**: Proposed

> "Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away." — Antoine de Saint-Exupéry
