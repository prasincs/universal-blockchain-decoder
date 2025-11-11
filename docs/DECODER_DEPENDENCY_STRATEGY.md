# Decoder Dependency Strategy: Test-Only Blockchain Libraries

## Core Principle

> **Blockchain-specific libraries (bitcoin, alloy, solana-sdk) should ONLY be used in dev-dependencies for testing purposes.**

This ensures:
1. Decoders remain **pure Rust implementations**
2. No dependency on external parsing libraries in production
3. Tests can validate against **real blockchain libraries**
4. Clear separation between implementation and validation

## Architecture

### Current (Problematic) Structure

```toml
# ❌ WRONG: decoder-bitcoin/Cargo.toml
[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
bitcoin = "0.31"  # ❌ BAD: Production dependency on external library
serde = "1.0"
hex = "0.4"
```

**Problem**: The decoder **depends on** the `bitcoin` crate for parsing. This:
- Adds 50k+ LOC to the TCB
- Makes our decoder a thin wrapper around `bitcoin` crate
- Defeats the purpose of having a universal decoder
- Cannot be formally verified (depends on unverified code)

### Target (Correct) Structure

```toml
# ✅ GOOD: decoder-bitcoin/Cargo.toml
[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
# NO blockchain-specific dependencies here!
# Decoder should be pure Rust parsing

[dev-dependencies]
bitcoin = "0.31"  # ✅ GOOD: Only for testing/validation
proptest = "1.4"
hex-literal = "0.4"
```

**Benefits**:
- Decoder is **self-contained** pure Rust
- Can be formally verified
- Tests validate against reference implementation
- Minimal TCB

## Implementation Strategy

### Phase 1: Understand Current Dependencies

```bash
# Check which decoders use blockchain libraries in production
grep -r "bitcoin\|alloy\|solana-sdk" crates/*/Cargo.toml
```

### Phase 2: Reimplement Parsing Logic

#### Bitcoin Decoder: Remove `bitcoin` Crate Dependency

**Current (Wrong) Approach**:
```rust
// ❌ Using bitcoin crate for parsing
use bitcoin::Transaction as BitcoinTx;

pub fn decode(bytes: &[u8]) -> Result<BitcoinTransaction, DecoderError> {
    let tx = BitcoinTx::deserialize(bytes)?;  // ❌ Delegates to external library
    BitcoinTransaction::from_bitcoin_tx(&tx)
}
```

**Target (Correct) Approach**:
```rust
// ✅ Pure Rust implementation
pub fn decode(bytes: &[u8]) -> Result<BitcoinTransaction, DecoderError> {
    let mut cursor = Cursor::new(bytes);

    // Parse version (4 bytes, little-endian)
    let version = read_u32_le(&mut cursor)?;

    // Parse segwit marker and flag (optional)
    let (marker, flag) = peek_segwit_marker(&mut cursor)?;
    let is_segwit = marker == 0x00 && flag == 0x01;

    if is_segwit {
        cursor.advance(2)?; // Skip marker and flag
    }

    // Parse input count (varint)
    let input_count = read_varint(&mut cursor)?;

    // Parse inputs
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        inputs.push(parse_input(&mut cursor)?);
    }

    // Parse output count (varint)
    let output_count = read_varint(&mut cursor)?;

    // Parse outputs
    let mut outputs = Vec::with_capacity(output_count as usize);
    for _ in 0..output_count {
        outputs.push(parse_output(&mut cursor)?);
    }

    // Parse witness data (if segwit)
    let witnesses = if is_segwit {
        parse_witnesses(&mut cursor, input_count)?
    } else {
        vec![]
    };

    // Parse locktime (4 bytes, little-endian)
    let locktime = read_u32_le(&mut cursor)?;

    Ok(BitcoinTransaction {
        version,
        inputs,
        outputs,
        witnesses,
        locktime,
        raw_bytes: bytes.to_vec(),
    })
}
```

