# Re-encoding Implementation Plan

**Status**: In Progress
**Created**: 2025-11-18
**Updated**: 2025-11-18

## Overview

This document tracks the implementation of the `ChainEncoder` trait for all supported blockchain decoders. This is a **mandatory requirement** following CLAUDE.md v0.3.0, which establishes that all decoders must support the injective property: `encode(decode(tx_bytes)) = tx_bytes`.

## Background

**Decision**: Re-encoding (for verification) is now **IN SCOPE** and **MANDATORY** (see CLAUDE.md v0.3.0)

**Why**: Without re-encoding, we cannot verify the injective property, which is fundamental for:
- Formal verification of lossless decoding
- Forensic reconstruction of exact original bytes
- Integrity checks and auditing
- Property-based testing of codec correctness

**Critical Distinction**:
- ✅ **Re-encoding**: `decoded_tx.to_bytes()` - Reconstruct original bytes (stateless, deterministic)
- ❌ **Construction**: `TransactionBuilder::new()...` - Build new transactions (stateful, out of scope)

## ChainEncoder Trait

```rust
pub trait ChainEncoder {
    /// Re-encode the transaction back to its original chain-specific byte format
    ///
    /// This method MUST produce the exact same bytes that were originally decoded.
    ///
    /// # Formal Properties
    ///
    /// Must satisfy the injective property:
    /// ```text
    /// ∀ tx_bytes: ChainDecoder::decode(tx_bytes)?.to_bytes()? == tx_bytes
    /// ```
    fn to_bytes(&self) -> Result<Vec<u8>>;
}
```

## Implementation Strategy

### Option A: Store Raw Bytes (Preferred)

**Approach**: Store original `raw_bytes: Vec<u8>` field in transaction struct during decoding.

**Implementation**:
```rust
pub struct MyTransaction {
    // ... parsed fields ...
    pub raw_bytes: Vec<u8>,  // Store original bytes
}

impl ChainEncoder for MyTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())  // Trivially injective
    }
}
```

**Advantages**:
- ✅ Trivially guarantees injectivity (stores exact original bytes)
- ✅ Simple implementation (~5 LOC)
- ✅ Fast (just clone Vec)
- ✅ No risk of serialization bugs

**Disadvantages**:
- Memory overhead (~1-5 KB per transaction, negligible)

**Verdict**: **Use this approach for all decoders where possible**

### Option B: Reconstruct Bytes (Alternative)

**Approach**: Manually serialize each field back to the original format.

**Implementation**:
```rust
impl ChainEncoder for MyTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_le_bytes());
        encode_varint(&mut bytes, self.inputs.len());
        // ... serialize each field ...
        Ok(bytes)
    }
}
```

**Advantages**:
- ✅ No memory overhead (doesn't store raw_bytes)

**Disadvantages**:
- ❌ Complex implementation (~200-300 LOC)
- ❌ Prone to bugs (easy to get byte ordering wrong)
- ❌ Harder to verify injectivity
- ❌ Requires extensive testing

**Verdict**: **Only use if raw_bytes storage is infeasible**

## Implementation Status

### ✅ Completed (3/40)

| Decoder | Status | Approach | Lines Changed | Notes |
|---------|--------|----------|---------------|-------|
| `decoder-bitcoin` | ✅ Done | Store raw_bytes | 17 | Already had raw_bytes field |
| `decoder-ethereum` | ✅ Done | Store raw_bytes | 18 | Already had raw_bytes field |
| `decoder-solana` | ✅ Done | Store raw_bytes | 19 | Already had raw_bytes field |

### 🚧 In Progress (0/40)

None currently.

### 📋 Pending Implementation (37/40)

The following decoders need `ChainEncoder` implementation:

**UTXO Family** (7 decoders):
- [ ] `decoder-zcash` - Needs investigation (enum type)
- [ ] `decoder-litecoin`
- [ ] `decoder-dogecoin`
- [ ] `decoder-bitcoin-cash`
- [ ] `decoder-bitcoin-sv`
- [ ] `decoder-dash`
- [ ] `decoder-bittensor`

**EVM Family** (8 decoders):
- [ ] `decoder-evm` (base)
- [ ] `decoder-polygon`
- [ ] `decoder-optimism`
- [ ] `decoder-arbitrum`
- [ ] `decoder-avalanche`
- [ ] `decoder-bnb`

**Account Model** (6 decoders):
- [ ] `decoder-xrp`
- [ ] `decoder-tron`
- [ ] `decoder-stellar`
- [ ] `decoder-algorand`
- [ ] `decoder-near`
- [ ] `decoder-filecoin`

**Move VM Family** (3 decoders):
- [ ] `decoder-aptos`
- [ ] `decoder-sui`
- [ ] `decoder-move` (base)

**Actor Model** (2 decoders):
- [ ] `decoder-ao`
- [ ] `decoder-polkadot` (ICP-like?)

**Cosmos SDK** (1 decoder):
- [ ] `decoder-cosmos`

**Instruction Model** (2 decoders):
- [ ] `decoder-svm` (Solana VM base)
- [ ] `decoder-ton`

**ZK Chains** (3 decoders):
- [ ] `decoder-starknet`
- [ ] `decoder-mina`
- [ ] `decoder-aleo`

**Cardano** (1 decoder):
- [ ] `decoder-cardano`

**Supporting Crates** (4 crates):
- [ ] `decoder-primitives`
- [ ] `decoder-encodings`
- [ ] `decoder-chains-common`
- [ ] `decoder-test-utils`

### Implementation Priorities

**Phase 1 (Immediate - Week 1)**:
1. UTXO family (Bitcoin forks) - Similar structure to Bitcoin
2. EVM family - Similar to Ethereum
3. Property tests for implemented decoders

**Phase 2 (Week 2)**:
4. Account model chains (XRP, Stellar, etc.)
5. Move VM family
6. More property tests

**Phase 3 (Week 3)**:
7. Cosmos SDK
8. Instruction model (TON)
9. Actor model (AO, Polkadot)

**Phase 4 (Week 4)**:
10. ZK chains (complex, may need custom approach)
11. Cardano (complex CBOR encoding)
12. Final property tests and integration testing

## Testing Requirements

For **EACH** decoder implementation, we MUST add:

### 1. Basic Roundtrip Test

```rust
#[test]
fn test_roundtrip() {
    let original_bytes = load_test_fixture("valid_tx.bin");
    let decoded = MyDecoder::decode(&original_bytes).unwrap();
    let re_encoded = decoded.to_bytes().unwrap();
    assert_eq!(original_bytes, re_encoded);
}
```

### 2. Property-Based Test (proptest)

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn injective_property(tx_bytes: Vec<u8>) {
            if let Ok(decoded) = MyDecoder::decode(&tx_bytes) {
                let re_encoded = decoded.to_bytes()?;
                prop_assert_eq!(tx_bytes, re_encoded);
            }
        }
    }
}
```

