# Polkadot Test Fixtures

This directory contains real Polkadot mainnet transaction data for integration testing.

## Fixture Format

Each fixture consists of two files:
- `.hex` - Raw SCALE-encoded extrinsic bytes (hex string)
- `.json` - Expected values for validation

## Sources

All transactions are from Polkadot mainnet and can be verified on:
- Polkadot.js Apps: https://polkadot.js.org/apps/
- Subscan: https://polkadot.subscan.io/

## Fixture List

### Basic Transfer
- `transfer_simple.hex` - Simple DOT transfer (Balances::transfer)
- Block: Example from early mainnet
- Type: Signed extrinsic with Sr25519 signature

### Staking Operations
- `stake_nominate.hex` - Nominate validators (Staking::nominate)
- Type: Signed extrinsic

### Governance
- `democracy_vote.hex` - Democracy vote
- Type: Signed extrinsic

## How to Add New Fixtures

1. Find a transaction on Subscan or Polkadot.js
2. Extract the raw extrinsic bytes (SCALE encoded)
3. Save as `<name>.hex` (hex string, no 0x prefix)
4. Create `<name>.json` with expected values:
   ```json
   {
     "block_number": 12345,
     "extrinsic_index": 0,
     "is_signed": true,
     "pallet": "Balances",
     "call": "transfer",
     "sender": "1FRMM8PEiWXYax7rpS6X4XZX1aAAxSWx1CrKTyrVYhV24fg",
     "signature_type": "Sr25519"
   }
   ```

## Notes

- All fixtures use real mainnet data
- Signatures are included but not verified (signature verification requires runtime metadata)
- Focus is on SCALE decoding correctness, not signature validation
