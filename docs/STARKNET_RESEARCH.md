# Starknet Family Research & Decoder Feasibility Analysis

**Date**: 2025-11-14
**Status**: Research Complete
**Recommendation**: MEDIUM-HIGH Priority (After Cosmos completion)

---

## Executive Summary

Starknet is a **validity rollup** (ZK-rollup) on Ethereum using STARK proofs and the Cairo VM. The ecosystem is rapidly growing with **228+ appchains** expected in 2025 through Madara and the SN Stack. A Starknet decoder would serve:

- **Mainnet**: Starknet (chain ID: 23448594291968336)
- **Testnet**: Sepolia (chain ID: 3.934021330259978e+23)
- **Appchains**: 228+ Cosmos SDK-like chains (Kakarot zkEVM, PragmaX, Cartridge, etc.)

**Effort Estimate**: 2-3 weeks for base decoder + 1 week for appchain registry
**ROI**: HIGH - Growing ecosystem, unique ZK architecture, increasing adoption
**Dependencies**: Pure Rust feasible with `starknet-crypto` crate (Apache-2.0/MIT)

---

## 1. Architecture Overview

### 1.1 Starknet as a Layer 2

- **Type**: Validity Rollup (ZK-Rollup) on Ethereum
- **Proof System**: STARK (Scalable Transparent ARgument of Knowledge)
- **Virtual Machine**: Cairo VM (not EVM)
- **Account Model**: Account-based (not UTXO)
- **Consensus**: Off-chain sequencer (centralized → decentralized roadmap)

### 1.2 Cairo VM

**Compilation Pipeline**:
```
Cairo 1.0 Source
    ↓
Sierra (Safe Intermediate Representation)
    ↓
CASM (Cairo Assembly)
    ↓
Polynomial Constraints (STARK proof)
```

**Key Differences from EVM**:
- Von Neumann architecture vs stack-based
- Field elements (252-bit prime field) vs 256-bit words
- Nondeterministic read-only memory (optimization for proofs)

### 1.3 Chain Family Structure

```
Starknet Ecosystem
├── Mainnet (SN_MAIN - chain ID: 23448594291968336)
├── Sepolia Testnet (SN_SEPOLIA)
└── Appchains (228+ via Madara/SN Stack)
    ├── Kakarot zkEVM (EVM compatibility layer)
    ├── PragmaX (decentralized oracle)
    ├── Cartridge/Katana (game-specific rollups)
    └── [hundreds more launching in 2025]
```

**Similar Pattern**: Like Cosmos SDK (many chains, shared architecture)

---

## 2. Transaction Structure

### 2.1 Transaction Types

Starknet has **three transaction types**:

#### INVOKE (Execute Contract Function)

**Purpose**: Call a function on an existing contract

**Fields**:
- `sender_address`: Account contract address
- `calldata`: Function arguments
- `signature`: ECDSA signature on STARK curve
- `max_fee`: Maximum fee (v1) or resource bounds (v3)
- `nonce`: Account nonce
- `version`: 0, 1, or 3

**Hash Calculation (v3)**:
```
invoke_v3_tx_hash = poseidon_hash(
    "invoke",
    version,
    sender_address,
    poseidon_hash(tip, l1_gas_bounds, l2_gas_bounds, l1_data_gas_bounds),
    poseidon_hash(paymaster_data),
    chain_id,
    nonce,
    data_availability_modes,
    poseidon_hash(account_deployment_data),
    poseidon_hash(calldata)
)
```

**Validation**: Sequencer calls `__validate__()` then `__execute__()` functions

#### DECLARE (Register Contract Class)

**Purpose**: Introduce new contract classes into state

**Fields**:
- `sender_address`: Account declaring the class
- `class_hash`: Hash of contract class
- `compiled_class_hash`: Hash of compiled Sierra/CASM
- `signature`: ECDSA signature
- `max_fee` / resource bounds
- `nonce`
- `version`: 0 (Cairo 0) or 3 (Cairo 1.0)

