# Universal Transaction Decoder CLI

A unified command-line tool for decoding raw blockchain transactions from any supported blockchain.

## Installation

```bash
cargo build --release --bin universal-tx-decoder
```

## Usage

```bash
universal-tx-decoder --chain <CHAIN> [OPTIONS] <HEX_STRING>
universal-tx-decoder --chain <CHAIN> [OPTIONS] --file <FILE>
universal-tx-decoder --chain <CHAIN> [OPTIONS] --stdin
```

### Required Arguments

- `--chain <CHAIN>` - Blockchain to decode (bitcoin, ethereum)

### Options

- `-h, --help` - Show help message
- `-c, --canonical` - Show canonical IR representation
- `-f, --file <FILE>` - Read transaction from file
- `--stdin` - Read transaction from stdin

### Supported Chains

- `bitcoin`, `btc` - Bitcoin blockchain (✓ implemented)
- `ethereum`, `eth` - Ethereum blockchain (coming soon)

## Examples

### Decode Bitcoin Transaction

```bash
# From hex string
cargo run --bin universal-tx-decoder -- --chain bitcoin 0100000001...

# From file
cargo run --bin universal-tx-decoder -- --chain bitcoin --file transaction.hex

# With canonical IR
cargo run --bin universal-tx-decoder -- --chain bitcoin --canonical 0100000001...
```

### Pipe from Bitcoin Core

```bash
bitcoin-cli getrawtransaction <txid> | cargo run --bin universal-tx-decoder -- --chain bitcoin --stdin
```

### Decode Ethereum Transaction (Coming Soon)

```bash
cargo run --bin universal-tx-decoder -- --chain ethereum f86c0a8504a817c800825208...
```

## Output

### Bitcoin Transaction Details

The CLI outputs:
- Transaction ID (TXID)
- Version
- Locktime
- SegWit detection
- Coinbase detection
- Input details (previous TXID, output index, script length, sequence, witness data)
- Output details (value in satoshis and BTC, script length, script type detection)
- Total output value

### Script Type Detection

The CLI automatically detects common Bitcoin script types:
- P2PKH (Pay-to-PubKey-Hash)
- P2SH (Pay-to-Script-Hash)
- P2WPKH (Pay-to-Witness-PubKey-Hash)
- P2WSH (Pay-to-Witness-Script-Hash)
- P2TR (Taproot)
- P2PK (Pay-to-PubKey)

### Canonical IR Representation

With `--canonical` flag, the CLI also outputs:
- Canonical hash (deterministic hash of the transaction)
- Canonical size
- Operations (Transfer, ContractCall, ContractDeploy, Stake, Generic)
- State deltas (inputs consumed, outputs created)

## Example Output

```
=== Universal Blockchain Transaction Decoder ===

Chain:                  BITCOIN
Raw transaction size:   204 bytes
Hex preview:            0100000001000000000000000000000000000000000000000000000000000000...

=== Bitcoin Transaction Details ===
TXID:           3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a
Version:        1
Locktime:       0
SegWit:         false
Coinbase:       true

=== Inputs (1) ===
Input #0:
  Previous TXID:  0000000000000000000000000000000000000000000000000000000000000000
  Output Index:   4294967295
  Script Length:  77 bytes
  Sequence:       0xffffffff

=== Outputs (1) ===
Output #0:
  Value:          5000000000 satoshis (50.00000000 BTC)
  Script Length:  67 bytes
  Script Type:    P2PK

=== Summary ===
Total Output:   5000000000 satoshis (50.00000000 BTC)
```

## Architecture

The CLI is built on top of the universal-blockchain-decoder library:

1. **Universal Interface**: Single CLI for all blockchain decoders
2. **Extensible**: Easy to add new chains
3. **Type-Safe**: Rust's type system ensures correctness
4. **Pure Rust**: No external blockchain dependencies in production
5. **Canonical IR**: Unified intermediate representation for all chains

## Development

### Adding a New Chain

1. Implement decoder in `crates/decoder-<chain>/`
2. Add decoder to `Cargo.toml` dependencies
3. Import decoder in `src/bin/universal-tx-decoder.rs`
4. Add match case in `run()` function
5. Add chain-specific printing function

### Testing

```bash
# Run built-in tests
cargo test --bin universal-tx-decoder

# Test with fixture
cargo run --bin universal-tx-decoder -- --chain bitcoin --file crates/decoder-bitcoin/tests/fixtures/btc_genesis_coinbase.hex
```

## Roadmap

- [x] Bitcoin transaction decoding
- [x] Canonical IR representation
- [ ] Ethereum transaction decoding
- [ ] JSON output format
- [ ] Batch decoding from file
- [ ] Signature validation
- [ ] Fee calculation (with UTXO lookup)
- [ ] Script decompilation
- [ ] Address extraction

## See Also

- [ROADMAP.md](ROADMAP.md) - Project roadmap and phases
- [CLAUDE.md](CLAUDE.md) - Core library architecture
- [TESTING_STRATEGY.md](docs/TESTING_STRATEGY.md) - Testing approach
