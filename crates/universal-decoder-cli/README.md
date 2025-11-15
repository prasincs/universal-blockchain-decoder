# Universal Transaction Decoder CLI

A secure, multi-chain command-line tool for decoding raw blockchain transactions.

## Features

- **Multi-Chain Support**: Dynamic decoder registry supports 14+ blockchains
- **Privacy-Aware**: Special handling for privacy chains (Zcash, Monero)
- **Security-First**: No shell history pollution, memory protection, file permission validation
- **Flexible Input**: Accept transactions from hex, files, or stdin
- **Canonical IR**: Universal intermediate representation for cross-chain analysis

## Supported Chains

| Chain | Family | Privacy | Symbol | Chain ID |
|-------|--------|---------|--------|----------|
| Bitcoin | UTXO | No | btc | 0 |
| Litecoin | UTXO | No | ltc | 2 |
| Dogecoin | UTXO | No | doge | 3 |
| Dash | UTXO | No | dash | 5 |
| Bitcoin Cash | UTXO | No | bch | 145 |
| Bitcoin SV | UTXO | No | bsv | 236 |
| **Zcash** | **Privacy** | **Yes** | zec | 133 |
| Ethereum | Account | No | eth | 1 |
| BNB Smart Chain | Account | No | bnb | 56 |
| Polygon | Account | No | matic | 137 |
| Avalanche C-Chain | Account | No | avax | 43114 |
| Optimism | Account | No | op | 10 |
| Arbitrum One | Account | No | arb | 42161 |
| Solana | Instruction | No | sol | 900 |

## Installation

```bash
cargo build --release --package universal-decoder-cli
sudo cp target/release/universal-tx-decoder /usr/local/bin/
```

## Quick Start

### Basic Usage

```bash
# Decode a Bitcoin transaction
universal-tx-decoder -c btc 0100000001...

# Decode an Ethereum transaction
universal-tx-decoder -c eth f86c...

# List all supported chains
universal-tx-decoder --list-chains

# Show only privacy chains
universal-tx-decoder --list-privacy-chains
```

### Using Files (Recommended for Privacy)

```bash
# Read transaction from file
universal-tx-decoder -c btc -f transaction.hex

# Read from stdin (for piping)
cat transaction.hex | universal-tx-decoder -c eth --stdin
```

### Canonical IR Output

```bash
# Show universal intermediate representation
universal-tx-decoder -c btc -C transaction.hex
```

## Security Best Practices

### 1. Avoid Shell History Pollution ⚠️

**DON'T** paste sensitive data directly on command line:

```bash
# ❌ BAD: Visible in shell history (.bash_history, .zsh_history)
universal-tx-decoder -c zec 0400008085202f89abc123...
```

**DO** use files or stdin instead:

```bash
# ✅ GOOD: Read from file
universal-tx-decoder -c zec -f shielded_tx.hex

# ✅ GOOD: Pipe from stdin
cat shielded_tx.hex | universal-tx-decoder -c zec --stdin

# ✅ GOOD: Use environment variable for viewing keys
export VIEWING_KEY="abc123..."
universal-tx-decoder -c zec --viewing-key-env VIEWING_KEY -f tx.hex
```

### 2. Privacy Chain Support (Zcash)

For privacy chains, viewing keys enable decryption of shielded transactions.

**Secure Viewing Key Storage:**

```bash
# 1. Create key file with restricted permissions
echo "your_zcash_viewing_key_hex" > ~/.zcash/viewkey
chmod 600 ~/.zcash/viewkey  # Owner read/write only

# 2. Use with CLI
universal-tx-decoder -c zec \
  --viewing-key-file ~/.zcash/viewkey \
  --decrypt \
  -f shielded_transaction.hex
```

**File Permission Requirements:**
- Unix/Linux: File **must** have `0600` (rw-------) or `0400` (r--------)
- Tool will **reject** world-readable or group-readable key files
- This prevents accidental exposure of private viewing keys

**Viewing Key Types:**

```bash
# Zcash full viewing key (96 bytes, default)
universal-tx-decoder -c zec \
  --viewing-key-file key.bin \
  --viewing-key-type zcash-full \
  --decrypt -f tx.hex

# Zcash incoming viewing key (32 bytes)
universal-tx-decoder -c zec \
  --viewing-key-file ivk.bin \
  --viewing-key-type zcash-incoming \
  --decrypt -f tx.hex

# Monero view key (32 bytes)
universal-tx-decoder -c xmr \
  --viewing-key-file viewkey.bin \
  --viewing-key-type monero \
  --decrypt -f tx.hex
```

### 3. Memory Protection

The CLI uses `secrecy` and `zeroize` to protect sensitive data in memory:

- **Viewing keys** are wrapped in `Secret<Vec<u8>>` and zeroized on drop
- **Transaction hex** is sanitized after parsing
- **No logging** of sensitive data to stdout/stderr

### 4. Input Validation

All inputs are validated before processing:

```bash
# Hex validation
# - Must contain only hex characters (0-9, a-f, A-F)
# - Must have even length (2 chars per byte)
# - Automatically trimmed of whitespace

# File validation
# - Viewing key files: checked for secure permissions
# - Transaction files: validated as hex before decoding
```

### 5. Verbose Mode for Debugging

Use `-v` for security warnings:

```bash
universal-tx-decoder -c zec -v transaction.hex
```

Warnings shown in verbose mode:
- CLI argument usage (shell history risk)
- Viewing key provided for non-privacy chain
- Decrypt flag used without viewing key

## Advanced Usage

### Dynamic Chain Selection

