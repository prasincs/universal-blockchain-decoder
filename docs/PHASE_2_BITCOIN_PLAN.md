# Phase 2: Pure Rust Bitcoin Decoder Implementation Plan

**Status**: Planning Complete, Ready for Implementation
**Timeline**: 2 weeks (Weeks 1-2 of Phase 2)
**Priority**: HIGH - Reference UTXO implementation
**Aligned with**: Learn Me A Bitcoin transaction specification

## Executive Summary

This document outlines the complete implementation plan for Phase 2.1 of the Universal Blockchain Decoder roadmap: building a **pure Rust Bitcoin transaction decoder** with **zero production dependencies** on external blockchain libraries.

### Core Principles

1. **Pure Rust Implementation**: All Bitcoin transaction parsing logic implemented from scratch
2. **Test-Driven Validation**: Validate correctness against `bitcoin` crate in dev-dependencies
3. **Learn Me A Bitcoin Alignment**: Follow authoritative Bitcoin transaction specification
4. **Comprehensive Testing**: Unit, property, integration, and fuzz tests
5. **No Panics**: All parsing is fallible and returns `Result<T, DecoderError>`

## Current State Analysis

### Existing Implementation Issues

❌ **Problem**: Current implementation relies on `bitcoin` crate for parsing

```rust
// crates/decoder-bitcoin/src/lib.rs (CURRENT - WRONG)
use bitcoin::{consensus::Decodable, Transaction as BitcoinTx};

fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
    let tx = BitcoinTx::consensus_decode(&mut cursor)?;  // ❌ External dependency
    Ok(BitcoinTransaction::from_bitcoin_tx(tx, raw_bytes))
}
```

**Why This Violates Design Principles**:
- Adds ~50k+ LOC to Trusted Computing Base (TCB)
- Decoder is just a thin wrapper around external library
- Cannot be formally verified
- Defeats purpose of universal decoder
- External library controls our behavior

### Target Architecture

✅ **Solution**: Pure Rust implementation with test validation

```rust
// crates/decoder-bitcoin/src/lib.rs (TARGET - CORRECT)

fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
    let mut cursor = Cursor::new(raw_bytes);

    // Parse version
    let version = read_u32_le(&mut cursor)?;

    // Detect SegWit
    let is_segwit = detect_segwit(&mut cursor)?;

    // Parse inputs
    let inputs = parse_inputs(&mut cursor)?;

    // Parse outputs
    let outputs = parse_outputs(&mut cursor)?;

    // Parse witness (if SegWit)
    let witnesses = if is_segwit {
        parse_witnesses(&mut cursor, inputs.len())?
    } else {
        vec![]
    };

    // Parse locktime
    let locktime = read_u32_le(&mut cursor)?;

    Ok(BitcoinTransaction {
        version,
        inputs,
        outputs,
        witnesses,
        locktime,
        raw_bytes: raw_bytes.to_vec(),
    })
}
```

**Benefits**:
- ✅ Pure Rust (minimal TCB)
- ✅ Formally verifiable
- ✅ Full control over parsing logic
- ✅ Tests validate against `bitcoin` crate (in dev-deps)

## Bitcoin Transaction Structure (Learn Me A Bitcoin)

### Legacy Transaction Format (Pre-SegWit)

```
┌─────────────────────────────────────────────────────┐
│ Field            │ Size      │ Description          │
├─────────────────────────────────────────────────────┤
│ Version          │ 4 bytes   │ Transaction version  │
│ Input Count      │ VarInt    │ Number of inputs     │
│ Inputs           │ Variable  │ Transaction inputs   │
│ Output Count     │ VarInt    │ Number of outputs    │
│ Outputs          │ Variable  │ Transaction outputs  │
│ Locktime         │ 4 bytes   │ Lock time value      │
└─────────────────────────────────────────────────────┘
```

### SegWit Transaction Format (BIP 141)

```
┌─────────────────────────────────────────────────────┐
│ Field            │ Size      │ Description          │
├─────────────────────────────────────────────────────┤
│ Version          │ 4 bytes   │ Transaction version  │
│ Marker           │ 1 byte    │ Always 0x00          │
│ Flag             │ 1 byte    │ Always 0x01          │
│ Input Count      │ VarInt    │ Number of inputs     │
│ Inputs           │ Variable  │ Transaction inputs   │
│ Output Count     │ VarInt    │ Number of outputs    │
│ Outputs          │ Variable  │ Transaction outputs  │
│ Witnesses        │ Variable  │ Witness data         │
│ Locktime         │ 4 bytes   │ Lock time value      │
└─────────────────────────────────────────────────────┘
```

### Transaction Input Structure

