# Vendoring External Dependencies

## Overview

Vendoring (copying external crate source code into our repository) is a pragmatic approach to dependency management when:

1. **Reducing supply chain risk**: Code is frozen and audited once
2. **Minimizing dependency count**: No external dependency on crates.io
3. **Code is small and simple**: Easy to maintain and audit (~500-1000 LOC)
4. **License compatible**: MIT or Apache-2.0

## 🎯 Recommended Approach: Git Subtree

**Use `git subtree` instead of manual copying** for maximum verifiability:

✅ **Cryptographically verifiable** - Git history proves what was vendored
✅ **Easy to audit** - Anyone can verify against upstream
✅ **Updatable** - `git subtree pull` for updates
✅ **Transparent** - All modifications tracked in git

**See `docs/GIT_SUBTREE_VENDORING.md` for detailed implementation.**

Quick comparison:

| Method | Verifiable | Updatable | Complexity |
|--------|-----------|-----------|------------|
| **Git Subtree** ⭐ | ✅✅✅ | ✅ One command | Low |
| Manual Copy | ❌ | ❌ Manual | Very Low |
| Git Submodule | ✅ | ✅ | High |

## Vendored Dependencies

### `hex` crate (v0.4.3)

**Why vendor?**
- Small: ~686 LOC
- Simple: Pure Rust encoding/decoding
- No dependencies
- License: MIT OR Apache-2.0 ✅
- Stable API (hasn't changed in years)

**Repository**: https://github.com/KokaKiwi/rust-hex

**Vendoring Process**:

#### Step 1: Create vendored directory structure

```bash
cd crates/universal-decoder-core
mkdir -p src/vendored/hex
```

#### Step 2: Copy source files with attribution

```bash
# Copy the source files
cp ~/.cargo/registry/src/*/hex-0.4.3/src/lib.rs src/vendored/hex/lib.rs
cp ~/.cargo/registry/src/*/hex-0.4.3/src/error.rs src/vendored/hex/error.rs

# Copy license files for attribution
cp ~/.cargo/registry/src/*/hex-0.4.3/LICENSE-MIT src/vendored/hex/
cp ~/.cargo/registry/src/*/hex-0.4.3/LICENSE-APACHE src/vendored/hex/
```

#### Step 3: Add attribution header

Add to `src/vendored/hex/README.md`:

```markdown
# Vendored: hex v0.4.3

This directory contains a vendored copy of the `hex` crate v0.4.3.

**Original Source**: https://github.com/KokaKiwi/rust-hex
**Original Crate**: https://crates.io/crates/hex
**License**: MIT OR Apache-2.0
**Vendored Date**: 2025-11-13
**Reason**: Reduce dependency count while maintaining functionality

## Changes from Original

- Removed `serde` feature (not needed)
- File structure: Kept `lib.rs` and `error.rs` only

## Original Copyright

Copyright (c) 2013-2014 The Rust Project Developers.
Copyright (c) 2015-2020 The rust-hex Developers.

Licensed under the Apache License, Version 2.0 or the MIT license,
at your option.

## Updating

If a security issue is found in the upstream `hex` crate:

1. Check if it affects our vendored version
2. If yes, update the vendored code with the fix
3. Document the change in this README
4. Update tests
```

#### Step 4: Integrate into crate structure

Create `src/vendored/hex/mod.rs`:

```rust
// Vendored from: https://github.com/KokaKiwi/rust-hex (v0.4.3)
// License: MIT OR Apache-2.0
//
// This is a vendored copy to reduce external dependencies.
// See src/vendored/hex/README.md for details.

mod error;

// Re-export only what we need (stripped down version)
pub use self::error::FromHexError;

// Include the implementation
include!("lib.rs");
```

Update `src/lib.rs`:

```rust
// Use vendored hex instead of external dependency
mod vendored {
    pub mod hex;
}

// Re-export at crate level for convenience
pub use vendored::hex;
```

#### Step 5: Remove external dependency

```diff
# crates/universal-decoder-core/Cargo.toml

[dependencies]
serde = "1.0"
borsh = "1.3"
thiserror = "1.0"
sha2 = "0.10"
sha3 = "0.10"
-hex = "0.4"  # REMOVED: Now vendored internally

[dev-dependencies]
proptest = "1.4"
```

#### Step 6: Update imports throughout codebase

```bash
# Search and replace all uses of the hex crate
# Before: use hex;
# After:  use crate::hex; (or universal_decoder_core::hex in other crates)

# Can use this script:
find . -name "*.rs" -type f -exec sed -i 's/^use hex::/use crate::hex::/g' {} \;
find . -name "*.rs" -type f -exec sed -i 's/extern crate hex;/\/\/ hex is now vendored internally/g' {} \;
```

#### Step 7: Test thoroughly

```bash
# Run all tests to ensure nothing broke
cargo test --all

# Check that there are no references to external hex crate
cargo tree | grep -i "hex " && echo "ERROR: Still depends on external hex!" || echo "SUCCESS: hex vendored"
```

## Vendoring Workflow

### Minimal Changes Philosophy

When vendoring, apply the **minimal changes** principle:

1. ✅ **Keep original code intact** as much as possible
2. ✅ **Remove unused features** (e.g., serde support we don't need)
3. ✅ **Add clear attribution** in README and comments
4. ✅ **Document any modifications** explicitly

### File Structure

```
crates/universal-decoder-core/
├── src/
│   ├── vendored/
│   │   ├── hex/
│   │   │   ├── README.md           # Vendoring information & attribution
│   │   │   ├── LICENSE-MIT         # Original MIT license
│   │   │   ├── LICENSE-APACHE      # Original Apache license
│   │   │   ├── mod.rs              # Module declaration & integration
│   │   │   ├── lib.rs              # Original hex/src/lib.rs (minimal changes)
│   │   │   └── error.rs            # Original hex/src/error.rs
│   │   └── mod.rs                  # Declares vendored submodules
│   ├── lib.rs
│   └── ...
└── Cargo.toml
```

### Changes to Apply to Vendored `hex`

**Minimal modifications for our use case:**

1. **Remove serde feature** (we don't use it):
   ```diff
   # In lib.rs
   -#[cfg(feature = "serde")]
   -mod serde;
   ```

2. **Keep no_std support** (beneficial for formal verification):
   ```rust
   #![cfg_attr(not(feature = "std"), no_std)]
   ```

3. **Add internal attribution**:
   ```rust
   // At top of lib.rs
   //! # Vendored: hex v0.4.3
   //!
   //! This is a vendored copy of the `hex` crate to reduce external dependencies.
   //! Original: https://github.com/KokaKiwi/rust-hex
   //! License: MIT OR Apache-2.0
   ```

## Testing Strategy for Vendored Code

### 1. Keep original tests

Copy the tests from the original crate:

```bash
# Copy test files
cp ~/.cargo/registry/src/*/hex-0.4.3/tests/*.rs crates/universal-decoder-core/tests/vendored_hex/
```

### 2. Add regression tests

Ensure our vendored version matches the behavior of the original:

```rust
// crates/universal-decoder-core/tests/vendored_hex_validation.rs

#[test]
fn test_vendored_hex_matches_reference() {
    // Test with external hex crate (in dev-dependencies for comparison)
    #[cfg(test)]
    {
        use hex as external_hex;
        use universal_decoder_core::hex as vendored_hex;

        let data = b"Hello, world!";

        let external_encoded = external_hex::encode(data);
        let vendored_encoded = vendored_hex::encode(data);

        assert_eq!(external_encoded, vendored_encoded);

        let external_decoded = external_hex::decode(&external_encoded).unwrap();
        let vendored_decoded = vendored_hex::decode(&vendored_encoded).unwrap();

        assert_eq!(external_decoded, vendored_decoded);
    }
}
```

**Note**: Keep `hex = "0.4.3"` in `[dev-dependencies]` temporarily to validate our vendored version matches!

### 3. Security audit tracking

Document in `SECURITY.md`:

```markdown
## Vendored Dependencies

| Dependency | Version | Audit Date | Next Audit | Notes |
|------------|---------|------------|------------|-------|
| hex | 0.4.3 | 2025-11-13 | 2026-11-13 | Vendored from rust-hex |

### Hex crate audit checklist

- [ ] No unsafe code (verified: true)
- [ ] No dependencies (verified: true)
- [ ] Simple algorithm (verified: true)
- [ ] Test coverage (verified: 100%)
- [ ] No CVEs in original crate (check: https://rustsec.org/)
```

## Maintenance Strategy

### Updating Vendored Code

**When to update:**
1. Security vulnerability in upstream
2. Bug fix in upstream that affects us
3. Needed feature addition

**How to update:**

```bash
#!/bin/bash
# scripts/update_vendored_hex.sh

set -e

CRATE_NAME="hex"
OLD_VERSION="0.4.3"
NEW_VERSION="$1"

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <new-version>"
    exit 1
fi

echo "Updating vendored $CRATE_NAME from $OLD_VERSION to $NEW_VERSION"

# Download new version
cargo install $CRATE_NAME --version $NEW_VERSION --force

# Find the downloaded source
CARGO_SRC=$(find ~/.cargo/registry/src -name "${CRATE_NAME}-${NEW_VERSION}" -type d | head -1)

if [ -z "$CARGO_SRC" ]; then
    echo "Error: Could not find $CRATE_NAME version $NEW_VERSION"
    exit 1
fi

# Backup current vendored version
cp -r crates/universal-decoder-core/src/vendored/hex crates/universal-decoder-core/src/vendored/hex.backup

# Copy new version
cp "$CARGO_SRC/src/lib.rs" crates/universal-decoder-core/src/vendored/hex/lib.rs
cp "$CARGO_SRC/src/error.rs" crates/universal-decoder-core/src/vendored/hex/error.rs

# Update README
sed -i "s/v$OLD_VERSION/v$NEW_VERSION/g" crates/universal-decoder-core/src/vendored/hex/README.md
sed -i "s/Vendored Date: .*/Vendored Date: $(date +%Y-%m-%d)/g" crates/universal-decoder-core/src/vendored/hex/README.md

# Run tests
cargo test --all

echo "✓ Update complete. Review changes and commit if tests pass."
echo "  Don't forget to update CHANGELOG.md!"
```

## Legal & Attribution Requirements

### License Compliance Checklist

- [x] **Include original license files** (LICENSE-MIT, LICENSE-APACHE)
- [x] **Preserve copyright notices** in source files
- [x] **Document vendoring** in README.md
- [x] **List in NOTICE file** (if your project has one)
- [x] **Mention in your CHANGELOG** when vendoring

### Example NOTICE Entry

```
This software contains vendored code from the following projects:

--------------------------------------------------------------------------------
hex v0.4.3
https://github.com/KokaKiwi/rust-hex
License: MIT OR Apache-2.0

Copyright (c) 2013-2014 The Rust Project Developers.
Copyright (c) 2015-2020 The rust-hex Developers.

See crates/universal-decoder-core/src/vendored/hex/LICENSE-MIT
and crates/universal-decoder-core/src/vendored/hex/LICENSE-APACHE
--------------------------------------------------------------------------------
```

## Benefits vs Risks

### ✅ Benefits

1. **Reduced dependency count**: 8 → 7 → ... → 5
2. **Supply chain security**: Code is frozen and audited
3. **No external breakage**: Upstream changes don't affect us
4. **Full control**: Can patch immediately if needed
5. **Easier formal verification**: All code in one place

### ⚠️ Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| **Miss upstream security fixes** | Monitor RustSec advisories, annual audit |
| **Maintenance burden** | Only vendor stable, small crates (<1000 LOC) |
| **License violations** | Include all original licenses, proper attribution |
| **Code divergence** | Keep modifications minimal and documented |

## When NOT to Vendor

Don't vendor if:

- ❌ Crate is large (>5000 LOC) - too much to audit
- ❌ Crate is actively changing - high maintenance burden
- ❌ Crate has many dependencies - defeats the purpose
- ❌ Crate is cryptographic - dangerous to modify
- ❌ License is incompatible - legal issues

**Example**: We do NOT vendor `sha2` or `borsh` because:
- `sha2`: Cryptographic code (dangerous to modify)
- `borsh`: Actively maintained, larger codebase
- Both: Well-audited by third parties

## Comparison: Vendor vs Reimplement vs External Dependency

| Approach | LOC to Audit | Supply Chain Risk | Maintenance | Legal |
|----------|--------------|-------------------|-------------|-------|
| **External Dependency** | 0 (trust upstream) | High | Low | Simple |
| **Vendoring** | ~700 (one-time) | Low | Medium | Requires attribution |
| **Reimplementation** | ~200 (our code) | None | High | No issues |

**Recommendation for `hex`**: **Vendor** (best balance)

## Implementation Checklist

For vendoring the `hex` crate:

- [ ] Create `src/vendored/hex/` directory
- [ ] Copy `lib.rs`, `error.rs` from hex v0.4.3
- [ ] Copy LICENSE-MIT and LICENSE-APACHE
- [ ] Create `README.md` with attribution
- [ ] Create `mod.rs` for integration
- [ ] Remove `hex` from `Cargo.toml` dependencies
- [ ] Update all imports: `use hex::` → `use crate::hex::`
- [ ] Keep `hex = "0.4.3"` in `[dev-dependencies]` for validation tests
- [ ] Write comparison tests (vendored vs external)
- [ ] Run full test suite
- [ ] Update `DEPENDENCY_AUDIT.md`
- [ ] Update `CHANGELOG.md`
- [ ] Create NOTICE file with attribution
- [ ] Commit with clear message

## Timeline

- **Effort**: 2-3 hours (simple vendoring)
- **Risk**: Low (small, stable crate)
- **Testing**: 1 hour (validation tests)

**Total**: Half a day of work for permanent dependency reduction

---

**Next Step**: Review and approve vendoring strategy, then execute implementation