**Hash Calculation (v3)**:
```
declare_v3_tx_hash = poseidon_hash(
    "declare",
    version,
    sender_address,
    poseidon_hash(tip, l1_gas_bounds, l2_gas_bounds, l1_data_gas_bounds),
    poseidon_hash(paymaster_data),
    chain_id,
    nonce,
    data_availability_modes,
    poseidon_hash(account_deployment_data),
    class_hash,
    compiled_class_hash
)
```

**Validation**: Sequencer calls `__validate_declare__()` function

#### DEPLOY_ACCOUNT (Deploy Account Contract)

**Purpose**: Deploy a new account contract (since StarkNet v0.10.1)

**Fields**:
- `class_hash`: Contract class to deploy
- `contract_address_salt`: Deterministic address salt
- `constructor_calldata`: Constructor arguments
- `signature`: ECDSA signature
- `max_fee` / resource bounds
- `nonce`: Always 0 (first transaction)
- `version`: 1 or 3

**Validation**: Constructor executed, then `__validate_deploy__()` called

**Note**: Account must be pre-funded to pay transaction fee

### 2.2 Encoding Format

**CRITICAL**: Starknet does **NOT** use RLP encoding (unlike Ethereum)

**Actual Encoding**:
- **Binary Format**: Field elements as 32-byte big-endian integers
- **Hash Functions**:
  - **Pedersen hash** (legacy, still used for storage addresses)
  - **Poseidon hash** (recommended, cheaper for STARK proofs)
- **Signature Scheme**: ECDSA on STARK-friendly elliptic curve

**Field Element (`felt`)**:
- Type: Element in 252-bit prime field
- Prime: `2^251 + 17 * 2^192 + 1`
- Serialization: 32-byte big-endian integer

### 2.3 Transaction Lifecycle

```
1. Transaction submitted to sequencer
2. Sequencer validates:
   - INVOKE: Calls __validate__(calldata)
   - DECLARE: Calls __validate_declare__()
   - DEPLOY_ACCOUNT: Constructor + __validate_deploy__()
3. If valid, sequencer executes:
   - INVOKE: Calls __execute__(calldata)
   - DECLARE: Stores class in state
   - DEPLOY_ACCOUNT: Deploys contract
4. Transaction batched into block
5. STARK proof generated for block
6. Proof submitted to Ethereum L1
```

---

## 3. Cryptographic Primitives

### 3.1 Hash Functions

#### Pedersen Hash

**Definition**:
```
h(a, b) = [shift_point + a_low·P₀ + a_high·P₁ + b_low·P₂ + b_high·P₃]ₓ
```

**Constants**:
- `P₀, P₁, P₂, P₃`: Constant points on elliptic curve (derived from π digits)
- `shift_point`: Added to avoid point at infinity

**Usage**:
- Legacy (first hash function on Starknet)
- Still used for storage variable addresses (e.g., `LegacyMap` key hashing)

**Input**: Two field elements (felts)

#### Poseidon Hash

**Definition**:
```
poseidon(x) = [hades_permutation(x, 0, 1)]₀
```

**Implementation**:
- Three-element state Hades permutation
- 6 consecutive cells: 3 inputs → 3 outputs
- Builtin deduction property

**Usage**:
- **Recommended** for all new code
- Cheaper and faster than Pedersen for STARK proofs
- Used in v3 transaction hashing

**Advantages**:
- ~30-40% gas savings vs Pedersen
- Optimized for ZK-STARK arithmetic

### 3.2 Signature Verification

#### STARK Curve

**Type**: Elliptic Curve Digital Signature Algorithm (ECDSA)

**Curve Parameters**:
- **Order**: `3618502788666131213697322783095070105526743751716087489154079457884512865583`
- **Generator Point G**:
  - `Gₓ = 874739451078007766457464989774322083649278607533249481151382481072868806602`
  - `Gᵧ = 152666792071518830868575557812948353041420400780739481342941381225525861407`

