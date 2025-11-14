# Zcash Integration Plan: Complete Privacy Chain Support

**Status**: Planning Document
**Version**: 1.0.0
**Created**: 2025-01-14
**Phase**: 3.8 (Privacy Chains Family Decoder)
**Priority**: HIGH

---

## Executive Summary

This document provides a comprehensive plan for integrating Zcash (ZEC) into the Universal Blockchain Decoder, including support for all transaction types (transparent, shielding, deshielding, and shielded) across three protocol versions (Sprout, Sapling, Orchard).

**Key Features**:
- ✅ Transparent transactions (Bitcoin-compatible, reuse existing decoder)
- ✅ Shielded transactions (zk-SNARK privacy with Sapling/Orchard)
- ✅ Hybrid transactions (t→z, z→t, mixed)
- ✅ Viewing key support (selective disclosure for compliance)
- ✅ All three protocol versions (Sprout, Sapling, Orchard)

**Timeline**: 2-3 weeks (40-60 hours)
**Dependencies**: Privacy infrastructure (✅ complete), Bitcoin decoder (✅ complete)

---

## Table of Contents

1. [Zcash Transaction Type Taxonomy](#zcash-transaction-type-taxonomy)
2. [Chain Family Classification](#chain-family-classification)
3. [Architecture Overview](#architecture-overview)
4. [Implementation Phases](#implementation-phases)
5. [Technical Specifications](#technical-specifications)
6. [Testing Strategy](#testing-strategy)
7. [Roadmap Updates](#roadmap-updates)
8. [Success Criteria](#success-criteria)

---

## Zcash Transaction Type Taxonomy

### Overview

Zcash supports **4 transaction types** based on input/output transparency:

| Type | From | To | Privacy Level | Frequency | Complexity |
|------|------|-----|--------------|-----------|-----------|
| **t→t** | Transparent | Transparent | None | ~40% | Low (Bitcoin-like) |
| **t→z** | Transparent | Shielded | Medium | ~25% | Medium (shielding) |
| **z→t** | Shielded | Transparent | Medium | ~20% | Medium (deshielding) |
| **z→z** | Shielded | Shielded | High | ~15% | High (fully private) |

**Mixed**: Transactions can have multiple inputs/outputs of different types (e.g., 2 transparent inputs + 1 shielded output).

### Transaction Structure (Post-NU5/Orchard)

```
Zcash Transaction {
    header: {
        version: u32,              // 4 (Sapling) or 5 (Orchard)
        version_group_id: u32,     // Consensus branch ID
        consensus_branch_id: u32,  // Network upgrade
    },

    // Transparent component (Bitcoin-like)
    transparent: {
        inputs: Vec<TxIn>,         // UTXOs with signatures
        outputs: Vec<TxOut>,       // P2PKH/P2SH addresses
    },

    // Sapling shielded component
    sapling: Option<{
        spends: Vec<SpendDescription>,      // Consume shielded notes
        outputs: Vec<OutputDescription>,    // Create shielded notes
        value_balance: i64,                 // Net transparent ↔ shielded
        binding_sig: Signature,             // Proves value balance
    }>,

    // Orchard shielded component (NU5+)
    orchard: Option<{
        actions: Vec<ActionDescription>,    // Combined spend+output
        flags: u8,                          // Enable spends/outputs
        value_balance: i64,                 // Net transparent ↔ shielded
        anchor: [u8; 32],                   // Merkle root
        proof: GrothProof,                  // zk-SNARK proof
        binding_sig: Signature,             // Binding signature
    }>,

    // Sprout shielded component (legacy, deprecated)
    joinsplits: Vec<JoinSplitDescription>,  // Old shielded format
    joinsplit_pubkey: Option<PublicKey>,
    joinsplit_sig: Option<Signature>,

    locktime: u32,
    expiry_height: u32,                     // Transaction expiration
}
```

### Protocol Versions

| Version | Name | Status | Year | Notes |
|---------|------|--------|------|-------|
| **Sprout** | Original | Deprecated | 2016 | JoinSplit, slow, avoid |
| **Sapling** | Current | Active | 2018 | Fast, 100x smaller proofs |
| **Orchard** | Latest | Recommended | 2021 | Unified addresses, Halo2 |

**Implementation Priority**: Sapling > Orchard > Sprout (skip if possible)

---

## Chain Family Classification

### Zcash Position in Chain Families

Zcash is **hybrid**: UTXO (transparent) + Privacy Pool (shielded)

```
Chain Family Taxonomy:
├── UTXO Family
│   ├── Bitcoin ✅
│   ├── Bitcoin Forks (Dogecoin, Litecoin) ✅
│   └── Zcash (transparent) ✅ [NEW]
│
└── Privacy Family [NEW]
    ├── Zcash (shielded) ⭐ PRIMARY TARGET
    │   ├── Transparent: Reuse BitcoinDecoder
    │   ├── Sapling: zk-SNARK shielded transactions
    │   └── Orchard: Unified addresses, Halo2 proofs
    ├── Monero (planned)
    │   └── RingCT, stealth addresses
    └── Aleo (planned)
        └── Leo VM, programmable privacy
```

**Decision**: Create `decoder-zcash` as standalone decoder (not a Bitcoin fork wrapper) because:
1. Shielded transactions are fundamentally different from Bitcoin
2. Multiple protocol versions require version-specific parsing
3. Privacy metadata population is Zcash-specific
4. Viewing key decryption logic is unique

### Chain Family Definition Update

**File**: `crates/universal-decoder-core/src/chain_family.rs`

**Current**:
```rust
pub enum ChainFamily {
    Utxo,        // Bitcoin, Dogecoin, Litecoin
    Account,     // Ethereum, Polygon, BNB
    Instruction, // Solana, SVM chains
}
```

**Proposed** (Phase 3.8):
```rust
pub enum ChainFamily {
    Utxo,        // Bitcoin, Dogecoin, Litecoin
    Account,     // Ethereum, Polygon, BNB
    Instruction, // Solana, SVM chains
    Privacy,     // Zcash, Monero, Aleo [NEW]
    Hybrid,      // Zcash transparent (UTXO + Privacy) [ALTERNATIVE]
}
```

**Recommendation**: Use `ChainFamily::Privacy` for Zcash (even though it has transparent component).

**Rationale**:
- Primary distinguishing feature is privacy capability
- Transparent txs are special case, not the defining characteristic
- Groups naturally with Monero, Aleo (also privacy-focused)
- Allows separate trait implementations (PrivacyAwareDecoder)

---

## Architecture Overview

### Crate Structure

```
crates/decoder-zcash/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md
├── src/
│   ├── lib.rs                    # Chain identity, main decoder
│   ├── types.rs                  # ZcashTransaction enum
│   ├── transparent.rs            # Bitcoin-like parsing (reuse)
│   ├── sapling/
│   │   ├── mod.rs                # Sapling module
│   │   ├── spend.rs              # SpendDescription parsing
│   │   ├── output.rs             # OutputDescription parsing
│   │   └── proofs.rs             # zk-SNARK proof structures
│   ├── orchard/
│   │   ├── mod.rs                # Orchard module
│   │   ├── action.rs             # ActionDescription parsing
│   │   └── proofs.rs             # Halo2 proof structures
│   ├── sprout/                   # Legacy support (optional)
│   │   ├── mod.rs
│   │   └── joinsplit.rs          # JoinSplit parsing
│   ├── viewing_key/
│   │   ├── mod.rs
│   │   ├── sapling_vk.rs         # Sapling viewing key decryption
│   │   └── orchard_vk.rs         # Orchard viewing key decryption
│   └── privacy.rs                # Privacy metadata population
├── tests/
│   ├── transparent_tests.rs      # t→t transactions
│   ├── shielding_tests.rs        # t→z transactions
│   ├── deshielding_tests.rs      # z→t transactions
│   ├── shielded_tests.rs         # z→z transactions
│   ├── viewing_key_tests.rs      # VK decryption tests
│   └── fixtures/
│       ├── transparent/           # 30+ transparent tx hex files
│       ├── sapling/               # 20+ Sapling tx hex files
│       ├── orchard/               # 10+ Orchard tx hex files
│       └── mainnet/               # Real mainnet transactions
└── examples/
    ├── decode_transparent.rs
    ├── decode_shielded.rs
    └── decrypt_with_viewing_key.rs
```

### Dependency Strategy (Airgapped Compliant)

**Core Dependencies** (vendored via git subtree):
```toml
[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
decoder-encodings = { path = "../decoder-encodings" }  # VarInt
sha2 = "0.10"       # SHA-256 hashing
blake2b_simd = "1.0"  # BLAKE2b for Sapling
borsh = "1.3"       # Canonical serialization
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"

# Crypto (vendor via git subtree)
# Option A: Vendor from ZCash official repos
# Option B: Pure Rust implementations (preferred)
[dependencies.jubjub]
git = "https://github.com/zkcrypto/jubjub.git"
tag = "v0.10.0"
# TODO: Convert to git subtree vendoring

[dependencies.bls12_381]
git = "https://github.com/zkcrypto/bls12_381.git"
tag = "v0.8.0"
# TODO: Convert to git subtree vendoring
```

**Dev Dependencies** (test validation only):
```toml
[dev-dependencies]
zcash_primitives = "0.13"  # Reference implementation
zcash_client_backend = "0.12"  # Viewing keys
hex = "0.4"
```

**Cryptographic Dependency Decision**:

**Question**: How to handle zk-SNARK dependencies (jubjub, bls12_381)?

**Options**:
1. **Vendor via git subtree** (RECOMMENDED)
   - ✅ Airgapped compliant
   - ✅ Verifiable supply chain
   - ✅ Full control over updates
   - ⚠️ Adds ~50KB to repo per crate

2. **Move to production dependencies**
   - ✅ Easier to implement
   - ❌ Violates airgapped requirement
   - ❌ External dependencies at runtime

3. **Implement from scratch**
   - ✅ Maximum control
   - ❌ 1000+ LOC, high risk
   - ❌ Security audit required

**Decision**: Use Option 1 (git subtree) for Phase 3.8 initial implementation. Crypto libraries are small, well-audited, and essential for Zcash support.

**Action Items**:
```bash
# Vendor jubjub elliptic curve library
git subtree add \
    --prefix crates/decoder-zcash/vendored/jubjub \
    https://github.com/zkcrypto/jubjub.git \
    v0.10.0 \
    --squash

# Vendor bls12_381 curve library (Orchard)
git subtree add \
    --prefix crates/decoder-zcash/vendored/bls12_381 \
    https://github.com/zkcrypto/bls12_381.git \
    v0.8.0 \
    --squash

# Vendor blake2b for Sapling hashing
git subtree add \
    --prefix crates/decoder-zcash/vendored/blake2b_simd \
    https://github.com/oconnor663/blake2_simd.git \
    v1.0.2 \
    --squash
```

---

## Implementation Phases

### Phase 1: Transparent Transactions (Days 1-2, ~8 hours)

**Goal**: Support t→t transactions (Bitcoin-compatible subset)

**Tasks**:
1. Create `decoder-zcash` crate skeleton
2. Implement `ZcashChain` struct (ChainIdentity trait)
3. Reuse `BitcoinDecoder` parsing for transparent inputs/outputs
4. Add Zcash-specific fields (version_group_id, expiry_height)
5. Parse transaction header (version 4/5 detection)
6. Implement `ZcashTransaction::Transparent` variant
7. Write 15+ tests with real mainnet transparent transactions

**Deliverables**:
- ✅ `decoder-zcash/src/transparent.rs` (~150 LOC)
- ✅ `ZcashChain` implementing ChainIdentity
- ✅ Tests for t→t transactions (15 tests)
- ✅ Integration with TxIR (transparent operations only)

**Test Fixtures Needed**:
```
tests/fixtures/transparent/
├── simple_t2t.hex           # Basic transparent transfer
├── multisig_t2t.hex         # P2SH multisig
├── coinbase_transparent.hex # Miner coinbase (all transparent)
├── expired_transparent.hex  # Transaction with expiry_height
└── ... (10+ more)
```

**Complexity**: LOW (reuses existing Bitcoin decoder logic)

---

### Phase 2: Sapling Shielded Transactions (Days 3-6, ~24 hours)

**Goal**: Parse Sapling shielded components (spend/output descriptions, proofs)

#### 2.1: Spend Descriptions (Days 3-4, ~12 hours)

**SpendDescription Structure**:
```rust
pub struct SpendDescription {
    /// Commitment value (note being spent)
    pub cv: ValueCommitment,        // 32 bytes (compressed point)

    /// Merkle tree anchor (state root)
    pub anchor: [u8; 32],           // BLAKE2b commitment root

    /// Nullifier (prevents double-spend)
    pub nullifier: [u8; 32],        // Unique per note

    /// Randomized public key
    pub rk: [u8; 32],               // Re-randomized key

    /// zk-SNARK proof (spend authorization)
    pub zkproof: [u8; 192],         // Groth16 proof

    /// Spend authorization signature
    pub spend_auth_sig: [u8; 64],   // Signature over sighash
}
```

**Parsing Tasks**:
1. Implement `parse_spend_description()` function
2. Extract nullifier (critical for double-spend detection)
3. Extract commitment (for value balance verification)
4. Store zk-SNARK proof structure (without verification)
5. Populate `PrivacyMetadata` with HiddenSender primitive
6. Write 10+ tests

**Complexity**: MEDIUM (binary parsing, no cryptographic verification)

#### 2.2: Output Descriptions (Days 4-5, ~8 hours)

**OutputDescription Structure**:
```rust
pub struct OutputDescription {
    /// Commitment value (note being created)
    pub cv: ValueCommitment,        // 32 bytes (compressed point)

    /// Note commitment (to recipient)
    pub cmu: [u8; 32],              // BLAKE2b commitment

    /// Ephemeral public key (for ECDH)
    pub ephemeral_key: [u8; 32],    // One-time DH key

    /// Encrypted note ciphertext
    pub enc_ciphertext: [u8; 580],  // ChaCha20-Poly1305 encrypted note

    /// Encrypted outgoing ciphertext
    pub out_ciphertext: [u8; 80],   // For sender recovery

    /// zk-SNARK proof (output correctness)
    pub zkproof: [u8; 192],         // Groth16 proof
}
```

**Parsing Tasks**:
1. Implement `parse_output_description()` function
2. Extract note commitment (cmu)
3. Store encrypted ciphertext (for viewing key decryption later)
4. Store ephemeral key (for ECDH key agreement)
5. Populate `PrivacyMetadata` with HiddenRecipient + HiddenAmount
6. Write 10+ tests

**Complexity**: MEDIUM (encryption handling, no decryption yet)

#### 2.3: Sapling Integration (Day 6, ~4 hours)

**Tasks**:
1. Implement `ZcashTransaction::Sapling` variant
2. Parse value_balance (net transparent ↔ shielded)
3. Parse binding_sig (proves value conservation)
4. Combine transparent + shielded components
5. Populate complete `PrivacyMetadata` for Sapling transactions
6. Write 20+ integration tests (t→z, z→t, z→z)

**Deliverables**:
- ✅ `decoder-zcash/src/sapling/` module (~400 LOC)
- ✅ Support for all Sapling transaction types
- ✅ Privacy metadata correctly populated
- ✅ 40+ comprehensive tests

**Test Fixtures Needed**:
```
tests/fixtures/sapling/
├── shielding_t2z.hex        # Transparent → shielded
├── deshielding_z2t.hex      # Shielded → transparent
├── fully_shielded_z2z.hex   # Fully private
├── mixed_inputs.hex         # 2 transparent + 1 shielded input
├── mixed_outputs.hex        # 1 transparent + 2 shielded outputs
├── zero_value_balance.hex   # Pure z→z (no transparent interaction)
└── ... (15+ more)
```

---

### Phase 3: Viewing Key Decryption (Days 7-8, ~12 hours)

**Goal**: Decrypt shielded transaction details with viewing keys

#### 3.1: Viewing Key Types

**Sapling Viewing Keys**:
```rust
pub struct SaplingFullViewingKey {
    /// Authorizing key
    pub ak: [u8; 32],

    /// Nullifier deriving key
    pub nk: [u8; 32],

    /// Outgoing viewing key
    pub ovk: [u8; 32],
}

pub struct SaplingIncomingViewingKey {
    /// Derived from FVK
    pub ivk: [u8; 32],
}
```

**Decryption Workflow**:
```
Encrypted Note (580 bytes)
    ↓
[ephemeral_key] + [IVK] → ECDH shared secret
    ↓
ChaCha20-Poly1305 decryption
    ↓
Note Plaintext (memo, amount, recipient)
```

**Tasks**:
1. Implement `ViewingKeyDecryptor` trait for Sapling
2. ECDH key agreement (ephemeral_key + IVK)
3. ChaCha20-Poly1305 decryption
4. Parse decrypted note plaintext
5. Populate TxIR operations with decrypted amounts/recipients
6. Handle decryption failure gracefully (wrong key, corrupted data)
7. Write 15+ tests (successful decryption, wrong key, invalid ciphertext)

**Deliverables**:
- ✅ `decoder-zcash/src/viewing_key/sapling_vk.rs` (~200 LOC)
- ✅ Full viewing key decryption support
- ✅ Examples showing VK usage
- ✅ 15+ tests

**Example Usage**:
```rust
use decoder_zcash::{ZcashDecoder, SaplingIncomingViewingKey};

let tx_bytes = hex::decode("...")?;
let viewing_key = SaplingIncomingViewingKey::from_bytes(&ivk_bytes)?;

// Without viewing key: Extract proofs only
let tx_ir = ZcashDecoder::decode(&tx_bytes, None)?;
assert_eq!(tx_ir.privacy.observability, ObservabilityLevel::FullyPrivate);

// With viewing key: Decrypt amounts/recipients
let tx_ir = ZcashDecoder::decode_with_viewing_key(&tx_bytes, Some(&viewing_key))?;
assert_eq!(tx_ir.operations.len(), 2);  // Decrypted transfers visible
```

---

### Phase 4: Orchard Support (Days 9-11, ~16 hours)

**Goal**: Support latest Orchard protocol (NU5+, unified addresses)

**Orchard Differences from Sapling**:
- **Actions** instead of separate spends/outputs (more compact)
- **Halo2 proofs** instead of Groth16 (no trusted setup)
- **Unified addresses** (single address for transparent + Sapling + Orchard)
- **Better efficiency** (smaller proofs, faster verification)

**ActionDescription Structure**:
```rust
pub struct ActionDescription {
    /// Commitment value (spent note)
    pub cv_net: ValueCommitment,    // 32 bytes

    /// Nullifier (prevents double-spend)
    pub nullifier: [u8; 32],

    /// Randomized verification key
    pub rk: [u8; 32],

    /// Note commitment (created note)
    pub cmx: [u8; 32],

    /// Ephemeral public key
    pub ephemeral_key: [u8; 32],

    /// Encrypted note ciphertext
    pub enc_ciphertext: [u8; 580],  // Encrypted action

    /// Encrypted outgoing ciphertext
    pub out_ciphertext: [u8; 80],
}
```

**Tasks**:
1. Implement `parse_orchard_actions()` function
2. Parse Orchard flags (enable_spends, enable_outputs)
3. Parse Halo2 proof structure (different from Groth16)
4. Implement Orchard viewing key decryption
5. Populate `PrivacyMetadata` for Orchard transactions
6. Write 15+ tests

**Deliverables**:
- ✅ `decoder-zcash/src/orchard/` module (~400 LOC)
- ✅ Full Orchard support (NU5+)
- ✅ Viewing key decryption for Orchard
- ✅ 15+ tests

**Test Fixtures Needed**:
```
tests/fixtures/orchard/
├── orchard_action.hex       # Single Orchard action
├── unified_address.hex      # Transaction to unified address
├── orchard_z2z.hex          # Fully shielded Orchard
├── mixed_sapling_orchard.hex  # Both protocols in one tx
└── ... (10+ more)
```

---

### Phase 5: Privacy Metadata & Testing (Days 12-14, ~8 hours)

**Goal**: Comprehensive testing and privacy metadata accuracy

**Tasks**:
1. Validate privacy metadata population for all transaction types
2. Add property-based tests (proptest)
3. Fuzzing infrastructure (cargo-fuzz)
4. Integration tests with 100+ real mainnet transactions
5. Documentation and examples
6. Performance benchmarking

**Deliverables**:
- ✅ 100+ integration tests
- ✅ Property tests for all parsers
- ✅ Fuzz targets for each parser
- ✅ Comprehensive documentation
- ✅ Performance benchmarks

---

## Technical Specifications

### Transaction Version Detection

```rust
pub fn detect_version(bytes: &[u8]) -> Result<ZcashVersion> {
    let header = read_u32_le(&bytes[0..4])?;
    let version = header & 0x7FFFFFFF;  // Mask out overwinter bit
    let is_overwinter = (header & 0x80000000) != 0;

    match version {
        1 | 2 | 3 => Ok(ZcashVersion::PreSapling),
        4 if is_overwinter => {
            // Check version_group_id
            let vgi = read_u32_le(&bytes[4..8])?;
            match vgi {
                0x892F2085 => Ok(ZcashVersion::Sapling),
                _ => Err(DecoderError::unsupported_version(version)),
            }
        },
        5 if is_overwinter => {
            // NU5+ (Orchard)
            Ok(ZcashVersion::Orchard)
        },
        _ => Err(DecoderError::unsupported_version(version)),
    }
}
```

### Privacy Metadata Population

**Example: Sapling z→z Transaction**

```rust
pub fn populate_privacy_metadata(
    tx: &ZcashTransaction,
    viewing_key: Option<&ViewingKey>,
) -> PrivacyMetadata {
    let mut features = vec![];

    // Detect shielded spends (HiddenSender)
    if !tx.sapling_spends.is_empty() {
        for spend in &tx.sapling_spends {
            features.push(PrivacyFeature::HiddenSender(PrivateAddress {
                privacy_type: AddressPrivacyType::ZkSnark {
                    protocol: "Sapling".to_string(),
                },
                public_address: spend.nullifier.to_vec(),  // Public nullifier
                viewing_hint: None,  // No hint without viewing key
            }));
        }
    }

    // Detect shielded outputs (HiddenRecipient + HiddenAmount)
    if !tx.sapling_outputs.is_empty() {
        for output in &tx.sapling_outputs {
            features.push(PrivacyFeature::HiddenRecipient(PrivateAddress {
                privacy_type: AddressPrivacyType::ZkSnark {
                    protocol: "Sapling".to_string(),
                },
                public_address: output.cmu.to_vec(),  // Note commitment
                viewing_hint: Some(output.ephemeral_key.to_vec()),
            }));

            features.push(PrivacyFeature::HiddenAmount(ConfidentialAmount {
                commitment_type: CommitmentType::Pedersen,
                commitment: output.cv.to_bytes(),
                range_proof: Some(output.zkproof.to_vec()),
                revealed_amount: None,  // Decrypt with viewing key
            }));
        }
    }

    // Determine observability level
    let observability = if tx.transparent_inputs.is_empty()
        && tx.transparent_outputs.is_empty() {
        ObservabilityLevel::FullyPrivate
    } else if !tx.sapling_spends.is_empty() || !tx.sapling_outputs.is_empty() {
        ObservabilityLevel::PartiallyObservable
    } else {
        ObservabilityLevel::FullyObservable
    };

    PrivacyMetadata {
        features,
        observability,
        viewing_key: viewing_key.map(|vk| ViewingKey {
            key_type: ViewingKeyType::Zcash,
            key_bytes: vk.to_bytes(),
        }),
    }
}
```

---

## Testing Strategy

### Test Pyramid (5 Levels)

#### Level 1: Unit Tests (~60 tests)

**Coverage**: Every parsing function

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn parse_transparent_input() { /* ... */ }

    #[test]
    fn parse_sapling_spend_description() { /* ... */ }

    #[test]
    fn parse_sapling_output_description() { /* ... */ }

    #[test]
    fn parse_orchard_action() { /* ... */ }

    #[test]
    fn detect_version_sapling() { /* ... */ }

    #[test]
    fn detect_version_orchard() { /* ... */ }

    #[test]
    fn populate_privacy_metadata_shielded() { /* ... */ }

    // ... 53+ more
}
```

#### Level 2: Property Tests (~20 tests)

**Coverage**: Invariants and safety properties

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn decoder_never_panics(tx_bytes in prop::collection::vec(any::<u8>(), 0..10000)) {
        let _ = ZcashDecoder::decode(&tx_bytes);  // Should return Err, not panic
    }

    #[test]
    fn spend_nullifiers_unique(tx in arbitrary_zcash_tx()) {
        let nullifiers: HashSet<_> = tx.sapling_spends.iter()
            .map(|s| s.nullifier)
            .collect();
        assert_eq!(nullifiers.len(), tx.sapling_spends.len());
    }

    #[test]
    fn value_balance_bounded(tx in arbitrary_zcash_tx()) {
        assert!(tx.sapling_value_balance.abs() <= MAX_MONEY);
    }

    // ... 17+ more
}
```

#### Level 3: Integration Tests (~100 tests)

**Coverage**: Real mainnet transactions

```rust
#[test]
fn mainnet_shielding_transaction() {
    // Real mainnet tx: t→z (block 1234567)
    let tx_hex = include_str!("fixtures/mainnet/shielding_t2z_block1234567.hex");
    let tx_bytes = hex::decode(tx_hex).unwrap();

    let tx_ir = ZcashDecoder::decode(&tx_bytes, None).unwrap();

    // Assertions
    assert_eq!(tx_ir.metadata.chain_id, 133);  // Zcash mainnet
    assert_eq!(tx_ir.operations.len(), 2);  // 1 transparent in, 1 shielded out
    assert!(tx_ir.privacy.is_some());
    assert_eq!(
        tx_ir.privacy.unwrap().observability,
        ObservabilityLevel::PartiallyObservable
    );
}

// ... 99+ more
```

**Test Fixture Sources**:
1. Zcash block explorer API (mainnet transactions)
2. Zcash testnet transactions
3. Reference implementation test vectors
4. Community-provided examples

#### Level 4: Fuzz Testing (Continuous)

**Coverage**: Adversarial inputs

```rust
// fuzz/fuzz_targets/fuzz_zcash_decoder.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use decoder_zcash::ZcashDecoder;

fuzz_target!(|data: &[u8]| {
    let _ = ZcashDecoder::decode(data, None);
});
```

**Fuzz Targets**:
1. `fuzz_zcash_decoder` - Full decoder pipeline
2. `fuzz_sapling_spend` - Spend description parsing
3. `fuzz_sapling_output` - Output description parsing
4. `fuzz_orchard_action` - Orchard action parsing
5. `fuzz_viewing_key_decrypt` - Viewing key decryption

**Fuzzing Strategy**: Run 1 hour/night in CI, report crashes

#### Level 5: Formal Verification (Phase 4)

**Coverage**: Critical safety properties

**Verification Targets**:
- **VT-30**: Zcash parsing never panics (bounds-checked)
- **VT-31**: Nullifier uniqueness within transaction
- **VT-32**: Value balance correctness (transparent ↔ shielded)
- **VT-33**: Privacy metadata accuracy

---

## Roadmap Updates

### ROADMAP.md Changes

**Section**: Phase 3.8 (Privacy Chains Family Decoder)

**Before** (lines 1127-1200):
```markdown
### 3.8: Privacy Chains Family Decoder (Week 9-10)

**Priority**: MEDIUM (Privacy features require special handling)
**Decoder**: `decoder-privacy-chains`
**Prerequisites**: Phase 3.0 (Privacy-Aware TxIR Extensions) must be complete

**Chains Supported**: Zcash, Aleo, Monero (limited)

Tasks:
- [ ] Create `decoder-privacy-chains` crate
- [ ] Implement Zcash decoder (transparent + shielded transactions)
  - [ ] Transparent transactions (reuse `BitcoinDecoder` base)
  - [ ] Shielded transactions (zk-SNARK components)
  - [ ] Viewing key decryption
```

**After** (detailed expansion):
```markdown
### 3.8: Privacy Chains Family Decoder (Weeks 9-11, 3 weeks)

**Priority**: HIGH (Privacy is core differentiator)
**Timeline**: 3 weeks (40-60 hours)
**Status**: Ready to implement (privacy infrastructure complete)
**Prerequisites**: ✅ Privacy infrastructure (privacy.rs complete), ✅ Bitcoin decoder

#### 3.8.1: Zcash Decoder (Weeks 9-11) ⭐ PRIMARY TARGET

**Decoder**: `decoder-zcash` (standalone crate)
**See**: `docs/ZCASH_INTEGRATION_PLAN.md` (comprehensive guide)

**Chains Supported**: Zcash mainnet (chain ID 133), Zcash testnet (chain ID 1)

**Transaction Types**:
- ✅ t→t (Transparent): Reuse BitcoinDecoder (~40% of txs)
- ✅ t→z (Shielding): Transparent → Shielded (~25% of txs)
- ✅ z→t (Deshielding): Shielded → Transparent (~20% of txs)
- ✅ z→z (Fully Shielded): Maximum privacy (~15% of txs)

**Protocol Versions**:
- ✅ Sapling (Version 4): Primary implementation (2018+)
- ✅ Orchard (Version 5): Latest protocol (2021+, NU5)
- ⏳ Sprout (Version 1-3): Legacy, optional (deprecated)

**Week 9 (Days 1-2): Transparent Transactions**
- [ ] Create `decoder-zcash` crate with proper structure
- [ ] Implement `ZcashChain` (ChainIdentity trait)
- [ ] Reuse Bitcoin parsing for transparent inputs/outputs
- [ ] Add Zcash-specific fields (version_group_id, expiry_height)
- [ ] Write 15+ tests with mainnet transparent transactions
- **Deliverable**: t→t transactions fully working

**Week 9 (Days 3-6): Sapling Shielded Transactions**
- [ ] Implement SpendDescription parsing (nullifiers, commitments, proofs)
- [ ] Implement OutputDescription parsing (encrypted notes, ephemeral keys)
- [ ] Parse value_balance (net transparent ↔ shielded flow)
- [ ] Populate PrivacyMetadata (HiddenSender, HiddenRecipient, HiddenAmount)
- [ ] Write 40+ tests (t→z, z→t, z→z, mixed)
- **Deliverable**: Full Sapling support (no viewing key decryption yet)

**Week 10 (Days 7-8): Viewing Key Decryption**
- [ ] Implement SaplingIncomingViewingKey support
- [ ] ECDH key agreement (ephemeral_key + IVK)
- [ ] ChaCha20-Poly1305 decryption for encrypted notes
- [ ] Parse decrypted note plaintext (amount, recipient, memo)
- [ ] Populate TxIR operations with decrypted data
- [ ] Write 15+ tests (successful decryption, wrong key, failures)
- [ ] Create examples showing VK usage
- **Deliverable**: Viewing key decryption working

**Week 11 (Days 9-11): Orchard Support**
- [ ] Implement Orchard ActionDescription parsing
- [ ] Parse Halo2 proof structures (different from Groth16)
- [ ] Implement Orchard viewing key decryption
- [ ] Support unified addresses (transparent + Sapling + Orchard)
- [ ] Write 15+ tests for Orchard transactions
- **Deliverable**: Full Orchard support (NU5+)

**Week 11 (Days 12-14): Testing & Integration**
- [ ] Property-based testing (proptest) - 20+ tests
- [ ] Fuzzing infrastructure (cargo-fuzz) - 5 fuzz targets
- [ ] Integration tests with 100+ real mainnet transactions
- [ ] Documentation and examples
- [ ] Performance benchmarking
- **Deliverable**: Production-ready Zcash decoder

**Cryptographic Dependencies** (airgapped via git subtree):
- [ ] Vendor `jubjub` elliptic curve library (Sapling)
- [ ] Vendor `bls12_381` curve library (Orchard)
- [ ] Vendor `blake2b_simd` hashing library
- All vendored via git subtree for verifiable, airgapped operation

**Test Coverage**:
- ✅ 60+ unit tests (every parsing function)
- ✅ 20+ property tests (safety invariants)
- ✅ 100+ integration tests (real mainnet transactions)
- ✅ 5 fuzz targets (adversarial inputs)
- ✅ Examples for all transaction types

**Success Criteria**:
- ✅ All 4 transaction types (t→t, t→z, z→t, z→z) supported
- ✅ Sapling and Orchard protocols fully implemented
- ✅ Viewing key decryption working for compliance use cases
- ✅ Privacy metadata accurately populated
- ✅ 100+ integration tests with real transactions
- ✅ Airgapped operation (all dependencies vendored)
- ✅ Documentation and examples complete

**Why Zcash First**:
- Most mature privacy protocol (7+ years in production)
- Largest privacy-focused blockchain by market cap
- Clear specification (ZIP documents)
- Reference implementation available for validation
- Viewing keys enable compliance/auditing use cases

**ROI**: HIGH
- **Impact**: Demonstrates privacy-aware decoding capability
- **Complexity**: Medium-high (zk-SNARKs, multiple protocols)
- **Timeline**: 3 weeks (focused implementation)
- **Dependencies**: Already available (privacy.rs complete)
- **Use Cases**: Forensics, compliance, auditing, analytics

#### 3.8.2: Aleo Decoder (Week 12-13) [OPTIONAL]

**Decoder**: `decoder-aleo`
**Status**: After Zcash complete

[... rest of Aleo plan ...]

#### 3.8.3: Monero Decoder (Week 14) [OPTIONAL]

**Decoder**: `decoder-monero`
**Status**: After Zcash + Aleo complete

[... rest of Monero plan ...]
```

### Chain Family Update

**File**: `crates/universal-decoder-core/src/chain_family.rs`

**Add**:
```rust
/// Privacy-focused chains with shielded transactions
Privacy,
```

**Update documentation**:
```rust
/// - `Privacy`: Zcash (shielded), Monero (RingCT), Aleo (Leo VM)
```

---

## Success Criteria

### Technical Completeness

- ✅ All 4 transaction types supported (t→t, t→z, z→t, z→z)
- ✅ Sapling protocol fully implemented (spend/output descriptions)
- ✅ Orchard protocol fully implemented (action descriptions)
- ✅ Viewing key decryption working (Sapling + Orchard)
- ✅ Privacy metadata accurately populated for all transaction types
- ✅ Canonical serialization (Borsh) for all Zcash transactions

### Testing Coverage

- ✅ 60+ unit tests (100% of parsing functions)
- ✅ 20+ property tests (safety invariants)
- ✅ 100+ integration tests (real mainnet transactions)
- ✅ 5 fuzz targets (decoder, Sapling, Orchard, viewing key)
- ✅ All tests passing with 0 warnings

### Documentation

- ✅ `decoder-zcash/README.md` - User guide
- ✅ `decoder-zcash/ARCHITECTURE.md` - Technical architecture
- ✅ `docs/ZCASH_INTEGRATION_PLAN.md` - This document
- ✅ Examples for all transaction types
- ✅ Viewing key usage examples

### Performance

- ✅ Transparent transactions: < 1ms decode time
- ✅ Sapling transactions: < 5ms decode time (without proof verification)
- ✅ Orchard transactions: < 5ms decode time (without proof verification)
- ✅ Viewing key decryption: < 10ms per output

### Airgapped Operation

- ✅ All cryptographic dependencies vendored via git subtree
- ✅ Zero runtime network dependencies
- ✅ All data embedded at compile time
- ✅ Verifiable supply chain (git commit audit trail)

---

## Next Steps

### Immediate Actions (Today)

1. **Update chain_family.rs** to add `ChainFamily::Privacy`
2. **Update ROADMAP.md** with detailed Zcash implementation plan (from this document)
3. **Create tracking issue** for Phase 3.8.1 (Zcash Decoder)
4. **Set up project branch**: `claude/zcash-integration-phase-3.8-<session-id>`

### Week 1 Start (Tomorrow)

1. **Create `decoder-zcash` crate** skeleton
2. **Implement transparent transaction** parsing (reuse Bitcoin)
3. **Write first 15 tests** for t→t transactions

### Ongoing

- Keep `docs/ZCASH_INTEGRATION_PLAN.md` updated as implementation progresses
- Use TodoWrite tool to track daily tasks
- Commit frequently with descriptive messages
- Run `cargo fmt && cargo clippy` before every commit

---

**Document Version**: 1.0.0
**Status**: Ready for implementation
**Approval**: Pending review
**Implementation Start**: TBD (after roadmap update)
