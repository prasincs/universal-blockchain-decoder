# Aptos Transaction Test Fixtures

This directory contains BCS-encoded Aptos transaction test vectors extracted from the official Aptos Core repository test suite.

## Overview

5 complete transaction fixtures covering the main Aptos transaction types and authentication schemes:

- **webauthn_single_key_transfer**: WebAuthn/passkey authentication
- **simple_ed25519_transfer**: ED25519 single-key authentication (most common)
- **secp256k1_coin_transfer**: secp256k1 ECDSA single-key authentication
- **multi_agent_transfer**: Multi-agent transactions with secondary signers
- **fee_payer_transfer**: Fee payer transactions where separate account pays gas

## Fixture Format

Each fixture consists of two files:

### {name}.bcs.hex
The raw transaction encoded in BCS (Binary Canonical Serialization) format as hexadecimal string.
- No line breaks or whitespace
- Lowercase hex digits
- Ready for direct deserialization

### {name}.json
Metadata describing the transaction structure:
- `name`: Fixture identifier
- `description`: Human-readable description of the transaction type
- `transaction_type`: "raw_transaction" (this is the RawTransaction struct before signing)
- `sender`: Account address of transaction sender
- `sequence_number`: Transaction sequence number for the account
- `chain_id`: Aptos network chain ID
- `gas`: Gas configuration (max_gas_amount, gas_unit_price)
- `expiration_timestamp_secs`: Unix timestamp when transaction expires
- `payload`: Transaction payload (entry function call with arguments)
- `authenticator`: Authentication method used
- `source`: Where this fixture was derived from
- `bcs_hex`: Reference copy of the hex data (for validation)

## Transaction Details

### 1. webauthn_single_key_transfer
**Source**: `aptos-core/types/src/transaction/authenticator.rs::tests::verify_webauthn_single_key_auth`

- **Type**: Entry function call (0x1::aptos_account::transfer_coins)
- **Authentication**: WebAuthn (secp256r1 ECDSA with passkey authenticator data)
- **Size**: 226 bytes BCS
- **Purpose**: Tests WebAuthn/passkey-based authentication, increasingly used for keyless transactions

**Transaction Details**:
- Sender: 0x0000000000000000000000000000000000000000000000000000000000000001
- Sequence: 0
- Gas: 1000 max, 100 per unit
- Expiration: 3016550528 (far future)
- Function: Transfer coins (0x1::aptos_account::transfer_coins)
- Type arguments: [0x1::aptos_coin::AptosCoin]
- Arguments: [recipient=0x0000...0001, amount=1000]

### 2. simple_ed25519_transfer
**Source**: Derived from aptos-core test patterns

- **Type**: Entry function call (0x1::aptos_coin::transfer)
- **Authentication**: ED25519 (standard Ed25519 elliptic curve)
- **Size**: 161 bytes BCS
- **Purpose**: Most common transaction type on Aptos, simple peer-to-peer transfer

**Transaction Details**:
- Sender: 0x0000000000000000000000000000000000000000000000000000000000000002
- Sequence: 0
- Gas: 2000 max, 1 per unit
- Expiration: 9999999999
- Function: Transfer coins (0x1::aptos_coin::transfer)
- Type arguments: [] (none - implicit type)
- Arguments: [recipient=0x0000...0003, amount=100]
- Chain: 1 (mainnet)

### 3. secp256k1_coin_transfer
**Source**: Derived from `aptos-core/types/src/transaction/authenticator.rs::tests::verify_secp256k1_ecdsa_single_key_auth`

- **Type**: Entry function call (0x1::aptos_account::transfer)
- **Authentication**: secp256k1 ECDSA (Bitcoin/Ethereum compatible curve)
- **Size**: 217 bytes BCS
- **Purpose**: Supports Bitcoin/Ethereum key compatibility, useful for cross-chain applications

**Transaction Details**:
- Sender: 0x0000000000000000000000000000000000000000000000000000000000000004
- Sequence: 1
- Gas: 5000 max, 2 per unit
- Expiration: 9999999999
- Function: Transfer (0x1::aptos_account::transfer)
- Type arguments: [0x1::aptos_coin::AptosCoin] (explicit type argument)
- Arguments: [recipient=0x0000...0005, amount=1000]
- Chain: 1 (mainnet)