```
┌─────────────────────────────────────────────────────┐
│ Field            │ Size      │ Description          │
├─────────────────────────────────────────────────────┤
│ Previous Hash    │ 32 bytes  │ TXID being spent     │
│ Previous Index   │ 4 bytes   │ Output index         │
│ Script Length    │ VarInt    │ Length of script_sig │
│ Script Sig       │ Variable  │ Unlocking script     │
│ Sequence         │ 4 bytes   │ Sequence number      │
└─────────────────────────────────────────────────────┘
```

### Transaction Output Structure

```
┌─────────────────────────────────────────────────────┐
│ Field            │ Size      │ Description          │
├─────────────────────────────────────────────────────┤
│ Value            │ 8 bytes   │ Satoshis (u64)       │
│ Script Length    │ VarInt    │ Length of script     │
│ Script PubKey    │ Variable  │ Locking script       │
└─────────────────────────────────────────────────────┘
```

### VarInt Encoding (Variable-Length Integer)

```
┌──────────────────────────────────────────────────────┐
│ First Byte │ Size      │ Range                       │
├──────────────────────────────────────────────────────┤
│ 0x00-0xFC  │ 1 byte    │ 0 to 252                    │
│ 0xFD       │ 3 bytes   │ 253 to 65,535               │
│ 0xFE       │ 5 bytes   │ 65,536 to 4,294,967,295     │
│ 0xFF       │ 9 bytes   │ 4,294,967,296 to 2^64-1     │
└──────────────────────────────────────────────────────┘
```

**Examples**:
- `0x12` → 18 (1 byte)
- `0xFD FD 00` → 253 (3 bytes: 0xFD + little-endian u16)
- `0xFE 00 01 00 00` → 256 (5 bytes: 0xFE + little-endian u32)
- `0xFF 00 00 00 00 01 00 00 00` → 4,294,967,296 (9 bytes: 0xFF + little-endian u64)

### Witness Data Structure

For SegWit transactions, each input has a witness field:

```
┌─────────────────────────────────────────────────────┐
│ Field            │ Size      │ Description          │
├─────────────────────────────────────────────────────┤
│ Stack Item Count │ VarInt    │ Number of items      │
│ Stack Items      │ Variable  │ Witness stack items  │
└─────────────────────────────────────────────────────┘

Each stack item:
┌─────────────────────────────────────────────────────┐
│ Item Length      │ VarInt    │ Length of item       │
│ Item Data        │ Variable  │ Witness data         │
└─────────────────────────────────────────────────────┘
```

## Implementation Plan

### Phase 2.1.1: Core Parsing Utilities (Day 1-2)

**File**: `crates/decoder-bitcoin/src/parsing.rs`

#### Task 1.1: VarInt Parser

```rust
/// Parse a Bitcoin VarInt from the cursor
/// Returns (value, bytes_consumed)
pub fn read_varint<R: Read>(reader: &mut R) -> Result<u64, DecoderError> {
    let first_byte = read_u8(reader)?;

    match first_byte {
        0x00..=0xFC => Ok(first_byte as u64),
        0xFD => {
            let value = read_u16_le(reader)?;
            Ok(value as u64)
        }
        0xFE => {
            let value = read_u32_le(reader)?;
            Ok(value as u64)
        }
        0xFF => {
            read_u64_le(reader)
        }
    }
}
```

**Tests Required**:
- `test_varint_single_byte`: 0x12 → 18
- `test_varint_fd`: 0xFD FD 00 → 253
- `test_varint_fe`: 0xFE 00 01 00 00 → 256
- `test_varint_ff`: Maximum u64 value
- `test_varint_truncated`: Returns error on incomplete data
- `test_varint_property`: Encode/decode roundtrip

#### Task 1.2: Primitive Readers

```rust
/// Read u8
pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8, DecoderError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::insufficient_data(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

/// Read u16 (little-endian)
pub fn read_u16_le<R: Read>(reader: &mut R) -> Result<u16, DecoderError> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::insufficient_data(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_le_bytes(buf))
}

/// Read u32 (little-endian)
pub fn read_u32_le<R: Read>(reader: &mut R) -> Result<u32, DecoderError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::insufficient_data(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(buf))
}

/// Read u64 (little-endian)
pub fn read_u64_le<R: Read>(reader: &mut R) -> Result<u64, DecoderError> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::insufficient_data(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_le_bytes(buf))
}

/// Read exactly N bytes
pub fn read_bytes<R: Read>(reader: &mut R, len: usize) -> Result<Vec<u8>, DecoderError> {
    if len > MAX_SCRIPT_SIZE {
        return Err(DecoderError::invalid_structure("Script too large"));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)
        .map_err(|e| DecoderError::insufficient_data(format!("Failed to read {} bytes: {}", len, e)))?;
    Ok(buf)
}
```

