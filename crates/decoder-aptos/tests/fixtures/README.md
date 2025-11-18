# Aptos Test Fixtures

## Sources

1. **Aptos Core Test Vectors**
   - Repository: https://github.com/aptos-labs/aptos-core
   - Version: aptos-release-v1.12
   - Location: `testsuite/`, `aptos-move/aptos-vm/tests/`

2. **Real Mainnet Transactions**
   - Explorer: https://explorer.aptoslabs.com
   - TypeScript SDK test vectors

## Transaction Types

- Entry function calls
- Script transactions
- Multi-sig transactions
- Multi-agent transactions

## Format

All fixtures are stored as:
- `.bcs` - BCS-encoded transaction (binary)
- `.json` - Expected decoded output with metadata

## License

Aptos is licensed under Apache 2.0
