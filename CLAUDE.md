# CLAUDE: Core Library Architecture & Unified Design Ethos

## Quick Reference

**Current Phase**: Phase 1.5 - Testing & Dependency Infrastructure ⚡

**Documentation Map**:
- 📋 `ROADMAP.md` - Project phases and timeline
- 📊 `TESTING_AND_DEPENDENCIES_SUMMARY.md` - Testing strategy overview
- 📦 `docs/GIT_SUBTREE_VENDORING.md` - Verifiable dependency vendoring
- 🧪 `docs/TESTING_STRATEGY.md` - 5-level testing pyramid
- 🔧 `docs/DECODER_DEPENDENCY_STRATEGY.md` - Pure Rust decoder pattern

**Next Actions** (Week 1):
1. Vendor `hex` using git subtree (see below)
2. Move `serde_json` to dev-dependencies
3. Benchmark `smallvec` vs `Vec`
4. Move blockchain libs to dev-dependencies

---

## Design Philosophy

This document outlines the **fundamental design criteria** for the Universal Blockchain Decoder. These principles are **immutable** and guide all architectural decisions.

## Core Principle: Minimal Trusted Computing Base (TCB)

> "The best code is no code. The second best is code that can be formally verified."

### Objective

Create a **small, reviewable, formally verifiable core library** that serves as the trusted foundation for all blockchain transaction decoding, while allowing **unlimited extensibility through external implementations**.

## Design Criteria

### 1. Minimal Core ⚡

**Goal**: Core library should be < 3000 LOC

**Why**: Smaller core = easier audit, faster verification, fewer bugs

**How**:
- Core defines **traits**, not implementations
- Core provides **types** and **guarantees**, not algorithms
- Chain-specific logic lives in **separate crates**
- Use **trait-based extensibility**, not enum-based closed systems

**Example**:

```rust
// ✅ GOOD: Core defines behavior
pub trait ChainIdentity {
    fn chain_id(&self) -> u64;
    fn chain_name(&self) -> &str;
}

// ❌ BAD: Core enumerates all chains
pub enum ChainId {
    Bitcoin,
    Ethereum,
    // ... endless additions
}
```

**Rationale**: Enums require core changes for new chains, bloating the TCB. Traits allow extension without modification.

### 2. Formally Verifiable 🔬

**Goal**: Core library amenable to formal verification with Verus

**Requirements**:
- No `unsafe` code in core
- Explicit preconditions and postconditions
- Provable panic-freedom
- Deterministic behavior

**Critical Properties to Verify**:

1. **Injectivity of Canonicalization**:
   ```
   ∀ tx_bytes: encode(canonicalize(decode(tx_bytes))) = tx_bytes
   ```

2. **Panic-Freedom**:
   ```
   ∀ input: decode(input) either returns Result::Ok or Result::Err (never panics)
   ```

3. **Determinism**:
   ```
   ∀ tx: to_canonical_bytes(tx) = to_canonical_bytes(tx)
   ```

4. **Resource Bounds**:
   ```
   ∀ tx: size(canonical_repr(tx)) ≤ K * size(tx) for constant K
   ```

**Verification Strategy**:
- Phase 1: Verify core traits and types (2 months)
- Phase 2: Verify canonical serialization (2 months)
- Phase 3: Verify reference implementations (4 months)

**See**: `docs/FORMAL_VERIFICATION.md` for detailed plan

### 3. Reviewable & Auditable 📖

**Goal**: Any security expert can audit the core in one sitting

**Requirements**:
- Clear module boundaries
- Minimal dependencies (only `serde`, `borsh`, `thiserror`, crypto primitives)
- Comprehensive inline documentation
- No "clever" code - prefer explicit over implicit
- Every public function has safety documentation

**Audit Checklist**:
- [ ] Core library < 3000 LOC
- [ ] No `unsafe` blocks
- [ ] All panics documented and justified
- [ ] All arithmetic operations checked for overflow
- [ ] All array accesses bounds-checked
- [ ] All error paths explicitly handled
- [ ] Canonical serialization uses battle-tested library (Borsh)

### 4. Trait-Based Extensibility 🔌

**Goal**: Zero core changes to add new blockchains

**Anti-Pattern**: Enum-based chains (current design)

```rust
// ❌ CLOSED: Requires core modification
pub enum ChainId {
    Bitcoin,
    Ethereum,
    Solana,
    // To add Dogecoin, must edit this!
}
```

**Correct Pattern**: Trait-based chains

