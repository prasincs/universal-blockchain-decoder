# Bitcoin Decoder: Pure Rust Implementation Strategy

## Current Status (Phase 1.5)

**Dependencies Moved**: ✅ `bitcoin` crate moved to dev-dependencies
**Pure Rust Parsing**: ❌ Not yet implemented (Phase 2)
**Status**: Decoders will not compile in production - this is expected

## Rationale

The Universal Blockchain Decoder follows a minimal trusted computing base (TCB) philosophy:

1. **Core Library** (< 3000 LOC): Formally verifiable, minimal dependencies
2. **Decoder Libraries**: Pure Rust, independently auditable

### Why Remove `bitcoin` Crate from Production Dependencies?

1. **Supply Chain Security**: Reduces attack surface
2. **Auditability**: Pure Rust parser is easier to audit than wrapped library
3. **Control**: Full control over parsing logic and error handling
4. **Verification**: Enables formal verification of decoder logic
5. **Minimal TCB**: Keeps the trusted codebase small

## Phase 2: Pure Rust Implementation (Weeks 1-2)

### Bitcoin Transaction Parsing

Bitcoin transactions use a simple binary format:
- **Version**: 4 bytes (little-endian u32)
- **Input Count**: VarInt
- **Inputs**: Array of `TxIn`
- **Output Count**: VarInt
- **Outputs**: Array of `TxOut`
- **Locktime**: 4 bytes (little-endian u32)

### Implementation Plan

#### Week 1: Core Parsing
```rust
// Pure Rust parser (no external dependencies)
pub struct BitcoinDecoder;

impl BitcoinDecoder {
    pub fn parse_transaction(bytes: &[u8]) -> Result<ParsedTx> {
        let mut cursor = 0;

        // Parse version
        let version = read_u32_le(&bytes[cursor..cursor+4])?;
        cursor += 4;

        // Parse inputs
        let (input_count, len) = read_varint(&bytes[cursor..])?;
        cursor += len;
        let inputs = parse_inputs(&bytes[cursor..], input_count)?;

        // ... continue parsing
    }
}
```

#### Week 2: Validation & Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin; // Available in dev-dependencies

    #[test]
    fn test_against_bitcoin_crate() {
        let raw_tx = include_bytes!("fixtures/tx_block_100000.bin");

        // Our pure Rust implementation
        let our_result = BitcoinDecoder::parse_transaction(raw_tx)?;

        // Reference implementation (bitcoin crate)
        let ref_tx: bitcoin::Transaction = deserialize(raw_tx)?;

        // Validate they match
        assert_eq!(our_result.version, ref_tx.version);
        assert_eq!(our_result.inputs.len(), ref_tx.input.len());
        // ... more assertions
    }
}
```

## Testing Strategy

The `bitcoin` crate remains in `dev-dependencies` for:

1. **Validation Testing**: Compare pure Rust output against reference implementation
2. **Fixture Generation**: Generate test fixtures from known-good transactions
3. **Property Testing**: Verify parsing invariants
4. **Fuzzing**: Cross-check against `bitcoin` crate for discovered inputs

## Migration Path

### Phase 1.5 (Current)
- ✅ Dependencies documented
- ✅ Strategy defined
- ❌ Decoders do not compile (expected)

### Phase 2 (Weeks 1-2)
- Implement pure Rust Bitcoin transaction parser
- VarInt encoding/decoding
- Script parsing (if needed for TxIR)
- Comprehensive test suite with `bitcoin` crate validation

### Phase 3 (Week 3)
- Integration tests with real blockchain data
- Performance benchmarks
- Fuzzing campaign

## References

- [Bitcoin Developer Reference](https://developer.bitcoin.org/reference/transactions.html)
- [Bitcoin Core Source](https://github.com/bitcoin/bitcoin/blob/master/src/primitives/transaction.h)
- [BIP 144 - Segregated Witness](https://github.com/bitcoin/bips/blob/master/bip-0144.mediawiki)

## Dependency Philosophy

> "The best code is no code. The second best is code that can be formally verified."

By implementing pure Rust parsing:
- Smaller trusted code base
- Independent auditability
- Formal verification possible
- Supply chain security

The `bitcoin` crate is excellent but adds ~50k LOC to the dependency tree. For a security-critical library, we choose auditability over convenience.
