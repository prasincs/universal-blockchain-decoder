# Emerging Blockchain Models & Future Architecture Considerations

**Status**: Research & Planning
**Created**: 2025-11-18
**Purpose**: Document experimental blockchain designs that may require architecture extensions

---

## Overview

This document catalogs emerging blockchain architectures that could challenge or extend the current TxIR pattern. These are **future considerations** - not immediate implementation priorities.

**Current Architecture Coverage**: The existing TxIR pattern successfully handles:
- ✅ UTXO models (Bitcoin, Cardano, Zcash)
- ✅ Account models (Ethereum, Cosmos, Polkadot)
- ✅ Instruction models (Solana, Sui, Aptos)
- ✅ Privacy models (Zcash shielded, Monero)
- ✅ Actor models (Sui objects, Aptos objects)

**Estimated Coverage**: ~80% of current production blockchains

---

## Category 1: Intent-Based Architectures 🔴

**Risk Level**: HIGH - Requires fundamental pattern changes
**Timeline**: 2025-2026 (research stage, limited production use)
**When to Revisit**: When Anoma/Namada reach mainnet with significant adoption

### What They Are

Transactions describe **desired outcomes** (intents) rather than explicit operations. Solvers determine execution paths.

### Examples

| Project | Status | Description |
|---------|--------|-------------|
| **Anoma/Namada** | Testnet | Intent-centric architecture with privacy |
| **Essential** | Development | Intent-based rollup |
| **CoW Protocol** | Production | Batch intent matching for DEX |
| **SUAVE (Flashbots)** | Research | MEV-aware intent execution |
| **UniswapX** | Production | Intent-based swaps |

### Why It Challenges Current Pattern

**Current TxIR Assumption**: Transactions contain explicit operations

```rust
// Current model (operational)
pub struct TxIR<'a, const V: u8> {
    pub operations: Vec<Operation>,  // Explicit: "transfer 10 ETH to 0x123"
    pub state_deltas: StateDeltas,   // Deterministic state changes
    // ...
}
```

**Intent-Based Model**: Transactions contain goals + constraints

```rust
// Intent model (not yet supported)
pub struct IntentTransaction {
    pub intent: Intent,  // "I want 100 USDC for my ETH"
    pub constraints: Constraints,  // "Within 1 hour, slippage < 0.5%"
    pub solver_signature: Vec<u8>,  // Who fulfilled the intent
    pub realized_operations: Option<Vec<Operation>>,  // How it was executed
}

pub struct Intent {
    pub desired_state: StateTransition,  // "End state: +100 USDC, -X ETH"
    pub max_cost: u128,
    pub deadline: u64,
}
```

### Key Differences

| Aspect | Current (Operational) | Intent-Based |
|--------|----------------------|--------------|
| **Execution** | Deterministic | Solver-determined |
| **Operations** | Known at decode time | Known after execution |
| **Validation** | Pre-execution | Post-execution (via proof) |
| **State Deltas** | Explicit in tx | Computed by solver |
| **Authorization** | User signs operations | User signs intent |

### Proposed Future Extension

```rust
pub enum TransactionSemantics<'a> {
    /// Traditional operational model
    Operational {
        operations: Vec<Operation>,
    },

    /// Intent-based model (future)
    IntentBased {
        intent: Intent,
        constraints: Constraints,
        realized_ops: Option<Vec<Operation>>,  // Available after execution
        solver_proof: Option<Vec<u8>>,
    },
}

pub struct TxIR<'a, const V: u8> {
    pub chain: ChainRef,
    pub metadata: TxMetadata,
    pub authorization: AuthorizationPackage,
    pub semantics: TransactionSemantics<'a>,  // ← New field
    pub state_deltas: StateDeltas,
    pub privacy: Option<PrivacyMetadata>,
    _phantom: PhantomData<&'a [u8]>,
}
```

### When to Implement

