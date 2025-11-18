# Actor Model Chain Family: ICP and AO

**Status**: Phase 3.11 - Implementation Ready
**Created**: 2025-11-18
**Chains**: Internet Computer (ICP), Arweave AO
**Purpose**: Design documentation for Actor Model family decoder based on block explorer research

---

## Executive Summary

Actor Model blockchains (ICP, AO) represent transactions as **asynchronous message-passing between autonomous actors**. Unlike synchronous models (UTXO/Account/Instruction), Actor chains create **message continuations** where a single user action spawns multiple independent messages.

**Key Finding**: Block explorers treat **each message as a separate transaction**, not as continuation chains. This informs our decoder design: **one TxIR per message** with linkage metadata.

---

## Block Explorer Research

### Internet Computer (ICP)

**Official Explorer**: `dashboard.internetcomputer.org`
**Community Explorer**: `ic.rocks`, `icpexplorer.org`

#### What Explorers Display

**Per-Transaction View**:
```
Transaction Hash: abc123...
Type: Call | Transfer | Deploy
From: Principal xyz...
To: Canister rrkah-fqaaa...
Timestamp: 2024-11-18 12:34:56 UTC
Fee: 0.0001 ICP
Status: Success
Memo: ...
Index: 12345
```

**Canister Detail View**:
- Candid interface methods
- Controllers, subnet ID, Wasm hash
- **NOT shown**: Cross-canister call graphs or continuation chains

#### Message Structure (Ingress)

```rust
struct IngressMessage {
    request_type: "call",
    sender: Principal,              // User principal
    nonce: [u8; 32],                // Random bytes for uniqueness
    ingress_expiry: u64,            // Nanoseconds since 1970-01-01
    canister_id: Principal,         // Target canister
    method_name: String,            // Canister method to call
    arg: Vec<u8>,                   // Candid-encoded arguments
}

// request_id is hash of all fields above
```

#### Cross-Canister Call Behavior

**Example Scenario**: User calls `canister_A.outer_call()` which calls `canister_B.inner_call()`

```
Message 1: Ingress (user → canister_A)
  ├─ request_id: "ingress_123"
  ├─ method: "outer_call"
  └─ Spawns inter-canister call...

Message 2: Inter-canister (canister_A → canister_B)
  ├─ message_id: "inter_456"  [separate from ingress]
  ├─ caller: canister_A
  ├─ callee: canister_B
  └─ await response...

Message 3: Callback (canister_A continuation)
  ├─ message_id: "callback_789"
  ├─ Triggered by: "inter_456" response
  └─ Completes "outer_call"
```

**Block Explorer Behavior**: Shows **three separate entries**, each with its own message ID. Users must manually trace relationships.

#### Callback Mechanism

From IC documentation:
- "Messages are divided into **requests** and **responses**"
- "The IC keeps track of the **callback** for responses"
- Each callback is executed as a **separate message execution** (atomically)
- Under the hood: CDKs translate `await` into multiple Wasm handler functions

**Key Insight**: IC runtime tracks continuations internally. Decoders see discrete messages, not continuation chains.

---

### Arweave AO

**Type**: Hyper-parallel computer on Arweave
**Explorer**: Various (ArScan, ViewBlock for Arweave, AO-specific explorers emerging)

#### Message Structure (ANS-104 Data-Items)

```rust
struct AOMessage {
    Data: String | Payload,         // Message content
    Target: ProcessID,              // Recipient process
    Tags: Vec<Tag>,                 // Routing/categorization
    Signature: Signature,           // Cryptographic signature
    Nonce: u64,                     // Message uniqueness
    Epoch: u64,                     // Assigned by Scheduler Unit
}

struct Tag {
    name: String,                   // E.g., "Action", "From"
    value: String,
}
```

#### Message Workflow

```
1. MU (Messenger Unit): Authenticates signature
2. SU (Scheduler Unit): Assigns Epoch + Nonce
3. Bundled with Assignment Type
4. Dispatched to Arweave (permanent storage)
```

#### Process State Derivation

**Event Sourcing Model**:
- AO processes have **no mutable state storage**
- State is **derived from ordered message history**
- Like Git: `state_N = reduce(messages[0..N])`

**Explorer Behavior**: Shows individual messages, process state at any message height (like commit history).

#### Message Ordering