**Utility Functions (Internal)**:
```rust
// Internal parsing utilities
// crates/decoder-bitcoin/src/parsing.rs

fn read_u32_le(cursor: &mut Cursor) -> Result<u32, DecoderError> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_varint(cursor: &mut Cursor) -> Result<u64, DecoderError> {
    let first = cursor.read_u8()?;
    match first {
        0..=0xfc => Ok(first as u64),
        0xfd => {
            let mut buf = [0u8; 2];
            cursor.read_exact(&mut buf)?;
            Ok(u16::from_le_bytes(buf) as u64)
        }
        0xfe => {
            let mut buf = [0u8; 4];
            cursor.read_exact(&mut buf)?;
            Ok(u32::from_le_bytes(buf) as u64)
        }
        0xff => {
            let mut buf = [0u8; 8];
            cursor.read_exact(&mut buf)?;
            Ok(u64::from_le_bytes(buf))
        }
    }
}

fn parse_input(cursor: &mut Cursor) -> Result<TxInput, DecoderError> {
    // Previous transaction hash (32 bytes)
    let mut prev_hash = [0u8; 32];
    cursor.read_exact(&mut prev_hash)?;

    // Previous output index (4 bytes, little-endian)
    let prev_index = read_u32_le(cursor)?;

    // Script length (varint)
    let script_len = read_varint(cursor)?;

    // Script bytes
    let mut script_bytes = vec![0u8; script_len as usize];
    cursor.read_exact(&mut script_bytes)?;

    // Sequence (4 bytes, little-endian)
    let sequence = read_u32_le(cursor)?;

    Ok(TxInput {
        prev_hash,
        prev_index,
        script_sig: script_bytes,
        sequence,
    })
}

fn parse_output(cursor: &mut Cursor) -> Result<TxOutput, DecoderError> {
    // Value (8 bytes, little-endian)
    let mut buf = [0u8; 8];
    cursor.read_exact(&mut buf)?;
    let value = u64::from_le_bytes(buf);

    // Script length (varint)
    let script_len = read_varint(cursor)?;

    // Script bytes
    let mut script_pubkey = vec![0u8; script_len as usize];
    cursor.read_exact(&mut script_pubkey)?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}
```

### Phase 3: Use Blockchain Libraries for Testing Only

```rust
// crates/decoder-bitcoin/tests/validation.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Transaction as BitcoinTx;  // ✅ OK in tests
    use bitcoin::consensus::deserialize;

    #[test]
    fn test_decode_matches_bitcoin_crate() {
        let tx_bytes = include_bytes!("fixtures/btc_genesis_coinbase.bin");

        // Our implementation
        let our_tx = BitcoinDecoder::decode(tx_bytes).unwrap();

        // Reference implementation (bitcoin crate)
        let ref_tx: BitcoinTx = deserialize(tx_bytes).unwrap();

        // Validate our parsing matches reference
        assert_eq!(our_tx.version, ref_tx.version as u32);
        assert_eq!(our_tx.inputs.len(), ref_tx.input.len());
        assert_eq!(our_tx.outputs.len(), ref_tx.output.len());

        for (our_input, ref_input) in our_tx.inputs.iter().zip(ref_tx.input.iter()) {
            assert_eq!(our_input.prev_hash, ref_input.previous_output.txid.as_ref());
            assert_eq!(our_input.prev_index, ref_input.previous_output.vout);
            assert_eq!(our_input.sequence, ref_input.sequence.0);
        }

        for (our_output, ref_output) in our_tx.outputs.iter().zip(ref_tx.output.iter()) {
            assert_eq!(our_output.value, ref_output.value);
            assert_eq!(our_output.script_pubkey, ref_output.script_pubkey.as_bytes());
        }
    }

    #[test]
    fn test_all_bitcoin_test_vectors() {
        // Test against all Bitcoin Core test vectors
        let test_vectors = load_bitcoin_test_vectors();

        for (tx_bytes, expected) in test_vectors {
            let our_result = BitcoinDecoder::decode(&tx_bytes);
            let ref_result: Result<BitcoinTx, _> = deserialize(&tx_bytes);

            // Both should succeed or fail together
            assert_eq!(our_result.is_ok(), ref_result.is_ok());

            if let (Ok(our_tx), Ok(ref_tx)) = (our_result, ref_result) {
                validate_match(&our_tx, &ref_tx);
            }
        }
    }
}
```

## Migration Plan

### Step 1: Audit Current Dependencies

