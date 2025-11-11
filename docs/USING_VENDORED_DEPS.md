# Using Vendored Dependencies in Decoder Crates

## Overview

The `universal-decoder-core` crate vendors certain dependencies (like `hex`) to reduce the total dependency count while making them available to all decoder crates.

## Architecture

```
┌─────────────────────────────────────┐
│  universal-decoder-core             │
│  ├── src/vendored/hex/              │  ← Vendored source code
│  └── src/lib.rs                     │  ← Re-exports: pub use vendored::hex
└─────────────────────────────────────┘
                ▲
                │ depends on
                │
┌───────────────┴─────────────────────┐
│  decoder-bitcoin                    │
│  Uses: universal_decoder_core::hex  │  ← Uses vendored hex
└─────────────────────────────────────┘
```

## Implementation

### Step 1: Core Re-exports Vendored Dependencies

```rust
// crates/universal-decoder-core/src/lib.rs

// Internal vendored dependencies
mod vendored {
    pub mod hex;
    // Future vendored crates go here
}

// Public re-exports (decoders can use these)
pub use vendored::hex;

// Rest of core API
pub mod ir;
pub mod traits;
pub mod canonical;
pub mod error;
// ...

pub mod prelude {
    pub use crate::hex;  // Include in prelude for convenience
    pub use crate::ir::*;
    pub use crate::traits::*;
    // ...
}
```

### Step 2: Decoder Uses Re-exported Dependency

```rust
// crates/decoder-bitcoin/src/lib.rs

// Option 1: Import directly
use universal_decoder_core::hex;

pub fn display_txid(txid: &[u8]) -> String {
    hex::encode(txid)
}

// Option 2: Use via prelude
use universal_decoder_core::prelude::*;

pub fn display_address(pubkey_hash: &[u8]) -> String {
    hex::encode(pubkey_hash)
}
```

### Step 3: Decoder Cargo.toml

```toml
# crates/decoder-bitcoin/Cargo.toml

[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
# That's it! hex is transitively available

[dev-dependencies]
# Optional: Keep external hex for validation tests
hex = "0.4.3"  # For comparing our vendored version
```

## Pattern for Multiple Vendored Dependencies

As we vendor more dependencies, the pattern scales:

```rust
// crates/universal-decoder-core/src/lib.rs

mod vendored {
    pub mod hex;
    // Future:
    // pub mod smallvec;  (if we vendor it)
}

// Public API
pub use vendored::hex;
// pub use vendored::smallvec;  (future)

// Convenience prelude
pub mod prelude {
    pub use crate::hex;
    // pub use crate::smallvec;
    pub use crate::ir::*;
    pub use crate::traits::*;
}
```

## Why This Approach?

### ✅ Advantages

1. **Single Source of Truth**
   - Only core vendors dependencies
   - Decoders automatically get updates

2. **Consistent Versions**
   - All decoders use the same vendored code
   - No version conflicts

3. **Minimal Duplication**
   - Vendor once, use everywhere
   - Reduces repository size

4. **Clear Ownership**
   - Core owns all common utilities
   - Decoders focus on chain-specific logic

5. **Easy Migration**
   - Just change core's re-export
   - All decoders automatically updated

### ❌ When NOT to Use This Pattern

Don't use this pattern if:

- **Decoder needs different version**: Then decoder should vendor separately
- **Dependency is decoder-specific**: Vendor in the decoder crate instead
- **Circular dependency**: Core can't depend on decoder-specific code

## Example: Decoder-Specific Vendoring

If a decoder needs a dependency that core doesn't use:

```rust
// crates/decoder-solana/src/vendored/
//   └── bincode/  ← Vendored just for Solana decoder

// crates/decoder-solana/src/lib.rs
mod vendored {
    pub mod bincode;  // Only Solana uses this
}
use vendored::bincode;
```

## Testing Strategy

### Validate Vendored Version Matches Upstream

