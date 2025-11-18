# AO Message Test Fixtures

This directory contains test fixtures for AO (Arweave Actor Oriented) messages.

## Fixture Types

### 1. Ethereum Signature Messages
- `ao_message_eth_eval.bin` - Eval action with Ethereum signature
- `ao_message_eth_transfer.bin` - Transfer action with Ethereum signature

### 2. Solana Signature Messages
- `ao_message_solana_spawn.bin` - Spawn-Process action with Solana signature
- `ao_message_solana_minimal.bin` - Minimal message (no target, no tags)

### 3. Complex Messages
- `ao_message_with_anchor.bin` - Message with replay protection anchor
- `ao_message_multi_tags.bin` - Message with multiple tags

## Structure

All fixtures follow the ANS-104 DataItem format:

```
[2 bytes] Signature Type (1=Arweave, 3=Ethereum, 4=Solana)
[N bytes] Signature (512 for Arweave, 65 for Ethereum, 64 for Solana)
[M bytes] Owner/Public Key (512 for Arweave, 65 for Ethereum, 32 for Solana)
[1 byte]  Target Present (0 or 1)
[32 bytes] Target (if present)
[1 byte]  Anchor Present (0 or 1)
[32 bytes] Anchor (if present)
[8 bytes] Number of Tags (big-endian u64)
[8 bytes] Number of Tag Bytes (big-endian u64)
[N bytes] Tags (Avro encoded)
[N bytes] Data (payload)
```

## Tags

Common AO tags:
- `Action`: Message action (e.g., "Eval", "Transfer", "Spawn-Process")
- `Data-Protocol`: Always "ao"
- `Type`: "Message" or "Process"
- `From`: Sender identifier
- `Amount`: Transfer amount (for token transfers)

## Usage

```rust
use std::fs;

let fixture = fs::read("tests/fixtures/ao_message_eth_eval.bin").unwrap();
let tx = AODecoder::decode(&fixture).unwrap();
```

## Generation

These fixtures were generated using the ANS-104 specification and represent realistic AO messages that would appear on mainnet.
