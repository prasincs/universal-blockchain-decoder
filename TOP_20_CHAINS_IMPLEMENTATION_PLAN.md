# Top 20 Blockchains - Implementation Plan

**Created**: 2025-11-12
**Status**: Phase 1 - Scaffolding Complete
**Next Phase**: Pure Rust Implementation (Phase 2)

## Overview

This document outlines the implementation strategy for scaffolding the top 20 blockchain decoders for the Universal Blockchain Decoder project. Each decoder follows the minimal TCB (Trusted Computing Base) philosophy with trait-based extensibility.

## Chain Selection Criteria

Selection based on:
1. Market capitalization (Top 20 as of 2025)
2. Transaction volume
3. Developer ecosystem
4. Unique architectural patterns

## Top 20 Chains

### ✅ Already Implemented

1. **Bitcoin** (decoder-bitcoin) - UTXO model, SegWit support
2. **Ethereum** (decoder-ethereum) - Account model, EVM
3. **Solana** (decoder-solana) - Instruction model, parallel runtime

### 🚧 To Be Scaffolded (17 chains)

| # | Chain | Family | Key Characteristics | Chain ID | Priority |
|---|-------|--------|---------------------|----------|----------|
| 4 | BNB Chain | Account (EVM) | Binance fork, PoSA consensus | 56 | High |
| 5 | XRP Ledger | Account | Payment-focused, RPCA consensus | 144 | High |
| 6 | Cardano | UTXO (eUTXO) | Haskell-based, formal methods | 1815 | High |
| 7 | Dogecoin | UTXO | Bitcoin fork, scrypt PoW | 3 | Medium |
| 8 | Tron | Account | DPoS, energy model | 195 | High |
| 9 | Polygon | Account (EVM) | Ethereum L2/sidechain | 137 | High |
| 10 | Avalanche | Account (EVM) | Subnet architecture, C-Chain | 43114 | High |
| 11 | Polkadot | Substrate | Relay chain, parachains | 0 (relay) | High |
| 12 | Litecoin | UTXO | Bitcoin fork, scrypt PoW | 2 | Medium |
| 13 | NEAR | Account | Sharded, PoS | 397 | Medium |
| 14 | Cosmos | Account | IBC, Tendermint | cosmos-hub | High |
| 15 | Stellar | Account | Payment-focused, SCP consensus | stellar | Medium |
| 16 | Algorand | Account | Pure PoS, AVM | 4160 | Medium |
| 17 | Optimism | Account (EVM) | Optimistic rollup, EVM-equivalent | 10 | High |
| 18 | Arbitrum | Account (EVM) | Optimistic rollup, EVM-compatible | 42161 | High |
| 19 | Sui | Instruction | Move-based, object-centric | 0 | High |
| 20 | Aptos | Account | Move-based, parallel execution | 1 | High |

---

## 🚀 Recent Updates

**Date**: 2025-11-12
**Update**: EVM-Compatible Chains Now Implemented!

Five EVM-compatible chains now **fully reuse the Ethereum decoder** with only chain ID validation:
- ✅ **BNB Chain** (ID: 56) - Implemented
- ✅ **Polygon** (ID: 137) - Implemented
- ✅ **Avalanche C-Chain** (ID: 43114) - Implemented
- ✅ **Optimism** (ID: 10) - Implemented
- ✅ **Arbitrum** (ID: 42161) - Implemented

**Implementation**: All five chains use `decoder-ethereum` as a dependency and reuse `EthereumTransaction` type directly. Each decoder validates chain-specific IDs. **Zero code duplication.**

**Testing**: All tests passing ✅
**LOC**: ~100 LOC per EVM decoder (vs ~2000 LOC for standalone implementation)
**Reuse Ratio**: 95% code reuse from Ethereum

---

## Detailed Implementation Plans

### 4. BNB Chain (Binance Smart Chain)

**Chain Family**: Account (EVM)
**Chain ID**: 56
**Consensus**: Proof of Staked Authority (PoSA)
**Status**: ✅ **IMPLEMENTED** (Phase 2 Complete)

**Transaction Format**:
- RLP-encoded (identical to Ethereum)
- EIP-2718 transaction types (legacy, EIP-2930, EIP-1559)
- Compatible with Ethereum tooling

