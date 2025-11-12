# Bitcoin Legacy Transactions

Pre-SegWit transactions (before BIP 141).

## Characteristics

- No witness data
- Uses P2PKH, P2SH addresses
- Version typically 1 or 2
- Standard transaction structure: [version][inputs][outputs][locktime]

## Test Coverage

Fixtures in this directory test:
- Simple P2PKH transfers
- P2SH multisig transactions
- Transaction with multiple inputs/outputs
- RBF (Replace-By-Fee) transactions
- CLTV/CSV timelocks

## Sources

- Bitcoin Core test vectors: `tx_valid.json`
- Historical mainnet transactions (pre-2017)
- Block explorers (mempool.space, blockchain.com)
