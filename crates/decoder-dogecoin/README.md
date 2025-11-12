# decoder-dogecoin

Dogecoin transaction decoder for the Universal Blockchain Decoder.

## Status

**Phase**: 1 (Scaffolding) ✅
**Complexity**: Very Low (Bitcoin clone)
**Timeline**: Week 1-2

## Overview

Dogecoin is a Bitcoin fork with identical transaction format. Will reuse Bitcoin decoder.

## Chain Specification

- **Chain ID**: 3
- **Consensus**: Proof of Work (Scrypt)
- **Chain Family**: UTXO
- **Transaction Format**: Identical to Bitcoin (no SegWit)

## Implementation Plan

### Phase 2: Pure Rust Implementation

```rust
// Reuse Bitcoin decoder:
pub struct DogecoinDecoder;

impl ChainDecoder for DogecoinDecoder {
    type TxSpecific = BitcoinTransaction;  // Reuse!

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        BitcoinDecoder::decode_legacy(raw_bytes)  // No SegWit
    }
}
```

## Key Differences from Bitcoin

1. Different chain ID (3 vs 0)
2. No SegWit support
3. Scrypt PoW (not relevant for tx decoding)
4. Different block reward schedule

## License

MIT OR Apache-2.0