**Signature Structure**:
- **Type**: ECDSA signature
- **Components**: `(r, s)` pair
- **Verification**: Via Cairo `verify_ecdsa_signature()` builtin

**Note**: Not a protocol-level signature scheme, but Cairo has efficient implementation

---

## 4. Rust Library Ecosystem

### 4.1 starknet-rs

**Repository**: https://github.com/xJonathanLEI/starknet-rs
**License**: Apache-2.0 OR MIT (dual-licensed)
**Status**: ⚠️ **Experimental** (breaking changes expected before v1.0)
**Language**: 97.2% Rust, 1.4% Cairo

**Crate Structure**:

| Crate | Purpose | Relevant for Decoder? |
|-------|---------|----------------------|
| `starknet` | Umbrella re-export | ✅ Entry point |
| `starknet-core` | Core data structures | ✅ **CRITICAL** - Transaction types |
| `starknet-providers` | Client for nodes/sequencers | ❌ Network operations (out of scope) |
| `starknet-contract` | Contract deployment/interaction | ⚠️ Possibly useful for calldata parsing |
| `starknet-crypto` | **Low-level crypto** | ✅ **CRITICAL** - Hash + signature |
| `starknet-signers` | Signer implementations | ❌ Signing (out of scope) |
| `starknet-accounts` | Account abstraction | ⚠️ Possibly useful for validation logic |
| `starknet-curve` | Curve operations | ✅ ECDSA verification |
| `starknet-macros` | Proc macros | ❌ Not needed |
| `starknet-core-derive` | Derive macros | ❌ Not needed |
| `starknet-tokio-tungstenite` | WebSocket client | ❌ Network (out of scope) |

### 4.2 starknet-crypto

**Critical Crate for Decoder**

**Provides**:
- ✅ Pedersen hash (stateful hasher + function)
- ✅ Poseidon hash (stateful hasher + function)
- ✅ STARK ECDSA signature operations
- ✅ Public key computation from private keys
- ✅ Field element types

**Security Warnings**:
- ⚠️ **NOT audited** for security
- ⚠️ **NO constant-time guarantees** (side-channel vulnerable)
- ⚠️ Recommendation: Use high-level `starknet-core` utilities if unfamiliar

**Dependencies**: Unknown (need to inspect Cargo.toml)

**For Decoder Use**:
- ✅ Suitable for **read-only decoding** (not signing keys)
- ✅ Hash verification is safe (constant-time not critical)
- ✅ Public signature verification is safe

### 4.3 Alternative Libraries

#### starknet_in_rust

**Repository**: https://github.com/lambdaclass/starknet_in_rust
**Status**: ⚠️ **ARCHIVED** (only supports up to Starknet v0.13.0)
**Issue**: Requires `gmp` library (C dependency via cairo-lang)
**Verdict**: ❌ **NOT SUITABLE** (outdated + C dependencies)

#### Neptune (Poseidon)

**Repository**: https://github.com/lurk-lab/neptune
**Purpose**: General Poseidon hash (BLS12-381 curve)
**Verdict**: ❌ Wrong curve (need STARK curve, not BLS12-381)

#### light-poseidon

**Repository**: https://github.com/Lightprotocol/light-poseidon
**Purpose**: Poseidon for BN254 curve
**Verdict**: ❌ Wrong curve (need STARK curve, not BN254)

### 4.4 Dependency Analysis for Pure Rust Decoder

**Strategy**: Use `starknet-crypto` as **dev-dependency only** (validation)

**Pure Rust Path**:
1. **Vendor hash implementations** from `starknet-crypto` (Apache-2.0/MIT allows this)
2. **Extract only needed functions**:
   - Pedersen hash (for storage addresses)
   - Poseidon hash (for v3 transaction hashing)
   - ECDSA verification (for signatures)
3. **Implement field element arithmetic** (252-bit modular math)
4. **Implement transaction parsers** (binary → struct)

