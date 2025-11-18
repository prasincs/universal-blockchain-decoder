# CLAUDE: Core Library Architecture & Unified Design Ethos

## Quick Reference

**Current Phase**: Phase 1.5 - Testing & Dependency Infrastructure ⚡

**Documentation Map**:
- 📋 `ROADMAP.md` - Project phases and timeline
- 📊 `docs/TESTING_AND_DEPENDENCIES_SUMMARY.md` - Testing strategy overview
- 📦 `docs/GIT_SUBTREE_VENDORING.md` - Verifiable dependency vendoring
- 🧪 `docs/TESTING_STRATEGY.md` - 5-level testing pyramid
- 🔧 `docs/DECODER_DEPENDENCY_STRATEGY.md` - Pure Rust decoder pattern
- 🌐 `docs/WASM_DEMO.md` - Interactive browser-based demo (Phase 3.10)
- 🎭 `docs/ACTOR_MODEL_CHAINS.md` - Actor Model family (ICP, AO) - Implementation-ready ✅

**Next Actions**:
1. Complete OP Stack implementation (Phase 3.2) - 90% done, ~4 hours
2. Build WASM demo (Phase 3.10) - 1-2 weeks, perfect for papers/blogs/conferences
3. Implement Actor Model family (Phase 3.11) - ICP + AO decoders, 2-3 weeks
4. Add more property tests (Phase 1.5.2) - Need 34 more (currently 16/50)

---

## Design Philosophy

This document outlines the **fundamental design criteria** for the Universal Blockchain Decoder. These principles are **immutable** and guide all architectural decisions.

## Core Principle: Minimal Trusted Computing Base (TCB)

> "The best code is no code. The second best is code that can be formally verified."

### Objective

Create a **small, reviewable, formally verifiable core library** that serves as the trusted foundation for all blockchain transaction decoding, while allowing **unlimited extensibility through external implementations**.

---

## Project Scope: Decoding & Verification 🎯

**IMPORTANT**: This project focuses on **transaction decoding, analysis, and codec verification**.

**In Scope** ✅:
- **Decoding** blockchain transactions (chain-specific bytes → TxSpecific → TxIR)
- **Re-encoding for verification** (TxSpecific → chain-specific bytes)
  - **CRITICAL**: Must support roundtrip: `encode(decode(tx_bytes)) = tx_bytes` (injective property)
  - Purpose: Verify lossless decoding, forensic reconstruction, formal verification
  - **NOT** for building new transactions - only for encoding existing decoded transactions back to bytes
- **Canonical serialization** for hashing/analysis (TxIR → Borsh bytes)
- **Transaction validation** and structural analysis
- **Signature verification** (checking existing signatures)
- **Chain-agnostic intermediate representation** (TxIR)

**Out of Scope** ❌:
- **Transaction construction** (building new transactions from scratch)
- **Transaction signing** (creating new signatures)
- **Fee estimation** (requires chain state, mempool data)
- **UTXO selection** (wallet functionality)
- **Nonce management** (account state tracking)
- **Gas price estimation** (requires market data)
- **Transaction broadcasting** (network operations)

**Critical Distinction**:
- ✅ **Re-encoding** = `decoded_tx.to_bytes()` (reconstruct original bytes for verification)
- ❌ **Construction** = `TransactionBuilder::new().add_output(...).build()` (create new transactions)

**Rationale**:
1. **Injective Property is Mandatory**: Without re-encoding, we cannot verify `encode(decode(x)) = x`, which is a core formal property
2. **Forensics & Analysis**: Being able to reconstruct exact original bytes is critical for forensic work
3. **No State Dependencies**: Re-encoding a decoded transaction requires no external state (unlike construction)
4. **TCB Preservation**: Re-encoding existing structures is simple and auditable; construction is complex
5. **Clear Use Cases**:
   - ✅ Decode transaction → analyze → re-encode to verify no data loss
   - ❌ Build new transaction with fee estimation, UTXO selection, etc.

