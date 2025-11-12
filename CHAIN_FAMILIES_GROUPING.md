# Chain Families & Ecosystem Grouping Strategy

**Extends**: `NEXT_STEPS_CHAINLIST_INTEGRATION.md`
**Status**: Planning
**Priority**: High

---

## Overview

Many blockchain ecosystems spawn multiple chains using the same underlying technology. Instead of creating decoders for each chain, we can create **family-based decoders** that handle all chains in an ecosystem.

---

## Chain Families

### 1. EVM Family (Standard)

**Technology**: Ethereum Virtual Machine
**Chains**: 500+ (Ethereum, BSC, Polygon, Avalanche C-Chain, etc.)
**Decoder**: `decoder-evm` (planned)

**Characteristics**:
- RLP encoding
- EIP-2718 transaction types
- Standard gas model
- No custom transaction types

---

### 2. Optimism Stack (OP Stack)

**Technology**: Optimism Bedrock
**Chains**: Optimism, Base, Zora, Mode, Public Goods Network, Manta Pacific, etc.
**Decoder**: `decoder-op-stack`

**Custom Features**:
- Deposit transactions (0x7E)
- L1 data fee calculation
- Bedrock vs legacy format

**Chain List** (as of 2025):
| Chain | Chain ID | Notes |
|-------|----------|-------|
| Optimism | 10 | Original OP Stack chain |
| Base | 8453 | Coinbase L2 |
| Zora | 7777777 | NFT-focused |
| Public Goods Network | 424 | Gitcoin |
| Mode | 34443 | DeFi-focused |
| Orderly | 291 | Trading |
| Cyber | 7560 | Social |
| Fraxtal | 252 | Frax Finance |

**Implementation**:
```rust
// decoder-op-stack/src/lib.rs

pub struct OpStackDecoder {
    evm: EvmDecoder,
    op_stack_chains: HashSet<u64>,
}

impl OpStackDecoder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            evm: EvmDecoder::new()?,
            op_stack_chains: HashSet::from([
                10, 8453, 7777777, 424, 34443, 291, 7560, 252,
                // Load from config/chainlist
            ]),
        })
    }

    pub fn decode(&self, raw_bytes: &[u8], chain_id: Option<u64>) -> Result<OpStackTransaction> {
        // Check for deposit transaction (0x7E)
        if raw_bytes.first() == Some(&0x7E) {
            return Ok(OpStackTransaction::Deposit(
                self.decode_deposit_transaction(raw_bytes)?
            ));
        }

        // Standard EVM transaction
        let (tx, chain_info) = self.evm.decode(raw_bytes, chain_id)?;

        // Validate it's an OP Stack chain
        if let Some(cid) = chain_id {
            if !self.op_stack_chains.contains(&cid) {
                return Err(DecoderError::invalid_structure(
                    format!("Chain {} is not an OP Stack chain", cid)
                ));
            }
        }

        Ok(OpStackTransaction::Standard(tx, chain_info))
    }
}

pub enum OpStackTransaction {
    Standard(EthereumTransaction, ChainInfo),
    Deposit(DepositTransaction),
}

#[derive(Debug, Clone)]
pub struct DepositTransaction {
    pub source_hash: [u8; 32],
    pub from: [u8; 20],
    pub to: Option<[u8; 20]>,
    pub mint: u128,          // L2 ETH minted
    pub value: u128,
    pub gas: u64,
    pub is_system_tx: bool,
    pub data: Vec<u8>,
}
```

**Benefits**:
- Single decoder for 10+ OP Stack chains
- Automatic support for new OP Stack deployments
- Shared deposit transaction logic

---

### 3. Arbitrum Orbit

**Technology**: Arbitrum Nitro
**Chains**: Arbitrum One, Arbitrum Nova, Xai, Proof of Play, etc.
**Decoder**: `decoder-arbitrum-orbit`

