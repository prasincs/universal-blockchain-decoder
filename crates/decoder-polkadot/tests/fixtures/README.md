# Polkadot Test Fixtures

## Sources

1. **Polkadot SDK Test Vectors**
   - Repository: https://github.com/paritytech/polkadot-sdk
   - Version: polkadot-v1.7.0
   - Location: `substrate/frame/*/src/tests.rs`

2. **Real Mainnet Extrinsics**
   - Explorer: https://polkadot.subscan.io
   - Polkadot.js examples

## Extrinsic Types

- Signed vs Unsigned
- Balance transfers
- Staking operations
- Governance votes
- XCM (cross-chain) messages

## Format

All fixtures are stored as:
- `.scale` - SCALE-encoded extrinsic (hex or binary)
- `.json` - Expected decoded output with metadata

## License

Polkadot SDK is licensed under Apache 2.0 or GPL-3.0
