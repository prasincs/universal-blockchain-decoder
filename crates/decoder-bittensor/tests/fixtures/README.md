# Bittensor Transaction Test Fixtures

This directory contains real Bittensor transaction data for integration testing.

## Adding Fixtures

To add a new test fixture:

1. Obtain the raw SCALE-encoded extrinsic bytes from a Bittensor node or explorer
2. Save as a `.bin` file with a descriptive name
3. Create a corresponding `.json` file with metadata:

```json
{
  "description": "Balances transfer from Alice to Bob",
  "block_height": 1234567,
  "tx_hash": "0x...",
  "expected_pallet": "Balances",
  "expected_call": "transfer"
}
```

## Example

Get transaction data from Bittensor explorer:
- Mainnet: https://taostats.io
- Testnet: https://testnet.taostats.io

Or using `subxt` or similar Substrate RPC tools.

## Current Fixtures

(To be added)