**Custom Features**:
- Retryable tickets (L1 → L2 messaging)
- ArbOS internal transactions
- Custom precompiles

**Chain List**:
| Chain | Chain ID | Notes |
|-------|----------|-------|
| Arbitrum One | 42161 | Main L2 |
| Arbitrum Nova | 42170 | Gaming-focused |
| Xai | 660279 | Gaming L3 |
| Proof of Play | 70700 | Gaming |
| Rari | 1380012617 | L3 |

**Implementation**:
```rust
// decoder-arbitrum-orbit/src/lib.rs

pub struct ArbitrumOrbitDecoder {
    evm: EvmDecoder,
    orbit_chains: HashSet<u64>,
}

pub enum ArbitrumTransaction {
    Standard(EthereumTransaction, ChainInfo),
    Retryable(RetryableTicket),
    ArbosInternal(ArbosTransaction),
}

impl ArbitrumOrbitDecoder {
    pub fn decode(&self, raw_bytes: &[u8], chain_id: Option<u64>) -> Result<ArbitrumTransaction> {
        // Detect transaction type
        let tx_type = self.detect_arbitrum_tx_type(raw_bytes)?;

        match tx_type {
            ArbitrumTxType::Retryable => {
                Ok(ArbitrumTransaction::Retryable(
                    self.decode_retryable(raw_bytes)?
                ))
            }
            ArbitrumTxType::ArbosInternal => {
                Ok(ArbitrumTransaction::ArbosInternal(
                    self.decode_arbos_internal(raw_bytes)?
                ))
            }
            ArbitrumTxType::Standard => {
                let (tx, chain_info) = self.evm.decode(raw_bytes, chain_id)?;
                Ok(ArbitrumTransaction::Standard(tx, chain_info))
            }
        }
    }
}
```

---

### 4. zkSync Era & Ecosystem

**Technology**: zkSync Era (zkEVM)
**Chains**: zkSync Era, ZKFair, Cronos zkEVM, etc.
**Decoder**: `decoder-zksync-era`

**Custom Features**:
- Custom transaction encoding (not RLP)
- Account abstraction (EIP-4337)
- Paymaster support
- Custom signature schemes

**Chain List**:
| Chain | Chain ID | Notes |
|-------|----------|-------|
| zkSync Era | 324 | Main zkEVM |
| zkSync Era Testnet | 280 | Testnet |
| ZKFair | 42766 | Community fork |

**Implementation**:
```rust
// decoder-zksync-era/src/lib.rs

pub struct ZkSyncEraDecoder;

pub enum ZkSyncTransaction {
    Legacy(LegacyTransaction),
    Eip712(Eip712Transaction),  // zkSync custom
    Eip1559(Eip1559Transaction),
    Eip2930(Eip2930Transaction),
}

impl ZkSyncEraDecoder {
    pub fn decode(&self, raw_bytes: &[u8]) -> Result<ZkSyncTransaction> {
        // zkSync uses custom encoding, not standard RLP
        // First byte indicates transaction type

        match raw_bytes.first() {
            Some(0x71) => Ok(ZkSyncTransaction::Eip712(
                self.decode_eip712(raw_bytes)?
            )),
            Some(0x02) => Ok(ZkSyncTransaction::Eip1559(
                self.decode_eip1559(raw_bytes)?
            )),
            Some(0x01) => Ok(ZkSyncTransaction::Eip2930(
                self.decode_eip2930(raw_bytes)?
            )),
            _ => Ok(ZkSyncTransaction::Legacy(
                self.decode_legacy(raw_bytes)?
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Eip712Transaction {
    pub tx_type: u8,           // 0x71
    pub nonce: u64,
    pub gas_limit: u128,
    pub to: [u8; 20],
    pub value: u128,
    pub data: Vec<u8>,
    pub gas_per_pubdata: u128,  // zkSync specific
    pub custom_signature: Vec<u8>,
    pub paymaster_params: PaymasterParams,  // zkSync specific
    pub factory_deps: Vec<Vec<u8>>,  // Contract deployments
}

#[derive(Debug, Clone)]
pub struct PaymasterParams {
    pub paymaster: [u8; 20],
    pub paymaster_input: Vec<u8>,
}
```

