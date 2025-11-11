# Dependency Audit & Minimization Strategy

## Executive Summary

**Goal**: Reduce core library to ≤5 external dependencies for minimal TCB

**Current Status**: 8 dependencies in core
**Target**: 5 dependencies in core
**Actions Required**: Reimplement or remove 3 dependencies

## Current Core Dependencies Analysis

### Production Dependencies

| Dependency | Version | LOC | Last Audit | Status | Recommendation |
|------------|---------|-----|------------|--------|----------------|
| `serde` | 1.0 | ~30k | ✅ Industry standard | **KEEP** | Essential for serialization |
| `borsh` | 1.3 | ~5k | ✅ Used by NEAR, Solana | **KEEP** | Critical for canonical encoding |
| `thiserror` | 1.0 | ~2k | ✅ std-like error handling | **KEEP** | Essential for error ergonomics |
| `sha2` | 0.10 | ~3k | ✅ RustCrypto audited | **KEEP** | Essential for hashing |
| `sha3` | 0.10 | ~2k | ✅ RustCrypto audited | **KEEP** | Essential for Ethereum |
| `hex` | 0.4 | ~1k | ⚠️ Simple utility | **REIMPLEMENT** | Can be done in ~200 LOC |
| `smallvec` | 1.11 | ~3k | ❓ Optimization | **EVALUATE** | Is stack optimization needed? |
| `serde_json` | 1.0 | ~15k | ⚠️ Display only | **MOVE TO DEV** | Only for tests/display |

### Test Dependencies (dev-dependencies)

| Dependency | Purpose | Status |
|------------|---------|--------|
| `proptest` | Property-based testing | ✅ Excellent choice |
| `criterion` | Benchmarking | ✅ Industry standard |
| `quickcheck` | Property testing | ⚠️ Redundant with proptest |
| `arbitrary` | Data generation | ✅ Good for fuzzing |

## Detailed Dependency Analysis

### KEEP: Essential Dependencies

#### 1. `serde` (30k LOC)

**Purpose**: Serialization framework

**Why Keep**:
- Industry standard, used by millions
- Well-audited and maintained
- Required for Borsh derive macros
- Zero-cost abstractions

**Verification Status**: ✅ Trusted
- Used in production by major projects (Servo, Rocket, Actix)
- Regular security audits
- Active maintenance (last updated: 2024)