```rust
// crates/universal-decoder-core/tests/vendored_hex_validation.rs

#[cfg(test)]
mod validation {
    // External hex (from dev-dependencies)
    use hex as external_hex;

    // Our vendored hex (re-exported from core)
    use universal_decoder_core::hex as vendored_hex;

    #[test]
    fn test_encode_matches() {
        let data = b"Hello, world!";

        let external = external_hex::encode(data);
        let vendored = vendored_hex::encode(data);

        assert_eq!(external, vendored);
    }

    #[test]
    fn test_decode_matches() {
        let hex_str = "48656c6c6f2c20776f726c6421";

        let external = external_hex::decode(hex_str).unwrap();
        let vendored = vendored_hex::decode(hex_str).unwrap();

        assert_eq!(external, vendored);
    }

    #[test]
    fn test_roundtrip() {
        let data = b"Test data for roundtrip";

        let encoded = vendored_hex::encode(data);
        let decoded = vendored_hex::decode(&encoded).unwrap();

        assert_eq!(data, decoded.as_slice());
    }
}
```

### Decoder Tests Can Also Validate

```rust
// crates/decoder-bitcoin/tests/integration.rs

#[cfg(test)]
mod tests {
    use universal_decoder_core::hex;  // Vendored version

    #[test]
    fn test_txid_formatting() {
        let txid = [0xde, 0xad, 0xbe, 0xef];
        let formatted = hex::encode(&txid);
        assert_eq!(formatted, "deadbeef");
    }
}
```

## Migration Checklist

When vendoring a new dependency in core:

- [ ] Vendor the crate in `src/vendored/<crate>/`
- [ ] Add `mod <crate>` to `src/vendored/mod.rs` (if exists)
- [ ] Re-export at crate root: `pub use vendored::<crate>`
- [ ] Add to prelude: `pub use crate::<crate>` in `prelude` module
- [ ] Write validation tests comparing with external version
- [ ] Update all decoder imports (search/replace)
- [ ] Remove external dependency from decoder Cargo.toml files
- [ ] Keep in core's `[dev-dependencies]` for validation
- [ ] Run `cargo test --all` to verify
- [ ] Update documentation

## Common Patterns

### Pattern 1: Simple Re-export

```rust
// For simple vendored crates
pub use vendored::hex;
```

### Pattern 2: Selective Re-export

```rust
// Only expose specific items
pub use vendored::hex::{encode, decode, FromHexError};
// Don't expose: hex::encode_upper (if we don't need it)
```

### Pattern 3: Wrapped API

```rust
// Provide a higher-level API wrapping vendored code
pub mod hex {
    // Re-export most things
    pub use crate::vendored::hex::{encode, decode, FromHexError};

    // Add convenience methods
    pub fn encode_txid(txid: &[u8; 32]) -> String {
        encode(txid)
    }
}
```

## FAQ

### Q: Can decoders vendor their own dependencies?

**A**: Yes! If a dependency is decoder-specific (not used by core or other decoders), vendor it in the decoder crate:

```
crates/decoder-solana/
  └── src/
      └── vendored/
          └── bincode/  ← Solana-specific vendoring
```

### Q: What if a decoder needs a different version?

**A**: Vendor that version separately in the decoder. But try to avoid this - it defeats the purpose of shared vendoring.

### Q: How do we update a vendored dependency?

**A**: Update in core, run validation tests, and all decoders automatically get the update. See `docs/VENDORING_GUIDE.md` for update process.

### Q: Can we vendor dependencies with dependencies?

**A**: **Avoid it**. Only vendor dependencies with zero or minimal transitive dependencies. If a crate has many dependencies, don't vendor it.

### Q: What about proc-macro crates?

**A**: **Don't vendor proc-macros**. They must be compiled separately and can't be vendored like regular code. Keep them as external dependencies.

## Summary

**Pattern**: Vendor once in core, re-export for all decoders

**Benefits**:
- Single source of truth
- Consistent versions
- Minimal duplication
- Easy to update

**Structure**:
```rust
// Core
mod vendored { pub mod hex; }
pub use vendored::hex;

// Decoders
use universal_decoder_core::hex;
```

**Result**: All decoder crates can use vendored dependencies through core without duplicating vendoring effort.

---

**See Also**:
- `docs/VENDORING_GUIDE.md` - Detailed vendoring process
- `docs/DEPENDENCY_AUDIT.md` - Which dependencies to vendor
- `docs/TESTING_STRATEGY.md` - Testing vendored code