```bash
# For each decoder crate
cd crates/decoder-bitcoin

# Check production dependencies
cargo tree --depth 1

# Identify blockchain-specific dependencies
# bitcoin, alloy, solana-sdk, etc.
```

### Step 2: Implement Pure Rust Parsing

For each decoder:

1. **Identify parsing logic**: What does the external library do?
2. **Reimplement in pure Rust**: Use standard library only
3. **Add comprehensive tests**: Validate against external library

**Example: Bitcoin Transaction Parsing**

```rust
// Bitcoin transaction format:
// - Version (4 bytes)
// - [Segwit marker + flag (2 bytes, optional)]
// - Input count (varint)
// - Inputs (variable)
// - Output count (varint)
// - Outputs (variable)
// - [Witness data (variable, if segwit)]
// - Locktime (4 bytes)

// Reference: https://developer.bitcoin.org/reference/transactions.html
```

### Step 3: Move Blockchain Libraries to dev-dependencies

```diff
# crates/decoder-bitcoin/Cargo.toml

[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
-bitcoin = "0.31"  # REMOVE from dependencies

[dev-dependencies]
+bitcoin = "0.31"  # MOVE to dev-dependencies
proptest = "1.4"
hex-literal = "0.4"
```

### Step 4: Update Tests

```rust
// Before: Tests use our wrapper around bitcoin crate
#[test]
fn test_decode() {
    let tx = BitcoinDecoder::decode(bytes).unwrap();
    assert!(tx.version > 0);  // Weak test
}

// After: Tests validate against reference implementation
#[test]
fn test_decode_matches_reference() {
    use bitcoin::consensus::deserialize;

    let our_tx = BitcoinDecoder::decode(bytes).unwrap();
    let ref_tx: bitcoin::Transaction = deserialize(bytes).unwrap();

    // Strong validation
    assert_eq!(our_tx.version, ref_tx.version as u32);
    assert_eq!(our_tx.inputs.len(), ref_tx.input.len());
    // ... comprehensive comparison
}
```

## Dependency Structure

### Final Target Structure

```
universal-blockchain-decoder/
├── crates/
│   ├── universal-decoder-core/
│   │   ├── [dependencies]
│   │   │   ├── serde
│   │   │   ├── borsh
│   │   │   ├── thiserror
│   │   │   ├── sha2
│   │   │   └── sha3
│   │   └── [dev-dependencies]
│   │       ├── proptest
│   │       └── criterion
│   │
│   ├── decoder-bitcoin/
│   │   ├── [dependencies]
│   │   │   └── universal-decoder-core  # ONLY core dependency
│   │   └── [dev-dependencies]
│   │       ├── bitcoin         # ✅ For testing only
│   │       ├── proptest
│   │       └── hex-literal
│   │
│   ├── decoder-ethereum/
│   │   ├── [dependencies]
│   │   │   └── universal-decoder-core  # ONLY core dependency
│   │   └── [dev-dependencies]
│   │       ├── alloy     # ✅ For testing only
│   │       ├── proptest
│   │       └── hex-literal
│   │
│   └── decoder-solana/
│       ├── [dependencies]
│       │   └── universal-decoder-core  # ONLY core dependency
│       └── [dev-dependencies]
│           ├── solana-sdk              # ✅ For testing only
│           ├── solana-transaction-status
│           └── proptest
```

## Benefits

### 1. Minimal TCB

**Before**:
- Core: 2.5k LOC + 42k dependencies
- Bitcoin decoder: 1k LOC + **50k bitcoin crate**
- **Total TCB**: ~95k LOC

**After**:
- Core: 2.5k LOC + 42k dependencies
- Bitcoin decoder: 3k LOC (pure Rust parsing)
- **Total TCB**: ~47k LOC

**Reduction**: ~50% smaller TCB

### 2. Formal Verification Possible

```rust
verus! {

// Can verify pure Rust parsing
#[verifier::proof]
pub fn parse_varint(bytes: &[u8]) -> (result: Result<(u64, usize), DecoderError>)
    requires
        bytes.len() > 0,
    ensures
        result.is_ok() ==> {
            let (value, consumed) = result.unwrap();
            consumed > 0 && consumed <= 9 &&
            consumed <= bytes.len()
        }
{
    // Pure Rust implementation can be verified
}

} // verus!
```

