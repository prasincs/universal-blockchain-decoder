# Ethereum Test Fixtures

This directory contains real Ethereum transaction data for integration testing.

## Fixtures

### eth_legacy.hex
- **Description**: Legacy Ethereum transaction (pre-EIP-1559)
- **Type**: Type 0 (Legacy)
- **Encoding**: RLP

### eth_eip2930.hex
- **Description**: EIP-2930 transaction with access list
- **Type**: Type 1 (EIP-2930)
- **EIP**: EIP-2930 (Optional access lists)

### eth_eip1559.hex
- **Description**: EIP-1559 transaction with base fee
- **Type**: Type 2 (EIP-1559)
- **EIP**: EIP-1559 (Fee market change)

### eth_contract_creation.hex
- **Description**: Contract deployment transaction
- **Type**: Legacy transaction with no `to` address

### eth_erc20_transfer.hex
- **Description**: ERC-20 token transfer
- **Type**: Contract call with token transfer data

## Data Format

All fixtures are stored as:
- `.hex` files: Hexadecimal representation of RLP-encoded transaction
- `.json` files: Metadata about the transaction (for validation)

## Usage

```rust
#[test]
fn test_decode_legacy_transaction() {
    let tx_hex = include_str!("fixtures/eth_legacy.hex");
    let tx_bytes = hex::decode(tx_hex.trim()).unwrap();
    let decoded = EthereumDecoder::decode(&tx_bytes).unwrap();
    // ... assertions
}
```

## Sources

Transaction data sourced from:
- Ethereum nodes (eth_getTransactionByHash)
- Etherscan.io
- EIP specification test vectors

## License

Transaction data is factual information from the Ethereum blockchain (public domain).
