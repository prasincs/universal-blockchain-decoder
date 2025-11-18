# Polkadot/Substrate Test Fixtures Summary

## Overview

This document summarizes the Polkadot extrinsic test fixtures created from the Polkadot SDK source code.

## Creation Date
- **Generated**: 2025-11-18
- **Source Repository**: https://github.com/paritytech/polkadot-sdk
- **Reference Documentation**: `docs/sdk/src/reference_docs/extrinsic_encoding.rs`

## Fixture Files Created (6 total)

### `/simple/` Directory

All fixtures are created in the `simple/` subdirectory as basic, single-purpose test vectors.

#### 1. **unsigned_remark_v4**
- **Files**:
  - `unsigned_remark_v4.scale.hex` (10 bytes)
  - `unsigned_remark_v4.json`
- **Type**: Unsigned extrinsic (v4 format)
- **Call**: `System::remark()` with empty data
- **Description**: Minimal unsigned extrinsic demonstrating basic structure
- **SCALE Structure**:
  - Compact length: 04 (4 bytes)
  - Version: 04 (unsigned, v4)
  - Pallet index: 00 (System)
  - Call index: 00 (remark)
  - Data length: 00 (empty)
  - Data: (empty)

#### 2. **unsigned_system_remark**
- **Files**:
  - `unsigned_system_remark.scale.hex` (78 bytes)
  - `unsigned_system_remark.json`
- **Type**: Unsigned extrinsic (v4 format)
- **Call**: `System::remark()` with 32 bytes of data
- **Description**: Unsigned remark with actual payload data
- **SCALE Structure**:
  - Compact length: 0c (12 bytes)
  - Version: 04 (unsigned, v4)
  - Pallet index: 00 (System)
  - Call index: 00 (remark)
  - Data length: 20 (32 bytes, compact encoded)
  - Data: 32 bytes of zeros

#### 3. **signed_transfer_v4**
- **Files**:
  - `signed_transfer_v4.scale.hex` (218 bytes)
  - `signed_transfer_v4.json`
- **Type**: Signed extrinsic (v4 format)
- **Call**: `Balances::transfer()`
- **Description**: Basic signed balance transfer transaction
- **SCALE Structure**:
  - Compact length: a5 01 (165 bytes)
  - Version byte: 84 (signed, v4)
  - From address: 32-byte AccountId32
  - Signature: 64-byte Sr25519 signature
  - Era: 01 (Immortal - never expires)
  - Nonce: 00 (first transaction)
  - Tip: 00 (no priority fee)
  - Pallet index: 04 (Balances)
  - Call index: 00 (transfer)
  - To: 32-byte AccountId32 (0x000...001)
  - Amount: 100000000000 (100 DOT, compact encoded)

#### 4. **signed_transfer_mortal_era**
- **Files**:
  - `signed_transfer_mortal_era.scale.hex` (228 bytes)
  - `signed_transfer_mortal_era.json`
- **Type**: Signed extrinsic with mortal era
- **Call**: `Balances::transfer()`
- **Description**: Signed transfer with transaction mortality (expires after N blocks)
- **Key Differences**:
  - Era: Mortal with period=64, phase=16 (2 bytes)
  - Nonce: 1 (second transaction)
  - Tip: 100000000 (priority fee)
- **SCALE Structure**:
  - Compact length: c5 05 (197 bytes)
  - Version byte: 84 (signed, v4)
  - From address: 32-byte AccountId32
  - Signature: 64-byte Sr25519
  - Era: b0 01 (mortal, 2 bytes)
  - Nonce: 04 (1, compact encoded)
  - Tip: 00 10 a5 d4 01 (100000000, variable length)
  - Rest: Balances::transfer call

#### 5. **signed_transfer_keep_alive**
- **Files**:
  - `signed_transfer_keep_alive.scale.hex` (218 bytes)
  - `signed_transfer_keep_alive.json`
- **Type**: Signed extrinsic
- **Call**: `Balances::transfer_keep_alive()` (variant index 3)
- **Description**: Balance transfer that fails if recipient would be below existential deposit
- **Difference from transfer**:
  - Call index: 03 (instead of 00)
  - Ensures minimum balance requirements are met

#### 6. **signed_with_nonce_tip**
- **Files**:
  - `signed_with_nonce_tip.scale.hex` (224 bytes)
  - `signed_with_nonce_tip.json`
- **Type**: Signed extrinsic with custom nonce and tip
- **Call**: `Balances::transfer()`
- **Description**: Demonstrates nonce sequencing and priority fee mechanism
- **Parameters**:
  - Nonce: 5 (6th transaction from sender)
  - Tip: 256 (priority fee in smallest units)
  - Era: Immortal