- ⏳ **Not Now**: Intent-based chains are research/testnet stage
- ⏰ **Consider When**: Anoma mainnet launches with >10k daily active users
- 🎯 **Trigger**: 3+ production intent-based chains request decoder support

---

## Category 2: DAG-Based Consensus 🟡

**Risk Level**: MEDIUM - Needs metadata extensions
**Timeline**: Already in production (IOTA, Hedera, Kaspa)
**When to Revisit**: When DAG chains represent >5% market cap or user requests

### What They Are

Transactions form Directed Acyclic Graph (DAG) instead of linear chain. Each transaction validates multiple previous transactions.

### Examples

| Project | Status | Model | Notes |
|---------|--------|-------|-------|
| **IOTA (Tangle)** | Production | Each tx validates 2+ parents | IoT-focused |
| **Hedera Hashgraph** | Production | Gossip + virtual voting | Enterprise adoption |
| **Nano** | Production | Block-lattice (1 chain/account) | Feeless transfers |
| **Kaspa** | Production | BlockDAG with parallel blocks | High throughput |
| **Constellation** | Development | DAG with microservices | Data layer |

### Why It Challenges Current Pattern

**Current Assumption**: Transaction is atomic, references single blockchain state

```rust
pub struct TxMetadata {
    pub tx_hash: Option<Vec<u8>>,
    pub block_hash: Option<Vec<u8>>,  // Single parent block
    pub timestamp: Option<u64>,
    // ...
}
```

**DAG Reality**: Transaction references multiple parent transactions

```rust
// IOTA Tangle structure
pub struct TangleTransaction {
    pub transaction: Transaction,
    pub trunk_hash: [u8; 32],   // Parent 1
    pub branch_hash: [u8; 32],  // Parent 2
    pub weight_magnitude: u8,   // Proof of work
}

// Kaspa BlockDAG
pub struct KaspaBlock {
    pub transactions: Vec<Transaction>,
    pub parent_hashes: Vec<[u8; 32]>,  // Multiple parents!
    pub blue_score: u64,  // Consensus ordering
}
```

### Key Differences

| Aspect | Linear Chain | DAG |
|--------|-------------|-----|
| **Parent Reference** | Single block | Multiple transactions/blocks |
| **Consensus** | Block-level | Transaction-level or hybrid |
| **Ordering** | Explicit (block height) | Computed (topological sort) |
| **Finality** | Confirmation depth | Cumulative weight/virtual voting |

### Proposed Future Extension

**Option 1: Extend TxMetadata**

```rust
pub struct TxMetadata {
    // ... existing fields
    pub block_hash: Option<Vec<u8>>,  // Keep for compatibility

    /// DAG-specific metadata (future)
    pub dag_metadata: Option<DagMetadata>,
}

pub struct DagMetadata {
    pub parent_refs: Vec<[u8; 32]>,  // 2+ parent transaction hashes
    pub weight: u64,  // Cumulative weight in DAG
    pub confirmation_confidence: Option<f64>,  // Probabilistic finality
    pub topological_order: Option<u64>,  // Position in DAG
}
```

**Option 2: Chain-Specific Metadata** (preferred for now)

```rust
// Keep in decoder-specific types, map to standard TxMetadata
impl Canonicalizer for IotaTransaction {
    fn canonicalize(&self) -> Result<TxIR> {
        Ok(TxIR {
            metadata: TxMetadata {
                // Encode DAG refs in metadata JSON
                metadata: Some(json!({
                    "dag_parents": [self.trunk_hash, self.branch_hash],
                    "weight": self.weight_magnitude
                }).to_string()),
                // ...
            },
            // ...
        })
    }
}
```

### When to Implement

- ✅ **Now**: Use chain-specific metadata field (JSON) for DAG info
- ⏰ **Later**: Add `DagMetadata` struct when 3+ DAG chains are supported
- 🎯 **Trigger**: User requests explicit DAG analysis/visualization tools

