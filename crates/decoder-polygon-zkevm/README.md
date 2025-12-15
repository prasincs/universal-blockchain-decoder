# Polygon zkEVM Decoder

Polygon zkEVM transaction decoder for the Universal Blockchain Decoder project.

## Overview

Polygon zkEVM is an EVM-compatible zero-knowledge rollup that uses the **exact same transaction format as Ethereum**. This decoder achieves ~80% code reuse by delegating to `decoder-ethereum`.

## Implementation Strategy

### Reusability Highlights

| Component | Reusability | Source |
|-----------|------------|--------|
| Transaction parsing | ✅ 100% | `decoder-ethereum` |
| RLP decoding | ✅ 100% | `decoder-ethereum` |
| Transaction types | ✅ 100% | `EthereumTransaction` |
| Chain validation | 🔧 Custom | zkEVM chain IDs (1101, 1442) |
| zkTrie analysis | ⚡ New | Poseidon-based Merkle tree |

**Total Code:**
- Decoder implementation: ~200 lines
- zkTrie module: ~300 lines
- Tests: ~150 lines
- **Total: ~650 lines** (vs ~2000+ if implemented from scratch)

### Architecture Pattern

```rust
// Reuse Ethereum transaction type
pub struct PolygonZkevmDecoder;

impl ChainDecoder for PolygonZkevmDecoder {
    type TxSpecific = EthereumTransaction; // ← 100% reuse

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // 1. Use Ethereum decoder
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // 2. Validate zkEVM chain ID
        if let Some(chain_id) = tx.chain_id {
            if ![1101, 1442].contains(&chain_id) {
                return Err(/* ... */);
            }
        }

        Ok(tx)
    }
}
```

## Supported Networks

- **Chain ID 1101**: Polygon zkEVM Mainnet
- **Chain ID 1442**: Polygon zkEVM Testnet (Cardona)

## Transaction Format

Polygon zkEVM transactions are **identical** to Ethereum:

- ✅ RLP-encoded
- ✅ EIP-2718 transaction types (Legacy, EIP-2930, EIP-1559, EIP-4844)
- ✅ Same field layout (nonce, gasPrice, gasLimit, to, value, data, v, r, s)
- ✅ Same signature scheme (ECDSA secp256k1)
- ✅ Same address format (20 bytes)
- ⚠️ **Only difference**: Chain ID (1101/1442 vs 1)

## Zero-Knowledge Proof System

While transactions use standard EVM format, Polygon zkEVM uses different cryptography for **block-level** proofs:

### Transaction Layer (This Decoder)
- **Hash**: Keccak256 (identical to Ethereum)
- **Encoding**: RLP (identical to Ethereum)
- **Signatures**: ECDSA secp256k1 (identical to Ethereum)

### State/Proof Layer (zkTrie Module)
- **Field**: Goldilocks (p = 2^64 - 2^32 + 1)
- **Hash**: Poseidon/Rescue Prime (algebraic hash)
- **Trie**: zkTrie (Poseidon-based Merkle tree)
- **Proofs**: zk-STARK

## Modules

### `lib.rs` - Transaction Decoder
Decodes Polygon zkEVM transactions by reusing `decoder-ethereum` with chain ID validation.

**Usage:**
```rust
use decoder_polygon_zkevm::*;

let tx_bytes = hex::decode("f86c...")?;
let tx = PolygonZkevmDecoder::decode(&tx_bytes)?;
```

### `zktrie.rs` - Zero-Knowledge Trie
Utilities for analyzing zkTrie structures (Poseidon-based Merkle tree).

**Features:**
- zkTrie node types (Branch, Leaf, Empty)
- Poseidon hash computation using Goldilocks field
- Path utilities for trie traversal

**Usage:**
```rust
use decoder_polygon_zkevm::zktrie::*;

let leaf = ZkTrieNode::Leaf {
    key: ZkTrieHash::from_u64(123),
    value: ZkTrieHash::from_u64(456),
};

let hash = leaf.compute_hash();
```

## Dependencies

### Production
- `decoder-ethereum` - Transaction parsing (100% reuse)
- `decoder-crypto-zk` - Goldilocks field & Poseidon hash
- `decoder-primitives` - Core traits
- `universal-decoder-core` - TxIR types

### Development
- `decoder-test-utils` - Testing utilities
- `serde_json` - Test data parsing
- `proptest` - Property-based testing

## Testing

```bash
# Run all tests (14 tests)
cargo test -p decoder-polygon-zkevm

# Run only decoder tests (5 tests)
cargo test -p decoder-polygon-zkevm --lib '::tests::'

# Run only zkTrie tests (9 tests)
cargo test -p decoder-polygon-zkevm --lib 'zktrie::tests'
```

**Test Coverage:**
- ✅ Chain identity validation
- ✅ Chain ID enum (mainnet/testnet)
- ✅ Transaction format validation
- ✅ Type reuse verification
- ✅ zkTrie hash computation (5 scenarios)
- ✅ Path conversion utilities
- ✅ Collision resistance

## Comparison with decoder-polygon

Both decoders follow the same pattern:

| Feature | `decoder-polygon` | `decoder-polygon-zkevm` |
|---------|------------------|------------------------|
| Chain ID | 137 | 1101, 1442 |
| Transaction format | EVM (RLP) | EVM (RLP) |
| Reuses decoder-ethereum | ✅ Yes | ✅ Yes |
| Lines of code | ~130 | ~650 (includes zkTrie) |
| Additional features | None | zkTrie analysis |

## Reusability Opportunities Identified

### 1. **Direct Reuse** (~80%)
- RLP parsing functions (all 7 functions)
- Transaction type enum (TxType)
- EthereumTransaction struct
- Keccak256 hashing (Canonicalizer)
- Signature verification

### 2. **Adaptation** (~10%)
- Chain ID validation (changed from 1 to 1101/1442)
- Chain identity metadata

### 3. **New Implementation** (~10%)
- zkTrie module (Poseidon-based Merkle tree)
- Goldilocks field operations (already in decoder-crypto-zk)
- Poseidon hash (already in decoder-crypto-zk)

## Future Work

1. **Integration Tests**
   - Add real Polygon zkEVM transaction test vectors
   - Cross-validate with zkEVM node implementation

2. **Advanced zkTrie Features**
   - Full trie construction from state data
   - Merkle proof verification
   - State root computation

3. **Proof Verification**
   - Parse batch proofs from L1 contracts
   - Verify zk-STARK proofs (requires additional dependencies)

## References

- [Polygon zkEVM Documentation](https://docs.polygon.technology/zkevm/)
- [zkEVM Prover](https://github.com/0xPolygonHermez/zkevm-prover)
- [Goldilocks Field Paper](https://eprint.iacr.org/2022/1542.pdf)
- [Poseidon Hash](https://eprint.iacr.org/2019/458.pdf)
- [Rescue Prime](https://eprint.iacr.org/2020/1143.pdf)

## Contributing

This decoder demonstrates the project's core principle: **maximum reuse through trait-based architecture**. When adding new EVM-compatible chains, follow this pattern:

1. Reuse `decoder-ethereum` for transaction parsing
2. Add chain-specific validation (chain IDs, precompiles, etc.)
3. Document differences from base Ethereum
4. Add integration tests with real transactions

## License

Same as Universal Blockchain Decoder project.