**Constants**:
```rust
/// Maximum script size (520 bytes for standard scripts)
pub const MAX_SCRIPT_SIZE: usize = 10_000; // Conservative limit

/// Maximum transaction size (100 KB for standard transactions)
pub const MAX_TRANSACTION_SIZE: usize = 100_000;

/// Maximum number of inputs/outputs (sanity check)
pub const MAX_INPUTS_OUTPUTS: usize = 10_000;
```

**Tests Required**:
- Test all primitive readers with valid data
- Test with truncated data (should return error)
- Test with oversized data (should return error)
- Property tests for roundtrip

### Phase 2.1.2: Input/Output Parsers (Day 3-4)

**File**: `crates/decoder-bitcoin/src/parsing.rs`

#### Task 2.1: Transaction Input Parser

```rust
/// Bitcoin transaction input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInput {
    /// Previous transaction hash (32 bytes, little-endian)
    pub prev_hash: [u8; 32],
    /// Previous output index
    pub prev_index: u32,
    /// Unlocking script (scriptSig)
    pub script_sig: Vec<u8>,
    /// Sequence number
    pub sequence: u32,
}

/// Parse a transaction input
pub fn parse_input<R: Read>(reader: &mut R) -> Result<TxInput, DecoderError> {
    // Read previous transaction hash (32 bytes)
    let mut prev_hash = [0u8; 32];
    reader.read_exact(&mut prev_hash)
        .map_err(|e| DecoderError::insufficient_data(format!("Failed to read prev_hash: {}", e)))?;

    // Read previous output index (4 bytes, little-endian)
    let prev_index = read_u32_le(reader)?;

    // Read script length (varint)
    let script_len = read_varint(reader)?;
    if script_len > MAX_SCRIPT_SIZE as u64 {
        return Err(DecoderError::invalid_structure("Script too large"));
    }

    // Read script bytes
    let script_sig = read_bytes(reader, script_len as usize)?;

    // Read sequence (4 bytes, little-endian)
    let sequence = read_u32_le(reader)?;

    Ok(TxInput {
        prev_hash,
        prev_index,
        script_sig,
        sequence,
    })
}
```

**Tests Required**:
- `test_parse_input_simple`: Parse a simple input
- `test_parse_input_coinbase`: Parse coinbase input (prev_hash all zeros)
- `test_parse_input_empty_script`: Parse input with empty script_sig
- `test_parse_input_large_script`: Parse input with large script
- `test_parse_input_truncated`: Returns error on truncated data
- `test_parse_input_oversized_script`: Returns error on script > MAX_SCRIPT_SIZE

#### Task 2.2: Transaction Output Parser

```rust
/// Bitcoin transaction output
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    /// Value in satoshis
    pub value: u64,
    /// Locking script (scriptPubKey)
    pub script_pubkey: Vec<u8>,
}

/// Parse a transaction output
pub fn parse_output<R: Read>(reader: &mut R) -> Result<TxOutput, DecoderError> {
    // Read value (8 bytes, little-endian)
    let value = read_u64_le(reader)?;

    // Read script length (varint)
    let script_len = read_varint(reader)?;
    if script_len > MAX_SCRIPT_SIZE as u64 {
        return Err(DecoderError::invalid_structure("Script too large"));
    }

    // Read script bytes
    let script_pubkey = read_bytes(reader, script_len as usize)?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}
```

**Tests Required**:
- `test_parse_output_simple`: Parse a simple output
- `test_parse_output_zero_value`: Parse output with 0 satoshis
- `test_parse_output_max_value`: Parse output with max satoshis
- `test_parse_output_p2pkh`: Parse P2PKH output
- `test_parse_output_p2sh`: Parse P2SH output
- `test_parse_output_p2wpkh`: Parse P2WPKH output
- `test_parse_output_truncated`: Returns error on truncated data

### Phase 2.1.3: Witness Parser (Day 5)

**File**: `crates/decoder-bitcoin/src/parsing.rs`

#### Task 3.1: Witness Data Parser

```rust
/// Witness data for a single input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    /// Stack items (witness elements)
    pub items: Vec<Vec<u8>>,
}

impl Witness {
    /// Create an empty witness
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    /// Check if witness is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Parse witness data for a single input
pub fn parse_witness<R: Read>(reader: &mut R) -> Result<Witness, DecoderError> {
    // Read number of stack items (varint)
    let stack_count = read_varint(reader)?;

    if stack_count > MAX_INPUTS_OUTPUTS as u64 {
        return Err(DecoderError::invalid_structure("Too many witness items"));
    }

    // Read each stack item
    let mut items = Vec::with_capacity(stack_count as usize);
    for _ in 0..stack_count {
        let item_len = read_varint(reader)?;
        if item_len > MAX_SCRIPT_SIZE as u64 {
            return Err(DecoderError::invalid_structure("Witness item too large"));
        }

        let item = read_bytes(reader, item_len as usize)?;
        items.push(item);
    }

    Ok(Witness { items })
}

/// Parse witness data for all inputs
pub fn parse_witnesses<R: Read>(
    reader: &mut R,
    input_count: usize,
) -> Result<Vec<Witness>, DecoderError> {
    let mut witnesses = Vec::with_capacity(input_count);

    for _ in 0..input_count {
        witnesses.push(parse_witness(reader)?);
    }

    Ok(witnesses)
}
```