---

## Category 3: Parallel Execution Models 🔴

**Risk Level**: HIGH - Execution semantics differ fundamentally
**Timeline**: Production (Aptos, Sui), growing adoption
**When to Revisit**: Phase 3.5+ when implementing Sui/Aptos decoders

### What They Are

Transactions declare dependencies, allowing parallel execution. Execution order determined at runtime.

### Examples

| Project | Status | Parallelism Model | Notes |
|---------|--------|-------------------|-------|
| **Aptos (Block-STM)** | Production | Optimistic parallel exec | Re-execution on conflicts |
| **Sui** | Production | Object-centric parallelism | Tx declares object deps |
| **Fuel Labs** | Testnet | UTXO-based parallelism | Predicates + strict state access |
| **Linera** | Development | Microchains | Cross-chain coordination |
| **Solana** | Production | Account-based parallelism | Read/write locks |

### Why It Challenges Current Pattern

**Current Assumption**: Sequential execution, deterministic order

```rust
pub struct TxIR {
    pub operations: Vec<Operation>,  // Implicit: execute in order
    pub authorization: AuthorizationPackage,  // Single signature bundle
}
```

**Parallel Reality** (Sui Programmable Transactions):

```rust
pub struct SuiProgrammableTransaction {
    pub inputs: Vec<Input>,  // Shared objects, owned objects
    pub commands: Vec<Command>,  // Can execute in parallel!
}

pub struct Command {
    pub command_type: CommandType,
    pub arguments: Vec<Argument>,
    pub type_arguments: Vec<TypeTag>,
}

// Execution order determined by:
// 1. Data dependencies (which commands need results from others)
// 2. Object ownership (owned objects can run in parallel)
// 3. Shared objects (require consensus, sequential)
```

**Aptos Block-STM**: Optimistic parallel execution

```rust
// Transactions in block execute in parallel
// If conflict detected (read/write to same resource):
//   → Abort conflicting tx
//   → Re-execute sequentially
//   → Update dependency graph
```

### Key Differences

| Aspect | Sequential | Parallel |
|--------|-----------|----------|
| **Execution Order** | Explicit (operation order) | Runtime-determined |
| **Dependencies** | Implicit (all ops sequential) | Explicit (data deps) |
| **Consensus** | Single round | Multi-round (shared objects) |
| **Validation** | Pre-execution | During execution |
| **State Access** | Unspecified | Declared (read/write sets) |

### Proposed Future Extension

**Option 1: Execution Metadata** (lightweight)

```rust
pub struct TxMetadata {
    // ... existing fields

    /// Parallel execution metadata (future)
    pub execution_model: Option<ExecutionModel>,
}

pub enum ExecutionModel {
    Sequential,
    Parallel {
        /// Operation dependency graph: (from_idx, to_idx)
        dependencies: Vec<(usize, usize)>,

        /// Realized execution order (if known post-execution)
        execution_order: Option<Vec<usize>>,

        /// Objects accessed (for object-centric models)
        read_set: Option<Vec<Vec<u8>>>,
        write_set: Option<Vec<Vec<u8>>>,
    },
}
```

**Option 2: Operation-Level Dependencies** (more structured)

```rust
pub struct Operation {
    pub op_type: OperationType,
    pub from: Option<Vec<u8>>,
    pub to: Option<Vec<u8>>,
    pub amount: Option<u128>,
    pub data: Option<Vec<u8>>,

    /// Parallel execution metadata (future)
    pub depends_on: Option<Vec<usize>>,  // Which operation indices must run first
    pub access_mode: Option<AccessMode>,  // Read vs Write
}

pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}
```

### When to Implement

- ⏰ **Phase 3.5+**: When implementing Sui/Aptos decoders
- 🎯 **Design Pattern**: Add execution metadata to TxMetadata (lightweight, non-breaking)
- ⚠️ **Caveat**: Execution order may not be known at decode time (require post-execution analysis)