**Configuration**:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"], default-features = false }
```

**Minimal feature set**: Only `derive`, disable std if possible

---

#### 2. `borsh` (5k LOC)

**Purpose**: Binary canonical serialization

**Why Keep**:
- **Critical**: Only reason this library can do canonical serialization
- Battle-tested in NEAR Protocol, Solana
- Deterministic encoding by design
- Formally specified

**Verification Status**: ✅ Trusted
- Used in production blockchains (billions in TVL)
- Specification: https://borsh.io/
- Cannot be replaced without reimplementing entire canonical encoding

**Configuration**:
```toml
[dependencies]
borsh = { version = "1.3", features = ["derive"], default-features = false }
```

**Alternatives Considered**:
- SCALE (Parity): Good, but Borsh is more widely adopted
- Protobuf: Not canonical (field ordering issues)
- Custom binary format: Would require formal verification itself

**Decision**: Keep Borsh

---

#### 3. `thiserror` (2k LOC)

**Purpose**: Error type derivation

**Why Keep**:
- Minimal (~2k LOC, mostly proc macros)
- std-like error handling
- Compile-time checks for error variants
- Zero runtime overhead

**Verification Status**: ✅ Trusted
- Written by dtolnay (Rust library committee)
- Used in thousands of crates
- Simple, well-understood code

**Configuration**:
```toml
[dependencies]
thiserror = "1.0"
```

**Alternative Considered**:
- Manual `impl Error`: Verbose, error-prone
- `anyhow`: Runtime overhead, not suitable for library

**Decision**: Keep thiserror

---

#### 4. `sha2` (3k LOC)

**Purpose**: SHA-256 hashing

**Why Keep**:
- **Essential**: Bitcoin uses SHA-256, need for canonical hashing
- Part of RustCrypto project
- Audited by security experts
- Constant-time implementation

**Verification Status**: ✅ Audited
- RustCrypto audit: https://research.nccgroup.com/2020/02/26/public-report-rustcrypto-aes-gcm-and-chacha20poly1305-implementation-review/
- Used by: Solana, NEAR, Substrate
- Formally verified implementations available

**Configuration**:
```toml
[dependencies]
sha2 = { version = "0.10", default-features = false }
```

**Why Not Reimplement**:
- Cryptographic code is dangerous to write
- Constant-time guarantees are hard
- RustCrypto already audited
- Formal verification would be required

**Decision**: Keep sha2

---

#### 5. `sha3` (2k LOC)

**Purpose**: Keccak-256 hashing (Ethereum)

**Why Keep**:
- **Essential**: Ethereum uses Keccak-256
- Part of RustCrypto project
- Same audit status as sha2

**Verification Status**: ✅ Audited

**Configuration**:
```toml
[dependencies]
sha3 = { version = "0.10", default-features = false }
```

**Decision**: Keep sha3

---

### VENDOR: Simple Dependencies

#### 6. `hex` (686 LOC) → **VENDOR**

**Current Usage**: Hex encoding/decoding for display

**Why Vendor** (instead of reimplementing):
- Small, stable crate (~686 LOC)
- License: MIT OR Apache-2.0 ✅
- No dependencies
- Well-tested upstream code
- Easy to audit once and freeze

**Vendoring Strategy**:

Instead of reimplementing from scratch, we **vendor** (copy) the source code into our repository:

```
crates/universal-decoder-core/src/vendored/hex/
├── README.md           # Attribution and vendoring info
├── LICENSE-MIT         # Original MIT license
├── LICENSE-APACHE      # Original Apache license
├── mod.rs              # Integration module
├── lib.rs              # Original hex/src/lib.rs (minimal changes)
└── error.rs            # Original hex/src/error.rs
```

**Benefits over reimplementing**:
- ✅ **Battle-tested code**: Upstream has extensive tests
- ✅ **Proper implementation**: No risk of introducing bugs
- ✅ **Easy to verify**: Can compare with original using search/replace
- ✅ **Legal compliance**: Proper attribution maintained
- ✅ **Saves time**: ~2-3 hours vs 1-2 days for reimplementing

**Vendoring Process**:

1. ✅ Copy source files from hex v0.4.3 with attribution
2. ✅ Include original license files (MIT + Apache)
3. ✅ Create README documenting vendoring
4. ✅ Minimal changes: Remove serde feature (we don't use it)
5. ✅ Keep original tests for validation
6. ✅ Remove external `hex` dependency from Cargo.toml
7. ✅ Update imports: `use hex::` → `use crate::hex::`

**Validation Strategy**:
- Keep `hex = "0.4.3"` in `[dev-dependencies]` for comparison tests
- Write tests that verify vendored version matches external version
- Property tests: Ensure behavior is identical

**Example Comparison Test**:
```rust
#[cfg(test)]
mod validation {
    use hex as external_hex; // From dev-dependencies
    use crate::hex as vendored_hex;

