# Visualizing the Universal Blockchain Decoder Type System

> **Blog Post**: How 2200+ Blockchains Share One Type System
>
> *Exploring the fascinating similarities and differences across Bitcoin, Ethereum, Solana, and beyond*

---

## Introduction

What if I told you that Bitcoin, Ethereum, Solana, and 2200+ other blockchains can be represented using **one unified type system**?

Sounds impossible, right? After all:
- Bitcoin uses **UTXOs** (Unspent Transaction Outputs)
- Ethereum uses **accounts** with nonces and gas
- Solana uses **instructions** with parallel execution
- Zcash adds **zero-knowledge privacy**

How can one type capture all this diversity?

**The secret:** Focus on *what transactions do*, not *how they're encoded*.

This is the story of **TxIR** (Transaction Intermediate Representation) - a type system that unifies blockchain transaction models while preserving their unique characteristics.

---

## The Core Insight: All Transactions Have the Same Job

Despite their differences, **all blockchain transactions share four fundamental responsibilities**:

```
┌─────────────────────────────────────────────────────────┐
│          What Every Transaction Must Answer             │
├─────────────────────────────────────────────────────────┤
│  1. WHO authorized this?     → Authorization            │
│  2. WHAT does it do?          → Operations              │
│  3. HOW does state change?    → State Deltas            │
│  4. WHERE/WHEN did it happen? → Metadata                │
└─────────────────────────────────────────────────────────┘
```

Let's see how TxIR captures these universals.

---

## The TxIR Type: A Universal Lens

Here's the complete TxIR type that represents **every blockchain transaction**:

```rust
pub struct TxIR<'a, const V: u8> {
    pub chain: ChainRef,                    // Which blockchain
    pub metadata: TxMetadata,               // When, where, how big
    pub authorization: AuthorizationPackage, // Signatures + keys
    pub operations: Vec<Operation>,          // What it does
    pub state_deltas: StateDeltas,          // State changes
    pub privacy: Option<PrivacyMetadata>,   // Privacy features (if any)
    _phantom: PhantomData<&'a [u8]>,
}
```

### Breaking It Down

#### 1. ChainRef - Where Are We?

```rust
pub struct ChainRef {
    pub id: u64,              // Ethereum = 1, Bitcoin = 0, Polygon = 137
    pub name: String,         // "Ethereum Mainnet"
    pub family: ChainFamily,  // UTXO / Account / Instruction / Privacy
    pub network: Option<String>, // "mainnet" / "testnet" / "devnet"
}
```

**Examples:**
- Bitcoin: `{ id: 0, name: "Bitcoin", family: UTXO, network: "mainnet" }`
- Ethereum: `{ id: 1, name: "Ethereum", family: Account, network: "mainnet" }`
- Solana: `{ id: 101, name: "Solana", family: Instruction, network: "mainnet-beta" }`

#### 2. Metadata - Context Information

```rust
pub struct TxMetadata {
    pub tx_hash: Vec<u8>,           // Transaction ID
    pub block_height: Option<u64>,  // Block number (if known)
    pub timestamp: Option<u64>,     // Unix timestamp
    pub size: usize,                // Bytes
    pub extra: String,              // Chain-specific JSON
}
```

**Chain-Specific Extras:**
- Bitcoin: `{ "version": 2, "locktime": 0, "weight": 548 }`
- Ethereum: `{ "nonce": 42, "gas_price": "20000000000", "gas_limit": 21000 }`
- Solana: `{ "compute_units": 200000, "fee_payer": "..." }`

#### 3. Authorization - Who Signed This?

```rust
pub struct AuthorizationPackage {
    pub signatures: Vec<Signature>,
    pub public_keys: Vec<PublicKey>,
    pub signature_scheme: SignatureScheme,
}

pub enum SignatureScheme {
    Ecdsa,    // Bitcoin, Ethereum (secp256k1)
    EdDsa,    // Solana, Algorand, Cardano (ed25519)
    Schnorr,  // Bitcoin Taproot (secp256k1)
    Custom(u32),
}
```

**Signature Diversity:**

| Chain | Scheme | Curve | Signature Size |
|-------|--------|-------|----------------|
| Bitcoin | ECDSA | secp256k1 | 71-73 bytes (DER) |
| Ethereum | ECDSA | secp256k1 | 65 bytes (v,r,s) |
| Solana | EdDSA | ed25519 | 64 bytes |
| Algorand | EdDSA | ed25519 | 64 bytes |
| Cardano | EdDSA | ed25519-extended | 64 bytes |

#### 4. Operations - What Does It Do?

```rust
pub enum Operation {
    Transfer(Transfer),           // Send value A → B
    ContractCall(ContractCall),   // Execute smart contract
    ContractDeploy(ContractDeploy), // Deploy new contract
    Stake(Stake),                 // Staking operations
    Generic(GenericOperation),    // Chain-specific ops
}
```

**Transfer** (most common):
```rust
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub amount: Amount { value: u128, decimals: u8 },
    pub asset: AssetId,  // Native / Token(address) / Custom(id)
}
```

