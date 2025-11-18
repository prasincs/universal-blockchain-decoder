# Solana Test Fixtures

## Sources

1. **Solana SDK Test Vectors**
   - Repository: https://github.com/solana-labs/solana
   - Version: v1.18.0
   - Location: `sdk/src/transaction/tests`

2. **Real Mainnet Transactions**
   - Explorer: https://solscan.io
   - Example transactions curated for testing

## Fixture Types

- `simple/` - Basic SOL transfers
- `complex/` - Multi-instruction transactions, program interactions
- `edge_cases/` - Maximum size, unusual formats
- `invalid/` - Malformed transactions (should fail to decode)

## Format

All fixtures are stored as:
- `.base64` - Base64-encoded transaction (Solana native format)
- `.json` - Expected decoded output with metadata

## License

Solana is licensed under Apache 2.0
Test vectors derived from official Solana repository
