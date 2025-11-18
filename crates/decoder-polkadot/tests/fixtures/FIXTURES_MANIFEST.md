# Polkadot Test Fixtures Manifest

## Quick Reference

| Name | Type | Pallet | Call | Size (hex) | Era | Nonce | Tip |
|------|------|--------|------|-----------|-----|-------|-----|
| `unsigned_remark_v4` | Unsigned | System | remark | 10 B | N/A | N/A | N/A |
| `unsigned_system_remark` | Unsigned | System | remark | 78 B | N/A | N/A | N/A |
| `signed_transfer_v4` | Signed | Balances | transfer | 218 B | Immortal | 0 | 0 |
| `signed_transfer_mortal_era` | Signed | Balances | transfer | 228 B | Mortal(64,16) | 1 | 100M |
| `signed_transfer_keep_alive` | Signed | Balances | keep_alive | 218 B | Immortal | 0 | 0 |
| `signed_with_nonce_tip` | Signed | Balances | transfer | 224 B | Immortal | 5 | 256 |

## File Structure

Each fixture consists of two files:

### `.scale.hex`
- **Purpose**: SCALE-encoded extrinsic bytes
- **Format**: Hexadecimal (0-9, a-f), no whitespace
- **Usage**: Input to decoder under test

### `.json`
- **Purpose**: Expected output and metadata
- **Format**: Structured JSON with full details
- **Fields**:
  - `name`: Human-readable fixture name
  - `description`: What this tests
  - `extrinsic_type`: "signed" or "unsigned"
  - `version`: Format version (4)
  - `is_signed`: Boolean
  - `from_address`: (signed only) Sender details
  - `signature`: (signed only) Signature info
  - `signed_extensions`: Era, nonce, tip info
  - `call`: Pallet and function details
  - `metadata`: Source and reference info

## Fixture Details

### 1. unsigned_remark_v4

**SCALE Bytes**: `0414000000`

**Breakdown**:
```
04         - Compact length: 4 bytes
14         - Version: 0x04 (unsigned, v4)
00         - Pallet: 0x00 (System)
00         - Call: 0x00 (remark)
```

**Purpose**: Minimal unsigned extrinsic

**Use Cases**:
- Parser initialization tests
- Minimal size validation
- Empty payload handling

---

### 2. unsigned_system_remark

**SCALE Bytes**: `0c0400200000000000000000000000000000000000000000000000000000000000000000000000`

**Breakdown**:
```
0c         - Compact length: 12 bytes (rest of extrinsic)
04         - Version: 0x04 (unsigned, v4)
00         - Pallet: 0x00 (System)
00         - Call: 0x00 (remark)
20         - Data length: 32 bytes (compact encoded as 0x80 bit trick)
(32 x 00) - Data payload: 32 zero bytes
```

**Purpose**: Unsigned extrinsic with data payload

**Use Cases**:
- Data embedding tests
- Compact encoding validation
- Remark functionality tests

---

### 3. signed_transfer_v4

**SCALE Bytes** (hex): `a501847d0a7f1df9e...` (218 bytes total)

**Breakdown**:
```
a501       - Compact length: 165 bytes
84         - Version: 0x84 (signed, v4)
[32 B]     - From address (AccountId32)
[64 B]     - Sr25519 signature
01         - Era: 0x01 (Immortal)
00         - Nonce: 0 (compact)
00         - Tip: 0 (compact)
04         - Pallet: 0x04 (Balances)
00         - Call: 0x00 (transfer)
[dest]     - To address (32 bytes)
[amount]   - Transfer amount (compact)
```

**Purpose**: Basic signed balance transfer

**Use Cases**:
- Signature extraction tests
- Address parsing validation
- Basic transfer flow
- Immortal era handling

---

### 4. signed_transfer_mortal_era

**SCALE Bytes** (hex): `c505847d0a7f1df9e...` (228 bytes total)

**Key Differences**:
```
c505       - Compact length: 197 bytes (longer due to mortal era)
[...]      - Same structure as #3, but:
b001       - Era: Mortal (2 bytes) - period=64, phase=16
04         - Nonce: 1 (compact: 0x04 = 1)
00 10 a5 d4 01 - Tip: 100,000,000 (variable-length compact)
```

**Purpose**: Signed transfer with transaction mortality

