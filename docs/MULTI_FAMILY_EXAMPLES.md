# Multi-Family Chain Examples

Real-world examples of blockchains that support multiple transaction formats.

## Table of Contents

1. [Cosmos SDK + EVM Chains](#cosmos-sdk--evm-chains)
2. [Substrate + EVM Chains](#substrate--evm-chains)
3. [NEAR + EVM](#near--evm)
4. [Avalanche Multi-VM](#avalanche-multi-vm)
5. [Implementation Patterns](#implementation-patterns)

---

## Cosmos SDK + EVM Chains

### 1. Evmos

**Description**: Cosmos SDK chain with full EVM compatibility

**Families**:
- **Cosmos SDK**: Protobuf-encoded transactions (MsgSend, MsgDelegate, MsgVote)
- **EVM**: RLP-encoded Ethereum transactions (EIP-155, EIP-1559, EIP-2930)

**Chain IDs**:
- Cosmos: `evmos_9001-2`
- EVM: `9001`

**Address Formats**:
- Cosmos: `evmos1...` (Bech32)
- EVM: `0x...` (hex)

**Detection Logic**:
```rust
fn is_evm_transaction(bytes: &[u8]) -> bool {
    !bytes.is_empty() && (bytes[0] <= 0x7f || bytes[0] >= 0xc0)
}
```

**Example Transactions**:
```bash
# Cosmos SDK MsgSend
curl "https://api.mintscan.io/v1/evmos/txs?message.action=/cosmos.bank.v1beta1.MsgSend&limit=1"

# EVM Transfer
curl "https://api.evmos.org/api/v1/tx/0x..."
```

**Use Cases**:
- Run Ethereum dApps on Cosmos infrastructure
- Cross-chain DeFi (Cosmos ↔ Ethereum)
- IBC + EVM interoperability

---

### 2. Kava

**Description**: Cosmos SDK with Ethereum Co-Chain

**Families**:
- **Cosmos SDK**: Primary chain (lending, staking)
- **EVM**: Ethereum Co-Chain (smart contracts)

**Chain IDs**:
- Cosmos: `kava_2222-10`
- EVM: `2222`

**Address Formats**:
- Cosmos: `kava1...`
- EVM: `0x...`

**Unique Features**:
- Separate state machines (Cosmos + Ethereum)
- Bridge between chains
- Different transaction fees (Cosmos gas vs EVM gas)

**Detection Logic**:
```rust
// Similar to Evmos, but with bridge transaction support
fn decode(bytes: &[u8]) -> Result<KavaTransaction> {
    // 1. Try EVM
    if is_evm_transaction(bytes) {
        return Ok(KavaTransaction::Evm(EvmDecoder::decode(bytes)?));
    }

    // 2. Try Cosmos
    if let Ok(tx) = CosmosDecoder::decode(bytes) {
        // Check if it's a bridge transaction
        if is_bridge_transaction(&tx) {
            return Ok(KavaTransaction::Bridge(extract_bridge_data(tx)));
        }
        return Ok(KavaTransaction::Cosmos(tx));
    }

    Err(...)
}
```

---

### 3. Canto

**Description**: Cosmos SDK with EVM module

**Families**:
- **Cosmos SDK**: Governance, staking
- **EVM**: Smart contracts

**Chain IDs**:
- Cosmos: `canto_7700-1`
- EVM: `7700`

**Unique Features**:
- Free public infrastructure (no transaction fees for certain operations)
- EVM tightly integrated with Cosmos SDK

**Detection**: Same as Evmos

---

### 4. Injective

**Description**: Cosmos SDK with EVM (via IBC)

**Families**:
- **Cosmos SDK**: Primary (decentralized exchange)
- **EVM**: Via IBC bridge

**Chain IDs**:
- Cosmos: `injective-1`
- EVM: Connected via IBC

**Unique Features**:
- High-performance DEX
- Orderbook-based trading
- EVM via Inter-Blockchain Communication (IBC)

---

### 5. Cronos

**Description**: Cosmos SDK fork with EVM

**Families**:
- **Cosmos SDK**: Fork of Cosmos SDK
- **EVM**: Full Ethereum compatibility

**Chain IDs**:
- Cosmos: `crypto-org-chain-mainnet-1` (Crypto.org chain)
- EVM: `25` (Cronos EVM)

**Unique Features**:
- Part of Crypto.com ecosystem
- Payment and NFT focus

---

## Substrate + EVM Chains

### 1. Moonbeam (Polkadot Parachain)

**Description**: Substrate-based parachain with full EVM

**Families**:
- **Substrate**: SCALE-encoded extrinsics (Polkadot native)
- **EVM**: Full Ethereum compatibility

**Chain IDs**:
- Parachain ID: `2004`
- EVM Chain ID: `1284`

**Address Formats**:
- Substrate: `0x...` (SS58 format with prefix 1284)
- EVM: `0x...` (standard Ethereum hex)

**Detection Logic**:
```rust
fn is_scale_encoded(bytes: &[u8]) -> bool {
    // Substrate extrinsics have specific format
    // Compact-encoded length prefix + version byte
    if bytes.len() < 2 {
        return false;
    }

    // Check for version byte (currently 4)
    let version = bytes[0] & 0b01111111;
    version == 4 || version == 5
}

fn decode(bytes: &[u8]) -> Result<MoonbeamTransaction> {
    // 1. Try Substrate (SCALE-encoded extrinsic)
    if is_scale_encoded(bytes) {
        return Ok(MoonbeamTransaction::Substrate(
            SubstrateDecoder::decode(bytes)?
        ));
    }

    // 2. Try EVM (RLP-encoded)
    if is_evm_transaction(bytes) {
        return Ok(MoonbeamTransaction::Evm(
            EvmDecoder::decode(bytes)?
        ));
    }

    Err(...)
}
```

**Example Transactions**:
```bash
# Substrate extrinsic
curl "https://moonbeam.api.subscan.io/api/scan/extrinsic"

# EVM transaction
curl "https://api-moonbeam.moonscan.io/api/..."
```

---

### 2. Moonriver (Kusama Parachain)

**Description**: Kusama version of Moonbeam

**Families**:
- **Substrate**: Kusama parachain
- **EVM**: Ethereum compatibility

**Chain IDs**:
- Parachain ID: `2023`
- EVM Chain ID: `1285`

**Detection**: Same as Moonbeam

---

### 3. Astar

**Description**: Substrate with EVM + WASM

**Families**:
- **Substrate**: Native Polkadot
- **EVM**: Ethereum contracts
- **WASM**: ink! smart contracts

**Chain IDs**:
- Parachain ID: `2006`
- EVM Chain ID: `592`

**Detection Logic**:
```rust
pub enum AstarTransaction {
    Substrate(SubstrateTransaction),
    Evm(EvmTransaction),
    Wasm(WasmTransaction),
}

fn decode(bytes: &[u8]) -> Result<AstarTransaction> {
    // Try each format
    if is_scale_encoded(bytes) {
        return Ok(AstarTransaction::Substrate(...));
    }
    if is_evm_transaction(bytes) {
        return Ok(AstarTransaction::Evm(...));
    }
    if is_wasm_contract(bytes) {
        return Ok(AstarTransaction::Wasm(...));
    }
    Err(...)
}
```

---

## NEAR + EVM

### Aurora

**Description**: EVM on NEAR Protocol

**Families**:
- **NEAR**: Borsh-encoded NEAR transactions
- **EVM**: Ethereum transactions (via Aurora engine)

**Chain IDs**:
- NEAR: N/A (network ID: mainnet/testnet)
- Aurora EVM: `1313161554`

**Address Formats**:
- NEAR: `account.near` (human-readable)
- Aurora: `0x...` (Ethereum hex)

**Detection Logic**:
```rust
fn decode(bytes: &[u8]) -> Result<AuroraTransaction> {
    // 1. Try NEAR transaction (Borsh-encoded)
    if bytes.len() > 0 && bytes[0] == 0x00 {  // Borsh encoding marker
        if let Ok(tx) = NearDecoder::decode(bytes) {
            return Ok(AuroraTransaction::Near(tx));
        }
    }

    // 2. Try EVM transaction
    if is_evm_transaction(bytes) {
        return Ok(AuroraTransaction::Evm(
            EvmDecoder::decode(bytes)?
        ));
    }

    Err(...)
}
```

**Unique Features**:
- EVM is a smart contract on NEAR
- Transactions can be NEAR native or EVM-wrapped
- Different fee models (NEAR gas vs EVM gas)

---

## Avalanche Multi-VM

### Avalanche

**Description**: Three chains with different VMs

**Families**:
- **X-Chain**: UTXO-based (like Bitcoin) - for asset transfers
- **C-Chain**: EVM - for smart contracts
- **P-Chain**: Platform chain - for validators/subnets

**Chain IDs**:
- X-Chain: `avm` (Avalanche Virtual Machine)
- C-Chain: `43114` (EVM)
- P-Chain: `platform`

**Detection Logic**:
```rust
pub enum AvalancheTransaction {
    XChain(UtxoTransaction),   // Asset transfers
    CChain(EvmTransaction),    // Smart contracts
    PChain(PlatformTransaction), // Staking
}

fn decode(chain_hint: ChainHint, bytes: &[u8]) -> Result<AvalancheTransaction> {
    match chain_hint {
        ChainHint::XChain => {
            // UTXO format
            Ok(AvalancheTransaction::XChain(UtxoDecoder::decode(bytes)?))
        }
        ChainHint::CChain => {
            // EVM format
            Ok(AvalancheTransaction::CChain(EvmDecoder::decode(bytes)?))
        }
        ChainHint::PChain => {
            // Platform format (custom)
            Ok(AvalancheTransaction::PChain(PlatformDecoder::decode(bytes)?))
        }
    }
}
```

**Note**: Avalanche requires external chain hint (cannot auto-detect from bytes alone)

---

## Implementation Patterns

### Pattern 1: Format-Based Detection (Most Common)

**Used by**: Evmos, Kava, Moonbeam, Aurora

**Strategy**: Inspect first byte(s) to determine format

```rust
fn decode(bytes: &[u8]) -> Result<Transaction> {
    // Try formats in order of likelihood
    if is_format_a(bytes) {
        return decode_as_a(bytes);
    }
    if is_format_b(bytes) {
        return decode_as_b(bytes);
    }
    Err(UnknownFormat)
}

fn is_format_a(bytes: &[u8]) -> bool {
    // Quick check based on format markers
    !bytes.is_empty() && (bytes[0] <= 0x7f || bytes[0] >= 0xc0)
}
```

**Pros**:
- Fast (single byte check)
- No external state needed
- Works offline

**Cons**:
- Can have false positives (need fallback)

---

### Pattern 2: Try-Decode (Fallback)

**Used by**: All chains as fallback

**Strategy**: Try decoding with each decoder until one succeeds

```rust
fn decode(bytes: &[u8]) -> Result<Transaction> {
    // Try decoder A
    if let Ok(tx) = DecoderA::decode(bytes) {
        return Ok(Transaction::A(tx));
    }

    // Try decoder B
    if let Ok(tx) = DecoderB::decode(bytes) {
        return Ok(Transaction::B(tx));
    }

    Err(NoDecoderSucceeded)
}
```

**Pros**:
- Always correct (if any decoder succeeds)
- No false positives

**Cons**:
- Slower (multiple decode attempts)
- Higher CPU usage

---

### Pattern 3: External Hint (Avalanche)

**Used by**: Avalanche (multi-chain)

**Strategy**: Require external information about which chain

```rust
pub enum ChainHint {
    XChain,
    CChain,
    PChain,
}

fn decode(hint: ChainHint, bytes: &[u8]) -> Result<Transaction> {
    match hint {
        ChainHint::XChain => decode_utxo(bytes),
        ChainHint::CChain => decode_evm(bytes),
        ChainHint::PChain => decode_platform(bytes),
    }
}
```

**Pros**:
- No ambiguity
- Fast (single decode)

**Cons**:
- Requires external information
- Not self-contained

---

### Pattern 4: Hybrid (Format + Try-Decode)

**Best practice**

**Strategy**: Fast format check, fallback to try-decode

```rust
fn decode(bytes: &[u8]) -> Result<Transaction> {
    // 1. Fast path: Format detection
    if is_evm_format(bytes) {
        if let Ok(tx) = EvmDecoder::decode(bytes) {
            return Ok(Transaction::Evm(tx));
        }
    }

    // 2. Slow path: Try remaining decoders
    if let Ok(tx) = CosmosDecoder::decode(bytes) {
        return Ok(Transaction::Cosmos(tx));
    }

    Err(NoMatch)
}
```

**Pros**:
- Fast common case
- Reliable fallback

**Cons**:
- Slightly more complex

---

## Testing Multi-Family Chains

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test each family
    #[test]
    fn test_decode_family_a() {
        let bytes = include_bytes!("fixtures/chain_family_a.hex");
        let tx = ChainDecoder::decode(bytes).unwrap();
        assert!(matches!(tx, ChainTransaction::FamilyA(_)));
    }

    #[test]
    fn test_decode_family_b() {
        let bytes = include_bytes!("fixtures/chain_family_b.hex");
        let tx = ChainDecoder::decode(bytes).unwrap();
        assert!(matches!(tx, ChainTransaction::FamilyB(_)));
    }

    // Test format detection
    #[test]
    fn test_format_detection() {
        let evm_bytes = vec![0x02, 0xf8, 0x6c];  // EIP-1559
        assert!(is_evm_transaction(&evm_bytes));

        let cosmos_bytes = vec![0x0a, 0x10, 0x20];  // Protobuf
        assert!(!is_evm_transaction(&cosmos_bytes));
    }
}
```

### Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_never_panics(bytes in arb_small_bytes()) {
        // Decoder should never panic, even on random bytes
        let _ = ChainDecoder::decode(&bytes);
    }

    #[test]
    fn prop_deterministic(bytes in arb_small_bytes()) {
        // Same input always produces same result
        let result1 = ChainDecoder::decode(&bytes);
        let result2 = ChainDecoder::decode(&bytes);

        match (result1, result2) {
            (Ok(tx1), Ok(tx2)) => {
                assert_eq!(tx1.family(), tx2.family());
            }
            (Err(_), Err(_)) => {},
            _ => panic!("Non-deterministic result"),
        }
    }
}
```

---

## Summary Table

| Chain | Families | Detection | Difficulty |
|-------|----------|-----------|------------|
| **Evmos** | Cosmos + EVM | Format byte | Easy |
| **Kava** | Cosmos + EVM + Bridge | Format byte + bridge check | Medium |
| **Canto** | Cosmos + EVM | Format byte | Easy |
| **Injective** | Cosmos + EVM (IBC) | Format byte + IBC | Medium |
| **Cronos** | Cosmos + EVM | Format byte | Easy |
| **Moonbeam** | Substrate + EVM | SCALE vs RLP | Medium |
| **Astar** | Substrate + EVM + WASM | Multiple formats | Hard |
| **Aurora** | NEAR + EVM | Borsh vs RLP | Medium |
| **Avalanche** | UTXO + EVM + Platform | External hint | Hard |

---

## Next Steps

1. **Implement Evmos**: Start with simplest Cosmos + EVM chain
2. **Implement Moonbeam**: Substrate + EVM pattern
3. **Implement Aurora**: NEAR + EVM pattern
4. **Implement Avalanche**: Multi-VM with hints
5. **Generalize patterns**: Create multi-family decoder framework

---

## References

- Evmos: https://docs.evmos.org/
- Kava: https://docs.kava.io/
- Moonbeam: https://docs.moonbeam.network/
- Aurora: https://doc.aurora.dev/
- Avalanche: https://docs.avax.network/
