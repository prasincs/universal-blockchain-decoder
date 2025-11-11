# Architecture Refactoring: Shared Primitives Extraction

**Status**: Proposed
**Date**: 2025-01-11
**Issue**: Bitcoin decoder implementation added ~1500 LOC. Need to extract reusable components before implementing Ethereum and other decoders.

## Current State Analysis

### Total Codebase: 7,423 LOC

**By Module**:
```
Core Library:           ~2,900 LOC
  canonical.rs:           647 LOC
  ir.rs:                  449 LOC
  hooks.rs:               389 LOC
  traits.rs:              297 LOC
  chain.rs:               224 LOC
  vendored/hex:           525 LOC (external)
  verus_annotations.rs:   164 LOC
  tests:                1,038 LOC

Bitcoin Decoder:        ~1,452 LOC
  parsing.rs:             604 LOC (⚠️ EXTRACTABLE)
  types.rs:               507 LOC
  lib.rs:                 341 LOC

Ethereum Decoder:        ~577 LOC
  types.rs:               357 LOC
  tests:                  220 LOC
```

### Problem Identification

**Bitcoin `parsing.rs` (604 LOC) contains**:
- ✅ **Universal primitives** (reusable across chains):
  - `read_u8`, `read_u16_le`, `read_u32_le`, `read_u64_le` (~40 LOC)
  - `read_bytes` with bounds checking (~20 LOC)
  - Generic error handling patterns

- ⚠️ **Bitcoin-specific** (not reusable):
  - `read_varint` - Bitcoin VarInt encoding (~50 LOC)
  - `parse_input`, `parse_output`, `parse_witness` (~250 LOC)
  - `TxInput`, `TxOutput`, `Witness` types (~50 LOC)
  - Bitcoin transaction structure knowledge

**Ethereum will need**:
- ✅ Same primitive readers (u8, u16, u32, u64, u256)
- ✅ BUT big-endian (Ethereum uses big-endian)
- ❌ RLP encoding (NOT VarInt)
- ❌ Different transaction structure

**Solana will need**:
- ✅ Same primitive readers
- ✅ Little-endian (like Bitcoin)
- ❌ Borsh encoding (NOT VarInt)
- ❌ Different transaction structure

## Proposed Architecture

### Create `decoder-primitives` Crate

**Purpose**: Provide low-level, blockchain-agnostic parsing primitives

**Location**: `crates/decoder-primitives/`

**Contents**:
```
decoder-primitives/
├── src/
│   ├── lib.rs                 # Re-exports
│   ├── readers/
│   │   ├── mod.rs
│   │   ├── little_endian.rs   # LE readers (Bitcoin, Solana)
│   │   └── big_endian.rs      # BE readers (Ethereum)
│   ├── bytes.rs               # Byte operations with bounds checking
│   ├── cursor.rs              # Cursor wrapper with position tracking
│   └── error.rs               # Primitive-level errors (optional)
└── tests/
    └── readers_tests.rs
```

### Extracted Components

#### 1. Universal Primitive Readers

**File**: `decoder-primitives/src/readers/little_endian.rs`

```rust
//! Little-endian primitive readers (Bitcoin, Solana, etc.)

use std::io::Read;
use universal_decoder_core::prelude::Result;

/// Read u8 (1 byte)
#[inline]
pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

/// Read u16 (2 bytes, little-endian)
#[inline]
pub fn read_u16_le<R: Read>(reader: &mut R) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_le_bytes(buf))
}

/// Read u32 (4 bytes, little-endian)
#[inline]
pub fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(buf))
}

/// Read u64 (8 bytes, little-endian)
#[inline]
pub fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_le_bytes(buf))
}

/// Read u128 (16 bytes, little-endian)
#[inline]
pub fn read_u128_le<R: Read>(reader: &mut R) -> Result<u128> {
    let mut buf = [0u8; 16];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u128: {}", e)))?;
    Ok(u128::from_le_bytes(buf))
}
```

**File**: `decoder-primitives/src/readers/big_endian.rs`

```rust
//! Big-endian primitive readers (Ethereum, etc.)

use std::io::Read;
use universal_decoder_core::prelude::Result;

/// Read u16 (2 bytes, big-endian)
#[inline]
pub fn read_u16_be<R: Read>(reader: &mut R) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_be_bytes(buf))
}

/// Read u32 (4 bytes, big-endian)
#[inline]
pub fn read_u32_be<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_be_bytes(buf))
}

/// Read u64 (8 bytes, big-endian)
#[inline]
pub fn read_u64_be<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_be_bytes(buf))
}

/// Read u256 (32 bytes, big-endian) - for Ethereum
pub fn read_u256_be<R: Read>(reader: &mut R) -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to read u256: {}", e)))?;
    Ok(buf)
}
```

#### 2. Byte Operations with Bounds Checking

**File**: `decoder-primitives/src/bytes.rs`

