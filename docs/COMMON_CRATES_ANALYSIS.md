# Common Crates and Functionalities Analysis

**Date**: 2025-11-12
**Status**: Phase 1.5 - Post-Implementation Review
**Chains Analyzed**: Bitcoin, Ethereum, Solana (+ 18 scaffolded chains)

## Executive Summary

After implementing pure Rust decoders for Bitcoin, Ethereum, and Solana, clear patterns have emerged that suggest opportunities for extracting common functionalities into shared crates. This analysis identifies what already exists, what's duplicated, and what should be extracted.

## 1. Current State: decoder-primitives ✅

**Location**: `crates/decoder-primitives/`
**Status**: Implemented and working well
**LOC**: ~500
**Dependencies**: Zero (only universal-decoder-core)

### What It Provides

#### 1.1 Byte Operations (`bytes.rs`)
- `read_bytes_bounded()` - Safe bounded byte reading
- `read_bytes()` - Read with default 10MB limit
- `read_array::<N>()` - Read fixed-size arrays (const generics)
- `read_remaining()` - Read all remaining bytes

**Used by**: All three decoders (Bitcoin, Ethereum, Solana)

#### 1.2 Little-Endian Readers (`readers/little_endian.rs`)
- `read_u8()`, `read_u16_le()`, `read_u32_le()`, `read_u64_le()`, `read_u128_le()`
- `read_i32_le()` - Signed integers

**Used by**: Bitcoin (UTXO chains), Solana

#### 1.3 Big-Endian Readers (`readers/big_endian.rs`)
- `read_u16_be()`, `read_u32_be()`, `read_u64_be()`, `read_u128_be()`
- `read_u256_be()` - 32-byte values (Ethereum)
- `read_address()` - 20-byte Ethereum addresses

**Used by**: Ethereum (EVM chains), Cosmos, Polkadot

### Why It Works Well

1. **Zero dependencies** - No external crates, only core
2. **Consistent API** - All functions return `Result<T>`
3. **Security-first** - Bounds checking on all operations
4. **Well-tested** - Comprehensive unit tests
5. **Inline-able** - Performance through `#[inline]`

---

## 2. Variable-Length Encoding (NEW SHARED CRATE) 🆕

### Current State: Duplicated Across Decoders

| Decoder | Encoding | Location | LOC |
|---------|----------|----------|-----|
| Bitcoin | VarInt | `decoder-bitcoin/src/varint.rs` | ~70 |
| Solana | Compact-u16 | `decoder-solana/src/parsing.rs` | ~100 |
| Ethereum | RLP | `decoder-ethereum/src/rlp.rs` | ~340 |

### Recommendation: Create `decoder-encodings` Crate

**Rationale**:
- Variable-length encoding is common across many chains
- RLP is used by all EVM chains (Ethereum, Polygon, Arbitrum, Optimism, Avalanche C-Chain, BNB)
- VarInt is used by Bitcoin-derived chains (Dogecoin, Litecoin)
- Compact-u16 is Solana-specific but shows the pattern

#### Proposed Structure

```
crates/decoder-encodings/
├── Cargo.toml              # Zero external dependencies
└── src/
    ├── lib.rs              # Re-exports
    ├── varint.rs           # Bitcoin VarInt (move from decoder-bitcoin)
    ├── compact_u16.rs      # Solana compact-u16 (move from decoder-solana)
    ├── rlp/
    │   ├── mod.rs          # Re-exports
    │   ├── decoder.rs      # RLP decoding (move from decoder-ethereum)
    │   ├── encoder.rs      # RLP encoding (for future)
    │   └── types.rs        # RlpItem enum
    └── leb128.rs           # LEB128 (for NEAR, Polkadot, future chains)
```

#### API Design

```rust
// decoder-encodings/src/varint.rs
pub fn read_varint<R: Read>(reader: &mut R) -> Result<u64> { /* ... */ }
pub fn write_varint(buf: &mut Vec<u8>, value: u64) { /* ... */ }

// decoder-encodings/src/compact_u16.rs
pub fn read_compact_u16<R: Read>(reader: &mut R) -> Result<u16> { /* ... */ }
pub fn write_compact_u16(buf: &mut Vec<u8>, value: u16) { /* ... */ }

// decoder-encodings/src/rlp/decoder.rs
pub enum RlpItem {
    Data(Vec<u8>),
    List(Vec<RlpItem>),
}
impl RlpItem {
    pub fn decode(bytes: &[u8]) -> Result<Self> { /* ... */ }
    pub fn as_u64(&self) -> Result<u64> { /* ... */ }
    pub fn as_u128(&self) -> Result<u128> { /* ... */ }
}
```