```rust
// ✅ OPEN: Implement trait in external crate
pub trait ChainIdentity: Send + Sync {
    fn chain_id(&self) -> u64;
    fn chain_name(&self) -> &str;
    fn chain_family(&self) -> ChainFamily;
}

// In decoder-dogecoin (separate crate)
pub struct DogecoinChain;
impl ChainIdentity for DogecoinChain {
    fn chain_id(&self) -> u64 { 42 }
    fn chain_name(&self) -> &str { "Dogecoin" }
    fn chain_family(&self) -> ChainFamily { ChainFamily::Utxo }
}
```

**Rationale**:
- Preserves minimal TCB (no core changes)
- Enables ecosystem growth
- Each decoder is independently audited
- Follows open-closed principle

**See**: `docs/TRAIT_BASED_ARCHITECTURE.md`

### 5. Canonical Serialization (Non-Negotiable) 🔐

**Requirement**: Core must enforce deterministic encoding

**Anti-Pattern**: JSON serialization

```rust
// ❌ DANGEROUS: JSON is not canonical
let json = serde_json::to_string(&tx_ir)?;
let hash = sha256(json.as_bytes()); // INSECURE
```

**Correct Pattern**: Borsh serialization

```rust
// ✅ SAFE: Borsh is canonical
let bytes = tx_ir.to_canonical_bytes()?;
let hash = sha256(&bytes); // Deterministic
```

**Rationale**:
- JSON has undefined key ordering
- JSON has variable whitespace
- JSON breaks signature verification
- Borsh guarantees determinism

**Non-Negotiable Rules**:
1. **NEVER** use JSON for hashing
2. **NEVER** use JSON for signature verification
3. **ALWAYS** use Borsh for canonical representation
4. JSON is **ONLY** for human display

**See**: `docs/CANONICAL_SERIALIZATION.md`

### 6. Zero-Cost Abstractions ⚡

**Goal**: No runtime overhead from the architecture

**Techniques**:
- Static dispatch (generics) over dynamic dispatch (trait objects)
- Const generics for compile-time constraints
- Inline-able functions
- Zero-sized types (PhantomData)

**Example**:

```rust
// ✅ Static dispatch (monomorphized at compile time)
pub fn decode_transaction<D: ChainDecoder>(bytes: &[u8]) -> Result<D::TxSpecific> {
    D::decode(bytes)
}

// ❌ Dynamic dispatch (vtable lookup at runtime)
pub fn decode_transaction(decoder: &dyn ChainDecoder, bytes: &[u8]) -> Result<Box<dyn Any>> {
    decoder.decode(bytes)
}
```

**Trade-off**: Static dispatch increases compile time but eliminates runtime cost. For a foundational library, we prefer runtime performance.

### 7. Layered Security Architecture 🛡️

```
┌────────────────────────────────────────────┐
│  Chain Decoders (External, Untrusted)     │  ← Anyone can implement
│  - decoder-bitcoin                         │  ← Independent audit
│  - decoder-ethereum                        │  ← Ecosystem-driven
│  - decoder-custom-chain                    │
└──────────────┬─────────────────────────────┘
               │ ChainDecoder trait
               │ Canonicalizer trait
               ▼
┌────────────────────────────────────────────┐
│  Core Library (Minimal, Trusted)          │  ← Formally verified
│  - Trait definitions                       │  ← < 3000 LOC
│  - TxIR type                               │  ← Security-critical
│  - Canonical serialization                 │  ← Must be correct
│  - Error types                             │
└────────────────────────────────────────────┘
```

**Rationale**:
- Core defines **what** (traits, types, guarantees)
- Decoders implement **how** (parsing, validation)
- Core is small → can be verified
- Decoders are pluggable → can be untrusted

**Security Model**:
1. Core library: **Trusted** (formally verified)
2. Decoder libraries: **Untrusted** (user responsibility to audit)
3. Application: **Composition** (trust propagates through types)

### 8. Supply Chain Security 🔗

**Goal**: Minimize dependencies in core, enable vendoring

**Core Dependencies (Minimal)**:
```toml
[dependencies]
# Serialization (battle-tested)
serde = { version = "1.0", features = ["derive"] }
borsh = { version = "1.3", features = ["derive"] }

# Error handling (std-like)
thiserror = "1.0"

# Cryptography (audited)
sha2 = "0.10"
sha3 = "0.10"
```

**Rationale**:
- Each dependency is a trust boundary
- Fewer dependencies = smaller attack surface
- Well-known crates have been audited
- Can vendor all dependencies for air-gapped deployments