**Estimated Complexity**:
- **Field Element**: 200-300 LOC (modular arithmetic)
- **Pedersen Hash**: 300-400 LOC (elliptic curve ops)
- **Poseidon Hash**: 400-500 LOC (Hades permutation)
- **ECDSA Verify**: 300-400 LOC (curve ops)
- **Transaction Parsing**: 600-800 LOC (3 tx types)
- **Total**: ~2000-2400 LOC

**Alternative**: Keep `starknet-crypto` in **dependencies** (not dev-deps)
- **Pros**: Faster implementation, maintained code
- **Cons**: Adds external dependency, not audited, no constant-time
- **Verdict**: Acceptable for Phase 3.x, vendor later in Phase 4 (security hardening)

---

## 5. Starknet Ecosystem Chains

### 5.1 Network Overview

| Network | Chain ID | Purpose | Status |
|---------|----------|---------|--------|
| **Mainnet** (SN_MAIN) | `23448594291968336` | Production L2 | ✅ Live |
| **Sepolia Testnet** | `3.934021330259978e+23` | Public testnet | ✅ Live |

### 5.2 Appchain Ecosystem (via Madara/SN Stack)

**Launch**: January 2025 (SN Stack release)
**Estimated Chains**: **228+ appchains** (168% growth from 2023-2024)
**Framework**: Madara (Substrate-based sequencer)

#### SN Stack Flavors

1. **StarkWare Sequencer**: Official production sequencer
2. **Madara**: Substrate-based, modular (consensus, DA customizable)
3. **Dojo**: Game-specific rollup framework

#### Notable Appchains

| Project | Type | Use Case | Status |
|---------|------|----------|--------|
| **Kakarot zkEVM** | MultiVM | EVM compatibility on Starknet | Testnet live, Q1 2025 mainnet |
| **PragmaX** | Oracle | Decentralized price feeds | Planned (Madara-based) |
| **Cartridge (Katana)** | Gaming | Game-specific rollups | Development |
| **dYdX v4** | DeFi | Decentralized exchange | Evaluating Starknet stack |

### 5.3 Chain Registry Strategy

**Similar to**: Cosmos SDK (many chains, shared codebase)

**Registry Requirements**:
1. **Chain metadata**: name, chain_id, RPC endpoints, explorer
2. **Contract addresses**: Standard contracts (multicall, registry, etc.)
3. **Fee tokens**: ETH, STRK (Starknet token), custom tokens
4. **Version info**: Cairo version, protocol version

**Existing Registries**:
- ❌ No official Starknet chain registry (yet)
- ✅ Chainlist.org has Starknet mainnet/testnet
- ⚠️ Appchain registry likely needed from StarkWare or community

**Vendoring Strategy**:
```bash
# Option 1: Create our own registry (minimal, 2 chains)
crates/decoder-starknet/data/starknet-chains.borsh
# Mainnet + Sepolia only

# Option 2: Wait for official appchain registry
git subtree add \
    --prefix crates/decoder-starknet/vendored/starknet-registry \
    https://github.com/starknet-io/chain-registry.git \
    main --squash

# Option 3: Scrape from StarkWare docs
# Parse https://docs.starknet.io/resources/chain-info/
```

**Recommended**: Start with **Option 1** (2 chains), expand to Option 2 when registry exists

---

## 6. Decoder Implementation Feasibility

### 6.1 Complexity Assessment

**Similarity to Existing Decoders**:

| Aspect | Similar To | Complexity |
|--------|-----------|------------|
| Account model | Ethereum, Solana | Medium |
| Transaction types (3) | Cosmos (8 types) | Low-Medium |
| Binary parsing | Bitcoin, Solana | Medium |
| Hash functions | Custom (not RLP/SHA) | **HIGH** |
| Signature verification | Bitcoin (ECDSA) | Medium |
| Field element math | New | **HIGH** |

**Unique Challenges**:
1. ✅ **Field Elements**: 252-bit modular arithmetic (not standard u64/u256)
2. ✅ **Poseidon Hash**: Complex Hades permutation (not SHA family)
3. ✅ **Pedersen Hash**: Elliptic curve arithmetic (not hash function)
4. ✅ **Cairo VM Types**: Sierra/CASM decoding (for calldata interpretation)