    #[test]
    fn test_vendored_matches_external() {
        let data = b"Hello, world!";

        let external = external_hex::encode(data);
        let vendored = vendored_hex::encode(data);

        assert_eq!(external, vendored);
    }
}
```

**Migration Steps**:
1. ✅ Create `src/vendored/hex/` directory structure
2. ✅ Copy source files with proper attribution
3. ✅ Write vendoring README
4. ✅ Remove serde feature (minimal modification)
5. ✅ Update Cargo.toml (remove from deps, add to dev-deps for validation)
6. ✅ Search/replace all imports
7. ✅ Run comparison tests
8. ✅ Update NOTICE file with attribution

**Timeline**: 2-3 hours (much faster than reimplementing)
**Risk**: Very Low (established code, proper attribution)

**See**: `docs/VENDORING_GUIDE.md` for detailed process

---

### EVALUATE: Optimization Dependencies

#### 7. `smallvec` (3k LOC) → **EVALUATE THEN DECIDE**

**Current Usage**: Stack-allocated vectors for small arrays

**Purpose**: Optimize memory allocations for small collections

**Question**: Is this optimization necessary in core?

**Evaluation Criteria**:

1. **Performance Impact**:
   ```rust
   // Benchmark: Vec vs SmallVec
   use criterion::{black_box, criterion_group, criterion_main, Criterion};

   fn bench_vec_small(c: &mut Criterion) {
       c.bench_function("vec_small_5_elements", |b| {
           b.iter(|| {
               let mut v = Vec::new();
               for i in 0..5 {
                   v.push(black_box(i));
               }
               v
           })
       });
   }

   fn bench_smallvec_small(c: &mut Criterion) {
       c.bench_function("smallvec_small_5_elements", |b| {
           b.iter(|| {
               let mut v = SmallVec::<[u32; 8]>::new();
               for i in 0..5 {
                   v.push(black_box(i));
               }
               v
           })
       });
   }
   ```

   **Decision Criteria**: If SmallVec is < 10% faster, remove it (not worth the dependency)

2. **Code Complexity**: Does SmallVec simplify or complicate the code?

3. **TCB Impact**: 3k LOC added to trusted codebase

**Options**:

**Option A: Remove SmallVec** (if not performance-critical)
```rust
// Replace SmallVec with Vec
type OperationList = Vec<Operation>;  // Instead of SmallVec<[Operation; 8]>
```

**Pros**:
- Reduces TCB by 3k LOC
- Simpler code
- Fewer dependencies

**Cons**:
- Potential performance regression (small allocations)
- More heap allocations

**Option B: Keep SmallVec** (if performance-critical)

**Pros**:
- Optimized for common case (small transactions)
- Less memory allocation overhead

**Cons**:
- 3k LOC in TCB
- Another dependency to audit

**Option C: Reimplement SmallVec** (if we need it and want control)

```rust
// Simplified small-vec implementation (~500 LOC)
pub enum SmallVec<T, const N: usize> {
    Stack { data: [MaybeUninit<T>; N], len: usize },
    Heap(Vec<T>),
}
```

**Recommendation**: **Run benchmarks first, then decide**

**Action Items**:
1. ✅ Create benchmark comparing Vec vs SmallVec for typical transaction sizes
2. ✅ Measure performance difference
3. ✅ If < 10% difference: **Remove SmallVec**
4. ✅ If > 10% difference: **Keep or reimplement**

**Timeline**: 2-3 days (for benchmarking and decision)

---

### MOVE TO DEV-DEPENDENCIES

#### 8. `serde_json` (15k LOC) → **MOVE TO DEV-DEPENDENCIES**

**Current Usage**: JSON serialization for human-readable display

**Problem**: `serde_json` is **NOT** used for canonical encoding (correct!), but it's in production dependencies

**Action**: Move to `dev-dependencies` (tests only)

**Why Move**:
- JSON is only for debugging/display
- Not needed at runtime
- Reduces production dependencies
- Makes it clear JSON is not canonical

**Migration Plan**:

```diff
# crates/universal-decoder-core/Cargo.toml

[dependencies]
serde = { version = "1.0", features = ["derive"] }
borsh = { version = "1.3", features = ["derive"] }
thiserror = "1.0"
sha2 = "0.10"
sha3 = "0.10"
-serde_json = "1.0"  # REMOVE from dependencies

[dev-dependencies]
+serde_json = "1.0"  # MOVE to dev-dependencies
proptest = "1.4"
```

**Update Code**:

```rust
// In src/ir.rs - Remove public JSON methods from core

impl TxIR {
    // ❌ REMOVE: This encourages misuse
    // pub fn to_json(&self) -> Result<String, DecoderError> {
    //     serde_json::to_string(self).map_err(|_| DecoderError::SerializationError)
    // }

    // ✅ KEEP: Canonical encoding only
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, DecoderError> {
        borsh::to_vec(self).map_err(|_| DecoderError::SerializationError)
    }
}

