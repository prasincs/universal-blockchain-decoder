# Bitcoin Test Fixtures

This directory contains real Bitcoin transaction data for integration testing.

## Fixtures

### btc_genesis_coinbase.hex
- **Description**: Bitcoin genesis block coinbase transaction
- **Block**: 0 (Genesis Block)
- **TXID**: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
- **Details**: First Bitcoin transaction, 50 BTC reward to Satoshi

### btc_simple_p2pkh.hex
- **Description**: Simple Pay-to-PubKey-Hash transaction
- **Block**: Example from early Bitcoin blockchain
- **Type**: Legacy transaction (pre-SegWit)

### btc_segwit.hex
- **Description**: SegWit (P2WPKH) transaction
- **Type**: Native SegWit transaction with witness data
- **BIP**: BIP 141, BIP 143, BIP 144

### btc_taproot.hex
- **Description**: Taproot (P2TR) transaction
- **Type**: Taproot transaction (post-activation block 709,632)
- **BIP**: BIP 340, BIP 341, BIP 342

### btc_multisig.hex
- **Description**: Multi-signature P2SH transaction
- **Type**: 2-of-3 multisig wrapped in P2SH

## Data Format

All fixtures are stored as:
- `.hex` files: Hexadecimal representation of raw transaction bytes
- `.json` files: Metadata about the transaction (for validation)

## Usage

```rust
#[test]
fn test_decode_genesis_coinbase() {
    let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
    let tx_bytes = hex::decode(tx_hex.trim()).unwrap();
    let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();
    // ... assertions
}
```

## Sources

Transaction data sourced from:
- Bitcoin Core (bitcoin-cli getrawtransaction)
- Blockchain explorers (blockchain.com, blockchair.com)
- Bitcoin test vectors from BIPs

## License

Transaction data is factual information from the Bitcoin blockchain (public domain).