---

### 5. Polygon CDK (Chain Development Kit)

**Technology**: Polygon zkEVM
**Chains**: Polygon zkEVM, Immutable zkEVM, etc.
**Decoder**: `decoder-polygon-cdk`

**Custom Features**:
- zkEVM-specific transaction types
- Forced batches
- Sequencer/aggregator transactions

**Chain List**:
| Chain | Chain ID | Notes |
|-------|----------|-------|
| Polygon zkEVM | 1101 | Main zkEVM |
| Immutable zkEVM | 13371 | Gaming |
| X Layer | 196 | OKX |

---

### 6. Cosmos SDK Chains

**Technology**: Cosmos SDK + Tendermint
**Chains**: Cosmos Hub, Osmosis, Celestia, dYdX, etc.
**Decoder**: `decoder-cosmos-sdk`

**Format**: Protobuf encoding

**Chain List** (100+ chains):
| Chain | Chain ID | Notes |
|-------|----------|-------|
| Cosmos Hub | cosmoshub-4 | Main hub |
| Osmosis | osmosis-1 | DEX |
| Celestia | celestia | DA layer |
| dYdX | dydx-mainnet-1 | Trading |

**Implementation**:
```rust
// decoder-cosmos-sdk/src/lib.rs

pub struct CosmosSDKDecoder {
    chain_registry: CosmosChainRegistry,
}

impl CosmosSDKDecoder {
    /// Decode Cosmos SDK transaction (any chain)
    pub fn decode(&self, raw_bytes: &[u8], chain_id: Option<&str>) -> Result<CosmosTx> {
        // All Cosmos chains use same Protobuf format
        let tx = self.decode_protobuf(raw_bytes)?;

        // Validate chain ID if provided
        if let Some(expected) = chain_id {
            if tx.auth_info.chain_id != expected {
                return Err(/* chain ID mismatch */);
            }
        }

        Ok(tx)
    }
}
```

---

### 7. Solana VM (SVM) Chains

**Technology**: Solana Virtual Machine (SVM)
**Chains**: Solana, Eclipse, Pyth, Drift, etc.
**Decoder**: `decoder-svm`
**Status**: ✅ **Solana Implemented**

**Format**: Bincode serialization (compact binary)

**Chain List** (SVM ecosystem):
| Chain | Chain ID | Type | Notes |
|-------|----------|------|-------|
| Solana | 101 | L1 | Main SVM chain ✅ |
| Eclipse | TBD | L2 | SVM on Ethereum |
| Pyth Network | pyth | Oracle | Solana-based oracle |
| Drift | drift | App | Trading protocol |
| Jito | jito | Infra | MEV infrastructure |

**Implementation**:
```rust
// decoder-svm/src/lib.rs (extends existing decoder-solana)

pub struct SvmDecoder {
    solana: SolanaDecoder,
    svm_chains: HashSet<u64>,
}

impl SvmDecoder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            solana: SolanaDecoder,
            svm_chains: HashSet::from([
                101,  // Solana mainnet
                102,  // Solana testnet
                103,  // Solana devnet
                // Add SVM-based chains
            ]),
        })
    }

    pub fn decode(&self, raw_bytes: &[u8], chain_id: Option<u64>) -> Result<SvmTransaction> {
        // All SVM chains use same transaction format as Solana
        let tx = SolanaDecoder::decode(raw_bytes)?;

        // Validate chain ID if provided
        if let Some(cid) = chain_id {
            if !self.svm_chains.contains(&cid) {
                return Err(DecoderError::invalid_structure(
                    format!("Chain {} is not an SVM chain", cid)
                ));
            }
        }

        Ok(SvmTransaction { tx, chain_id })
    }
}

#[derive(Debug, Clone)]
pub struct SvmTransaction {
    pub tx: SolanaTransaction,
    pub chain_id: Option<u64>,
}

// Reuse existing Solana types
pub use decoder_solana::{
    SolanaTransaction,
    Message,
    MessageHeader,
    CompiledInstruction,
};
```