**See**: "Decision Log" section below for detailed analysis of re-encoding vs construction.

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

1. **Injectivity (Roundtrip Property)** - **MANDATORY**:
   ```
   ∀ tx_bytes: encode(decode(tx_bytes)) = tx_bytes
   ```
   This is the **fundamental requirement**: decoding must be lossless and reversible.
   Without this property, forensic analysis and verification are impossible.

2. **Panic-Freedom**:
   ```
   ∀ input: decode(input) either returns Result::Ok or Result::Err (never panics)
   ```

3. **Determinism of Canonical Encoding**:
   ```
   ∀ tx: to_canonical_bytes(tx) = to_canonical_bytes(tx)
   ```
   Note: This is for TxIR → Borsh bytes (canonical format), separate from chain-specific encoding.

4. **Resource Bounds**:
   ```
   ∀ tx: size(encode(decode(tx))) = size(tx) (exact reconstruction)
   ∀ tx: size(canonical_bytes(tx)) ≤ K * size(tx) for constant K
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
| Formal verification | Basic annotations | Verus annotations | Basic annotations | ✅ Done |
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
- ✅ PR #10: Install Verus + first annotations (COMPLETED - PR #36)

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

## Git Workflow & Quality Checks

### Pre-Commit Requirements (MANDATORY)

**CRITICAL**: Before every `git commit` and `git push`, you MUST run the following checks:

```bash
# 1. Format all code
cargo fmt --all

# 2. Run clippy with warnings as errors
cargo clippy --all --all-targets --all-features -- -D warnings

# 3. Run tests (optional but recommended)
cargo test --all
```

**Workflow**:
```bash
# Make your changes
# ...

# STEP 1: Format
cargo fmt --all

# STEP 2: Lint
cargo clippy --all --all-targets --all-features -- -D warnings

# STEP 3: Fix any warnings/errors from clippy
# (repeat STEP 1 & 2 until clean)

# STEP 4: Commit
git add .
git commit -m "Your commit message"

# STEP 5: Push
git push -u origin your-branch-name
```

### Why This Matters

1. **CI/CD Pipeline**: GitHub Actions runs these checks automatically. Failing to run them locally means your PR will fail CI.

2. **Code Quality**: Consistent formatting and linting ensures:
   - Readable code
   - Catches common bugs
   - Maintains project standards
   - Makes code review easier

3. **Time Savings**: Catching issues locally is faster than waiting for CI to fail.

### Common Clippy Fixes

```rust
// ❌ BAD: Borrowed expression implements required traits
Err(DecoderError::invalid_structure(&format!("error: {}", x)))

// ✅ GOOD: Direct ownership
Err(DecoderError::invalid_structure(format!("error: {}", x)))

// ❌ BAD: Length comparison to zero
if vec.len() > 0 { }

// ✅ GOOD: Use is_empty()
if !vec.is_empty() { }

// ❌ BAD: Useless vec! for single element
let items = vec!["single"];

// ✅ GOOD: Use array
let items = ["single"];
```

### Documentation Updates

When adding new features or making significant changes:
1. Update relevant documentation in `docs/`
2. Update `ROADMAP.md` to mark tasks complete
3. Add/update code comments for public APIs
4. Update examples if behavior changes

---

## Claude CLI Tools & Workflows 🤖

**IMPORTANT**: When working with Claude CLI on this project, use these tools efficiently to maximize productivity and maintain code quality.

### Core Tools Overview

```
┌─────────────────────────────────────────────────────────────┐
│  File Operations    │  Search & Discovery │  Execution      │
├─────────────────────┼─────────────────────┼─────────────────┤
│  Read              │  Glob               │  Bash           │
│  Edit              │  Grep               │  Task (Agents)  │
│  Write             │  Task (Explore)     │  TodoWrite      │
└─────────────────────────────────────────────────────────────┘
```

### 1. Task Tool: Exploring the Codebase 🔍

**When to Use**: Open-ended codebase exploration, understanding architecture, finding patterns

**DON'T** do this:
```bash
# ❌ Manually searching for patterns
grep -r "ChainDecoder" crates/
find . -name "*decoder*.rs"
```

**DO** this instead:
```
# ✅ Use Task tool with Explore agent
Task(
  subagent_type="Explore",
  prompt="Find all implementations of the ChainDecoder trait across the codebase.
  Show me where they are defined and what chains they support.",
  description="Explore ChainDecoder implementations"
)
```

**Examples**:

```bash
# Find all error handling patterns
Task(Explore): "How is error handling implemented? Find all custom error types
and show how they're used across decoders."