**Tests Required**:
- `test_parse_witness_empty`: Parse empty witness (0 items)
- `test_parse_witness_p2wpkh`: Parse P2WPKH witness (2 items: signature, pubkey)
- `test_parse_witness_p2wsh`: Parse P2WSH witness (multiple items)
- `test_parse_witness_truncated`: Returns error on truncated data
- `test_parse_witnesses_multiple`: Parse witnesses for multiple inputs

### Phase 2.1.4: Transaction Parser (Day 6-7)

**File**: `crates/decoder-bitcoin/src/lib.rs`

#### Task 4.1: SegWit Detection

```rust
/// Detect if transaction uses SegWit format
/// Peeks at bytes 4 and 5 after version
fn detect_segwit<R: Read + Seek>(reader: &mut R) -> Result<bool, DecoderError> {
    // Save current position
    let pos = reader.stream_position()
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to get position: {}", e)))?;

    // Read marker and flag
    let marker = read_u8(reader)?;
    let flag = read_u8(reader)?;

    // Restore position
    reader.seek(SeekFrom::Start(pos))
        .map_err(|e| DecoderError::chain_decoding(format!("Failed to seek: {}", e)))?;

    // SegWit transactions have marker=0x00, flag=0x01
    Ok(marker == 0x00 && flag == 0x01)
}
```

**Alternative (for non-seekable readers)**:
```rust
/// Detect SegWit format by peeking ahead
/// Returns (is_segwit, input_count_position)
fn detect_segwit_and_parse_marker(bytes: &[u8], offset: usize) -> Result<(bool, usize), DecoderError> {
    if offset + 2 > bytes.len() {
        return Err(DecoderError::insufficient_data("Not enough bytes to detect SegWit"));
    }

    let marker = bytes[offset];
    let flag = bytes[offset + 1];

    if marker == 0x00 && flag == 0x01 {
        // SegWit: skip marker and flag
        Ok((true, offset + 2))
    } else {
        // Legacy: marker is actually input count
        Ok((false, offset))
    }
}
```

#### Task 4.2: Full Transaction Parser

```rust
/// Bitcoin transaction (parsed)
#[derive(Debug, Clone)]
pub struct BitcoinTransaction {
    /// Transaction version
    pub version: u32,
    /// Transaction inputs
    pub inputs: Vec<TxInput>,
    /// Transaction outputs
    pub outputs: Vec<TxOutput>,
    /// Witness data (if SegWit)
    pub witnesses: Vec<Witness>,
    /// Lock time
    pub locktime: u32,
    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl ChainDecoder for BitcoinDecoder {
    type TxSpecific = BitcoinTransaction;
    type Chain = BitcoinChain;

    fn chain() -> Self::Chain {
        BitcoinChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction too small (minimum 10 bytes)"
            ));
        }

        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction too large"
            ));
        }

        let mut cursor = Cursor::new(raw_bytes);

        // Parse version (4 bytes, little-endian)
        let version = read_u32_le(&mut cursor)?;

        // Detect SegWit
        let pos_after_version = cursor.position() as usize;
        let (is_segwit, input_count_pos) = detect_segwit_and_parse_marker(
            raw_bytes,
            pos_after_version,
        )?;

        // Skip marker and flag if SegWit
        if is_segwit {
            cursor.set_position(input_count_pos as u64);
        }

        // Parse input count (varint)
        let input_count = read_varint(&mut cursor)?;
        if input_count > MAX_INPUTS_OUTPUTS as u64 {
            return Err(DecoderError::invalid_structure("Too many inputs"));
        }

        // Parse inputs
        let mut inputs = Vec::with_capacity(input_count as usize);
        for i in 0..input_count {
            inputs.push(parse_input(&mut cursor)
                .map_err(|e| DecoderError::chain_decoding(
                    format!("Failed to parse input {}: {}", i, e)
                ))?);
        }

        // Parse output count (varint)
        let output_count = read_varint(&mut cursor)?;
        if output_count > MAX_INPUTS_OUTPUTS as u64 {
            return Err(DecoderError::invalid_structure("Too many outputs"));
        }

        // Parse outputs
        let mut outputs = Vec::with_capacity(output_count as usize);
        for i in 0..output_count {
            outputs.push(parse_output(&mut cursor)
                .map_err(|e| DecoderError::chain_decoding(
                    format!("Failed to parse output {}: {}", i, e)
                ))?);
        }

        // Parse witness data (if SegWit)
        let witnesses = if is_segwit {
            parse_witnesses(&mut cursor, inputs.len())?
        } else {
            vec![Witness::empty(); inputs.len()]
        };

        // Parse locktime (4 bytes, little-endian)
        let locktime = read_u32_le(&mut cursor)?;

        // Verify we consumed all bytes
        let consumed = cursor.position() as usize;
        if consumed != raw_bytes.len() {
            return Err(DecoderError::invalid_structure(
                format!(
                    "Transaction has {} trailing bytes (consumed {}, total {})",
                    raw_bytes.len() - consumed,
                    consumed,
                    raw_bytes.len()
                )
            ));
        }

        Ok(BitcoinTransaction {
            version,
            inputs,
            outputs,
            witnesses,
            locktime,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction cannot be empty",
            ));
        }

        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction too small (minimum 10 bytes)",
            ));
        }

        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction too large",
            ));
        }

        Ok(())
    }
}
```