**Key Features**:
- **Compact-u16 encoding**: Variable-length integers
- **Account-based model**: Instructions reference account indices
- **Program execution**: Each instruction calls a program
- **Signature verification**: Ed25519 signatures
- **Versioned transactions**: v0 (legacy) and v1 (with address lookups)

**Transaction Structure**:
```rust
pub struct SolanaTransaction {
    pub signatures: Vec<[u8; 64]>,  // Ed25519 signatures
    pub message: Message,
}

pub struct Message {
    pub header: MessageHeader,
    pub account_keys: Vec<[u8; 32]>,
    pub recent_blockhash: [u8; 32],
    pub instructions: Vec<CompiledInstruction>,
}

pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}

pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,      // Account indices
    pub data: Vec<u8>,          // Program-specific data
}
```

**SVM Ecosystem Chains**:

1. **Solana** (101) - Main L1 ✅ Implemented
   - Proof of Stake + Proof of History
   - ~400ms block time
   - ~50k TPS

2. **Eclipse** - SVM on Ethereum
   - Uses Solana runtime
   - Settles to Ethereum
   - Same transaction format as Solana

3. **Pyth Network** - Price oracle
   - Runs on Solana
   - Provides price feeds
   - Crosschain via Wormhole

4. **Nitro SVM** - SVM rollup framework
   - Offchain Labs technology
   - SVM execution, Ethereum settlement

**Benefits of SVM Decoder**:
- Support all SVM-based chains with single decoder
- Reuse Solana parser (already implemented)
- Auto-detect SVM transactions by format

---

### 8. Move-Based Chains

**Technology**: Move VM
**Chains**: Aptos, Sui, Movement, etc.
**Decoder**: `decoder-move`

**Format**: BCS (Binary Canonical Serialization)

**Chain List**:
| Chain | Chain ID | Notes |
|-------|----------|-------|
| Aptos | 1 | LayerZero-based |
| Sui | sui-mainnet | Object-centric |
| Movement | TBD | Move on EVM |

**Implementation**:
```rust
// decoder-move/src/lib.rs

pub struct MoveDecoder {
    variant: MoveVariant,
}

pub enum MoveVariant {
    Aptos,
    Sui,
    Movement,
}

pub enum MoveTransaction {
    Aptos(AptosTransaction),
    Sui(SuiTransaction),
    Movement(MovementTransaction),
}

impl MoveDecoder {
    pub fn decode(&self, raw_bytes: &[u8]) -> Result<MoveTransaction> {
        match self.variant {
            MoveVariant::Aptos => {
                Ok(MoveTransaction::Aptos(
                    decode_aptos_bcs(raw_bytes)?
                ))
            }
            MoveVariant::Sui => {
                Ok(MoveTransaction::Sui(
                    decode_sui_bcs(raw_bytes)?
                ))
            }
            MoveVariant::Movement => {
                Ok(MoveTransaction::Movement(
                    decode_movement(raw_bytes)?
                ))
            }
        }
    }
}
```

---

## Decoder Organization

### Proposed Structure

```
crates/
├── decoder-evm/              # 500+ standard EVM chains
├── decoder-op-stack/         # 10+ OP Stack chains
├── decoder-arbitrum-orbit/   # 5+ Arbitrum Orbit chains
├── decoder-zksync-era/       # zkSync ecosystem
├── decoder-polygon-cdk/      # Polygon CDK chains
├── decoder-cosmos-sdk/       # 100+ Cosmos chains
├── decoder-svm/              # Solana VM chains (Solana, Eclipse, etc.) ✅
├── decoder-move/             # Aptos, Sui, Movement
├── decoder-bitcoin/          # Bitcoin ✅
├── decoder-bitcoin-forks/    # Dogecoin, Litecoin, etc.
├── decoder-substrate/        # Polkadot, Kusama, parachains
└── decoder-specialized/      # XRP, Cardano, Stellar, etc.
```

