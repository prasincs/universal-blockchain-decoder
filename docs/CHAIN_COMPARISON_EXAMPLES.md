# Chain Comparison Examples: Same Operation, Different Encodings

This document provides **side-by-side comparisons** of the same logical operation (sending value) across different blockchains, showing how they encode differently but decode to the same TxIR.

---

## Example: Send $100 Worth of Native Currency

Let's compare sending approximately $100 worth of native currency across three major chains:
- **Bitcoin**: Send 0.002 BTC (~$100 at $50,000/BTC)
- **Ethereum**: Send 0.04 ETH (~$100 at $2,500/ETH)
- **Solana**: Send 1 SOL (~$100 at $100/SOL)

---

## Bitcoin: UTXO Model

### Raw Transaction (Hex)
```
0200000001a1b2c3d4e5f6071829...  [~250 bytes total]
```

### Decoded Bitcoin Transaction
```json
{
  "version": 2,
  "inputs": [
    {
      "prev_tx": "a1b2c3d4e5f60718293a4b5c6d7e8f90",
      "prev_index": 0,
      "script_sig": "483045022100...",
      "sequence": 4294967295,
      "value_satoshis": 300000
    }
  ],
  "outputs": [
    {
      "index": 0,
      "value_satoshis": 200000,
      "script_pubkey": "76a914...88ac",
      "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
    },
    {
      "index": 1,
      "value_satoshis": 99000,
      "script_pubkey": "76a914...88ac",
      "address": "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"
    }
  ],
  "locktime": 0,
  "fee_satoshis": 1000
}
```

### Canonicalized TxIR
```rust
TxIR {
    chain: ChainRef {
        id: 0,
        name: "Bitcoin",
        family: ChainFamily::Utxo,
        network: Some("mainnet"),
    },
    metadata: TxMetadata {
        tx_hash: hex!("3f7a...8e2d"),  // Double SHA-256
        block_height: Some(800000),
        timestamp: Some(1699564800),
        size: 226,
        extra: r#"{
            "version": 2,
            "locktime": 0,
            "weight": 904,
            "fee_rate": "4.42 sat/vB"
        }"#,
    },
    authorization: AuthorizationPackage {
        signatures: vec![
            Signature {
                data: hex!("3045022100..."),
                key_index: 0,
                metadata: Some("ECDSA signature (DER)"),
            }
        ],
        public_keys: vec![
            PublicKey {
                data: hex!("0279be..."),  // Compressed pubkey
                key_type: KeyType::Secp256k1,
            }
        ],
        signature_scheme: SignatureScheme::Ecdsa,
    },
    operations: vec![
        Operation::Transfer(Transfer {
            from: Address::Bitcoin("1Sender...".into()),
            to: Address::Bitcoin("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".into()),
            amount: Amount {
                value: 200000,
                decimals: 8,  // 0.002 BTC
            },
            asset: AssetId::Native,
        })
    ],
    state_deltas: StateDeltas {
        inputs: vec![
            InputReference {
                prev_tx: hex!("a1b2c3d4..."),
                output_index: 0,
                value: Amount { value: 300000, decimals: 8 },
                script: hex!("483045..."),
            }
        ],
        outputs: vec![
            OutputValue {
                index: 0,
                address: Address::Bitcoin("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".into()),
                value: Amount { value: 200000, decimals: 8 },
                script: hex!("76a914...88ac"),
            },
            OutputValue {
                index: 1,
                address: Address::Bitcoin("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2".into()),
                value: Amount { value: 99000, decimals: 8 },  // Change
                script: hex!("76a914...88ac"),
            }
        ],
        account_changes: vec![],  // Empty for UTXO
    },
    privacy: None,
}
```

**Key Characteristics:**
- ✅ Stateless: Inputs reference previous outputs
- ✅ No nonce: Replay protection via UTXO uniqueness
- ✅ Implicit fee: 300000 - 200000 - 99000 = 1000 sats
- ✅ Change output: 99000 sats back to sender
- ✅ Script-based: scriptPubKey defines spending conditions