**Tests Required**:
- `test_decode_legacy_transaction`: Parse legacy (pre-SegWit) transaction
- `test_decode_segwit_transaction`: Parse SegWit transaction with witness data
- `test_decode_genesis_coinbase`: Parse Bitcoin genesis coinbase
- `test_decode_simple_p2pkh`: Parse simple P2PKH transaction
- `test_decode_multisig`: Parse multisig transaction
- `test_decode_taproot`: Parse Taproot transaction (if supported)
- `test_decode_truncated`: Returns error on truncated data
- `test_decode_trailing_bytes`: Returns error on trailing bytes
- `test_decode_empty`: Returns error on empty input
- `test_decode_too_large`: Returns error on oversized input

### Phase 2.1.5: BitcoinTransaction Methods (Day 8)

**File**: `crates/decoder-bitcoin/src/types.rs`

#### Task 5.1: Update BitcoinTransaction

```rust
impl BitcoinTransaction {
    /// Get transaction version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get the number of inputs
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Get the number of outputs
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Check if transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1
            && self.inputs[0].prev_hash == [0u8; 32]
            && self.inputs[0].prev_index == 0xFFFFFFFF
    }

    /// Check if transaction uses SegWit
    pub fn is_segwit(&self) -> bool {
        self.witnesses.iter().any(|w| !w.is_empty())
    }

    /// Calculate transaction ID (TXID)
    /// For SegWit transactions, this is the hash of the non-witness serialization
    pub fn txid(&self) -> Vec<u8> {
        // TODO: Implement non-witness serialization for SegWit
        // For now, use raw_bytes (incorrect for SegWit)
        use sha2::{Sha256, Digest};
        let hash1 = Sha256::digest(&self.raw_bytes);
        let hash2 = Sha256::digest(hash1);
        hash2.to_vec()
    }

    /// Calculate witness transaction ID (WTXID)
    /// This includes witness data (same as TXID for non-SegWit)
    pub fn wtxid(&self) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        let hash1 = Sha256::digest(&self.raw_bytes);
        let hash2 = Sha256::digest(hash1);
        hash2.to_vec()
    }

    /// Calculate total output value
    pub fn total_output_value(&self) -> Result<u64> {
        self.outputs
            .iter()
            .try_fold(0u64, |acc, output| {
                acc.checked_add(output.value)
                    .ok_or_else(|| DecoderError::overflow("Output value overflow"))
            })
    }

    /// Calculate fee (requires input values from UTXO set)
    pub fn calculate_fee(&self, input_values: &[u64]) -> Option<u64> {
        if input_values.len() != self.inputs.len() {
            return None;
        }

        let total_input: u64 = input_values.iter().sum();
        let total_output: u64 = self.outputs.iter().map(|o| o.value).sum();

        total_input.checked_sub(total_output)
    }
}
```

### Phase 2.1.6: Canonicalization (Day 9)

**File**: `crates/decoder-bitcoin/src/types.rs`

Update the `Canonicalizer` implementation to use the new pure Rust types:

