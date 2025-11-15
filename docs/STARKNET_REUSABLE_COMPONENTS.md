# Starknet Decoder: Reusable Components Analysis

**Date**: 2025-11-14
**Purpose**: Identify existing infrastructure that can be reused for Starknet decoder implementation
**Related**: `docs/STARKNET_RESEARCH.md`

---

## Executive Summary

**Reusable**: ~60-70% of infrastructure already exists
**New Implementation**: ~30-40% (custom cryptography + field elements)
**Estimated Savings**: 5-7 days of development time

We can leverage extensive existing infrastructure from the universal decoder project, significantly reducing implementation time for the Starknet decoder. This document maps Starknet requirements to existing reusable components.

---

## 1. Core Infrastructure (100% Reusable ✅)

### 1.1 Trait System

**From**: `crates/universal-decoder-core/`

**Fully Reusable**:
```rust
// ✅ Use exactly as-is
pub trait ChainIdentity {
    fn chain_id(&self) -> u64;          // Starknet: 23448594291968336
    fn chain_name(&self) -> &str;       // "Starknet"
    fn chain_family(&self) -> ChainFamily; // ChainFamily::Account
}

pub trait ChainDecoder {
    type TxSpecific;                    // StarknetTransaction
    type Chain: ChainIdentity;          // StarknetChain

    fn chain() -> Self::Chain;
    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific>;
    fn validate_format(raw_bytes: &[u8]) -> Result<()>;
}

pub trait Canonicalizer {
    fn to_canonical_bytes(&self) -> Result<Vec<u8>>; // Use Borsh
}

pub trait TxHashable {
    fn canonical_hash(&self) -> Result<[u8; 32]>; // Use Poseidon/Pedersen
}
```

**Pattern for Starknet**:
```rust
// crates/decoder-starknet/src/lib.rs
use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct StarknetChain;

impl ChainIdentity for StarknetChain {
    fn chain_id(&self) -> u64 {
        23448594291968336 // Mainnet
    }

    fn chain_name(&self) -> &str {
        "Starknet"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

pub struct StarknetDecoder;

impl ChainDecoder for StarknetDecoder {
    type TxSpecific = StarknetTransaction;
    type Chain = StarknetChain;

    fn chain() -> Self::Chain {
        StarknetChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Use existing pattern from Bitcoin/Ethereum/Solana
        Self::validate_format(raw_bytes)?;
        let mut cursor = Cursor::new(raw_bytes);
        parse_starknet_transaction(&mut cursor)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::empty_input());
        }
        // Starknet-specific validation
        Ok(())
    }
}
```

**Savings**: ✅ 100% reuse, 0 lines to write

---

## 2. Byte Reading Primitives (90% Reusable ✅)

### 2.1 Big-Endian Readers

**From**: `crates/decoder-primitives/src/readers/big_endian.rs` (167 LOC)

**Starknet Uses Big-Endian**:
- Field elements: 32-byte big-endian integers (252-bit, top bits zero)
- Addresses: 32-byte big-endian
- Hashes: 32-byte big-endian

**Directly Reusable Functions**:
```rust
// ✅ Already implemented, use as-is
use decoder_primitives::prelude::*;

// Read field element (32 bytes)
let felt: [u8; 32] = read_u256_be(&mut cursor)?;

// Read shorter integers (for lengths, counts)
let version: u32 = read_u32_be(&mut cursor)?;
let nonce: u64 = read_u64_be(&mut cursor)?;

// Read address (same as felt, 32 bytes)
let address: [u8; 32] = read_u256_be(&mut cursor)?;
```

**What We Have**:
- ✅ `read_u16_be()` - u16 big-endian
- ✅ `read_u32_be()` - u32 big-endian
- ✅ `read_u64_be()` - u64 big-endian
- ✅ `read_u128_be()` - u128 big-endian
- ✅ `read_u256_be()` - **Perfect for field elements!**
- ✅ `read_address()` - 20 bytes (can create `read_felt()` wrapper)

