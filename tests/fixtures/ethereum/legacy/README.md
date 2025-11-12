# Ethereum Legacy Transactions

Pre-EIP-1559 transactions (before London hard fork).

## Characteristics

- RLP-encoded
- Gas price (single value)
- No transaction type prefix
- Fields: [nonce, gasPrice, gasLimit, to, value, data, v, r, s]

## Test Coverage

Fixtures in this directory test:
- Simple ETH transfers
- ERC-20 token transfers
- Contract deployments
- Contract interactions
- Large data payloads

## Sources

- Pre-London fork mainnet transactions (before block 12,965,000)
- Alloy test vectors
- Etherscan API
