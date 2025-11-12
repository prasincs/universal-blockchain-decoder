# Solana Simple Transactions

Basic SOL transfer transactions.

## Characteristics

- Bincode-encoded
- Compact-u16 length encoding
- Ed25519 signatures (64 bytes)
- Message structure: [header, accounts, recent_blockhash, instructions]
- Minimal instruction count (1-2)

## Test Coverage

Fixtures in this directory test:
- Simple SOL transfers
- Account creation
- System program interactions
- Minimal transaction size
- Single signature transactions

## Sources

- Solana validator test vectors
- Mainnet-beta simple transfers
- Solscan API