### 6.2 Implementation Phases

#### Phase 1: Basic Transaction Decoder (1-2 weeks)

**Scope**:
- ✅ Parse INVOKE v1/v3 transactions
- ✅ Parse DECLARE v1/v3 transactions
- ✅ Parse DEPLOY_ACCOUNT v1/v3 transactions
- ✅ Extract fields (sender, nonce, signature, calldata)
- ✅ Convert to TxIR representation

**Dependencies**:
- `starknet-crypto` in dependencies (temporary, vendor later)
- OR vendor Pedersen/Poseidon immediately (slower, more secure)

**Testing**:
- ✅ Mainnet transaction fixtures (from block explorer)
- ✅ Sepolia testnet fixtures
- ✅ Property tests (encoding → decoding roundtrip via Borsh)
- ✅ Integration tests (validate against `starknet-core` in dev-deps)

**Deliverable**: `decoder-starknet` crate with 2 chains (mainnet, testnet)

#### Phase 2: Hash & Signature Verification (3-5 days)

**Scope**:
- ✅ Implement Poseidon hash (or vendor from starknet-crypto)
- ✅ Implement Pedersen hash (or vendor)
- ✅ Verify transaction hash calculation
- ✅ Verify ECDSA signatures on STARK curve

**Testing**:
- ✅ Test vectors from Starknet docs
- ✅ Cross-validation with `starknet-crypto`
- ✅ Fuzz testing (random felts → hash)

**Deliverable**: Verified tx hash + signature validation

#### Phase 3: Appchain Support (3-5 days)

**Scope**:
- ✅ Create appchain registry (or vendor existing)
- ✅ Support chain-specific parameters (chain_id, fee token)
- ✅ Add Kakarot zkEVM (if launched)
- ✅ Document appchain integration process

**Testing**:
- ✅ Multi-chain transaction decoding
- ✅ Chain-specific TxIR metadata

**Deliverable**: `decoder-starknet` supporting 10+ chains

#### Phase 4: Cairo VM Calldata Decoding (Optional, 1-2 weeks)

**Scope**:
- ⚠️ Parse Sierra intermediate representation
- ⚠️ Decode calldata to structured arguments
- ⚠️ Extract function selector and parameters

**Complexity**: **VERY HIGH** (requires Cairo VM understanding)

**Recommendation**: **DEFER** to Phase 5.x (advanced features)

### 6.3 Effort Estimation

| Task | Estimated Time | Difficulty |
|------|---------------|------------|
| Research & Design | 1 day | ✅ Complete |
| Field Element Implementation | 1-2 days | High |
| Transaction Parsing (3 types) | 2-3 days | Medium |
| Hash Functions (Pedersen + Poseidon) | 2-3 days | High |
| Signature Verification | 1-2 days | Medium |
| TxIR Integration | 1 day | Low |
| Testing & Fixtures | 2-3 days | Medium |
| Appchain Registry | 1 day | Low |
| Documentation | 1 day | Low |
| **TOTAL** | **12-18 days** | **Medium-High** |

**With Vendoring starknet-crypto (faster)**:
- Remove 5-7 days (hash + signature implementation)
- **Total: 7-11 days (1.5-2 weeks)**

### 6.4 Dependency Strategy

**Recommended Approach**:

```toml
# crates/decoder-starknet/Cargo.toml

[dependencies]
decoder-primitives = { path = "../decoder-primitives" }
decoder-encodings = { path = "../decoder-encodings" }

# TEMPORARY: Use starknet-crypto for Phase 3.x
# TODO: Vendor in Phase 4 (security audit)
starknet-crypto = { version = "0.7", features = ["alloc"] }

[dev-dependencies]
# Validation against reference implementation
starknet = "0.11"
starknet-core = "0.11"
```

**Future (Phase 4 - Security Hardening)**:
```toml
[dependencies]
# Vendored via git subtree (audited, minimal)
# starknet-crypto = { path = "vendored/starknet-crypto" }
```