**Use Cases**:
- Mortal era encoding/decoding
- Variable-length compact integers
- Nonce sequencing
- Priority fee (tip) handling
- Transaction expiry simulation

---

### 5. signed_transfer_keep_alive

**SCALE Bytes** (hex): `a501847d0a7f1df9e...` (218 bytes total)

**Key Differences**:
```
[Same as #3 except:]
03         - Call: 0x03 (transfer_keep_alive variant)
```

**Purpose**: Alternative transfer variant

**Use Cases**:
- Call variant handling
- Existential deposit validation
- Account preservation tests
- Multiple call dispatch

---

### 6. signed_with_nonce_tip

**SCALE Bytes** (hex): `a501847d0a7f1df9e...` (224 bytes total)

**Key Differences**:
```
a501       - Compact length: 165 bytes
[...]      - Same structure as #3, but:
05         - Nonce: 5 (compact: 0x05 = 5)
0100       - Tip: 256 (compact encoding)
```

**Purpose**: Transfer with nonce sequencing and priority fee

**Use Cases**:
- Nonce tracking/validation
- Sequential transaction ordering
- Priority fee mechanisms
- Block inclusion ordering

---

## SCALE Encoding Reference

### Compact Integer Encoding

| Value Range | Encoding | Example |
|------------|----------|---------|
| 0-63 | Single byte | `0x05` → 5 |
| 64-16383 | Two bytes | `0x00 0x01` → 64 |
| 16384+ | Variable length | `0x00 0x00 0x01 0x00` → 16384 |

### Era Encoding

**Immortal**:
```
00 - Single byte, transaction never expires
```

**Mortal**:
```
[2 bytes]: period and phase
- Byte 1 (lower): (period - 4) >> 12, encoded with phase bits
- Byte 2 (upper): phase as u8
- Period must be power of 2: 4, 8, 16, 32, 64, 128, ..., 65536
- Phase: 0 to period-1, indicates first valid block
```

### Version Byte

```
Bit 7    | Bits 6-0
---------|----------
0        | 0000100  → 0x04 (unsigned, version 4)
1        | 0000100  → 0x84 (signed, version 4)
```

---

## Integration with Decoder

These fixtures are designed to be used in tests like:

```rust
#[test]
fn test_decode_unsigned_remark() {
    let hex = "0414000000";
    let bytes = hex::decode(hex).unwrap();
    let result = PolkadotDecoder::decode(&bytes).unwrap();
    
    assert!(!result.extrinsic.is_signed());
    assert_eq!(result.call().pallet_index, 0); // System
    assert_eq!(result.call().call_index, 0);   // remark
}

#[test]
fn test_decode_signed_transfer() {
    let json_fixture: FixtureData = serde_json::from_str(
        include_str!("fixtures/simple/signed_transfer_v4.json")
    ).unwrap();
    
    let hex = std::fs::read_to_string(
        "fixtures/simple/signed_transfer_v4.scale.hex"
    ).unwrap();
    
    let bytes = hex::decode(hex.trim()).unwrap();
    let result = PolkadotDecoder::decode(&bytes).unwrap();
    
    // Validate against JSON expectations
    assert!(result.extrinsic.is_signed());
    assert_eq!(result.call().pallet_index, json_fixture.call.pallet_index);
    // ... more assertions
}
```

---

## Testing Strategies

### Unit Tests
- **Single fixture per test**: Each test validates one specific aspect
- **Minimal assertions**: Check only what's being tested
- **Named fixtures**: Use fixture name in test name

### Property-Based Tests
- **Use as seeds**: Generate variations from base fixtures
- **Fuzzing**: Mutate valid bytes to find edge cases
- **Roundtrip**: encode(decode(fixture)) == fixture

### Integration Tests
- **End-to-end**: Load fixture → decode → validate all fields
- **Cross-validation**: Compare with reference implementations
- **Performance**: Measure decode times across fixtures

### Documentation Tests
- **Example code**: Use fixtures in documentation examples
- **Reference**: Show expected decode output
- **Learning**: Help users understand format

---

## Version History

| Date | Version | Changes |
|------|---------|---------|
| 2025-11-18 | 1.0 | Initial fixture set (6 vectors) |

## License

These fixtures are derived from Polkadot SDK, licensed under:
- Apache License 2.0
- GPL v3.0

Use accordingly.