**ContractCall** (Ethereum, EVM):
```rust
pub struct ContractCall {
    pub contract: Address,        // Contract address
    pub method: Vec<u8>,          // Function selector (4 bytes)
    pub data: Vec<u8>,            // ABI-encoded arguments
    pub value: Option<Amount>,    // ETH sent with call
    pub resource_limits: ResourceLimits, // Gas limit
}
```

**Generic** (Solana instructions, custom ops):
```rust
pub struct GenericOperation {
    pub op_type: String,          // "stake", "vote", "swap"
    pub data: Vec<u8>,            // Chain-specific data
    pub metadata: String,         // JSON context
}
```

#### 5. StateDeltas - How Does State Change?

This is where we unify **three different transaction models**:

```rust
pub struct StateDeltas {
    // UTXO Model (Bitcoin, Litecoin, Cardano)
    pub inputs: Vec<InputReference>,   // Consumed UTXOs
    pub outputs: Vec<OutputValue>,     // Created UTXOs

    // Account Model (Ethereum, Solana)
    pub account_changes: Vec<AccountChange>, // Balance/nonce changes
}
```

**UTXO Model (Bitcoin):**
```rust
// Transaction consumes inputs, creates outputs
inputs: [
    { prev_tx: "abc123...", output_index: 0, value: 0.5 BTC }
]
outputs: [
    { address: "1A1zP1eP...", value: 0.3 BTC },  // Recipient
    { address: "1BvBMSE...", value: 0.19 BTC },  // Change
]
// 0.01 BTC implicit fee = inputs - outputs
```

**Account Model (Ethereum):**
```rust
// Transaction modifies account state
account_changes: [
    { address: "0x742d35...", nonce: 42→43, balance: -0.1 ETH },
    { address: "0x8dAF17...", balance: +0.1 ETH }
]
```

**Hybrid (Solana - instruction-based but tracked as account changes):**
```rust
account_changes: [
    { address: "Fee payer", balance: -0.00005 SOL },
    { address: "Program account", storage_changes: [...] }
]
```

#### 6. Privacy - Optional Privacy Features

```rust
pub struct PrivacyMetadata {
    pub features: Vec<PrivacyFeature>,
    pub observability: ObservabilityLevel,
    pub viewing_key: Option<ViewingKey>,
}

pub enum PrivacyFeature {
    HiddenSender(PrivateAddress),      // Ring signatures
    HiddenRecipient(PrivateAddress),   // Stealth addresses
    HiddenAmount(ConfidentialAmount),  // Pedersen commitments
    HiddenGraph(PrivacyPool),          // Mixers, pools
    HiddenExistence(EncryptedTransaction), // Encrypted mempools
}

pub enum ObservabilityLevel {
    FullyObservable,      // Bitcoin, Ethereum
    PartiallyObservable,  // Zcash transparent → shielded
    FullyPrivate,         // Zcash shielded → shielded
}
```

**Example (Zcash shielded transaction):**
```rust
privacy: Some(PrivacyMetadata {
    features: vec![
        HiddenSender(sapling_address),
        HiddenRecipient(sapling_address),
        HiddenAmount(commitment),
    ],
    observability: FullyPrivate,
    viewing_key: Some(viewing_key),
})
```

---

## The Four Chain Families

Blockchains cluster into **four distinct families** based on their transaction model:

```
                        TxIR (Universal)
                              |
        ┌─────────────────────┼─────────────────────┬───────────┐
        │                     │                     │           │
        ▼                     ▼                     ▼           ▼
┌───────────────┐   ┌─────────────────┐   ┌──────────────┐  ┌────────┐
│ UTXO Family   │   │ Account Family  │   │ Instruction  │  │Privacy │
│               │   │                 │   │   Family     │  │Family  │
├───────────────┤   ├─────────────────┤   ├──────────────┤  ├────────┤
│ • Bitcoin     │   │ • Ethereum      │   │ • Solana     │  │• Zcash │
│ • Litecoin    │   │ • Polygon       │   │              │  │• Monero│
│ • Dogecoin    │   │ • BNB Chain     │   │              │  └────────┘
│ • Cardano     │   │ • Avalanche C   │   │              │
│ • Dash        │   │ • Arbitrum      │   │              │
│ • BCH, BSV    │   │ • Optimism      │   │              │
│ • Avalanche X │   │ • 2000+ EVM     │   │              │
│               │   │ • Aptos, Sui    │   │              │
│               │   │ • NEAR, Stellar │   │              │
│               │   │ • Cosmos, Algo  │   │              │
└───────────────┘   └─────────────────┘   └──────────────┘
```

### Family 1: UTXO (Unspent Transaction Output)

**Philosophy:** "Coins are like physical cash - you consume old bills and create new ones"

**Characteristics:**
- ✅ Parallel validation (stateless)
- ✅ Natural privacy (pseudonymous addresses)
- ❌ Larger transactions (must reference all inputs)
- ❌ No smart contract state (except script-based like Cardano)

**TxIR Mapping:**
```rust
// Bitcoin transaction
TxIR {
    chain: Bitcoin,
    operations: vec![
        Transfer { from: inputs, to: outputs[0] },
    ],
    state_deltas: StateDeltas {
        inputs: vec![
            InputReference { prev_tx, output_index, value, script_sig }
        ],
        outputs: vec![
            OutputValue { address, value, script_pubkey }
        ],
        account_changes: vec![], // Empty for UTXO
    },
    ...
}
```

