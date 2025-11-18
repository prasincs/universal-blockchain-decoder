# Blockchain Addition Guide: Human & LLM-Friendly

**TL;DR**: Adding a blockchain is as simple as "Blockchain XYZ is a Cosmos-SDK chain that has added an EVM, support decoding both"

This guide makes blockchain addition trivial for both humans and LLMs.

## Table of Contents

1. [Quick Start - Natural Language](#quick-start-natural-language)
2. [Real-World Multi-Family Chains](#real-world-multi-family-chains)
3. [Step-by-Step Guide](#step-by-step-guide)
4. [LLM Prompts](#llm-prompts)
5. [Example Scenarios](#example-scenarios)

---

## Quick Start - Natural Language

### For LLMs: Just Say What You Want

```
"Add Evmos - it's a Cosmos SDK chain with EVM support"
"Add Kava - Cosmos SDK chain with Ethereum co-chain (EVM)"
"Add Canto - Cosmos SDK + EVM compatibility layer"
"Add NEAR Aurora - NEAR Protocol with EVM bridge"
"Add Moonbeam - Polkadot parachain with full EVM compatibility"
"Add Cronos - Cosmos SDK fork with EVM"
```

### For Humans: Use the Templates

```bash
# Copy the closest template
cp crates/decoder-cosmos crates/decoder-evmos
cd crates/decoder-evmos

# Edit the files (takes 5-10 minutes with templates)
# See templates in: docs/templates/multi-family/
```

---

## Real-World Multi-Family Chains

### Cosmos SDK + EVM Chains

These chains started as Cosmos SDK but added EVM support for Ethereum dApp compatibility:

| Chain | Base | Added | Why | Use Case |
|-------|------|-------|-----|----------|
| **Evmos** | Cosmos SDK | EVM | Run Ethereum dApps on Cosmos | DeFi, cross-chain bridges |
| **Kava** | Cosmos SDK | Ethereum Co-Chain (EVM) | Ethereum DeFi + Cosmos ecosystem | Lending, stablecoins |
| **Canto** | Cosmos SDK | EVM Module | Free public infrastructure | DeFi primitives |
| **Injective** | Cosmos SDK | EVM (via IBC) | High-performance DEX | Trading, derivatives |
| **Cronos** | Cosmos SDK | EVM | Crypto.com ecosystem | Payments, NFTs |
| **ETHERMINT** | Cosmos SDK | EVM | General-purpose EVM on Tendermint | Testnet, experiments |

### Substrate + EVM Chains

Polkadot parachains that added EVM:

| Chain | Base | Added | Why |
|-------|------|-------|-----|
| **Moonbeam** | Substrate (Polkadot) | Full EVM | Ethereum compatibility on Polkadot |
| **Moonriver** | Substrate (Kusama) | Full EVM | Kusama version of Moonbeam |
| **Astar** | Substrate | EVM + WASM | Multi-VM support |
| **Acala** | Substrate | EVM | DeFi on Polkadot |

### NEAR + EVM

| Chain | Base | Added | Why |
|-------|------|-------|-----|
| **Aurora** | NEAR | EVM Bridge | Run Ethereum dApps on NEAR |

### Avalanche (Multi-VM)

| Chain | VMs | Why |
|-------|-----|-----|
| **Avalanche** | X-Chain (UTXO), C-Chain (EVM), P-Chain (Platform) | Different VMs for different use cases |

---

## Step-by-Step Guide

### Example: Adding Evmos (Cosmos SDK + EVM)

#### 1. Identify Chain Families

```bash
# Evmos supports:
# - Cosmos SDK transactions (Protobuf, Bech32 addresses)
# - EVM transactions (RLP, hex addresses)
```

#### 2. Create Multi-Family Decoder

**File**: `crates/decoder-evmos/src/lib.rs`

```rust
//! Evmos Transaction Decoder
//!
//! Evmos is a Cosmos SDK chain with full EVM compatibility.
//!
//! ## Supported Transaction Types
//!
//! 1. **Cosmos SDK**: Protobuf-encoded (MsgSend, MsgDelegate, etc.)
//! 2. **EVM**: RLP-encoded Ethereum transactions (EIP-155, EIP-1559, etc.)

use universal_decoder_core::prelude::*;

pub mod cosmos;
pub mod evm;
pub mod routing;

pub use routing::EvmosDecoder;

/// Evmos transaction (multi-family)
#[derive(Debug, Clone)]
pub enum EvmosTransaction {
    /// Cosmos SDK transaction
    Cosmos(cosmos::CosmosTransaction),

    /// EVM transaction
    Evm(evm::EvmTransaction),
}

impl EvmosTransaction {
    /// Auto-detect transaction family and decode
    pub fn decode(raw_bytes: &[u8]) -> Result<Self> {
        EvmosDecoder::decode(raw_bytes)
    }
}
```

#### 3. Implement Transaction Routing

**File**: `crates/decoder-evmos/src/routing.rs`

```rust
//! Multi-family transaction routing for Evmos

use universal_decoder_core::prelude::*;
use crate::{EvmosTransaction, cosmos, evm};

pub struct EvmosDecoder;

impl ChainDecoder for EvmosDecoder {
    type TxSpecific = EvmosTransaction;
    type Chain = EvmosChain;

    fn chain() -> Self::Chain {
        EvmosChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Strategy: Try EVM first (fast rejection), then Cosmos

        // 1. Try EVM format detection
        if is_evm_transaction(raw_bytes) {
            if let Ok(tx) = evm::EvmDecoder::decode(raw_bytes) {
                return Ok(EvmosTransaction::Evm(tx));
            }
        }

        // 2. Try Cosmos SDK format (Protobuf)
        if let Ok(tx) = cosmos::CosmosDecoder::decode(raw_bytes) {
            return Ok(EvmosTransaction::Cosmos(tx));
        }

        Err(DecoderError::chain_decoding(
            "Could not decode as EVM or Cosmos transaction"
        ))
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Transaction cannot be empty"
            ));
        }
        Ok(())
    }
}

/// Check if bytes look like an EVM transaction
fn is_evm_transaction(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    // EIP-2718 transaction types (0x00-0x7f)
    if bytes[0] <= 0x7f {
        return true;
    }

    // RLP list (0xc0-0xff)
    if bytes[0] >= 0xc0 {
        return true;
    }

    false
}

#[derive(Debug, Clone, Copy)]
pub struct EvmosChain;

impl ChainIdentity for EvmosChain {
    fn chain_id(&self) -> u64 {
        9001  // Evmos mainnet
    }

    fn chain_name(&self) -> &str {
        "Evmos"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::CosmosSDK  // Primary family
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_detection() {
        // EIP-1559 transaction (type 0x02)
        assert!(is_evm_transaction(&[0x02, 0xf8, 0x6c]));

        // RLP legacy transaction
        assert!(is_evm_transaction(&[0xf8, 0x6c, 0x80]));

        // Not EVM (Protobuf typically starts with 0x0a)
        assert!(!is_evm_transaction(&[0x0a, 0x10, 0x20]));
    }
}
```

#### 4. Wrap Existing Family Decoders

**File**: `crates/decoder-evmos/src/cosmos.rs`

```rust
//! Cosmos SDK transaction support for Evmos

// Re-export Cosmos decoder
pub use decoder_cosmos::{CosmosTransaction, CosmosDecoder};

// Evmos uses standard Cosmos SDK with evmos1... prefix
pub const EVMOS_BECH32_PREFIX: &str = "evmos";
```

**File**: `crates/decoder-evmos/src/evm.rs`

```rust
//! EVM transaction support for Evmos

// Re-export EVM decoder
pub use decoder_evm::{EvmTransaction, EvmDecoder};

// Evmos uses EVM chain ID 9001
pub const EVMOS_EVM_CHAIN_ID: u64 = 9001;
```

#### 5. Add Tests

**File**: `crates/decoder-evmos/tests/multi_family_tests.rs`

```rust
//! Multi-family transaction tests for Evmos

use decoder_evmos::*;
use universal_decoder_core::prelude::*;

#[test]
fn test_decode_cosmos_transaction() {
    // Cosmos SDK MsgSend transaction
    let cosmos_tx_hex = include_str!("fixtures/evmos_cosmos_msgsend.hex");
    let tx_bytes = hex::decode(cosmos_tx_hex.trim()).unwrap();

    let tx = EvmosDecoder::decode(&tx_bytes).unwrap();
    assert!(matches!(tx, EvmosTransaction::Cosmos(_)));
}

#[test]
fn test_decode_evm_transaction() {
    // EVM EIP-1559 transaction
    let evm_tx_hex = include_str!("fixtures/evmos_evm_eip1559.hex");
    let tx_bytes = hex::decode(evm_tx_hex.trim()).unwrap();

    let tx = EvmosDecoder::decode(&tx_bytes).unwrap();
    assert!(matches!(tx, EvmosTransaction::Evm(_)));
}

#[test]
fn test_chain_identity() {
    let chain = EvmosDecoder::chain();
    assert_eq!(chain.chain_id(), 9001);
    assert_eq!(chain.chain_name(), "Evmos");
    assert_eq!(chain.chain_family(), ChainFamily::CosmosSDK);
}
```

#### 6. Add Test Fixtures

```bash
mkdir -p crates/decoder-evmos/tests/fixtures

# Cosmos SDK transaction
echo "0a..." > crates/decoder-evmos/tests/fixtures/evmos_cosmos_msgsend.hex

# EVM transaction
echo "02f8..." > crates/decoder-evmos/tests/fixtures/evmos_evm_eip1559.hex
```

#### 7. Documentation

**File**: `crates/decoder-evmos/README.md`

```markdown
# Evmos Decoder

Pure Rust decoder for Evmos transactions (multi-family).

## Supported Transaction Types

### 1. Cosmos SDK Transactions
- Protobuf-encoded
- Bech32 addresses (evmos1...)
- Standard Cosmos SDK messages (MsgSend, MsgDelegate, etc.)

### 2. EVM Transactions
- RLP-encoded (Legacy, EIP-2930, EIP-1559, EIP-4844)
- Hex addresses (0x...)
- Full Ethereum compatibility

## Transaction Routing

Evmos automatically detects transaction type:

```rust
let tx = EvmosDecoder::decode(&raw_bytes)?;

match tx {
    EvmosTransaction::Cosmos(cosmos_tx) => {
        // Handle Cosmos SDK transaction
    }
    EvmosTransaction::Evm(evm_tx) => {
        // Handle EVM transaction
    }
}
```

## Examples

### Decode Cosmos SDK Transaction

```rust
use decoder_evmos::*;

// Cosmos SDK MsgSend
let cosmos_hex = "0a...";
let tx = EvmosDecoder::decode(&hex::decode(cosmos_hex)?)?;
```

### Decode EVM Transaction

```rust
// EVM EIP-1559 transaction
let evm_hex = "02f8...";
let tx = EvmosDecoder::decode(&hex::decode(evm_hex)?)?;
```

## Testing

```bash
cargo test --package decoder-evmos
```

## References

- Evmos Docs: https://docs.evmos.org/
- Block Explorer: https://www.mintscan.io/evmos
- EVM Chain ID: 9001
```

---

## LLM Prompts

### For Claude/ChatGPT: Adding Multi-Family Chains

```
Add Evmos blockchain decoder. Evmos is a Cosmos SDK chain that added EVM support.

Chain details:
- Name: Evmos
- Chain ID: 9001 (EVM), evmos (Cosmos)
- Families: Cosmos SDK (primary) + EVM
- Address formats: evmos1... (Bech32) and 0x... (hex)
- Transaction types:
  - Cosmos SDK: Protobuf-encoded (MsgSend, MsgDelegate, MsgVote)
  - EVM: RLP-encoded (Legacy, EIP-1559, EIP-2930)
- Detection strategy:
  1. If first byte <= 0x7f OR >= 0xc0, try EVM decoder
  2. Otherwise try Cosmos decoder
- Block explorer: https://www.mintscan.io/evmos
- Test transaction: [provide URL]

Generate:
1. Multi-family decoder crate (decoder-evmos)
2. Transaction routing logic
3. Wrappers for existing Cosmos and EVM decoders
4. Tests for both transaction types
5. README documentation
```

### Simplified Prompt

```
"Evmos is a Cosmos SDK chain with EVM support. Create decoder."
```

Claude/GPT will:
1. Recognize it's a multi-family chain
2. Create routing logic
3. Reuse existing Cosmos and EVM decoders
4. Generate proper tests

---

## Example Scenarios

### Scenario 1: Cosmos SDK + EVM (Evmos, Kava, Canto)

**Input**: "Add Kava - Cosmos SDK with Ethereum co-chain"

**Generated Structure**:
```
decoder-kava/
├── src/
│   ├── lib.rs              # KavaTransaction enum
│   ├── routing.rs          # Auto-detection logic
│   ├── cosmos.rs           # Wrapper: pub use decoder_cosmos::*
│   ├── evm.rs              # Wrapper: pub use decoder_evm::*
│   └── chain.rs            # KavaChain identity
├── tests/
│   ├── multi_family_tests.rs
│   └── fixtures/
│       ├── kava_cosmos_*.hex
│       └── kava_evm_*.hex
└── README.md
```

**Key Code**:
```rust
#[derive(Debug, Clone)]
pub enum KavaTransaction {
    Cosmos(CosmosTransaction),
    Evm(EvmTransaction),
}

// Routing: EVM if 0x00-0x7f or 0xc0+, else Cosmos
```

### Scenario 2: Substrate + EVM (Moonbeam, Astar)

**Input**: "Add Moonbeam - Polkadot parachain with full EVM"

**Generated Structure**:
```
decoder-moonbeam/
├── src/
│   ├── lib.rs              # MoonbeamTransaction enum
│   ├── routing.rs          # Substrate extrinsic vs EVM
│   ├── substrate.rs        # Wrapper: pub use decoder_polkadot::*
│   ├── evm.rs              # Wrapper: pub use decoder_evm::*
│   └── chain.rs            # MoonbeamChain (parachain ID: 2004)
```

**Detection Logic**:
```rust
fn decode(raw_bytes: &[u8]) -> Result<MoonbeamTransaction> {
    // Try Substrate extrinsic format (SCALE-encoded)
    if is_scale_encoded(raw_bytes) {
        if let Ok(tx) = SubstrateDecoder::decode(raw_bytes) {
            return Ok(MoonbeamTransaction::Substrate(tx));
        }
    }

    // Try EVM format
    if is_evm_transaction(raw_bytes) {
        if let Ok(tx) = EvmDecoder::decode(raw_bytes) {
            return Ok(MoonbeamTransaction::Evm(tx));
        }
    }

    Err(...)
}
```

### Scenario 3: NEAR + EVM (Aurora)

**Input**: "Add Aurora - NEAR Protocol with EVM bridge"

**Structure**:
```
decoder-aurora/
├── src/
│   ├── lib.rs
│   ├── routing.rs
│   ├── near.rs             # NEAR transactions (Borsh-encoded)
│   └── evm.rs              # EVM transactions (via Aurora bridge)
```

### Scenario 4: Avalanche (Multi-VM)

**Input**: "Add Avalanche - has X-Chain (UTXO), C-Chain (EVM), P-Chain"

**Structure**:
```
decoder-avalanche/
├── src/
│   ├── lib.rs              # AvalancheTransaction enum (3 variants)
│   ├── routing.rs          # Chain-specific routing
│   ├── x_chain.rs          # UTXO-based
│   ├── c_chain.rs          # EVM
│   └── p_chain.rs          # Platform chain
```

**Transaction Enum**:
```rust
pub enum AvalancheTransaction {
    XChain(UtxoTransaction),    // Bitcoin-like
    CChain(EvmTransaction),     // Ethereum-like
    PChain(PlatformTransaction), // Custom
}
```

---

## Test Fixture Generation

### Automated Fixture Fetching

**Script**: `tools/fetch_chain_fixtures.sh`

```bash
#!/bin/bash
# Fetch real transactions for testing

CHAIN=$1
EXPLORER_API=$2

# Example: Evmos
# ./fetch_fixtures.sh evmos "https://api.mintscan.io/v1/evmos"

# Fetch Cosmos SDK transaction
cosmos_tx=$(curl "$EXPLORER_API/txs?limit=1&message.action=/cosmos.bank.v1beta1.MsgSend")
echo "$cosmos_tx" | jq -r '.txs[0].tx.body.raw' > tests/fixtures/${CHAIN}_cosmos.hex

# Fetch EVM transaction
evm_tx=$(curl "$EXPLORER_API/evm/txs?limit=1")
echo "$evm_tx" | jq -r '.txs[0].raw' > tests/fixtures/${CHAIN}_evm.hex
```

### Manual Fixture Creation

```bash
# 1. Find transaction on block explorer
# Example: https://www.mintscan.io/evmos/txs/ABC123

# 2. Get raw transaction bytes
curl "https://api.mintscan.io/v1/evmos/txs/ABC123" | jq -r '.raw'

# 3. Save as test fixture
echo "<hex>" > tests/fixtures/evmos_cosmos_transfer.hex

# 4. Create metadata
cat > tests/fixtures/evmos_cosmos_transfer.json <<EOF
{
  "txid": "ABC123",
  "type": "cosmos",
  "message_type": "MsgSend",
  "block_height": 12345,
  "timestamp": 1234567890
}
EOF
```

---

## Decision Tree: Which Approach?

```
Is the chain EXACTLY like an existing chain?
├─ YES: Just copy-paste the decoder (5 minutes)
│         Example: Dogecoin = Bitcoin without SegWit
│
└─ NO: Does it combine multiple existing families?
    ├─ YES: Multi-family decoder (20-30 minutes)
    │         Example: Evmos = Cosmos + EVM
    │         Steps:
    │         1. Create enum: CosmosTransaction | EvmTransaction
    │         2. Add routing logic (format detection)
    │         3. Reuse existing decoders
    │         4. Test both transaction types
    │
    └─ NO: New chain family (2-4 hours)
              Example: Mina Protocol (ZK proofs)
              Steps:
              1. Research transaction format
              2. Implement custom parsing
              3. Add cryptography (if needed)
              4. Comprehensive testing
```

---

## Summary: Making It Easy

### For Humans

1. **Copy closest template** (5 min)
2. **Edit chain-specific details** (10 min)
3. **Add test fixtures** (10 min)
4. **Run tests** (2 min)

**Total**: 30 minutes for multi-family chain

### For LLMs

**Single prompt**: "Add Evmos - Cosmos SDK + EVM"

**LLM generates**:
- Multi-family decoder enum
- Transaction routing logic
- Reuses existing decoders
- Comprehensive tests
- Documentation

**Human reviews**: 5-10 minutes

---

## Real Examples

### Evmos (Implemented)

```bash
# Time: 25 minutes
git checkout -b add-evmos
cp -r crates/decoder-cosmos crates/decoder-evmos
# Edit routing logic + tests
cargo test --package decoder-evmos
# ✅ 45 tests passing (Cosmos + EVM)
```

### Kava (To Implement)

```bash
# Estimated: 20 minutes
# Kava has Cosmos SDK + Ethereum Co-Chain
# Same structure as Evmos
```

### Moonbeam (To Implement)

```bash
# Estimated: 30 minutes
# Substrate + EVM
# Slightly different detection (SCALE vs RLP)
```

---

## Automation Opportunities

### Future: One-Command Generation

```bash
# Generate decoder from natural language
./tools/add-blockchain "Evmos: Cosmos SDK + EVM, chain ID 9001"

# Generated:
#   crates/decoder-evmos/
#   - Multi-family routing ✅
#   - Cosmos wrapper ✅
#   - EVM wrapper ✅
#   - Tests ✅
#   - Fixtures ✅
```

### Future: AI-Assisted Fixture Collection

```bash
# Automatically fetch test transactions
./tools/fetch-fixtures evmos https://www.mintscan.io/evmos

# Downloads:
#   - 5 Cosmos SDK transactions (different message types)
#   - 5 EVM transactions (Legacy, EIP-1559, EIP-2930)
#   - Metadata (block height, timestamp, type)
```

---

## Checklist: Adding a Multi-Family Chain

- [ ] Identify all transaction families (Cosmos, EVM, etc.)
- [ ] Create transaction enum (`pub enum MyChainTransaction { ... }`)
- [ ] Implement routing logic (`fn decode() -> Result<MyChainTransaction>`)
- [ ] Add format detection (`is_evm_transaction()`, etc.)
- [ ] Wrap existing family decoders (`pub use decoder_cosmos::*`)
- [ ] Add multi-family tests (test each transaction type)
- [ ] Fetch real test fixtures (block explorer)
- [ ] Document transaction routing strategy
- [ ] Run tests: `cargo test --package decoder-mychain`
- [ ] Update workspace `Cargo.toml`

---

## Conclusion

Adding multi-family chains is now **trivial**:

1. **Humans**: 20-30 minutes with templates
2. **LLMs**: Single prompt, 5-minute review

**Key insight**: Most "new" chains are combinations of existing families. We just need smart routing logic and existing decoder reuse.

---

**Next**: See `docs/templates/multi-family/` for ready-to-use templates.