**Implementation Strategy**:
```rust
// ✅ IMPLEMENTED: Reuses Ethereum decoder with chain ID validation

pub struct BnbDecoder;

impl ChainDecoder for BnbDecoder {
    type TxSpecific = EthereumTransaction;  // Reuse Ethereum!

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID is 56 (mainnet) or 97 (testnet)
        if let Some(chain_id) = tx.chain_id {
            if chain_id != 56 && chain_id != 97 {
                return Err(/* invalid chain ID */);
            }
        }

        Ok(tx)
    }
}
```

**Key differences from Ethereum**:
1. Different chain ID (56 for mainnet, 97 for testnet)
2. PoSA consensus (21 validators, not relevant for tx decoding)
3. BEP-2/BEP-20 token standards (same as ERC-20 at tx level)

**Dependencies**:
- Production: `decoder-ethereum` (crate)
- Dev: None (shares Ethereum tests)

**Validation Strategy**:
- Reuses Ethereum decoder's RLP parsing
- Chain ID validation for BSC-specific networks
- All Ethereum tests apply

**Complexity**: Very Low ✅ (Direct Ethereum reuse)

---

### 5. XRP Ledger

**Chain Family**: Account
**Chain ID**: 144 (custom)
**Consensus**: Ripple Protocol Consensus Algorithm (RPCA)

**Transaction Format**:
- Binary serialization (custom format, not Protobuf/RLP)
- 16 transaction types (Payment, OfferCreate, TrustSet, etc.)
- Field ordering matters for canonical serialization
- Uses canonical field ordering (sorted by field ID)

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with minimal parsing
// Phase 2: Implement binary codec (ripple-binary-codec spec)

// Key components:
// 1. Binary parser for XRP format
// 2. 16 transaction type handlers
// 3. Amount encoding (XRP drops + IOU amounts)
// 4. Account address decoding (base58 with checksum)

// Transaction types:
pub enum XrpTransactionType {
    Payment,           // 0
    EscrowCreate,      // 1
    EscrowFinish,      // 2
    AccountSet,        // 3
    EscrowCancel,      // 4
    SetRegularKey,     // 5
    NickNameSet,       // 6 (deprecated)
    OfferCreate,       // 7
    OfferCancel,       // 8
    Contract,          // 9 (unmaintained)
    TicketCreate,      // 10
    SignerListSet,     // 12
    PaymentChannelCreate,  // 13
    PaymentChannelFund,    // 14
    PaymentChannelClaim,   // 15
    CheckCreate,       // 16
    // ... more types
}
```

**Dependencies** (dev-only):
- No standard Rust library exists, will need custom validation

**Validation Strategy**:
- Test against XRP Ledger testnet transactions
- Verify canonical field ordering
- Validate amount encoding (drops for XRP, custom for IOUs)

**Estimated Complexity**: High (custom binary format)

---

### 6. Cardano

**Chain Family**: UTXO (Extended UTXO - eUTXO)
**Chain ID**: 1815 (custom)
**Consensus**: Ouroboros PoS

**Transaction Format**:
- CBOR encoding (Concise Binary Object Representation)
- eUTXO model (UTXOs can carry state)
- Plutus smart contracts
- Multi-asset support (native tokens)

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with pallas (Rust Cardano library)
// Phase 2: Pure Rust CBOR parser + Cardano primitives

// Key components:
// 1. CBOR decoder
// 2. eUTXO parser (inputs, outputs, datums, redeemers)
// 3. Plutus script handling
// 4. Multi-asset support

// Transaction structure:
pub struct CardanoTransaction {
    pub body: TransactionBody,
    pub witness_set: TransactionWitnessSet,
    pub is_valid: bool,  // Smart contract validation flag
    pub auxiliary_data: Option<AuxiliaryData>,
}

pub struct TransactionBody {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub fee: Coin,
    pub ttl: Option<u64>,  // Time-to-live
    pub certificates: Vec<Certificate>,
    pub withdrawals: BTreeMap<RewardAccount, Coin>,
    pub update: Option<Update>,
    pub auxiliary_data_hash: Option<Hash32>,
    pub validity_interval_start: Option<u64>,
    pub mint: Option<MultiAsset>,
    pub script_data_hash: Option<Hash32>,
    pub collateral: Vec<TransactionInput>,
    pub required_signers: Vec<Hash28>,
    pub network_id: Option<NetworkId>,
}
```

**Dependencies** (dev-only):
- `pallas-primitives` - For validation testing
- `minicbor` - For CBOR comparison

**Validation Strategy**:
- Test against Cardano mainnet transactions
- Verify eUTXO model correctness
- Validate Plutus script hashing

**Estimated Complexity**: High (CBOR + eUTXO complexity)