**Chains:** Bitcoin (0), Litecoin (2), Dogecoin (3), Dash (5), Bitcoin Cash (145), Bitcoin SV (236), Cardano (1010), Zcash transparent (133), Avalanche X-Chain (43114-X)

### Family 2: Account (Account-Based Model)

**Philosophy:** "Accounts are like bank accounts - track balances and modify them"

**Characteristics:**
- ✅ Simple transaction structure (from, to, amount)
- ✅ Smart contract state storage
- ✅ Sequential ordering (nonces prevent replay)
- ❌ Sequential validation (must know account state)

**TxIR Mapping:**
```rust
// Ethereum transaction
TxIR {
    chain: Ethereum,
    operations: vec![
        ContractCall {
            contract: "0xUniswapV2Router",
            method: "swapExactTokensForTokens",
            data: abi_encoded_args,
            value: Some(0.1 ETH),
            resource_limits: Gas(21000),
        }
    ],
    state_deltas: StateDeltas {
        inputs: vec![],       // Empty for account model
        outputs: vec![],      // Empty for account model
        account_changes: vec![
            AccountChange { address: from, nonce: 42→43, balance: -0.1 },
            AccountChange { address: to, balance: +0.1 },
        ],
    },
    ...
}
```

**Chains:**
- **EVM (2000+):** Ethereum (1), Polygon (137), BNB Chain (56), Avalanche C-Chain (43114), Arbitrum (42161), Optimism (10), Base (8453), and 2000+ more
- **Non-EVM:** Aptos (1001), Sui (1002), NEAR (1003), Stellar (1004), XRP (1005), Algorand (1006), Tron (1007), Cosmos (118), Polkadot (1009), Avalanche P-Chain (43114-P)

### Family 3: Instruction (Program-Based)

**Philosophy:** "Transactions are bundles of instructions for programs"

**Characteristics:**
- ✅ Multiple operations per transaction
- ✅ Parallel execution (account locking model)
- ✅ Deterministic resource metering (compute units)
- ❌ Complex transaction structure

**TxIR Mapping:**
```rust
// Solana transaction
TxIR {
    chain: Solana,
    operations: vec![
        Generic {
            op_type: "SPL_TOKEN_TRANSFER",
            data: instruction_data,
            metadata: { "program": "TokenkegQfeZ...", "accounts": [...] }
        },
        Generic {
            op_type: "STAKE_DELEGATE",
            data: instruction_data,
            metadata: { "program": "Stake11111...", "accounts": [...] }
        },
    ],
    state_deltas: StateDeltas {
        account_changes: vec![
            AccountChange { address: fee_payer, balance: -0.00005 },
            AccountChange { address: token_account, storage_changes: [...] },
        ],
        ...
    },
    ...
}
```

**Chains:** Solana (101)

### Family 4: Privacy (Shielded Transactions)

**Philosophy:** "Privacy is a fundamental right, not an afterthought"

**Characteristics:**
- ✅ Hidden sender, recipient, or amount
- ✅ Selective disclosure (viewing keys)
- ✅ Compliance-friendly (auditable with keys)
- ❌ Larger proof sizes (zk-SNARKs)
- ❌ Higher computational cost

**TxIR Mapping:**
```rust
// Zcash shielded transaction
TxIR {
    chain: Zcash,
    operations: vec![
        Transfer {
            from: Address::Private("zs1..."),
            to: Address::Private("zs1..."),
            amount: Amount::Confidential(commitment),
            asset: Native,
        }
    ],
    privacy: Some(PrivacyMetadata {
        features: vec![
            HiddenSender(sapling_spend),
            HiddenRecipient(sapling_output),
            HiddenAmount(value_commitment),
        ],
        observability: FullyPrivate,
        viewing_key: Some(incoming_viewing_key),
    }),
    ...
}
```

**Chains:** Zcash (133), Monero (TBD)

---

## How Decoding Works: From Bytes to TxIR

The Universal Blockchain Decoder uses a **two-phase transformation**:

```
┌─────────────────────────────────────────────────────────────┐
│                Phase 1: Decode (Chain-Specific)             │
└─────────────────────────────────────────────────────────────┘

Raw Bytes (Bitcoin)                Raw Bytes (Ethereum)
   [0x01, 0x00, ...]                  [0xf8, 0x6c, ...]
        │                                    │
        ▼                                    ▼
  ChainDecoder::decode()              ChainDecoder::decode()
        │                                    │
        ▼                                    ▼
  BitcoinTransaction                   EthereumTransaction
  {                                    {
    version: 2,                          nonce: 42,
    inputs: [...],                       gas_price: 20 gwei,
    outputs: [...],                      to: 0x742d35...,
    locktime: 0,                         value: 0.1 ETH,
  }                                      data: 0x...,
                                       }

┌─────────────────────────────────────────────────────────────┐
│            Phase 2: Canonicalize (Universal)                │
└─────────────────────────────────────────────────────────────┘

  BitcoinTransaction               EthereumTransaction
        │                                    │
        ▼                                    ▼
  Canonicalizer::canonicalize()      Canonicalizer::canonicalize()
        │                                    │
        └────────────────┬───────────────────┘
                         │
                         ▼
                    TxIR (Universal)
                    {
                      chain: ...,
                      metadata: ...,
                      authorization: ...,
                      operations: ...,
                      state_deltas: ...,
                      privacy: None,
                    }
```