---

## Ethereum: Account Model

### Raw Transaction (RLP-encoded Hex)
```
f86d8201a48504a817c8008252089...  [~110 bytes total]
```

### Decoded Ethereum Transaction
```json
{
  "chain_id": 1,
  "nonce": 420,
  "gas_price": "20000000000",
  "gas_limit": 21000,
  "to": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "value": "40000000000000000",
  "data": "",
  "v": 37,
  "r": "0x123456...",
  "s": "0x789abc...",
  "from": "0xSenderAddress..."
}
```

### Canonicalized TxIR
```rust
TxIR {
    chain: ChainRef {
        id: 1,
        name: "Ethereum",
        family: ChainFamily::Account,
        network: Some("mainnet"),
    },
    metadata: TxMetadata {
        tx_hash: hex!("5a9f...3c1e"),  // Keccak-256
        block_height: Some(18500000),
        timestamp: Some(1699564812),
        size: 109,
        extra: r#"{
            "nonce": 420,
            "gas_price": "20000000000",
            "gas_limit": 21000,
            "gas_used": 21000,
            "effective_gas_price": "20000000000",
            "cumulative_gas_used": 21000
        }"#,
    },
    authorization: AuthorizationPackage {
        signatures: vec![
            Signature {
                data: hex!("37..."),  // v, r, s combined
                key_index: 0,
                metadata: Some("ECDSA signature (Ethereum format)"),
            }
        ],
        public_keys: vec![
            PublicKey {
                data: hex!("04f3a8..."),  // Uncompressed pubkey (recovered)
                key_type: KeyType::Secp256k1,
            }
        ],
        signature_scheme: SignatureScheme::Ecdsa,
    },
    operations: vec![
        Operation::Transfer(Transfer {
            from: Address::Ethereum(hex!("SenderAddress...")),
            to: Address::Ethereum(hex!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb")),
            amount: Amount {
                value: 40000000000000000,
                decimals: 18,  // 0.04 ETH
            },
            asset: AssetId::Native,
        })
    ],
    state_deltas: StateDeltas {
        inputs: vec![],   // Empty for account model
        outputs: vec![],  // Empty for account model
        account_changes: vec![
            AccountChange {
                address: Address::Ethereum(hex!("SenderAddress...")),
                nonce: Some(420),  // 420 → 421
                balance_change: -40420000000000000,  // -0.04 ETH - 0.00042 ETH fee
                storage_changes: vec![],
            },
            AccountChange {
                address: Address::Ethereum(hex!("742d35Cc6634C0532925a3b844Bc9e7595f0bEb")),
                nonce: None,
                balance_change: 40000000000000000,  // +0.04 ETH
                storage_changes: vec![],
            },
            AccountChange {
                address: Address::Ethereum(hex!("MinerAddress...")),
                nonce: None,
                balance_change: 420000000000000,  // +0.00042 ETH (fee)
                storage_changes: vec![],
            }
        ],
    },
    privacy: None,
}
```

**Key Characteristics:**
- ✅ Stateful: Global account state (balances, nonces)
- ✅ Explicit nonce: Sequential counter prevents replay
- ✅ Explicit fee: gas_used × gas_price = 21000 × 20 gwei
- ✅ No change output: Direct account-to-account transfer
- ✅ Signature recovery: Public key recovered from (v, r, s)

---

## Solana: Instruction Model

### Raw Transaction (Borsh-encoded Hex)
```
01000103c8d842a4f87f75a9...  [~335 bytes total]
```