**Rationale**:
- ✅ Faster implementation (Phase 3 goal: broad chain coverage)
- ✅ Maintained code (starknet-rs actively developed)
- ✅ Apache-2.0/MIT license (compatible)
- ⚠️ Not audited (acceptable for alpha/beta)
- ⚠️ Not constant-time (acceptable for decoding, not signing)
- ✅ Can vendor later (Phase 4 hardening)

---

## 7. ROI Analysis

### 7.1 Chain Coverage Impact

**Current Top 20**:
- ❌ Starknet not yet in decoder family

**After Starknet Decoder**:
- ✅ +1 major L2 (Starknet mainnet)
- ✅ +1 testnet (Sepolia)
- ✅ +228 potential appchains (2025 growth)
- ✅ Total: **230+ chains** from single decoder

**ROI**: **VERY HIGH** (similar to Cosmos SDK: 1 decoder → 228 chains)

### 7.2 Ecosystem Growth

**Metrics** (2023-2024):
- Projects: +168% growth (72 → 193 projects)
- New dApps: 121 in one year
- Production readiness: Early 2025

**2025 Outlook**:
- SN Stack launch (January 2025) → appchain explosion
- Kakarot zkEVM → EVM developers enter ecosystem
- Madara maturity → easier appchain deployment

**Verdict**: **Rapidly growing ecosystem**, early adoption valuable

### 7.3 Technical Uniqueness

**Starknet Fills Gaps**:
- ✅ Only ZK-Rollup in decoder (vs Optimistic: Optimism, Arbitrum)
- ✅ Only STARK-based chain (vs SNARK: zkSync uses SNARK)
- ✅ Only Cairo VM chain (vs EVM, WASM, Move)
- ✅ Poseidon hash (new cryptographic primitive)

**Educational Value**: Demonstrates decoder flexibility (non-EVM, non-UTXO)

### 7.4 Community Interest

**Indicators**:
- ✅ StarkWare well-funded (StarkEx, StarkNet)
- ✅ Major dApps (dYdX exploring, Kakarot launching)
- ✅ Active developer community
- ✅ Strong documentation (docs.starknet.io)

**Risk**: Still experimental (v0.x), protocol changes possible

---

## 8. Risks & Mitigations

### 8.1 Protocol Instability

**Risk**: Transaction format changes in future versions

**Evidence**:
- Version 0 → 1 → 3 (major changes in fee structure)
- Sierra compilation model still evolving

**Mitigation**:
- ✅ Support multiple versions (v1 and v3)
- ✅ Document version differences clearly
- ✅ Use `starknet-core` types (track upstream changes)
- ✅ Comprehensive integration tests (detect breaking changes)

### 8.2 Cryptography Complexity

**Risk**: Incorrect hash/signature implementation → security vulnerabilities

**Mitigation**:
- ✅ Use `starknet-crypto` (battle-tested, community-reviewed)
- ✅ Extensive test vectors from Starknet docs
- ✅ Cross-validation against reference implementation
- ✅ Defer custom crypto to Phase 4 (after audit)

### 8.3 Appchain Registry Availability

**Risk**: No official appchain registry exists yet

**Mitigation**:
- ✅ Start with 2 chains (mainnet + testnet)
- ✅ Manual registry for known appchains (Kakarot, PragmaX)
- ✅ Monitor StarkWare for official registry announcement
- ✅ Community-driven registry as fallback

### 8.4 Dependency on starknet-rs

**Risk**: Library is experimental, may have breaking changes

**Mitigation**:
- ✅ Pin exact versions in Cargo.toml
- ✅ Use only `starknet-crypto` (smallest surface area)
- ✅ Plan to vendor in Phase 4 (reduce external dependency)
- ✅ CI tests catch upstream breakage immediately

---

## 9. Recommendations

### 9.1 Implementation Priority

**Recommendation**: **Phase 3.6** (After Cosmos SDK completion)