### Example 1: Bitcoin Transaction → TxIR

**Input:** Raw Bitcoin transaction bytes
```
0100000001abc123...def789
```

**Phase 1: Decode**
```rust
let bitcoin_tx = BitcoinDecoder::decode(raw_bytes)?;
// BitcoinTransaction {
//   version: 2,
//   inputs: [
//     TxInput {
//       prev_hash: abc123...,
//       prev_index: 0,
//       script_sig: [0x48, 0x30, ...], // ECDSA signature
//       sequence: 0xffffffff,
//     }
//   ],
//   outputs: [
//     TxOutput { value: 50000000, script_pubkey: OP_DUP OP_HASH160 ... },
//     TxOutput { value: 49950000, script_pubkey: OP_DUP OP_HASH160 ... },
//   ],
//   locktime: 0,
// }
```

**Phase 2: Canonicalize**
```rust
let tx_ir = bitcoin_tx.canonicalize()?;
// TxIR {
//   chain: ChainRef { id: 0, name: "Bitcoin", family: UTXO },
//   metadata: TxMetadata {
//     tx_hash: "def789...",
//     size: 225,
//     extra: '{"version":2,"locktime":0}',
//   },
//   authorization: AuthorizationPackage {
//     signatures: [Signature { data: [0x48, 0x30, ...] }],
//     signature_scheme: Ecdsa,
//   },
//   operations: [
//     Transfer {
//       from: inputs_combined,
//       to: "1A1zP1eP...",
//       amount: Amount { value: 50000000, decimals: 8 }, // 0.5 BTC
//     }
//   ],
//   state_deltas: StateDeltas {
//     inputs: [InputReference { prev_tx: "abc123...", output_index: 0 }],
//     outputs: [
//       OutputValue { address: "1A1zP1eP...", value: 50000000 },
//       OutputValue { address: "1BvBMSE...", value: 49950000 },
//     ],
//     account_changes: [],
//   },
//   privacy: None,
// }
```

### Example 2: Ethereum Transaction → TxIR

**Input:** RLP-encoded Ethereum transaction
```
f86c2a8504a817c800825208947...
```

**Phase 1: Decode**
```rust
let eth_tx = EthereumDecoder::decode(raw_bytes)?;
// EthereumTransaction {
//   chain_id: 1,
//   nonce: 42,
//   gas_price: 20000000000, // 20 gwei
//   gas_limit: 21000,
//   to: Some(0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb),
//   value: 100000000000000000, // 0.1 ETH
//   data: vec![],
//   v: 37, r: 0x123..., s: 0x456..., // ECDSA signature
// }
```

**Phase 2: Canonicalize**
```rust
let tx_ir = eth_tx.canonicalize()?;
// TxIR {
//   chain: ChainRef { id: 1, name: "Ethereum", family: Account },
//   metadata: TxMetadata {
//     tx_hash: keccak256(rlp_bytes),
//     size: 108,
//     extra: '{"nonce":42,"gas_price":"20000000000","gas_limit":21000}',
//   },
//   authorization: AuthorizationPackage {
//     signatures: [Signature { data: ecdsa_recover(v,r,s) }],
//     public_keys: [PublicKey { data: recovered_pubkey }],
//     signature_scheme: Ecdsa,
//   },
//   operations: [
//     Transfer {
//       from: ecdsa_recover_address(v,r,s),
//       to: "0x742d35...",
//       amount: Amount { value: 100000000000000000, decimals: 18 },
//       asset: Native,
//     }
//   ],
//   state_deltas: StateDeltas {
//     inputs: [],
//     outputs: [],
//     account_changes: [
//       AccountChange { address: from, nonce: 42→43, balance_change: -0.1 },
//       AccountChange { address: to, balance_change: +0.1 },
//     ],
//   },
//   privacy: None,
// }
```

### Example 3: Solana Transaction → TxIR

**Input:** Borsh-encoded Solana transaction
```
01000103c8d842...
```

**Phase 1: Decode**
```rust
let solana_tx = SolanaDecoder::decode(raw_bytes)?;
// SolanaTransaction {
//   signatures: [
//     [0xab, 0xcd, ...], // Ed25519 signature (64 bytes)
//   ],
//   message: Message {
//     header: MessageHeader { num_required_signatures: 1, ... },
//     account_keys: [
//       Pubkey("FeePayerPublicKey..."),
//       Pubkey("TokenAccountPublicKey..."),
//       Pubkey("TokenProgramId..."),
//     ],
//     recent_blockhash: Hash("BlockhashValue..."),
//     instructions: [
//       CompiledInstruction {
//         program_id_index: 2, // TokenProgram
//         accounts: [0, 1], // Fee payer, token account
//         data: [3, 0x10, 0x27, 0x00, 0x00, ...], // Transfer 10000 tokens
//       }
//     ],
//   },
// }
```