**What We Need to Add** (10% new):
```rust
// crates/decoder-primitives/src/readers/big_endian.rs
// Add 10-15 LOC

/// Read Starknet field element (32 bytes, big-endian, 252-bit)
///
/// Note: Top 4 bits must be zero (252-bit constraint)
pub fn read_felt<R: Read>(reader: &mut R) -> Result<[u8; 32]> {
    let bytes = read_u256_be(reader)?;

    // Validate 252-bit constraint (top 4 bits = 0)
    if bytes[0] & 0xF0 != 0 {
        return Err(DecoderError::invalid_structure(
            "Field element exceeds 252 bits"
        ));
    }

    Ok(bytes)
}
```

**Savings**: ✅ 90% reuse, 10-15 new lines

### 2.2 Bounded Byte Reading

**From**: `crates/decoder-primitives/src/bytes.rs` (181 LOC)

**Fully Reusable for Starknet**:
```rust
// ✅ Use for variable-length data (calldata, signatures)
use decoder_primitives::prelude::*;

// Read signature (2 field elements = 64 bytes)
let signature = read_bytes_bounded(&mut cursor, 64, 128)?;

// Read calldata (variable length, max 10 MB)
let calldata_len = read_u64_be(&mut cursor)? as usize;
let calldata = read_bytes_bounded(&mut cursor, calldata_len, 10 * 1024 * 1024)?;

// Read constructor arguments (bounded)
let constructor_calldata = read_bytes_bounded(&mut cursor, len, 1024 * 1024)?;
```

**Functions We'll Use**:
- ✅ `read_bytes_bounded()` - Safe variable-length reads
- ✅ `read_array::<N>()` - Fixed-size arrays (felts, addresses)
- ✅ `read_remaining()` - Read rest of transaction

**Savings**: ✅ 100% reuse, 0 lines to write

---

## 3. Error Handling (100% Reusable ✅)

**From**: `crates/universal-decoder-core/src/error.rs`

**Use Existing Error Types**:
```rust
// ✅ All error scenarios covered
use decoder_primitives::prelude::*;

// Invalid transaction structure
Err(DecoderError::invalid_structure("Invalid INVOKE transaction"))

// Chain-specific decoding error
Err(DecoderError::chain_decoding("Failed to parse field element"))

// Empty input
Err(DecoderError::empty_input())

// Custom error with context
Err(DecoderError::chain_decoding(format!(
    "Unknown transaction version: {}", version
)))
```

**Starknet-Specific Errors to Add**:
```rust
// All handled by existing error types
- Invalid field element (> 252 bits) → DecoderError::invalid_structure()
- Unknown transaction type → DecoderError::chain_decoding()
- Invalid signature → DecoderError::invalid_structure()
- Hash mismatch → DecoderError::invalid_structure()
```

**Savings**: ✅ 100% reuse, 0 lines to write

---

## 4. Testing Infrastructure (95% Reusable ✅)

### 4.1 Property Test Helpers

**From**: `crates/decoder-test-utils/src/proptest_helpers.rs`

**Directly Reusable**:
```rust
use decoder_test_utils::proptest_helpers::*;

// ✅ Test decoder never panics on arbitrary input
proptest! {
    #[test]
    fn prop_starknet_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<StarknetDecoder>(&bytes);
    }
}

// ✅ Test canonical serialization is deterministic
proptest! {
    #[test]
    fn prop_borsh_deterministic(tx in arb_starknet_tx()) {
        let bytes1 = borsh::to_vec(&tx).unwrap();
        let bytes2 = borsh::to_vec(&tx).unwrap();
        prop_assert_eq!(bytes1, bytes2);
    }
}
```

**Pattern from Bitcoin/Cosmos**:
- ✅ `prop_decoder_never_panics()` - Generic, works for any decoder
- ✅ `arb_small_bytes()` - Generate random byte sequences
- ✅ `ProptestConfig::with_cases(1000)` - Standard configuration