### Workspace Reduction

**Before** (individual chain approach):
- 500+ EVM chains = 500 crates
- 10 OP Stack chains = 10 crates
- 5 Arbitrum Orbit chains = 5 crates
- 100 Cosmos chains = 100 crates
- 5 SVM chains = 5 crates
- **Total**: 620+ crates

**After** (family approach):
- EVM: 1 crate
- OP Stack: 1 crate
- Arbitrum Orbit: 1 crate
- zkSync Era: 1 crate
- Polygon CDK: 1 crate
- Cosmos SDK: 1 crate
- SVM: 1 crate ✅
- Move: 1 crate
- Specialized: ~10 crates
- **Total**: ~18 crates (97% reduction!)

---

## Chain Registry Schema

### Extended ChainInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: ChainId,
    pub name: String,
    pub short_name: String,

    // Family grouping
    pub family: ChainFamily,
    pub decoder: String,  // "evm", "op-stack", "arbitrum-orbit", etc.

    // Network info
    pub network_id: u64,
    pub is_testnet: bool,

    // Ecosystem metadata
    pub native_currency: CurrencyInfo,
    pub rpc: Vec<String>,
    pub explorers: Vec<ExplorerInfo>,

    // Custom features
    #[serde(default)]
    pub features: Vec<String>,  // ["eip1559", "deposit-tx", "retryable", etc.]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChainId {
    Numeric(u64),      // EVM chains
    String(String),    // Cosmos chains (e.g., "cosmoshub-4")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainFamily {
    Evm,
    OpStack,
    ArbitrumOrbit,
    ZkSyncEra,
    PolygonCdk,
    CosmosSdk,
    Svm,           // Solana Virtual Machine
    Move,
    Bitcoin,
    Substrate,
    Specialized,
}
```

### Data Sources (Vendored for Airgapped Operation)

**CRITICAL**: All chain registries are **vendored via git subtree** to support airgapped deployments.

1. **EVM Chains**: https://github.com/ethereum-lists/chains
   - Vendored to: `crates/decoder-evm/vendored/chainlist`
   - Contains: EIP-155 chain data for 500+ EVM chains
   - Embedded at: Compile time via build.rs

2. **Cosmos Chains**: https://github.com/cosmos/chain-registry
   - Vendored to: `crates/decoder-cosmos-sdk/vendored/chain-registry`
   - Contains: 100+ Cosmos SDK chain specifications
   - Embedded at: Compile time via build.rs

3. **OP Stack**: https://github.com/ethereum-optimism/superchain-registry
   - Vendored to: `crates/decoder-op-stack/vendored/superchain-registry`
   - Contains: All OP Stack chains (Optimism, Base, Zora, etc.)
   - Embedded at: Compile time via build.rs

4. **Arbitrum Orbit**: Manual curation
   - Hardcoded in: `crates/decoder-arbitrum-orbit/src/chains.rs`
   - No external dependency

5. **SVM Chains**: Manual curation
   - Hardcoded in: `crates/decoder-svm/src/chains.rs`
   - Solana, Eclipse, Pyth, etc.

6. **Others**: Manual curation + community contributions

**Vendoring Strategy**:
```bash
# Example: Vendor EVM chains
cd /path/to/universal-blockchain-decoder
git subtree add \
    --prefix crates/decoder-evm/vendored/chainlist \
    https://github.com/ethereum-lists/chains.git \
    master \
    --squash

