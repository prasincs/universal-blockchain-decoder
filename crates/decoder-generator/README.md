# decoder-generator

**Status**: 🚧 Early prototype - Architecture under review

## What This Is

A code generator for blockchain decoders. Currently explores TOML-based specs, but will likely evolve to **trait-based extension** (see `ARCHITECTURE.md`).

## The Problem with Specs

TOML specs have a fundamental sync issue:

```
spec.toml → generated code → developer edits → spec is now wrong
```

**Better approach**: Make the code itself configurable via traits, eliminating drift.

## Current Usage (TOML - Temporary)

```bash
# Generate a decoder skeleton from TOML spec
cargo run -p decoder-generator -- generate specs/dogecoin.toml

# Validate a spec
cargo run -p decoder-generator -- validate specs/litecoin.toml
```

## Future Direction (Trait-Based)

Instead of specs, use **configuration traits**:

```rust
// Adding Litecoin = 7 lines of config
pub struct Litecoin;

impl UtxoChainConfig for Litecoin {
    const CHAIN_ID: u64 = 2;
    const CHAIN_NAME: &'static str = "Litecoin";
    const HAS_SEGWIT: bool = true;
    type HashAlgorithm = DoubleSha256;
}

// That's it! Decoder is auto-generated
pub type LitecoinDecoder = UtxoDecoder<Litecoin>;
```

**Benefits**:
- ✅ Type-safe (won't compile if wrong)
- ✅ Can't drift (config IS the code)
- ✅ IDE support (autocomplete, refactoring)
- ✅ Zero-cost (monomorphized at compile time)

See `ARCHITECTURE.md` for full explanation and `examples/trait_based_approach.rs` for working example.

## Recommended Approach

**For new chains**:

1. **Short term**: Copy similar decoder (e.g., copy Bitcoin → Dogecoin)
2. **Medium term**: Use trait-based config (when implemented)
3. **Long term**: Proc macro DSL

**Don't rely on TOML specs** - they're for exploration only.

## Architecture Documents

- `ARCHITECTURE.md` - Why traits > TOML, migration plan
- `examples/trait_based_approach.rs` - Working trait-based example
- `specs/*.toml` - TOML examples (temporary, for exploration)

## Contributing

If you're adding a chain similar to an existing one:

1. **Copy the existing decoder** (e.g., Bitcoin)
2. **Change the constants** (chain ID, name, hash)
3. **Remove/modify features** you don't need (e.g., SegWit)
4. **Test against real transactions**

This is faster and safer than code generation from specs.

## Future Work

- [ ] Refactor Bitcoin decoder to be generic over `UtxoChainConfig`
- [ ] Create `decoder-families` crate with generic decoders
- [ ] Add Litecoin/Dogecoin as config-only implementations
- [ ] Design proc macro DSL for maximum convenience
- [ ] Deprecate TOML specs (or make them documentation-only)

## See Also

- Bitcoin decoder: `../decoder-bitcoin/` - Reference implementation
- Shared primitives: `../decoder-primitives/` - Reusable parsing
- Encodings: `../decoder-encodings/` - VarInt, RLP, etc.