```rust
//! Byte-level operations with bounds checking

use std::io::Read;
use universal_decoder_core::prelude::{Result, DecoderError};

/// Read exactly N bytes with bounds checking
///
/// # Arguments
/// * `reader` - Input reader
/// * `len` - Number of bytes to read
/// * `max_len` - Maximum allowed length (for safety)
///
/// # Errors
/// Returns error if:
/// - `len > max_len`
/// - Not enough bytes available in reader
pub fn read_bytes_bounded<R: Read>(
    reader: &mut R,
    len: usize,
    max_len: usize,
) -> Result<Vec<u8>> {
    if len > max_len {
        return Err(DecoderError::invalid_structure(format!(
            "Requested {} bytes, but maximum is {}",
            len, max_len
        )));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!(
            "Failed to read {} bytes: {}",
            len, e
        )))?;

    Ok(buf)
}

/// Read exactly N bytes into fixed-size array
pub fn read_array<R: Read, const N: usize>(reader: &mut R) -> Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::chain_decoding(format!(
            "Failed to read {} bytes: {}",
            N, e
        )))?;
    Ok(buf)
}

/// Peek at next N bytes without consuming them (requires seekable reader)
#[cfg(feature = "peek")]
pub fn peek_bytes<R: Read + Seek>(reader: &mut R, len: usize) -> Result<Vec<u8>> {
    use std::io::Seek;

    let pos = reader.stream_position()
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to get position: {}", e)))?;

    let bytes = read_bytes_bounded(reader, len, len)?;

    reader.seek(SeekFrom::Start(pos))
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to seek: {}", e)))?;

    Ok(bytes)
}
```

#### 3. Cursor Wrapper with Position Tracking

**File**: `decoder-primitives/src/cursor.rs`

```rust
//! Cursor wrapper with position tracking and bounds checking

use std::io::{Cursor, Read, Seek, SeekFrom};
use universal_decoder_core::prelude::{Result, DecoderError};

/// Wrapper around std::io::Cursor with additional safety checks
pub struct BoundedCursor<'a> {
    cursor: Cursor<&'a [u8]>,
    max_position: usize,
}

impl<'a> BoundedCursor<'a> {
    /// Create new bounded cursor
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
            max_position: data.len(),
        }
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        self.max_position - self.position()
    }

    /// Check if at end
    pub fn is_at_end(&self) -> bool {
        self.position() >= self.max_position
    }

    /// Verify all bytes consumed
    pub fn verify_consumed(&self) -> Result<()> {
        if !self.is_at_end() {
            return Err(DecoderError::invalid_structure(format!(
                "Transaction has {} trailing bytes",
                self.remaining()
            )));
        }
        Ok(())
    }
}

impl<'a> Read for BoundedCursor<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl<'a> Seek for BoundedCursor<'a> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.cursor.seek(pos)
    }
}
```

### Refactored Bitcoin Decoder

**File**: `decoder-bitcoin/src/parsing.rs` (AFTER refactoring)

```rust
//! Bitcoin-specific transaction parsing

use decoder_primitives::prelude::*;  // Import primitives
use universal_decoder_core::prelude::*;
use std::io::Read;

// Bitcoin-specific constants (keep here)
pub const MAX_SCRIPT_SIZE: usize = 10_000;
pub const MAX_TRANSACTION_SIZE: usize = 100_000;
pub const MAX_INPUTS_OUTPUTS: usize = 10_000;

// Bitcoin-specific VarInt (keep here - Bitcoin-only encoding)
pub fn read_varint<R: Read>(reader: &mut R) -> Result<u64> {
    let first_byte = read_u8(reader)?;  // ← Use primitives
    match first_byte {
        0x00..=0xFC => Ok(first_byte as u64),
        0xFD => {
            let value = read_u16_le(reader)?;  // ← Use primitives
            if value < 0xFD {
                return Err(DecoderError::invalid_structure(format!(
                    "Non-canonical VarInt: 0xFD prefix for value {}",
                    value
                )));
            }
            Ok(value as u64)
        }
        0xFE => {
            let value = read_u32_le(reader)?;  // ← Use primitives
            if value < 0x10000 {
                return Err(DecoderError::invalid_structure(format!(
                    "Non-canonical VarInt: 0xFE prefix for value {}",
                    value
                )));
            }
            Ok(value as u64)
        }
        0xFF => {
            let value = read_u64_le(reader)?;  // ← Use primitives
            if value < 0x100000000 {
                return Err(DecoderError::invalid_structure(format!(
                    "Non-canonical VarInt: 0xFF prefix for value {}",
                    value
                )));
            }
            Ok(value)
        }
    }
}

// Bitcoin-specific types and parsers (keep here)
// ... rest of Bitcoin-specific code
```

## Benefits of Refactoring