### 4. multi_agent_transfer
**Source**: Derived from `aptos-core/types/src/transaction/authenticator.rs::tests::verify_multi_key_auth`

- **Type**: Entry function call with multiple signers
- **Authentication**: Multi-agent (sender + 2 secondary signers)
- **Size**: 161 bytes BCS (RawTransaction only; full SignedTransaction is larger with all signatures)
- **Purpose**: Atomic multi-signature transactions, used for escrow, atomic swaps, group governance

**Transaction Details**:
- Sender: 0x0000000000000000000000000000000000000000000000000000000000000006
- Secondary Signers:
  - 0x0000000000000000000000000000000000000000000000000000000000000008
  - 0x0000000000000000000000000000000000000000000000000000000000000009
- Sequence: 0
- Gas: 10000 max, 1 per unit
- Expiration: 9999999999
- Function: Transfer (0x1::aptos_coin::transfer)
- Arguments: [recipient=0x0000...0007, amount=1000]

### 5. fee_payer_transfer
**Source**: Derived from `aptos-core/types/src/transaction/authenticator.rs::tests::verify_fee_payer_with_optional_fee_payer_address`

- **Type**: Entry function call with fee payer
- **Authentication**: Fee payer (sender + secondary signers + separate fee payer)
- **Size**: 161 bytes BCS (RawTransaction only)
- **Purpose**: Sponsor payments - application pays gas for users, important for user onboarding

**Transaction Details**:
- Sender: 0x000000000000000000000000000000000000000000000000000000000000000a
- Fee Payer: 0x000000000000000000000000000000000000000000000000000000000000000c
- Sequence: 0
- Gas: 5000 max, 1 per unit
- Expiration: 9999999999
- Function: Transfer (0x1::aptos_coin::transfer)
- Arguments: [recipient=0x000000...0b, amount=1000]

## BCS Structure Reference

All fixtures follow the Aptos BCS encoding for RawTransaction:

```
RawTransaction {
  sender: AccountAddress (32 bytes),
  sequence_number: u64 (8 bytes),
  payload: TransactionPayload (variable, type discriminant + data),
  max_gas_amount: u64 (8 bytes),
  gas_unit_price: u64 (8 bytes),
  expiration_timestamp_secs: u64 (8 bytes),
  chain_id: ChainId (1 byte)
}

TransactionPayload variants:
  0 = Script(...)
  1 = EntryFunction(...)
  2 = ModuleBundle (deprecated)
  3 = Multisig(...)

EntryFunction {
  module: ModuleId (address + name),
  function: Identifier (string),
  ty_args: Vec<TypeTag>,
  args: Vec<Vec<u8>>
}
```

## Verification

These fixtures were extracted from the official Aptos Core test suite:
- **Repository**: https://github.com/aptos-labs/aptos-core
- **Test File**: `types/src/transaction/authenticator.rs`
- **Test Module**: `mod tests`

All test functions include signature verification, confirming the BCS encoding is valid.

## Usage

For testing an Aptos transaction decoder:

1. Read the `.bcs.hex` file and convert hex string to bytes
2. Deserialize using BCS decoder for `RawTransaction` struct
3. Validate fields match the expected values in the `.json` file
4. For full SignedTransaction testing, add authenticator data and serialize again
5. Compute transaction hash using SHA3-256

## Chain IDs

- 1 = Mainnet
- 2 = Testnet
- 3 = Devnet
- 4 = Local
- 89 = Custom (used in webauthn fixture from core tests)

## Additional Resources

- BCS Format Specification: https://github.com/zefchain/bcs
- Aptos Move Documentation: https://aptos.dev/en/build/smart-contracts
- Aptos Transaction Types: https://github.com/aptos-labs/aptos-core/blob/main/types/src/transaction/mod.rs

## Notes

- All fixtures use simple, recognizable addresses (0x0000...0001, etc.) for clarity
- None of these transactions are actual signed transactions from the blockchain
- These are test vectors for decoder validation only
- Real transactions on mainnet will have different addresses, signatures, and amounts
- RawTransaction is the pre-signature form; SignedTransaction adds the authenticator field

---

**Generated**: November 18, 2025
**Format Version**: 1.0
**Total Fixtures**: 5
**Coverage**: Single-key (ED25519, secp256k1), WebAuthn, Multi-agent, Fee payer
