# CLAUDE: Core Library Architecture & Unified Design Ethos

## Quick Reference

**Current Phase**: Phase 1.5 - Testing & Dependency Infrastructure

**Self-Improvement Loop** (start here when resuming work):
- `loop/report.json` - MEASURED project state (regenerate: `python3 scripts/loop/health_report.py`)
- `loop/BACKLOG.md` - prioritized work items with mechanical acceptance checks
- `docs/SELF_IMPROVEMENT_LOOP.md` - loop design + anti-Goodhart rules
- `docs/ASSUMPTIONS_REVIEW.md` - 2026-06 audit; several claims below are
  currently violated (core LOC budget, JSON-free canonical form, differential
  testing coverage, non-vacuous roundtrip). Trust `loop/report.json` over
  status claims in this file or ROADMAP.md.
- Run one iteration with the `improve-loop` skill (`/improve-loop`).

**Documentation**:
- `ROADMAP.md` - Project phases and implementation details
- `docs/TESTING_STRATEGY.md` - 5-level testing pyramid
- `docs/GIT_SUBTREE_VENDORING.md` - Verifiable dependency vendoring
- `docs/FORMAL_VERIFICATION.md` - Verus verification plan
- `docs/CANONICAL_SERIALIZATION.md` - Why Borsh, not JSON
- `docs/blockchain-addition/` - Add new blockchains in 5-30 min (LLM & human-friendly)

---

## Core Principle: Minimal Trusted Computing Base (TCB)

Create a **small, reviewable, formally verifiable core library** (< 3000 LOC) that serves as the trusted foundation for all blockchain transaction decoding.

---

## Project Scope: Decoding & Verification

**In Scope**:
- Decoding blockchain transactions (chain-specific bytes -> TxIR)
- Re-encoding for verification (`encode(decode(tx_bytes)) = tx_bytes` - MANDATORY)
- Canonical serialization (TxIR -> Borsh bytes)
- Transaction validation and signature verification

**Out of Scope**:
- Transaction construction, signing, broadcasting
- Fee estimation, UTXO selection, nonce management

**Critical Distinction**:
- **Re-encoding** = `decoded_tx.to_bytes()` - reconstruct original bytes for verification
- **Construction** = `TransactionBuilder::new().build()` - create new transactions (OUT OF SCOPE)

---

## Design Criteria

### 1. Minimal Core
- Core < 3000 LOC
- Core defines **traits**, not implementations
- Chain-specific logic in **separate crates**
- Trait-based extensibility (not enum-based)

### 2. Formally Verifiable
- No `unsafe` code in core
- Explicit preconditions/postconditions
- Provable panic-freedom
- **Critical Property**: `encode(decode(tx_bytes)) = tx_bytes` (injective/roundtrip)

### 3. Canonical Serialization (Non-Negotiable)
- **ALWAYS** use Borsh for canonical representation
- **NEVER** use JSON for hashing or signature verification
- JSON is **ONLY** for human display

### 4. Supply Chain Security
- Minimal dependencies: `serde`, `borsh`, `thiserror`, crypto primitives
- Vendor dependencies via git subtree for airgapped operation
- No runtime network calls in production code

### 5. Zero-Cost Abstractions
- Static dispatch (generics) over dynamic dispatch (trait objects)
- Const generics for compile-time constraints

---

## Git Workflow (MANDATORY)

Before every commit:
```bash
cargo fmt --all
cargo clippy --all --all-targets --all-features -- -D warnings
cargo test --all  # recommended
```

**Common Clippy Fixes**:
```rust
// Use is_empty() instead of len() > 0
if !vec.is_empty() { }

// Don't borrow when ownership works
Err(DecoderError::invalid_structure(format!("error: {}", x)))
```

---

## Property Tests Required

Every decoder MUST include:
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

## Testing Levels

1. **Unit Tests**: Every public function
2. **Property-Based Tests**: With proptest (50+ tests target)
3. **Formal Verification**: Verus annotations
4. **Integration Tests**: Real blockchain data fixtures

---

## Architecture

```
Chain Decoders (External, Untrusted)
  - decoder-bitcoin, decoder-ethereum, etc.
  - Anyone can implement, independently audited
         │
         │ ChainDecoder trait
         ▼
Core Library (Minimal, Trusted)
  - Trait definitions (< 3000 LOC)
  - TxIR type
  - Canonical serialization (Borsh)
  - Formally verified
```

---

## Contributing Criteria

1. No core changes for new chains (use traits)
2. Maintain formal verifiability (no unsafe)
3. Preserve minimal TCB (< 3000 LOC core)
4. Use canonical serialization (Borsh, not JSON)
5. Zero-cost abstractions (static dispatch)
6. Comprehensive tests (unit + property + integration)
7. Re-encoding support MANDATORY (`encode(decode(x)) = x`)
8. Property tests for injectivity required

---

## Decision Log

### Re-encoding vs Transaction Construction
- **Re-encoding (IN SCOPE)**: `decoded_tx.to_bytes()` - stateless, ~200-300 LOC per decoder
- **Construction (OUT OF SCOPE)**: Requires chain state, fee oracles, ~2500+ LOC per chain

### Why Borsh over Protobuf?
- Borsh: Designed for deterministic encoding, simpler, native Rust
- Protobuf: Better cross-language but we prioritize canonicity

### Why Traits over Enums?
- Enums: Require core changes for new chains
- Traits: Enable ecosystem growth, core stays minimal

---

## References

- `docs/FORMAL_VERIFICATION.md` - Verus verification plan
- `docs/CANONICAL_SERIALIZATION.md` - Borsh requirements
- `docs/TRAIT_BASED_ARCHITECTURE.md` - Extension pattern
- https://github.com/verus-lang/verus
- https://borsh.io/

---

**Last Updated**: 2025-12-27
**Version**: 0.4.1
