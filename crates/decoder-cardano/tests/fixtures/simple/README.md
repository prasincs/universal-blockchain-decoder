# Cardano Simple Test Fixtures

## Overview

This directory contains synthetic, hand-crafted Cardano transaction fixtures for initial decoder testing. These are not real blockchain transactions but representative examples that demonstrate the structure and encoding of Cardano transactions.

## Files

### cardano_simple_ada_transfer

**Description**: Simple ADA transfer transaction

- **File**: `cardano_simple_ada_transfer.cbor.hex`
- **Metadata**: `cardano_simple_ada_transfer.json`
- **Format**: CBOR (Concise Binary Object Representation)
- **Era**: Shelley

**Structure**:
- 1 input (previous UTXO)
- 1 output (recipient address)
- 0.2 ADA fee
- No native tokens

**Expected Values**:
- Input count: 1
- Output count: 1
- Total output: 2.0 ADA
- Fee: 0.2 ADA

### cardano_multi_asset_transfer

**Description**: Multi-asset transaction with native tokens

- **File**: `cardano_multi_asset_transfer.cbor.hex`
- **Metadata**: `cardano_multi_asset_transfer.json`
- **Format**: CBOR
- **Era**: Shelley (Mary era token support)

**Structure**:
- 1 input
- 1 output with:
  - 1.5 ADA
  - 1000 units of a custom native token
- 0.25 ADA fee

**Expected Values**:
- Input count: 1
- Output count: 1
- ADA output: 1.5
- Native token count: 1
- Token quantity: 1000
- Fee: 0.25 ADA

## Format Details

### CBOR Encoding

Cardano transactions are encoded using CBOR (Concise Binary Object Representation), a compact binary data format.

Transaction body structure (CBOR array):
```
[
  inputs: [[tx_hash, index], ...],
  outputs: [[address, value], ...],
  fee: integer,
  ttl: integer (optional),
  validity_start: integer (optional)
]
```

### Hex Format

Each `.cbor.hex` file contains the raw CBOR-encoded transaction bytes as a hexadecimal string (no spaces or newlines).

**Example**:
```
8381825820f8d9459f007df845c666c4a4c3b6f1e8d9459f007df845c666c4a4c3b6f1e8d9008182...
```

### JSON Metadata

The `.json` files document expected transaction properties and decoding results:
- Transaction description
- Input/output details
- Amounts in lovelace and ADA
- Expected properties for validation

## Units

Cardano amounts are in **lovelace**, the smallest unit:
- 1 ADA = 1,000,000 lovelace
- Displayed amounts: use `amount / 1,000,000` for ADA conversion

## Usage

Load and decode these fixtures in integration tests:

```rust
#[test]
fn test_decode_cardano_simple_transfer() {
    let hex = include_str!("fixtures/simple/cardano_simple_ada_transfer.cbor.hex");
    let cbor_bytes = hex::decode(hex.trim()).expect("Valid hex");

    let tx = CardanoDecoder::decode(&cbor_bytes)
        .expect("Valid Cardano transaction");

    assert_eq!(tx.inputs().len(), 1);
    assert_eq!(tx.outputs().len(), 1);
    assert_eq!(tx.fee(), 200000); // 0.2 ADA
}
```

## Notes

- **Synthetic**: These are hand-crafted examples, not real blockchain transactions
- **Deterministic**: CBOR encoding is deterministic and canonical
- **Testnet Compatible**: Addresses are examples; not valid on mainnet
- **Era**: All fixtures use Shelley era or later (post-2020)

## References

- [Cardano CBOR Specification](https://github.com/input-output-hk/cardano-ledger)
- [Cardano Address Format](https://cips.cardano.org/cips/cip19/)
- [Shelley Era Details](https://github.com/input-output-hk/cardano-node/wiki/important-pre-release-readiness-information)