#### Benefits

1. **Reusability**:
   - RLP shared by 7+ EVM chains
   - VarInt shared by 3+ Bitcoin forks
2. **Single source of truth**: Fix bugs once, benefit all chains
3. **Better testing**: Focus testing effort on shared encoding logic
4. **Maintains TCB**: Still zero external dependencies

#### Migration Strategy

```bash
# Phase 2.2 - Week 1
1. Create decoder-encodings crate skeleton
2. Move Bitcoin VarInt with tests → decoder-encodings/src/varint.rs
3. Update decoder-bitcoin to use decoder-encodings
4. Move Solana compact-u16 → decoder-encodings/src/compact_u16.rs
5. Move Ethereum RLP → decoder-encodings/src/rlp/
6. Update decoder-ethereum to use decoder-encodings
7. Update EVM-family decoders (Polygon, Arbitrum, etc.)

# Validation
cargo test --package decoder-encodings
cargo test --all
```

---

## 3. Address/Hash Formatting (NEW SHARED CRATE) 🆕

### Current State: Ad-hoc String Formatting

| Chain | Address Format | Implementation |
|-------|---------------|----------------|
| Bitcoin | Base58Check | Manual in decoder (if needed) |
| Ethereum | Hex (0x...) | `universal_decoder_core::hex::encode()` |
| Solana | Base58 | base64 crate in dev-deps for testing |

### Observation: Missing Common Encodings

Many chains use:
- **Base58** (Bitcoin, Solana, Stellar, Cardano)
- **Base32** (Stellar, Algorand)
- **Bech32** (Bitcoin SegWit, Cosmos, many modern chains)
- **SS58** (Polkadot, Kusama, Substrate chains)

### Recommendation: Create `decoder-encodings` Submodule

**Rationale**:
- Address formatting is display logic, not parsing logic
- But commonly needed across decoders for human-readable output
-Vendoring base58/bech32 implementations aligns with airgapped strategy

#### Proposed Structure (Addition to decoder-encodings)

```
crates/decoder-encodings/src/
├── address/
│   ├── mod.rs
│   ├── base58.rs           # Bitcoin, Solana
│   ├── bech32.rs           # Bitcoin SegWit, Cosmos
│   ├── ss58.rs             # Polkadot/Substrate
│   └── checksummed_hex.rs  # Ethereum EIP-55
```

#### API Design

```rust
// decoder-encodings/src/address/base58.rs
pub fn encode_base58(bytes: &[u8]) -> String { /* ... */ }
pub fn decode_base58(s: &str) -> Result<Vec<u8>> { /* ... */ }
pub fn encode_base58_check(version: u8, bytes: &[u8]) -> String { /* ... */ }

// decoder-encodings/src/address/checksummed_hex.rs
pub fn to_checksummed_address(address: &[u8; 20]) -> String {
    // EIP-55: Mixed-case checksum encoding
}
```

#### Implementation Strategy

**Option A: Vendor existing crates** (Recommended)
```bash
# Use git subtree to vendor battle-tested implementations
git subtree add --prefix crates/decoder-encodings/vendored/bs58 \
    https://github.com/mycorrhiza/bs58-rs.git v0.5.0 --squash

git subtree add --prefix crates/decoder-encodings/vendored/bech32 \
    https://github.com/rust-bitcoin/rust-bech32.git v0.11.0 --squash
```

**Option B: Pure Rust minimal implementations**
- Implement only what's needed
- Smaller, easier to audit
- But requires more verification effort

**Recommendation**: Use **Option A** with vendoring
- Leverage existing audited code
- Aligns with hex vendoring precedent
- Verifiable via git subtree (supply chain security)

---

## 4. Cryptographic Hash Wrappers (OPTIONAL) 🤔

### Current State: Direct Dependencies