### Decoded Solana Transaction
```json
{
  "signatures": [
    "5Zf7xV3yN2mK8pW1..."
  ],
  "message": {
    "header": {
      "num_required_signatures": 1,
      "num_readonly_signed_accounts": 0,
      "num_readonly_unsigned_accounts": 1
    },
    "account_keys": [
      "FeePayer111111111111111111111111111111111",
      "Recipient1111111111111111111111111111111",
      "11111111111111111111111111111111"
    ],
    "recent_blockhash": "GH7ome3EiwEr7tu9JuTh2dpYWBJK3z69Xm1ZE3MEE6JC",
    "instructions": [
      {
        "program_id_index": 2,
        "accounts": [0, 1],
        "data": [2, 0, 0, 0, 0, 202, 154, 59, 0, 0, 0, 0]
      }
    ]
  }
}
```

### Canonicalized TxIR
```rust
TxIR {
    chain: ChainRef {
        id: 101,
        name: "Solana",
        family: ChainFamily::Instruction,
        network: Some("mainnet-beta"),
    },
    metadata: TxMetadata {
        tx_hash: hex!("7d2a...9f3b"),  // SHA-256 of signature
        block_height: Some(220000000),
        timestamp: Some(1699564820),
        size: 335,
        extra: r#"{
            "compute_units": 200000,
            "compute_units_consumed": 150,
            "fee": 5000,
            "recent_blockhash": "GH7ome3EiwEr7tu9JuTh2dpYWBJK3z69Xm1ZE3MEE6JC",
            "num_instructions": 1
        }"#,
    },
    authorization: AuthorizationPackage {
        signatures: vec![
            Signature {
                data: hex!("5Zf7xV3yN2mK8pW1..."),  // Ed25519 (64 bytes)
                key_index: 0,
                metadata: Some("Ed25519 signature"),
            }
        ],
        public_keys: vec![
            PublicKey {
                data: hex!("c8d842a4f87f75a9..."),  // Ed25519 pubkey (32 bytes)
                key_type: KeyType::Ed25519,
            }
        ],
        signature_scheme: SignatureScheme::EdDsa,
    },
    operations: vec![
        Operation::Generic(GenericOperation {
            op_type: "SYSTEM_TRANSFER".into(),
            data: hex!("02000000000000ca9a3b00000000"),  // Instruction data
            metadata: r#"{
                "program": "11111111111111111111111111111111",
                "program_name": "System",
                "instruction": "Transfer",
                "accounts": [
                    {"index": 0, "role": "from", "is_signer": true, "is_writable": true},
                    {"index": 1, "role": "to", "is_signer": false, "is_writable": true}
                ],
                "transfer_amount": "1000000000",
                "transfer_amount_human": "1 SOL"
            }"#,
        })
    ],
    state_deltas: StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes: vec![
            AccountChange {
                address: Address::Solana("FeePayer111111111111111111111111111111111".into()),
                nonce: None,  // Solana uses blockhash, not nonce
                balance_change: -1000005000,  // -1 SOL - 0.000005 SOL fee
                storage_changes: vec![],
            },
            AccountChange {
                address: Address::Solana("Recipient1111111111111111111111111111111".into()),
                nonce: None,
                balance_change: 1000000000,  // +1 SOL
                storage_changes: vec![],
            }
        ],
    },
    privacy: None,
}
```

**Key Characteristics:**
- ✅ Instruction-based: Operations defined as program instructions
- ✅ Blockhash replay protection: Transaction expires after ~2 minutes
- ✅ Parallel execution: Account locking enables parallelism
- ✅ Fixed fee: Per-signature (5000 lamports)
- ✅ Ed25519 signatures: Faster verification than ECDSA

---

## Side-by-Side Comparison

