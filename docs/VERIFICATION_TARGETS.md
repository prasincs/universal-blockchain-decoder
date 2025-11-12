# Verification Targets for Universal Blockchain Decoder

**Last Updated**: 2025-11-12
**Coverage**: 20% (3 targets annotated, awaiting Verus verification)

This document tracks formal verification progress using Verus. Each verification target (VT) represents a critical safety property that must be proven.

**Phase 4 Status**: 🚧 In Progress - VT-1 implementation complete with annotations and tests

---

## Coverage Summary

| Phase | Total VTs | Verified | Annotations Ready | Todo | Coverage |
|-------|-----------|----------|-------------------|------|----------|
| Core  | 5         | 0        | 3 (VT-1.1-1.3)    | 2    | 60%      |
| Bitcoin | 5       | 0        | 0                 | 5    | 0%       |
| Ethereum | 5      | 0        | 0                 | 5    | 0%       |
| **Total** | **15** | **0**   | **3**            | **12** | **20%** |

---

## Phase 1: Core Library (universal-decoder-core)

### VT-1: Amount Arithmetic Safety ⚡ PRIORITY: CRITICAL

**Status**: ✅ Annotations Complete (Awaiting Verus Verification)
**Properties**:
- ✅ VT-1.1: `checked_add` overflow detection (annotated, 8 tests)
- ✅ VT-1.2: `checked_sub` underflow detection (annotated, 7 tests)
- ✅ VT-1.3: `checked_mul` overflow detection (annotated, 7 tests)
- ⏳ VT-1.4: Decimal conversion correctness (implementation pending)

**Files**:
- `crates/universal-decoder-core/src/ir.rs:361-572` (implementation)
- `crates/universal-decoder-core/src/ir.rs:674-910` (tests)

**Verification Conditions**: 0 / ~15 proven (annotations ready)
**Test Coverage**: 22 comprehensive unit tests passing
**Estimated Effort**: 1-2 weeks (implementation complete)
**Next Step**: Install Verus and run verification
**Last Updated**: 2025-11-12

**Why Critical**:
- Amount calculations are security-critical (fee calculation, balance transfers)
- Overflow bugs can lead to financial loss
- Easy to verify (good starting point)

**Example Proof**:
```rust
verus! {

impl Amount {
    pub fn checked_add(self, other: Amount) -> (result: Option<Amount>)
        requires self.decimals == other.decimals,
        ensures
            result.is_some() ==> {
                let sum = result.unwrap();
                sum.value == self.value + other.value &&
                sum.decimals == self.decimals
            },
            result.is_none() ==> self.value + other.value > u128::MAX,
    {
        match self.value.checked_add(other.value) {
            Some(sum) => Some(Amount { value: sum, decimals: self.decimals }),
            None => None,
        }
    }
}

} // verus!
```

---

### VT-2: Canonicalization Determinism ⚡ PRIORITY: CRITICAL

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-2.1: `to_canonical_bytes()` is deterministic (same input → same output)
- ❌ VT-2.2: Borsh encoding never panics
- ❌ VT-2.3: Canonical bytes are bounded in size

**Files**:
- `crates/universal-decoder-core/src/traits.rs:45-90`
- `crates/universal-decoder-core/src/ir.rs:200-250`

**Verification Conditions**: 0 / ~20 proven
**Estimated Effort**: 2-3 weeks
**Blocked By**: VT-1 completion, Borsh library verification
**Last Verified**: Never

**Why Critical**:
- Canonicalization is used for signature verification
- Non-determinism breaks signature validation
- Core security property for transaction integrity

**Example Proof**:
```rust
verus! {

impl<'a> TxIR<'a, V> {
    pub fn to_canonical_bytes(&self) -> (result: Vec<u8>)
        ensures
            // Determinism: calling twice yields same result
            result == self.to_canonical_bytes(),
            // Size bound: output is reasonable
            result.len() <= MAX_CANONICAL_SIZE,
    {
        borsh::to_vec(self).expect("serialization cannot fail")
    }

    #[proof]
    pub fn canonical_hash_deterministic()
        ensures
            forall |tx: &TxIR|
                tx.canonical_hash() == tx.canonical_hash()
    {
        // Proof by Borsh determinism
    }
}

} // verus!
```

---

### VT-3: Error Propagation Safety

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-3.1: Error conversion preserves error information
- ❌ VT-3.2: Error types are exhaustive (no missing cases)
- ❌ VT-3.3: Error propagation never panics