**Airgapped Operation Requirement** 🔒:

**CRITICAL**: This system **MUST** work completely offline for security-critical deployments.

**Strategy**:
1. **Vendor Rust dependencies**: Using git subtree (e.g., `hex` crate)
2. **Vendor chain registries**: Using git subtree for all external data sources
3. **Compile-time embedding**: All data embedded in binary at build time
4. **No runtime network calls**: Zero network dependencies in production code

**Chain Registry Vendoring**:
```bash
# EVM chains (500+)
git subtree add \
    --prefix crates/decoder-evm/vendored/chainlist \
    https://github.com/ethereum-lists/chains.git \
    master --squash

# Cosmos chains (100+)
git subtree add \
    --prefix crates/decoder-cosmos-sdk/vendored/chain-registry \
    https://github.com/cosmos/chain-registry.git \
    master --squash

# OP Stack chains
git subtree add \
    --prefix crates/decoder-op-stack/vendored/superchain-registry \
    https://github.com/ethereum-optimism/superchain-registry.git \
    main --squash
```

**Build-time Embedding**:
```rust
// crates/decoder-evm/build.rs
fn main() {
    // Embed vendored chain data at compile time
    let chains_dir = "vendored/chainlist/_data/chains";
    // Generate Rust code from JSON
    // Result: Zero runtime dependencies
}
```