**Phase 2: Canonicalize**
```rust
let tx_ir = solana_tx.canonicalize()?;
// TxIR {
//   chain: ChainRef { id: 101, name: "Solana", family: Instruction },
//   metadata: TxMetadata {
//     tx_hash: sha256(signature),
//     size: 334,
//     extra: '{"compute_units":200000,"fee":0.00005}',
//   },
//   authorization: AuthorizationPackage {
//     signatures: [Signature { data: [0xab, 0xcd, ...] }],
//     public_keys: [PublicKey { data: fee_payer_pubkey }],
//     signature_scheme: EdDsa,
//   },
//   operations: [
//     Generic {
//       op_type: "SPL_TOKEN_TRANSFER",
//       data: [3, 0x10, 0x27, ...],
//       metadata: '{
//         "program": "TokenkegQfeZy...",
//         "accounts": ["FeePayer...", "TokenAccount..."],
//         "instruction_index": 0
//       }',
//     }
//   ],
//   state_deltas: StateDeltas {
//     account_changes: [
//       AccountChange { address: fee_payer, balance_change: -0.00005 },
//       AccountChange { address: token_account, storage_changes: [...] },
//     ],
//     ...
//   },
//   privacy: None,
// }
```

---

## What Makes Chains Similar

Despite their diversity, blockchains share **remarkable similarities** when viewed through the TxIR lens:

### Similarity 1: Authorization (Everyone Uses Digital Signatures)

**All chains use public-key cryptography:**

| Chain | Signature | Curve | Hash | Verification |
|-------|-----------|-------|------|--------------|
| Bitcoin | ECDSA | secp256k1 | Double SHA-256 | `verify(pubkey, sig, tx_hash)` |
| Ethereum | ECDSA | secp256k1 | Keccak-256 | `ecrecover(hash, v, r, s)` |
| Solana | EdDSA | ed25519 | SHA-256 | `ed25519::verify(pubkey, sig, msg)` |
| Algorand | EdDSA | ed25519 | SHA-512/256 | `ed25519::verify(...)` |
| Cardano | EdDSA | ed25519-ext | Blake2b-256 | `ed25519::verify(...)` |

**TxIR unification:**
```rust
// All map to this:
AuthorizationPackage {
    signatures: vec![Signature { data: signature_bytes }],
    public_keys: vec![PublicKey { data: pubkey_bytes, key_type: Secp256k1/Ed25519 }],
    signature_scheme: Ecdsa/EdDsa/Schnorr,
}
```

### Similarity 2: Value Transfer (The #1 Use Case)

**90%+ of transactions are simple value transfers:**

```rust
// Bitcoin: Send 0.5 BTC
Transfer {
    from: inputs,
    to: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    amount: Amount { value: 50000000, decimals: 8 },
    asset: Native,
}

// Ethereum: Send 0.1 ETH
Transfer {
    from: "0xSender",
    to: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    amount: Amount { value: 100000000000000000, decimals: 18 },
    asset: Native,
}

// Solana: Send 1 SOL (as a Generic operation, but semantically a transfer)
Transfer {
    from: "SenderPubkey...",
    to: "RecipientPubkey...",
    amount: Amount { value: 1000000000, decimals: 9 },
    asset: Native,
}
```

### Similarity 3: Fee Mechanisms (Every Transaction Pays)

**All chains require fees to prevent spam:**

| Chain | Fee Model | Fee Field | Typical Fee |
|-------|-----------|-----------|-------------|
| Bitcoin | Fee = inputs - outputs | Implicit | ~0.00001 BTC ($0.43) |
| Ethereum | `gas_used * gas_price` | Explicit (`gas_limit`, `gas_price`) | 21000 gas × 20 gwei = 0.00042 ETH ($1.05) |
| Solana | Per-signature + compute units | Explicit (`signatures`, `compute_budget`) | 0.000005 SOL ($0.0001) |
| Polygon | Same as Ethereum | Explicit (`gas_limit`, `gas_price`) | 21000 gas × 30 gwei = 0.00063 MATIC ($0.0006) |

**TxIR representation:**
```rust
// Stored in metadata.extra as JSON
metadata.extra: '{
  "fee": "0.00001",
  "fee_asset": "BTC",
  "fee_mechanism": "implicit" | "gas" | "compute_units"
}'
```

### Similarity 4: Replay Protection (Prevent Double-Spending)

**All chains prevent transaction replay:**

| Chain | Mechanism | TxIR Field |
|-------|-----------|------------|
| Bitcoin | UTXO uniqueness (each input spent once) | `state_deltas.inputs` (prev_tx + index unique) |
| Ethereum | Nonce (sequential counter per account) | `account_changes[0].nonce` |
| Solana | Recent blockhash (expires after ~2 min) | `metadata.extra["blockhash"]` |

### Similarity 5: Hashing for Identity

**Every transaction has a unique ID:**

| Chain | Hash Algorithm | Input | Output Format |
|-------|----------------|-------|---------------|
| Bitcoin | Double SHA-256 | Serialized tx (no witness) | 32 bytes (hex reversed) |
| Ethereum | Keccak-256 | RLP-encoded tx | 32 bytes (hex) |
| Solana | SHA-256 | First signature | 32 bytes (base58) |
| Cosmos | SHA-256 | Protobuf-encoded tx | 32 bytes (hex uppercase) |

