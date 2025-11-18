# Algorand Simple Test Fixtures

## Overview

This directory contains synthetic, hand-crafted Algorand transaction fixtures for initial decoder testing. These are not real blockchain transactions but representative examples that demonstrate the structure and encoding of Algorand transactions.

## Files

### algorand_simple_payment

**Description**: Simple payment transaction (ALGO transfer)

- **File**: `algorand_simple_payment.msgpack.hex`
- **Metadata**: `algorand_simple_payment.json`
- **Format**: MessagePack (binary serialization)
- **Network**: Testnet

**Structure**:
- Sender: 72f1991d... (32-byte public key)
- Receiver: 932b6bd8...
- Amount: 1,000,000 microAlgos (1 ALGO)
- Fee: 1,000 microAlgos
- Validity: 1000 rounds

**Expected Values**:
- Transaction type: payment
- Amount: 1.0 ALGO
- Fee: 0.001 ALGO
- Validity rounds: 1000

### algorand_asset_transfer

**Description**: Asset transfer (Algorand Standard Asset - ASA)

- **File**: `algorand_asset_transfer.msgpack.hex`
- **Metadata**: `algorand_asset_transfer.json`
- **Format**: MessagePack
- **Network**: Testnet

**Structure**:
- Token (ASA) ID: 12345
- Asset amount: 500,000 units
- Fee: 1,000 microAlgos
- Sender and receiver as in simple payment

**Expected Values**:
- Transaction type: asset_transfer (axfer)
- Asset ID: 12345
- Asset amount: 500,000
- Fee: 0.001 ALGO

### algorand_key_registration

**Description**: Key registration for stake participation

- **File**: `algorand_key_registration.msgpack.hex`
- **Metadata**: `algorand_key_registration.json`
- **Format**: MessagePack
- **Network**: Testnet

**Structure**:
- Vote key: VRF verification key (32 bytes)
- Selection key: VRF selection key (32 bytes)
- Vote first valid round: 1000
- Vote last valid round: 10000
- Vote key dilution: 1000

**Expected Values**:
- Transaction type: key_registration (keyreg)
- Participation enabled: true
- Key validity: 9000 rounds

## Format Details

### MessagePack Encoding

Algorand transactions are encoded using MessagePack, a compact binary serialization format.

Transaction structure (MessagePack map):
```
{
  "snd": bytes(32),           // Sender address
  "rcv": bytes(32),           // Receiver address (optional)
  "amt": integer,             // Amount in microAlgos
  "fee": integer,             // Fee in microAlgos
  "fv": integer,              // First valid round
  "lv": integer,              // Last valid round
  "note": bytes,              // Transaction note (optional)
  "type": string,             // Transaction type: "pay", "axfer", "keyreg", etc.
  // ... type-specific fields
}
```

### Hex Format

Each `.msgpack.hex` file contains the raw MessagePack-encoded transaction bytes as a hexadecimal string.

**Example**:
```
88a3736e64c42072f1991d4f6d643bbc69ee49fa7286926d7f002b5f113f88becc4baeb78f820e...
```

### JSON Metadata

The `.json` files document:
- Transaction description and type
- Field-by-field breakdown with descriptions
- Amounts in both microAlgos and ALGO
- Expected properties for validation

## Units

Algorand amounts are in **microAlgos**, the smallest unit:
- 1 ALGO = 1,000,000 microAlgos
- Display as decimal: `amount / 1_000_000`

## Transaction Types

- **pay**: Simple ALGO payment
- **axfer**: Asset (token) transfer
- **afrz**: Asset freeze
- **acfg**: Asset configuration
- **keyreg**: Key registration for consensus participation

## Usage

Load and decode these fixtures in integration tests:

```rust
#[test]
fn test_decode_algorand_payment() {
    let hex = include_str!("fixtures/simple/algorand_simple_payment.msgpack.hex");
    let msgpack_bytes = hex::decode(hex.trim()).expect("Valid hex");

    let tx = AlgorandDecoder::decode(&msgpack_bytes)
        .expect("Valid Algorand transaction");

    assert_eq!(tx.transaction_type(), "pay");
    assert_eq!(tx.amount(), 1_000_000);
    assert_eq!(tx.fee(), 1_000);
}
```

## Notes

- **Synthetic**: Hand-crafted examples, not from real blockchain
- **Deterministic**: MessagePack encoding is deterministic
- **Testnet**: Example addresses are for testnet demonstration
- **Round Numbers**: Validity rounds are example values

## References

- [Algorand Transaction Format](https://developer.algorand.org/docs/get-details/transactions/)
- [MessagePack Specification](https://msgpack.org/)
- [Algorand SDK Documentation](https://github.com/algorand/go-algorand)