```rust
impl<'a> Canonicalizer<'a> for BitcoinTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Build metadata
        let extra = format!(
            r#"{{"version":{},"lock_time":{},"is_coinbase":{},"is_segwit":{}}}"#,
            self.version,
            self.locktime,
            self.is_coinbase(),
            self.is_segwit()
        );

        let metadata = TxMetadata {
            tx_hash: self.txid(),
            block_height: None,
            timestamp: Some(self.locktime as u64),
            size: self.raw_bytes.len(),
            extra,
        };

        // Build authorization (extract signatures from inputs and witnesses)
        let mut signatures = Vec::new();

        for (idx, input) in self.inputs.iter().enumerate() {
            // scriptSig signatures (legacy)
            if !input.script_sig.is_empty() {
                signatures.push(Signature {
                    data: input.script_sig.clone(),
                    key_index: idx,
                    metadata: Some(format!(r#"{{"input_index":{}}}"#, idx)),
                });
            }
        }

        // Witness signatures (SegWit)
        for (idx, witness) in self.witnesses.iter().enumerate() {
            for (item_idx, item) in witness.items.iter().enumerate() {
                signatures.push(Signature {
                    data: item.clone(),
                    key_index: idx,
                    metadata: Some(format!(
                        r#"{{"input_index":{},"witness_index":{}}}"#,
                        idx, item_idx
                    )),
                });
            }
        }

        let authorization = AuthorizationPackage {
            signatures,
            public_keys: vec![], // TODO: Extract from scripts
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Build operations (transfers from outputs)
        let operations = self.outputs
            .iter()
            .map(|output| Operation::Transfer(Transfer {
                from: Address {
                    bytes: vec![],
                    human_readable: None,
                },
                to: Address {
                    bytes: output.script_pubkey.clone(),
                    human_readable: None, // TODO: Decode address
                },
                amount: Amount {
                    value: output.value as u128,
                    decimals: 8,
                },
                asset: AssetId::Native,
            }))
            .collect();

        // Build state deltas
        let inputs = self.inputs
            .iter()
            .map(|input| InputReference {
                prev_tx: input.prev_hash.to_vec(),
                output_index: input.prev_index,
                value: Amount {
                    value: 0, // Requires UTXO set
                    decimals: 8,
                },
                script: input.script_sig.clone(),
            })
            .collect();

        let outputs = self.outputs
            .iter()
            .enumerate()
            .map(|(idx, output)| OutputValue {
                index: idx as u32,
                address: Address {
                    bytes: output.script_pubkey.clone(),
                    human_readable: None,
                },
                value: Amount {
                    value: output.value as u128,
                    decimals: 8,
                },
                script: output.script_pubkey.clone(),
            })
            .collect();

        let state_deltas = StateDeltas {
            inputs,
            outputs,
            account_changes: vec![],
        };

        Ok(TxIR::new(
            &BitcoinChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Check version
        if self.version < 1 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid Bitcoin transaction version: {}",
                self.version
            )));
        }

        // Check inputs
        if self.inputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction must have at least one input",
            ));
        }

        // Check outputs
        if self.outputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction must have at least one output",
            ));
        }

        // Check for overflow in output values
        self.total_output_value()?;

        // Check witness data consistency
        if self.is_segwit() && self.witnesses.len() != self.inputs.len() {
            return Err(DecoderError::invalid_structure(
                "Witness count must match input count for SegWit transactions",
            ));
        }

        Ok(())
    }
}
```

## Testing Strategy

### Unit Tests (Phase 2.1.7, Day 10)

**File**: `crates/decoder-bitcoin/src/parsing.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_varint_single_byte() {
        let data = vec![0x12];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 18);
    }

    #[test]
    fn test_read_varint_fd() {
        let data = vec![0xFD, 0xFD, 0x00]; // 253 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 253);
    }

    #[test]
    fn test_read_varint_fe() {
        let data = vec![0xFE, 0x00, 0x01, 0x00, 0x00]; // 256 in little-endian
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), 256);
    }

    #[test]
    fn test_read_varint_ff() {
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let mut cursor = Cursor::new(data);
        assert_eq!(read_varint(&mut cursor).unwrap(), u64::MAX);
    }

    #[test]
    fn test_read_varint_truncated() {
        let data = vec![0xFD, 0x00]; // Incomplete
        let mut cursor = Cursor::new(data);
        assert!(read_varint(&mut cursor).is_err());
    }

    #[test]
    fn test_parse_input_simple() {
        let data = vec![
            // prev_hash (32 bytes, all zeros for simplicity)
            0x00; 32,
            // prev_index (4 bytes)
            0x00, 0x00, 0x00, 0x00,
            // script_len (varint: 0)
            0x00,
            // sequence (4 bytes)
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let mut cursor = Cursor::new(data);
        let input = parse_input(&mut cursor).unwrap();

        assert_eq!(input.prev_hash, [0u8; 32]);
        assert_eq!(input.prev_index, 0);
        assert_eq!(input.script_sig, vec![]);
        assert_eq!(input.sequence, 0xFFFFFFFF);
    }

    #[test]
    fn test_parse_output_simple() {
        let data = vec![
            // value (8 bytes: 50 BTC in satoshis)
            0x00, 0xF2, 0x05, 0x2A, 0x01, 0x00, 0x00, 0x00,
            // script_len (varint: 0)
            0x00,
        ];

        let mut cursor = Cursor::new(data);
        let output = parse_output(&mut cursor).unwrap();

        assert_eq!(output.value, 5_000_000_000); // 50 BTC
        assert_eq!(output.script_pubkey, vec![]);
    }
}
```

### Integration Tests (Phase 2.1.8, Day 11-12)

**File**: `crates/decoder-bitcoin/tests/validation_tests.rs`