# Understand testing structure
Task(Explore): "What is the current testing structure? Find all test files
and describe the testing patterns used."

# Find security-critical code
Task(Explore): "Find all uses of 'unsafe' code, crypto operations, and
serialization logic that are security-critical."
```

**Thoroughness Levels**:
- `quick`: Fast scan, top-level overview
- `medium`: Balanced exploration (recommended)
- `very thorough`: Deep dive, comprehensive analysis

### 2. TodoWrite Tool: Task Management 📋

**MANDATORY**: Use for all non-trivial tasks (3+ steps)

**When to Use**:
- Multi-step implementations
- Complex refactoring
- Following Phase 1.5 PRs
- User provides multiple tasks

**Task States**:
```rust
pending      // Not started
in_progress  // Currently working (EXACTLY ONE at a time)
completed    // Finished successfully
```

**Example Workflow**:

```rust
// STEP 1: Create todo list at start
TodoWrite([
    { content: "Vendor hex crate using git subtree",
      activeForm: "Vendoring hex crate",
      status: "pending" },
    { content: "Update imports in decoder crates",
      activeForm: "Updating imports",
      status: "pending" },
    { content: "Run validation tests",
      activeForm: "Running validation tests",
      status: "pending" },
    { content: "Commit and push changes",
      activeForm: "Committing changes",
      status: "pending" }
])

// STEP 2: Mark first task in_progress BEFORE starting
TodoWrite([
    { content: "Vendor hex crate using git subtree",
      activeForm: "Vendoring hex crate",
      status: "in_progress" },  // ← Changed
    { content: "Update imports in decoder crates",
      activeForm: "Updating imports",
      status: "pending" },
    // ... rest pending
])

// STEP 3: Complete IMMEDIATELY after finishing
TodoWrite([
    { content: "Vendor hex crate using git subtree",
      activeForm: "Vendoring hex crate",
      status: "completed" },  // ← Changed
    { content: "Update imports in decoder crates",
      activeForm: "Updating imports",
      status: "in_progress" },  // ← Next task
    // ... rest
])
```

**Rules**:
- ✅ Create todos for complex tasks (3+ steps)
- ✅ Update status in real-time
- ✅ EXACTLY ONE task `in_progress` at a time
- ✅ Complete tasks IMMEDIATELY (don't batch)
- ❌ Don't use for trivial single-step tasks
- ❌ Don't leave tasks as `in_progress` if blocked/failed

### 3. File Operations: Read, Edit, Write 📄

**Read Tool**: View file contents

```rust
// ✅ GOOD: Read files in parallel when independent
Read("crates/universal-decoder-core/Cargo.toml")
Read("crates/decoder-bitcoin/Cargo.toml")
Read("crates/decoder-ethereum/Cargo.toml")
// Send all three Read calls in a SINGLE message

// ❌ BAD: Sequential reads when parallel is possible
Read("file1.rs")  // wait for result
Read("file2.rs")  // then read this
Read("file3.rs")  // then read this
```

**Edit Tool**: Modify existing files

```rust
// ✅ GOOD: Preserve exact indentation from Read output
// When you see:
//   123→    fn example() {
//   124→        let x = 5;
//   125→    }
//
// The actual file content (after line number prefix) is:
//     "    fn example() {\n        let x = 5;\n    }"
//
// Use EXACTLY that indentation in old_string and new_string