## SCALE Encoding Details

### Extrinsic Format (V4)

```
extrinsic = compact_length + version_byte + signed_data + call_data

// For unsigned:
version_byte = 0x04

// For signed:
version_byte = 0x84 (0x80 | 0x04)
signed_data = from_address + signature + signed_extensions_extra
signed_extensions_extra = era + nonce + tip
```

### Era Encoding

- **Immortal**: Single byte `0x00`
- **Mortal**: Two bytes representing period and phase
  - Period must be power of 2 (16-1 << 12)
  - Phase indicates the starting block within the period

### Compact Integer Encoding

- Values 0-63: single byte
- Values 64-16383: two bytes
- Values 16384+: variable length

## Sources and References

### Primary Source
- **Repository**: https://github.com/paritytech/polkadot-sdk
- **Documentation**: `docs/sdk/src/reference_docs/extrinsic_encoding.rs`
- **Test Files**: `substrate/frame/balances/src/tests/` and `substrate/frame/system/src/tests.rs`

### Key Files Referenced
1. `/docs/sdk/src/reference_docs/extrinsic_encoding.rs` - Detailed extrinsic encoding specification
2. `/substrate/frame/balances/src/tests/dispatchable_tests.rs` - Balance transfer tests
3. `/substrate/frame/system/src/tests.rs` - System pallet tests

### Encoding Specification
The fixtures are created according to the official Substrate extrinsic encoding specification:
- Version format: 4
- Canonical format: SCALE (Substrate Compact All-purpose Encoding)
- Hash: Blake2b-256 (for transaction IDs)

## Fixture Characteristics

### Coverage

| Aspect | Covered |
|--------|---------|
| Unsigned extrinsics | ✓ (2 fixtures) |
| Signed extrinsics | ✓ (4 fixtures) |
| Balance transfers | ✓ (4 fixtures) |
| System calls | ✓ (2 fixtures) |
| Immortal era | ✓ (3 fixtures) |
| Mortal era | ✓ (1 fixture) |
| Nonce sequencing | ✓ (2 fixtures) |
| Priority tips | ✓ (2 fixtures) |
| Different call variants | ✓ (transfer vs transfer_keep_alive) |

### Test Vector Characteristics

- **Total fixtures**: 6
- **Hex files**: 6 × `.scale.hex` (10-228 bytes each)
- **JSON files**: 6 × `.json` (structured metadata)
- **Coverage**: Basic unsigned, basic signed, variants, era options, nonce tracking

## Usage in Tests

These fixtures are designed for:

1. **Extrinsic Parsing Tests**: Verify the decoder can parse SCALE bytes
2. **Structure Validation**: Check proper extraction of signature, nonce, era, etc.
3. **Call Decoding**: Ensure correct pallet and call identification
4. **Roundtrip Testing**: Verify encoded → decoded → encoded consistency
5. **Property-Based Testing**: Use as seeds for fuzzing campaigns

## Format Specification

### `.scale.hex` File
- Contains hex-encoded SCALE bytes
- No whitespace (except newlines for large files)
- Exactly represents the extrinsic bytes that would be transmitted on-chain

### `.json` File
- Structured metadata about the extrinsic
- Includes:
  - Descriptive name and purpose
  - Extrinsic type (signed/unsigned)
  - Address and signature info (for signed)
  - Era, nonce, and tip details
  - Call structure (pallet, function, parameters)
  - Source and reference information
  - Use case notes

## Future Enhancements

Potential test fixtures to add:

1. **Edge Cases**:
   - Maximum compact-encoded values
   - Empty vs non-empty payloads
   - Minimum required sizes

2. **Chain-Specific Examples**:
   - Kusama extrinsics
   - Polkadot mainnet real transactions (anonymized)
   - Parachain-specific calls

3. **Complex Calls**:
   - Utility batch/batch_all calls
   - Staking operations
   - Governance votes
   - XCM messages

4. **Real Mainnet Vectors**:
   - Pulled from Polkascan/Subscan
   - Anonymized account addresses
   - Validated against live decoder

## Notes

- All accounts use zero or minimal values for simplicity
- Signatures are zero-filled (not cryptographically valid)
- Suitable for parser/decoder testing but not for transaction validation
- Real transaction testing should use fixtures from `polkadot-sdk/` test suites

## License

These fixtures are derived from the Polkadot SDK which is licensed under:
- Apache License 2.0
- GPL v3.0

## Version History

- **2025-11-18**: Initial fixture creation (6 basic test vectors)