---

### 7. Dogecoin

**Chain Family**: UTXO
**Chain ID**: 3
**Consensus**: Proof of Work (Scrypt)

**Transaction Format**:
- Identical to Bitcoin (legacy transactions)
- No SegWit support
- Same serialization format

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding (reuse Bitcoin decoder)
// Phase 2: Pure Rust (share parser with Bitcoin)

// Key differences from Bitcoin:
// 1. Different chain ID (3)
// 2. No SegWit
// 3. Different block reward schedule
// 4. Scrypt PoW (not relevant for tx decoding)

// Can literally reuse BitcoinDecoder with minor changes:
pub struct DogecoinDecoder;

impl ChainDecoder for DogecoinDecoder {
    type TxSpecific = BitcoinTransaction;  // Reuse!
    type Chain = DogecoinChain;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Reuse Bitcoin parser, no SegWit detection
        BitcoinDecoder::decode_legacy(raw_bytes)
    }
}
```

**Dependencies** (dev-only):
- None (reuse Bitcoin)

**Validation Strategy**:
- Test against Dogecoin mainnet transactions
- Verify no SegWit transactions

**Estimated Complexity**: Very Low (Bitcoin clone)

---

### 8. Tron

**Chain Family**: Account
**Chain ID**: 195
**Consensus**: Delegated Proof of Stake (DPoS)

**Transaction Format**:
- Protobuf encoding
- Contract-based architecture
- Energy and bandwidth model
- TRC-10/TRC-20 tokens

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with tron-rs (if exists) or manual protobuf
// Phase 2: Pure Rust with prost (protobuf library)

// Key components:
// 1. Protobuf decoder
// 2. Contract type handlers (Transfer, TriggerSmartContract, etc.)
// 3. Resource accounting (energy, bandwidth)

// Transaction structure:
pub struct TronTransaction {
    pub raw_data: RawData,
    pub signatures: Vec<Vec<u8>>,
}

pub struct RawData {
    pub contract: Vec<Contract>,
    pub ref_block_bytes: Vec<u8>,
    pub ref_block_hash: Vec<u8>,
    pub expiration: i64,
    pub timestamp: i64,
    pub fee_limit: i64,
}

pub enum ContractType {
    TransferContract,
    TransferAssetContract,
    TriggerSmartContract,
    CreateSmartContract,
    // ... 50+ contract types
}
```

**Dependencies** (dev-only):
- `prost` - Protobuf library
- Generate from `.proto` files

**Validation Strategy**:
- Test against Tron mainnet transactions
- Verify contract execution
- Validate resource accounting

**Estimated Complexity**: Medium (Protobuf + many contract types)

---

### 9. Polygon (Matic)

**Chain Family**: Account (EVM)
**Chain ID**: 137
**Consensus**: PoS (validators + checkpoints to Ethereum)
**Status**: ✅ **IMPLEMENTED** (Phase 2 Complete)

**Transaction Format**:
- RLP-encoded (identical to Ethereum)
- EIP-2718 transaction types
- EVM-compatible

**Implementation Strategy**:
```rust
// ✅ IMPLEMENTED: Direct Ethereum reuse

pub struct PolygonDecoder;

impl ChainDecoder for PolygonDecoder {
    type TxSpecific = EthereumTransaction;  // Reuse!

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID is 137 (mainnet) or 80001 (testnet)
        if let Some(chain_id) = tx.chain_id {
            if chain_id != 137 && chain_id != 80001 {
                return Err(/* invalid chain ID */);
            }
        }

        Ok(tx)
    }
}
```

**Dependencies**:
- Production: `decoder-ethereum`
- Dev: None

**Complexity**: Very Low ✅ (Direct Ethereum reuse)

---

### 10. Avalanche C-Chain

**Chain Family**: Account (EVM)
**Chain ID**: 43114
**Consensus**: Avalanche Consensus (Snowman for C-Chain)

**Transaction Format**:
- RLP-encoded (EVM-compatible on C-Chain)
- X-Chain and P-Chain have custom formats
- Focus on C-Chain (majority of transactions)

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding (C-Chain only, reuse Ethereum)
// Phase 2: Pure Rust (C-Chain RLP + custom X/P chain formats)

// C-Chain (EVM-compatible):
pub struct AvalancheCChainDecoder;

impl ChainDecoder for AvalancheCChainDecoder {
    type TxSpecific = EthereumTransaction;  // Reuse for C-Chain
    type Chain = AvalancheCChain;
}