- **Epoch**: Global ordering (assigned by SU)
- **Nonce**: Uniqueness within epoch
- Ordering guarantee: `messages.sort_by(|a, b| (a.epoch, a.nonce).cmp(&(b.epoch, b.nonce)))`

**Key Insight**: AO is message-centric, not transaction-centric. A "transaction" is a sequence of messages to a process.

---

## Design Decision: Per-Message TxIR

### Three Options Considered

#### Option 1: One TxIR per Message ✅ **SELECTED**

**Rationale**:
- Matches how ICP and AO actually work
- Each message has independent message ID / request ID
- Decoding scope = one message (simple, fast)
- Linkage via `metadata.extra` (parent/child IDs)
- Block explorers decide aggregation strategy

**Example**:
```rust
// TxIR for ingress message
TxIR {
    metadata: TxMetadata {
        tx_id: "ingress_123",
        extra: {
            "message_type": "ingress",
            "spawned_calls": ["inter_456"],  // Links to children
        }
    },
    operations: vec![
        Operation::ContractCall {
            contract: "canister_A",
            function: "outer_call",
            args: ...,
        }
    ],
    state_deltas: StateDeltas {
        inputs: vec![/* caller principal, cycles */],
        account_changes: vec![/* canister_A state before await */],
    }
}

// Separate TxIR for inter-canister call
TxIR {
    metadata: TxMetadata {
        tx_id: "inter_456",
        extra: {
            "message_type": "inter_canister",
            "parent_message": "ingress_123",  // Links back
            "caller": "canister_A",
            "callee": "canister_B",
        }
    },
    operations: vec![
        Operation::ContractCall {
            contract: "canister_B",
            function: "inner_call",
        }
    ],
}

// Separate TxIR for callback
TxIR {
    metadata: TxMetadata {
        tx_id: "callback_789",
        extra: {
            "message_type": "callback",
            "parent_message": "ingress_123",
            "triggered_by": "inter_456",
        }
    },
    // Continuation state changes
    state_deltas: StateDeltas {
        account_changes: vec![/* canister_A final state */],
    }
}
```

#### Option 2: One TxIR with Embedded Continuation Chain ❌

**Pros**: Complete call tree in one TxIR
**Cons**:
- Decoder must query multiple messages (violates "decode one tx")
- Complex reconstruction logic (bloats TCB)
- Doesn't match how chains expose data
- State at intermediate await points ambiguous

#### Option 3: Hybrid ❌

**Complexity**: Two modes (per-message + aggregated) = confusing API

### Async Continuations Are Not a Challenge

**Original concern**: "TxIR is synchronous - how to represent async?"

**Resolution**: Async continuations are represented as **sequential messages** with linkage metadata. Each message is independent and deterministic.

**Analogy**: Git commits
- Each commit is independent (one TxIR per commit)
- Parent commit linked via hash (one message per TxIR, parent linked via metadata)
- Full history reconstructed by following links (explorers aggregate TxIRs)

**Updated "Challenges"**:
- ✅ ~~Async Semantics~~: **Solved** - per-message TxIRs with parent/child links
- ✅ **Scope Boundary**: Decode **one message** (not entire call tree)
- ⚠️ **Observability**: Canister/process state might be private (use `ObservabilityLevel`)
- 📊 **Indexing**: How explorers aggregate TxIRs into call graphs (**out of scope** for decoder)

---

## Implementation Guide

### ICP Decoder (`decoder-icp`)

#### Message Types to Support

```rust
enum ICPMessageType {
    Ingress,         // User → Canister
    InterCanister,   // Canister → Canister
    Callback,        // Canister continuation after await
}
```

#### Ingress Message Decoding

