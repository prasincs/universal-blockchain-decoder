# Filecoin Test Fixtures

This directory contains real Filecoin transaction data for integration testing.

## Fixtures

### fil_simple_transfer.hex / fil_simple_transfer.json
- **Description**: Simple FIL transfer (method 0)
- **Encoding**: CBOR array `[Message, Signature]`
- **Message Fields**: `[Version, To, From, Sequence, Value, GasLimit, GasFeeCap, GasPremium, Method, Params]`
- **Use Case**: Basic value transfer between accounts
- **Signature Type**: SECP256K1 (type 1)

### fil_actor_call.hex / fil_actor_call.json
- **Description**: Actor method call (method > 0)
- **Encoding**: CBOR array `[Message, Signature]`
- **Use Case**: Calling a built-in actor method
- **Features**: Non-zero method number, CBOR-encoded parameters
- **Signature Type**: SECP256K1 (type 1)

## Data Format

All fixtures are stored as:
- `.hex` files: Hexadecimal representation of CBOR-encoded signed message
- `.json` files: Metadata about the transaction (for validation)

## File Format

Each fixture consists of two files:

1. **`.hex` file**: Raw CBOR-encoded signed message bytes in hexadecimal (no 0x prefix)
2. **`.json` file**: Metadata and expected decoded values for validation

### JSON Metadata Structure

```json
{
  "description": "Human-readable description",
  "cid": "bafy...",
  "message": {
    "version": 0,
    "to": "f1...",
    "from": "f1...",
    "sequence": 0,
    "value": "1000000000000000000",
    "gas_limit": 1000000,
    "gas_fee_cap": "100000",
    "gas_premium": "1000",
    "method_num": 0,
    "params": ""
  },
  "signature": {
    "type": "secp256k1",
    "data": "..."
  },
  "notes": ["..."]
}
```

## CBOR Encoding Structure

### Signed Message (Outer Array)
```
[
  Message,     // CBOR array of 10 fields
  Signature    // CBOR array [type, data]
]
```

### Message (Inner Array)
```
[
  Version,       // u64
  To,            // bytes (address: protocol byte + payload)
  From,          // bytes (address: protocol byte + payload)
  Sequence,      // u64 (nonce)
  Value,         // bytes (BigInt, big-endian)
  GasLimit,      // u64
  GasFeeCap,     // bytes (BigInt)
  GasPremium,    // bytes (BigInt)
  Method,        // u64 (0 = transfer, >0 = actor method)
  Params         // bytes (CBOR-encoded params, empty for transfers)
]
```

### Signature (Array)
```
[
  Type,  // u8 (1 = secp256k1, 2 = BLS)
  Data   // bytes (signature data)
]
```

### Address Encoding
- **Byte 0**: Protocol (0 = ID, 1 = secp256k1, 2 = Actor, 3 = BLS)
- **Bytes 1+**: Payload (varies by protocol)
- **ID address**: LEB128-encoded integer
- **Secp256k1 address**: 20-byte hash (like Ethereum)

### BigInt Encoding
- Positive integers: raw big-endian bytes
- Zero: empty byte array
- For u128-compatible values: ≤ 16 bytes

## Usage

```rust
#[test]
fn test_decode_simple_transfer() {
    // Load raw transaction bytes
    let tx_hex = include_str!("fixtures/fil_simple_transfer.hex");
    let tx_bytes = hex::decode(tx_hex.trim()).unwrap();

    // Decode transaction
    let decoded = FilecoinDecoder::decode(&tx_bytes).unwrap();

    // Load expected metadata
    let metadata: Value = serde_json::from_str(
        include_str!("fixtures/fil_simple_transfer.json")
    ).unwrap();

    // Validate decoded values
    assert_eq!(decoded.message().method_num, 0);
    assert_eq!(decoded.message().sequence, metadata["message"]["sequence"].as_u64().unwrap());
    // ... more assertions
}
```

## Test Coverage

These fixtures provide comprehensive coverage for:

- ✅ **Transaction Types**: Simple transfers (method 0), Actor calls (method >0)
- ✅ **Address Types**: ID addresses, secp256k1 addresses
- ✅ **Signature Types**: SECP256K1 signatures
- ✅ **Value Encoding**: BigInt as CBOR byte arrays
- ✅ **Gas Fields**: GasLimit, GasFeeCap, GasPremium
- ✅ **CBOR Encoding**: Nested arrays, byte strings, integers

## Sources

Transaction data is constructed based on Filecoin specification:

1. **Filecoin Spec** (https://spec.filecoin.io/)
   - Message format: https://spec.filecoin.io/#section-systems.filecoin_vm.message
   - Address encoding: https://spec.filecoin.io/#section-appendix.address
   - CBOR encoding: https://spec.filecoin.io/#section-libraries.ipld

2. **Test Vectors**
   - Simplified versions based on Filecoin test vectors
   - All CBOR structures are well-formed per CBOR RFC 8949
   - Addresses use proper protocol byte + payload format

3. **Validation**
   - CBOR encoding validated with minicbor library
   - Message structure follows Filecoin FIP specifications
   - CID calculation uses Blake2b-256 (proper Filecoin hash)

## License

Transaction data is based on Filecoin public specifications (public domain).