// X-Chain (UTXO-based, custom format):
pub struct AvalancheXChainDecoder;  // Future work

// P-Chain (Platform chain, staking/subnets):
pub struct AvalanchePChainDecoder;  // Future work
```

**Dependencies** (dev-only):
- `alloy-primitives` - For C-Chain validation

**Validation Strategy**:
- Test against C-Chain transactions
- Document X/P-Chain for future implementation

**Estimated Complexity**: Low (C-Chain), High (X/P-Chain in future)

---

### 11. Polkadot

**Chain Family**: Substrate
**Chain ID**: 0 (relay chain)
**Consensus**: GRANDPA + BABE

**Transaction Format**:
- SCALE encoding (Simple Concatenated Aggregate Little-Endian)
- Extrinsics (signed and unsigned)
- Multi-chain architecture (relay chain + parachains)

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with parity-scale-codec
// Phase 2: Pure Rust SCALE decoder

// Key components:
// 1. SCALE decoder
// 2. Extrinsic parser (signed, unsigned, inherent)
// 3. Pallet calls (Balances, Staking, etc.)

// Extrinsic structure:
pub struct Extrinsic {
    pub version: u8,
    pub signature: Option<ExtrinsicSignature>,
    pub call: Call,
}

pub struct ExtrinsicSignature {
    pub signer: AccountId,
    pub signature: MultiSignature,
    pub era: Era,
    pub nonce: u32,
    pub tip: Balance,
}

pub enum Call {
    Balances(BalancesCall),
    Staking(StakingCall),
    Utility(UtilityCall),
    // ... many pallets
}
```

**Dependencies** (dev-only):
- `parity-scale-codec` - SCALE encoding
- `sp-runtime` - For validation

**Validation Strategy**:
- Test against Polkadot relay chain extrinsics
- Verify SCALE encoding correctness
- Validate signature schemes (Sr25519, Ed25519, ECDSA)

**Estimated Complexity**: High (SCALE encoding + pallet complexity)

---

### 12. Litecoin

**Chain Family**: UTXO
**Chain ID**: 2
**Consensus**: Proof of Work (Scrypt)

**Transaction Format**:
- Identical to Bitcoin
- SegWit support (activated 2017)
- Same serialization format

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding (reuse Bitcoin decoder)
// Phase 2: Pure Rust (share parser with Bitcoin)

// Key differences from Bitcoin:
// 1. Different chain ID (2)
// 2. Different address prefixes (L/M for mainnet)
// 3. Faster block time (2.5 min vs 10 min - not relevant for tx decoding)

// Can literally reuse BitcoinDecoder:
pub struct LitecoinDecoder;

impl ChainDecoder for LitecoinDecoder {
    type TxSpecific = BitcoinTransaction;  // Reuse!
    type Chain = LitecoinChain;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        BitcoinDecoder::decode(raw_bytes)  // Same format
    }
}
```

**Dependencies** (dev-only):
- None (reuse Bitcoin)

**Validation Strategy**:
- Test against Litecoin mainnet transactions
- Verify SegWit transactions

**Estimated Complexity**: Very Low (Bitcoin clone)

---

### 13. NEAR Protocol

**Chain Family**: Account
**Chain ID**: 397 (custom)
**Consensus**: Nightshade (sharded PoS)

**Transaction Format**:
- Borsh encoding (same as our canonical format!)
- Action-based (similar to instructions)
- Sharded architecture

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with near-primitives
// Phase 2: Pure Rust Borsh decoder (we already use Borsh!)

// Key components:
// 1. Borsh decoder (already have it!)
// 2. Action parser (CreateAccount, Transfer, DeployContract, FunctionCall, etc.)
// 3. Receipt handling (cross-shard)

// Transaction structure:
pub struct NearTransaction {
    pub signer_id: AccountId,
    pub public_key: PublicKey,
    pub nonce: u64,
    pub receiver_id: AccountId,
    pub block_hash: CryptoHash,
    pub actions: Vec<Action>,
}

pub enum Action {
    CreateAccount,
    DeployContract(DeployContractAction),
    FunctionCall(FunctionCallAction),
    Transfer(TransferAction),
    Stake(StakeAction),
    AddKey(AddKeyAction),
    DeleteKey(DeleteKeyAction),
    DeleteAccount(DeleteAccountAction),
}
```

**Dependencies** (dev-only):
- `near-primitives` - For validation
- `borsh` - Already in workspace!

