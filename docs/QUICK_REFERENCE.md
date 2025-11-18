# Polygon zkEVM Decoder - Quick Reference

## TL;DR

**Polygon zkEVM transactions are 100% identical to Ethereum in format.**

**Reuse decoder-ethereum with chain ID validation (1101/1442) - zero duplication.**

**Implementation: ~120 lines of code. Time: 2-3 hours.**

---

## What Can Be Reused Directly

### From decoder-ethereum/src/types.rs

- ✅ `parse_legacy_transaction()` - 30 lines
- ✅ `parse_eip2930()` - 20 lines
- ✅ `parse_eip1559()` - 20 lines
- ✅ `parse_eip4844()` - 15 lines
- ✅ `parse_address_field()` - 15 lines
- ✅ `parse_signature_component()` - 15 lines
- ✅ `parse_access_list()` - 35 lines
- ✅ `TxType` enum - 50 lines
- ✅ `EthereumTransaction` struct - 40 lines
- ✅ `AccessListItem` struct - 20 lines
- ✅ All Borsh serialization - 50 lines
- ✅ `Canonicalizer` impl - 170 lines
- ✅ `TxHashable` impl - 10 lines

**Total: ~500 lines of reusable code**

### From decoder-ethereum/src/lib.rs

- ✅ `ChainDecoder` trait pattern - 30 lines (adapt chain IDs)
- ✅ `validate_format()` - 10 lines (reuse as-is)
- ✅ Hook system - 25 lines (reuse as-is)
- ✅ Test patterns - reuse all

---

## What Needs Small Changes

| Change | Location | Effort |
|--------|----------|--------|
| Chain ID validation | `ChainIdentity::chain_id()` | 1 line |
| Chain name | `ChainIdentity::chain_name()` | 1 line |
| Chain ID checks | `ChainDecoder::decode()` | 5 lines |

---

## Minimal Implementation

```rust
// crates/decoder-polygon-zkevm/src/lib.rs

use decoder_ethereum::{EthereumDecoder, types::EthereumTransaction};
use universal_decoder_core::prelude::*;

pub struct PolygonZkEvmChain;

impl ChainIdentity for PolygonZkEvmChain {
    fn chain_id(&self) -> u64 { 1101 }
    fn chain_name(&self) -> &str { "Polygon zkEVM" }
    fn chain_family(&self) -> ChainFamily { ChainFamily::Account }
}

pub struct PolygonZkEvmDecoder;

impl ChainDecoder for PolygonZkEvmDecoder {
    type TxSpecific = EthereumTransaction;
    type Chain = PolygonZkEvmChain;

    fn decode(raw_bytes: &[u8]) -> Result<EthereumTransaction> {
        let tx = EthereumDecoder::decode(raw_bytes)?;
        if let Some(chain_id) = tx.chain_id {
            if chain_id != 1101 && chain_id != 1442 {
                return Err(DecoderError::invalid_structure(
                    format!("Invalid zkEVM chain ID: {}", chain_id)
                ));
            }
        }
        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        EthereumDecoder::validate_format(raw_bytes)
    }
}
```

**Total: ~50 lines**

---

## File Paths

### Source Code

**Ethereum Decoder** (Model to reuse)
```
/home/user/universal-blockchain-decoder/crates/decoder-ethereum/src/lib.rs
/home/user/universal-blockchain-decoder/crates/decoder-ethereum/src/types.rs
```

**Polygon Decoder** (Same pattern, different chain ID)
```
/home/user/universal-blockchain-decoder/crates/decoder-polygon/src/lib.rs
```

**Polygon zkEVM** (Where to add new code)
```
/home/user/universal-blockchain-decoder/crates/decoder-polygon-zkevm/src/lib.rs (NEW)
/home/user/universal-blockchain-decoder/crates/decoder-polygon-zkevm/Cargo.toml (NEW)
```

**Cryptographic Primitives** (For future block verification)
```
/home/user/universal-blockchain-decoder/crates/decoder-crypto-zk/src/field/goldilocks.rs
/home/user/universal-blockchain-decoder/crates/decoder-crypto-zk/src/hash/poseidon_goldilocks.rs
```

### Core Infrastructure

**Traits & Types**
```
/home/user/universal-blockchain-decoder/crates/universal-decoder-core/src/traits.rs
/home/user/universal-blockchain-decoder/crates/universal-decoder-core/src/chain.rs
/home/user/universal-blockchain-decoder/crates/universal-decoder-core/src/ir.rs
```

**RLP Encoding**
```
/home/user/universal-blockchain-decoder/crates/decoder-encodings/src/rlp.rs
```

### Documentation Generated