---

## Category 4: Modular Blockchain Architectures 🟠

**Risk Level**: MEDIUM - Cross-layer references need modeling
**Timeline**: Production (Celestia, Fuel), active development
**When to Revisit**: Phase 3.10+ (after initial modular chain requests)

### What They Are

Blockchains separate consensus, execution, and data availability into independent layers.

### Examples

| Project | Status | Role | Notes |
|---------|--------|------|-------|
| **Celestia** | Production | Data Availability | No execution, just blob storage |
| **Fuel** | Testnet | Execution Layer | Uses Celestia for DA |
| **Eigenlayer** | Production | Shared Security | Restaking for custom AVSs |
| **Dymension** | Production | RollApp Hub | Cosmos SDK rollup-as-a-service |
| **Avail** | Testnet | Data Availability | Polkadot-based DA layer |

### Why It Challenges Current Pattern

**Current Assumption**: Monolithic transaction (consensus + execution + DA in one)

```rust
pub struct TxIR {
    pub chain: ChainRef,  // Single chain reference
    // All operations happen on this chain
}
```

**Modular Reality**: Transaction spans multiple layers

```rust
// Celestia: Just data availability (no execution)
pub struct CelestiaBlob {
    pub namespace_id: [u8; 29],  // Identifies rollup
    pub data: Vec<u8>,  // Rollup transaction batch
    pub share_commitment: [u8; 32],  // Merkle commitment
    // ❌ No execution, no operations!
}

// Fuel: Execution references Celestia
pub struct FuelTransaction {
    pub script_or_predicate: ScriptData,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,

    // Where is state stored?
    pub da_layer: DaReference,  // Points to Celestia blob
}

pub struct DaReference {
    pub chain: ChainId,  // "celestia"
    pub blob_id: [u8; 32],
    pub height: u64,
}
```

### Key Differences

| Aspect | Monolithic | Modular |
|--------|-----------|---------|
| **Transaction Scope** | Single chain | Multi-layer |
| **Data Availability** | Same chain as execution | Separate DA layer |
| **Settlement** | Same chain | Can be different (L2 → L1) |
| **State Roots** | Single chain state | Cross-layer commitments |

### Proposed Future Extension

```rust
pub struct TxMetadata {
    // ... existing fields

    /// Modular blockchain metadata (future)
    pub layer_separation: Option<ModularLayers>,
}

pub struct ModularLayers {
    /// Where transaction executes
    pub execution_layer: Option<ChainRef>,

    /// Where transaction data is stored
    pub da_layer: Option<ChainRef>,

    /// Where transaction settles (for rollups)
    pub settlement_layer: Option<ChainRef>,

    /// Cross-layer commitments
    pub da_commitment: Option<Vec<u8>>,  // Hash/root in DA layer
    pub settlement_proof: Option<Vec<u8>>,  // Proof submitted to L1
}
```

### Example: Fuel Transaction on Celestia

```rust
// Decoded Fuel transaction
TxIR {
    chain: ChainRef::new("fuel-mainnet"),
    metadata: TxMetadata {
        layer_separation: Some(ModularLayers {
            execution_layer: Some(ChainRef::new("fuel-mainnet")),
            da_layer: Some(ChainRef::new("celestia")),
            settlement_layer: Some(ChainRef::new("ethereum")),
            da_commitment: Some(celestia_blob_hash),
        }),
        // ...
    },
    operations: vec![/* fuel operations */],
    // ...
}
```

### When to Implement

- ⏰ **Phase 3.10+**: When Fuel/Celestia reach significant adoption
- 🎯 **Incremental**: Start with metadata field, add structured types later
- ✅ **Now**: Document pattern for future reference

---

## Category 5: Zero-Knowledge Batching 🔴