```rust
impl ChainDecoder for ICPDecoder {
    fn decode(&self, bytes: &[u8]) -> Result<TxIR> {
        // Parse Candid-encoded ingress message
        let msg = parse_ingress(bytes)?;

        let metadata = TxMetadata {
            tx_id: compute_request_id(&msg),  // Hash of all fields
            timestamp: None,  // Not in ingress (added by replica)
            extra: json!({
                "message_type": "ingress",
                "sender": msg.sender.to_string(),
                "nonce": hex::encode(&msg.nonce),
                "ingress_expiry": msg.ingress_expiry,
                "canister_id": msg.canister_id.to_string(),
                "method_name": msg.method_name,
                // Future: spawned_calls (requires querying IC)
            }),
        };

        let operations = vec![
            Operation::ContractCall(ContractCall {
                contract: msg.canister_id.to_string(),
                function: msg.method_name.clone(),
                args: msg.arg.clone(),
                gas_limit: None,  // ICP uses cycles, not gas
                value: None,      // No value transfer in call
            })
        ];

        let state_deltas = StateDeltas {
            inputs: vec![
                InputReference {
                    prev_tx: vec![],  // No UTXO model
                    output_index: 0,
                    value: Amount::new(0, 0),  // Cycles handled separately
                    script: msg.sender.as_slice().to_vec(),
                }
            ],
            outputs: vec![],  // Populated by callback TxIR
            account_changes: vec![
                AccountChange {
                    address: Address {
                        bytes: msg.canister_id.as_slice().to_vec(),
                        human_readable: Some(msg.canister_id.to_string()),
                    },
                    nonce: None,  // Canisters don't have nonces
                    balance_change: None,  // State change in callback
                    storage_changes: vec![],
                }
            ],
        };

        Ok(TxIR::new(
            &ICPChain,
            metadata,
            authorization,  // Derived from sender principal
            operations,
            state_deltas,
        ))
    }
}
```

#### Cycles Tracking

```rust
// In metadata.extra:
{
    "cycles_sent": 1_000_000,        // Cycles attached to call
    "cycles_returned": 500_000,      // Cycles refunded (callback)
    "cycles_consumed": 500_000,      // Net cycles used
}

// In state_deltas.account_changes:
AccountChange {
    address: canister_id,
    balance_change: Some(-500_000),  // Cycles consumed (negative = spent)
}
```

#### Request ID Computation

```rust
use sha2::{Sha256, Digest};

fn compute_request_id(msg: &IngressMessage) -> String {
    let mut hasher = Sha256::new();

    // Deterministic encoding (CBOR or similar)
    hasher.update(b"ic-request");
    hasher.update(&msg.request_type.as_bytes());
    hasher.update(&msg.sender.as_slice());
    hasher.update(&msg.nonce);
    hasher.update(&msg.ingress_expiry.to_le_bytes());
    hasher.update(&msg.canister_id.as_slice());
    hasher.update(&msg.method_name.as_bytes());
    hasher.update(&msg.arg);

    hex::encode(hasher.finalize())
}
```

---

### AO Decoder (`decoder-ao`)

#### ANS-104 Data-Item Parsing

```rust
impl ChainDecoder for AODecoder {
    fn decode(&self, bytes: &[u8]) -> Result<TxIR> {
        // Parse ANS-104 data item
        let msg = parse_ans104(bytes)?;

        let metadata = TxMetadata {
            tx_id: msg.id.clone(),  // ANS-104 ID
            timestamp: None,  // Derived from Arweave block
            extra: json!({
                "message_type": "ao_message",
                "target": msg.target,
                "epoch": msg.epoch,
                "nonce": msg.nonce,
                "tags": msg.tags,
                "signature": hex::encode(&msg.signature),
            }),
        };

        // Extract action from tags
        let action = msg.tags.iter()
            .find(|t| t.name == "Action")
            .map(|t| t.value.clone())
            .unwrap_or_default();

        let operations = vec![
            Operation::ContractCall(ContractCall {
                contract: msg.target.clone(),
                function: action,
                args: msg.data.as_bytes().to_vec(),
                gas_limit: None,
                value: None,
            })
        ];

        let state_deltas = StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![
                AccountChange {
                    address: Address {
                        bytes: msg.target.as_bytes().to_vec(),
                        human_readable: Some(msg.target.clone()),
                    },
                    nonce: Some(msg.nonce),  // Message nonce
                    balance_change: None,    // AO doesn't track balances in messages
                    storage_changes: vec![],  // State derived from message history
                }
            ],
        };

        Ok(TxIR::new(
            &AOChain,
            metadata,
            authorization,  // Derived from signature
            operations,
            state_deltas,
        ))
    }
}
```

#### Event Sourcing Representation

```rust
// In metadata.extra:
{
    "process_id": "abc123...",
    "message_height": 42,  // This is the 42nd message to this process
    "state_snapshot": {    // Optional: state after applying this message
        "observability": "PartiallyObservable",
        "public_state": { /* ... */ },
    }
}
```

---

## StateDeltas Mapping Strategy