**New Property Tests Needed** (5% new):
```rust
// Starknet-specific properties
proptest! {
    /// Property: Field element modular arithmetic
    #[test]
    fn prop_felt_addition_commutative(a in arb_felt(), b in arb_felt()) {
        prop_assert_eq!(felt_add(a, b), felt_add(b, a));
    }

    /// Property: Poseidon hash is deterministic
    #[test]
    fn prop_poseidon_deterministic(a in arb_felt(), b in arb_felt()) {
        let hash1 = poseidon_hash(a, b);
        let hash2 = poseidon_hash(a, b);
        prop_assert_eq!(hash1, hash2);
    }
}

// Helper: Generate arbitrary field elements
fn arb_felt() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>().prop_map(|mut bytes| {
        bytes[0] &= 0x0F; // Ensure 252-bit constraint
        bytes
    })
}
```

**Savings**: ✅ 95% reuse, 30-50 new lines for Starknet-specific properties

### 4.2 CI/CD Pipeline

**From**: `.github/workflows/test.yml`, `.github/workflows/nightly.yml`

**Fully Reusable**:
- ✅ Unit tests (`cargo test --all`)
- ✅ Property tests (included in `cargo test`)
- ✅ Clippy linting (`cargo clippy -- -D warnings`)
- ✅ Format checking (`cargo fmt -- --check`)
- ✅ Coverage reporting (`cargo llvm-cov`)
- ✅ Nightly fuzz tests

**No Changes Needed**: Starknet decoder automatically included when added to workspace

**Savings**: ✅ 100% reuse, 0 lines to write

---

## 5. Transaction Structure Patterns (80% Reusable ✅)

### 5.1 Multiple Transaction Types (Like Cosmos)

**From**: `crates/decoder-cosmos/src/types.rs`

**Cosmos Has 8+ Message Types, Starknet Has 3 Transaction Types**:

**Reusable Pattern**:
```rust
// Similar to Cosmos CosmosMessage enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StarknetTransaction {
    /// INVOKE transaction (v1 or v3)
    Invoke(InvokeTransaction),
    /// DECLARE transaction (v0 or v3)
    Declare(DeclareTransaction),
    /// DEPLOY_ACCOUNT transaction (v1 or v3)
    DeployAccount(DeployAccountTransaction),
}

// Similar to Cosmos Tx wrapper
#[derive(Debug, Clone, PartialEq)]
pub struct StarknetTx {
    /// Transaction type and data
    pub transaction: StarknetTransaction,
    /// Transaction hash (computed)
    pub hash: [u8; 32],
    /// Chain ID
    pub chain_id: u64,
}
```

**Parsing Pattern from Cosmos**:
```rust
// Similar: Read discriminant, match on type
fn parse_starknet_transaction(cursor: &mut Cursor<&[u8]>) -> Result<StarknetTransaction> {
    // Read version byte (discriminant)
    let version = read_u32_be(cursor)?;

    match version {
        1 => parse_invoke_v1(cursor),
        3 => {
            // Read transaction type
            let tx_type = read_u32_be(cursor)?;
            match tx_type {
                0 => parse_invoke_v3(cursor),
                1 => parse_declare_v3(cursor),
                2 => parse_deploy_account_v3(cursor),
                _ => Err(DecoderError::chain_decoding(
                    format!("Unknown v3 transaction type: {}", tx_type)
                )),
            }
        }
        _ => Err(DecoderError::chain_decoding(
            format!("Unsupported version: {}", version)
        )),
    }
}
```

**Savings**: ✅ 80% pattern reuse, 20% Starknet-specific structure

### 5.2 Field Extraction Pattern (Like Bitcoin/Ethereum)

**From**: `crates/decoder-bitcoin/src/parsing.rs`, `crates/decoder-ethereum/src/parsing.rs`

**Reusable Parse-Extract-Validate Pattern**:
```rust
// Similar to parse_bitcoin_transaction()
fn parse_invoke_v3(cursor: &mut Cursor<&[u8]>) -> Result<InvokeTransaction> {
    // 1. Read fixed-size fields
    let sender_address = read_felt(cursor)?;
    let nonce = read_u64_be(cursor)?;

    // 2. Read variable-length arrays
    let calldata_len = read_u64_be(cursor)? as usize;
    let calldata = read_bytes_bounded(cursor, calldata_len, MAX_CALLDATA_SIZE)?;

    // 3. Read signature (2 felts = 64 bytes)
    let signature = read_bytes_bounded(cursor, 64, 128)?;

    // 4. Validate and construct
    Ok(InvokeTransaction {
        version: 3,
        sender_address,
        nonce,
        calldata,
        signature,
        // ... other fields
    })
}
```

