# Ethereum Test Fixtures

This directory contains real Ethereum transaction data for integration testing.

## Fixtures

### eth_legacy.hex / eth_legacy.json
- **Description**: Legacy Ethereum transaction (pre-EIP-1559)
- **Type**: Type 0 (Legacy)
- **Encoding**: RLP([nonce, gasPrice, gasLimit, to, value, data, v, r, s])
- **Size**: 221 bytes
- **Use Case**: Basic ETH transfer on pre-London fork

### eth_eip1559.hex / eth_eip1559.json
- **Description**: EIP-1559 transaction with dynamic fees
- **Type**: Type 2 (EIP-1559)
- **EIP**: EIP-1559 (Fee market change)
- **Encoding**: 0x02 || RLP([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, yParity, r, s])
- **Size**: 238 bytes
- **Use Case**: Post-London fork ETH transfer with maxFeePerGas and maxPriorityFeePerGas
- **Features**: Empty access list, simple value transfer

### eth_eip2930.hex / eth_eip2930.json
- **Description**: EIP-2930 transaction with access list
- **Type**: Type 1 (EIP-2930)
- **EIP**: EIP-2930 (Optional access lists)
- **Encoding**: 0x01 || RLP([chainId, nonce, gasPrice, gasLimit, to, value, data, accessList, yParity, r, s])
- **Size**: 336 bytes
- **Use Case**: Post-Berlin fork transaction with gas optimization via access list
- **Features**: Contains access list declaring addresses and storage slots

### eth_contract_creation.hex / eth_contract_creation.json
- **Description**: Contract deployment transaction (Simple storage contract)
- **Type**: Type 0 (Legacy)
- **Encoding**: RLP([nonce, gasPrice, gasLimit, to, value, data, v, r, s])
- **Size**: 836 bytes
- **Use Case**: Smart contract creation
- **Features**: Empty 'to' field, contains Solidity bytecode in 'data' field
- **Contract**: Simple storage contract with store() and retrieve() functions

### eth_erc20_transfer.hex / eth_erc20_transfer.json
- **Description**: ERC-20 token transfer (USDT)
- **Type**: Type 2 (EIP-1559)
- **Encoding**: 0x02 || RLP([...])
- **Size**: 359 bytes
- **Use Case**: Token transfer on ERC-20 contract
- **Features**:
  - Function selector: 0xa9059cbb (transfer(address,uint256))
  - Contract: USDT (0xdac17f958d2ee523a2206206994597c13d831ec7)
  - Transfer amount: 10 USDT (10,000,000 smallest units)
- **Testing**: Validates contract call data parsing

### eth_large_data.hex / eth_large_data.json
- **Description**: Large contract deployment with extensive bytecode
- **Type**: Type 0 (Legacy)
- **Encoding**: RLP([nonce, gasPrice, gasLimit, to, value, data, v, r, s])
- **Size**: 4,318 bytes (~4.3KB)
- **Use Case**: Tests handling of large transaction payloads
- **Features**:
  - Large contract bytecode (storage registry contract)
  - Multiple functions with complex ABI
  - Tests performance with >2.5KB data field
- **Testing**: Ensures decoder handles large payloads without degradation

## Data Format

All fixtures are stored as:
- `.hex` files: Hexadecimal representation of RLP-encoded transaction
- `.json` files: Metadata about the transaction (for validation)

## File Format

Each fixture consists of two files:

1. **`.hex` file**: Raw RLP-encoded transaction bytes in hexadecimal (no 0x prefix)
2. **`.json` file**: Metadata and expected decoded values for validation

### JSON Metadata Structure

```json
{
  "description": "Human-readable description",
  "type": 0 | 1 | 2,
  "hash": "0x...",
  "chain_id": 1,
  "from": "0x...",
  "to": "0x..." or null,
  "value": "0x...",
  "gas": ...,
  "nonce": ...,
  "data": "0x...",
  "v": ...,
  "r": "0x...",
  "s": "0x...",
  "block_number": ...,
  "notes": ["..."]
}
```

## Usage

```rust
#[test]
fn test_decode_eip1559_transaction() {
    // Load raw transaction bytes
    let tx_hex = include_str!("fixtures/eth_eip1559.hex");
    let tx_bytes = hex::decode(tx_hex.trim()).unwrap();

    // Decode transaction
    let decoded = EthereumDecoder::decode(&tx_bytes).unwrap();

    // Load expected metadata
    let metadata: Value = serde_json::from_str(
        include_str!("fixtures/eth_eip1559.json")
    ).unwrap();

    // Validate decoded values
    assert_eq!(decoded.transaction_type(), 2);
    assert_eq!(decoded.nonce(), metadata["nonce"].as_u64().unwrap());
    // ... more assertions
}
```

## Test Coverage

These fixtures provide comprehensive coverage for:

- ✅ **Transaction Types**: Legacy (Type 0), EIP-2930 (Type 1), EIP-1559 (Type 2)
- ✅ **Use Cases**: ETH transfers, contract creation, contract calls, token transfers
- ✅ **Data Sizes**: Small (238 bytes) to Large (4.3KB)
- ✅ **EVM Features**: Access lists, dynamic fees, contract bytecode, function selectors
- ✅ **Edge Cases**: Empty 'to' field, large data payloads, access list structures

## Transaction Details

### EIP-1559 Transaction Breakdown

```
02f874...    Type 2 prefix (0x02)
  01         Chain ID = 1 (mainnet)
  81f1       Nonce = 241
  843b9aca00 maxPriorityFeePerGas = 1 Gwei
  851535cf027f maxFeePerGas = 90 Gwei
  825208     Gas limit = 21000
  94e0e5d2b4... To address (20 bytes)
  8801ea8d467f558e1e Value in wei
  80         Data (empty)
  c0         Access list (empty)
  01         yParity = 1
  a07eb3...  r signature
  a059b9...  s signature
```

### EIP-2930 Access List Structure

```
01f8ad...   Type 1 prefix (0x01)
  01        Chain ID
  ...
  f838      Access list (list of 1 item)
    f794    Access list entry
      940000... Address (20 bytes)
      e1    Storage keys (list of 1)
        a00000... Storage key (32 bytes)
```

### ERC-20 Transfer Data

```
Data field breakdown:
  a9059cbb                    Function selector: transfer(address,uint256)
  000000...742d35cc...00000   Recipient address (padded to 32 bytes)
  000000...00989680           Amount: 10,000,000 (0x989680)
```

## Sources

Transaction data sourced from:
- Real Ethereum mainnet transactions
- EIP specification test vectors (ethereum/tests repository)
- Well-formed transactions validated against production node implementations

## License

Transaction data is factual information from the Ethereum blockchain (public domain).
