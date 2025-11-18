# Mina Protocol Simple Test Fixtures

## Overview

This directory contains synthetic, hand-crafted Mina Protocol transaction fixtures for initial decoder testing. These are not real blockchain transactions but representative examples that demonstrate the structure and encoding of Mina transactions.

## Files

### mina_simple_payment

**Description**: Simple payment transaction (MINA transfer)

- **File**: `mina_simple_payment.hex`
- **Metadata**: `mina_simple_payment.json`
- **Format**: Custom binary encoding (version byte + fields)
- **Network**: testnet

**Structure**:
- Version: 1
- Nonce: 1 (account sequence number)
- From: B62qrPN... (base58check address)
- To: B62qnzbX... (base58check address)
- Amount: 1,000,000,000 nanoMINA (1000 MINA)
- Fee: 1,000,000 nanoMINA (1 MINA)
- Memo: "Test payment"

**Expected Values**:
- Transaction type: payment
- Amount: 1000 MINA
- Fee: 1 MINA
- Network: testnet

### mina_delegation

**Description**: Stake delegation transaction

- **File**: `mina_delegation.hex`
- **Metadata**: `mina_delegation.json`
- **Format**: Custom binary encoding (version 2 for delegation)
- **Network**: testnet

**Structure**:
- Version: 2 (delegation type)
- Nonce: 2
- Delegator: B62qrPN...
- Delegatee (block producer): B62qpJiA...
- Fee: 1,000,000 nanoMINA (1 MINA)
- Memo: "Delegate stake"

**Expected Values**:
- Transaction type: delegation
- Delegator: B62qrPN...
- Delegatee: B62qpJiA...
- Fee: 1 MINA

### mina_payment_memo

**Description**: Payment with extended memo field

- **File**: `mina_payment_memo.hex`
- **Metadata**: `mina_payment_memo.json`
- **Format**: Custom binary encoding
- **Network**: mainnet

**Structure**:
- Version: 1
- Nonce: 3
- From: B62qpJiA... (different sender)
- To: B62qrPN...
- Amount: 5,000,000,000 nanoMINA (5000 MINA)
- Fee: 1,000,000 nanoMINA (1 MINA)
- Memo: "Invoice #12345 - Monthly subscription payment"

**Expected Values**:
- Transaction type: payment
- Amount: 5000 MINA
- Fee: 1 MINA
- Memo length: 45 bytes
- Network: mainnet

## Format Details

### Binary Encoding

Mina transactions use a custom binary format with the following structure:

```
Version byte (1 byte)
Nonce (8 bytes, big-endian)
From address (32 bytes: SHA256 of public key)
[To address (32 bytes) - for payments]
[Amount (8 bytes, big-endian) - for payments]
[Delegatee (32 bytes) - for delegations]
Fee (8 bytes, big-endian)
Memo length (1 byte)
Memo (variable)
```

### Hex Format

Each `.hex` file contains the raw binary-encoded transaction as a hexadecimal string.

**Example**:
```
01000000000000000164896dcc079e18c58728674f3e0e6f6bd2b8d64205d981a69b4e...
```

**Structure Breakdown**:
- `01` - Version byte
- `0000000000000001` - Nonce (8 bytes)
- `64896dcc...` - From address (32 bytes)
- Rest: To address, amount, fee, memo

### JSON Metadata

The `.json` files document:
- Transaction description and type
- All fields with explanations
- Amounts in both nanoMINA and MINA
- Network designation
- Expected validation properties

## Units

Mina amounts are in **nanoMINA**, the smallest unit:
- 1 MINA = 1,000,000,000 nanoMINA
- Display as decimal: `amount / 1_000_000_000`

## Addresses

Mina uses **base58check encoding** for addresses:
- Format: `B62q...` prefix
- 32-byte public key hash, base58-encoded
- Example: `B62qrPN5Y5yxFQnqVfVawc6fV8YsZbWvsBGg6HkXBw1tMHkdAZRrVLe`

## Transaction Types

Based on version byte:
- **Version 1**: Payment transaction (sender → receiver)
- **Version 2**: Delegation transaction (delegator → delegatee)

## Usage

Load and decode these fixtures in integration tests:

```rust
#[test]
fn test_decode_mina_payment() {
    let hex = include_str!("fixtures/simple/mina_simple_payment.hex");
    let tx_bytes = hex::decode(hex.trim()).expect("Valid hex");

    let tx = MinaDecoder::decode(&tx_bytes)
        .expect("Valid Mina transaction");

    assert_eq!(tx.version(), 1);
    assert_eq!(tx.nonce(), 1);
    assert_eq!(tx.amount(), 1_000_000_000);
    assert_eq!(tx.fee(), 1_000_000);
}
```

## Notes

- **Synthetic**: Hand-crafted examples for testing, not real blockchain data
- **Deterministic**: Binary encoding is deterministic and stable
- **Simplified**: Real Mina transactions include signatures and other fields
- **Testing Only**: These fixtures are for decoder verification

## Mina Protocol Details

### Consensus Model
Mina uses Proof-of-Stake consensus with delegated block production.

### Accounts
- Accounts are identified by Ed25519 public keys
- Addresses are base58check-encoded public key hashes
- Each account has a nonce (sequence number) that increments with each transaction

### Fees
- Network requires minimum fee per byte
- Fees incentivize block producers
- Range: 1-2 MINA per transaction typical

## References

- [Mina Protocol Documentation](https://docs.minaprotocol.com/)
- [Mina Transaction Format](https://github.com/minaProtocol/mina/tree/develop/src/lib/transaction)
- [Base58Check Encoding](https://en.bitcoin.it/wiki/Base58Check_encoding)
