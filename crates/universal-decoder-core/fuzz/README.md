# Fuzz Testing

This directory contains fuzz targets for the universal-decoder-core crate using `cargo-fuzz` and `libFuzzer`.

## Setup

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

## Fuzz Targets

### fuzz_canonical

Tests canonical serialization/deserialization of core types:
- TxMetadata
- Amount
- Address

Ensures:
- No panics on arbitrary input
- Deterministic serialization
- Valid round-trip behavior

```bash
cargo fuzz run fuzz_canonical
```

### fuzz_amount_ops

Tests Amount arithmetic operations:
- `checked_add` never panics
- `checked_sub` never panics
- Overflow is handled correctly
- Equality works correctly

```bash
cargo fuzz run fuzz_amount_ops
```

### fuzz_borsh_serialization

Tests Borsh serialization round-trips:
- Arbitrary data deserialization doesn't panic
- Successful deserializations can be re-serialized
- Round-trip preserves data integrity

```bash
cargo fuzz run fuzz_borsh_serialization
```

## Running Fuzz Tests

### Quick test (1 minute)
```bash
cargo fuzz run fuzz_canonical -- -max_total_time=60
```

### Extended test (1 hour, used in nightly CI)
```bash
cargo fuzz run fuzz_canonical -- -max_total_time=3600
```

### Run with specific options
```bash
cargo fuzz run fuzz_canonical -- \
    -max_len=100000 \
    -timeout=30 \
    -rss_limit_mb=8192 \
    -use_value_profile=1
```

## Handling Crashes

If a fuzz target finds a crash, the input will be saved to `fuzz/artifacts/`:

```bash
# Reproduce the crash
cargo fuzz run fuzz_canonical fuzz/artifacts/fuzz_canonical/crash-xxx

# Minimize the crash input
cargo fuzz cmin fuzz_canonical

# Add as regression test
cp fuzz/artifacts/fuzz_canonical/crash-xxx ../tests/regression/
```

## Corpus

The corpus (interesting inputs discovered during fuzzing) is stored in:
- `fuzz/corpus/fuzz_canonical/`
- `fuzz/corpus/fuzz_amount_ops/`
- `fuzz/corpus/fuzz_borsh_serialization/`

These are automatically maintained by libFuzzer and should be committed to git.

## Coverage

Generate coverage report from fuzzing:

```bash
cargo fuzz coverage fuzz_canonical
```

## Integration with CI

Fuzz tests run nightly in GitHub Actions (`.github/workflows/nightly.yml`):
- 1 hour fuzzing session
- Artifacts uploaded on crash
- Coverage tracking

## Best Practices

1. **Add fuzz targets for new code**: Any new parser, deserializer, or arithmetic should have a fuzz target
2. **Commit interesting corpus**: When fuzzing locally finds good inputs, commit them
3. **Minimize before committing**: Use `cargo fuzz cmin` to reduce corpus size
4. **Write regression tests**: Convert crashes to unit tests

## Resources

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing Rust code guide](https://rust-fuzz.github.io/book/)
