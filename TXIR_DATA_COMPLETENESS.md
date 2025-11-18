# TxIR Data Completeness Analysis

## Summary

Analysis of what data from Ethereum/EVM transactions is captured in the TxIR (Transaction Intermediate Representation) after decoding.

## ✅ What IS Captured

### Transaction-Level Data (in `TxIR.metadata.extra` as JSON)

| Field | Location | Example |
|-------|----------|---------|
| Transaction Type | `metadata.extra.tx_type` | `"Legacy"`, `"Eip1559"`, `"Eip2930"`, `"Eip4844"` |
| Chain ID | `chain.id` + `metadata.extra.chain_id` | `42161` (Arbitrum), `137` (Polygon) |
| Nonce | `metadata.extra.nonce` | `81033` |
| Gas Limit | `metadata.extra.gas_limit` | `20000000` |
| Gas Price (Legacy) | `metadata.extra.gas_price` | `20000000` |
| Max Fee Per Gas (EIP-1559) | `metadata.extra.max_fee_per_gas` | `10000000` |
| Max Priority Fee (EIP-1559) | `metadata.extra.max_priority_fee_per_gas` | `0` |
| **Access List (EIP-2930/1559)** | `metadata.extra.access_list` | `[{"address":"0x...","storage_keys":["0x..."]}]` |
| Transaction Hash | `metadata.tx_hash` | Keccak256 hash as bytes |
| Transaction Size | `metadata.size` | Size in bytes |

### Operation Data

| Field | Location | Notes |
|-------|----------|-------|
| To Address | `operations[0].contract.bytes` / `.human_readable` | Contract or recipient address |
| Value (ETH amount) | `operations[0].value` | Amount with 18 decimals |
| Call Data | `operations[0].data` | **Full contract call data** (including method selector) |
| Method Selector | `operations[0].method` | First 4 bytes of data |
| Gas Limit | `operations[0].resource_limits.max_units` | Same as transaction gas limit |
| Gas Price | `operations[0].resource_limits.unit_price` | Effective gas price |

### Authorization Data

| Field | Location | Notes |
|-------|----------|-------|
| Signature R | `authorization.signatures[0].data[0..32]` | First 32 bytes |
| Signature S | `authorization.signatures[0].data[32..64]` | Next 32 bytes |
| Signature V | `authorization.signatures[0].data[64]` | Last byte; also in `metadata: {"v":...}` |
| Signature Scheme | `authorization.signature_scheme` | `"Ecdsa"` |

### State Delta Data

| Field | Location | Notes |
|-------|----------|-------|
| Sender Nonce | `state_deltas.account_changes[0].nonce` | Sender's nonce |
| Sender Balance Change | `state_deltas.account_changes[0].balance_change` | Negative value (spent) |
| Recipient Balance Change | `state_deltas.account_changes[1].balance_change` | Positive value (received) |
| Recipient Address | `state_deltas.account_changes[1].address` | Receiving address |

## ⚠️ What is PARTIALLY Captured

### From (Sender) Address

**Status**: Placeholder only

- **Location**: `state_deltas.account_changes[0].address` and `operations[0].from` (for transfers)
- **Current Value**: Empty bytes `[]` or zero address `0x0000...`
- **Reason**: ECDSA public key recovery not implemented
- **To Fix**: Need to implement secp256k1 ECDSA recovery from (v, r, s) signature

**Impact**:
- Sender address cannot be determined from TxIR alone
- Must be recovered separately or provided externally

## ✅ What is Stored But Not in TxIR

### Raw Transaction Bytes

**Status**: Available via `ChainEncoder` trait

- **Method**: `tx.to_bytes()` returns the original raw bytes
- **Purpose**: Enables re-encoding for verification (injective property)
- **Not in TxIR**: Raw bytes are not part of the canonical representation
- **Use Case**: Roundtrip testing, forensic reconstruction

## 📊 Example: Complete Data Mapping

### Arbitrum Transaction
```
Raw TX: 0xf8f083013c898401312d008401312d009447a894c806d0091247b982e31474fc9acb27a48380b884d5d860b5...
```

**Decoded to TxIR:**