**Pattern Match from Existing Decoders**:
- ✅ Bitcoin: Fixed-size header → variable inputs → variable outputs
- ✅ Ethereum: RLP list → extract fields → validate
- ✅ Solana: Compact-u16 lengths → variable arrays → validate
- ✅ **Starknet**: Version → type → fields → felts → validate

**Savings**: ✅ 80% pattern reuse, 20% Starknet field types

---

## 6. TxIR Conversion (90% Reusable ✅)

### 6.1 Account-Based Model (Like Ethereum/Solana)

**From**: `crates/decoder-ethereum/src/lib.rs`, `crates/decoder-solana/src/lib.rs`

**Starknet Uses Account Model** (not UTXO):

**Reusable Conversion Pattern**:
```rust
impl From<StarknetTransaction> for TxIR {
    fn from(tx: StarknetTransaction) -> Self {
        let chain = StarknetChain;

        // 1. Extract metadata (similar to Ethereum)
        let metadata = TxMetadata {
            chain_id: chain.chain_id(),
            version: tx.version().to_string(),
            timestamp: None, // Not in transaction
            block_height: None, // Not in transaction
            ..Default::default()
        };

        // 2. Extract authorization (similar to Ethereum)
        let authorization = Authorization {
            signatures: vec![Signature {
                public_key: extract_public_key(&tx)?,
                signature_data: tx.signature().to_vec(),
                algorithm: "ECDSA-STARK".to_string(),
            }],
            nonce: Some(tx.nonce()),
            ..Default::default()
        };

        // 3. Extract operations (similar to Solana instructions)
        let operations = match &tx {
            StarknetTransaction::Invoke(invoke) => {
                // Parse calldata to extract function calls
                vec![Operation {
                    op_type: OperationType::ContractCall,
                    from: Some(Address::from_bytes(invoke.sender_address)),
                    to: Some(extract_contract_address(&invoke.calldata)?),
                    // ... extract amounts from calldata
                }]
            }
            StarknetTransaction::Declare(declare) => {
                vec![Operation {
                    op_type: OperationType::ContractDeployment,
                    from: Some(Address::from_bytes(declare.sender_address)),
                    // ... class hash, compiled class hash
                }]
            }
            StarknetTransaction::DeployAccount(deploy) => {
                vec![Operation {
                    op_type: OperationType::AccountCreation,
                    to: Some(compute_contract_address(&deploy)?),
                    // ... constructor calldata
                }]
            }
        };

        // 4. Extract state deltas (similar to Solana account writes)
        let state_deltas = extract_state_changes(&tx);

        TxIR::new(
            &chain,
            metadata,
            authorization,
            operations,
            state_deltas,
        )
    }
}
```

**Similarities**:
- ✅ Account model (like Ethereum/Solana, not UTXO like Bitcoin)
- ✅ Nonce-based ordering (like Ethereum)
- ✅ Signatures (like all chains)
- ✅ Operations/Instructions (like Solana)
- ✅ State deltas (like Solana account writes)

**Savings**: ✅ 90% pattern reuse, 10% Starknet-specific operations

---

## 7. What's NOT Reusable (30-40% New Code)

### 7.1 Field Element Type (NEW)

**Need to Implement**: 252-bit modular arithmetic

```rust
// crates/decoder-starknet/src/felt.rs
// Estimated: 200-300 LOC

/// Starknet field element (252-bit, prime field)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Felt([u8; 32]);

impl Felt {
    /// Prime: 2^251 + 17 * 2^192 + 1
    pub const PRIME: [u8; 32] = [
        // ... prime constant
    ];

    /// Create from bytes (validates 252-bit constraint)
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self> {
        if bytes[0] & 0xF0 != 0 {
            return Err(DecoderError::invalid_structure(
                "Field element exceeds 252 bits"
            ));
        }
        Ok(Felt(bytes))
    }

    /// Modular addition
    pub fn add(&self, other: &Felt) -> Felt {
        // Implement modular addition
    }

    /// Modular multiplication
    pub fn mul(&self, other: &Felt) -> Felt {
        // Implement modular multiplication
    }

    // ... other operations
}
```