### 3. Independence from External Changes

- **Before**: `bitcoin` crate updates could break our decoder
- **After**: Our decoder is independent, tests validate compatibility

### 4. Clear Testing Strategy

```rust
// Test hierarchy:

// 1. Unit tests (pure Rust, no external deps)
#[test]
fn test_parse_varint() {
    assert_eq!(parse_varint(&[0xfc]), Ok((252, 1)));
    assert_eq!(parse_varint(&[0xfd, 0xfd, 0x00]), Ok((253, 3)));
}

// 2. Property tests (pure Rust)
proptest! {
    #[test]
    fn prop_parse_varint_bounds(input in any::<Vec<u8>>()) {
        if let Ok((value, consumed)) = parse_varint(&input) {
            prop_assert!(consumed > 0);
            prop_assert!(consumed <= input.len());
        }
    }
}

// 3. Validation tests (compare with bitcoin crate)
#[test]
fn test_matches_bitcoin_crate() {
    use bitcoin::consensus::deserialize;  // ✅ OK in dev-dependencies

    let our_result = BitcoinDecoder::decode(bytes);
    let ref_result = deserialize::<bitcoin::Transaction>(bytes);

    assert_eq!(our_result.is_ok(), ref_result.is_ok());
}
```

## Implementation Checklist

### Bitcoin Decoder

- [ ] Implement `read_varint()`
- [ ] Implement `parse_input()`
- [ ] Implement `parse_output()`
- [ ] Implement `parse_witness()`
- [ ] Implement segwit marker detection
- [ ] Implement full transaction parsing
- [ ] Write unit tests for each parser
- [ ] Write property tests
- [ ] Write validation tests (against `bitcoin` crate)
- [ ] Move `bitcoin` to dev-dependencies
- [ ] Remove production usage of `bitcoin` crate
- [ ] Update documentation

### Ethereum Decoder

- [ ] Implement RLP parsing (recursive length prefix)
- [ ] Implement legacy transaction parsing
- [ ] Implement EIP-2930 (access list) parsing
- [ ] Implement EIP-1559 (fee market) parsing
- [ ] Write comprehensive tests
- [ ] Validate against `alloy`
- [ ] Move `alloy` to dev-dependencies

### Solana Decoder

- [ ] Implement Solana transaction parsing
- [ ] Implement instruction decoding
- [ ] Write comprehensive tests
- [ ] Validate against `solana-sdk`
- [ ] Move `solana-sdk` to dev-dependencies

## Timeline

### Phase 1: Bitcoin Decoder (2 weeks)
- Week 1: Implement pure Rust parsing
- Week 2: Comprehensive testing and validation

### Phase 2: Ethereum Decoder (2 weeks)
- Week 1: Implement RLP and transaction parsing
- Week 2: Testing and validation

### Phase 3: Solana Decoder (2 weeks)
- Week 1: Implement transaction parsing
- Week 2: Testing and validation

**Total**: 6 weeks for all three decoders

## Risk Mitigation

### Risk 1: Parsing Bugs

**Mitigation**:
- Comprehensive test suite
- Validation against reference implementations
- Property-based testing
- Fuzzing

### Risk 2: Incomplete Specification

**Mitigation**:
- Study official specifications (BIPs, EIPs)
- Compare with reference implementations
- Test with real blockchain data
- Community review

### Risk 3: Performance Regression

**Mitigation**:
- Benchmark parsing performance
- Optimize hot paths
- Compare with reference implementations
- Profile and optimize

## Conclusion

**Key Principle**: Blockchain-specific libraries are **validation tools**, not implementation dependencies.

**Structure**:
- ✅ **Production**: Pure Rust parsing (minimal dependencies)
- ✅ **Testing**: Use blockchain libraries to validate correctness
- ✅ **Result**: Minimal TCB, formally verifiable, independent

**Next Steps**:
1. Review and approve this strategy
2. Implement Bitcoin decoder (pure Rust)
3. Move `bitcoin` crate to dev-dependencies
4. Repeat for Ethereum and Solana
5. Update documentation

---

**Remember**: We're not building wrappers around existing libraries. We're building **the reference implementation** that others will validate against.