| Decoder | Hash Functions | Crate |
|---------|---------------|-------|
| Bitcoin | SHA-256, RIPEMD-160, SHA-256d | `sha2` (0.10) |
| Ethereum | Keccak-256 | `sha3` (0.10) |
| Solana | (none - uses raw Ed25519 signatures) | - |

### Observation: Minimal Duplication

- Each chain uses different hash functions
- Direct dependencies are fine (audited crates: `sha2`, `sha3`)
- No need to wrap unless we want to:
  - Add security checks (e.g., double-SHA256 is always 32 bytes)
  - Provide chain-specific hash helpers

### Recommendation: Keep as-is (No New Crate)

**Rationale**:
- `sha2` and `sha3` are minimal, well-audited dependencies
- No code duplication (each chain uses different hashes)
- Adding a wrapper crate would increase complexity without clear benefit

**However**, consider adding **chain-specific hash helpers**:

```rust
// decoder-bitcoin/src/hashing.rs
/// Bitcoin's double SHA-256 (hash256)
pub fn hash256(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let first = Sha256::digest(data);
    Sha256::digest(first).into()
}

/// Bitcoin's hash160 (SHA-256 followed by RIPEMD-160)
pub fn hash160(data: &[u8]) -> [u8; 20] {
    use sha2::{Sha256, Digest};
    use ripemd::{Ripemd160};
    let sha = Sha256::digest(data);
    Ripemd160::digest(sha).into()
}

// decoder-ethereum/src/hashing.rs
/// Ethereum's Keccak-256
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Keccak256, Digest};
    Keccak256::digest(data).into()
}
```

**Keep these chain-specific** - they're domain logic, not shared primitives.

---

## 5. Test Utilities (NEW SHARED CRATE) 🆕

### Current State: Duplicated Test Patterns

All three decoders follow similar testing patterns:

#### 5.1 Common Test Structure

```rust
// Pattern in all decoders
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_valid_transaction() { /* ... */ }

    #[test]
    fn test_decode_empty_fails() { /* ... */ }

    #[test]
    fn test_decode_truncated_fails() { /* ... */ }
}

// Property-based tests (using proptest)
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_decode_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
            // Fuzz test: decoder should never panic
            let _ = MyDecoder::decode(&bytes);
        }
    }
}
```

#### 5.2 Test Fixtures

```
crates/decoder-bitcoin/tests/fixtures/
crates/decoder-ethereum/tests/fixtures/
crates/decoder-solana/tests/fixtures/
```

### Recommendation: Create `decoder-test-utils` Crate

