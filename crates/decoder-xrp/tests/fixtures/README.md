# XRP Test Fixtures

This directory contains real XRP transaction data for integration testing.

## Structure

- `payment_xrp.hex` - Simple XRP payment transaction
- `payment_iou.hex` - Payment with issued currency (token)
- `trustset.hex` - Trust line creation for token
- `offer_create.hex` - DEX order creation

## Data Sources

Real transaction data can be obtained from:
- XRP Ledger Explorer: https://livenet.xrpl.org/
- XRPL API: https://xrpl.org/public-servers.html
- xrpl.js library test fixtures

## Format

All fixtures are in hex-encoded binary format (ripple-binary-codec).

## Adding New Fixtures

1. Find a real transaction on XRP Ledger
2. Get the binary blob (not JSON)
3. Hex encode it
4. Add to this directory with descriptive name
5. Document in this README

## Example Transactions

### Payment (XRP)
```
Account: rN7n7otQDd6FczFgLdlqtyMVrn3HMfXtYF
Destination: rLHzPsX6oXkzU9rXvQBZBJuPQhSGz6SqxL
Amount: 1000000 drops (1 XRP)
Fee: 10 drops
```

### TrustSet (Token)
```
Account: rN7n7otQDd6FczFgLdlqtyMVrn3HMfXtYF
LimitAmount: 1000000 USD.rExample...
```

## TODO

- [ ] Add real Payment transaction hex
- [ ] Add real TrustSet transaction hex
- [ ] Add real OfferCreate transaction hex
- [ ] Add NFT transaction examples