```rust
//! Validation tests: Compare pure Rust implementation with bitcoin crate

use decoder_bitcoin::*;
use universal_decoder_core::prelude::*;

#[cfg(test)]
mod validation {
    use super::*;
    use bitcoin::{consensus::Decodable, Transaction as BitcoinTx};
    use std::io::Cursor;

    #[test]
    fn test_genesis_coinbase_matches_bitcoin_crate() {
        let tx_hex = include_str!("fixtures/btc_genesis_coinbase.hex");
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim())
            .expect("Failed to decode hex");

        // Our implementation
        let our_tx = BitcoinDecoder::decode(&tx_bytes)
            .expect("Failed to decode with our implementation");

        // Reference implementation (bitcoin crate)
        let mut cursor = Cursor::new(&tx_bytes);
        let ref_tx = BitcoinTx::consensus_decode(&mut cursor)
            .expect("Failed to decode with bitcoin crate");

        // Validate version
        assert_eq!(our_tx.version, ref_tx.version.0 as u32);

        // Validate input count
        assert_eq!(our_tx.inputs.len(), ref_tx.input.len());

        // Validate output count
        assert_eq!(our_tx.outputs.len(), ref_tx.output.len());

        // Validate locktime
        assert_eq!(our_tx.locktime, ref_tx.lock_time.to_consensus_u32());

        // Validate each input
        for (our_input, ref_input) in our_tx.inputs.iter().zip(ref_tx.input.iter()) {
            assert_eq!(
                our_input.prev_hash,
                ref_input.previous_output.txid.as_ref()
            );
            assert_eq!(our_input.prev_index, ref_input.previous_output.vout);
            assert_eq!(our_input.script_sig, ref_input.script_sig.as_bytes());
            assert_eq!(our_input.sequence, ref_input.sequence.0);
        }

        // Validate each output
        for (our_output, ref_output) in our_tx.outputs.iter().zip(ref_tx.output.iter()) {
            assert_eq!(our_output.value, ref_output.value.to_sat());
            assert_eq!(our_output.script_pubkey, ref_output.script_pubkey.as_bytes());
        }
    }

    #[test]
    fn test_simple_p2pkh_matches_bitcoin_crate() {
        let tx_hex = include_str!("fixtures/btc_simple_p2pkh.hex");
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim())
            .expect("Failed to decode hex");

        // Decode with both implementations
        let our_tx = BitcoinDecoder::decode(&tx_bytes).expect("Our decoder failed");
        let mut cursor = Cursor::new(&tx_bytes);
        let ref_tx = BitcoinTx::consensus_decode(&mut cursor).expect("Bitcoin crate failed");

        // Comprehensive validation
        assert_eq!(our_tx.version, ref_tx.version.0 as u32);
        assert_eq!(our_tx.inputs.len(), ref_tx.input.len());
        assert_eq!(our_tx.outputs.len(), ref_tx.output.len());
        assert_eq!(our_tx.locktime, ref_tx.lock_time.to_consensus_u32());
    }
}
```

### Property Tests (Phase 2.1.9, Day 13)

**File**: `crates/decoder-bitcoin/tests/property_tests.rs`

```rust
use proptest::prelude::*;
use decoder_bitcoin::*;
use universal_decoder_core::prelude::*;

proptest! {
    /// Property: Decoder never panics on random input
    #[test]
    fn prop_decode_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        // Should either succeed or return error, never panic
        let _ = BitcoinDecoder::decode(&bytes);
    }

    /// Property: Valid transaction decodes successfully
    #[test]
    fn prop_real_transactions_decode(
        tx_fixture in prop::sample::select(vec![
            include_bytes!("fixtures/btc_genesis_coinbase.hex"),
            include_bytes!("fixtures/btc_simple_p2pkh.hex"),
        ])
    ) {
        let tx_hex = std::str::from_utf8(tx_fixture).unwrap();
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex.trim()).unwrap();

        let result = BitcoinDecoder::decode(&tx_bytes);
        prop_assert!(result.is_ok());
    }

    /// Property: Canonical serialization is deterministic
    #[test]
    fn prop_canonical_serialization_deterministic(
        tx_fixture in prop::sample::select(vec![
            include_str!("fixtures/btc_genesis_coinbase.hex"),
            include_str!("fixtures/btc_simple_p2pkh.hex"),
        ])
    ) {
        let tx_bytes = universal_decoder_core::hex::decode(tx_fixture.trim()).unwrap();
        let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();
        let tx_ir = decoded.canonicalize().unwrap();

        let bytes1 = tx_ir.to_canonical_bytes().unwrap();
        let bytes2 = tx_ir.to_canonical_bytes().unwrap();

        prop_assert_eq!(bytes1, bytes2);
    }
}
```

### Fuzzing Tests (Phase 2.1.10, Day 14)

**Setup**:
```bash
cd crates/decoder-bitcoin
cargo fuzz init
```

