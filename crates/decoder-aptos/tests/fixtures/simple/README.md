# Simple Aptos Transaction Fixtures

This directory contains 5 basic Aptos transaction test vectors in BCS (Binary Canonical Serialization) format.

## Quick Start

Each fixture file pair consists of:
- `{name}.bcs.hex` - The raw BCS-encoded bytes as hexadecimal
- `{name}.json` - Metadata describing the transaction

To use in tests:
```rust
// Read hex file
let hex = include_str!("simple/webauthn_single_key_transfer.bcs.hex");
let bytes = hex::decode(hex).unwrap();

// Deserialize RawTransaction using BCS
let raw_txn: RawTransaction = bcs::from_bytes(&bytes).unwrap();

// Validate against JSON metadata
assert_eq!(raw_txn.sender(), Address::from_str(
    "0x0000000000000000000000000000000000000000000000000000000000000001"
).unwrap());
```

## Fixture Overview

| Name | Size | Type | Authentication |
|------|------|------|---|
| webauthn_single_key_transfer | 226 bytes | Entry Function | WebAuthn/secp256r1 |
| simple_ed25519_transfer | 161 bytes | Entry Function | ED25519 |
| secp256k1_coin_transfer | 217 bytes | Entry Function | secp256k1 ECDSA |
| multi_agent_transfer | 161 bytes | Entry Function | Multi-agent (3 signers) |
| fee_payer_transfer | 161 bytes | Entry Function | Fee payer model |

## Fixture Details

### webauthn_single_key_transfer
- **Source**: Official Aptos Core test (`authenticator.rs::tests::verify_webauthn_single_key_auth`)
- **Real test data**: ✓ Yes, extracted directly from test
- **Includes actual bytes**: ✓ Yes (raw_txn_bcs_bytes from test)
- **Tests**: WebAuthn/passkey authentication (increasingly common for keyless transactions)

### simple_ed25519_transfer
- **Pattern**: Standard coin transfer
- **Tests**: Most common transaction type on mainnet
- **Covers**: Basic ED25519 single-sender flow

### secp256k1_coin_transfer
- **Pattern**: Cross-chain compatible signatures
- **Tests**: Bitcoin/Ethereum-compatible key support
- **Uses**: secp256k1 curve (same as Bitcoin/Ethereum)

### multi_agent_transfer
- **Pattern**: Multi-signature atomic transactions
- **Tests**: Transactions requiring multiple signers
- **Uses**: Escrow, atomic swaps, group governance

### fee_payer_transfer
- **Pattern**: Sponsor payment model
- **Tests**: Gas sponsorship and fee abstraction
- **Uses**: User onboarding, dApp transaction sponsorship

## Data Format

### BCS Hex Files

Raw hexadecimal encoding of `RawTransaction`:
```
RawTransaction {
    sender: AccountAddress,           // 32 bytes
    sequence_number: u64,              // 8 bytes
    payload: TransactionPayload,       // variable length
    max_gas_amount: u64,               // 8 bytes
    gas_unit_price: u64,               // 8 bytes
    expiration_timestamp_secs: u64,    // 8 bytes
    chain_id: u8                       // 1 byte
}
```

### JSON Metadata Files

Structured metadata for validation:
```json
{
  "name": "fixture_name",
  "description": "Human-readable description",
  "transaction_type": "raw_transaction",
  "sender": "0x...",
  "sequence_number": 0,
  "chain_id": 1,
  "gas": { "max_gas_amount": 1000, "gas_unit_price": 100 },
  "expiration_timestamp_secs": 9999999999,
  "payload": {
    "type": "entry_function",
    "function": "0x1::module::function",
    "type_arguments": [],
    "arguments": []
  },
  "authenticator": { ... },
  "source": "origin of this fixture",
  "bcs_hex": "raw hex for verification"
}
```

## Chain IDs

- 1 = Mainnet
- 2 = Testnet
- 3 = Devnet
- 4 = Local
- 89 = Custom (used in webauthn fixture from official tests)

## Important Notes

1. **RawTransaction Only**: These fixtures contain `RawTransaction` (pre-signature form), not `SignedTransaction`
   - To test full signing flow, you'll need to add the `authenticator` field
   - See authenticator.rs tests for full SignedTransaction examples

2. **Test Addresses**: All fixtures use simple recognizable test addresses (0x0000...0001, etc.)
   - Not real mainnet addresses
   - Chosen for clarity in testing

3. **Test Data**: webauthn fixture contains actual test data from Aptos Core
   - The raw_txn_bcs_bytes are directly from the verify_webauthn_single_key_auth test
   - Other fixtures derived from test patterns in the official codebase

4. **BCS Format**: All data uses Aptos BCS (Binary Canonical Serialization)
   - Deterministic encoding
   - Canonical form used for hashing and signatures
   - See https://github.com/zefchain/bcs for spec

## Validation Checklist

When implementing an Aptos decoder, validate against these fixtures:

- [ ] Parse BCS hex to bytes correctly
- [ ] Deserialize RawTransaction with correct field ordering
- [ ] Extract sender address (32 bytes)
- [ ] Parse sequence number (u64 LE)
- [ ] Identify payload type (discriminant byte)
- [ ] Parse entry function details (module, function, args)
- [ ] Extract gas parameters
- [ ] Extract expiration timestamp
- [ ] Identify chain ID
- [ ] Compare all parsed values against JSON metadata
- [ ] Verify byte counts match expected sizes
- [ ] Re-serialize and verify round-trip (encode(decode(x)) == x)

## Extension

To add more fixtures:

1. Extract test data from `aptos-core/types/src/transaction/authenticator.rs`
2. Create `.bcs.hex` file with hex-encoded bytes
3. Create `.json` metadata file with parsed structure
4. Add to FIXTURES_SUMMARY.md with source reference
5. Run verification script to validate format

## Related Resources

- **Aptos Core Repo**: https://github.com/aptos-labs/aptos-core
- **Test Source**: `types/src/transaction/authenticator.rs` (tests module)
- **BCS Spec**: https://github.com/zefchain/bcs
- **Aptos Transactions**: https://aptos.dev/reference/blockchain
- **Move Language**: https://aptos.dev/move/guide

## Testing Integration

Example Rust test:
```rust
#[test]
fn test_decode_webauthn_fixture() {
    let hex = include_str!("fixtures/simple/webauthn_single_key_transfer.bcs.hex");
    let bytes = hex::decode(hex).expect("invalid hex");
    let raw_txn: RawTransaction = bcs::from_bytes(&bytes)
        .expect("failed to deserialize");

    assert_eq!(
        raw_txn.sender().to_hex_literal(),
        "0x0000000000000000000000000000000000000000000000000000000000000001"
    );
    assert_eq!(raw_txn.sequence_number(), 0);
    assert_eq!(raw_txn.chain_id().id(), 89);
}
```

---

**Version**: 1.0
**Created**: November 18, 2025
**Total Fixtures**: 5
**Total Size**: 926 bytes
**Format**: BCS (Binary Canonical Serialization)