# Verify vendored commit
git log --oneline crates/decoder-evm/vendored/chainlist | head -1
```

**Benefits**:
- ✅ Works completely offline (airgapped deployments)
- ✅ Verifiable supply chain (git commit history)
- ✅ Reproducible builds (pinned versions)
- ✅ No runtime network dependencies
- ✅ Audit-friendly (all data in repository)

---

## Universal Decoder API

### Unified Interface

```rust
// universal-decoder/src/lib.rs

pub struct UniversalDecoder {
    evm: EvmDecoder,
    op_stack: OpStackDecoder,
    arbitrum_orbit: ArbitrumOrbitDecoder,
    zksync: ZkSyncEraDecoder,
    cosmos: CosmosSDKDecoder,
    svm: SvmDecoder,
    move_vm: MoveDecoder,
    bitcoin: BitcoinDecoder,
    // ... other family decoders
}

impl UniversalDecoder {
    pub fn decode(&self, raw_bytes: &[u8], chain_hint: Option<ChainId>) -> Result<Transaction> {
        // Auto-detect chain family from transaction format or use hint
        let family = self.detect_family(raw_bytes, chain_hint)?;

        match family {
            ChainFamily::Evm => {
                let (tx, chain) = self.evm.decode(raw_bytes, chain_hint.as_numeric())?;
                Ok(Transaction::Evm(tx, chain))
            }
            ChainFamily::OpStack => {
                let tx = self.op_stack.decode(raw_bytes, chain_hint.as_numeric())?;
                Ok(Transaction::OpStack(tx))
            }
            ChainFamily::ArbitrumOrbit => {
                let tx = self.arbitrum_orbit.decode(raw_bytes, chain_hint.as_numeric())?;
                Ok(Transaction::ArbitrumOrbit(tx))
            }
            ChainFamily::CosmosSdk => {
                let tx = self.cosmos.decode(raw_bytes, chain_hint.as_string())?;
                Ok(Transaction::Cosmos(tx))
            }
            ChainFamily::Svm => {
                let tx = self.svm.decode(raw_bytes, chain_hint.as_numeric())?;
                Ok(Transaction::Svm(tx))
            }
            ChainFamily::Move => {
                let tx = self.move_vm.decode(raw_bytes)?;
                Ok(Transaction::Move(tx))
            }
            // ... other families
        }
    }

    /// Auto-detect chain family from transaction bytes
    fn detect_family(&self, raw_bytes: &[u8], hint: Option<ChainId>) -> Result<ChainFamily> {
        // Use hint if available
        if let Some(chain_id) = hint {
            return self.registry.get_family(chain_id);
        }

        // Auto-detect from format
        // EVM: starts with RLP list (0xf8+) or typed tx (0x01-0x03)
        // Cosmos: starts with Protobuf tag
        // Solana: Compact-u16 length prefix + signatures
        // Bitcoin: specific structure
        // etc.

        // Check for Solana (SVM) format
        // Solana starts with compact-u16 for signature count, typically 0x01 or 0x02
        // Followed by 64-byte signatures
        if self.detect_solana_format(raw_bytes) {
            return Ok(ChainFamily::Svm);
        }

        match raw_bytes.first() {
            Some(0xf8..=0xff) => Ok(ChainFamily::Evm),  // RLP list
            Some(0x01..=0x03) => Ok(ChainFamily::Evm),  // Typed tx (if not Solana)
            Some(0x7E) => Ok(ChainFamily::OpStack),     // Deposit tx
            Some(0x71) => Ok(ChainFamily::ZkSyncEra),   // EIP-712
            Some(0x0a) => Ok(ChainFamily::CosmosSdk),   // Protobuf
            _ => Err(DecoderError::unknown_format()),
        }
    }