**Benefits**:
- ✅ Complete offline operation (financial institutions/banks/enterprise)
- ✅ Verifiable supply chain (git commit audit trail)
- ✅ Reproducible builds (all data in repo)
- ✅ No TOCTOU attacks (data can't change at runtime)
- ✅ Faster startup (no network I/O)

**Forbidden in Production Code**:
- ❌ HTTP/HTTPS clients (reqwest, ureq, etc.)
- ❌ DNS resolution
- ❌ Runtime config fetching from URLs
- ❌ Dynamic chain registry updates
- ✅ All network code MUST be in dev-dependencies only (for testing)

**Forbidden in Core**:
- Complex parsing libraries (use in decoders)
- Network libraries (violates airgapped requirement)
- Async runtime
- Proc macros (except derive)

### 9. Testing Strategy 🧪

**Levels**:

1. **Unit Tests** (Every public function):
   ```rust
   #[test]
   fn test_canonical_bytes_deterministic() {
       let tx = create_test_tx();
       assert_eq!(tx.to_canonical_bytes(), tx.to_canonical_bytes());
   }
   ```

2. **Property-Based Tests** (with proptest):
   ```rust
   proptest! {
       #[test]
       fn canonicalize_is_deterministic(tx in arbitrary_tx()) {
           let b1 = tx.to_canonical_bytes()?;
           let b2 = tx.to_canonical_bytes()?;
           prop_assert_eq!(b1, b2);
       }
   }
   ```

3. **Formal Verification** (with Verus):
   ```rust
   verus! {
       #[verifier::proof]
       fn canonicalize_injective(tx_bytes: &[u8])
           requires valid_tx_bytes(tx_bytes)
           ensures encode(decode(tx_bytes)) == tx_bytes
       { /* proof */ }
   }
   ```

4. **Integration Tests** (Real blockchain data):
   ```rust
   #[test]
   fn decode_bitcoin_block_100000() {
       let tx_bytes = include_bytes!("fixtures/btc_block_100000_tx0.bin");
       let tx = BitcoinDecoder::decode(tx_bytes).unwrap();
       assert_eq!(tx.version(), 1);
   }
   ```

### 10. Documentation as Code 📝

**Requirements**:
- Every public type has module-level docs
- Every public function has examples
- Complex algorithms have inline explanations
- Architecture decisions documented in `/docs`

**Example**:

```rust
/// Canonical intermediate representation for blockchain transactions.
///
/// The TxIR normalizes transactions from different blockchain models
/// (UTXO, Account, Instruction) into a unified semantic structure.
///
/// # Formal Properties
///
/// 1. **Deterministic Encoding**: `encode(x) = encode(x)`
/// 2. **Injective**: `encode(x) = encode(y) ⟹ x = y`
///
/// # Example
///
/// ```
/// use universal_decoder_core::prelude::*;
///
/// let tx_ir = TxIR::new(
///     &BitcoinChain,
///     metadata,
///     authorization,
///     operations,
///     state_deltas,
/// );
///
/// let canonical_bytes = tx_ir.to_canonical_bytes()?;
/// let hash = tx_ir.canonical_hash()?;
/// ```
///
/// # Security
///
/// **CRITICAL**: Do not use JSON for canonical representation.
/// See `docs/CANONICAL_SERIALIZATION.md` for details.
pub struct TxIR<'a, const V: u8> { /* ... */ }
```

## Current Status vs Goals

| Criterion | Current | Goal | Phase 1.5 Target | Status |
|-----------|---------|------|------------------|--------|
| Core LOC | ~2500 | < 3000 | ~2700 | ✅ Good |
| Core dependencies | 8 | ≤ 5 | 5 (hex vendored) | 🚧 In Progress |
| Decoder dependencies | Yes (bitcoin, ethers) | 0 (pure Rust) | 0 (dev-deps only) | 📋 Planned |
| Formal verification | No | Verus annotations | Basic annotations | 📋 Week 2 |
| Canonical serialization | Borsh ✅ | Borsh ✅ | Borsh ✅ | ✅ Done |
| Test coverage | 0% | 100% core, 90% decoders | 50%+ | 🚧 Week 2 |
| Property tests | 0 | 50+ | 20+ | 📋 Week 2 |
| Integration tests | 0 | 100+ fixtures | 50+ | 📋 Week 2 |
| Fuzz testing | No | Continuous | Infrastructure | 📋 Week 2 |
| CI/CD | No | Comprehensive | Full pipeline | 📋 Week 2 |
| Documentation | Architecture | Testing + Deps | Complete | ✅ Done |

## Phased Development & Stacked PRs

### Current: Phase 1.5 - Testing & Dependency Infrastructure (2 weeks)

**Week 1: Dependency Minimization**
- PR #1: Vendor `hex` using git subtree
- PR #2: Move `serde_json` to dev-dependencies
- PR #3: Benchmark `smallvec`, decide keep/remove
- PR #4: Move blockchain libs to dev-dependencies

**Week 2: Testing Infrastructure**
- PR #5: Set up test organization + first unit tests
- PR #6: Add property-based testing (proptest)
- PR #7: Create test fixture repository + integration tests
- PR #8: Set up CI/CD pipeline (GitHub Actions)
- PR #9: Add fuzzing infrastructure (cargo-fuzz)
- PR #10: Install Verus + first annotations

**Stacking Strategy**:
```bash
# Create feature branches stacked on each other
git checkout -b phase1.5/vendor-hex
# ... implement PR #1 ...
git push -u origin phase1.5/vendor-hex

# Stack PR #2 on top of PR #1
git checkout -b phase1.5/move-serde-json phase1.5/vendor-hex
# ... implement PR #2 ...
git push -u origin phase1.5/move-serde-json

# Continue stacking...
```

**Merge Strategy**: Merge PRs sequentially (#1 → #2 → #3...) after each passes CI

### Next: Phase 2 - Pure Rust Decoders (6-8 weeks)

**Weeks 1-2: Bitcoin Decoder**
- PR #11-15: Pure Rust Bitcoin transaction parsing
- PR #16-18: Validation tests against `bitcoin` crate
- PR #19-20: Property tests + fuzzing

**Weeks 3-4: Ethereum Decoder**
- PR #21-25: Pure Rust RLP + Ethereum parsing
- PR #26-28: Validation tests against `ethers-core`
- PR #29-30: Property tests + fuzzing

**Weeks 5-6: Integration**
- PR #31-33: End-to-end tests with real blockchain data
- PR #34-35: Performance benchmarking
- PR #36: Documentation updates

### Future Phases

**Phase 3** (Months 3-4): Extended Chain Support (Solana, Cardano, Polkadot)
**Phase 4** (Months 4-5): Formal Verification (Verus proofs)
**Phase 5** (Months 5-6): Security Audit & Production Hardening
**Phase 6** (Month 6): v1.0.0 Release 🎉

**See**: `ROADMAP.md` for detailed timeline

---

## Quick Start: Phase 1.5 Implementation

### PR #1: Vendor `hex` using Git Subtree

**Goal**: Replace external `hex` dependency with vendored version using git subtree for maximum verifiability.

**Commands**:
```bash
# 1. Add hex as git subtree
cd /home/user/universal-blockchain-decoder
git subtree add \
    --prefix crates/universal-decoder-core/src/vendored/hex \
    https://github.com/KokaKiwi/rust-hex.git \
    v0.4.3 \
    --squash

# 2. Create integration module
cat > crates/universal-decoder-core/src/vendored/mod.rs <<'EOF'
pub mod hex {
    include!("hex/src/lib.rs");
}
pub use hex::FromHexError;
EOF

# 3. Re-export in core
# Edit crates/universal-decoder-core/src/lib.rs:
#   mod vendored;
#   pub use vendored::hex;

# 4. Update Cargo.toml
# Remove: hex = "0.4" from [dependencies]
# Add: hex = "0.4.3" to [dev-dependencies] (for validation)

# 5. Update imports in all decoder crates
find crates/decoder-* -name "*.rs" -type f \
    -exec sed -i 's/use hex::/use universal_decoder_core::hex::/g' {} \;

# 6. Write validation tests
# crates/universal-decoder-core/tests/vendored_hex_validation.rs

# 7. Run tests
cargo test --all

# 8. Verify dependency count
cargo tree -p universal-decoder-core | grep -v "└──" | wc -l
# Should show 5 dependencies (not counting hex)

# 9. Commit
git add -A
git commit -m "Vendor hex crate using git subtree for verifiable supply chain"
git push -u origin phase1.5/vendor-hex
```

**Validation**:
- ✅ `hex` not in production dependencies
- ✅ Git history shows exact upstream commit
- ✅ Can verify: `git diff v0.4.3 -- crates/.../vendored/hex`
- ✅ All tests pass
- ✅ Decoders can use `universal_decoder_core::hex`

**See**: `docs/GIT_SUBTREE_VENDORING.md` for detailed guide

### PR #2: Move `serde_json` to dev-dependencies

**Goal**: Remove `serde_json` from production dependencies (JSON is for display only, not canonical encoding).

**Commands**:
```bash
git checkout -b phase1.5/move-serde-json phase1.5/vendor-hex

# 1. Update Cargo.toml
# crates/universal-decoder-core/Cargo.toml:
#   [dependencies]
#   - Remove: serde_json = "1.0"
#   [dev-dependencies]
#   + Add: serde_json = "1.0"

# 2. Remove public JSON APIs (if any)
# Search for public methods that use serde_json and move to tests

# 3. Ensure JSON only in tests
grep -r "serde_json" crates/universal-decoder-core/src/
# Should only appear in #[cfg(test)] blocks

# 4. Run tests
cargo test --all

# 5. Commit
git commit -am "Move serde_json to dev-dependencies (display/test only)"
git push -u origin phase1.5/move-serde-json
```

**Validation**:
- ✅ `serde_json` not in production dependencies
- ✅ JSON only used in tests
- ✅ Canonical encoding uses Borsh only
- ✅ All tests pass

### PR #3: Benchmark `smallvec`

**Goal**: Decide whether to keep, remove, or vendor `smallvec` based on performance data.

**Commands**:
```bash
git checkout -b phase1.5/benchmark-smallvec phase1.5/move-serde-json

# 1. Create benchmark
mkdir -p crates/universal-decoder-core/benches
cat > crates/universal-decoder-core/benches/vec_vs_smallvec.rs <<'EOF'
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smallvec::SmallVec;

fn bench_vec(c: &mut Criterion) {
    c.bench_function("vec_5_elements", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..5 {
                v.push(black_box(i));
            }
            v
        })
    });
}