**Risk Level**: HIGH - Individual transactions may be irrecoverable
**Timeline**: Production (zkSync, StarkNet, Mina), growing
**When to Revisit**: Phase 4+ (privacy/ZK focus)

### What They Are

Multiple transactions proven valid with single zero-knowledge proof. Individual transaction details may be hidden.

### Examples

| Project | Status | ZK System | Batch Model |
|---------|--------|-----------|-------------|
| **Mina Protocol** | Production | Recursive SNARKs | Entire blockchain → constant size |
| **zkSync Era** | Production | PLONK | Tx batch → single proof |
| **StarkNet** | Production | STARKs | State updates → batch proof |
| **Aztec** | Testnet | PLONK | Private execution → public proof |
| **Polygon zkEVM** | Production | PLONK | EVM batch → validity proof |

### Why It Breaks Current Pattern

**Current Assumption**: 1 transaction = 1 decodable unit

```rust
// Decoder expects individual transaction bytes
pub trait ChainDecoder {
    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific>;
}
```

**ZK Batch Reality**: N transactions → 1 proof

```rust
// What user submits (individual, may be encrypted)
pub struct AztecPrivateTx {
    pub encrypted_state: Vec<u8>,
    pub nullifiers: Vec<[u8; 32]>,  // Hidden inputs
    pub commitments: Vec<[u8; 32]>,  // Hidden outputs
}

// What appears on-chain (batch proof)
pub struct AztecBatchProof {
    pub proof: Vec<u8>,  // Proves validity of N transactions
    pub public_inputs: Vec<FieldElement>,
    pub new_state_root: [u8; 32],
    pub transaction_count: u32,  // How many txs
    // ❌ Individual transactions NOT included!
}
```

**Mina Protocol**: Constant-size blockchain

```rust
// On-chain: Only current state + proof
pub struct MinaBlockchainState {
    pub ledger_hash: FieldElement,  // Current state
    pub proof: RecursiveSnark,  // Proves entire history
    pub blockchain_length: u64,
    // ❌ Previous transactions discarded (verified, not stored)
}
```

### Key Challenges

| Challenge | Description | Mitigation |
|-----------|-------------|------------|
| **Transaction Recovery** | Can't decode individual tx from batch | Store off-chain, use explorer APIs |
| **Privacy** | Encrypted transactions | Only public outputs visible |
| **Constant-Size Chains** | History pruned (Mina) | Trust prover or archive nodes |
| **Proof Verification** | Need ZK verifier, not decoder | Separate verification tool |

### Proposed Future Extension

**Option 1: Batch-Aware Decoder**

```rust
pub enum DecodedUnit {
    /// Standard individual transaction
    Individual(TxIR),

    /// Zero-knowledge batch proof (future)
    ZkBatch {
        proof: Vec<u8>,
        metadata: ZkBatchMetadata,

        /// Individual transactions (if available off-chain)
        individual_txs: Option<Vec<TxIR>>,
    },
}

pub struct ZkBatchMetadata {
    pub proof_system: ZkProofSystem,  // PLONK, STARK, etc.
    pub transaction_count: u32,
    pub public_state_deltas: StateDeltas,  // Only visible outcomes
    pub verification_key: Option<Vec<u8>>,
}

pub enum ZkProofSystem {
    Plonk,
    Stark,
    RecursiveSnark,
    Halo2,
}
```

**Option 2: Batch as Metadata** (simpler)

```rust
pub struct TxMetadata {
    // ... existing fields

    /// ZK batch information (future)
    pub zk_batch: Option<ZkBatchInfo>,
}

pub struct ZkBatchInfo {
    pub is_batch_proof: bool,
    pub batch_size: u32,
    pub proof_system: String,
    pub individual_tx_available: bool,
}
```

### When to Implement

- ⏰ **Phase 4+**: After privacy infrastructure (Zcash, Monero decoders)
- 🎯 **Precondition**: Need ZK proof verification library (Verus-verified)
- ⚠️ **Limitation**: Many ZK batches cannot be fully decoded (by design)