**Validation Strategy**:
- Test against NEAR mainnet transactions
- Verify Borsh encoding (should match our canonical format)
- Validate cross-shard receipts

**Estimated Complexity**: Medium (Borsh is easy, but sharding adds complexity)

---

### 14. Cosmos Hub

**Chain Family**: Account
**Chain ID**: "cosmoshub-4" (string-based)
**Consensus**: Tendermint BFT

**Transaction Format**:
- Protobuf encoding (Cosmos SDK)
- Amino encoding (legacy, being phased out)
- Message-based architecture

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with cosmos-sdk-proto
// Phase 2: Pure Rust with prost (protobuf)

// Key components:
// 1. Protobuf decoder
// 2. Message type handlers (MsgSend, MsgDelegate, etc.)
// 3. Any type unwrapping (google.protobuf.Any)

// Transaction structure:
pub struct CosmosTransaction {
    pub body: TxBody,
    pub auth_info: AuthInfo,
    pub signatures: Vec<Vec<u8>>,
}

pub struct TxBody {
    pub messages: Vec<Any>,  // google.protobuf.Any
    pub memo: String,
    pub timeout_height: u64,
    pub extension_options: Vec<Any>,
}

pub enum Message {
    MsgSend,
    MsgMultiSend,
    MsgDelegate,
    MsgUndelegate,
    MsgBeginRedelegate,
    // ... many message types
}
```

**Dependencies** (dev-only):
- `prost` - Protobuf library
- `cosmos-sdk-proto` - For validation

**Validation Strategy**:
- Test against Cosmos Hub transactions
- Verify IBC transactions (Inter-Blockchain Communication)
- Validate signature schemes

**Estimated Complexity**: Medium (Protobuf + many message types)

---

### 15. Stellar

**Chain Family**: Account
**Chain ID**: "stellar" (string-based)
**Consensus**: Stellar Consensus Protocol (SCP)

**Transaction Format**:
- XDR encoding (External Data Representation)
- Operation-based (similar to XRP)
- Asset-centric (native multi-asset support)

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with stellar-xdr
// Phase 2: Pure Rust XDR decoder

// Key components:
// 1. XDR decoder
// 2. Operation parser (Payment, PathPaymentStrictReceive, CreateAccount, etc.)
// 3. Asset handling (native XLM + custom assets)

// Transaction structure:
pub struct StellarTransaction {
    pub source_account: AccountId,
    pub fee: u32,
    pub seq_num: i64,
    pub time_bounds: Option<TimeBounds>,
    pub memo: Memo,
    pub operations: Vec<Operation>,
    pub ext: TransactionExt,
}

pub enum Operation {
    CreateAccount(CreateAccountOp),
    Payment(PaymentOp),
    PathPaymentStrictReceive(PathPaymentStrictReceiveOp),
    ManageSellOffer(ManageSellOfferOp),
    CreatePassiveSellOffer(CreatePassiveSellOfferOp),
    SetOptions(SetOptionsOp),
    ChangeTrust(ChangeTrustOp),
    AllowTrust(AllowTrustOp),
    // ... 20+ operation types
}
```

**Dependencies** (dev-only):
- `stellar-xdr` - XDR encoding library

**Validation Strategy**:
- Test against Stellar mainnet transactions
- Verify XDR encoding
- Validate multi-asset operations

**Estimated Complexity**: High (XDR encoding + many operation types)

---

### 16. Algorand

**Chain Family**: Account
**Chain ID**: 4160 (custom)
**Consensus**: Pure Proof of Stake

**Transaction Format**:
- MessagePack encoding (msgpack)
- Transaction types: Payment, KeyReg, AssetConfig, AssetTransfer, AssetFreeze, AppCall
- AVM (Algorand Virtual Machine) for smart contracts

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with algorand-rs (if exists) or manual msgpack
// Phase 2: Pure Rust with rmp-serde (msgpack library)

// Key components:
// 1. MessagePack decoder
// 2. Transaction type handlers
// 3. AVM application calls

// Transaction structure:
pub struct AlgorandTransaction {
    pub txn_type: TransactionType,
    pub sender: Address,
    pub fee: u64,
    pub first_valid: u64,
    pub last_valid: u64,
    pub note: Vec<u8>,
    pub genesis_id: String,
    pub genesis_hash: Hash,
    pub group: Option<Hash>,
    pub lease: Option<Hash>,
    pub rekey_to: Option<Address>,
    // Type-specific fields based on txn_type
}