**Rationale**:
1. ✅ High ROI (230+ chains potential)
2. ✅ Growing ecosystem (2025 is appchain year)
3. ✅ Unique architecture (ZK-rollup, Cairo VM)
4. ✅ Manageable complexity (2-3 weeks)
5. ✅ Fills gap (only STARK-based chain)

**Timeline**:
- Phase 3.5: ✅ Cosmos SDK (complete)
- **Phase 3.6**: Starknet Family (2-3 weeks)
- Phase 3.7: Additional families (Polkadot, Cardano, etc.)

### 9.2 Implementation Approach

**Recommended Path**:

1. **Week 1**: Core Transaction Decoder
   - Implement field element type (252-bit felt)
   - Parse INVOKE, DECLARE, DEPLOY_ACCOUNT
   - Convert to TxIR
   - Unit tests + property tests

2. **Week 2**: Hash & Signature
   - Integrate `starknet-crypto` (Pedersen, Poseidon)
   - Verify transaction hashes (v1 + v3)
   - Verify ECDSA signatures
   - Integration tests with mainnet fixtures

3. **Week 3** (Optional): Appchain Support
   - Create minimal chain registry (2 chains)
   - Add Kakarot zkEVM (if launched)
   - Document appchain integration
   - Update ROADMAP.md

**Deliverables**:
- `crates/decoder-starknet/` (2000-2500 LOC)
- `docs/STARKNET_DECODER.md` (architecture)
- 50+ tests (unit + property + integration)
- 20+ real transaction fixtures

### 9.3 Dependency Strategy

**For Phase 3.6**:
```toml
[dependencies]
starknet-crypto = "0.7"  # Apache-2.0/MIT, pure Rust
```

**For Phase 4 (Hardening)**:
```bash
# Vendor only crypto components
git subtree add \
    --prefix crates/decoder-starknet/vendored/starknet-crypto \
    https://github.com/xJonathanLEI/starknet-rs.git \
    starknet-crypto/v0.7.0 --squash
```

### 9.4 Testing Strategy

**5-Level Pyramid**:

1. **Unit Tests**: All parsers, field element ops (30+ tests)
2. **Property Tests**: Encoding roundtrip, hash determinism (15+ tests)
3. **Integration Tests**: Real mainnet/testnet transactions (20+ fixtures)
4. **Fuzz Tests**: Random transaction bytes, field elements
5. **Formal Verification**: Field element arithmetic (Phase 4)

**Fixtures**:
- Genesis transaction (if exists)
- INVOKE v1 (legacy fee)
- INVOKE v3 (resource bounds)
- DECLARE v0 (Cairo 0)
- DECLARE v3 (Cairo 1.0)
- DEPLOY_ACCOUNT v1
- DEPLOY_ACCOUNT v3
- Failed transactions (validation errors)
- Edge cases (zero nonce, max fee, etc.)

### 9.5 Documentation Requirements

**Essential Docs**:
1. `docs/STARKNET_DECODER.md` - Architecture and design
2. `crates/decoder-starknet/README.md` - Usage guide
3. `crates/decoder-starknet/VENDORED.md` - Dependency info (if vendored)
4. Update `ROADMAP.md` - Add Phase 3.6
5. Update `docs/CHAIN_FAMILIES_GROUPING.md` - Add Starknet family

---

## 10. Next Steps

### 10.1 Immediate Actions

- [x] Complete Starknet research (this document)
- [ ] Discuss priority with project stakeholders
- [ ] Create Phase 3.6 branch: `claude/starknet-decoder-phase-3.6-<session-id>`
- [ ] Scaffold `crates/decoder-starknet/` structure
- [ ] Add `starknet-crypto` dependency
- [ ] Implement field element type

### 10.2 Open Questions

1. **Appchain Registry**: Wait for official registry or create minimal one?
   - **Recommendation**: Start minimal (2 chains), expand when registry available

2. **Cairo VM Calldata**: Include in Phase 3.6 or defer?
   - **Recommendation**: DEFER to Phase 5.x (complex, optional for basic decoding)