**Recommendation**: Treat ZK batches as "opaque proofs" with public inputs/outputs only

---

## Category 6: Extended UTXO Models 🟠

**Risk Level**: MEDIUM - Straightforward extensions needed
**Timeline**: Production (Cardano), established model
**When to Revisit**: Phase 3.6+ (Cardano decoder implementation)

### What They Are

UTXO model extended with:
- Native multi-asset support
- State attached to UTXOs (datums)
- Validators/scripts attached to UTXOs

### Examples

| Project | Status | eUTXO Features |
|---------|--------|---------------|
| **Cardano** | Production | Datums, script refs, multi-asset |
| **Ergo** | Production | ErgoScript, registers, multi-asset |
| **Fuel** | Testnet | Predicates, type-safe state |
| **Nervos CKB** | Production | Cell model, type scripts |

### Why It Challenges Current Pattern

**Current UTXO Support**: Simple inputs/outputs

```rust
pub struct Input {
    pub tx_hash: Vec<u8>,
    pub output_index: u32,
    pub script_sig: Option<Vec<u8>>,
}

pub struct Output {
    pub address: Vec<u8>,
    pub amount: u128,
    pub asset_id: Option<Vec<u8>>,  // ✅ Single asset
}
```

**Cardano eUTXO Reality**: Stateful, multi-asset outputs

```rust
pub struct CardanoOutput {
    pub address: Address,

    /// Multi-asset value (ADA + native tokens)
    pub value: MultiAsset,  // ❌ Not u128!

    /// State attached to UTXO
    pub datum: Option<Datum>,  // ❌ Arbitrary data

    /// Validator script reference
    pub script_ref: Option<Script>,  // ❌ Executable code
}

pub struct MultiAsset {
    pub coin: u64,  // ADA (lovelace)
    pub assets: Vec<(PolicyId, AssetName, u64)>,  // Native tokens
}

pub enum Datum {
    Hash([u8; 32]),  // Hash of actual datum
    Inline(PlutusData),  // Full datum in output
}
```

### Key Differences

| Feature | Bitcoin UTXO | Extended UTXO (Cardano) |
|---------|-------------|------------------------|
| **Assets** | Single (BTC) | Multi-asset (ADA + tokens) |
| **State** | None | Datum (arbitrary data) |
| **Scripts** | Separate (scriptSig) | Can be attached (script ref) |
| **Validation** | Input scripts | Output validators (Plutus) |

### Proposed Future Extension

**Option 1: Extend Output Struct** (breaking change)

```rust
pub struct Output {
    pub address: Vec<u8>,
    pub amount: u128,  // Keep for simple cases

    /// Extended UTXO features (future)
    pub eutxo: Option<ExtendedUtxoData>,
}

pub struct ExtendedUtxoData {
    /// Multi-asset support
    pub assets: Vec<(Vec<u8>, u128)>,  // (asset_id, amount)

    /// State attached to UTXO (datum)
    pub datum: Option<Vec<u8>>,

    /// Validator/script reference
    pub script_ref: Option<Vec<u8>>,
}
```

**Option 2: Use Existing Metadata** (non-breaking, preferred)

```rust
pub struct Output {
    pub address: Vec<u8>,
    pub amount: u128,  // Primary asset
    pub asset_id: Option<Vec<u8>>,

    /// Store eUTXO data in metadata (JSON)
    pub metadata: Option<String>,  // JSON: { "datum": "...", "assets": [...] }
}
```

### When to Implement

- ✅ **Now**: Use `metadata` field for eUTXO extensions (Cardano decoder)
- ⏰ **Later**: Add structured `ExtendedUtxoData` when 3+ eUTXO chains supported
- 🎯 **Trigger**: Performance issues with JSON metadata or user requests structured API

---

## Category 7: Actor Model Chains 🟢

