# Algorand Test Fixtures

## Sources

1. **go-algorand Test Data**
   - Repository: https://github.com/algorand/go-algorand
   - Location: `data/transactions/logic/testdata/`

2. **Real Mainnet Transactions**
   - Explorer: https://algoexplorer.io
   - SDK examples

## Transaction Types

- Payment transactions
- Asset transfers (ASA)
- Application calls (smart contracts)
- Key registration
- Asset configuration

## Format

All fixtures are stored as:
- `.msgpack` - MessagePack-encoded transaction
- `.json` - Expected decoded output with metadata

## License

Algorand is licensed under AGPL-3.0