**TxIR:**
```rust
metadata.tx_hash: Vec<u8>  // Chain-specific hash bytes
```

---

## What Makes Chains Different

While the **goals** are similar, the **mechanisms** vary wildly:

### Difference 1: Transaction Model (UTXO vs Account vs Instruction)

**Bitcoin (UTXO):**
```
Transaction:
  Inputs: [
    { prev_tx: "tx1", index: 0, value: 1.0 BTC } ← Consumed
  ]
  Outputs: [
    { address: "Alice", value: 0.7 BTC } ← Created
    { address: "Bob (change)", value: 0.29 BTC } ← Created
  ]
  Fee: 0.01 BTC (implicit: 1.0 - 0.7 - 0.29)

State Change: UTXO set modified (1 removed, 2 added)
```

**Ethereum (Account):**
```
Transaction:
  From: 0xAlice
  To: 0xBob
  Value: 0.1 ETH
  Nonce: 5
  Gas: 21000 × 20 gwei = 0.00042 ETH

State Change:
  Alice: { balance: 10 ETH → 9.89958 ETH, nonce: 5 → 6 }
  Bob: { balance: 5 ETH → 5.1 ETH }
  Miner: { balance: +0.00042 ETH (fee) }
```

**Solana (Instruction):**
```
Transaction:
  Instructions: [
    { program: "System", instruction: "Transfer", data: [1 SOL] }
    { program: "Token", instruction: "Approve", data: [...] }
  ]
  Accounts: [Alice (signer, writable), Bob (writable), ...]

State Change:
  Alice (SOL account): -1 SOL
  Bob (SOL account): +1 SOL
  Alice (Token account): storage updated (approval)
```

**TxIR captures all three:**
```rust
// UTXO: Use inputs/outputs
state_deltas: StateDeltas {
    inputs: [InputReference { ... }],
    outputs: [OutputValue { ... }],
    account_changes: [],
}

// Account: Use account_changes
state_deltas: StateDeltas {
    inputs: [],
    outputs: [],
    account_changes: [AccountChange { address, nonce, balance_change }],
}

// Instruction: Use account_changes + Generic operations
operations: [Generic { op_type: "Transfer", ... }, Generic { op_type: "Approve", ... }]
state_deltas: StateDeltas { account_changes: [...] }
```

### Difference 2: Smart Contract Capabilities

**Bitcoin: Script-based (limited)**
```
scriptPubKey: OP_DUP OP_HASH160 <pubkey_hash> OP_EQUALVERIFY OP_CHECKSIG
```
- Stack-based language
- No loops (Turing-incomplete)
- ~200 opcodes
- Validation only (no state)

**Ethereum: EVM (Turing-complete)**
```solidity
contract ERC20 {
    mapping(address => uint256) public balances;
    function transfer(address to, uint256 amount) public { ... }
}
```
- Turing-complete (gas limits prevent infinite loops)
- State storage (key-value)
- ~140 opcodes
- Gas metering

**Solana: SVM (parallel, deterministic)**
```rust
#[program]
pub mod token {
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> { ... }
}
```
- Rust-based (compiled to BPF)
- Account model (programs read/write account data)
- Compute units (CU) metering
- Parallel execution (account locking)

**TxIR representation:**
```rust
// Bitcoin: No ContractCall (just Transfer with script metadata)
operations: [Transfer { ..., metadata: '{"script_type":"P2PKH"}' }]

// Ethereum: ContractCall with ABI data
operations: [ContractCall {
    contract: "0xUniswapV2Router",
    method: [0xa9, 0x05, 0x9c, 0xbb], // swapExactTokensForTokens selector
    data: abi_encoded_params,
    resource_limits: Gas(21000),
}]

// Solana: Generic operations per instruction
operations: [Generic {
    op_type: "SPL_TOKEN_TRANSFER",
    data: borsh_encoded_args,
    metadata: '{"program":"TokenkegQfeZ...","accounts":[...]}'
}]
```

### Difference 3: Encoding Formats

**Bitcoin: Custom binary + VarInt**
```
Version (4 bytes, LE) | TxIn count (VarInt) | TxIns | TxOut count | TxOuts | Locktime
  0x02000000          |       0x01          | [...]  |    0x02    | [...]  | 0x00000000
```

**Ethereum: RLP (Recursive Length Prefix)**
```
RLP([nonce, gasPrice, gasLimit, to, value, data, v, r, s])
→ 0xf86c2a8504a817c800825208...
```

**Solana: Borsh (Binary Object Representation Serializer for Hashing)**
```
Compact-u16(num_signatures) | Signatures | Message
  0x0100                    | [64 bytes] | [...]
```

**Cosmos: Protobuf**
```protobuf
message Tx {
  repeated bytes signatures = 1;
  TxBody body = 2;
  AuthInfo auth_info = 3;
}
```

**Avalanche: Custom Codec**
```
CodecID (2 bytes) | TypeID (4 bytes) | NetworkID | BlockchainID | ...
  0x0000          |   0x00000000     |   0x01    | [32 bytes]   | ...
```