```json
{
  "chain": {
    "id": 42161,
    "name": "Arbitrum One",
    "family": "Account"
  },
  "metadata": {
    "tx_hash": "0x...",
    "size": 244,
    "extra": {
      "tx_type": "Legacy",
      "nonce": 81033,
      "gas_limit": 20000000,
      "gas_price": 20000000,
      "max_fee_per_gas": null,
      "max_priority_fee_per_gas": null,
      "chain_id": 42161,
      "access_list": []
    }
  },
  "operations": [
    {
      "ContractCall": {
        "contract": {
          "bytes": "47a894c806d0091247b982e31474fc9acb27a483",
          "human_readable": "0x47a894c806d0091247b982e31474fc9acb27a483"
        },
        "method": "d5d860b5",
        "data": "d5d860b55303875cab9228c24f426ae2fe87081feb69e00c363b98342541612a93da86a31cc9011eb440dc9c0f5d2296c220b1cd4af0a517eb6970acbf449fe175919b800000000000000000000000000000000000000000000000000000000000005ad200000000000000000000000000000000000000000000000000000000aa142a27",
        "value": {
          "value": 0,
          "decimals": 18
        },
        "resource_limits": {
          "max_units": 20000000,
          "unit_price": 20000000,
          "resource_type": "Gas"
        }
      }
    }
  ],
  "authorization": {
    "signatures": [
      {
        "data": "5837f57b369b78c12f9e3bc2d9c6da3ba8be60ae66f84d5096118e5c013e012a5e1deb79e1cd5fb91a8396dc165f01a37c4d08794cc468c0c9c1d565b1c2b1ab85",
        "key_index": 0,
        "metadata": {"v": 84357}
      }
    ],
    "public_keys": [],
    "signature_scheme": "Ecdsa"
  },
  "state_deltas": {
    "account_changes": [
      {
        "address": {
          "bytes": [],
          "human_readable": null
        },
        "nonce": 81033,
        "balance_change": 0
      },
      {
        "address": {
          "bytes": "47a894c806d0091247b982e31474fc9acb27a483",
          "human_readable": "0x47a894c806d0091247b982e31474fc9acb27a483"
        },
        "nonce": null,
        "balance_change": 0
      }
    ]
  }
}
```

## 🔍 Data Verification

### Your Test Transactions

#### Transaction 1 (Arbitrum Legacy)
```
0xf8f083013c898401312d008401312d009447a894c806d0091247b982e31474fc9acb27a48380b884d5d860b5...
```

**Captured in TxIR:**
- ✅ Chain ID: 42161 (Arbitrum One)
- ✅ Nonce: 81033
- ✅ Gas: 20000000
- ✅ To: 0x47a894c806d0091247b982e31474fc9acb27a483
- ✅ Data: 132 bytes fully captured
- ✅ Signatures: R, S, V all present
- ⚠️ From: Not recovered (placeholder)

#### Transaction 2 (Arbitrum EIP-1559)
```
0x02f9013082a4b182192d808398968083092e0294802b65b5d9016621e66003aed0b16615093f328b80b8c5a00597a0...
```

**Captured in TxIR:**
- ✅ Chain ID: 42161 (Arbitrum One)
- ✅ Nonce: 6445
- ✅ Max Fee: 10000000, Priority Fee: 0
- ✅ Gas Limit: 601602
- ✅ To: 0x802b65b5d9016621e66003aed0b16615093f328b
- ✅ Data: 197 bytes fully captured
- ✅ Access List: Empty array [] (this tx has no access list)
- ✅ Signatures: R, S, V all present
- ⚠️ From: Not recovered (placeholder)

## 🎯 Summary

### Complete Capture ✅
- Transaction type, chain ID, nonce
- Gas parameters (price, limit, max fees)
- To address, value, full call data
- Access list (EIP-2930/EIP-1559)
- Signatures (R, S, V)
- Transaction hash and size

### Needs Implementation ⚠️
- **From address recovery** - Currently returns zero address
  - Requires: secp256k1 ECDSA recovery from (v, r, s)
  - Library: Could use `k256` or `secp256k1` crate
  - Impact: Low for read-only analysis, High for wallet operations

### By Design 📋
- **Raw bytes** - Not in TxIR (available via `ChainEncoder::to_bytes()`)
  - This is intentional - TxIR is the canonical representation
  - Raw bytes are preserved for roundtrip verification

## 🚀 Recommendations

1. **For your use case**: If you need the from address, you can either:
   - Implement ECDSA recovery in the decoder
   - Provide it externally (e.g., from block data)
   - Parse it from WASM context if available

2. **Data completeness**: All transaction data IS preserved in TxIR, just in different locations:
   - Chain-specific details → `metadata.extra` (JSON)
   - Operations → `operations` array
   - Signatures → `authorization`
   - State changes → `state_deltas`

3. **What's missing from your output**:
   - Please specify which field you expected to see but didn't
   - I can help identify where it's stored or if it needs to be added

## 📝 Recent Fixes

- **2024-11-18**: Fixed Polygon/Arbitrum chain ID (was showing 1, now shows correct ID)
- **2024-11-18**: Added access_list to metadata.extra (was parsed but not preserved)

---

**Question**: Which specific field are you not seeing in the TxIR? I can help locate it or add it if it's missing.