Edit(
    file_path="src/lib.rs",
    old_string="    fn example() {\n        let x = 5;\n    }",
    new_string="    fn example() {\n        let x = 10;\n    }"
)

// ❌ BAD: Including line numbers or wrong indentation
Edit(
    old_string="123→    fn example() {",  // Wrong: includes line prefix
    ...
)
```

**Write Tool**: Create new files (use sparingly!)

```rust
// ⚠️ PREFER editing existing files over creating new ones
// Only use Write when:
// 1. User explicitly requests a new file
// 2. No existing file fits the purpose

// ❌ BAD: Creating new file when existing one could be edited
Write("docs/NEW_ARCHITECTURE.md", content)

// ✅ GOOD: Edit existing documentation
Edit("docs/ARCHITECTURE.md", old, new)
```

### 4. Search Tools: Grep & Glob 🔎

**Glob**: Find files by pattern

```rust
// Find all Rust files in decoder crates
Glob(pattern="crates/decoder-*/src/**/*.rs")

// Find all test files
Glob(pattern="crates/**/tests/**/*.rs")

// Find all Cargo.toml files
Glob(pattern="**/Cargo.toml")
```

**Grep**: Search file contents

```rust
// ✅ GOOD: Parallel searches for independent patterns
Grep(pattern="use hex::", output_mode="files_with_matches")
Grep(pattern="use serde_json::", output_mode="files_with_matches")
Grep(pattern="ChainDecoder", output_mode="content", -C=3)

// Find all TODO comments
Grep(pattern="TODO|FIXME", output_mode="content")

// Find unsafe code
Grep(pattern="unsafe ", output_mode="content", -B=2, -A=5)

// Case-insensitive search
Grep(pattern="decoder", -i=true, output_mode="files_with_matches")
```

**Output Modes**:
- `files_with_matches`: Show only file paths (default, fast)
- `content`: Show matching lines with context
- `count`: Show match counts per file

**Context Options** (only with `output_mode="content"`):
- `-A=N`: Show N lines after match
- `-B=N`: Show N lines before match
- `-C=N`: Show N lines before and after match
- `-n=true`: Show line numbers (default)

### 5. Bash Tool: Running Commands 🔧

**Use For**:
- Git operations
- Cargo commands (build, test, clippy, fmt)
- System operations
- Docker, npm, etc.

**DON'T Use For**:
- ❌ File reading (`cat` → use `Read`)
- ❌ File searching (`find`, `grep` → use `Glob`, `Grep`)
- ❌ File editing (`sed`, `awk` → use `Edit`)
- ❌ File writing (`echo >`, `cat <<EOF` → use `Write`)
- ❌ Communication (`echo "message"` → use text output)

**Parallel vs Sequential**:

```bash
# ✅ GOOD: Independent commands in parallel (single message, multiple Bash calls)
Bash("git status")
Bash("git diff")
Bash("cargo tree -p universal-decoder-core")

# ✅ GOOD: Dependent commands sequentially (single Bash call with &&)
Bash("cargo fmt --all && cargo clippy --all --all-targets -- -D warnings && cargo test --all")

# ❌ BAD: Sequential commands that could be parallel
Bash("git status")  // wait
Bash("git diff")    // then this
```

**Common Patterns**:

```bash
# Pre-commit checks (sequential, must pass)
Bash("cargo fmt --all && cargo clippy --all --all-targets --all-features -- -D warnings")

# Parallel status checks
Bash("git status")
Bash("cargo tree -p universal-decoder-core -e normal --depth 1")
Bash("cargo --version")