```bash
# By name (case-insensitive)
universal-tx-decoder -c bitcoin ...
universal-tx-decoder -c BITCOIN ...

# By short name
universal-tx-decoder -c btc ...

# By chain ID
universal-tx-decoder -c 0 ...    # Bitcoin
universal-tx-decoder -c 1 ...    # Ethereum
universal-tx-decoder -c 133 ...  # Zcash
```

### Output Formats

```bash
# Human-readable (default)
universal-tx-decoder -c btc -o human transaction.hex

# JSON (for programmatic parsing)
universal-tx-decoder -c btc -o json transaction.hex

# Hex only (echo input)
universal-tx-decoder -c btc -o hex transaction.hex
```

### Combining Options

```bash
# Full example: Zcash shielded transaction with all options
universal-tx-decoder \
  --chain zec \
  --file shielded_tx.hex \
  --viewing-key-file ~/.zcash/mainnet_viewkey \
  --viewing-key-type zcash-full \
  --decrypt \
  --canonical \
  --output human \
  --verbose
```

## Environment Variables

The CLI respects the following environment variables:

```bash
# Viewing key (fallback if --viewing-key-file not provided)
export VIEWING_KEY="abc123..."

# Use in command
universal-tx-decoder -c zec --viewing-key-env VIEWING_KEY -f tx.hex
```

## Troubleshooting

### Permission Denied (Viewing Key File)

```
Error: Insecure file permissions for /path/to/viewkey: 644. Must be 0600 or 0400
```

**Fix:**
```bash
chmod 600 /path/to/viewkey
```

### Invalid Hex String

```
Error: Invalid hex string (contains non-hex characters)
```

**Fix:**
- Remove any whitespace, newlines, or non-hex characters
- Ensure even length (2 hex chars per byte)
- Use `tr -d '\n'` to remove newlines: `cat tx.hex | tr -d '\n' > tx_clean.hex`

### Unknown Chain

```
Error: Unknown chain: foo
```

**Fix:**
- Run `universal-tx-decoder --list-chains` to see supported chains
- Use exact name, short name, or chain ID

## Development

### Building from Source

```bash
git clone https://github.com/prasincs/universal-blockchain-decoder
cd universal-blockchain-decoder
cargo build --release --package universal-decoder-cli
```

### Running Tests

```bash
cargo test --package universal-decoder-cli
```

### Security Checklist for Contributors

When adding new features, ensure:

- [ ] No sensitive data logged to stdout/stderr
- [ ] File permission validation for any new key/secret inputs
- [ ] Input sanitization and validation
- [ ] Use `secrecy::Secret` for sensitive in-memory data
- [ ] Implement `Zeroize` for sensitive structs
- [ ] Document security considerations in function docs
- [ ] Add tests for input validation edge cases
- [ ] Verify no shell history pollution vectors

## Security Model

### Threat Model

**In Scope:**
- ✅ Shell history pollution (viewing keys, private txs)
- ✅ Insecure file permissions (world-readable keys)
- ✅ Memory dumps of sensitive data
- ✅ Input validation (malformed hex, overlength inputs)

**Out of Scope:**
- ❌ Key generation (use chain-specific tools)
- ❌ Transaction signing (decoding only)
- ❌ Network operations (no broadcasting)
- ❌ Chain state queries (no RPC calls)

### Trust Boundaries

1. **User Input**: All external input is untrusted and validated
2. **Viewing Keys**: Treated as high-sensitivity secrets
3. **Decoder Output**: Chain-specific decoders are sandboxed

### Audit Trail

- Decoder implementations: See `crates/decoder-*/src/`
- Security primitives: See `src/secure_input.rs`
- Registry logic: See `src/registry.rs`

## FAQ

### Q: Why can't I pass viewing keys as CLI arguments?

**A:** Command-line arguments are visible in:
- Shell history files (`~/.bash_history`, `~/.zsh_history`)
- Process listings (`ps aux`, `top`)
- System logs (audit logs, bash logging)

Use `--viewing-key-file` or `--viewing-key-env` instead.

### Q: Why do I need 0600 permissions on key files?

**A:** Prevents accidental exposure to other users on shared systems. If your key file is world-readable (`0644`), any user can read your viewing keys and decrypt your transactions.

### Q: Does this tool support transaction construction/signing?

**A:** No. This is a **decoding-only** tool. For transaction construction, use:
- Bitcoin: `bitcoin` crate, Bitcoin Core, BDK
- Ethereum: `ethers-rs`, `alloy`, `web3`
- Zcash: `zcash-client-backend`, `librustzcash`

### Q: Can I use this in production systems?

**A:** Yes, but:
1. Audit the decoder implementations for your specific chains
2. Test with known-good transactions from testnets
3. Implement additional logging/monitoring as needed
4. Review the security model above

### Q: How do I add support for a new chain?

**A:** See `docs/ADDING_NEW_CHAINS.md` (TODO)

## License

Dual-licensed under MIT and Apache 2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.

## Contributing

See `CONTRIBUTING.md` for guidelines.

## Support

- GitHub Issues: https://github.com/prasincs/universal-blockchain-decoder/issues
- Documentation: https://github.com/prasincs/universal-blockchain-decoder/tree/main/docs

## Credits

Built with:
- `clap` - Command-line argument parsing
- `secrecy` - Memory-protected secrets
- `zeroize` - Memory zeroization
- Universal Decoder Core - Chain-agnostic IR

---

**Security Notice:** This tool handles potentially sensitive blockchain data. Always verify you're using the latest version and report security issues privately to the maintainers.