### 1. Code Reuse
- ✅ Ethereum decoder can use `decoder_primitives::big_endian::*`
- ✅ Solana decoder can use `decoder_primitives::little_endian::*`
- ✅ All decoders share bounds checking logic

### 2. Reduced Duplication
- **Before**: Each decoder implements own primitive readers (~60 LOC each)
- **After**: All decoders import from `decoder-primitives` (0 LOC)
- **Savings**: ~60 LOC × 10 decoders = 600 LOC saved

### 3. Improved Testability
- Primitive readers tested once in `decoder-primitives`
- Decoders only test blockchain-specific logic

### 4. Better Separation of Concerns
```
decoder-primitives     → Universal byte-level operations
decoder-bitcoin        → Bitcoin VarInt, transaction structure
decoder-ethereum       → RLP encoding, transaction structure
decoder-solana         → Borsh encoding, instruction format
```

### 5. Smaller TCB Per Decoder
- Bitcoin decoder: ~1400 LOC → ~1340 LOC (60 LOC moved to primitives)
- Primitives crate: ~100 LOC (shared, tested once)
- Future decoders start with primitives, not from scratch

## Implementation Plan

### Phase 1: Create Primitives Crate (2 hours)
1. Create `crates/decoder-primitives/` structure
2. Extract little-endian readers from Bitcoin decoder
3. Add big-endian readers for Ethereum
4. Add byte operations with bounds checking
5. Add comprehensive tests

### Phase 2: Refactor Bitcoin Decoder (1 hour)
1. Add `decoder-primitives` dependency
2. Replace local primitives with imports
3. Keep Bitcoin-specific VarInt and parsing
4. Run all tests to verify

### Phase 3: Update Documentation (30 min)
1. Update CLAUDE.md with architecture
2. Document primitives crate API
3. Update contribution guide

## File Structure After Refactoring

```
crates/
├── universal-decoder-core/     (~2900 LOC)
│   ├── Core traits and IR
│   └── Canonical serialization
│
├── decoder-primitives/         (~100 LOC) ← NEW
│   ├── Little-endian readers
│   ├── Big-endian readers
│   ├── Byte operations
│   └── Cursor wrappers
│
├── decoder-bitcoin/            (~1340 LOC) ← REDUCED
│   ├── Bitcoin VarInt (Bitcoin-specific)
│   ├── Transaction parsing
│   └── Uses decoder-primitives
│
└── decoder-ethereum/           (~577 LOC)
    ├── RLP encoding (Ethereum-specific)
    ├── Transaction parsing
    └── Will use decoder-primitives
```

## Decision Matrix

| Component | Location | Reason |
|-----------|----------|--------|
| `read_u8`, `read_u16_le`, etc. | `decoder-primitives` | Universal across all chains |
| `read_bytes_bounded` | `decoder-primitives` | Universal bounds checking |
| Bitcoin VarInt | `decoder-bitcoin` | Bitcoin-specific encoding |
| Ethereum RLP | `decoder-ethereum` | Ethereum-specific encoding |
| Solana Borsh | `decoder-solana` | Solana uses standard Borsh crate |
| Transaction types | Chain decoders | Chain-specific structures |

## Risks and Mitigations

### Risk 1: Breaking Changes
- **Mitigation**: Keep all tests, run full suite after refactoring
- **Rollback**: Keep git commit before refactoring

### Risk 2: Dependency Complexity
- **Mitigation**: `decoder-primitives` has ZERO external dependencies
- **Import**: Only depends on `universal-decoder-core` for error types

### Risk 3: Over-Abstraction
- **Mitigation**: Only extract truly universal primitives
- **Guideline**: If 3+ decoders need it, extract. Otherwise, keep local.

## Success Criteria

- ✅ All existing tests pass (56/56)
- ✅ Bitcoin decoder reduces by ~60 LOC
- ✅ `decoder-primitives` is < 200 LOC
- ✅ No performance regression (benchmarks)
- ✅ Documentation updated

## Alternative Considered: Keep Everything in Bitcoin Decoder

**Pros**:
- No refactoring needed
- Simpler for now

**Cons**:
- Ethereum decoder will duplicate ~60 LOC
- Solana decoder will duplicate ~60 LOC
- By 10 decoders: ~600 LOC duplicated
- Harder to maintain (bug fixes in 10 places)

**Decision**: Extract now before duplication happens.

## Conclusion

**Recommendation**: ✅ **Proceed with refactoring**

**Reasoning**:
1. Prevents duplication before it happens
2. Cleaner architecture for Phase 2 (Ethereum decoder)
3. Small effort now (3 hours) vs. large tech debt later
4. Aligns with "minimal TCB" principle (shared code is reviewed once)

**Timeline**:
- Phase 1 (primitives): 2 hours
- Phase 2 (refactor Bitcoin): 1 hour
- Phase 3 (docs): 30 min
- **Total**: ~3.5 hours

---

**Status**: Ready for approval
**Next**: Create `decoder-primitives` crate upon approval