# File path quoting (CRITICAL for paths with spaces)
Bash('cd "/path/with spaces" && ls')  # ✅ Correct
Bash('cd /path/with spaces && ls')    # ❌ Will fail
```

### 6. Parallel Tool Execution ⚡

**Rule**: When tool calls are independent, make them in parallel (single message)

**Example: Starting Phase 1.5 PR #1**

```rust
// ✅ OPTIMAL: All independent operations in ONE message
Read("crates/universal-decoder-core/Cargo.toml")
Read("crates/decoder-bitcoin/Cargo.toml")
Read("crates/decoder-ethereum/Cargo.toml")
Bash("git status")
Bash("git log --oneline -5")
Grep(pattern="use hex::", output_mode="files_with_matches")

// ❌ SLOW: Sequential when parallel is possible
// Message 1: Read("Cargo.toml")
// Wait for result...
// Message 2: Bash("git status")
// Wait for result...
// Message 3: Grep(...)
```

**When to Use Sequential**:
```rust
// ✅ Correct: Second command depends on first
Bash("mkdir -p new_dir && cp file.txt new_dir/")

// ✅ Correct: Need to see results before next action
Read("config.toml")
// [analyze results, decide what to do next]
Edit("config.toml", old, new)
```

### 7. Project-Specific Workflows 🔄

#### Workflow: Pre-Commit Checks

```bash
# ALWAYS run before git commit (in ONE sequential Bash call)
Bash("cargo fmt --all && cargo clippy --all --all-targets --all-features -- -D warnings")

# If warnings found, fix them and repeat
# Optional but recommended:
Bash("cargo test --all")
```

#### Workflow: Dependency Analysis

```bash
# Parallel: Check current state
Bash("cargo tree -p universal-decoder-core -e normal --depth 1")
Bash("cargo tree -p decoder-bitcoin -e normal --depth 1")
Read("crates/universal-decoder-core/Cargo.toml")
Grep(pattern='hex = ', path="crates", output_mode="content")
```

#### Workflow: Vendoring a Dependency (e.g., hex)

```rust
// STEP 1: Create todo list
TodoWrite([...])

// STEP 2: Add git subtree (sequential, must succeed)
Bash("git subtree add --prefix crates/universal-decoder-core/src/vendored/hex https://github.com/KokaKiwi/rust-hex.git v0.4.3 --squash")

// STEP 3: Update todos
TodoWrite([...mark first completed, next in_progress...])

// STEP 4: Parallel file operations
Write("crates/universal-decoder-core/src/vendored/mod.rs", content)
Read("crates/universal-decoder-core/src/lib.rs")
Read("crates/universal-decoder-core/Cargo.toml")

// STEP 5: Edit files based on reads
Edit(...)
Edit(...)

// STEP 6: Validate
Bash("cargo test --all")
Bash("cargo tree -p universal-decoder-core -e normal")
```

#### Workflow: Exploring Codebase for Architecture Understanding

```rust
// Use Task tool with Explore agent
Task(
    subagent_type="Explore",
    prompt="I need to understand the trait-based architecture. Find:
    1. All trait definitions in universal-decoder-core
    2. All implementations of these traits in decoder crates
    3. Examples of how decoders use these traits
    Provide a summary of the architecture pattern.",
    description="Explore trait architecture",
    model="haiku"  // Optional: use faster model for quick exploration
)
```

### 8. Common Mistakes to Avoid ❌

```rust
// ❌ Using Bash for file reading
Bash("cat src/lib.rs")
// ✅ Use Read instead
Read("src/lib.rs")

// ❌ Using Bash for searching
Bash("grep -r 'pattern' src/")
// ✅ Use Grep instead
Grep(pattern="pattern", path="src", output_mode="content")

// ❌ Using echo for communication
Bash("echo 'Starting to vendor hex crate...'")
// ✅ Just output text directly
"I'm going to vendor the hex crate now..."

// ❌ Sequential reads when parallel is better
Read("file1.rs")  // Message 1
// [wait]
Read("file2.rs")  // Message 2
// ✅ Parallel reads in one message
Read("file1.rs")
Read("file2.rs")

