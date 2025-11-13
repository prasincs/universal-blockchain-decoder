# decoder-generator

**A scaffolding tool for bootstrapping blockchain decoders (one-time use only)**

## What This Is

Like `cargo new` but for blockchain decoders. Generates initial code skeleton, then **you own the code**.

```
Config → Generate once → You maintain the code
         (bootstrap)     (never regenerate)
```

## Quick Start

```bash
# Generate from TOML spec
cargo run -p decoder-generator -- generate specs/dogecoin.toml

# Interactive mode (stub - not implemented yet)
cargo run -p decoder-generator -- interactive
```

## Important: One-Time Generation Only!

⚠️ **After generation, YOU OWN THE CODE**
- Never regenerate (you'll lose changes)
- Code is source of truth, not the spec
- Spec file becomes documentation only

## Better Long-Term Approach

See `ARCHITECTURE.md` for the recommended **trait-based extension**:

```rust
// Instead of generating code, configure generic decoder
impl UtxoChainConfig for Litecoin {
    const CHAIN_ID: u64 = 2;
    const HAS_SEGWIT: bool = true;
    type HashAlgorithm = DoubleSha256;
}

pub type LitecoinDecoder = UtxoDecoder<Litecoin>;
```

## When to Use This Tool

✅ **Use when:** Bootstrapping a totally new chain family
❌ **Don't use:** Chain similar to existing one (just copy it)
❌ **Never:** Regenerate existing code

## See Also

- `ARCHITECTURE.md` - Why traits > specs, long-term plan
- `examples/trait_based_approach.rs` - Trait-based alternative