**Options**:
1. **Implement from scratch**: 200-300 LOC, 2-3 days
2. **Vendor from `starknet-crypto`**: Extract minimal code, 1 day
3. **Use `starknet-crypto` as dependency**: Add to Cargo.toml, 0 days

**Recommendation**: Option 3 for Phase 3.6, Option 2 for Phase 4

### 7.2 Cryptographic Primitives (NEW)

#### Poseidon Hash (HIGHEST PRIORITY)

**Need to Implement**: Hades permutation for Poseidon hash

```rust
// crates/decoder-starknet/src/crypto/poseidon.rs
// Estimated: 400-500 LOC

/// Poseidon hash (Starknet version)
pub fn poseidon_hash(a: Felt, b: Felt) -> Felt {
    // 1. Hades permutation on (a, b, 1)
    let state = [a, b, Felt::ONE];
    let permuted = hades_permutation(state);

    // 2. Return first element
    permuted[0]
}

fn hades_permutation(state: [Felt; 3]) -> [Felt; 3] {
    // Full rounds + partial rounds + full rounds
    // Uses round constants and MDS matrix
    // ... complex implementation
}
```

**Options**:
1. **Implement from scratch**: 400-500 LOC, 3-4 days
2. **Vendor from `starknet-crypto`**: Extract minimal code, 1-2 days
3. **Use `starknet-crypto` as dependency**: Add to Cargo.toml, 0 days

**Recommendation**: Option 3 (dependency) for speed

#### Pedersen Hash (MEDIUM PRIORITY)

**Need to Implement**: Elliptic curve operations

```rust
// crates/decoder-starknet/src/crypto/pedersen.rs
// Estimated: 300-400 LOC

/// Pedersen hash (Starknet version)
pub fn pedersen_hash(a: Felt, b: Felt) -> Felt {
    // Elliptic curve point operations
    // h(a,b) = [shift_point + a_low·P₀ + a_high·P₁ + b_low·P₂ + b_high·P₃]ₓ
}
```

**Usage**: Legacy transactions (v1), storage addresses
**Options**: Same as Poseidon

#### ECDSA on STARK Curve (MEDIUM PRIORITY)

**Need to Implement**: Signature verification

```rust
// crates/decoder-starknet/src/crypto/ecdsa.rs
// Estimated: 300-400 LOC

/// Verify ECDSA signature on STARK curve
pub fn verify_signature(
    message_hash: Felt,
    public_key: Felt,
    signature: (Felt, Felt), // (r, s)
) -> Result<bool> {
    // ECDSA verification on STARK curve
}
```

**Usage**: All transactions (signature validation)

### 7.3 Transaction Parsing (NEW)

**Need to Implement**: 3 transaction types

```rust
// crates/decoder-starknet/src/parsing.rs
// Estimated: 600-800 LOC

fn parse_invoke_v1(...) -> Result<InvokeTransaction> { ... }
fn parse_invoke_v3(...) -> Result<InvokeTransaction> { ... }
fn parse_declare_v0(...) -> Result<DeclareTransaction> { ... }
fn parse_declare_v3(...) -> Result<DeclareTransaction> { ... }
fn parse_deploy_account_v1(...) -> Result<DeployAccountTransaction> { ... }
fn parse_deploy_account_v3(...) -> Result<DeployAccountTransaction> { ... }
```

**Estimated**: 100-150 LOC per transaction type × 6 = 600-900 LOC

---

## 8. Summary Table

