# Bitcoin SegWit Transactions

SegWit (Segregated Witness) transactions (BIP 141, 143, 144).

## Characteristics

- Witness data separated from transaction body
- Uses native SegWit addresses (bc1...)
- Transaction marker/flag: 0x00 0x01
- Witness version 0 (P2WPKH, P2WSH)

## Test Coverage

Fixtures in this directory test:
- P2WPKH (native SegWit) transactions
- P2WSH (native SegWit multisig) transactions
- Nested SegWit (P2SH-P2WPKH)
- Mixed inputs (legacy + SegWit)
- TXID vs WTXID calculation

## Sources

- SegWit activation block (481,824) transactions
- Bitcoin Core SegWit test vectors
- Post-2017 mainnet transactions