pub enum TransactionType {
    Payment { receiver: Address, amount: u64, close_remainder_to: Option<Address> },
    KeyRegistration { vote_pk: Vec<u8>, selection_pk: Vec<u8>, vote_first: u64, vote_last: u64, vote_key_dilution: u64 },
    AssetConfig { /* ... */ },
    AssetTransfer { /* ... */ },
    AssetFreeze { /* ... */ },
    ApplicationCall { /* ... */ },
}
```

**Dependencies** (dev-only):
- `rmp-serde` - MessagePack encoding

**Validation Strategy**:
- Test against Algorand mainnet transactions
- Verify MessagePack encoding
- Validate AVM application calls

**Estimated Complexity**: Medium (MessagePack + AVM)

---

### 17. Optimism

**Chain Family**: Account (EVM, Layer 2)
**Chain ID**: 10
**Consensus**: Optimistic Rollup (single sequencer + fraud proofs)

**Transaction Format**:
- RLP-encoded (EVM-equivalent to Ethereum)
- EIP-2718 transaction types
- Deposit transactions (L1 → L2)
- Withdrawal transactions (L2 → L1)

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding (reuse Ethereum decoder)
// Phase 2: Pure Rust (share RLP parser + add deposit/withdrawal types)

// Key differences from Ethereum:
// 1. Different chain ID (10)
// 2. Deposit transactions (0x7E type)
// 3. L1 data fee calculation
// 4. Different fee structure

pub struct OptimismDecoder;

impl ChainDecoder for OptimismDecoder {
    type TxSpecific = OptimismTransaction;
    type Chain = OptimismChain;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Check for deposit transaction (0x7E)
        if raw_bytes[0] == 0x7E {
            decode_deposit_transaction(raw_bytes)
        } else {
            // Standard Ethereum transaction
            EthereumDecoder::decode(raw_bytes)
        }
    }
}

pub enum OptimismTransaction {
    Standard(EthereumTransaction),
    Deposit(DepositTransaction),
}
```

**Dependencies** (dev-only):
- `alloy-primitives` - For validation

**Validation Strategy**:
- Test against Optimism mainnet transactions
- Verify deposit transactions
- Validate L1 data fee calculation

**Estimated Complexity**: Low (Ethereum + deposit transactions)

---

### 18. Arbitrum

**Chain Family**: Account (EVM, Layer 2)
**Chain ID**: 42161
**Consensus**: Optimistic Rollup (ArbOS)

**Transaction Format**:
- RLP-encoded (EVM-compatible)
- EIP-2718 transaction types
- Retryable tickets (L1 → L2)
- ArbOS special transaction types

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding (reuse Ethereum decoder)
// Phase 2: Pure Rust (share RLP parser + add Arbitrum-specific types)

// Key differences from Ethereum:
// 1. Different chain ID (42161)
// 2. ArbOS internal transactions
// 3. Retryable tickets
// 4. Different gas pricing (L1 + L2 components)

pub struct ArbitrumDecoder;

impl ChainDecoder for ArbitrumDecoder {
    type TxSpecific = ArbitrumTransaction;
    type Chain = ArbitrumChain;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Check for ArbOS internal transactions
        // Most transactions are standard Ethereum format
        EthereumDecoder::decode(raw_bytes)
    }
}
```

**Dependencies** (dev-only):
- `alloy-primitives` - For validation

**Validation Strategy**:
- Test against Arbitrum mainnet transactions
- Verify retryable tickets
- Validate gas pricing

**Estimated Complexity**: Low (Ethereum + Arbitrum-specific types)

---

### 19. Sui

**Chain Family**: Instruction (Object-centric)
**Chain ID**: 0 (custom)
**Consensus**: Narwhal + Bullshark (DAG-based)

**Transaction Format**:
- BCS encoding (Binary Canonical Serialization, similar to Borsh)
- Object-centric model
- Move-based smart contracts
- Programmable transaction blocks (PTBs)

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with sui-types
// Phase 2: Pure Rust BCS decoder

// Key components:
// 1. BCS decoder (similar to Borsh)
// 2. Programmable transaction block parser
// 3. Object reference handling
// 4. Move module/function calls

// Transaction structure:
pub struct SuiTransaction {
    pub data: TransactionData,
    pub tx_signatures: Vec<Signature>,
}

pub struct TransactionData {
    pub kind: TransactionKind,
    pub sender: SuiAddress,
    pub gas_data: GasData,
    pub expiration: TransactionExpiration,
}

pub enum TransactionKind {
    ProgrammableTransaction(ProgrammableTransaction),
    // Legacy types (deprecated):
    // Call, Publish, ChangeEpoch, etc.
}

pub struct ProgrammableTransaction {
    pub inputs: Vec<CallArg>,
    pub commands: Vec<Command>,
}
```