| Aspect | Bitcoin (UTXO) | Ethereum (Account) | Solana (Instruction) |
|--------|----------------|--------------------|-----------------------|
| **Transaction Size** | 226 bytes | 109 bytes | 335 bytes |
| **Signature Scheme** | ECDSA (secp256k1) | ECDSA (secp256k1) | EdDSA (ed25519) |
| **Signature Size** | 71-73 bytes (DER) | 65 bytes (v,r,s) | 64 bytes |
| **Hash Algorithm** | Double SHA-256 | Keccak-256 | SHA-256 |
| **Encoding** | Custom binary | RLP | Borsh |
| **Fee Model** | Implicit (inputs - outputs) | Gas (21000 × 20 gwei) | Per-signature (5000 lamports) |
| **Fee Amount** | 0.00001 BTC (~$0.50) | 0.00042 ETH (~$1.05) | 0.000005 SOL (~$0.0005) |
| **Replay Protection** | UTXO uniqueness | Nonce (420) | Recent blockhash |
| **State Model** | UTXO set | Account balances | Account data |
| **Parallelization** | ✅ Yes (stateless) | ❌ No (sequential nonces) | ✅ Yes (account locking) |
| **Change Output** | ✅ Yes (99000 sats) | ❌ No (direct transfer) | ❌ No (direct transfer) |
| **Public Key Storage** | In signature | Recovered from sig | In transaction |
| **Block Time** | ~10 minutes | ~12 seconds | ~400 milliseconds |
| **Finality** | ~60 minutes (6 blocks) | ~15 minutes (2 epochs) | ~6 seconds (32 slots) |

---

## Unified TxIR Comparison

Despite encoding differences, **all three map to similar TxIR structures**:

```rust
// Common fields across all three
TxIR {
    chain: ChainRef { ... },  // Different: Bitcoin vs Ethereum vs Solana
    metadata: TxMetadata {
        tx_hash: [...],       // Different hash algorithms
        size: ...,            // Different sizes
        extra: {...},         // Chain-specific metadata
    },
    authorization: AuthorizationPackage {
        signatures: [sig],    // Different schemes (ECDSA vs EdDSA)
        signature_scheme: ..., // ECDSA vs EdDSA
    },
    operations: [
        Transfer { ... }      // Same semantic operation: transfer value
    ],
    state_deltas: StateDeltas {
        // Bitcoin uses inputs/outputs
        // Ethereum/Solana use account_changes
    },
    privacy: None,           // All transparent
}
```

**Universal Properties (Preserved by TxIR):**
1. ✅ All have **authorization** (signatures)
2. ✅ All perform **value transfer** (Operation::Transfer)
3. ✅ All modify **state** (UTXO set or account balances)
4. ✅ All have **fees** (different mechanisms)
5. ✅ All have **replay protection** (different mechanisms)
6. ✅ All have **transaction IDs** (different hash algorithms)

---

## Advanced Example: Smart Contract Interaction

### Ethereum: Uniswap V2 Token Swap

**Raw Transaction:**
```
f8ab82...  [~200 bytes]
```

**Decoded:**
```json
{
  "to": "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
  "value": "0",
  "data": "0x38ed1739000000000000000000000000000000000000000000000000016345785d8a00000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000deadbeef000000000000000000000000000000000000000000000000000000000000000000000000000000000000000061c9c3b80000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000006b175474e89094c44da98b954eedeac495271d0f"
}
```

**TxIR:**
```rust
operations: vec![
    Operation::ContractCall(ContractCall {
        contract: Address::Ethereum(hex!("7a250d5630B4cF539739dF2C5dAcb4c659F2488D")),
        method: hex!("38ed1739"),  // swapExactTokensForTokens
        data: hex!("00000000..."),
        value: Some(Amount { value: 0, decimals: 18 }),
        resource_limits: ResourceLimits::Gas {
            max_gas: 300000,
            max_fee_per_gas: Some(50000000000),
        },
    })
],
state_deltas: StateDeltas {
    account_changes: vec![
        AccountChange {
            address: sender,
            balance_change: -420000000000000,  // Gas fee
            storage_changes: vec![],
        },
        // Uniswap router doesn't change ETH balance (token swap)
    ],
}
```

### Solana: Serum DEX Token Swap

**Raw Transaction:**
```
010001...  [~500 bytes]
```

**Decoded:**
```json
{
  "instructions": [
    {
      "program_id": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
      "accounts": [/* 10+ accounts */],
      "data": [9, /* swap instruction data */]
    }
  ]
}
```

