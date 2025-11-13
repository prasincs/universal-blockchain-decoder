# Privacy Roadmap: Anticipating Chain Privacy Features in TxIR

**Status**: Design Document
**Version**: 0.1.0
**Last Updated**: 2025-01-13
**Authors**: Universal Blockchain Decoder Team

---

## Executive Summary

This document outlines how TxIR (Transaction Intermediate Representation) can anticipate emerging privacy features in blockchain protocols—particularly Ethereum's privacy roadmap—without over-designing for hypothetical futures.

**Core Principle**: Use **trait-based extensibility** to create **privacy primitives** that can be composed, not exhaustive enums that try to predict every privacy mechanism.

**Concrete Examples**:
- ✅ **Ethereum Stealth Addresses** (2023-2025): EIP-5564 standard, address-per-transaction
- ✅ **Privacy Pools** (2024+): ZK proofs for compliant privacy (live on mainnet)
- ✅ **Account Abstraction + Privacy** (EIP-4337): Private bundled transactions
- ⏳ **Encrypted Mempools**: PBS (Proposer-Builder Separation) with private orderflow
- ❌ **Full Homomorphic Encryption**: Too speculative, no concrete standards

---

## Table of Contents

1. [The Challenge: Privacy vs Over-Design](#the-challenge)
2. [Privacy Primitive Taxonomy](#privacy-primitive-taxonomy)
3. [TxIR Privacy Extension Strategy](#txir-privacy-extension-strategy)
4. [Concrete Privacy Features (2024-2027)](#concrete-privacy-features)
5. [What NOT to Design For](#what-not-to-design-for)
6. [Implementation Phases](#implementation-phases)
7. [Verification Considerations](#verification-considerations)

---

## The Challenge: Privacy vs Over-Design

### Problem Statement

Blockchain privacy is evolving rapidly:

1. **Ethereum**: Stealth addresses, privacy pools, encrypted mempools (Vitalik's 2025 roadmap)
2. **Zcash/Aztec**: ZK-SNARK shielded transactions
3. **Monero**: Ring signatures, stealth addresses, confidential transactions
4. **Manta/Aleo**: Programmable privacy (ZK smart contracts)

**Question**: How can TxIR represent private transactions **without**:
- ❌ Enumerating every possible privacy mechanism (breaks open-closed principle)
- ❌ Over-designing for hypothetical futures (violates minimal TCB)
- ❌ Making privacy a special case (should be composable)

**Answer**: **Privacy as trait-based composition**, not hardcoded enums.

---

## Privacy Primitive Taxonomy

### Observable Privacy Patterns (Across Chains)

Through analysis of existing privacy protocols, we identify **5 fundamental privacy primitives**:

#### 1. **Hidden Sender** (Stealth Addresses, Ring Signatures)

**What's Hidden**: Transaction sender's identity
**What's Revealed**: Transaction occurred, amount (optionally), recipient

**Examples**:
- Ethereum stealth addresses (EIP-5564)
- Monero ring signatures (1-of-N signer anonymity)
- Tornado Cash deposit addresses

**TxIR Representation** (extensible):
```rust
pub struct PrivateAddress {
    /// The privacy mechanism used
    pub privacy_type: AddressPrivacyType,

    /// Ephemeral or stealth address bytes (what's on-chain)
    pub public_address: Vec<u8>,

    /// Optional: Viewing key for auditing
    pub viewing_hint: Option<Vec<u8>>,
}

pub enum AddressPrivacyType {
    Stealth {
        /// EIP-5564: stealth meta-address scheme
        scheme_id: u32,
    },
    RingSig {
        /// Number of possible signers (anonymity set)
        ring_size: usize,
    },
    /// Future mechanisms can be added without core changes
    Custom {
        mechanism_name: String,
        metadata: Vec<u8>,
    },
}
```

**Why This Works**:
- ✅ Extensible: New privacy mechanisms add variants, don't change core
- ✅ Auditable: Optional viewing hints enable compliance
- ✅ Verifiable: Clear distinction between public/private data

#### 2. **Hidden Recipient** (One-Time Addresses, Encrypted Outputs)

**What's Hidden**: Who receives funds
**What's Revealed**: Transaction occurred, outputs exist

**Examples**:
- Zcash shielded addresses
- Monero one-time addresses
- Aztec private NFT transfers

**TxIR Representation**:
```rust
pub struct PrivateOutput {
    /// Commitment to the output (on-chain)
    pub commitment: Vec<u8>,

    /// Note encryption scheme
    pub encryption_scheme: EncryptionScheme,

    /// Encrypted payload (only recipient can decrypt)
    pub encrypted_note: Vec<u8>,

    /// Optional: Amount commitment (for range proofs)
    pub amount_commitment: Option<Vec<u8>>,
}

pub enum EncryptionScheme {
    /// ChaCha20-Poly1305 (used in Zcash Sapling)
    ChaCha20Poly1305,
    /// AES-GCM with ECDH key agreement
    AesGcm { curve: String },
    /// Custom scheme
    Custom { name: String },
}
```

#### 3. **Hidden Amount** (Confidential Transactions, Pedersen Commitments)

**What's Hidden**: Transaction value
**What's Revealed**: Transaction occurred, participants (unless also hidden)

**Examples**:
- Monero confidential transactions (RingCT)
- Elements (Liquid) confidential assets
- Mimblewimble (Grin, Beam)

**TxIR Representation**:
```rust
pub struct ConfidentialAmount {
    /// Pedersen commitment: C = vG + rH
    pub commitment: Vec<u8>,

    /// Range proof (proves amount is positive without revealing value)
    pub range_proof: Option<Vec<u8>>,

    /// Proof system used
    pub proof_system: RangeProofSystem,
}

pub enum RangeProofSystem {
    /// Bulletproofs (logarithmic size, no trusted setup)
    Bulletproofs,
    /// Bulletproofs+ (improved efficiency)
    BulletproofsPlus,
    /// Borromean ring signatures (older)
    Borromean,
    Custom(String),
}
```

**Key Property**: Amounts are **commitments**, not plaintext values. Verifiers check:
```
∑ input_commitments = ∑ output_commitments + fee_commitment
```

#### 4. **Hidden Transaction Graph** (Privacy Pools, Mixers)

**What's Hidden**: Link between input and output addresses
**What's Revealed**: Set of inputs, set of outputs (but not mapping)

**Examples**:
- Privacy Pools (Ethereum, 2024+)
- Tornado Cash (deprecated due to sanctions)
- CoinJoin protocols (Bitcoin)

**TxIR Representation**:
```rust
pub struct PrivacyPool {
    /// Merkle root of anonymity set
    pub anonymity_set_root: Vec<u8>,

    /// Zero-knowledge proof of membership (without revealing which member)
    pub membership_proof: Vec<u8>,

    /// Optional: Association set proof (for regulatory compliance)
    pub compliance_proof: Option<ComplianceProof>,
}

/// Privacy Pools innovation: prove funds are NOT from illicit sources
pub struct ComplianceProof {
    /// Proof that deposit is in "clean" association set
    pub association_set_proof: Vec<u8>,

    /// Association set identifier (e.g., "non-sanctioned addresses")
    pub association_set_id: Vec<u8>,
}
```

**Why This Matters**: Privacy Pools (2024) are **live on Ethereum mainnet** with Vitalik as first user. This is **not hypothetical**.

#### 5. **Hidden Transaction Existence** (Stealth Payments, Encrypted Mempools)

**What's Hidden**: That a transaction occurred at all
**What's Revealed**: Only to sender/recipient (or validators with decryption key)

**Examples**:
- Encrypted mempools (MEV protection)
- Flashbots Protect (private transactions)
- Taiga (private DEX swaps)

**TxIR Representation**:
```rust
pub struct EncryptedTransaction {
    /// Encrypted transaction payload (only validators can decrypt)
    pub encrypted_payload: Vec<u8>,

    /// Public proof that encrypted tx is valid (without revealing contents)
    pub validity_proof: Vec<u8>,

    /// Decryption timeline (e.g., "after block inclusion")
    pub decryption_policy: DecryptionPolicy,
}

pub enum DecryptionPolicy {
    /// Decrypt immediately after inclusion
    PostInclusion,
    /// Decrypt after N blocks
    DelayedBy(u64),
    /// Never decrypt (full privacy)
    Never,
}
```

**Status**: Encrypted mempools are **production** on Ethereum (Flashbots, MEV-Boost).

---

## TxIR Privacy Extension Strategy

### Core Principle: Composition, Not Enumeration

**Anti-Pattern** (Closed, Breaks Minimal TCB):
```rust
// ❌ BAD: Enumerates all privacy mechanisms (requires core changes)
pub enum PrivacyType {
    None,
    StealthAddress,
    RingSignature,
    ConfidentialTransaction,
    PrivacyPool,
    ZkSnark,
    ZkStark,
    // ... endless additions, bloats core
}
```

**Correct Pattern** (Open, Trait-Based):
```rust
// ✅ GOOD: Trait-based privacy capabilities
pub trait PrivacyCapable {
    /// Returns privacy primitives used in this transaction
    fn privacy_features(&self) -> Vec<PrivacyFeature>;

    /// Can this transaction be fully decoded without private keys?
    fn is_fully_observable(&self) -> bool;

    /// Optional: Viewing key for auditing
    fn viewing_key_type(&self) -> Option<ViewingKeyType>;
}

/// Privacy feature descriptor (open for extension)
pub enum PrivacyFeature {
    HiddenSender(PrivateAddress),
    HiddenRecipient(PrivateOutput),
    HiddenAmount(ConfidentialAmount),
    HiddenGraph(PrivacyPool),
    HiddenExistence(EncryptedTransaction),
    /// Future mechanisms extend here, no core changes
    Custom {
        name: String,
        description: String,
        metadata: Vec<u8>,
    },
}
```

### Integration with TxIR

**Current TxIR Structure** (from `crates/universal-decoder-core/src/ir.rs:16`):
```rust
pub struct TxIR<'a, const V: u8> {
    pub chain: ChainRef,
    pub metadata: TxMetadata,
    pub authorization: AuthorizationPackage,
    pub operations: Vec<Operation>,
    pub state_deltas: StateDeltas,
    _phantom: PhantomData<&'a [u8]>,
}
```

**Extended for Privacy** (additive, no breaking changes):
```rust
pub struct TxIR<'a, const V: u8> {
    pub chain: ChainRef,
    pub metadata: TxMetadata,
    pub authorization: AuthorizationPackage,
    pub operations: Vec<Operation>,
    pub state_deltas: StateDeltas,

    // ✅ NEW: Optional privacy features (None for transparent chains)
    pub privacy: Option<PrivacyMetadata>,

    _phantom: PhantomData<&'a [u8]>,
}

pub struct PrivacyMetadata {
    /// List of privacy primitives used
    pub features: Vec<PrivacyFeature>,

    /// Overall observability level
    pub observability: ObservabilityLevel,

    /// Optional viewing key for auditing
    pub viewing_key: Option<ViewingKey>,
}

pub enum ObservabilityLevel {
    /// Fully transparent (Bitcoin, Ethereum legacy)
    FullyObservable,
    /// Partially private (stealth addresses, confidential amounts)
    PartiallyObservable,
    /// Fully private (Monero, Zcash shielded)
    FullyPrivate,
}
```

**Why This Design Works**:
1. ✅ **Non-Breaking**: `privacy: Option<_>` is backward compatible (None for non-privacy chains)
2. ✅ **Extensible**: New privacy features add to `PrivacyFeature` enum, not core
3. ✅ **Composable**: Transactions can use multiple privacy primitives simultaneously
4. ✅ **Auditable**: `viewing_key` enables regulatory compliance without breaking privacy
5. ✅ **Minimal TCB**: Privacy logic in decoders, not core (core just provides types)

---

## Concrete Privacy Features (2024-2027)

These are **not hypothetical**—they have specifications, implementations, or live deployments.

### 1. Ethereum Stealth Addresses (EIP-5564) ⚡ HIGH PRIORITY

**Status**: Specification finalized (2023), implementations emerging (2024-2025)
**Timeline**: Mainnet adoption expected 2025-2026
**Vitalik Quote** (Jan 2023): "Stealth addresses would give Ethereum users privacy by default"

**How It Works**:
1. Alice publishes a **stealth meta-address** (permanent, on-chain)
2. Bob generates a **one-time stealth address** from Alice's meta-address
3. Bob sends ETH to stealth address (on-chain transaction is public)
4. Only Alice can **scan** for stealth payments using her private key
5. Alice can spend from stealth address without linking to her identity

**TxIR Representation**:
```rust
impl Operation {
    Transfer(Transfer {
        from: Address::standard(sender_address),
        to: Address::stealth(StealthAddress {
            ephemeral_pubkey: bob_generated_pubkey, // On-chain
            stealth_address: one_time_address,      // On-chain
            scheme_id: 5564,                        // EIP-5564
        }),
        amount: Amount::new(1_000_000, 18), // Still visible (amount privacy is separate)
        asset: AssetId::Native,
    })
}

// Decoder responsibility (NOT core):
impl EthereumDecoder {
    fn decode_stealth_transfer(&self, tx_bytes: &[u8]) -> Result<TxIR> {
        // Parse ephemeral pubkey from transaction calldata
        // Construct PrivacyFeature::HiddenRecipient
        // Set observability to PartiallyObservable
    }
}
```

**What TxIR Anticipates**:
- ✅ `Address` type can represent stealth addresses (add `AddressType` enum)
- ✅ `PrivacyFeature::HiddenRecipient` captures the mechanism
- ✅ Decoders handle parsing (core just provides types)

**What TxIR Does NOT Design For**:
- ❌ Specific ECDH curve choices (decoders decide)
- ❌ View key derivation paths (implementation detail)
- ❌ Stealth address registry contracts (chain-specific)

### 2. Privacy Pools (2024) ⚡ CRITICAL (Already Live!)

**Status**: **Live on Ethereum Mainnet** (2024)
**First User**: Vitalik Buterin (deposited 1 ETH)
**Innovation**: Compliant privacy using **association sets**

**How It Works**:
1. User deposits ETH into Privacy Pool contract (creates note)
2. Note is added to Merkle tree (anonymity set)
3. User generates ZK proof: "My note is in tree AND in association set X"
4. Association set X = "addresses not linked to sanctioned entities"
5. User withdraws to new address (compliance proven, but specific source hidden)

**TxIR Representation**:
```rust
impl Operation {
    // Deposit into privacy pool
    ContractCall(ContractCall {
        contract: Address::from_hex("0x...PrivacyPool"),
        method: vec![0xa1, 0xb2, 0xc3, 0xd4], // deposit() selector
        data: commitment_bytes,                // Pedersen commitment
        value: Some(Amount::new(1_000_000_000_000_000_000, 18)), // 1 ETH
        resource_limits: gas_limits,
    })
}

// Privacy metadata:
pub privacy: Some(PrivacyMetadata {
    features: vec![
        PrivacyFeature::HiddenGraph(PrivacyPool {
            anonymity_set_root: merkle_root,
            membership_proof: zk_snark_proof,
            compliance_proof: Some(ComplianceProof {
                association_set_proof: association_proof,
                association_set_id: b"non-sanctioned-v1".to_vec(),
            }),
        }),
    ],
    observability: ObservabilityLevel::PartiallyObservable,
    viewing_key: None, // Privacy Pools don't have viewing keys
})
```

**What TxIR Anticipates**:
- ✅ `PrivacyPool` struct with compliance proofs
- ✅ `ComplianceProof` for regulatory-friendly privacy
- ✅ ZK proof representation (opaque bytes, decoder interprets)

**What TxIR Does NOT Design For**:
- ❌ Specific ZK proof systems (Groth16, PLONK, etc.)—decoder concern
- ❌ Association set construction algorithms—chain-specific
- ❌ Proof verification logic—decoders validate, core just stores

### 3. Account Abstraction + Privacy (EIP-4337 + Stealth) 🔄 EMERGING

**Status**: Account Abstraction live (2023), privacy integration starting (2024-2025)
**Idea**: Bundled transactions can include stealth address creation

**How It Works**:
1. User submits `UserOperation` to bundler (not yet on-chain)
2. Bundler packages multiple UserOps into single transaction
3. UserOp can include: "Create stealth address, transfer funds to it"
4. On-chain: Only bundled transaction is visible, not individual UserOps

**TxIR Representation**:
```rust
// TxIR represents the BUNDLED transaction (what's on-chain)
pub operations: vec![
    Operation::ContractCall(ContractCall {
        contract: Address::from_hex("0x...EntryPoint"),
        method: vec![0x1f, 0xad, 0x94, 0x8c], // handleOps() selector
        data: bundled_user_ops_calldata,      // Multiple UserOps encoded
        // ...
    }),
],

// Privacy metadata indicates UserOps contain private operations
pub privacy: Some(PrivacyMetadata {
    features: vec![
        PrivacyFeature::Custom {
            name: "AccountAbstraction-Privacy".to_string(),
            description: "Bundled UserOps with stealth addresses".to_string(),
            metadata: user_op_hashes,
        },
    ],
    observability: ObservabilityLevel::PartiallyObservable,
    // ...
})
```

**What TxIR Anticipates**:
- ✅ `PrivacyFeature::Custom` for novel combinations
- ✅ Nested transaction structure (bundled UserOps)

**What TxIR Does NOT Design For**:
- ❌ UserOperation parsing (decoder responsibility)
- ❌ Bundler selection logic (off-chain)

### 4. Encrypted Mempools (MEV Protection) 🔄 IN PRODUCTION

**Status**: **Live** (Flashbots Protect, MEV-Boost+)
**Use Case**: Hide transaction from public mempool until inclusion

**How It Works**:
1. User encrypts transaction, submits to private relay
2. Only block builder can decrypt (using threshold encryption)
3. Builder includes transaction in block
4. Transaction revealed after block finality

**TxIR Representation**:
```rust
// IMPORTANT: TxIR represents DECRYPTED transaction (post-inclusion)
// Encrypted payload is NOT part of TxIR (can't decode what's encrypted)

// Instead, decoders mark transactions that WERE encrypted:
pub metadata: TxMetadata {
    // ...
    extra: json!({
        "was_private_mempool": true,
        "relay": "flashbots-protect",
        "decryption_block": 19123456,
    }).to_string(),
}

// OR: Privacy metadata
pub privacy: Some(PrivacyMetadata {
    features: vec![
        PrivacyFeature::HiddenExistence(EncryptedTransaction {
            encrypted_payload: vec![], // Not available (already decrypted)
            validity_proof: vec![],    // Not available
            decryption_policy: DecryptionPolicy::PostInclusion,
        }),
    ],
    observability: ObservabilityLevel::FullyObservable, // Observable AFTER inclusion
    // ...
})
```

**What TxIR Anticipates**:
- ✅ `PrivacyFeature::HiddenExistence` for mempool privacy
- ✅ Distinction between "hidden before inclusion" and "hidden forever"

**What TxIR Does NOT Design For**:
- ❌ Encryption algorithms (off-chain, pre-inclusion)
- ❌ Relay selection logic (user preference)
- ❌ Builder decryption keys (infrastructure concern)

---

## What NOT to Design For

### Hypothetical Privacy Mechanisms (No Concrete Plans)

#### ❌ Full Homomorphic Encryption (FHE) for Smart Contracts

**Why NOT Anticipate**:
- No EVM-compatible FHE standard
- Computational cost is prohibitive (1000x+ overhead)
- No major chain has concrete implementation timeline
- Research stage, not engineering stage

**If It Happens**: Use `PrivacyFeature::Custom` (no core changes needed)

#### ❌ Post-Quantum Privacy Protocols

**Why NOT Anticipate**:
- Post-quantum cryptography is emerging, but privacy protocols are far future
- Standards (NIST PQC) are for signatures/encryption, not privacy primitives
- Premature to design TxIR types for this

**If It Happens**: Extend `SignatureScheme` and `EncryptionScheme` enums (minor change)

#### ❌ Multi-Chain Private Transfers (Cross-Chain Privacy)

**Why NOT Anticipate**:
- No production cross-chain privacy protocol (bridges are transparent)
- Requires consensus across chains (unlikely)
- Complexity is extreme (each chain has different privacy model)

**If It Happens**: Each chain's transaction is separate TxIR, link via metadata

#### ❌ AI-Assisted Transaction Obfuscation

**Why NOT Anticipate**:
- Speculative technology
- Not a protocol-level feature
- Would be off-chain preprocessing

**If It Happens**: Doesn't affect TxIR (obfuscation is pre-submission)

### Red Flags for Over-Design

When evaluating new privacy features, ask:

1. **Is there a specification?** (EIP, ZIP, SNIP, etc.)
   - ✅ YES → Consider anticipating
   - ❌ NO → Use `Custom` variant

2. **Is there an implementation?** (Testnet, mainnet)
   - ✅ YES → Priority HIGH
   - ❌ NO → Defer until concrete

3. **Is it blockchain-level or application-level?**
   - ✅ Blockchain → TxIR concern
   - ❌ Application → Decoder/hook concern

4. **Can it be represented with existing primitives?**
   - ✅ YES → Use composition
   - ❌ NO → Add minimal extension

5. **Does it require core changes or just decoder changes?**
   - ✅ Decoder only → Perfect (minimal TCB preserved)
   - ❌ Core → Scrutinize heavily

---

## Implementation Phases

### Phase 3.5: Privacy Primitive Support (Month 5-6)

**Prerequisites**:
- ✅ Phase 3 complete (chain family decoders)
- ✅ Ethereum decoder complete
- ✅ Canonical serialization verified

**Timeline**: 6-8 weeks

#### Week 1-2: Core Privacy Types

**Tasks**:
- [ ] Add `PrivacyMetadata` to TxIR (optional field)
- [ ] Define `PrivacyFeature` enum with 5 primitive types
- [ ] Define `ObservabilityLevel` enum
- [ ] Add `ViewingKey` type (opaque bytes + metadata)
- [ ] Add `AddressPrivacyType` enum to `Address` type
- [ ] Update `Amount` to support `ConfidentialAmount` variant
- [ ] Write comprehensive unit tests (20+ tests)

**Deliverables**:
- `crates/universal-decoder-core/src/privacy.rs` (~400 LOC)
- Updated `ir.rs` with privacy field (~50 LOC change)
- Tests for privacy type creation and serialization

**Validation**:
- ✅ Backward compatible (privacy is optional)
- ✅ Zero core logic (just types)
- ✅ Canonical serialization works with privacy fields
- ✅ All existing tests pass

#### Week 3-4: Ethereum Stealth Address Support

**Tasks**:
- [ ] Extend `EthereumDecoder` to detect stealth addresses
- [ ] Parse EIP-5564 stealth meta-address announcements
- [ ] Detect stealth transfers (ephemeral pubkey in calldata)
- [ ] Construct `PrivacyFeature::HiddenRecipient` from transaction
- [ ] Set `ObservabilityLevel::PartiallyObservable`
- [ ] Integration tests with stealth address test vectors
- [ ] Property tests (stealth detection never panics)

**Deliverables**:
- `crates/decoder-ethereum/src/stealth.rs` (~300 LOC)
- Integration tests with real stealth transactions (~10 fixtures)
- Documentation: "Ethereum Privacy Support"

**Validation**:
- ✅ Detects EIP-5564 stealth addresses in calldata
- ✅ Correctly parses ephemeral pubkey
- ✅ Integration tests pass with real Ethereum stealth transactions
- ✅ No false positives (normal transfers not marked as stealth)

#### Week 5-6: Privacy Pools Support

**Tasks**:
- [ ] Extend `EthereumDecoder` to detect Privacy Pool contracts
- [ ] Parse deposit/withdrawal events
- [ ] Extract ZK proof from calldata
- [ ] Detect compliance proofs (association set membership)
- [ ] Construct `PrivacyFeature::HiddenGraph`
- [ ] Integration tests with Privacy Pool transactions
- [ ] Document Privacy Pool support

**Deliverables**:
- `crates/decoder-ethereum/src/privacy_pools.rs` (~350 LOC)
- Integration tests with real Privacy Pool transactions (~5 fixtures)
- Support for multiple Privacy Pool implementations

**Validation**:
- ✅ Detects Privacy Pool deposits/withdrawals
- ✅ Extracts ZK proofs and compliance proofs
- ✅ Integration tests with Vitalik's first Privacy Pool transaction
- ✅ Correctly handles different Privacy Pool contracts

#### Week 7-8: Documentation & Examples

**Tasks**:
- [ ] Update `docs/ARCHITECTURE.md` with privacy section
- [ ] Create `docs/PRIVACY_SUPPORT.md` (user guide)
- [ ] Add examples: "Decoding Ethereum stealth transfers"
- [ ] Add examples: "Analyzing Privacy Pool usage"
- [ ] Update README with privacy feature list
- [ ] Blog post: "Privacy-Preserving Transaction Decoding"

**Deliverables**:
- Comprehensive privacy documentation
- Working examples for all supported privacy features
- Blog post or article

### Phase 4.5: Privacy Verification (Month 7-8) [OPTIONAL]

**Goal**: Formally verify privacy metadata handling

**Verification Targets**:

**VT-30: Privacy Metadata Consistency**
- [ ] VT-30.1: `privacy.is_some()` ⟹ `features.len() > 0`
- [ ] VT-30.2: `ObservabilityLevel` consistent with features
- [ ] VT-30.3: Privacy serialization deterministic

**VT-31: Viewing Key Safety**
- [ ] VT-31.1: Viewing keys never panicking on malformed data
- [ ] VT-31.2: Viewing key presence doesn't affect tx hash

**Effort**: 2-3 weeks (depends on Verus maturity)

---

## Verification Considerations

### Privacy-Specific Properties to Verify

#### 1. Canonical Serialization with Privacy

**Property**: Privacy fields must be canonically serializable

```text
∀ tx_ir with privacy metadata,
  serialize(deserialize(serialize(tx_ir))) = serialize(tx_ir)
```

**Why Critical**: Privacy features must not break canonical hashing

**Verification Target**: VT-2 (extend existing canonicalization proofs)

#### 2. Privacy Metadata Consistency

**Property**: Observability level matches features

```text
∀ tx_ir,
  tx_ir.privacy.observability == FullyObservable ⟹ tx_ir.privacy.features.is_empty()
  tx_ir.privacy.observability == FullyPrivate ⟹ ∃ feature in HiddenSender | HiddenRecipient | HiddenAmount
```

**Why Critical**: Prevent inconsistent privacy state

**Verification Target**: VT-30 (new)

#### 3. Viewing Key Isolation

**Property**: Viewing keys don't affect transaction hash

```text
∀ tx_ir_1, tx_ir_2 where tx_ir_1 differs from tx_ir_2 only in viewing_key,
  canonical_hash(tx_ir_1) == canonical_hash(tx_ir_2)
```

**Why Critical**: Viewing keys are for audit, not consensus

**Verification Target**: VT-31 (new)

---

## Conclusion

### Summary of Approach

✅ **DO Anticipate**:
- Stealth addresses (EIP-5564): Concrete spec, emerging implementations
- Privacy Pools: **Live on mainnet** with Vitalik's endorsement
- Encrypted mempools: **In production** (Flashbots)
- Account abstraction + privacy: Natural extension of EIP-4337

✅ **DO Design**:
- Privacy primitives (5 fundamental patterns)
- Trait-based extensibility (`PrivacyFeature::Custom`)
- Optional privacy metadata (backward compatible)
- Observability levels (FullyObservable → FullyPrivate spectrum)

❌ **DON'T Design For**:
- Hypothetical FHE smart contracts (no concrete plans)
- Post-quantum privacy protocols (too speculative)
- Cross-chain private transfers (no standard)
- AI-assisted obfuscation (application-level)

### Success Criteria

**Phase 3.5 Complete When**:
- ✅ TxIR has optional `privacy` field (backward compatible)
- ✅ 5 privacy primitives defined (HiddenSender, HiddenRecipient, HiddenAmount, HiddenGraph, HiddenExistence)
- ✅ Ethereum decoder supports stealth addresses (EIP-5564)
- ✅ Ethereum decoder supports Privacy Pools
- ✅ Integration tests with real privacy transactions
- ✅ Documentation complete

**Verification**:
- ✅ Core TCB unchanged (privacy is decoder concern)
- ✅ Canonical serialization works with privacy
- ✅ All existing tests pass (backward compatibility)
- ✅ No over-design (no FHE, no PQ, no hypotheticals)

### Future Work (Post-v1.0)

When new privacy features emerge:
1. Evaluate against "Red Flags for Over-Design" checklist
2. If concrete specification exists, add to `PrivacyFeature` enum
3. If truly novel, use `PrivacyFeature::Custom` temporarily
4. Update decoders (NOT core) to parse new features
5. Add integration tests with real transactions
6. Document in `docs/PRIVACY_SUPPORT.md`

**Core remains minimal, decoders handle complexity.**

---

## References

1. **Vitalik Buterin**: "An incomplete guide to stealth addresses" (Jan 2023)
   https://vitalik.eth.limo/general/2023/01/20/stealth.html

2. **EIP-5564**: Stealth Addresses
   https://eips.ethereum.org/EIPS/eip-5564

3. **Privacy Pools Protocol** (2024)
   https://medium.com/@gokun4621/privacy-pools-2a904dfee520

4. **Ethereum Privacy Roadmap** (2025)
   Multiple sources via Vitalik's posts and Ethereum Foundation announcements

5. **Monero Research Lab**: RingCT and Stealth Addresses
   https://www.getmonero.org/resources/research-lab/

6. **Zcash Protocol Specification**: Sapling and Orchard
   https://zips.z.cash/protocol/protocol.pdf

7. **Bulletproofs**: Short Proofs for Confidential Transactions
   https://eprint.iacr.org/2017/1066

8. **Flashbots Documentation**: MEV-Boost and Private Transactions
   https://docs.flashbots.net/

---

**Last Updated**: 2025-01-13
**Version**: 0.1.0
**Status**: Living Document - Update as privacy features evolve