**Dependencies** (dev-only):
- `bcs` - Binary Canonical Serialization
- `sui-types` - For validation

**Validation Strategy**:
- Test against Sui mainnet transactions
- Verify BCS encoding
- Validate PTB execution

**Estimated Complexity**: High (object model + PTBs)

---

### 20. Aptos

**Chain Family**: Account
**Chain ID**: 1
**Consensus**: AptosBFT (variant of HotStuff)

**Transaction Format**:
- BCS encoding (Binary Canonical Serialization)
- Move-based smart contracts
- Parallel execution (Block-STM)
- Multi-agent transactions

**Implementation Strategy**:
```rust
// Phase 1: Scaffolding with aptos-types
// Phase 2: Pure Rust BCS decoder

// Key components:
// 1. BCS decoder (similar to Borsh)
// 2. Transaction payload parser (script, module, entry function)
// 3. Multi-agent support
// 4. Account authenticator (Ed25519, MultiEd25519, MultiAgent)

// Transaction structure:
pub struct AptosTransaction {
    pub sender: AccountAddress,
    pub sequence_number: u64,
    pub payload: TransactionPayload,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_timestamp_secs: u64,
    pub chain_id: ChainId,
    pub authenticator: TransactionAuthenticator,
}

pub enum TransactionPayload {
    Script(Script),
    ModuleBundle(ModuleBundle),
    EntryFunction(EntryFunction),
    Multisig(Multisig),
}

pub enum TransactionAuthenticator {
    Ed25519 { public_key: Ed25519PublicKey, signature: Ed25519Signature },
    MultiEd25519 { public_key: MultiEd25519PublicKey, signature: MultiEd25519Signature },
    MultiAgent { sender: AccountAuthenticator, secondary_signers: Vec<AccountAuthenticator> },
}
```

**Dependencies** (dev-only):
- `bcs` - Binary Canonical Serialization
- `aptos-types` - For validation

**Validation Strategy**:
- Test against Aptos mainnet transactions
- Verify BCS encoding
- Validate multi-agent transactions

**Estimated Complexity**: High (BCS + Move + parallel execution model)

---

## Implementation Phases

### Phase 1: Scaffolding (Current) ✅

**Goal**: Create minimal decoder crates for all 17 chains

**Deliverables**:
- [ ] Cargo.toml for each decoder
- [ ] lib.rs with ChainIdentity impl
- [ ] ChainDecoder trait stub
- [ ] Basic tests (format validation only)
- [ ] README.md with implementation plan

**Timeline**: 1 day

### Phase 2: Pure Rust Implementation (Next)

**Grouped by complexity**:

**Week 1-2: EVM Clones (Very Low Complexity)**
- BNB Chain (reuse Ethereum)
- Polygon (reuse Ethereum)
- Avalanche C-Chain (reuse Ethereum)
- Optimism (Ethereum + deposits)
- Arbitrum (Ethereum + retryables)

**Week 3-4: Bitcoin Forks (Very Low Complexity)**
- Dogecoin (reuse Bitcoin, no SegWit)
- Litecoin (reuse Bitcoin, with SegWit)

**Week 5-8: Medium Complexity**
- NEAR (Borsh encoding - we already use it!)
- Tron (Protobuf)
- Cosmos (Protobuf)
- Algorand (MessagePack)

**Week 9-12: High Complexity**
- XRP (custom binary format)
- Cardano (CBOR + eUTXO)
- Stellar (XDR encoding)
- Polkadot (SCALE encoding)
- Sui (BCS + object model)
- Aptos (BCS + Move)

### Phase 3: Testing & Validation

**For each decoder**:
1. Unit tests (parser correctness)
2. Integration tests (real transactions)
3. Property tests (fuzz testing)
4. Validation against official libraries

### Phase 4: Documentation & Examples

**For each decoder**:
1. API documentation
2. Architecture decision records
3. Example usage
4. Migration guides (from official libraries)

---

## Dependency Strategy

### Core Principle: Dev-Dependencies Only

All blockchain-specific libraries (bitcoin, alloy, parity-scale-codec, etc.) are in **dev-dependencies** for validation testing only. Production code uses **pure Rust** parsing.

### Shared Parsing Libraries (Allowed in Production)