    fn detect_solana_format(&self, raw_bytes: &[u8]) -> bool {
        // Solana format detection:
        // - Starts with compact-u16 length (1-3 bytes)
        // - Followed by N × 64-byte Ed25519 signatures
        // - Typically has 1-2 signatures

        if raw_bytes.len() < 66 {  // Minimum: 1 byte length + 1 signature (64 bytes)
            return false;
        }

        // Check if first byte is valid compact-u16 length (1-4 typically)
        let first_byte = raw_bytes[0];
        if first_byte > 10 {  // Solana txs rarely have >10 signatures
            return false;
        }

        // Heuristic: If length is 1-4 and total size matches signature pattern
        // This is a simplified check - actual implementation would be more robust
        true  // Placeholder - proper implementation in actual code
    }
}

pub enum Transaction {
    Evm(EthereumTransaction, ChainInfo),
    OpStack(OpStackTransaction),
    ArbitrumOrbit(ArbitrumTransaction),
    ZkSyncEra(ZkSyncTransaction),
    Cosmos(CosmosTx),
    Svm(SvmTransaction),
    Move(MoveTransaction),
    Bitcoin(BitcoinTransaction),
    // ... others
}
```

### Example Usage

```rust
use universal_decoder::UniversalDecoder;

let decoder = UniversalDecoder::new()?;

// Decode without knowing the chain
let tx_bytes = hex::decode("f86c...")?;
let tx = decoder.decode(&tx_bytes, None)?;

match tx {
    Transaction::Evm(tx, chain) => {
        println!("EVM chain: {} (ID: {})", chain.name, chain.chain_id);
        println!("Value: {}", tx.value);
    }
    Transaction::OpStack(OpStackTransaction::Deposit(deposit)) => {
        println!("OP Stack deposit transaction");
        println!("Minted: {}", deposit.mint);
    }
    Transaction::Cosmos(tx) => {
        println!("Cosmos chain: {}", tx.chain_id);
    }
    Transaction::Svm(tx) => {
        println!("Solana VM chain");
        println!("Signatures: {}", tx.tx.signatures.len());
        println!("Instructions: {}", tx.tx.message.instructions.len());
    }
    _ => {}
}

// Or with chain hint
let tx = decoder.decode(&tx_bytes, Some(ChainId::Numeric(10)))?;  // Optimism
```

---

## Implementation Priority

### Phase 1: EVM Ecosystem (Week 1-2)
- `decoder-evm` - Standard EVM chains ✅
- `decoder-op-stack` - OP Stack chains 🔄
- `decoder-arbitrum-orbit` - Arbitrum Orbit 🔄

### Phase 2: zkEVM Ecosystem (Week 3-4)
- `decoder-zksync-era` - zkSync ecosystem
- `decoder-polygon-cdk` - Polygon CDK chains

### Phase 3: VM Ecosystems (Week 5-7)
- `decoder-svm` - Solana VM chains ✅ (Base already implemented)
- `decoder-cosmos-sdk` - 100+ Cosmos chains
- `decoder-move` - Aptos, Sui

### Phase 4: Bitcoin Ecosystem (Week 8)
- `decoder-bitcoin-forks` - Dogecoin, Litecoin, etc. (reuse Bitcoin)

### Phase 5: Integration (Week 9-10)
- `UniversalDecoder` - Unified interface
- Auto-detection logic
- Comprehensive testing

---

## Benefits Summary

**Scalability**:
- 620+ chains → 18 decoders (97% reduction)
- New chain in family: 0 code changes (just registry update)
- SVM chains automatically supported via Solana decoder

**Maintainability**:
- Update once, affects entire family
- Shared testing infrastructure
- Common bug fixes
- Solana decoder improvements benefit all SVM chains

**User Experience**:
- Single import for all chains
- Auto-detection of chain family
- Consistent API across families
- Seamless SVM chain support

**Performance**:
- No overhead vs chain-specific decoders
- O(1) chain family lookup
- Lazy loading of family decoders
- Solana's compact encoding already optimized

---

**Status**: Ready for implementation
**Next Action**: Implement `decoder-op-stack` and `decoder-arbitrum-orbit`
**Estimated Timeline**: 10 weeks for all families