**Full Analysis** (16KB, comprehensive)
```
/home/user/universal-blockchain-decoder/docs/ETHEREUM_DECODER_REUSABILITY.md
```

**Architecture Guide** (12KB, with diagrams)
```
/home/user/universal-blockchain-decoder/docs/ZKEVM_DECODER_ARCHITECTURE.md
```

**This Quick Reference**
```
/home/user/universal-blockchain-decoder/docs/QUICK_REFERENCE.md
```

---

## Implementation Checklist

### Phase 1: Basic Decoder (2-3 hours)

- [ ] Create `crates/decoder-polygon-zkevm/Cargo.toml`
  ```toml
  [dependencies]
  decoder-ethereum = { path = "../decoder-ethereum" }
  universal-decoder-core = { path = "../universal-decoder-core" }
  serde = { workspace = true }
  ```

- [ ] Create `crates/decoder-polygon-zkevm/src/lib.rs`
  - Copy ChainDecoder pattern from decoder-ethereum
  - Replace chain IDs: 1 → 1101 (mainnet) or 1442 (testnet)
  - Add chain ID validation logic

- [ ] Create unit tests
  - test_chain_identity()
  - test_chain_id_validation()
  - test_format_validation()

### Phase 2: Integration Testing (1-2 hours)

- [ ] Get real zkEVM transactions from https://zkevm.polygonscan.com/
- [ ] Create `tests/integration_tests.rs`
- [ ] Create `tests/zkevm_fixtures.rs`
- [ ] Verify TxIR canonicalization

### Phase 3: Advanced (Optional, Future)

- [ ] Integrate decoder-crypto-zk for zkTrie analysis
- [ ] Add SNARK proof verification hooks
- [ ] Document block verification (separate from transaction decoding)

---

## Key Numbers

| Metric | Value |
|--------|-------|
| Code reuse from decoder-ethereum | ~500 lines (80%) |
| New code needed | ~70 lines (20%) |
| Total decoder size | ~570 lines |
| Implementation time | 2-3 hours |
| Testing time | 1-2 hours |
| Code duplication | 0% |
| Dependencies added | 0 (decoder-ethereum already imported) |

---

## Why This Works

1. **EVM Compatibility**: zkEVM uses identical RLP encoding to Ethereum
2. **Same Field Layout**: nonce, gas_limit, to, value, data, etc. are identical
3. **Same Signature Scheme**: ECDSA secp256k1 (v, r, s)
4. **Same TX Types**: Legacy, EIP-2930, EIP-1559, EIP-4844 all supported
5. **Same Hashing**: Keccak256 for transaction hashes
6. **Only Chain ID Differs**: 1101 vs 1 for Ethereum

The proof layer (SNARK verification, zkTrie) is separate from transaction decoding.

---

## Alternative: If Duplication Preferred

Copy `decoder-ethereum/src/types.rs` into `decoder-polygon-zkevm/src/types.rs` and modify chain IDs inline. This gives:

- More explicit control
- No dependency on decoder-ethereum
- Maintenance burden of keeping two copies in sync

**Not recommended** - use Option A (dependency approach) instead.

---

## Testing Examples

### Test 1: Chain ID Validation
```rust
#[test]
fn test_rejects_ethereum_chain_id() {
    let ethereum_tx = create_ethereum_tx_with_chain_id(1);
    assert!(PolygonZkEvmDecoder::decode(&ethereum_tx).is_err());
}

#[test]
fn test_accepts_zkevm_mainnet() {
    let zk_tx = create_tx_with_chain_id(1101);
    assert!(PolygonZkEvmDecoder::decode(&zk_tx).is_ok());
}

#[test]
fn test_accepts_zkevm_testnet() {
    let zk_tx = create_tx_with_chain_id(1442);
    assert!(PolygonZkEvmDecoder::decode(&zk_tx).is_ok());
}
```

### Test 2: Real Data
```rust
#[test]
fn test_real_zkevm_transaction() {
    // Real tx: https://zkevm.polygonscan.com/tx/0x...
    let tx_hex = "f86c0180825208...";
    let tx_bytes = hex::decode(tx_hex).unwrap();
    
    let tx = PolygonZkEvmDecoder::decode(&tx_bytes).unwrap();
    
    assert_eq!(tx.chain_id, Some(1101));
    assert!(tx.validate().is_ok());
}
```

---

## Further Reading

1. Full reusability analysis: `/docs/ETHEREUM_DECODER_REUSABILITY.md`
2. Architecture diagrams: `/docs/ZKEVM_DECODER_ARCHITECTURE.md`
3. zkEVM docs: https://docs.polygon.technology/zkEVM/
4. Chain registry: https://chainlist.org/ (search "Polygon zkEVM")

---

**Status**: Ready to implement Phase 1. All information gathered and documented.
