# decoder-ao

Arweave AO (Actor Oriented) blockchain transaction decoder.

## Overview

This decoder implements support for **Arweave AO**, a hyper-parallel computer built on the Arweave network. AO uses the **Actor Model** for transaction semantics, where transactions are represented as async messages passing between autonomous actors (processes).

## Key Features

- **ANS-104 DataItem parsing**: Fully compliant with the ANS-104 bundled data standard
- **Multi-signature support**: Arweave (RSA), Ethereum (ECDSA), Solana (Ed25519)
- **Per-message TxIR**: Each message decoded as a separate transaction (per-message decoding strategy)
- **Event sourcing**: State derived from message history
- **Pure Rust**: Zero external dependencies for parsing (airgapped operation)

## Architecture

### Actor Model Semantics

Unlike traditional blockchains (UTXO/Account/Instruction models), AO uses the **Actor Model**:

| Aspect | Traditional Chains | AO (Actor Model) |
|--------|-------------------|------------------|
| **Transaction Unit** | Full transaction | Individual message |
| **Concurrency** | Sequential | Async message passing |
| **State Model** | Balance/UTXO | Message-derived (event sourcing) |
| **Continuations** | None | Parent/child message links |

### Per-Message Decoding

Each AO message is decoded as a **separate TxIR**:

```
User → Process A → Process B → Process C
  └─ TxIR #1       └─ TxIR #2       └─ TxIR #3
     (parent)         (child of #1)    (child of #2)
```

Messages are linked via `metadata.extra`:
- `parent_message`: ID of the message that spawned this one
- `spawned_calls`: IDs of child messages created

## Usage

```rust
use decoder_ao::{AODecoder, AOChain};
use universal_decoder_core::traits::ChainDecoder;

// Decode AO mainnet message
let decoder = AODecoder::new();
let tx_ir = decoder.decode(message_bytes)?;

// Access AO-specific metadata
let action = tx_ir.metadata.extra["action"];
let target_process = tx_ir.metadata.extra["target"];
let tags = tx_ir.metadata.extra["tags"];

// Check message ordering
if let Some(epoch) = tx_ir.metadata.extra.get("epoch") {
    println!("Message epoch: {}", epoch);
}
if let Some(nonce) = tx_ir.metadata.extra.get("nonce") {
    println!("Message nonce: {}", nonce);
}
```

## ANS-104 DataItem Format

AO messages follow the ANS-104 specification:

```
| Field               | Size          | Description                     |
|---------------------|---------------|---------------------------------|
| signature_type      | 2 bytes       | Signature algorithm ID          |
| signature           | Variable      | Cryptographic signature         |
| owner               | Variable      | Public key                      |
| target_present      | 1 byte        | Target present flag (0/1)       |
| target              | 32 bytes      | Process ID (if present)         |
| anchor_present      | 1 byte        | Anchor present flag (0/1)       |
| anchor              | 32 bytes      | Replay prevention (if present)  |
| number_of_tags      | 8 bytes       | Tag count                       |
| number_of_tag_bytes | 8 bytes       | Total tag data size             |
| tags                | Variable      | Avro-encoded tags               |
| data                | Variable      | Message payload                 |
```

## Signature Types

| Type | ID | Signature Size | Owner Size | Algorithm |
|------|----|----------------|------------|-----------|
| Arweave | 1 | 512 bytes | 512 bytes | RSA-PSS 4096-bit |
| Ethereum | 3 | 65 bytes | 65 bytes | ECDSA secp256k1 |
| Solana | 4 | 64 bytes | 32 bytes | Ed25519 |

## State Deltas Mapping

### AO Message State Delta

```rust
StateDeltas {
    inputs: vec![],   // AO doesn't use UTXO model
    outputs: vec![],  // AO doesn't use UTXO model

    account_changes: vec![
        AccountChange {
            address: process_id,
            nonce: Some(message_nonce),
            balance_change: None,  // AO doesn't track balances in messages
            storage_changes: vec![],  // State derived from message history
        }
    ],
}
```

### Metadata Extras

```json
{
  "message_type": "ao_message",
  "target": "process_xyz...",
  "signature_type": "Solana",
  "epoch": 42,
  "nonce": 100,
  "tags": [
    {"name": "Action", "value": "Transfer"},
    {"name": "From", "value": "sender123"}
  ]
}
```

## Message Ordering

AO messages are ordered by:
1. **Epoch**: Global ordering assigned by Scheduler Unit
2. **Nonce**: Uniqueness within epoch

```rust
messages.sort_by(|a, b| {
    (a.epoch, a.nonce).cmp(&(b.epoch, b.nonce))
});
```

## Common AO Tags

- `Action`: Message action (e.g., "Transfer", "Eval", "Spawn-Process")
- `From`: Sender identifier
- `Target`: Recipient process
- `Data-Protocol`: Data format (e.g., "ao")
- `Type`: Message type (e.g., "Message", "Process")

## Testing

```bash
# Run all tests
cargo test -p decoder-ao

# Run with output
cargo test -p decoder-ao -- --nocapture

# Run specific test
cargo test -p decoder-ao test_decode_message_with_action
```

## References

- [AO Whitepaper](https://5z7leszqicjtb6bjtij34ipnwjcwk3owtp7szjirboxmwudpd2tq.arweave.net/7n6ySzBAkzD4KZoTviHtskVlbdab_yylEQuuy1BvHqc)
- [ANS-104 Specification](https://github.com/ArweaveTeam/arweave-standards/blob/master/ans/ANS-104.md)
- [AO Messaging Guide](https://cookbook_ao.arweave.dev/tutorials/begin/messaging.html)
- [Actor Model Chains Design](../../docs/ACTOR_MODEL_CHAINS.md)

## License

MIT OR Apache-2.0