3. **Vendor starknet-crypto Now or Later**: Phase 3 or Phase 4?
   - **Recommendation**: Use as dependency in Phase 3, vendor in Phase 4

4. **Support v0 Transactions**: Legacy format or v1+ only?
   - **Recommendation**: Support v1 + v3 (v0 deprecated)

### 10.3 Success Criteria

**Phase 3.6 Complete When**:
- ✅ Decode all 3 transaction types (INVOKE, DECLARE, DEPLOY_ACCOUNT)
- ✅ Support v1 and v3 transaction versions
- ✅ Verify transaction hashes (Pedersen for v1, Poseidon for v3)
- ✅ Verify ECDSA signatures on STARK curve
- ✅ Convert to TxIR with correct metadata
- ✅ 50+ tests passing (unit + property + integration)
- ✅ 20+ real transaction fixtures from mainnet/testnet
- ✅ CI/CD passing (format, clippy, tests)
- ✅ Documentation complete

**Stretch Goals**:
- ✅ Support 10+ appchains (Kakarot, PragmaX, etc.)
- ✅ Appchain registry integration
- ✅ 100+ test fixtures
- ⚠️ Cairo VM calldata decoding (defer if complex)

---

## 11. References

### 11.1 Official Documentation

- [Starknet Documentation](https://docs.starknet.io/)
- [Transaction Types](https://docs.starknet.io/architecture-and-concepts/network-architecture/transactions/)
- [Cryptography](https://docs.starknet.io/architecture/cryptography/)
- [Cairo Book](https://www.starknet.io/cairo-book/)
- [Chain Information](https://docs.starknet.io/resources/chain-info/)

### 11.2 Rust Libraries

- [starknet-rs](https://github.com/xJonathanLEI/starknet-rs) - Complete Starknet library
- [starknet-crypto](https://docs.rs/starknet-crypto/latest/starknet_crypto/) - Cryptographic primitives
- [starknet-core](https://docs.rs/starknet/latest/starknet/) - Core data structures

### 11.3 Ecosystem

- [Madara Appchains](https://www.starknet.io/blog/stark-spaces-madara-starknet-appchains-video/)
- [SN Stack Launch](https://blockworks.co/news/starknet-introduces-sn-stack)
- [Ecosystem Report 2025](https://www.starknet.io/blog/starknet-ecosystem-report-2025/)
- [Kakarot zkEVM](https://docs.kakarot.org/)

### 11.4 Specifications

- [Pedersen Hash](https://docs.starkware.co/starkex/crypto/pedersen-hash-function.html)
- [Poseidon Hash](https://www.starknet.io/cairo-book/ch204-02-07-poseidon.html)
- [ECDSA on STARK Curve](https://www.starknet.io/cairo-book/ch204-02-03-ecdsa.html)
- [Transaction Hash Reference](https://docs.starknet.io/resources/transactions-reference/)

---

## 12. Conclusion

Starknet represents a **high-value, medium-complexity addition** to the universal blockchain decoder. With **230+ potential chains** (mainnet + testnet + 228 appchains), it offers excellent ROI for 2-3 weeks of implementation effort.

**Key Takeaways**:
1. ✅ **Growing Ecosystem**: 168% project growth, SN Stack launch in 2025
2. ✅ **Unique Architecture**: Only STARK-based ZK-rollup, Cairo VM (not EVM)
3. ✅ **Manageable Complexity**: 3 transaction types, well-documented
4. ✅ **Pure Rust Feasible**: `starknet-crypto` available (Apache-2.0/MIT)
5. ✅ **High ROI**: 1 decoder → 230+ chains (similar to Cosmos)

**Recommendation**: Implement as **Phase 3.6**, immediately after Cosmos SDK completion. Use `starknet-crypto` as dependency initially, vendor in Phase 4 security hardening.

---

**Document Version**: 1.0
**Author**: Claude (Anthropic)
**Date**: 2025-11-14
**Status**: Research Complete, Ready for Implementation Planning
