# TON (The Open Network) Decoder

Pure Rust implementation of a TON blockchain transaction decoder for the universal-blockchain-decoder project.

## Features

✅ **Implemented**:
- BoC (Bag of Cells) format validation
- Magic number detection (standard, idx, crc32c variants)
- Flags byte parsing (has_idx, has_crc32c, has_cache_bits)
- Basic cell structure parsing
- Transaction type definitions
- TxIR (Transaction Intermediate Representation) conversion
- Integration with `tonlib-core` for validation testing

🚧 **In Progress**:
- Complete BoC cell parsing for multi-cell structures
- Full TL-B transaction schema parsing
- Message parsing (in_msg, out_msgs)
- Operation extraction from messages

## Architecture

### Chain Identity
- **Chain ID**: 607 (SLIP-44 coin type for TON)
- **Chain Family**: Account model (message-passing actor architecture)
- **Signature Scheme**: EdDsa (standard for TON)
- **Network**: Mainnet

### Dependencies
- **Production**: Zero external blockchain libraries
  - `universal-decoder-core`: Core types and traits
  - `decoder-primitives`: Byte reading utilities
- **Dev-only**: `tonlib-core` for validation testing

## Testing

```bash
# Run basic unit tests (all passing)
cargo test -p decoder-ton

# Run validation tests against tonlib-core
cargo test -p decoder-ton test_validate_against_tonlib

# Run real mainnet transaction tests (currently ignored - WIP)
cargo test -p decoder-ton --test real_transactions -- --ignored
```

## TON Transaction Format

TON uses a unique cell-based data structure:

### Bag of Cells (BoC) Format

```
┌─────────────────────────────────────────────────────────┐
│ Magic (4 bytes)                                         │
│   0xb5ee9c72 (standard)                                 │
│   0x68ff65f3 (with index)                               │
│   0xacc3a728 (with CRC32C)                              │
├─────────────────────────────────────────────────────────┤
│ Flags (1 byte)                                          │
│   has_idx | has_crc32c | has_cache_bits | flags | size │
├─────────────────────────────────────────────────────────┤
│ OffBytes (1 byte) - offset size (1-8)                   │
│ CellsCount (OffBytes) - number of cells                │
│ RootsCount (OffBytes) - number of root cells           │
│ AbsentCount (OffBytes) - number of absent cells        │
│ TotCellsSize (OffBytes) - total cells data size        │
│ RootList (RootsCount * OffBytes) - root cell indices   │
│ [Index] (optional, if has_idx)                          │
│ CellData (TotCellsSize) - all cell data                │
│ [CRC32C] (optional 4 bytes, if has_crc32c)              │
└─────────────────────────────────────────────────────────┘
```

### Cell Format

Each cell consists of:
- **Descriptor** (2 bytes): d1, d2
  - `d1 = refs_count + 8*is_exotic + 32*level`
  - `d2 = bits encoding`
- **Data**: Up to 1023 bits
- **References**: Up to 4 references to other cells

### Transaction TL-B Schema

```
transaction$0111
  account_addr:bits256
  lt:uint64
  prev_trans_hash:bits256
  prev_trans_lt:uint64
  now:uint32
  outmsg_cnt:uint15
  orig_status:AccountStatus
  end_status:AccountStatus
  ^[in_msg:(Maybe ^(Message Any))]
  ^[out_msgs:(HashmapE 15 ^(Message Any))]
  total_fees:CurrencyCollection
  ^[state_update:^(HASH_UPDATE Account)]
  description:^TransactionDescr
  = Transaction;
```

## Current Limitations

1. **Multi-cell BoC parsing**: The current implementation has issues parsing complex BoC structures with multiple cells and references. This is being actively debugged against `tonlib-core`.

2. **Transaction parsing**: While the BoC structure is recognized, full TL-B transaction parsing is not yet complete.

3. **Message extraction**: In/out message parsing from cell references is not yet implemented.

## Real Mainnet Examples

The tests include real TON mainnet transactions from:
- TON Connect SDK examples
- TON documentation
- Live network data

Example BoC (base64):
```
te6cckEBBAEAOgACATQCAQAAART/APSkE/S88sgLAwBI0wHQ0wMBcbCRW+D6QDBwgBDIywVYzxYh+gLLagHPFsmAQPsAlxCarA==
```

## Contributing

The BoC parser needs work to correctly handle:
1. Cell descriptor parsing edge cases
2. Cell data boundaries
3. Cell reference resolution

Reference implementation: `tonlib-core` (used in tests for validation)

## Resources

- [TON Documentation](https://docs.ton.org/)
- [TON BoC Specification](https://docs.ton.org/develop/data-formats/cell-boc)
- [TL-B Language](https://docs.ton.org/v3/documentation/data-formats/tlb/tl-b-language)
- [tonlib-rs](https://github.com/ston-fi/tonlib-rs) - Reference Rust implementation

## License

MIT OR Apache-2.0 (same as parent project)
