# Ethereum Decoder: Pure Rust Implementation Strategy

## Current Status (Phase 1.5)

**Dependencies Moved**: ✅ `ethers-core` removed from production dependencies
**Pure Rust Parsing**: ❌ Not yet implemented (Phase 2)
**Status**: Decoders will not compile in production - this is expected

## Rationale

The Universal Blockchain Decoder follows a minimal trusted computing base (TCB) philosophy. Ethereum transaction parsing requires:

1. **RLP Decoding**: Recursive Length Prefix encoding (simple, ~500 LOC)
2. **Transaction Types**: Legacy, EIP-2930, EIP-1559 (different formats)
3. **Signature Recovery**: Keccak-256 + secp256k1 (use crypto primitives)

## Phase 2: Pure Rust Implementation (Weeks 3-4)

### Ethereum Transaction Format

Ethereum supports multiple transaction types:

#### Legacy Transactions (Type 0)
```
RLP([nonce, gasPrice, gasLimit, to, value, data, v, r, s])
```

#### EIP-2930 (Type 1)
```
0x01 || RLP([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList, v, r, s])
```

#### EIP-1559 (Type 2)
```
0x02 || RLP([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, v, r, s])
```

### Implementation Plan

#### Week 1: RLP Parser
```rust
// Pure Rust RLP decoder (~500 LOC)
pub struct RlpDecoder;

impl RlpDecoder {
    pub fn decode_list(bytes: &[u8]) -> Result<Vec<RlpItem>> {
        let (prefix, offset) = read_prefix(bytes)?;
        match prefix {
            0x00..=0x7f => Ok(vec![RlpItem::Data(&bytes[0..1])]),
            0x80..=0xb7 => {
                let len = (prefix - 0x80) as usize;
                Ok(vec![RlpItem::Data(&bytes[offset..offset+len])])
            }
            0xb8..=0xbf => {
                let len_bytes = (prefix - 0xb7) as usize;
                let len = read_uint(&bytes[offset..offset+len_bytes])?;
                Ok(vec![RlpItem::Data(&bytes[offset+len_bytes..offset+len_bytes+len])])
            }
            0xc0..=0xf7 => {
                let len = (prefix - 0xc0) as usize;
                decode_list_items(&bytes[offset..offset+len])
            }
            0xf8..=0xff => {
                let len_bytes = (prefix - 0xf7) as usize;
                let len = read_uint(&bytes[offset..offset+len_bytes])?;
                decode_list_items(&bytes[offset+len_bytes..offset+len_bytes+len])
            }
        }
    }
}
```

#### Week 2: Transaction Parser
```rust
pub struct EthereumDecoder;

impl EthereumDecoder {
    pub fn parse_transaction(bytes: &[u8]) -> Result<ParsedTx> {
        // Detect transaction type
        match bytes[0] {
            0x01 => Self::parse_eip2930(&bytes[1..]),
            0x02 => Self::parse_eip1559(&bytes[1..]),
            _ => Self::parse_legacy(bytes),
        }
    }

    fn parse_legacy(bytes: &[u8]) -> Result<ParsedTx> {
        let items = RlpDecoder::decode_list(bytes)?;
        if items.len() != 9 {
            return Err(DecoderError::invalid_format("Expected 9 RLP items"));
        }

        Ok(ParsedTx {
            nonce: decode_uint(&items[0])?,
            gas_price: decode_uint(&items[1])?,
            gas_limit: decode_uint(&items[2])?,
            to: decode_address(&items[3])?,
            value: decode_uint(&items[4])?,
            data: items[5].as_bytes()?,
            v: decode_uint(&items[6])?,
            r: items[7].as_bytes()?,
            s: items[8].as_bytes()?,
        })
    }

    fn parse_eip1559(bytes: &[u8]) -> Result<ParsedTx> {
        let items = RlpDecoder::decode_list(bytes)?;
        // EIP-1559 specific fields...
    }
}
```

## Testing Strategy

Migration to **Alloy** (modern successor to ethers):

```toml
[dev-dependencies]
alloy = "0.1"        # Modern Ethereum library
alloy-primitives = "0.7"  # Core types
alloy-rlp = "0.3"    # RLP reference implementation
```

### Validation Tests
```rust
#[cfg(test)]
mod tests {
    use alloy::primitives::Transaction;
    use alloy_rlp::Decodable;

    #[test]
    fn test_against_alloy() {
        let raw_tx = include_bytes!("fixtures/eip1559_tx.bin");

        // Our pure Rust implementation
        let our_result = EthereumDecoder::parse_transaction(raw_tx)?;

        // Reference implementation (alloy)
        let ref_tx = Transaction::decode(&mut &raw_tx[..])?;

        // Validate they match
        assert_eq!(our_result.nonce, ref_tx.nonce);
        assert_eq!(our_result.gas_limit, ref_tx.gas_limit);
        // ... more assertions
    }
}
```

## Why Alloy over ethers?

1. **Modern**: Actively developed, future-focused
2. **Modular**: Smaller dependency footprint
3. **Performance**: Better optimizations
4. **Type Safety**: Improved type system

## RLP Implementation

RLP is simple enough to implement in pure Rust (~500 LOC):

### Advantages of Pure Rust RLP:
1. **No Dependencies**: Zero external crates
2. **Formally Verifiable**: Simple algorithm, easy to prove correct
3. **Auditable**: Entire implementation readable in one sitting
4. **No Surprises**: Complete control over edge cases

### RLP Specification:
- [Ethereum RLP](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/)
- [EIP-2718: Typed Transaction Envelope](https://eips.ethereum.org/EIPS/eip-2718)
- [EIP-1559: Fee Market](https://eips.ethereum.org/EIPS/eip-1559)

## Migration Path

### Phase 1.5 (Current)
- ✅ Dependencies documented
- ✅ Strategy defined
- ✅ Alloy planned for test validation
- ❌ Decoders do not compile (expected)

### Phase 2 (Weeks 3-4)
- Week 3: Pure Rust RLP decoder
- Week 3: Legacy transaction parsing
- Week 4: EIP-2930 and EIP-1559 support
- Week 4: Comprehensive test suite with Alloy validation

### Phase 3 (Week 5)
- Integration tests with mainnet transactions
- Performance benchmarks vs alloy
- Fuzzing campaign

## Dependencies Comparison

### Before (ethers-core)
```
ethers-core v2.0
├── ~40 dependencies
├── ~30k LOC
└── Async runtime (tokio)
```

### After (Pure Rust)
```
decoder-ethereum
├── RLP decoder (~500 LOC, pure Rust)
├── Transaction parser (~300 LOC)
└── sha3 (already in core for Keccak-256)
```

### Test Validation (alloy)
```
[dev-dependencies]
alloy v0.1
├── Modular design
├── Modern API
└── Better performance than ethers
```

## Dependency Philosophy

> "The best code is no code. The second best is code that can be formally verified."

Ethereum transaction parsing is straightforward:
1. **RLP decoding**: Well-specified, ~500 LOC
2. **Transaction types**: 3 formats, ~300 LOC total
3. **Validation**: Compare against Alloy in tests

Total pure Rust code: ~800 LOC vs ~30k LOC dependency tree.

For a security-critical library, we choose auditability and verifiability.