**File**: `crates/decoder-bitcoin/fuzz/fuzz_targets/fuzz_decode.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use decoder_bitcoin::BitcoinDecoder;
use universal_decoder_core::prelude::*;

fuzz_target!(|data: &[u8]| {
    // Should never panic, even on completely random input
    let _ = BitcoinDecoder::decode(data);
});
```

**Run fuzzing**:
```bash
cargo fuzz run fuzz_decode -- -max_len=100000 -timeout=30
```

## Test Fixtures Required

### Real Bitcoin Transactions

Create these test fixtures in `crates/decoder-bitcoin/tests/fixtures/`:

1. **btc_genesis_coinbase.hex** ✅ (Already exists)
   - Bitcoin genesis block coinbase transaction
   - TXID: `4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b`

2. **btc_simple_p2pkh.hex** ✅ (Already exists)
   - Simple P2PKH transaction (Satoshi → Hal Finney)

3. **btc_segwit_p2wpkh.hex** (NEW)
   - SegWit P2WPKH transaction
   - First SegWit transaction on mainnet

4. **btc_segwit_p2wsh.hex** (NEW)
   - SegWit P2WSH (multisig) transaction

5. **btc_taproot.hex** (NEW)
   - Taproot (P2TR) transaction
   - From block 709,632 (first Taproot activation)

6. **btc_large_tx.hex** (NEW)
   - Transaction with 100+ inputs/outputs
   - Tests performance and memory handling

7. **btc_multisig_p2sh.hex** (NEW)
   - P2SH multisig transaction

### Malformed Transactions (for error testing)

8. **btc_truncated.hex** (NEW)
   - Incomplete transaction (truncated)

9. **btc_invalid_varint.hex** (NEW)
   - Invalid VarInt encoding

10. **btc_oversized_script.hex** (NEW)
    - Script exceeding MAX_SCRIPT_SIZE

## Dependency Changes

### Update Cargo.toml

**File**: `crates/decoder-bitcoin/Cargo.toml`

```toml
[package]
name = "decoder-bitcoin"
version = "0.1.0"
edition = "2021"

[dependencies]
# ONLY universal-decoder-core in production dependencies
universal-decoder-core = { path = "../universal-decoder-core" }

[dev-dependencies]
# Bitcoin crate ONLY for validation in tests
bitcoin = "0.31"

# Testing frameworks
proptest = "1.4"
hex-literal = "0.4"
```

**Verify**:
```bash
cargo tree -p decoder-bitcoin -e normal --depth 1
# Should show ONLY universal-decoder-core
```

## Success Criteria

Phase 2.1 is complete when:

- ✅ **Pure Rust implementation**: No `bitcoin` crate in production dependencies
- ✅ **All parsers implemented**: VarInt, input, output, witness, transaction
- ✅ **Validation tests pass**: All tests compare favorably with `bitcoin` crate
- ✅ **Unit test coverage**: > 90%
- ✅ **Integration tests**: All test fixtures decode correctly
- ✅ **Property tests**: 20+ property tests passing
- ✅ **Fuzz testing**: No panics after 1 hour of fuzzing
- ✅ **Canonical serialization**: Deterministic Borsh encoding
- ✅ **Documentation**: All public functions documented
- ✅ **CI passing**: All tests pass in CI

## Timeline Summary

| Day | Task | Deliverable |
|-----|------|-------------|
| 1-2 | Core parsing utilities | VarInt, primitive readers, tests |
| 3-4 | Input/output parsers | TxInput, TxOutput parsing, tests |
| 5 | Witness parser | Witness parsing, tests |
| 6-7 | Transaction parser | Full transaction decoding |
| 8 | BitcoinTransaction methods | TXID, validation, helpers |
| 9 | Canonicalization | Updated TxIR conversion |
| 10 | Unit tests | Comprehensive unit test suite |
| 11-12 | Integration tests | Validation against `bitcoin` crate |
| 13 | Property tests | Property-based tests |
| 14 | Fuzzing & polish | Fuzz testing, documentation |

**Total**: 14 days (2 weeks)

## Next Steps

After Phase 2.1 completion:
1. Review and merge PR
2. Proceed to Phase 2.2: Ethereum Pure Rust Decoder
3. Update ROADMAP.md with progress

## References

- [Learn Me A Bitcoin: Transactions](https://learnmeabitcoin.com/technical/transaction/)
- [BIP 141: Segregated Witness](https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki)
- [BIP 144: SegWit Serialization](https://github.com/bitcoin/bips/blob/master/bip-0144.mediawiki)
- [BIP 341: Taproot](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki)
- [Bitcoin Developer Reference](https://developer.bitcoin.org/reference/transactions.html)

---

**Status**: ✅ Planning Complete - Ready for Implementation
**Next**: Begin Day 1 implementation (VarInt parser)