### ICP: Ingress Message

```rust
StateDeltas {
    inputs: vec![
        InputReference {
            prev_tx: vec![],
            output_index: 0,
            value: Amount::new(cycles_sent, 0),  // Cycles attached
            script: caller_principal.as_slice().to_vec(),
        }
    ],

    outputs: vec![],  // Populated in callback TxIR

    account_changes: vec![
        AccountChange {
            address: canister_id,
            nonce: None,  // No nonces
            balance_change: None,  // State changes in callback
            storage_changes: vec![],  // Opaque until callback
        }
    ],
}
```

### ICP: Callback Message

```rust
StateDeltas {
    inputs: vec![],  // Already captured in ingress TxIR

    outputs: vec![
        OutputValue {
            index: 0,
            address: caller_address,  // Response sent back
            value: Amount::new(cycles_returned, 0),
            script: response_data,
        }
    ],

    account_changes: vec![
        AccountChange {
            address: canister_id,
            nonce: None,
            balance_change: Some(-cycles_consumed),  // Net cycles used
            storage_changes: vec![
                // Canister state mutations (if observable)
            ],
        }
    ],
}
```

### AO: Process Message

```rust
StateDeltas {
    inputs: vec![],
    outputs: vec![],

    account_changes: vec![
        AccountChange {
            address: process_id,
            nonce: Some(message_nonce),
            balance_change: None,  // AO doesn't have balances
            storage_changes: vec![],  // State derived from message history
        }
    ],
}

// In metadata.extra:
{
    "state_transition": {
        "before_hash": "previous_state_hash",
        "after_hash": "new_state_hash",
        "observability": "FullyObservable" | "PartiallyObservable" | "FullyPrivate",
    }
}
```

---

## Observability Levels

Using Phase 3.0 privacy extensions:

```rust
// In TxIR:
privacy: Some(PrivacyMetadata {
    features: vec![PrivacyFeature::HiddenState],  // Canister state opaque
    observability: ObservabilityLevel::PartiallyObservable,
    viewing_key: None,  // ICP doesn't use viewing keys (yet)
})
```

**Levels**:
- **FullyObservable**: Public canister state (rare)
- **PartiallyObservable**: Method name visible, state opaque (typical)
- **FullyPrivate**: Encrypted canisters (theoretical, not common)

---

## Testing Strategy

### Property Tests

```rust
proptest! {
    #[test]
    fn message_order_preserved(messages in vec_of_icp_messages()) {
        // Decode each message
        let txirs: Vec<TxIR> = messages.iter()
            .map(|m| ICPDecoder.decode(m))
            .collect::<Result<_>>()?;

        // Verify linkage via metadata
        for i in 1..txirs.len() {
            if is_continuation(&txirs[i]) {
                assert_eq!(
                    txirs[i].metadata.extra["parent_message"],
                    txirs[i-1].metadata.tx_id
                );
            }
        }
    }

    #[test]
    fn cycles_conservation(ingress_txir, callback_txir) {
        let cycles_sent = get_cycles_sent(&ingress_txir);
        let cycles_returned = get_cycles_returned(&callback_txir);
        let cycles_consumed = get_cycles_consumed(&callback_txir);

        assert_eq!(cycles_sent, cycles_returned + cycles_consumed);
    }
}
```

### Integration Tests

```rust
#[test]
fn decode_icp_nns_governance_vote() {
    // Real NNS governance canister vote transaction
    let bytes = include_bytes!("fixtures/icp_nns_vote.bin");

    let txir = ICPDecoder.decode(bytes).unwrap();

    assert_eq!(txir.chain.family, ChainFamily::Actor);
    assert_eq!(txir.metadata.extra["canister_id"], "rrkah-fqaaa-aaaaa-aaaaq-cai");
    assert_eq!(txir.metadata.extra["method_name"], "manage_neuron");

    // Verify operation
    match &txir.operations[0] {
        Operation::ContractCall(call) => {
            assert_eq!(call.contract, "rrkah-fqaaa-aaaaa-aaaaq-cai");
            assert_eq!(call.function, "manage_neuron");
        },
        _ => panic!("Expected ContractCall"),
    }
}

#[test]
fn decode_ao_process_spawn() {
    let bytes = include_bytes!("fixtures/ao_spawn_process.bin");

    let txir = AODecoder.decode(bytes).unwrap();

    assert_eq!(txir.chain.family, ChainFamily::Actor);
    assert_eq!(txir.metadata.extra["message_type"], "ao_message");

    // Verify ANS-104 structure
    let tags = &txir.metadata.extra["tags"];
    assert!(tags.as_array().unwrap().iter().any(|t|
        t["name"] == "Action" && t["value"] == "Spawn-Process"
    ));
}
```