| Component | LOC | Reusable % | New LOC | Effort |
|-----------|-----|------------|---------|--------|
| **Core Traits** | 200 | 100% | 0 | ✅ 0 days |
| **Byte Readers** | 350 | 95% | 15 | ✅ 0.5 days |
| **Error Handling** | 100 | 100% | 0 | ✅ 0 days |
| **Testing Infrastructure** | 500 | 95% | 50 | ✅ 0.5 days |
| **Transaction Patterns** | 300 | 80% | 60 | ✅ 1 day |
| **TxIR Conversion** | 200 | 90% | 20 | ✅ 0.5 days |
| **Field Element Type** | 300 | 0% | 300 | ⚠️ 2-3 days OR use dep |
| **Poseidon Hash** | 500 | 0% | 500 | ⚠️ 3-4 days OR use dep |
| **Pedersen Hash** | 400 | 0% | 400 | ⚠️ 2-3 days OR use dep |
| **ECDSA Verify** | 400 | 0% | 400 | ⚠️ 2-3 days OR use dep |
| **Transaction Parsing** | 800 | 20% | 640 | ⚠️ 3-4 days |
| **Documentation** | 200 | 50% | 100 | ✅ 1 day |
| **TOTAL** | **4,250** | **~60%** | **~2,485** | **15-20 days** |

**With starknet-crypto Dependency**:
| Component | LOC | Reusable % | New LOC | Effort |
|-----------|-----|------------|---------|--------|
| Crypto (dep) | 1,600 | 100% (external) | 0 | ✅ 0 days |
| Other | 2,650 | 70% | 885 | ✅ 4-5 days |
| **TOTAL** | **4,250** | **~80%** | **~885** | **4-5 days** |

---

## 9. Recommended Implementation Strategy

### Phase 3.6a: Core Decoder (3-4 days)

**Use `starknet-crypto` as dependency** (optimize for speed):

```toml
# crates/decoder-starknet/Cargo.toml
[dependencies]
decoder-primitives = { path = "../decoder-primitives" }
decoder-encodings = { path = "../decoder-encodings" }
starknet-crypto = "0.7"  # ✅ Apache-2.0/MIT

[dev-dependencies]
starknet = "0.11"  # Validation
starknet-core = "0.11"  # Validation
```

**Reuse from existing**:
- ✅ `decoder-primitives` → byte readers (big-endian, felts)
- ✅ `decoder-primitives` → bounds checking
- ✅ Core traits → `ChainDecoder`, `ChainIdentity`
- ✅ Testing → property test helpers, CI/CD
- ✅ Patterns → multiple tx types (like Cosmos), account model (like Ethereum)

**New implementation**:
- ⚠️ Transaction parsing (3 types × 2 versions = 6 parsers)
- ⚠️ TxIR conversion (Starknet → universal IR)
- ⚠️ Hash verification (use `starknet-crypto::poseidon_hash`)
- ⚠️ Signature verification (use `starknet-crypto::verify`)

**Deliverable**: Working decoder with 50+ tests

### Phase 4.x: Vendor Crypto (Optional, Future)

**Vendor `starknet-crypto` for minimal TCB**:

```bash
git subtree add \
    --prefix crates/decoder-starknet/vendored/starknet-crypto \
    https://github.com/xJonathanLEI/starknet-rs.git \
    starknet-crypto/v0.7.0 --squash
```

**Extract only**:
- Field element type
- Poseidon hash
- Pedersen hash (for v1 tx validation)
- ECDSA verification

**Remove**:
- Network code
- Signer implementations
- Account abstraction logic

**Estimated**: 2-3 days extraction + testing

---

## 10. Code Reuse Examples

### Example 1: Reading Transaction Header

**Reusable** from `decoder-primitives`:
```rust
use decoder_primitives::prelude::*;
use std::io::Cursor;

fn parse_invoke_v3(raw_bytes: &[u8]) -> Result<InvokeTransaction> {
    let mut cursor = Cursor::new(raw_bytes);

    // ✅ Reuse: Big-endian readers
    let version = read_u32_be(&mut cursor)?;
    let tx_type = read_u32_be(&mut cursor)?;

    // ✅ Reuse: Field element reader (add to primitives)
    let sender = read_felt(&mut cursor)?;
    let nonce = read_u64_be(&mut cursor)?;

    // ✅ Reuse: Bounded array read
    let calldata_len = read_u64_be(&mut cursor)? as usize;
    let calldata = read_bytes_bounded(&mut cursor, calldata_len, MAX_CALLDATA)?;

    // ✅ Reuse: Fixed array read
    let signature = read_array::<64>(&mut cursor)?; // 2 felts

    Ok(InvokeTransaction {
        version,
        sender,
        nonce,
        calldata,
        signature,
    })
}
```

**Reuse**: 95% of parsing logic