**Files**:
- `crates/universal-decoder-core/src/error.rs`

**Verification Conditions**: 0 / ~10 proven
**Estimated Effort**: 1 week
**Last Verified**: Never

---

### VT-4: Hook Execution Ordering

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-4.1: Hooks execute in defined order
- ❌ VT-4.2: Hook failures are properly propagated
- ❌ VT-4.3: Hook state is consistent

**Files**:
- `crates/universal-decoder-core/src/hooks.rs`

**Verification Conditions**: 0 / ~12 proven
**Estimated Effort**: 2 weeks
**Last Verified**: Never

---

### VT-5: Version Isolation (Const Generics)

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-5.1: TxIR<V1> cannot be cast to TxIR<V2>
- ❌ VT-5.2: Version is preserved through canonicalization

**Files**:
- `crates/universal-decoder-core/src/ir.rs`

**Verification Conditions**: 0 / ~5 proven
**Estimated Effort**: 1 week
**Last Verified**: Never

---

## Phase 2: Bitcoin Decoder (decoder-bitcoin)

### VT-10: Varint Parsing Safety ⚡ PRIORITY: HIGH

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-10.1: `parse_varint` never panics (bounds-checked)
- ❌ VT-10.2: Non-canonical varints are rejected
- ❌ VT-10.3: Varint value fits in u64

**Files**:
- `crates/decoder-encodings/src/varint.rs`

**Verification Conditions**: 0 / ~18 proven
**Estimated Effort**: 2 weeks
**Last Verified**: Never

**Why Important**:
- Varint parsing is a common attack vector
- Non-canonical encodings can cause consensus bugs
- Used in every Bitcoin transaction

**Example Proof**:
```rust
verus! {

pub fn parse_varint(bytes: &[u8]) -> (result: Result<(u64, usize), VarIntError>)
    ensures
        result.is_ok() ==> {
            let (value, len) = result.unwrap();
            // Bounds: length is valid
            len > 0 && len <= 9 && len <= bytes.len() &&
            // Canonical: no shorter encoding exists
            is_canonical_varint(value, len)
        },
        result.is_err() ==> {
            // Error only if input is invalid
            bytes.len() < 1 || !is_valid_varint_prefix(bytes[0])
        }
{
    // Implementation with proof annotations
}

} // verus!
```

---

### VT-11: Transaction Parsing Bounds

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-11.1: Input parsing never reads out of bounds
- ❌ VT-11.2: Output parsing never reads out of bounds
- ❌ VT-11.3: Witness parsing is bounds-checked

**Files**:
- `crates/decoder-bitcoin/src/parsing.rs`

**Verification Conditions**: 0 / ~45 proven
**Estimated Effort**: 4-6 weeks
**Last Verified**: Never

---

### VT-12: Fee Calculation Overflow Safety

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-12.1: Total input value doesn't overflow
- ❌ VT-12.2: Total output value doesn't overflow
- ❌ VT-12.3: Fee calculation handles underflow correctly

**Files**:
- `crates/decoder-bitcoin/src/lib.rs` (calculate_fee method)

**Verification Conditions**: 0 / ~8 proven
**Estimated Effort**: 1 week
**Last Verified**: Never

---

### VT-13: TXID Calculation Correctness

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-13.1: TXID is deterministic (same tx → same TXID)
- ❌ VT-13.2: SegWit TXID excludes witness data
- ❌ VT-13.3: Hash computation never panics

**Files**:
- `crates/decoder-bitcoin/src/lib.rs` (compute_txid method)

**Verification Conditions**: 0 / ~6 proven
**Estimated Effort**: 1 week
**Last Verified**: Never

---

### VT-14: Bitcoin Canonicalization Injectivity

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-14.1: `encode(decode(bytes)) == bytes` (roundtrip)
- ❌ VT-14.2: Signature verification is preserved

**Files**:
- `crates/decoder-bitcoin/src/lib.rs` (canonicalize method)

**Verification Conditions**: 0 / ~15 proven
**Estimated Effort**: 3-4 weeks
**Last Verified**: Never

---

## Phase 3: Ethereum Decoder (decoder-ethereum)

### VT-20: RLP Parsing Safety

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-20.1: RLP decode never panics
- ❌ VT-20.2: RLP decode rejects malformed input
- ❌ VT-20.3: RLP length fields are validated

**Files**:
- `crates/decoder-encodings/src/rlp.rs`