---

## Comparison with Other Models

| Aspect | UTXO (Bitcoin) | Account (Ethereum) | Instruction (Solana) | **Actor (ICP/AO)** |
|--------|----------------|--------------------|-----------------------|---------------------|
| **Transaction Unit** | Input/output set | State mutation | Instruction batch | **Message** |
| **Concurrency** | Sequential (UTXO ordering) | Sequential (nonce) | Parallel (multi-instruction) | **Async (message queue)** |
| **State Model** | Spent/unspent | Balance/nonce | Account read/write | **Message-derived** |
| **Atomicity** | Full tx atomic | Full tx atomic | Full tx atomic | **Per-message atomic** |
| **Continuations** | None | None | None | **Parent/child messages** |
| **Decoder Scope** | Entire transaction | Entire transaction | Entire transaction | **One message** |
| **StateDeltas** | inputs/outputs | account_changes | Mixed | **Message deltas + links** |

---

## Roadmap Integration

**Phase 3.11 Implementation**:

1. **Week 1**: ICP decoder
   - Candid ingress message parsing
   - Request ID computation
   - Cycles tracking
   - 20 unit tests

2. **Week 2**: AO decoder
   - ANS-104 data-item parsing
   - Tag extraction
   - Event sourcing metadata
   - 15 unit tests

3. **Week 3**: Integration
   - Property tests (message ordering, cycles conservation)
   - Real transaction fixtures (NNS votes, AO spawns)
   - Documentation finalization

**Success Criteria**:
- ✅ Decode ICP ingress messages
- ✅ Decode AO ANS-104 data-items
- ✅ Parent/child message linkage in metadata
- ✅ 35+ tests passing
- ✅ Integration with real ICP/AO transactions

---

## Use Cases for Conferences & Papers

### 1. Novel Concurrency Model

**Talking Point**: "Traditional blockchains serialize transactions. Actor chains enable true parallelism via message-passing."

**Demo**: Show TxIR for cross-canister call tree:
```
User → Canister A → Canister B → Canister C
  └─ TxIR chain with parent/child links
```

### 2. Continuation Representation

**Talking Point**: "Async isn't hard to represent - it's just sequential messages with linkage."

**Demo**: Compare:
- Ethereum (one TxIR, synchronous)
- ICP (three TxIRs, linked continuations)

### 3. Observability Spectrum

**Talking Point**: "Actor state can be opaque. TxIR handles this with observability levels."

**Demo**: Show `ObservabilityLevel` from privacy extensions applied to canister state.

### 4. Event Sourcing

**Talking Point**: "AO processes have no mutable state - state is derived from message history."

**Demo**: Show AO message sequence → state derivation (like Git commits).

---

## References

### ICP Documentation
- [IC Interface Specification](https://internetcomputer.org/docs/references/ic-interface-spec)
- [Ingress Messages](https://internetcomputer.org/docs/references/ingress-messages)
- [Inter-canister Calls](https://internetcomputer.org/docs/references/async-code)
- [Message Execution](https://internetcomputer.org/docs/references/message-execution-properties)

### AO Documentation
- [AO Protocol Whitepaper](https://5z7leszqicjtb6bjtij34ipnwjcwk3owtp7szjirboxmwudpd2tq.arweave.net/7n6ySzBAkzD4KZoTviHtskVlbdab_yylEQuuy1BvHqc)
- [AO Messaging](https://cookbook_ao.arweave.dev/tutorials/begin/messaging.html)
- [ANS-104 Specification](https://cookbook_ao.arweave.net/guides/aoconnect/spawning-processes.html)

### Block Explorers
- [ICP Dashboard](https://dashboard.internetcomputer.org/)
- [IC.rocks](https://ic.rocks/)
- [ArScan](https://arscan.io/)
- [ViewBlock Arweave](https://viewblock.io/arweave)

---

**Last Updated**: 2025-11-18
**Author**: Universal Blockchain Decoder Project
**Status**: Implementation-ready based on block explorer research
