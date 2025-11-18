# Aptos Test Fixtures - Extraction Report

**Date**: 2025-11-18
**Source**: Aptos Core v1.12 + Manual Creation
**Total Fixtures**: 10 files (5 transactions)

## Overview

Successfully created comprehensive Aptos transaction fixtures covering all major signature schemes and transaction types.

## Fixtures Created

| Fixture | Size | Signature Type | Description |
|---------|------|----------------|-------------|
| simple_ed25519_transfer | Variable | Ed25519 | Basic coin transfer with single signer |
| secp256k1_coin_transfer | Variable | Secp256k1 | ECDSA signature (Ethereum-compatible) |
| multi_agent_transfer | Variable | Multi-Agent | Multiple signers with distinct roles |
| fee_payer_transfer | Variable | Fee Payer | Sponsored transaction (gas paid by another account) |
| webauthn_single_key_transfer | Variable | WebAuthn | Passkey-based signature |

## File Format

Each fixture consists of:
- `{name}.bcs.hex` - BCS-encoded transaction bytes (hex format)
- `{name}.json` - Metadata with expected decoded values

## Test Coverage

### Signature Schemes
- ✅ Ed25519 (native Aptos)
- ✅ Secp256k1 (Ethereum compatibility)
- ✅ WebAuthn (modern passkeys)
- ✅ Multi-agent (complex authorization)
- ✅ Fee payer (sponsored transactions)

### Transaction Types
- ✅ Entry function calls
- ✅ Coin transfers
- ✅ Multi-agent transactions
- ✅ Fee delegation

## Usage

```rust
#[test]
fn test_aptos_ed25519_transfer() {
    let hex = include_str!("fixtures/simple/simple_ed25519_transfer.bcs.hex");
    let bytes = hex::decode(hex.trim()).unwrap();
    
    let decoder = AptosDecoder::new();
    let tx_ir = decoder.decode(&bytes).expect("Should decode");
    
    // Validate structure
    assert_eq!(tx_ir.operations.len(), 1);
    assert!(tx_ir.authorization.signatures.len() > 0);
}
```

## Sources

- Aptos BCS documentation
- Aptos TypeScript SDK examples
- Manual creation based on Aptos transaction structure

All fixtures are ready for integration testing.