// ❌ Not using TodoWrite for complex tasks
[Starts multi-step task without todo list]
// ✅ Create todo list first
TodoWrite([...all steps...])

// ❌ Forgetting pre-commit checks
Edit(...)
Bash("git commit -m 'message'")
// ✅ Always fmt + clippy first
Edit(...)
Bash("cargo fmt --all && cargo clippy --all --all-targets --all-features -- -D warnings")
Bash("git add . && git commit -m 'message'")
```

### 9. Performance Tips ⚡

1. **Batch Independent Operations**: Send multiple independent tool calls in one message
2. **Use Explore Agent**: For open-ended searches, don't grep/glob manually
3. **Use Haiku Model**: For quick tasks, specify `model="haiku"` in Task tool
4. **Glob Before Grep**: Find files first with Glob, then Grep specific files
5. **Limit Output**: Use `head_limit` parameter in Grep for large codebases
6. **Read Once**: Don't re-read files unnecessarily; store info from first read

### 10. Quick Reference Card 📇

```
SEARCH & DISCOVERY
  Glob(pattern="**/*.rs")           → Find files by pattern
  Grep(pattern="...", output_mode)  → Search file contents
  Task(Explore, prompt="...")       → Open-ended exploration

FILE OPERATIONS
  Read("path/to/file")              → View file contents
  Edit(file_path, old, new)         → Modify existing file
  Write(file_path, content)         → Create new file (avoid!)

EXECUTION
  Bash("command")                   → Run shell commands
  TodoWrite([...])                  → Track multi-step tasks
  Task(subagent_type, prompt)       → Launch specialized agents

REMEMBER
  ✅ Parallel: Independent operations in ONE message
  ✅ Sequential: Dependent operations with && or separate messages
  ✅ TodoWrite: All complex tasks (3+ steps)
  ✅ Pre-commit: cargo fmt + clippy before every commit
  ✅ Explore Agent: Open-ended codebase questions
