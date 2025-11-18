# Cardano Test Fixtures

## Sources

1. **Cardano Node Test Data**
   - Repository: https://github.com/input-output-hk/cardano-node
   - Location: `cardano-api/test/`, `cardano-ledger/eras/*/test-suite/`

2. **Real Mainnet Transactions**
   - Explorer: https://cardanoscan.io
   - Multiple eras: Shelley, Alonzo (Plutus), Babbage

## Transaction Types

- Simple ADA transfers
- Multi-asset transactions
- Plutus smart contracts
- Stake pool operations
- Governance actions

## Format

All fixtures are stored as:
- `.cbor` - CBOR-encoded transaction
- `.json` - Expected decoded output with metadata

## License

Cardano is licensed under Apache 2.0