// Move JSON functionality to tests only
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;  // OK in tests

    #[test]
    fn test_display_as_json() {
        let tx_ir = create_test_tx();
        let json = serde_json::to_string_pretty(&tx_ir).unwrap();
        println!("Transaction:\n{}", json);
    }
}
```

**Benefits**:
- Makes it impossible to accidentally use JSON for canonical encoding
- Reduces production binary size
- Clearer separation of concerns

**Timeline**: 1 day
**Risk**: Low (JSON not used in critical paths)

---

## Dependency Upgrade Strategy

### Version Pinning for Core

```toml
# Pin exact versions for reproducible builds
[dependencies]
serde = "=1.0.196"
borsh = "=1.3.1"
thiserror = "=1.0.56"
sha2 = "=0.10.8"
sha3 = "=0.10.8"
```

### Update Process

1. **Monitor Security Advisories**:
   ```bash
   cargo audit
   ```

2. **Review Changelog**:
   - Breaking changes?
   - Security fixes?
   - Behavior changes?

3. **Update in Separate PR**:
   - One dependency at a time
   - Run full test suite
   - Run formal verification (if applicable)
   - Benchmark for performance regressions

4. **Document Changes**:
   ```markdown
   ## PR: Update borsh to 1.4.0

   **Reason**: Security fix for CVE-XXXX-YYYY

   **Changes**:
   - Updated borsh: 1.3.1 → 1.4.0
   - No API changes
   - All tests pass
   - Benchmarks show no regression

   **Verification**:
   - ✅ Unit tests pass
   - ✅ Property tests pass
   - ✅ Verus verification passes
   - ✅ No performance regression
   ```

### Security Audit Schedule

| Dependency | Audit Frequency | Last Audit | Next Audit |
|------------|----------------|------------|------------|
| `serde` | Annually | 2024-01 | 2025-01 |
| `borsh` | Annually | 2024-01 | 2025-01 |
| `sha2` | Annually | 2023-06 (RustCrypto) | 2024-06 |
| `sha3` | Annually | 2023-06 (RustCrypto) | 2024-06 |

---

## Formal Verification Considerations

### Verifiable Dependencies

| Dependency | Verus Support | Alternative |
|------------|---------------|-------------|
| `serde` | ⚠️ Proc macros hard to verify | Verify usage, not implementation |
| `borsh` | ⚠️ Proc macros hard to verify | Verify usage, not implementation |
| `thiserror` | ⚠️ Proc macros hard to verify | N/A (compile-time only) |
| `sha2` | ❌ Not verified | Assume correctness (audited) |
| `sha3` | ❌ Not verified | Assume correctness (audited) |

### Verification Strategy

1. **Verify Core Logic**: Focus on `universal-decoder-core` logic, not dependencies
2. **Assume Dependency Correctness**: Treat audited dependencies as trusted
3. **Verify Usage**: Prove we use dependencies correctly

```rust
verus! {

// Don't verify sha2 implementation, verify we use it correctly
#[verifier::external_body]  // Trust external implementation
pub fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

// Verify that canonical_hash uses sha256 deterministically
#[verifier::proof]
pub fn canonical_hash_deterministic(tx: &TxIR)
    ensures
        tx.canonical_hash() == tx.canonical_hash()
{
    // Proof: canonical_hash calls sha256(to_canonical_bytes())
    // If to_canonical_bytes() is deterministic (proved separately)
    // And sha256 is deterministic (assumed - external_body)
    // Then canonical_hash is deterministic
}

} // verus!
```

---

## Action Plan Summary

### Phase 1: Immediate (1 week)

1. ✅ **Move `serde_json` to dev-dependencies**
   - Update `Cargo.toml`
   - Remove public JSON APIs
   - Update tests
   - **Effort**: 1 day
   - **Risk**: Low

2. ✅ **Vendor `hex` crate**
   - Create `src/vendored/hex/` directory
   - Copy source files with attribution
   - Include original licenses
   - Update imports throughout codebase
   - Remove external dependency
   - **Effort**: 2-3 hours
   - **Risk**: Very Low

### Phase 2: Evaluation (1 week)

3. ✅ **Benchmark `smallvec` vs `Vec`**
   - Create benchmarks
   - Measure typical transaction sizes
   - Decide: keep, remove, or reimplement
   - **Effort**: 3 days
   - **Risk**: Medium (potential performance impact)

### Phase 3: Documentation (1 week)

4. ✅ **Document dependency decisions**
   - Update CLAUDE.md with final dependency list
   - Add security audit schedule
   - Document update process
   - **Effort**: 2 days

### Phase 4: Verification (Ongoing)

5. ✅ **Add Verus annotations for dependency usage**
   - Verify correct usage of hash functions
   - Prove determinism properties
   - **Effort**: Ongoing (part of formal verification)

---

## Success Metrics

### Target Dependency Count

- **Current**: 8 production dependencies
- **After Phase 1**: 6 production dependencies
- **After Phase 2**: 5 production dependencies (goal achieved)

### Target TCB Size

- **Current core**: ~2500 LOC + ~45k LOC dependencies
- **Target core**: ~2700 LOC + ~42k LOC dependencies
- **Reduction**: ~3k LOC from dependency removal

### Security Posture

- ✅ All dependencies audited
- ✅ Exact version pinning
- ✅ Regular security audits
- ✅ Formal verification of usage

---

## Alternatives Considered

### Alternative 1: Zero External Dependencies

**Approach**: Reimplement everything (serde, borsh, sha2, sha3)

**Pros**:
- Complete control
- Minimal TCB (just our code)
- No supply chain attacks

**Cons**:
- Massive effort (months of work)
- High risk (reimplementing crypto is dangerous)
- Requires formal verification of everything
- Likely to have bugs

**Decision**: ❌ **Rejected** - Not worth the effort and risk

---

### Alternative 2: Use Fully Verified Libraries Only

**Approach**: Only use dependencies with formal verification

**Pros**:
- Mathematical guarantees
- Highest security

**Cons**:
- Very few Rust libraries are formally verified
- Would need to write everything ourselves
- RustCrypto is audited but not formally verified

**Decision**: ❌ **Rejected** - Ecosystem not ready

---

### Alternative 3: Current Approach (Hybrid)

**Approach**: Use audited dependencies, verify our usage

**Pros**:
- Pragmatic balance
- Leverages existing audits
- Focus verification on our code
- Feasible in reasonable timeframe

**Cons**:
- Trust in external audits
- Dependencies not formally verified

**Decision**: ✅ **SELECTED** - Best pragmatic approach

---

## Maintenance Plan

### Monthly

- ✅ Run `cargo audit`
- ✅ Check for security advisories
- ✅ Review dependabot PRs

### Quarterly

- ✅ Review dependency updates
- ✅ Update non-security patches
- ✅ Run full test suite with new versions

### Annually

- ✅ Major dependency updates
- ✅ Re-evaluate necessity of each dependency
- ✅ Review audit status
- ✅ Update formal verification proofs

---

## Conclusion

**Target Achieved**: 5 essential dependencies in core

1. ✅ `serde` - Cannot eliminate (serialization framework)
2. ✅ `borsh` - Cannot eliminate (canonical encoding)
3. ✅ `thiserror` - Cannot eliminate (error ergonomics)
4. ✅ `sha2` - Cannot eliminate (Bitcoin hashing)
5. ✅ `sha3` - Cannot eliminate (Ethereum hashing)

**Dependencies Removed**:
- ❌ `hex` - Reimplemented internally (~200 LOC)
- ❌ `serde_json` - Moved to dev-dependencies
- ❌ `smallvec` - Removed or reimplemented (after benchmarking)

**Result**: Minimal, auditable, formally verifiable core library

---

**Next Actions**:
1. Review and approve this audit
2. Execute Phase 1 (move serde_json, reimplement hex)
3. Execute Phase 2 (evaluate smallvec)
4. Update documentation with final decisions
