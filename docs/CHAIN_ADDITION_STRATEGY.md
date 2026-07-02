# Chain Addition Strategy: Template & DSL Design

## Executive Summary

This document provides a **battle-tested template** for adding new blockchain decoders to the Universal Blockchain Decoder, based on the complete Bitcoin implementation. It identifies reusable patterns, proposes a Domain-Specific Language (DSL) for decoder logic, and provides step-by-step guides for adding chains across different blockchain families.

**Status**: Based on completed work:
- ✅ Bitcoin decoder (UTXO model, 80+ tests, fuzzing, property tests)
- ✅ Ethereum decoder (Account model, RLP, EIP-2718)
- ✅ Solana decoder (Instruction model, compact-u16)
- ✅ Shared crates: `decoder-primitives`, `decoder-encodings`, `decoder-test-utils`

## Table of Contents

1. [Chain Addition Checklist](#chain-addition-checklist)
2. [Reusable Patterns from Bitcoin](#reusable-patterns-from-bitcoin)
3. [DSL Concept for Decoder Logic](#dsl-concept-for-decoder-logic)
4. [Chain Family Templates](#chain-family-templates)
5. [Code Reuse Matrix](#code-reuse-matrix)
6. [Testing Template](#testing-template)
7. [Example Walkthrough: Adding Dogecoin](#example-walkthrough-adding-dogecoin)

---

## Chain Addition Checklist

### Phase 1: Research & Planning (1-2 hours)

- [ ] **Identify chain family** (UTXO, Account, Instruction, Custom)
- [ ] **Transaction format specification** (where is the spec?)
- [ ] **Encoding schemes** (VarInt, RLP, Protobuf, custom?)
- [ ] **Serialization format** (binary, JSON, base64?)
- [ ] **Hashing algorithm** (SHA-256, Keccak-256, BLAKE2, etc.)
- [ ] **Signature schemes** (ECDSA, EdDSA, BLS, etc.)
- [ ] **Existing Rust libraries** (for test validation)
- [ ] **Test vector sources** (official test suites, block explorers)

### Phase 2: Crate Structure (30 minutes)

```bash
# 1. Create decoder crate
cd crates
cargo new decoder-<chain-name> --lib

# 2. Set up Cargo.toml
cat > decoder-<chain-name>/Cargo.toml <<'EOF'
[package]
name = "decoder-<chain-name>"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "<Chain Name> transaction decoder for universal-decoder"

[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
decoder-primitives = { path = "../decoder-primitives" }
decoder-encodings = { path = "../decoder-encodings" }  # If needed
serde = { workspace = true }
thiserror = { workspace = true }
# Add chain-specific crypto (sha2, sha3, blake2, etc.)

[dev-dependencies]
# Blockchain library for test validation ONLY
<chain-lib> = "x.y"
serde_json = { workspace = true }
proptest = { workspace = true }
decoder-test-utils = { path = "../decoder-test-utils" }
EOF

# 3. Add to workspace
# Edit /Cargo.toml to add "crates/decoder-<chain-name>" to members
```

### Phase 3: Core Implementation (4-8 hours)

- [ ] **Create module structure** (lib.rs, parsing.rs, types.rs)
- [ ] **Implement ChainIdentity** (chain ID, name, family)
- [ ] **Implement parsing functions** (pure Rust, no external libs)
- [ ] **Implement transaction type** (chain-specific representation)
- [ ] **Implement ChainDecoder trait**
- [ ] **Implement Canonicalizer trait** (TxIR mapping)
- [ ] **Implement TxHashable trait** (canonical hashing)

### Phase 4: Testing Infrastructure (4-6 hours)

- [ ] **Unit tests** (parsing functions, edge cases)
- [ ] **Property tests** (never panics, determinism, roundtrip)
- [ ] **Integration tests** (real transactions from chain)
- [ ] **Fuzzing setup** (fuzz/Cargo.toml, fuzz targets)
- [ ] **Validation tests** (compare against reference library)
- [ ] **Test fixtures** (tests/fixtures/ with hex + JSON)

### Phase 5: Documentation (1-2 hours)

- [ ] **Module documentation** (//! comments)
- [ ] **API examples** (in docs)
- [ ] **Testing guide** (how to run tests)
- [ ] **Known limitations** (unsupported features)
- [ ] **Reference links** (specs, explorers)

### Phase 6: CI/CD Integration (30 minutes)

- [ ] **Add to test.yml** (unit + integration tests)
- [ ] **Add to nightly.yml** (fuzzing)
- [ ] **Add to coverage.yml** (test coverage)
- [ ] **Verify all workflows pass**

**Total Time Estimate**: 10-20 hours for a new chain family, 4-8 hours for similar chain

---

## Reusable Patterns from Bitcoin

### 1. Module Structure (Universal Pattern)

```
crates/decoder-<chain-name>/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API, ChainDecoder impl
│   ├── parsing.rs       # Low-level parsers (VarInt, inputs, outputs)
│   └── types.rs         # Chain-specific transaction type
├── tests/
│   ├── integration_tests.rs      # Real transaction tests
│   ├── property_tests.rs         # Proptest-based properties
│   └── fixtures/                 # Test data (hex + JSON)
│       ├── README.md
│       ├── <chain>_genesis.hex
│       └── <chain>_genesis.json
└── fuzz/
    ├── Cargo.toml
    └── fuzz_targets/
        ├── fuzz_<chain>_decoder.rs
        └── fuzz_<chain>_varint.rs    # If custom encoding
```

### 2. Parsing Primitive Pattern (Reusable)

**Bitcoin Example** (`parsing.rs`):

```rust
use decoder_primitives::prelude::*;
use std::io::Read;

/// Constants (chain-specific)
pub const MAX_SCRIPT_SIZE: usize = 10_000;
pub const MAX_TRANSACTION_SIZE: usize = 100_000;

/// Parse a variable-length field
pub fn parse_input<R: Read>(reader: &mut R) -> Result<TxInput> {
    // 1. Read fixed-size fields
    let mut prev_hash = [0u8; 32];
    reader.read_exact(&mut prev_hash)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read prev_hash: {}", e)))?;

    let prev_index = read_u32_le(reader)?;

    // 2. Read variable-length field (with bounds check)
    let script_len = read_varint(reader)?;
    if script_len > MAX_SCRIPT_SIZE as u64 {
        return Err(DecoderError::invalid_structure(format!(
            "Script too large: {} bytes", script_len
        )));
    }

    let script_sig = read_bytes_bounded(reader, script_len as usize, MAX_SCRIPT_SIZE)?;

    // 3. Read remaining fields
    let sequence = read_u32_le(reader)?;

    Ok(TxInput { prev_hash, prev_index, script_sig, sequence })
}
```

**Reusable Pattern**:
1. Read fixed-size fields first (fail fast on truncation)
2. Read variable-length indicators (VarInt, length prefix)
3. Bounds-check before allocating
4. Use `read_bytes_bounded()` from decoder-primitives
5. Return structured error messages

### 3. Transaction Type Pattern (Universal)

```rust
use decoder_primitives::prelude::*;

/// Chain-specific transaction representation
#[derive(Debug, Clone)]
pub struct <Chain>Transaction {
    // Chain-specific fields - NO raw_bytes field!
    // Bytes must be reconstructed from parsed fields to ensure
    // the decoder actually parses the data (type-safe verification)
    pub version: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
    // NOTE: No raw_bytes field - use to_bytes() to reconstruct
}

impl <Chain>Transaction {
    /// Get transaction version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Reconstruct transaction bytes from parsed fields
    /// This is REQUIRED for the injective property: encode(decode(x)) == x
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.version.to_le_bytes());
        // ... serialize all fields back to bytes
        bytes
    }

    /// Calculate transaction ID (chain-specific hashing)
    pub fn txid(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        // Double SHA-256 for Bitcoin-like chains
        // Use reconstructed bytes, not stored raw_bytes
        let tx_bytes = self.to_bytes();
        let hash1 = Sha256::digest(&tx_bytes);
        let hash2 = Sha256::digest(hash1);
        hash2.to_vec()
    }

    /// Check if transaction is valid
    pub fn is_valid(&self) -> bool {
        !self.inputs.is_empty() && !self.outputs.is_empty()
    }
}

impl ReconstructableTransaction for <Chain>Transaction {
    fn reconstruct_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.to_bytes())
    }
}
```

### 4. ChainDecoder Implementation Pattern (Universal)

```rust
pub struct <Chain>Decoder;

impl ChainDecoder for <Chain>Decoder {
    type TxSpecific = <Chain>Transaction;
    type Chain = <Chain>Chain;

    fn chain() -> Self::Chain {
        <Chain>Chain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // 1. Validate format first (fast rejection)
        Self::validate_format(raw_bytes)?;

        // 2. Set up cursor
        let mut cursor = Cursor::new(raw_bytes);

        // 3. Parse version
        let version = read_u32_le(&mut cursor)?;

        // 4. Parse variable-count fields
        let input_count = read_varint(&mut cursor)?;
        if input_count > MAX_INPUTS as u64 {
            return Err(DecoderError::invalid_structure(format!(
                "Too many inputs: {}", input_count
            )));
        }

        let mut inputs = Vec::with_capacity(input_count as usize);
        for i in 0..input_count {
            inputs.push(parse_input(&mut cursor).map_err(|e| {
                DecoderError::chain_decoding(format!("Failed to parse input {}: {}", i, e))
            })?);
        }

        // 5. Verify all bytes consumed
        let consumed = cursor.position() as usize;
        if consumed != raw_bytes.len() {
            return Err(DecoderError::invalid_structure(format!(
                "Transaction has {} trailing bytes", raw_bytes.len() - consumed
            )));
        }

        Ok(<Chain>Transaction {
            version,
            inputs,
            locktime,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "<Chain> transaction cannot be empty"
            ));
        }

        if raw_bytes.len() < MIN_TX_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "<Chain> transaction too small: {} bytes (minimum {})",
                raw_bytes.len(), MIN_TX_SIZE
            )));
        }

        if raw_bytes.len() > MAX_TX_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "<Chain> transaction too large: {} bytes (maximum {})",
                raw_bytes.len(), MAX_TX_SIZE
            )));
        }

        Ok(())
    }
}
```

### 5. Canonicalizer Implementation Pattern (Universal)

```rust
impl<'a> Canonicalizer<'a> for <Chain>Transaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // 1. Build metadata
        let extra = format!(
            r#"{{"version":{},"locktime":{}}}"#,
            self.version, self.locktime
        );

        let metadata = TxMetadata {
            tx_hash: self.txid(),
            block_height: None,
            timestamp: Some(self.locktime as u64),
            size: self.raw_bytes.len(),
            extra,
        };

        // 2. Build authorization (signatures)
        let mut signatures = Vec::new();
        for (idx, input) in self.inputs.iter().enumerate() {
            if !input.script_sig.is_empty() {
                signatures.push(Signature {
                    data: input.script_sig.clone(),
                    key_index: idx,
                    metadata: Some(format!(r#"{{"input_index":{}}}"#, idx)),
                });
            }
        }

        let authorization = AuthorizationPackage {
            signatures,
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // 3. Build operations (transfers)
        let operations = self.outputs.iter().map(|output| {
            Operation::Transfer(Transfer {
                from: Address { bytes: vec![], human_readable: None },
                to: Address { bytes: output.script_pubkey.clone(), human_readable: None },
                amount: Amount { value: output.value as u128, decimals: 8 },
                asset: AssetId::Native,
            })
        }).collect();

        // 4. Build state deltas
        let inputs = self.inputs.iter().map(|input| InputReference {
            prev_tx: input.prev_hash.to_vec(),
            output_index: input.prev_index,
            value: Amount { value: 0, decimals: 8 },
            script: input.script_sig.clone(),
        }).collect();

        let outputs = self.outputs.iter().enumerate().map(|(idx, output)| OutputValue {
            index: idx as u32,
            address: Address { bytes: output.script_pubkey.clone(), human_readable: None },
            value: Amount { value: output.value as u128, decimals: 8 },
            script: output.script_pubkey.clone(),
        }).collect();

        let state_deltas = StateDeltas { inputs, outputs, account_changes: vec![] };

        Ok(TxIR::new(&<Chain>Chain, metadata, authorization, operations, state_deltas))
    }

    fn validate(&self) -> Result<()> {
        if self.inputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Transaction must have at least one input"
            ));
        }
        if self.outputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Transaction must have at least one output"
            ));
        }
        Ok(())
    }
}
```

### 6. Testing Pattern (Universal)

**Unit Tests** (`lib.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use universal_decoder_core::hex;

    #[test]
    fn test_validate_format() {
        assert!(BitcoinDecoder::validate_format(&[]).is_err());
        assert!(BitcoinDecoder::validate_format(&[0x01]).is_err());
        let dummy_tx = vec![0u8; 100];
        assert!(BitcoinDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_decode_minimal_transaction() {
        let mut tx_bytes = vec![];
        // ... construct minimal valid transaction
        let decoded = <Chain>Decoder::decode(&tx_bytes).expect("decode failed");
        assert_eq!(decoded.version(), 1);
    }
}
```

**Property Tests** (`tests/property_tests.rs`):
```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<BitcoinDecoder>(&bytes);
    }

    #[test]
    fn prop_txid_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = BitcoinDecoder::decode(&bytes) {
            let txid1 = tx.txid();
            let txid2 = tx.txid();
            prop_assert_eq!(txid1, txid2);
        }
    }
}
```

**Fuzzing** (`fuzz/fuzz_targets/fuzz_<chain>_decoder.rs`):
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = <Chain>Decoder::decode(data);

    if let Ok(tx) = <Chain>Decoder::decode(data) {
        let _ = tx.canonicalize();
        let _ = tx.txid();
        let _ = tx.validate();
    }
});
```

---

## DSL Concept for Decoder Logic

### Vision: Declarative Transaction Parsing

Once we have 5-10 chains implemented, we can extract common patterns into a **declarative DSL** that generates decoder logic. This reduces boilerplate and makes adding new chains trivial.

### DSL Example: Bitcoin Transaction

```rust
// Future: crates/decoder-bitcoin/spec.rs

use decoder_dsl::prelude::*;

transaction_decoder! {
    chain: Bitcoin,
    chain_id: 0,
    chain_family: Utxo,
    endianness: LittleEndian,
    hash: DoubleSha256,

    format: {
        version: u32,

        // Conditional field based on marker
        if [marker == 0x00, flag == 0x01] {
            segwit: true,
        }

        inputs: varint_array {
            prev_hash: bytes(32),
            prev_index: u32,
            script_sig: varint_bytes(max: 10_000),
            sequence: u32,
        },

        outputs: varint_array {
            value: u64,
            script_pubkey: varint_bytes(max: 10_000),
        },

        if segwit {
            witnesses: array(inputs.len()) {
                items: varint_array {
                    item: varint_bytes(max: 10_000),
                }
            }
        },

        locktime: u32,
    },

    txid: {
        // SegWit: hash without witness data
        serialize_without: if segwit { [witnesses] } else { [] },
        hash: DoubleSha256,
    },

    canonical_ir: {
        metadata: {
            tx_hash: self.txid(),
            timestamp: Some(self.locktime as u64),
            extra: json!({ "version": self.version, "is_segwit": self.segwit }),
        },

        authorization: {
            signatures: [
                ...self.inputs.map(|i| Signature {
                    data: i.script_sig,
                    key_index: i.index,
                }),
                ...self.witnesses.flat_map(|w| w.items.map(|item| Signature {
                    data: item,
                    key_index: w.index,
                })),
            ],
            signature_scheme: SignatureScheme::Ecdsa,
        },

        operations: self.outputs.map(|o| Operation::Transfer {
            to: Address { bytes: o.script_pubkey },
            amount: Amount { value: o.value, decimals: 8 },
            asset: AssetId::Native,
        }),

        state_deltas: {
            inputs: self.inputs.map(|i| InputReference {
                prev_tx: i.prev_hash,
                output_index: i.prev_index,
                script: i.script_sig,
            }),
            outputs: self.outputs.map(|o| OutputValue {
                address: Address { bytes: o.script_pubkey },
                value: Amount { value: o.value, decimals: 8 },
                script: o.script_pubkey,
            }),
        },
    },
}
```

### DSL Benefits

1. **Declarative**: Describe transaction format, not parsing logic
2. **Type-safe**: Compiler checks field types and constraints
3. **Maintainable**: Change format in one place
4. **Testable**: Auto-generate property tests from spec
5. **Documented**: Spec IS the documentation
6. **Provable**: Easier to formally verify generated code

### DSL Implementation Strategy

**Phase 1** (Now): Collect patterns from 5+ chains
- Bitcoin (UTXO, VarInt, SegWit complexity)
- Ethereum (Account, RLP, EIP-2718 types)
- Solana (Instruction, compact-u16)
- Cosmos (Protobuf, bech32)
- Polkadot (SCALE encoding)

**Phase 2** (After 10 chains): Extract common abstractions
- Encoding primitives (VarInt, RLP, Protobuf, SCALE)
- Field types (fixed bytes, arrays, conditional fields)
- Hashing strategies (single, double, prefix)
- Address formats (raw, base58, bech32, hex)

**Phase 3** (After 15 chains): Design DSL syntax
- Proc macro: `#[transaction_decoder(...)]`
- Parse DSL at compile-time
- Generate parsing.rs, types.rs, decoder impl
- Generate property tests automatically

**Phase 4** (After 20 chains): Production-ready DSL
- Comprehensive examples for all chain families
- Editor support (syntax highlighting, completion)
- Error messages guide users to correct syntax
- DSL becomes the primary way to add chains

### DSL Code Generation Example

```rust
// User writes this:
transaction_decoder! {
    chain: Litecoin,
    chain_id: 2,
    chain_family: Utxo,
    extends: Bitcoin,  // Reuse Bitcoin parsing

    differences: {
        hash: Scrypt,  // Different hashing
        // All other fields same as Bitcoin
    }
}

// DSL expands to ~500 LOC:
// - LitecoinChain struct
// - LitecoinTransaction type
// - parsing functions
// - ChainDecoder impl
// - Canonicalizer impl
// - TxHashable impl
// - Unit tests
// - Property tests
// - Fuzz targets
```

---

## Chain Family Templates

### Template 1: UTXO-Based Chains (Bitcoin-like)

**Examples**: Bitcoin, Litecoin, Dogecoin, Bitcoin Cash, Zcash

**Characteristics**:
- Inputs reference previous outputs (UTXO model)
- Outputs create new UTXOs
- Scripts for locking/unlocking
- Transaction size in bytes (not gas)

**Shared Code**:
```rust
// All UTXO chains use:
- decoder-encodings::varint  (VarInt encoding)
- decoder-primitives::read_*_le  (little-endian)
- Common TxInput/TxOutput structure
- Similar canonicalization (inputs → outputs mapping)
```

**Differences**:
- Hashing: SHA-256 (Bitcoin), Scrypt (Litecoin), X11 (Dash)
- Address encoding: base58 (Bitcoin), bech32 (SegWit), cashaddr (BCH)
- Script opcodes: mostly same, some additions
- SegWit support: Bitcoin/Litecoin yes, Dogecoin no

**Time to Add Similar Chain**: ~4 hours (80% code reuse)

### Template 2: Account-Based Chains (Ethereum-like)

**Examples**: Ethereum, BSC, Polygon, Avalanche C-Chain, Fantom

**Characteristics**:
- Account model (balances stored in state)
- Nonce for replay protection
- Gas limit and gas price
- RLP encoding
- EIP-2718 transaction types (legacy, EIP-2930, EIP-1559)

**Shared Code**:
```rust
// All EVM chains use:
- decoder-encodings::rlp  (RLP encoding)
- decoder-primitives::read_*_be  (big-endian)
- EIP-2718 envelope parsing
- Common transaction structure (to, value, data, gas)
```

**Differences**:
- Chain ID (for EIP-155 protection)
- Gas cost schedule (usually same as Ethereum)
- Precompiles (some chains add custom precompiles)
- Hashing: Keccak-256 (universal for EVM)

**Time to Add EVM Chain**: ~2 hours (95% code reuse)
**Note**: Most EVM chains can share a single decoder with chain ID configuration

### Template 3: Instruction-Based Chains (Solana-like)

**Examples**: Solana, NEAR (different encoding)

**Characteristics**:
- Instructions instead of method calls
- Program IDs identify smart contracts
- Accounts array (read/write permissions)
- Compact encodings for size optimization
- Blockhash for recent block reference

**Shared Code**:
```rust
// Solana-like chains use:
- decoder-encodings::compact_u16  (Solana's compact encoding)
- Instruction model (program_id, accounts, data)
- Similar state change model
```

**Differences**:
- Encoding: Solana (compact-u16), NEAR (Borsh native)
- Hashing: SHA-256 (Solana), SHA-256 (NEAR)
- Account model details

**Time to Add Instruction-Based Chain**: ~8 hours (60% code reuse)

### Template 4: Cosmos SDK Chains (Tendermint-based)

**Examples**: Cosmos Hub, Osmosis, Juno, Secret Network, Terra

**Characteristics**:
- Protobuf encoding
- Message-based (Msg types)
- Cosmos SDK modules (bank, staking, gov, etc.)
- Bech32 addresses
- Amino encoding (legacy) or Protobuf (modern)

**Shared Code**:
```rust
// Cosmos chains share:
- Protobuf decoding (if we add decoder-encodings::protobuf)
- Bech32 address encoding
- Standard Msg types (MsgSend, MsgDelegate, etc.)
- Similar canonicalization
```

**Differences**:
- Chain-specific modules
- Bech32 prefix (cosmos, osmo, juno, etc.)
- Custom Msg types

**Time to Add Cosmos Chain**: ~6 hours (70% code reuse after first chain)

### Template 5: Substrate-Based Chains (Polkadot-like)

**Examples**: Polkadot, Kusama, Moonbeam, Acala, Astar

**Characteristics**:
- SCALE encoding
- Extrinsics (signed or unsigned)
- Pallet-based architecture
- SS58 address format
- Runtime metadata (dynamic)

**Shared Code**:
```rust
// Substrate chains share:
- decoder-encodings::scale  (SCALE encoding, to be added)
- Extrinsic structure
- SS58 address decoding
- Common pallets (balances, staking, etc.)
```

**Differences**:
- Runtime configuration
- Custom pallets
- SS58 prefix
- Weights/fees

**Time to Add Substrate Chain**: ~8 hours (60% code reuse after first chain)

---

## Code Reuse Matrix

| Component | Bitcoin | Ethereum | Solana | Cosmos | Polkadot | Litecoin | Dogecoin |
|-----------|---------|----------|--------|--------|----------|----------|----------|
| **Parsing Primitives** | | | | | | | |
| `read_u8/u16/u32/u64` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `read_*_le` | ✅ | | ✅ | | | ✅ | ✅ |
| `read_*_be` | | ✅ | | ✅ | | | |
| `read_bytes_bounded` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Encodings** | | | | | | | |
| VarInt | ✅ | | | | | ✅ | ✅ |
| RLP | | ✅ | | | | | |
| Compact-u16 | | | ✅ | | | | |
| Protobuf | | | | ✅ | | | |
| SCALE | | | | | ✅ | | |
| **Hashing** | | | | | | | |
| SHA-256 | ✅ | | ✅ | | | | |
| Double SHA-256 | ✅ | | | | | ✅ | ✅ |
| Keccak-256 | | ✅ | | | | | |
| BLAKE2 | | | | | ✅ | | |
| Scrypt | | | | | | ✅ | |
| **Address Formats** | | | | | | | |
| Base58 | ✅ | | ✅ | | | ✅ | ✅ |
| Hex (0x...) | | ✅ | | | | | |
| Bech32 | | | | ✅ | | | |
| SS58 | | | | | ✅ | | |
| **Testing Utilities** | | | | | | | |
| `arb_small_bytes()` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `prop_decoder_never_panics()` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Fixture structure | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Reuse Percentages** (estimated):
- **Litecoin** (UTXO sibling): ~90% of Bitcoin code
- **Dogecoin** (UTXO sibling): ~85% of Bitcoin code
- **EVM chains** (Ethereum siblings): ~95% of Ethereum code
- **Cosmos chains** (first one): ~40% new, 60% reusable patterns
- **Substrate chains** (first one): ~50% new, 50% reusable patterns

---

## Testing Template

**NEW**: Ready-to-use test templates are available in `docs/templates/`:
- ✅ `PROPERTY_TEST_TEMPLATE.rs` - 8 property tests ready to copy-paste (150 lines)
- ✅ `INTEGRATION_TEST_TEMPLATE.rs` - Fixture-based testing patterns (200 lines)
- ✅ Usage: Copy templates, replace `{{CHAIN}}` with your chain name, customize
- ✅ See: `CLAUDE_PROPOSED.md` "Testing Quick Start" section for detailed instructions

**Recommended workflow**: Use templates first (10 min), then customize based on patterns below.

### 1. Unit Tests (Required for ALL chains)

```rust
// crates/decoder-<chain>/src/lib.rs

#[cfg(test)]
mod tests {
    use super::*;
    use universal_decoder_core::hex;

    // Test 1: Format validation
    #[test]
    fn test_validate_format_empty() {
        assert!(<Chain>Decoder::validate_format(&[]).is_err());
    }

    #[test]
    fn test_validate_format_too_small() {
        assert!(<Chain>Decoder::validate_format(&[0x01]).is_err());
    }

    #[test]
    fn test_validate_format_too_large() {
        let huge = vec![0u8; MAX_TX_SIZE + 1];
        assert!(<Chain>Decoder::validate_format(&huge).is_err());
    }

    // Test 2: Chain identity
    #[test]
    fn test_chain_identity() {
        let chain = <Chain>Decoder::chain();
        assert_eq!(chain.chain_id(), EXPECTED_ID);
        assert_eq!(chain.chain_name(), "Expected Name");
        assert_eq!(chain.chain_family(), ChainFamily::Expected);
    }

    // Test 3: Minimal transaction
    #[test]
    fn test_decode_minimal_transaction() {
        let tx_bytes = construct_minimal_tx();
        let decoded = <Chain>Decoder::decode(&tx_bytes)
            .expect("Failed to decode minimal transaction");
        assert_eq!(decoded.version(), 1);
    }

    // Test 4: Invalid transactions
    #[test]
    fn test_decode_truncated() {
        let truncated = vec![0x01, 0x00, 0x00];
        assert!(<Chain>Decoder::decode(&truncated).is_err());
    }

    // Test 5: Transaction properties
    #[test]
    fn test_txid_deterministic() {
        let tx = create_test_tx();
        assert_eq!(tx.txid(), tx.txid());
    }
}
```

### 2. Property Tests (Required for ALL chains)

```rust
// crates/decoder-<chain>/tests/property_tests.rs

use proptest::prelude::*;
use decoder_test_utils::proptest_helpers::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    // Property 1: Decoder never panics
    #[test]
    fn prop_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<<Chain>Decoder>(&bytes);
    }

    // Property 2: TXID is deterministic
    #[test]
    fn prop_txid_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = <Chain>Decoder::decode(&bytes) {
            let txid1 = tx.txid();
            let txid2 = tx.txid();
            prop_assert_eq!(txid1, txid2);
        }
    }

    // Property 3: Canonicalization is deterministic
    #[test]
    fn prop_canonical_hash_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = <Chain>Decoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                let hash1 = tx_ir.canonical_hash();
                let hash2 = tx_ir.canonical_hash();
                match (hash1, hash2) {
                    (Ok(h1), Ok(h2)) => prop_assert_eq!(h1, h2),
                    (Err(_), Err(_)) => {},
                    _ => return Err(TestCaseError::fail("Non-deterministic error")),
                }
            }
        }
    }

    // Property 4: Decode-canonicalize pipeline never panics
    #[test]
    fn prop_full_pipeline_never_panics(bytes in arb_small_bytes()) {
        use std::panic;
        let result = panic::catch_unwind(|| {
            if let Ok(tx) = <Chain>Decoder::decode(&bytes) {
                if let Ok(tx_ir) = tx.canonicalize() {
                    let _ = tx_ir.canonical_hash();
                }
            }
        });
        prop_assert!(result.is_ok());
    }
}
```

### 3. Integration Tests (Required for ALL chains)

```rust
// crates/decoder-<chain>/tests/integration_tests.rs

use decoder_<chain>::*;
use universal_decoder_core::prelude::*;

// Test against real transaction from blockchain
#[test]
fn test_real_transaction_genesis() {
    let hex = include_str!("fixtures/<chain>_genesis.hex").trim();
    let tx_bytes = universal_decoder_core::hex::decode(hex).unwrap();

    let tx = <Chain>Decoder::decode(&tx_bytes)
        .expect("Failed to decode genesis transaction");

    // Verify known properties
    assert_eq!(tx.version(), 1);
    assert_eq!(tx.input_count(), 1);
    assert_eq!(tx.output_count(), 1);

    // Verify TXID matches known value
    let expected_txid = universal_decoder_core::hex::decode(
        "<expected-txid-hex>"
    ).unwrap();
    assert_eq!(tx.txid(), expected_txid);
}

// Validate against reference library (dev-dependency)
#[test]
fn test_validation_against_reference_lib() {
    use <chain_lib>::Transaction;  // Reference library

    let hex = include_str!("fixtures/<chain>_genesis.hex").trim();
    let tx_bytes = universal_decoder_core::hex::decode(hex).unwrap();

    // Parse with our decoder
    let our_tx = <Chain>Decoder::decode(&tx_bytes).unwrap();

    // Parse with reference library
    let ref_tx: Transaction = <chain_lib>::deserialize(&tx_bytes).unwrap();

    // Compare results
    assert_eq!(our_tx.txid(), ref_tx.txid().as_ref());
    assert_eq!(our_tx.version(), ref_tx.version);
    assert_eq!(our_tx.input_count(), ref_tx.input.len());
    assert_eq!(our_tx.output_count(), ref_tx.output.len());
}
```

### 4. Fuzzing Setup (Required for ALL chains)

```bash
# Create fuzzing infrastructure
mkdir -p crates/decoder-<chain>/fuzz/fuzz_targets

# Create fuzz/Cargo.toml
cat > crates/decoder-<chain>/fuzz/Cargo.toml <<'EOF'
[package]
name = "decoder-<chain>-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"

[dependencies.decoder-<chain>]
path = ".."

[dependencies.universal-decoder-core]
path = "../../universal-decoder-core"

[workspace]
members = ["."]

[[bin]]
name = "fuzz_<chain>_decoder"
path = "fuzz_targets/fuzz_<chain>_decoder.rs"
test = false
doc = false
EOF
```

```rust
// fuzz/fuzz_targets/fuzz_<chain>_decoder.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use decoder_<chain>::<Chain>Decoder;
use universal_decoder_core::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Test 1: Decode should never panic
    let _ = <Chain>Decoder::decode(data);

    // Test 2: Validate format should never panic
    let _ = <Chain>Decoder::validate_format(data);

    // Test 3: If decode succeeds, canonicalization should not panic
    if let Ok(tx) = <Chain>Decoder::decode(data) {
        let _ = tx.canonicalize();
        let _ = tx.txid();
        let _ = tx.validate();

        if let Ok(tx_ir) = tx.canonicalize() {
            let _ = tx_ir.canonical_hash();
            let _ = tx_ir.to_canonical_bytes();
        }
    }

    // Test 4: Very large inputs should be rejected gracefully
    if data.len() > 1_000_000 {
        assert!(<Chain>Decoder::decode(data).is_err());
    }
});
```

**Run fuzzing**:
```bash
# Install cargo-fuzz (first time only)
cargo install cargo-fuzz

# Run fuzzing (continuously finds edge cases)
cd crates/decoder-<chain>
cargo +nightly fuzz run fuzz_<chain>_decoder

# Run for specific duration
cargo +nightly fuzz run fuzz_<chain>_decoder -- -max_total_time=300  # 5 minutes
```

### 5. Test Fixtures Structure

```
crates/decoder-<chain>/tests/fixtures/
├── README.md                     # Source of test data
├── <chain>_genesis.hex           # Genesis transaction (hex)
├── <chain>_genesis.json          # Genesis transaction (metadata)
├── <chain>_simple_transfer.hex   # Simple transfer
├── <chain>_simple_transfer.json
├── <chain>_complex.hex           # Complex transaction
├── <chain>_complex.json
└── <chain>_invalid_*.hex         # Invalid transactions (for error testing)
```

**README.md Template**:
```markdown
# <Chain Name> Test Fixtures

## Sources

- Genesis transaction: Block #0, TX #0
  - Explorer: https://<chain-explorer>/tx/<txid>
  - Block height: 0
  - Timestamp: <timestamp>

- Simple transfer: Block #<height>, TX #<index>
  - Explorer: https://<chain-explorer>/tx/<txid>
  - Type: Simple transfer
  - Amount: <amount> <currency>

## Validation

All fixtures validated against:
- Official <chain> implementation: v<version>
- Block explorer: <explorer-name>
- Test vector source: <source>

## Format

- `.hex` files: Raw transaction bytes in hexadecimal (no 0x prefix)
- `.json` files: Transaction metadata for validation

JSON schema:
```json
{
  "txid": "<expected transaction ID>",
  "version": <version>,
  "inputs": <count>,
  "outputs": <count>,
  "size": <bytes>,
  "notes": "<any special properties>"
}
```
```

---

## Example Walkthrough: Adding Dogecoin

### Step 1: Research (30 minutes)

```bash
# Dogecoin is Bitcoin-like (UTXO model)
Chain family: UTXO
Encoding: VarInt (same as Bitcoin)
Endianness: Little-endian (same as Bitcoin)
Hashing: Double SHA-256 (same as Bitcoin!)
Address: Base58 (different prefix)
Differences:
  - No SegWit support (simpler than Bitcoin)
  - Different magic bytes
  - Block time: 1 minute (vs Bitcoin's 10 minutes)
  - Different address prefixes (D... for mainnet)

# Reference implementation: dogecoin/dogecoin (C++)
# Rust library: dogecoin-rs (for test validation)
```

### Step 2: Create Crate (5 minutes)

```bash
cd crates
cargo new decoder-dogecoin --lib

# Copy Bitcoin structure
cp -r decoder-bitcoin/src/* decoder-dogecoin/src/
cp -r decoder-bitcoin/tests decoder-dogecoin/
cp -r decoder-bitcoin/fuzz decoder-dogecoin/

# Update Cargo.toml
cat > decoder-dogecoin/Cargo.toml <<'EOF'
[package]
name = "decoder-dogecoin"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Dogecoin transaction decoder for universal-decoder"

[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
decoder-primitives = { path = "../decoder-primitives" }
decoder-encodings = { path = "../decoder-encodings" }
serde = { workspace = true }
thiserror = { workspace = true }
sha2 = { workspace = true }  # Same hashing as Bitcoin

[dev-dependencies]
# No dogecoin-rs crate available, use bitcoin crate for validation
bitcoin = "0.31"
serde_json = { workspace = true }
proptest = { workspace = true }
decoder-test-utils = { path = "../decoder-test-utils" }
EOF

# Add to workspace
echo '  "crates/decoder-dogecoin",' >> ../Cargo.toml
```

### Step 3: Adapt Bitcoin Code (1 hour)

```rust
// crates/decoder-dogecoin/src/lib.rs

//! Dogecoin transaction decoder - Pure Rust implementation
//!
//! Dogecoin is a Bitcoin-like chain with the following differences:
//! - No SegWit support (simpler transaction format)
//! - Different address prefixes
//! - 1-minute block time
//! - Scrypt mining (PoW only, doesn't affect transactions)
//!
//! ## Transaction Format
//!
//! Dogecoin uses the same transaction format as Bitcoin pre-SegWit:
//! - Version (4 bytes, little-endian)
//! - Input count (VarInt)
//! - Inputs (prev_hash, prev_index, script_sig, sequence)
//! - Output count (VarInt)
//! - Outputs (value, script_pubkey)
//! - Locktime (4 bytes, little-endian)

use decoder_primitives::prelude::*;
use std::io::Cursor;

pub mod parsing;
pub mod types;

use parsing::*;
pub use types::DogecoinTransaction;

/// Dogecoin chain identity
#[derive(Debug, Clone, Copy)]
pub struct DogecoinChain;

impl ChainIdentity for DogecoinChain {
    fn chain_id(&self) -> u64 {
        3  // Dogecoin chain ID (unofficial)
    }

    fn chain_name(&self) -> &str {
        "Dogecoin"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

/// Dogecoin decoder implementing the ChainDecoder trait
pub struct DogecoinDecoder;

impl ChainDecoder for DogecoinDecoder {
    type TxSpecific = DogecoinTransaction;
    type Chain = DogecoinChain;

    fn chain() -> Self::Chain {
        DogecoinChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        let mut cursor = Cursor::new(raw_bytes);

        // Parse version
        let version = read_u32_le(&mut cursor)?;

        // Note: Dogecoin does NOT support SegWit, so no marker/flag check

        // Parse inputs (same as Bitcoin)
        let input_count = read_varint(&mut cursor)?;
        if input_count > MAX_INPUTS_OUTPUTS as u64 {
            return Err(DecoderError::invalid_structure(format!(
                "Too many inputs: {}", input_count
            )));
        }

        let mut inputs = Vec::with_capacity(input_count as usize);
        for i in 0..input_count {
            inputs.push(parse_input(&mut cursor).map_err(|e| {
                DecoderError::chain_decoding(format!("Failed to parse input {}: {}", i, e))
            })?);
        }

        // Parse outputs (same as Bitcoin)
        let output_count = read_varint(&mut cursor)?;
        if output_count > MAX_INPUTS_OUTPUTS as u64 {
            return Err(DecoderError::invalid_structure(format!(
                "Too many outputs: {}", output_count
            )));
        }

        let mut outputs = Vec::with_capacity(output_count as usize);
        for i in 0..output_count {
            outputs.push(parse_output(&mut cursor).map_err(|e| {
                DecoderError::chain_decoding(format!("Failed to parse output {}: {}", i, e))
            })?);
        }

        // Parse locktime
        let locktime = read_u32_le(&mut cursor)?;

        // Verify all bytes consumed
        let consumed = cursor.position() as usize;
        if consumed != raw_bytes.len() {
            return Err(DecoderError::invalid_structure(format!(
                "Transaction has {} trailing bytes", raw_bytes.len() - consumed
            )));
        }

        Ok(DogecoinTransaction {
            version,
            inputs,
            outputs,
            locktime,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Dogecoin transaction cannot be empty"
            ));
        }

        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(format!(
                "Dogecoin transaction too small: {} bytes (minimum 10)",
                raw_bytes.len()
            )));
        }

        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Dogecoin transaction too large: {} bytes (maximum {})",
                raw_bytes.len(), MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use universal_decoder_core::hex;

    #[test]
    fn test_chain() {
        let chain = DogecoinDecoder::chain();
        assert_eq!(chain.chain_id(), 3);
        assert_eq!(chain.chain_name(), "Dogecoin");
        assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    }

    #[test]
    fn test_decode_minimal_transaction() {
        // Construct minimal Dogecoin transaction (no SegWit, same as Bitcoin legacy)
        let mut tx_bytes = vec![];
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());  // version
        tx_bytes.push(0x01);  // 1 input
        tx_bytes.extend_from_slice(&[0u8; 32]);  // prev_hash
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());  // prev_index
        tx_bytes.push(0x00);  // script_sig length
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());  // sequence
        tx_bytes.push(0x01);  // 1 output
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes());  // value
        tx_bytes.push(0x00);  // script_pubkey length
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());  // locktime

        let decoded = DogecoinDecoder::decode(&tx_bytes)
            .expect("Failed to decode minimal Dogecoin transaction");

        assert_eq!(decoded.version(), 1);
        assert_eq!(decoded.input_count(), 1);
        assert_eq!(decoded.output_count(), 1);
        assert!(!decoded.is_segwit());  // Dogecoin does not support SegWit
    }
}
```

### Step 4: Update Types (15 minutes)

```rust
// crates/decoder-dogecoin/src/types.rs

// Almost identical to Bitcoin, just remove SegWit fields

use crate::parsing::{TxInput, TxOutput};
use crate::DogecoinChain;
use decoder_encodings::varint::encode_varint;
use universal_decoder_core::prelude::*;

#[derive(Debug, Clone)]
pub struct DogecoinTransaction {
    pub version: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
    // NOTE: No raw_bytes field - bytes must be reconstructed from fields
}

impl DogecoinTransaction {
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1
            && self.inputs[0].prev_hash == [0u8; 32]
            && self.inputs[0].prev_index == 0xFFFFFFFF
    }

    pub fn is_segwit(&self) -> bool {
        false  // Dogecoin does not support SegWit
    }

    /// Reconstruct transaction bytes from parsed fields
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.version.to_le_bytes());
        bytes.extend(encode_varint(self.inputs.len() as u64));
        for input in &self.inputs {
            bytes.extend(input.serialize());
        }
        bytes.extend(encode_varint(self.outputs.len() as u64));
        for output in &self.outputs {
            bytes.extend(output.serialize());
        }
        bytes.extend(&self.locktime.to_le_bytes());
        bytes
    }

    pub fn txid(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        // Same as Bitcoin: double SHA-256 of reconstructed bytes
        let tx_bytes = self.to_bytes();
        let hash1 = Sha256::digest(&tx_bytes);
        let hash2 = Sha256::digest(hash1);
        hash2.to_vec()
    }

    pub fn total_output_value(&self) -> Result<u64> {
        self.outputs.iter().try_fold(0u64, |acc, output| {
            acc.checked_add(output.value)
                .ok_or_else(|| DecoderError::invalid_structure("Output value overflow"))
        })
    }
}

impl<'a> Canonicalizer<'a> for DogecoinTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Same as Bitcoin, just mark is_segwit: false
        let extra = format!(
            r#"{{"version":{},"lock_time":{},"is_coinbase":{}}}"#,
            self.version, self.locktime, self.is_coinbase()
        );

        let metadata = TxMetadata {
            tx_hash: self.txid(),
            block_height: None,
            timestamp: Some(self.locktime as u64),
            size: self.raw_bytes.len(),
            extra,
        };

        // ... (rest same as Bitcoin)

        Ok(TxIR::new(&DogecoinChain, metadata, authorization, operations, state_deltas))
    }

    fn validate(&self) -> Result<()> {
        if self.inputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Dogecoin transaction must have at least one input"
            ));
        }
        if self.outputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Dogecoin transaction must have at least one output"
            ));
        }
        self.total_output_value()?;
        Ok(())
    }
}

impl TxHashable for DogecoinTransaction {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.raw_bytes.clone()
    }

    fn compute_hash(&self) -> Vec<u8> {
        self.compute_hash_with::<DoubleSha256>()
    }
}
```

### Step 5: Copy Parsing (5 minutes)

```bash
# Dogecoin parsing is IDENTICAL to Bitcoin (no SegWit)
# Just copy Bitcoin's parsing.rs
cp ../decoder-bitcoin/src/parsing.rs src/parsing.rs

# Update module comment
sed -i 's/Bitcoin/Dogecoin/g' src/parsing.rs
```

### Step 6: Update Tests (30 minutes)

```bash
# Copy test structure from Bitcoin
cp -r ../decoder-bitcoin/tests/property_tests.rs tests/
cp -r ../decoder-bitcoin/tests/integration_tests.rs tests/
cp -r ../decoder-bitcoin/fuzz fuzz/

# Update all references
find tests fuzz -type f -name "*.rs" -exec sed -i 's/Bitcoin/Dogecoin/g' {} \;
find tests fuzz -type f -name "*.rs" -exec sed -i 's/bitcoin/dogecoin/g' {} \;
```

### Step 7: Add Test Fixtures (1 hour)

```bash
mkdir -p tests/fixtures

# Find Dogecoin genesis transaction
# Source: https://dogechain.info/tx/<genesis-txid>
# (Download from block explorer or Dogecoin node)

cat > tests/fixtures/doge_genesis_coinbase.hex <<'EOF'
01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff1e04...
EOF

cat > tests/fixtures/doge_genesis_coinbase.json <<'EOF'
{
  "txid": "...",
  "version": 1,
  "inputs": 1,
  "outputs": 1,
  "is_coinbase": true,
  "block_height": 0
}
EOF

# Add more fixtures (simple transfer, multi-input, etc.)
```

### Step 8: Run Tests (10 minutes)

```bash
cd crates/decoder-dogecoin

# Run all tests
cargo test --all

# Run property tests (1000 cases)
cargo test prop_ -- --nocapture

# Run fuzzing (5 minutes)
cargo +nightly fuzz run fuzz_dogecoin_decoder -- -max_total_time=300

# Check coverage
cargo llvm-cov --html
# Open target/llvm-cov/html/index.html
```

### Step 9: Documentation (30 minutes)

```rust
//! # Dogecoin Decoder
//!
//! Pure Rust decoder for Dogecoin transactions.
//!
//! ## Features
//!
//! - ✅ Legacy transactions (Bitcoin-compatible)
//! - ❌ SegWit (not supported by Dogecoin)
//! - ✅ Coinbase transactions
//! - ✅ P2PKH, P2SH scripts
//!
//! ## Example
//!
//! ```rust
//! use decoder_dogecoin::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "01000000...";
//! let tx_bytes = universal_decoder_core::hex::decode(tx_hex)?;
//!
//! let decoded = DogecoinDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! let txid = decoded.txid();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Differences from Bitcoin
//!
//! - **No SegWit**: Dogecoin uses only legacy transaction format
//! - **Block time**: 1 minute (vs Bitcoin's 10 minutes)
//! - **Address prefix**: D... for mainnet, n... for testnet
//! - **PoW**: Scrypt instead of SHA-256 (doesn't affect transaction parsing)
//!
//! ## References
//!
//! - Dogecoin Core: https://github.com/dogecoin/dogecoin
//! - Block Explorer: https://dogechain.info/
//! - Specifications: (same as Bitcoin pre-SegWit)
```

### Step 10: CI/CD (5 minutes)

```yaml
# Already covered by workspace-level CI!
# .github/workflows/test.yml runs:
#   cargo test --all
#   cargo clippy --all
#   cargo fmt --all --check

# Just verify it works:
git add .
git commit -m "Add Dogecoin decoder (Bitcoin-compatible, no SegWit)"
git push

# Check GitHub Actions
# https://github.com/<user>/<repo>/actions
```

### Total Time: ~4 hours

**Breakdown**:
- Research: 30 min
- Setup: 5 min
- Implementation: 1h 15min (mostly copy-paste from Bitcoin)
- Testing: 1h 40min
- Documentation: 30 min
- CI/CD: 5 min

**Code Reuse**: ~85% (only changed chain name, ID, removed SegWit logic)

---

## Summary: Chain Addition Workflow

### Quick Reference

| Chain Family | Base Template | Code Reuse | Time Estimate |
|--------------|---------------|------------|---------------|
| **UTXO (Bitcoin-like)** | Bitcoin | 85-95% | 4-8 hours |
| **EVM (Ethereum-like)** | Ethereum | 95-98% | 2-4 hours |
| **Instruction (Solana-like)** | Solana | 60-70% | 8-12 hours |
| **Cosmos SDK** | First Cosmos chain | 40-60% (first), 80% (later) | 12-16 hours (first), 4-6 hours (later) |
| **Substrate** | First Substrate chain | 40-60% (first), 80% (later) | 12-16 hours (first), 4-6 hours (later) |
| **Novel Architecture** | From scratch | 0-30% | 20-40 hours |

### Shared Crates Usage

| Crate | Purpose | Used By |
|-------|---------|---------|
| `decoder-primitives` | Byte readers (LE/BE), bounds checking | ALL chains |
| `decoder-encodings` | VarInt, RLP, Compact-u16, Protobuf, SCALE | Most chains |
| `decoder-test-utils` | Property test helpers, fixtures | ALL chains |

### Future: DSL-Based Chain Addition

**Vision** (after 15+ chains implemented):

```rust
// New chain can be defined in ~100 lines
transaction_decoder! {
    chain: NewChain,
    extends: Bitcoin,  // Inherit parsing logic
    differences: {
        hash: BLAKE2,  // Different hashing
        address: Bech32,  // Different address format
    }
}
```

**Generated Code**: ~500 LOC
- Parsing functions
- Transaction type
- ChainDecoder impl
- Canonicalizer impl
- Unit tests
- Property tests
- Fuzz targets

**Time to Add Chain with DSL**: ~1 hour (90% automated)

---

## Next Steps

1. **Implement 3 more chains** from different families to validate patterns:
   - ✅ Bitcoin (UTXO) - Done
   - ✅ Ethereum (Account) - Done
   - ✅ Solana (Instruction) - Done
   - 🔄 Cosmos Hub (Cosmos SDK) - Next priority
   - 🔄 Polkadot (Substrate) - Next priority

2. **Extract common patterns** into helper functions/macros

3. **Design DSL syntax** based on collected patterns

4. **Implement DSL as proc macro** for code generation

5. **Document DSL usage** with comprehensive examples

6. **Achieve**: Add new chain in 1-2 hours with DSL

---

## Appendix: Pattern Library

### Parsing Patterns

```rust
// Pattern: Read fixed + variable field
pub fn parse_field<R: Read>(reader: &mut R) -> Result<Field> {
    let length = read_varint(reader)?;
    if length > MAX_SIZE {
        return Err(DecoderError::invalid_structure("Field too large"));
    }
    let data = read_bytes_bounded(reader, length as usize, MAX_SIZE)?;
    Ok(Field { data })
}

// Pattern: Parse array of items
pub fn parse_array<R: Read, T>(
    reader: &mut R,
    count: usize,
    parser: impl Fn(&mut R) -> Result<T>,
) -> Result<Vec<T>> {
    let mut items = Vec::with_capacity(count);
    for i in 0..count {
        items.push(parser(reader).map_err(|e| {
            DecoderError::chain_decoding(format!("Failed to parse item {}: {}", i, e))
        })?);
    }
    Ok(items)
}

// Pattern: Conditional parsing
pub fn parse_conditional<R: Read, T>(
    reader: &mut R,
    condition: bool,
    parser: impl Fn(&mut R) -> Result<T>,
    default: T,
) -> Result<T> {
    if condition {
        parser(reader)
    } else {
        Ok(default)
    }
}
```

### Hashing Patterns

```rust
// Pattern: Single hash
pub fn hash_single<H: Digest>(data: &[u8]) -> Vec<u8> {
    H::digest(data).to_vec()
}

// Pattern: Double hash (Bitcoin, Dogecoin)
pub fn hash_double<H: Digest>(data: &[u8]) -> Vec<u8> {
    let hash1 = H::digest(data);
    let hash2 = H::digest(hash1);
    hash2.to_vec()
}

// Pattern: Prefixed hash (Ethereum)
pub fn hash_prefixed<H: Digest>(prefix: &[u8], data: &[u8]) -> Vec<u8> {
    let mut combined = Vec::with_capacity(prefix.len() + data.len());
    combined.extend_from_slice(prefix);
    combined.extend_from_slice(data);
    H::digest(&combined).to_vec()
}
```

### Validation Patterns

```rust
// Pattern: Size bounds check
pub fn validate_size(data: &[u8], min: usize, max: usize) -> Result<()> {
    if data.len() < min {
        return Err(DecoderError::invalid_structure(format!(
            "Data too small: {} bytes (minimum {})", data.len(), min
        )));
    }
    if data.len() > max {
        return Err(DecoderError::invalid_structure(format!(
            "Data too large: {} bytes (maximum {})", data.len(), max
        )));
    }
    Ok(())
}

// Pattern: Count bounds check
pub fn validate_count(count: u64, max: u64, field: &str) -> Result<()> {
    if count > max {
        return Err(DecoderError::invalid_structure(format!(
            "Too many {}: {} (maximum {})", field, count, max
        )));
    }
    Ok(())
}

// Pattern: Complete consumption check
pub fn validate_complete_consumption(cursor: &Cursor<&[u8]>, expected: usize) -> Result<()> {
    let consumed = cursor.position() as usize;
    if consumed != expected {
        return Err(DecoderError::invalid_structure(format!(
            "Incomplete parsing: consumed {} bytes, expected {}",
            consumed, expected
        )));
    }
    Ok(())
}
```

---

**Version**: 1.0
**Last Updated**: 2025-01-13
**Status**: Living Document (will evolve as more chains are added)