```

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

### Re-encoding for Verification vs Transaction Construction

**Decision**: Re-encoding (for verification) is **IN SCOPE**; transaction construction is **OUT OF SCOPE**

**Date**: 2025-11-18 (Updated from 2025-11-13)

**Key Distinction**:
- ✅ **Re-encoding**: `decoded_tx.to_bytes()` - Reconstruct original transaction bytes from decoded structure
- ❌ **Construction**: `TransactionBuilder::new().add_output(...).build()` - Create new transactions

**Rationale for Re-encoding (IN SCOPE)**:
- **Mandatory for Formal Verification**: The injective property `encode(decode(x)) = x` is a fundamental requirement
- **Forensic Reconstruction**: Must be able to reconstruct exact original bytes for forensic analysis
- **No External Dependencies**: Re-encoding a decoded transaction requires no chain state, fee oracles, or UTXO sets
- **Simple & Auditable**: Re-encoding is deterministic byte serialization, not complex construction logic
- **TCB Impact**: Minimal (~200-300 LOC per decoder for serialization, well within budget)

**Rationale for Transaction Construction (OUT OF SCOPE)**:
- **Different Problem Domain**:
  - Re-encoding: Serialize existing decoded structure (deterministic, stateless)
  - Construction: Build valid transaction from user intent (stateful, requires external data)

- **Dependency Explosion**:
  - Re-encoding: Zero additional dependencies (just serialization)
  - Construction: Needs chain state providers, fee oracles, UTXO selectors, nonce managers

- **Complexity**:
  - Re-encoding: ~200-300 LOC per chain (simple byte serialization)
  - Construction: ~2500+ LOC per chain (validation, fee estimation, state management)

**Primary Use Cases Served**:
- ✅ Block explorers (decode + verify roundtrip)
- ✅ Forensics and auditing (decode + reconstruct exact bytes)
- ✅ Indexers and analytics (decode + verify)
- ✅ Formal verification (prove lossless codec)
- ✅ Chain monitoring (decode + re-encode for integrity checks)
- ❌ Wallets (need transaction construction with fee estimation)
- ❌ dApps (need transaction building with state management)

**Implementation Requirements**:
1. Every `ChainDecoder` implementation MUST provide re-encoding capability
2. Property test required: `encode(decode(tx_bytes)) = tx_bytes` for all chains
3. Re-encoding must be deterministic and produce exact original bytes
4. No external dependencies allowed (no chain state, network calls, etc.)

**Status**:
- ✅ Architecture updated to include re-encoding trait
- 🚧 Implementation in progress for all supported chains
- 📋 Property tests to be added for all decoders

---

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
8. ✅ **Re-encoding support** (must support `encode(decode(x)) = x` for verification)
9. ✅ **Property tests for injectivity** (roundtrip property test required for all decoders)

**See**: `CONTRIBUTING.md` for detailed guidelines

**IMPORTANT Distinctions**:
- ✅ **Re-encoding** (IN SCOPE): `decoded_tx.to_bytes()` - Reconstruct original bytes for verification
- ❌ **Transaction construction** (OUT OF SCOPE): Building new transactions with fee estimation, UTXO selection, etc.

**Note**: Contributions that add transaction **construction**, **signing**, or **broadcasting** will be rejected as out of scope. However, **re-encoding** (for roundtrip verification) is mandatory for all decoders.

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

## Frequently Asked Questions

### Q: Does this project support re-encoding transactions back to bytes?

**A**: **YES!** Re-encoding is **IN SCOPE** and **mandatory**. Every decoder MUST support:
```rust
let decoded_tx = BitcoinDecoder::decode(tx_bytes)?;
let re_encoded = decoded_tx.to_bytes()?;
assert_eq!(tx_bytes, re_encoded); // Must be identical
```

This is required for:
- ✅ Formal verification of the injective property: `encode(decode(x)) = x`
- ✅ Forensic reconstruction of exact original bytes
- ✅ Verifying lossless decoding
- ✅ Integrity checks and auditing

### Q: Why doesn't this project support transaction construction?

**A**: Transaction **construction** (building new transactions) is **OUT OF SCOPE**. There's a critical difference:
- ✅ **Re-encoding**: `decoded_tx.to_bytes()` - Serialize existing decoded transaction (simple, stateless)
- ❌ **Construction**: `TransactionBuilder::new().add_output(...)` - Build new transactions (complex, stateful)

Transaction construction requires:
- Chain state access (nonces, balances, UTXO sets)
- Fee estimation (mempool data, gas markets)
- Complex validation (sufficient funds, gas limits)
- Different security model (constructive vs defensive)

For transaction construction, use chain-specific wallet SDKs:
- Bitcoin: `bitcoin` crate, BDK (Bitcoin Dev Kit)
- Ethereum: `ethers-rs`, `alloy`
- Solana: `solana-sdk`

### Q: Can I use TxIR to build new transactions from scratch?

**A**: **No**. TxIR is designed for analysis of existing transactions. While you can re-encode a decoded transaction back to bytes, you cannot use TxIR to construct new transactions from scratch. That would require fee calculation, UTXO selection, nonce management, etc., which are out of scope. Use chain-specific wallet libraries for transaction creation.

### Q: What about roundtrip testing (encode(decode(x)) = x)?

**A**: **YES!** This is a **mandatory requirement** for all decoders:
1. **Chain-Specific Roundtrip**: `encode(decode(tx_bytes)) = tx_bytes` (REQUIRED property test for every decoder)
2. **Canonical Serialization**: `borsh_encode(borsh_decode(x)) = x` (for TxIR → Borsh bytes)
3. **Verification**: All decoders MUST pass property tests verifying lossless roundtrip

Every decoder implementation must include property tests like:
```rust
proptest! {
    #[test]
    fn roundtrip_property(tx_bytes: Vec<u8>) {
        if let Ok(decoded) = decode(&tx_bytes) {
            let re_encoded = decoded.to_bytes()?;
            prop_assert_eq!(tx_bytes, re_encoded);
        }
    }
}
```

---

## Changelog

### 2025-11-18 - v0.3.0 - **MAJOR UPDATE: Re-encoding is Mandatory**
- **BREAKING CHANGE**: Re-encoding (for verification) is now **IN SCOPE** and **MANDATORY**
  - Updated project scope from "Decoding Only" to "Decoding & Verification"
  - All decoders MUST support `encode(decode(tx_bytes)) = tx_bytes` (injective property)
  - Added re-encoding requirement to formal verification criteria
  - Clarified distinction: Re-encoding (✅ in scope) vs Transaction Construction (❌ out of scope)
- **Updated**: Decision log with "Re-encoding for Verification vs Transaction Construction"
  - Detailed rationale for why re-encoding is mandatory
  - TCB impact assessment: ~200-300 LOC per decoder (minimal)
  - No dependency explosion (re-encoding is stateless, unlike construction)
- **Updated**: FAQ section with new questions about re-encoding
  - Added mandatory roundtrip property test examples
  - Clarified use cases for re-encoding vs construction
- **Updated**: Contributing criteria
  - Added criterion #8: Re-encoding support mandatory
  - Added criterion #9: Property tests for injectivity required
- **Impact**: All existing decoders need to implement re-encoding capability
- **Timeline**: Implementation plan to be created in next phase

### 2025-11-18 - v0.2.1
- **Added**: Actor Model chain family documentation (Phase 3.11)
  - Added `ChainFamily::Actor` to roadmap for ICP and AO support
  - Documented Actor Model transaction semantics (async message passing)
  - Updated ROADMAP.md with Phase 3.11 implementation plan
  - Added to documentation map and next actions
  - Identified key differences from UTXO/Account/Instruction models

### 2025-11-13 - v0.1.2
- **Updated**: Rebased onto main with Verus formal verification infrastructure
  - Integrated Verus tooling (PR #36)
  - Updated status table: Formal verification marked as ✅ Done
  - Marked PR #10 (Install Verus) as completed
  - Privacy.rs now uses proper Verus annotations (commented for normal builds)

### 2025-11-13 - v0.1.1
- **Added**: Comprehensive "Claude CLI Tools & Workflows" section
  - Task tool usage for codebase exploration
  - TodoWrite tool for task management
  - File operations (Read, Edit, Write) best practices
  - Search tools (Grep, Glob) patterns
  - Bash tool guidelines
  - Parallel vs sequential execution strategies
  - Project-specific workflows
  - Common mistakes to avoid
  - Performance tips
  - Quick reference card

### 2025-11-13 - v0.1.0
- Initial CLAUDE.md with core design philosophy
- Design criteria (1-10)
- Phase 1.5 implementation plan
- Git workflow and quality checks

### 2025-11-13 - v0.2.0
- **Added**: "Project Scope: Decoding Only" section
  - Explicitly documents that encoding is out of scope
  - Lists in-scope and out-of-scope functionality
  - Provides rationale for decode-only architecture
- **Added**: "Why Decoding Only (No Encoding)?" decision log entry
  - Detailed analysis of decode vs encode problem domains
  - TCB impact assessment
  - Future path for separate encoder project
- **Added**: FAQ section
  - Answers common questions about encoding scope
  - Provides alternatives for transaction construction
  - Explains roundtrip testing strategy
- **Updated**: Contributing section
  - Added criterion #8: "Decoding only"
  - Clarifies that encoding PRs will be rejected

---

**Last Updated**: 2025-11-18
**Version**: 0.3.0
**Status**: Living Document

**Major Change in v0.3.0**: Re-encoding is now mandatory for all decoders to verify the injective property.