**Verification Conditions**: 0 / ~30 proven
**Estimated Effort**: 3-4 weeks
**Last Verified**: Never

---

### VT-21: Gas Calculation Overflow Safety

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-21.1: Gas limit * gas price doesn't overflow
- ❌ VT-21.2: EIP-1559 fee calculations are safe
- ❌ VT-21.3: Priority fee + base fee doesn't overflow

**Files**:
- `crates/decoder-ethereum/src/lib.rs`

**Verification Conditions**: 0 / ~10 proven
**Estimated Effort**: 2 weeks
**Last Verified**: Never

---

### VT-22: EIP-2718 Transaction Type Detection

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-22.1: Transaction type is correctly identified
- ❌ VT-22.2: Type-specific parsing is safe
- ❌ VT-22.3: Unknown types are rejected

**Files**:
- `crates/decoder-ethereum/src/lib.rs`

**Verification Conditions**: 0 / ~8 proven
**Estimated Effort**: 1 week
**Last Verified**: Never

---

### VT-23: Signature Recovery Safety

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-23.1: Recovery ID (v) is validated
- ❌ VT-23.2: Signature (r, s) are in valid range
- ❌ VT-23.3: Address recovery never panics

**Files**:
- `crates/decoder-ethereum/src/lib.rs`

**Verification Conditions**: 0 / ~12 proven
**Estimated Effort**: 2 weeks
**Last Verified**: Never

---

### VT-24: Ethereum Canonicalization Determinism

**Status**: ❌ Not Started
**Properties**:
- ❌ VT-24.1: RLP encoding is deterministic
- ❌ VT-24.2: Transaction hash is deterministic

**Files**:
- `crates/decoder-ethereum/src/lib.rs`

**Verification Conditions**: 0 / ~10 proven
**Estimated Effort**: 2 weeks
**Last Verified**: Never

---

## Verification Timeline

### Month 1: Foundation
- Week 1-2: Install Verus, verify VT-1.1 (Amount::checked_add)
- Week 3-4: Complete VT-1 (Amount arithmetic)

### Month 2: Core Properties
- Week 1-2: VT-2 (Canonicalization determinism)
- Week 3-4: VT-3 (Error propagation)

### Month 3-4: Bitcoin Decoder
- Week 1-2: VT-10 (Varint parsing)
- Week 3-4: VT-12 (Fee calculation)
- Week 5-6: VT-13 (TXID calculation)
- Week 7-8: VT-11 (Transaction parsing - partial)

### Month 5-6: Ethereum Decoder
- Week 1-2: VT-20 (RLP parsing)
- Week 3-4: VT-21 (Gas calculations)
- Week 5-6: VT-22, VT-23 (Type detection, signature recovery)

---

## How to Update This Document

When you complete a verification target:

1. **Change Status**: ❌ Not Started → ⏳ In Progress → ✅ Verified
2. **Update VCs**: Record actual VCs proven (e.g., `15 / 15 proven`)
3. **Add Verification Date**: Update "Last Verified" timestamp
4. **Update Coverage Table**: Recalculate percentages
5. **Link to PR**: Add link to the PR that added the verification

**Example**:
```markdown
### VT-1: Amount Arithmetic Safety ⚡ PRIORITY: CRITICAL

**Status**: ✅ Verified (PR #123)
**Properties**:
- ✅ VT-1.1: `checked_add` overflow detection (5 VCs proven)
- ✅ VT-1.2: `checked_sub` underflow detection (4 VCs proven)
- ✅ VT-1.3: `checked_mul` overflow detection (6 VCs proven)
- ❌ VT-1.4: Decimal conversion correctness (TODO)

**Verification Conditions**: 15 / 15 proven
**Last Verified**: 2025-01-20
```

---

## Quick Reference: Verification Status

| Symbol | Meaning |
|--------|---------|
| ❌ | Not started |
| ⏳ | In progress |
| ✅ | Verified |
| ⚠️ | Partial verification |
| 🚫 | Blocked |
| ⚡ | Critical priority |

---

## Resources

- **Verus Tutorial**: https://verus-lang.github.io/verus/guide/
- **Coverage Tracking**: See `docs/VERUS_VERIFICATION_COVERAGE.md`
- **Implementation Guide**: See `docs/FORMAL_VERIFICATION.md`
- **Verification Scripts**: See `tools/verus-coverage.sh`

---

**Next Action**: Install Verus and start with VT-1.1 (Amount::checked_add)