**TxIR normalization:**
All these formats decode into the same TxIR structure, then re-encode deterministically using **Borsh** for canonical hashing.

### Difference 4: Privacy Models

**Bitcoin: Pseudonymous (Transparent)**
- All transactions public
- Addresses pseudonymous (not linked to identity)
- Chain analysis can de-anonymize (clustering heuristics)

**Ethereum: Pseudonymous (Transparent)**
- Same as Bitcoin
- Some privacy tools (Tornado Cash - deprecated, Aztec - zk-rollup)

**Zcash: Optional Privacy (Shielded Pools)**
- Transparent addresses: Like Bitcoin
- Shielded addresses (Sapling, Orchard): zk-SNARKs hide sender, recipient, amount
- 4 transaction types: `t→t`, `t→z`, `z→t`, `z→z`

**Monero: Mandatory Privacy**
- Ring signatures: Hide sender among decoys
- Stealth addresses: Hide recipient
- RingCT: Hide amount

**TxIR privacy field:**
```rust
// Bitcoin, Ethereum
privacy: None

// Zcash shielded
privacy: Some(PrivacyMetadata {
    features: vec![HiddenSender, HiddenRecipient, HiddenAmount],
    observability: FullyPrivate,
    viewing_key: Some(ViewingKey { ... }),
})

// Zcash t→z (transparent to shielded)
privacy: Some(PrivacyMetadata {
    features: vec![HiddenRecipient, HiddenAmount],
    observability: PartiallyObservable,
    ...
})
```

### Difference 5: Consensus Mechanisms (Not in TxIR, but Important Context)

| Chain | Consensus | Block Time | Finality |
|-------|-----------|------------|----------|
| Bitcoin | PoW (SHA-256) | ~10 min | ~1 hour (6 blocks) |
| Ethereum | PoS (Gasper) | ~12 sec | ~15 min (2 epochs) |
| Solana | PoS + PoH | ~400 ms | ~6 sec (32 slots) |
| Algorand | Pure PoS | ~4.5 sec | Instant (1 block) |
| Cosmos | Tendermint BFT | ~6 sec | Instant (1 block) |

**Note:** TxIR doesn't capture consensus, only transaction structure. Block time affects `metadata.timestamp` and `metadata.block_height`.

---

## Practical Example: Universal Transaction Explorer

Imagine building a **multi-chain block explorer** that shows transactions from any blockchain in a unified format.

**Without TxIR (traditional approach):**
```rust
match chain {
    Chain::Bitcoin => {
        let btc_tx = bitcoin::decode(raw_bytes)?;
        html! {
            <div>
                <h2>Bitcoin Transaction</h2>
                <p>Version: {btc_tx.version}</p>
                <p>Inputs: {btc_tx.inputs.len()}</p>
                // ... 50 lines of Bitcoin-specific HTML ...
            </div>
        }
    }
    Chain::Ethereum => {
        let eth_tx = ethereum::decode(raw_bytes)?;
        html! {
            <div>
                <h2>Ethereum Transaction</h2>
                <p>Nonce: {eth_tx.nonce}</p>
                <p>Gas: {eth_tx.gas_limit}</p>
                // ... 50 lines of Ethereum-specific HTML ...
            </div>
        }
    }
    // ... 2200 more cases ...
}
```

**With TxIR (unified approach):**
```rust
// Decode ANY chain to TxIR
let tx_ir = universal_decoder::decode(chain, raw_bytes)?;

// Universal rendering
html! {
    <div>
        <h2>{tx_ir.chain.name} Transaction</h2>
        <p>Hash: {hex::encode(&tx_ir.metadata.tx_hash)}</p>
        <p>Size: {tx_ir.metadata.size} bytes</p>

        <h3>Authorization</h3>
        <p>Signatures: {tx_ir.authorization.signatures.len()}</p>
        <p>Scheme: {tx_ir.authorization.signature_scheme}</p>

        <h3>Operations</h3>
        for op in tx_ir.operations {
            {render_operation(op)}
        }

        <h3>State Changes</h3>
        if !tx_ir.state_deltas.inputs.is_empty() {
            <p>UTXO Inputs: {tx_ir.state_deltas.inputs.len()}</p>
        }
        if !tx_ir.state_deltas.account_changes.is_empty() {
            <p>Accounts Modified: {tx_ir.state_deltas.account_changes.len()}</p>
        }
    </div>
}
```

**Result:** **One template** renders 2200+ chains! ✨

---

## Interactive Companion: WASM Demo

The blog post includes an **interactive WASM-based demo** where you can:

1. **Paste raw transaction bytes** (hex) from any supported chain
2. **Select the chain** (Bitcoin, Ethereum, Solana, etc.)
3. **See the decoded TxIR** in real-time
4. **Compare transactions** side-by-side from different chains
5. **Explore chain families** with visual grouping

**Demo Features:**
- ✅ Runs entirely in the browser (no backend)
- ✅ Supports 2200+ chains
- ✅ Real transaction examples from mainnet
- ✅ Side-by-side comparison view
- ✅ Interactive type explorer
- ✅ Privacy feature visualization