fn bench_smallvec(c: &mut Criterion) {
    c.bench_function("smallvec_5_elements", |b| {
        b.iter(|| {
            let mut v = SmallVec::<[u32; 8]>::new();
            for i in 0..5 {
                v.push(black_box(i));
            }
            v
        })
    });
}

criterion_group!(benches, bench_vec, bench_smallvec);
criterion_main!(benches);
EOF

# 2. Run benchmarks
cargo bench --bench vec_vs_smallvec

# 3. Analyze results
# If SmallVec is < 10% faster: REMOVE it
# If SmallVec is > 10% faster: KEEP or VENDOR it

# 4. Make decision and update code accordingly
# Option A: Remove smallvec
#   - Replace SmallVec with Vec throughout codebase
# Option B: Keep smallvec
#   - Document performance justification
# Option C: Vendor smallvec (if critical + want control)

# 5. Commit
git commit -am "Benchmark smallvec vs Vec: [decision]"
git push -u origin phase1.5/benchmark-smallvec
```

### PR #4: Move Blockchain Libs to dev-dependencies

**Goal**: Decoders use pure Rust parsing; blockchain libs only for test validation.

**Commands**:
```bash
git checkout -b phase1.5/decoders-dev-deps phase1.5/benchmark-smallvec

# 1. Update decoder Cargo.toml files
# crates/decoder-bitcoin/Cargo.toml:
#   [dependencies]
#   - Remove: bitcoin = "0.31"
#   [dev-dependencies]
#   + Add: bitcoin = "0.31"

