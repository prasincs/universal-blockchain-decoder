# Bitcoin Decoder Fuzzing

This directory contains fuzz targets for the Bitcoin transaction decoder using `cargo-fuzz`.

## Fuzz Targets

### 1. `fuzz_bitcoin_decoder`

Comprehensive fuzzing of the entire Bitcoin decoder pipeline:
- Tests that decode never panics on arbitrary input
- Tests canonicalization safety
- Tests property accessors (version, txid, fees, etc.)
- Tests DoS protection (large inputs)

**Run**:
```bash
cd crates/decoder-bitcoin
cargo fuzz run fuzz_bitcoin_decoder
```

### 2. `fuzz_bitcoin_varint`

Focused fuzzing of VarInt encoding/decoding:
- Tests varint parsing never panics
- Tests roundtrip property (encode/decode identity)
- Tests non-canonical varint detection
- Tests truncated input handling

**Run**:
```bash
cd crates/decoder-bitcoin
cargo fuzz run fuzz_bitcoin_varint
```

### 3. `fuzz_bitcoin_txid`

Focused fuzzing of TXID calculation:
- Tests TXID determinism (same input → same output)
- Tests TXID length (always 32 bytes)
- Tests SegWit TXID calculation (excludes witness)
- Tests coinbase TXID calculation

**Run**:
```bash
cd crates/decoder-bitcoin
cargo fuzz run fuzz_bitcoin_txid
```

## Quick Start

### Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

### Run all fuzz targets (short test)

```bash
cd crates/decoder-bitcoin

# Run each target for 60 seconds
cargo fuzz run fuzz_bitcoin_decoder -- -max_total_time=60
cargo fuzz run fuzz_bitcoin_varint -- -max_total_time=60
cargo fuzz run fuzz_bitcoin_txid -- -max_total_time=60
```

### Run continuous fuzzing (nightly CI)

```bash
# Run for 1 hour (typical nightly CI duration)
cargo fuzz run fuzz_bitcoin_decoder -- -max_total_time=3600
```

### Minimize crash inputs

If a crash is found:

```bash
# Minimize the crashing input
cargo fuzz cmin fuzz_bitcoin_decoder

# Triage artifacts
cargo fuzz tmin fuzz_bitcoin_decoder artifacts/fuzz_bitcoin_decoder/crash-<hash>
```

## Coverage

To measure fuzzing coverage:

```bash
# Build with coverage instrumentation
cargo fuzz coverage fuzz_bitcoin_decoder

# Generate HTML report
cargo cov -- show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_bitcoin_decoder \
    --format=html -instr-profile=coverage/fuzz_bitcoin_decoder/coverage.profdata \
    > coverage.html
```

## Integration with CI/CD

The fuzz targets are integrated into the nightly CI pipeline:

- **Nightly fuzzing**: `.github/workflows/nightly.yml` runs all fuzz targets for 1 hour
- **Crash detection**: Any crashes fail the CI build
- **Corpus growth**: Interesting inputs are saved to the corpus

## What Fuzzing Catches

Fuzzing is particularly effective at finding:

1. **Panics**: Uncaught unwrap(), out-of-bounds access, arithmetic overflow
2. **Memory safety**: Buffer overflows, use-after-free (in unsafe code)
3. **Logic errors**: Edge cases in parsing, validation bypasses
4. **DoS vulnerabilities**: Inputs that cause excessive memory/CPU usage

## Best Practices

- **Run locally before PR**: Fuzz for at least 5-10 minutes
- **Corpus management**: Keep corpus directory in .gitignore
- **Artifact handling**: Commit crash reproducers as integration tests
- **Coverage-guided**: libfuzzer uses coverage feedback to explore code paths

## Expected Results

With the current Bitcoin decoder implementation:

- ✅ **No panics expected**: Pure Rust implementation with bounds checking
- ✅ **Fast execution**: ~100-1000 executions per second typical
- ✅ **High coverage**: Should reach 80-90% code coverage within hours
- ✅ **Corpus growth**: Interesting inputs accumulate over time

## Troubleshooting

### Slow fuzzing

```bash
# Use more cores
cargo fuzz run fuzz_bitcoin_decoder -- -jobs=4
```

### Out of memory

```bash
# Limit memory usage (2GB)
cargo fuzz run fuzz_bitcoin_decoder -- -rss_limit_mb=2048
```

### Timeout issues

```bash
# Reduce timeout per input (1 second)
cargo fuzz run fuzz_bitcoin_decoder -- -timeout=1
```

## References

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Bitcoin decoder implementation](../src/lib.rs)
- [Property tests](../tests/property_tests.rs)