**Try it:** [universal-decoder-demo.vercel.app](https://universal-decoder-demo.vercel.app) (Coming soon!)

---

## Key Takeaways

1. **Universality Through Abstraction**: Focus on *what* transactions do (authorization, operations, state changes) rather than *how* they're encoded.

2. **Trait-Based Extensibility**: The type system uses Rust traits, allowing unlimited blockchain support without modifying the core TxIR type.

3. **Four Chain Families**: UTXO, Account, Instruction, and Privacy models capture the spectrum of blockchain architectures.

4. **Preserve Uniqueness**: TxIR unifies *semantics* while preserving *chain-specific details* in metadata fields.

5. **2200+ Chains, One Type**: Through generic decoders (e.g., EVM family), the type system scales to thousands of chains with minimal code.

6. **Privacy as a First-Class Citizen**: Optional `PrivacyMetadata` field supports both transparent and shielded transactions.

7. **Practical Applications**: Block explorers, indexers, forensics tools, compliance systems - any multi-chain application benefits from a unified type system.

---

## What's Next?

**Explore the Interactive Demo:** Try decoding real transactions from Bitcoin, Ethereum, Solana, and more at [universal-decoder-demo.vercel.app](https://universal-decoder-demo.vercel.app)

**Read the Technical Deep Dive:** [Architecture Documentation](https://github.com/prasincs/universal-blockchain-decoder/tree/main/docs)

**Contribute a New Chain:** Follow the [Decoder Implementation Guide](https://github.com/prasincs/universal-blockchain-decoder/blob/main/docs/DECODER_IMPLEMENTATION_GUIDE.md)

**Ask Questions:** Join the discussion on [GitHub Discussions](https://github.com/prasincs/universal-blockchain-decoder/discussions)

---

## Appendix: Complete Chain Support Matrix

### UTXO Chains (9)

| Chain | Chain ID | Decoder Crate | Features |
|-------|----------|---------------|----------|
| Bitcoin | 0 | decoder-bitcoin | SegWit, Taproot, P2PKH, P2SH, P2WPKH |
| Litecoin | 2 | decoder-litecoin | Similar to Bitcoin, 2.5min blocks |
| Dogecoin | 3 | decoder-dogecoin | Inflationary supply |
| Dash | 5 | decoder-dash | PrivateSend, InstantSend |
| Bitcoin Cash | 145 | decoder-bitcoin-cash | Larger blocks (32MB) |
| Bitcoin SV | 236 | decoder-bitcoin-sv | Even larger blocks (4GB) |
| Cardano | 1010 | decoder-cardano | eUTXO (extended UTXO with Plutus scripts) |
| Zcash (transparent) | 133 | decoder-zcash | UTXO + shielded pools |
| Avalanche X-Chain | 43114-X | decoder-avalanche | AVM (Avalanche Virtual Machine) |

### Account Chains - EVM (2000+)

| Chain | Chain ID | Special Features |
|-------|----------|------------------|
| Ethereum | 1 | Original EVM, PoS (Gasper) |
| Polygon | 137 | Ethereum sidechain, ~2sec blocks |
| BNB Chain | 56 | Binance ecosystem |
| Avalanche C-Chain | 43114 | EVM on Avalanche (subnet) |
| Arbitrum One | 42161 | Optimistic rollup, custom tx types |
| Optimism | 10 | Optimistic rollup, OP Stack |
| Base | 8453 | Coinbase L2, OP Stack |
| zkSync Era | 324 | ZK rollup, account abstraction |
| Linea | 59144 | Consensys ZK rollup |
| Scroll | 534352 | ZK rollup |
| Mantle | 5000 | Modular L2 |
| ... | ... | 2000+ more via `ethereum-lists/chains` |

**All supported via `decoder-evm` generic decoder** with chain data embedded at compile time.

### Account Chains - Non-EVM (11)

| Chain | Chain ID | VM | Features |
|-------|----------|----|----|
| Aptos | 1001 | Move VM | Parallel execution, formal verification |
| Sui | 1002 | Move VM | Object-centric model |
| NEAR | 1003 | WASM | Sharding, async execution |
| Stellar | 1004 | Custom | Payment-focused, anchors |
| XRP Ledger | 1005 | Custom | Payment channels, escrow |
| Algorand | 1006 | AVM (Algorand) | Pure PoS, instant finality |
| Tron | 1007 | TVM (Tron) | EVM-like, high TPS |
| Cosmos Hub | 118 | Cosmos SDK | IBC protocol, Tendermint |
| Osmosis | 1008 | Cosmos SDK | AMM DEX, IBC |
| Polkadot | 1009 | Substrate | Relay chain, parachains |
| Avalanche P-Chain | 43114-P | Platform VM | Staking, validators |

### Instruction Chains (1)

| Chain | Chain ID | VM | Features |
|-------|----------|----|----------|
| Solana | 101 | SVM (BPF) | Parallel execution, PoH, ~400ms blocks |

### Privacy Chains (2)

| Chain | Chain ID | Privacy Mechanism | Features |
|-------|----------|-------------------|----------|
| Zcash | 133 | zk-SNARKs (Sapling, Orchard) | Shielded pools, viewing keys |
| Monero | TBD | RingCT, stealth addresses | Mandatory privacy |

---

**Total Supported:** 2200+ blockchains across 4 families, unified under one type system. 🌐✨