# crates/decoder-ethereum/Cargo.toml:
#   [dependencies]
#   - Remove: ethers-core (if present)
#   [dev-dependencies]
#   + Add: alloy = "0.1"  # Modern Ethereum library (successor to ethers)
#   + Add: alloy-primitives = "0.7"
#   + Add: alloy-rlp = "0.3"

# 2. Document that decoders are pure Rust
# Add to each decoder's README:
#   "Pure Rust implementation. Blockchain libraries (bitcoin, alloy-rs) are in dev-dependencies for test validation only."

# 3. Ensure no production code uses blockchain libs
grep -r "use bitcoin::" crates/decoder-bitcoin/src/
grep -r "use alloy" crates/decoder-ethereum/src/
grep -r "use ethers" crates/decoder-ethereum/src/
# Should find no matches

# 4. Run tests (will fail until we implement pure Rust parsing in Phase 2)
cargo test --all || echo "Expected: decoders need pure Rust impl (Phase 2)"

# 5. Commit
git commit -am "Move blockchain libs to dev-dependencies (pure Rust strategy)"
git push -u origin phase1.5/decoders-dev-deps
```

**Note**: This PR documents the strategy. Actual pure Rust implementations come in Phase 2.

### Success Criteria for Phase 1.5

After all PRs merged:
- ✅ Core has ≤ 5 production dependencies
- ✅ `hex` vendored via git subtree (verifiable)
- ✅ `serde_json` in dev-dependencies only
- ✅ `smallvec` benchmarked and decision made
- ✅ Blockchain libs in dev-dependencies
- ✅ Strategy documented for pure Rust decoders

**Dependency Count Check**:
```bash
# Count production dependencies
cargo tree -p universal-decoder-core -e normal --depth 1 | grep -v "^universal" | wc -l
# Should be ≤ 5
```

**Next**: Phase 2 - Implement pure Rust decoders

---

## Decision Log

### Why Borsh over Protobuf?

**Decision**: Use Borsh for canonical serialization

**Rationale**:
- Borsh is designed specifically for deterministic encoding
- Simpler than Protobuf (no schema management)
- Native Rust support
- Battle-tested in Solana, NEAR

**Trade-offs**:
- Protobuf has better cross-language support
- But we prioritize canonicity over interop

### Why Traits over Enums?

**Decision**: Move from enum-based to trait-based chains (v0.2.0)

**Rationale**:
- Enums violate open-closed principle
- Traits enable ecosystem growth
- Core stays minimal and verifiable
- Each decoder is independently audited

**Migration**: v0.1.0 uses enums (temporary), v0.2.0 will use traits

### Why Static Dispatch?

**Decision**: Prefer static dispatch (generics) over dynamic dispatch (trait objects)

**Rationale**:
- Zero-cost abstraction
- Monomorphization enables better optimization
- Preserves type information
- Simpler for formal verification

**Trade-offs**:
- Increases compile time
- Cannot store different decoders in homogeneous collections
- But core is compile-time library, so this is acceptable

## Contributing

All contributions must adhere to these design criteria:

1. ✅ **No core changes for new chains** (use traits)
2. ✅ **Maintain formal verifiability** (no unsafe, explicit contracts)
3. ✅ **Preserve minimal TCB** (< 3000 LOC core)
4. ✅ **Use canonical serialization** (Borsh, not JSON)
5. ✅ **Zero-cost abstractions** (static dispatch)
6. ✅ **Comprehensive tests** (unit + property + integration)
7. ✅ **Security-first** (audit-friendly code)

**See**: `CONTRIBUTING.md` for detailed guidelines

## References

1. **Formal Verification**: `docs/FORMAL_VERIFICATION.md`
2. **Canonical Serialization**: `docs/CANONICAL_SERIALIZATION.md`
3. **Trait-Based Architecture**: `docs/TRAIT_BASED_ARCHITECTURE.md`
4. **Verus**: https://github.com/verus-lang/verus
5. **Borsh**: https://borsh.io/
6. **Trusted Computing Base**: https://en.wikipedia.org/wiki/Trusted_computing_base

## Quotes

> "Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away."
> — Antoine de Saint-Exupéry

> "The best way to make software secure is to make it small."
> — Andrew Tannenbaum

> "Simplicity is prerequisite for reliability."
> — Edsger W. Dijkstra

---

**Last Updated**: 2025-01-XX
**Version**: 0.1.0
**Status**: Living Document