### 3. Fuzzing (cargo-fuzz)

```bash
# fuzz/fuzz_targets/roundtrip_mychain.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use decoder_mychain::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(decoded) = MyDecoder::decode(data) {
        if let Ok(re_encoded) = decoded.to_bytes() {
            assert_eq!(data, re_encoded.as_slice());
        }
    }
});
```

## Validation Checklist

Before marking a decoder as "Done", ensure:

- [ ] `ChainEncoder` trait implemented
- [ ] Basic roundtrip test passes
- [ ] Property test added (using proptest)
- [ ] Fuzzing target added (optional but recommended)
- [ ] Tests pass: `cargo test --package decoder-<chain>`
- [ ] Clippy passes: `cargo clippy --package decoder-<chain> -- -D warnings`
- [ ] Documentation updated with examples

## Special Cases

### Zcash Decoder

**Issue**: `ZcashTransaction` is an `enum` with multiple variants (Sprout, Sapling, Orchard).

**Solution**: Each variant must store raw_bytes:
```rust
pub enum ZcashTransaction {
    Sprout {
        // ... fields ...
        raw_bytes: Vec<u8>,
    },
    Sapling {
        // ... fields ...
        raw_bytes: Vec<u8>,
    },
    // etc.
}

impl ChainEncoder for ZcashTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            ZcashTransaction::Sprout { raw_bytes, .. } => Ok(raw_bytes.clone()),
            ZcashTransaction::Sapling { raw_bytes, .. } => Ok(raw_bytes.clone()),
            // etc.
        }
    }
}
```

### Decoder-EVM (Base Decoder)

**Issue**: This is a base crate used by all EVM chains. Need to verify if it has its own transaction type or just provides utilities.

**Action**: Investigate and document approach.

### Decoder-Primitives, Decoder-Encodings, etc.

**Issue**: These are utility crates, not chain decoders.

**Action**: No implementation needed, but verify they don't define transaction types that need ChainEncoder.

## Tracking Progress

**Current Status**:
- ✅ **Done**: 3/40 (7.5%)
- 🚧 **In Progress**: 0/40 (0%)
- 📋 **Pending**: 37/40 (92.5%)

**Estimated Effort**:
- Store raw_bytes approach: ~30 minutes per decoder (review + add field + implement trait + test)
- Reconstruct bytes approach: ~4-6 hours per decoder (implement serialization + extensive testing)

**Target Completion**: End of Week 4 (all decoders with ChainEncoder + comprehensive tests)

## Related Documents

- `CLAUDE.md` v0.3.0 - Design requirements
- `TESTING_STRATEGY.md` - Property testing guidelines
- `ROADMAP.md` - Overall project timeline

---

**Last Updated**: 2025-11-18
**Next Review**: After Phase 1 completion (Week 1)