**Rationale**:
- Reduce test boilerplate
- Provide common test fixtures
- Standard property-based test strategies
- Only in dev-dependencies (doesn't affect production TCB)

#### Proposed Structure

```
crates/decoder-test-utils/
├── Cargo.toml              # dev-dependencies only (proptest, etc.)
└── src/
    ├── lib.rs
    ├── assertions.rs       # Common test assertions
    ├── fixtures.rs         # Fixture loading utilities
    ├── properties.rs       # Standard property tests
    └── fuzzing.rs          # Fuzzing helpers
```

#### API Design

```rust
// decoder-test-utils/src/assertions.rs
pub fn assert_decode_never_panics<D: ChainDecoder>(bytes: &[u8]) {
    let result = std::panic::catch_unwind(|| {
        D::decode(bytes)
    });
    assert!(result.is_ok(), "Decoder panicked on input");
}

pub fn assert_canonical_roundtrip<T: Canonicalizer>(tx: &T) {
    let ir1 = tx.canonicalize().unwrap();
    let bytes1 = ir1.to_canonical_bytes().unwrap();
    let bytes2 = ir1.to_canonical_bytes().unwrap();
    assert_eq!(bytes1, bytes2, "Canonical encoding not deterministic");
}

// decoder-test-utils/src/fixtures.rs
pub struct TestFixture {
    pub raw_bytes: Vec<u8>,
    pub expected_hash: Option<String>,
    pub description: String,
}

pub fn load_fixture(path: &str) -> TestFixture { /* ... */ }

// decoder-test-utils/src/properties.rs
pub fn standard_decoder_properties<D: ChainDecoder>() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..MAX_TX_SIZE)
}
```

#### Usage Example

```rust
// In decoder-bitcoin/tests/integration_tests.rs
use decoder_test_utils::prelude::*;

#[test]
fn test_bitcoin_decode_properties() {
    proptest!(|(bytes in standard_decoder_properties::<BitcoinDecoder>())| {
        assert_decode_never_panics::<BitcoinDecoder>(&bytes);
    });
}

#[test]
fn test_fixture_mainnet_tx() {
    let fixture = load_fixture("fixtures/bitcoin/mainnet_tx_1.bin");
    let tx = BitcoinDecoder::decode(&fixture.raw_bytes).unwrap();
    assert_canonical_roundtrip(&tx);
}
```

---

## 6. Summary: Recommended New Crates

### High Priority (Phase 2.2 - Next 2 Weeks)

#### 1. `decoder-encodings` 🚀
- **Purpose**: Variable-length encodings and address formatting
- **Contents**: VarInt, compact-u16, RLP, Base58, Bech32
- **Benefit**: Shared by 10+ chains, eliminates duplication
- **LOC**: ~800 (moved from existing code)
- **Dependencies**: Zero (vendored if needed)
- **Effort**: 3-5 days

**Migration Path**:
```bash
Week 1:
- Day 1-2: Create crate, move VarInt
- Day 3: Move compact-u16
- Day 4-5: Move RLP, update all EVM decoders

Week 2:
- Day 1-2: Vendor base58 implementation
- Day 3: Add address formatting APIs
- Day 4-5: Documentation and tests
```

#### 2. `decoder-test-utils` 🧪
- **Purpose**: Shared testing utilities
- **Contents**: Assertions, fixtures, property tests
- **Benefit**: Reduce test boilerplate, standardize testing
- **LOC**: ~300
- **Dependencies**: proptest (dev-only)
- **Effort**: 2-3 days

**Implementation**:
```bash
Week 2:
- Day 1: Create crate skeleton
- Day 2: Extract common assertions
- Day 3: Add property test helpers
```

### Medium Priority (Phase 3 - Weeks 3-6)

#### 3. Chain Family Utilities (Optional)

Some chains share family-specific patterns:

**UTXO Family** (Bitcoin, Dogecoin, Litecoin):
- Common UTXO types
- Script parsing utilities
- Witness data handling

**EVM Family** (Ethereum, Polygon, Arbitrum, Optimism, etc.):
- RLP encoding/decoding (covered by decoder-encodings)
- Transaction types (Legacy, EIP-2930, EIP-1559)
- Log/Event parsing

**Account Family** (Solana, Aptos, Sui):
- Instruction-based transaction models
- Account addressing

**Recommendation**: Wait until we implement 5+ chains in each family, then extract common patterns.

---

## 7. Dependency Strategy: Vendoring vs Direct

Following CLAUDE.md principles, we have two strategies:

### Core Library (universal-decoder-core)
**Strategy**: Minimal direct dependencies
- `serde` ✅
- `borsh` ✅
- `thiserror` ✅
- `sha2`, `sha3` ✅ (cryptographic primitives)

### Decoder Libraries (decoder-*)
**Strategy**: Zero production dependencies except:
- `universal-decoder-core` ✅
- `decoder-primitives` ✅
- `decoder-encodings` 🆕 (proposed)

### Vendoring Candidates

| Crate | Vendor? | Rationale |
|-------|---------|-----------|
| `hex` | ✅ Done | Small, supply-chain security |
| `base58` (`bs58`) | 🆕 Recommend | Used by 5+ chains, airgapped requirement |
| `bech32` | 🆕 Recommend | Used by 3+ chains, airgapped requirement |
| `proptest` | ❌ No | Dev-dependency only, large, actively maintained |
| `serde` | ❌ No | Core ecosystem crate, ubiquitous, well-audited |
| `sha2`, `sha3` | ❌ No | Cryptographic primitives, must use audited versions |

---

## 8. Implementation Roadmap

### Phase 2.2: Extract Common Crates (Weeks 1-2)

**Week 1: decoder-encodings**
```bash
# PR #1: Create decoder-encodings crate + move VarInt
git checkout -b phase2.2/decoder-encodings-varint

# PR #2: Move Solana compact-u16
git checkout -b phase2.2/decoder-encodings-compact-u16 phase2.2/decoder-encodings-varint

# PR #3: Move Ethereum RLP
git checkout -b phase2.2/decoder-encodings-rlp phase2.2/decoder-encodings-compact-u16
```

**Week 2: Address encoding + test utils**
```bash
# PR #4: Vendor base58, add address formatting
git checkout -b phase2.2/decoder-encodings-base58 phase2.2/decoder-encodings-rlp

# PR #5: Create decoder-test-utils
git checkout -b phase2.2/decoder-test-utils main
```

### Success Criteria

- ✅ `decoder-encodings` crate created with zero external dependencies
- ✅ VarInt, compact-u16, RLP moved from individual decoders
- ✅ All EVM decoders use shared RLP implementation
- ✅ Base58 vendored using git subtree
- ✅ `decoder-test-utils` provides common test assertions
- ✅ All existing tests still pass
- ✅ No increase in core TCB

---

## 9. Metrics: Before and After

### Current State (Post Phase 2.1)

| Metric | Bitcoin | Ethereum | Solana | Total |
|--------|---------|----------|--------|-------|
| Decoder LOC | ~1200 | ~800 | ~1200 | ~3200 |
| Encoding LOC | 70 (VarInt) | 340 (RLP) | 100 (compact) | 510 |
| Unique deps | 0 | 0 | 0 | 0 ✅ |
| Shared code | decoder-primitives (~500 LOC) | - | - | - |

### Projected State (Post Phase 2.2)

| Metric | Bitcoin | Ethereum | Solana | Total | Shared |
|--------|---------|----------|--------|-------|--------|
| Decoder LOC | ~1150 | ~500 | ~1120 | ~2770 | -430 |
| Encoding LOC | 0 | 0 | 0 | 0 | +510 (extracted) |
| Unique deps | 0 | 0 | 0 | 0 ✅ | - |
| Shared crates | primitives, encodings, test-utils | - | - | - | 3 crates |

**Benefits**:
- 430 LOC reduction across decoders (13% reduction)
- 510 LOC of shared, well-tested encoding logic
- Future EVM chains get RLP "for free"
- Standardized testing reduces future test writing by ~30%

---

## 10. Conclusion

### Immediate Actions (Next 2 Weeks)

1. **Create `decoder-encodings`** - Highest impact, used by 10+ chains
2. **Create `decoder-test-utils`** - Reduce test boilerplate, improve quality
3. **Vendor base58** - Enable address formatting for Bitcoin, Solana, etc.

### Future Considerations (Phase 3+)

1. **Chain family utilities** - Wait for 5+ implementations per family
2. **Formal verification** - Shared encodings are prime candidates for Verus proofs
3. **Performance optimization** - Benchmark shared primitives, ensure zero-cost abstractions

### Alignment with Design Goals

✅ **Minimal TCB**: Shared crates have zero dependencies
✅ **Reviewable**: Each crate < 1000 LOC
✅ **Reusable**: DRY principle applied correctly
✅ **Airgapped**: Vendoring strategy for supply chain security
✅ **Trait-based**: No changes to core, only decoder implementations

---

## Appendix A: Full Dependency Tree (Proposed)

```
universal-decoder-core
  ├── serde
  ├── borsh
  ├── thiserror
  ├── sha2
  └── sha3

decoder-primitives
  └── universal-decoder-core

decoder-encodings (NEW)
  ├── universal-decoder-core
  └── vendored/
      ├── bs58/      (via git subtree)
      └── bech32/    (via git subtree)

decoder-test-utils (NEW, dev-only)
  ├── universal-decoder-core
  └── proptest

decoder-{bitcoin,ethereum,solana,...}
  ├── universal-decoder-core
  ├── decoder-primitives
  ├── decoder-encodings (NEW)
  └── [dev-dependencies]
      ├── decoder-test-utils (NEW)
      ├── proptest
      └── chain-specific validation libs
```

**Total Production Dependencies**: 5 (core) + 0 (primitives) + 0 (encodings) = **5 total** ✅

---

**Next Step**: Review this analysis, then proceed with Phase 2.2 implementation.

**Questions for Discussion**:
1. Should we extract chain family utilities now, or wait for more implementations?
2. Should address formatting be part of decoders or a separate display crate?
3. Do we need a `decoder-crypto` wrapper, or keep direct dependencies on `sha2`/`sha3`?
