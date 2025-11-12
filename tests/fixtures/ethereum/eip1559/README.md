# Ethereum EIP-1559 Transactions

EIP-1559 transactions (London hard fork).

## Characteristics

- Transaction type: 0x02
- Two-tier fee structure: maxFeePerGas + maxPriorityFeePerGas
- Base fee (burned)
- RLP-encoded with type prefix
- Fields: [chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, v, r, s]

## Test Coverage

Fixtures in this directory test:
- Simple EIP-1559 transfers
- EIP-1559 with access lists
- Priority fee variations
- Base fee edge cases
- Fee market dynamics

## Sources

- Post-London fork transactions (after block 12,965,000)
- Alloy EIP-1559 test vectors
- High-gas transactions (NFT mints, DeFi)
