# CLAUDE: Core Library Architecture & Unified Design Ethos

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

**Forbidden in Core**:
- Complex parsing libraries (use in decoders)
- Network libraries
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

| Criterion | Current | Goal | Status |
|-----------|---------|------|--------|
| Core LOC | ~2500 | < 3000 | ✅ Good |
| Enum-based chains | Yes | No (traits) | ⚠️ Needs refactor |
| Formal verification | No | Verus annotations | 📋 Planned |
| Canonical serialization | Borsh ✅ | Borsh ✅ | ✅ Done |
| Core dependencies | 7 | < 10 | ✅ Good |
| Test coverage | Basic | Comprehensive | 📋 In progress |
| Documentation | Partial | Complete | 📋 In progress |

## Roadmap

### v0.1.0 (Current)
- ✅ Basic trait hierarchy
- ✅ Bitcoin & Ethereum decoders
- ✅ Canonical serialization (Borsh)
- ⚠️ Enum-based chains (temporary)

### v0.2.0 (Next - 2 months)
- 🎯 **Trait-based chain identity**
- 🎯 **Refactor to open architecture**
- 🎯 Add Verus annotations
- 🎯 Property-based tests

### v0.3.0 (3-4 months)
- 🎯 Formal verification of core
- 🎯 Security audit
- 🎯 Production-ready

### v1.0.0 (6 months)
- 🎯 Fully verified core
- 🎯 Comprehensive test suite
- 🎯 Stable API
- 🎯 Ecosystem of decoders

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