**TxIR:**
```rust
operations: vec![
    Operation::Generic(GenericOperation {
        op_type: "SERUM_SWAP".into(),
        data: hex!("09..."),
        metadata: r#"{
            "program": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
            "program_name": "Serum DEX",
            "instruction": "Swap",
            "accounts": [
                {"role": "market", ...},
                {"role": "request_queue", ...},
                {"role": "event_queue", ...},
                ...
            ]
        }"#,
    })
],
state_deltas: StateDeltas {
    account_changes: vec![
        AccountChange {
            address: user_token_account_a,
            storage_changes: vec![/* Token balance decreased */],
        },
        AccountChange {
            address: user_token_account_b,
            storage_changes: vec![/* Token balance increased */],
        },
    ],
}
```

**Key Difference:**
- **Ethereum**: ABI-encoded function call (swapExactTokensForTokens)
- **Solana**: Instruction index (9) with account references
- **TxIR**: Both map to contract/program interactions with metadata

---

## Privacy Example: Zcash Shielded Transaction

### Transparent → Shielded (t→z)

**Raw Transaction:**
```
0400008085202f8901...  [~2KB due to zk-SNARK proof]
```

**Decoded:**
```json
{
  "version": 4,
  "transparent_inputs": [
    {
      "prev_tx": "abc123...",
      "prev_index": 0,
      "value": 1000000
    }
  ],
  "transparent_outputs": [],
  "shielded_outputs": [
    {
      "cv": "commitment_value",
      "cm": "commitment_note",
      "ephemeral_key": "epk",
      "enc_ciphertext": "encrypted_output",
      "out_ciphertext": "encrypted_viewing_key",
      "proof": "groth16_proof"
    }
  ],
  "binding_sig": "signature"
}
```

**TxIR:**
```rust
TxIR {
    chain: ChainRef { id: 133, name: "Zcash", family: ChainFamily::Privacy },
    operations: vec![
        Operation::Transfer(Transfer {
            from: Address::Zcash("t1abc...".into()),  // Transparent
            to: Address::Private("zs1xyz...".into()),  // Shielded
            amount: Amount::Confidential(hex!("commitment_value")),
            asset: AssetId::Native,
        })
    ],
    state_deltas: StateDeltas {
        inputs: vec![
            InputReference {
                prev_tx: hex!("abc123..."),
                output_index: 0,
                value: Amount { value: 1000000, decimals: 8 },
                script: hex!("..."),
            }
        ],
        outputs: vec![],  // Shielded outputs not visible in state_deltas
        account_changes: vec![],
    },
    privacy: Some(PrivacyMetadata {
        features: vec![
            PrivacyFeature::HiddenRecipient(PrivateAddress::Sapling {
                diversifier: hex!("..."),
                transmission_key: hex!("..."),
            }),
            PrivacyFeature::HiddenAmount(ConfidentialAmount {
                commitment: hex!("commitment_value"),
                range_proof: None,  // Implicit in Sapling proof
            }),
        ],
        observability: ObservabilityLevel::PartiallyObservable,  // Can see input, not output
        viewing_key: None,  // Receiver has viewing key
    }),
}
```

**Privacy Features:**
- ✅ Sender visible (transparent input)
- ❌ Recipient hidden (shielded output)
- ❌ Amount hidden (commitment only)
- ✅ Viewing key holder can decrypt output

---

## Takeaway: Same Intent, Different Encoding

All these examples perform **the same logical operation** (transfer value from A to B), but:
- **Encoding**: Completely different binary formats
- **State model**: UTXO vs Account vs Instruction
- **Signature scheme**: ECDSA vs EdDSA
- **Hash algorithm**: SHA-256 vs Keccak-256
- **Fee mechanism**: Implicit vs Gas vs Fixed

Yet **TxIR unifies them** into a common representation, enabling:
- ✅ Universal block explorers
- ✅ Cross-chain analytics
- ✅ Forensics and compliance tools
- ✅ Multi-chain indexers

---

**Next:** See these examples live in the [Interactive WASM Demo](../simulator/index.html)!