### Example 2: Property Testing

**Reusable** from `decoder-test-utils`:
```rust
use decoder_test_utils::proptest_helpers::*;

proptest! {
    /// ✅ Reuse: Generic decoder panic test
    #[test]
    fn prop_starknet_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<StarknetDecoder>(&bytes);
    }

    /// ✅ Reuse: Borsh determinism test
    #[test]
    fn prop_borsh_deterministic(tx in arb_starknet_tx()) {
        let bytes1 = borsh::to_vec(&tx).unwrap();
        let bytes2 = borsh::to_vec(&tx).unwrap();
        prop_assert_eq!(bytes1, bytes2);
    }

    /// ⚠️ New: Starknet-specific property
    #[test]
    fn prop_poseidon_deterministic(a in arb_felt(), b in arb_felt()) {
        use starknet_crypto::poseidon_hash;
        let h1 = poseidon_hash(a, b);
        let h2 = poseidon_hash(a, b);
        prop_assert_eq!(h1, h2);
    }
}
```

**Reuse**: 80% of test infrastructure

### Example 3: TxIR Conversion

**Reusable** pattern from Ethereum/Solana:
```rust
impl From<StarknetTransaction> for TxIR {
    fn from(tx: StarknetTransaction) -> Self {
        // ✅ Reuse: Metadata extraction pattern (like Ethereum)
        let metadata = TxMetadata {
            chain_id: StarknetChain.chain_id(),
            version: tx.version().to_string(),
            // ... same pattern as Ethereum
        };

        // ✅ Reuse: Authorization pattern (like all chains)
        let authorization = Authorization {
            signatures: vec![/* extract from tx */],
            nonce: Some(tx.nonce()),
            // ... same pattern as Ethereum
        };

        // ✅ Reuse: Operations pattern (like Solana instructions)
        let operations = match &tx {
            StarknetTransaction::Invoke(inv) => {
                // Parse calldata to operations (similar to Solana instructions)
                parse_invoke_operations(&inv.calldata)
            }
            // ... other types
        };

        // ✅ Reuse: TxIR constructor (same for all chains)
        TxIR::new(
            &StarknetChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        )
    }
}
```

**Reuse**: 90% of conversion logic

---

## 11. Conclusion

**Massive Infrastructure Already Built**:
- ✅ Core traits (100% reusable)
- ✅ Byte readers (95% reusable, just add `read_felt()`)
- ✅ Error handling (100% reusable)
- ✅ Testing framework (95% reusable)
- ✅ CI/CD pipeline (100% reusable)
- ✅ Transaction patterns (80% reusable)
- ✅ TxIR conversion (90% reusable)

**What We Need to Build**:
- ⚠️ Cryptography (Poseidon, Pedersen, ECDSA) → **Use `starknet-crypto` dependency**
- ⚠️ Transaction parsing (3 types, 6 variants) → **~800 LOC new**
- ⚠️ Starknet-specific tests → **~100 LOC new**

**Effort Reduction**:
- **Without reuse**: 15-20 days
- **With reuse + starknet-crypto dependency**: **4-5 days** ✅
- **Savings**: **70-75% time saved**

**Recommendation**:
1. **Phase 3.6** (current): Use `starknet-crypto` dependency, focus on decoder logic
2. **Phase 4.x** (future): Vendor crypto for minimal TCB (if needed for audit)

The existing infrastructure is **perfectly suited** for Starknet integration. We can leverage 60-80% of existing code, reducing implementation time from 3 weeks to less than 1 week.

---

**Next Steps**:
1. Add `starknet-crypto` to `Cargo.toml`
2. Implement `read_felt()` in `decoder-primitives` (10 LOC)
3. Create transaction type structs (100 LOC)
4. Implement 6 transaction parsers (800 LOC)
5. Write property tests (100 LOC)
6. Integrate with CI/CD (automatic)

**Total New Code**: ~1,000 LOC (vs ~2,500 LOC from scratch)
**Time to First Working Decoder**: 4-5 days (vs 15-20 days)

---

**Document Version**: 1.0
**Date**: 2025-11-14
**Related**: `docs/STARKNET_RESEARCH.md`