These are allowed because they are:
1. Battle-tested
2. Security-audited
3. Minimal TCB
4. Used across multiple decoders

**Encoding Libraries**:
- `borsh` - Already in workspace (NEAR, Sui, Aptos use BCS which is similar)
- `prost` - Protobuf (Tron, Cosmos)
- `bcs` - Binary Canonical Serialization (Sui, Aptos)
- `parity-scale-codec` - SCALE (Polkadot)
- `rmp-serde` - MessagePack (Algorand)

**Cryptography**:
- `sha2` - Already in workspace
- `sha3` - Already in workspace
- `blake2` - For Polkadot
- `ed25519-dalek` - For Solana, NEAR, Aptos
- `k256` - For EVM chains (ECDSA)

### Minimizing Dependencies

**Reuse existing decoders**:
- 5 EVM chains → 1 Ethereum decoder (with chain ID variations)
- 2 Bitcoin forks → 1 Bitcoin decoder (with SegWit flag)
- Total: 13 unique decoder implementations (not 17)

**Estimated dependency count per decoder**:
- EVM clones: 0 new deps (reuse Ethereum)
- Bitcoin forks: 0 new deps (reuse Bitcoin)
- Protobuf chains: 1 dep (`prost`)
- Custom encoding chains: 1 dep each (their encoding lib)

---

## Success Criteria

### Scaffolding Complete When:
- ✅ All 17 decoder crates created
- ✅ Workspace Cargo.toml updated
- ✅ Each decoder has ChainIdentity impl
- ✅ Each decoder has README with plan
- ✅ All decoders compile (even with stub implementations)
- ✅ Basic format validation tests pass

### Pure Rust Implementation Complete When:
- ✅ Zero production dependencies on blockchain libs
- ✅ All decoders parse real mainnet transactions
- ✅ 100% test coverage on parsers
- ✅ Property tests for all decoders
- ✅ Validation tests pass (against official libs)

### Production Ready When:
- ✅ Security audit completed
- ✅ Formal verification (Verus) for critical paths
- ✅ Benchmarks show acceptable performance
- ✅ Documentation complete
- ✅ Examples for all chains

---

## Risk Assessment

### High Risk

1. **XRP Binary Format**: No standard Rust library, custom format
   - **Mitigation**: Extensive testing against XRP Ledger, formal spec

2. **Cardano CBOR Complexity**: eUTXO model is complex
   - **Mitigation**: Use pallas for validation, property testing

3. **Polkadot SCALE Encoding**: Many pallets, runtime upgrades
   - **Mitigation**: Start with core pallets only, expand later

### Medium Risk

1. **Move-based Chains (Sui, Aptos)**: Object model is novel
   - **Mitigation**: Use official BCS library, validate against nodes

2. **Layer 2s (Optimism, Arbitrum)**: Evolving standards
   - **Mitigation**: Track EIP updates, test against testnets

### Low Risk

1. **EVM Clones**: Well-understood, reuse Ethereum decoder
2. **Bitcoin Forks**: Minimal changes from Bitcoin

---

## References

### Chain Documentation

1. **BNB Chain**: https://docs.bnbchain.org/
2. **XRP**: https://xrpl.org/
3. **Cardano**: https://docs.cardano.org/
4. **Dogecoin**: https://github.com/dogecoin/dogecoin
5. **Tron**: https://developers.tron.network/
6. **Polygon**: https://docs.polygon.technology/
7. **Avalanche**: https://docs.avax.network/
8. **Polkadot**: https://wiki.polkadot.network/
9. **Litecoin**: https://litecoin.org/
10. **NEAR**: https://docs.near.org/
11. **Cosmos**: https://docs.cosmos.network/
12. **Stellar**: https://developers.stellar.org/
13. **Algorand**: https://developer.algorand.org/
14. **Optimism**: https://docs.optimism.io/
15. **Arbitrum**: https://docs.arbitrum.io/
16. **Sui**: https://docs.sui.io/
17. **Aptos**: https://aptos.dev/

### Encoding Specifications

- **RLP**: https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/
- **CBOR**: https://cbor.io/
- **Protobuf**: https://protobuf.dev/
- **SCALE**: https://docs.substrate.io/reference/scale-codec/
- **BCS**: https://github.com/diem/bcs
- **MessagePack**: https://msgpack.org/
- **XDR**: https://datatracker.ietf.org/doc/html/rfc4506

---

**Last Updated**: 2025-11-12
**Status**: Living Document
**Next Review**: After Phase 1 completion