**Risk Level**: LOW - Already supported well
**Timeline**: Production (Sui, Aptos), established
**Status**: ✅ **Current architecture handles this**

### What They Are

Blockchains where transactions interact with **objects** (actors) with independent state.

### Examples

| Project | Status | Actor Model |
|---------|--------|------------|
| **Sui** | Production | Objects with ownership (owned, shared, immutable) |
| **Aptos** | Production | Resources (Move objects) |
| **Internet Computer** | Production | Canisters (actor-based smart contracts) |

### Why Current Pattern Works

**Objects are just special addresses**:

```rust
// Sui object interaction
pub struct MoveCall {
    pub package: ObjectID,  // Smart contract package
    pub module: String,
    pub function: String,
    pub arguments: Vec<Argument>,
    pub type_arguments: Vec<TypeTag>,
}

// Maps cleanly to TxIR Operation
Operation {
    op_type: OperationType::ContractCall,
    to: Some(package.to_vec()),  // Object address
    data: Some(encode_move_call(module, function, arguments)),
    // ...
}
```

**No Architecture Changes Needed**: ✅

---

## Summary Table: When to Revisit

| Category | Risk | Status | Revisit Trigger |
|----------|------|--------|----------------|
| Intent-Based | 🔴 HIGH | Research | Anoma mainnet + >10k DAU |
| DAG Consensus | 🟡 MEDIUM | Production | User requests or >5% market cap |
| Parallel Execution | 🔴 HIGH | Production | Phase 3.5+ (Sui/Aptos decoders) |
| Modular Layers | 🟠 MEDIUM | Production | Phase 3.10+ (Celestia/Fuel adoption) |
| ZK Batching | 🔴 HIGH | Production | Phase 4+ (privacy focus) |
| eUTXO | 🟠 MEDIUM | Production | Phase 3.6+ (Cardano decoder) |
| Actor Model | 🟢 LOW | Production | ✅ Already supported |

---

## Design Principles for Future Extensions

When implementing support for these models:

1. **Backward Compatibility**: Use `Option<T>` for new fields
2. **Incremental Adoption**: Start with metadata (JSON), add structured types later
3. **Zero-Cost Abstraction**: New fields should not impact existing decoders
4. **Minimal TCB**: Keep extensions out of core library when possible
5. **Chain-Specific First**: Prove pattern in decoder crate before promoting to core

---

## Recommended Next Actions

### Immediate (Document Only)
- ✅ Create this document (DONE)
- 📝 Reference in ROADMAP.md as "Future Considerations"
- 📝 Add to CHAIN_FAMILIES_GROUPING.md under "Emerging Models"

### Phase 3.5+ (Parallel Execution)
- Add `ExecutionModel` to `TxMetadata`
- Implement for Sui/Aptos decoders
- Document execution dependency analysis tools

### Phase 3.6+ (eUTXO)
- Implement Cardano decoder with eUTXO support
- Use existing `metadata` field for datums/multi-asset
- Benchmark: Structured types vs JSON metadata

### Phase 4+ (Privacy/ZK)
- Research ZK batch proof verification
- Design `DecodedUnit` enum for batch support
- Implement Mina decoder (constant-size blockchain)

### Phase 5+ (Intent-Based)
- Monitor Anoma/Namada mainnet progress
- Prototype `TransactionSemantics` extension
- Research intent verification/solver proofs

---

## References

- **Current Architecture**: `docs/TRAIT_BASED_ARCHITECTURE.md`
- **Chain Families**: `docs/CHAIN_FAMILIES_GROUPING.md`
- **TxIR Definition**: `crates/universal-decoder-core/src/ir.rs`
- **Privacy Support**: `docs/PRIVACY_ROADMAP.md`

---

**Last Updated**: 2025-11-18
**Next Review**: After Phase 3 completion (Q2 2025)
**Maintainer**: Architecture team
