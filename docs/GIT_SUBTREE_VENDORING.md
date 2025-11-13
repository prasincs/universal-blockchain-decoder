# Git Subtree Vendoring Strategy

## Why Git Subtree > Manual Vendoring

**Git subtree** provides verifiable, auditable vendoring with full git history:

| Approach | Verifiable | Updatable | Auditable | Complexity |
|----------|-----------|-----------|-----------|------------|
| **Git Subtree** | ✅✅✅ Git history | ✅ `git subtree pull` | ✅ `git log` | Low |
| Git Submodule | ✅ Linked repo | ✅ `git submodule update` | ⚠️ Separate repos | High |
| Manual Copy | ❌ Trust-based | ❌ Manual | ❌ No history | Very Low |

**Recommendation**: Use **git subtree** for maximum verifiability while keeping a single repository.

## Benefits

### 1. ✅ Verifiable Source

```bash
# See exactly what was vendored and when
git log --oneline crates/universal-decoder-core/src/vendored/hex

# Example output:
# abc123 Squashed 'hex' content from commit 'v0.4.3'
# def456 Add hex subtree from https://github.com/KokaKiwi/rust-hex
```

### 2. ✅ Easy to Audit

```bash
# Compare our vendored version with upstream tag
git diff v0.4.3..HEAD -- crates/universal-decoder-core/src/vendored/hex

# See only our modifications
# (e.g., removing serde feature)
```

### 3. ✅ Transparent Updates

```bash
# Pull upstream updates (if needed)
git subtree pull --prefix crates/universal-decoder-core/src/vendored/hex \
    https://github.com/KokaKiwi/rust-hex.git v0.4.4 --squash

# Git shows exactly what changed
```

### 4. ✅ Cryptographic Verification

```bash
# Verify the vendored code matches upstream tag
git show v0.4.3:src/lib.rs | sha256sum
git show HEAD:crates/universal-decoder-core/src/vendored/hex/src/lib.rs | sha256sum

# Should match (if no modifications)
```

## Implementation: Vendoring `hex` with Git Subtree

### Step 1: Add Upstream Repository as Subtree

```bash
cd /home/user/universal-blockchain-decoder

# Add hex crate as a subtree at specific version
git subtree add \
    --prefix crates/universal-decoder-core/src/vendored/hex \
    https://github.com/KokaKiwi/rust-hex.git \
    v0.4.3 \
    --squash

# This creates:
# - A squashed commit with all hex code
# - Clear git history showing what was added
```

**What happens**:
```
Squashed 'crates/universal-decoder-core/src/vendored/hex/' content from commit 'v0.4.3'
```

### Step 2: Verify What Was Added

```bash
# See all files that were vendored
git diff HEAD~1 --name-only

# Output:
# crates/universal-decoder-core/src/vendored/hex/.gitignore
# crates/universal-decoder-core/src/vendored/hex/Cargo.toml
# crates/universal-decoder-core/src/vendored/hex/LICENSE-APACHE
# crates/universal-decoder-core/src/vendored/hex/LICENSE-MIT
# crates/universal-decoder-core/src/vendored/hex/README.md
# crates/universal-decoder-core/src/vendored/hex/benches/hex.rs
# crates/universal-decoder-core/src/vendored/hex/src/error.rs
# crates/universal-decoder-core/src/vendored/hex/src/lib.rs
# crates/universal-decoder-core/src/vendored/hex/src/serde.rs
# crates/universal-decoder-core/src/vendored/hex/tests/serde.rs
# crates/universal-decoder-core/src/vendored/hex/tests/version-number.rs
```

### Step 3: Make Minimal Modifications (Optional)

```bash
# Remove files we don't need (optional)
cd crates/universal-decoder-core/src/vendored/hex
rm -rf benches/ tests/  # Keep only src/

# Remove serde support (we don't use it)
# Edit src/lib.rs to remove serde module

# Commit our modifications
git add -A
git commit -m "Remove unused features from vendored hex (benches, serde)"
```

### Step 4: Document the Vendoring

Create `crates/universal-decoder-core/src/vendored/hex/VENDORING.md`:

```markdown
# Vendored: hex v0.4.3

This directory contains hex v0.4.3 vendored via git subtree.

## Original Source

- **Repository**: https://github.com/KokaKiwi/rust-hex
- **Commit**: v0.4.3 tag
- **License**: MIT OR Apache-2.0
- **Vendored**: 2025-11-13 via git subtree

## Verification

To verify this code matches upstream:

\`\`\`bash
# Clone upstream
git clone https://github.com/KokaKiwi/rust-hex /tmp/hex
cd /tmp/hex
git checkout v0.4.3

# Compare with our vendored version
diff -r src /path/to/universal-blockchain-decoder/crates/universal-decoder-core/src/vendored/hex/src
\`\`\`

Or using git:

\`\`\`bash
# See what we vendored
git log --oneline -- crates/universal-decoder-core/src/vendored/hex

# See our modifications
git diff v0.4.3..HEAD -- crates/universal-decoder-core/src/vendored/hex
\`\`\`

## Modifications

- Removed `benches/` (not needed)
- Removed `tests/` (not needed, we have our own tests)
- Removed serde feature support (not used)

All modifications are tracked in git history.
```

### Step 5: Create Integration Module

```rust
// crates/universal-decoder-core/src/vendored/mod.rs

pub mod hex {
    // Include vendored hex crate
    // Note: We've removed the serde module
    include!("hex/src/lib.rs");

    pub use crate::vendored::hex_error::FromHexError;
}

// Re-export for backwards compatibility with hex crate API
mod hex_error {
    include!("hex/src/error.rs");
}
```

### Step 6: Update Core Library

```rust
// crates/universal-decoder-core/src/lib.rs

mod vendored;

// Public re-export (decoders can use this)
pub use vendored::hex;

// Rest of the library...
```

### Step 7: Update Dependencies

```diff
# crates/universal-decoder-core/Cargo.toml

[dependencies]
serde = "1.0"
borsh = "1.3"
thiserror = "1.0"
sha2 = "0.10"
sha3 = "0.10"
-hex = "0.4"  # REMOVED: Now vendored via git subtree

[dev-dependencies]
proptest = "1.4"
+hex = "0.4.3"  # For validation tests only
```

## Updating Vendored Code

### When to Update

- Security vulnerability in upstream
- Bug fix we need
- Feature we want to use

### How to Update

```bash
# Pull new version from upstream
git subtree pull \
    --prefix crates/universal-decoder-core/src/vendored/hex \
    https://github.com/KokaKiwi/rust-hex.git \
    v0.4.4 \
    --squash

# Git will merge the changes and show conflicts (if any)

# Verify the update
cargo test --all

# Commit the merge if tests pass
```

### Automated Update Script

```bash
#!/bin/bash
# scripts/update_vendored_hex.sh

set -e

NEW_VERSION="$1"

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <version-tag>"
    echo "Example: $0 v0.4.4"
    exit 1
fi

echo "Updating vendored hex to $NEW_VERSION..."

# Pull new version
git subtree pull \
    --prefix crates/universal-decoder-core/src/vendored/hex \
    https://github.com/KokaKiwi/rust-hex.git \
    "$NEW_VERSION" \
    --squash

# Run tests
echo "Running tests..."
cargo test --all

echo "✓ Update complete. Review changes and commit if all tests pass."
```

## Verification Commands

### Verify Vendored Code Matches Upstream

```bash
# 1. Compare file-by-file with upstream tag
git clone --depth 1 --branch v0.4.3 https://github.com/KokaKiwi/rust-hex /tmp/hex
diff -r /tmp/hex/src crates/universal-decoder-core/src/vendored/hex/src

# 2. Or use git to verify
git remote add hex-upstream https://github.com/KokaKiwi/rust-hex.git
git fetch hex-upstream v0.4.3
git diff v0.4.3:src crates/universal-decoder-core/src/vendored/hex/src

# 3. Cryptographic verification
git show v0.4.3:src/lib.rs | sha256sum
# Compare with:
sha256sum crates/universal-decoder-core/src/vendored/hex/src/lib.rs
```

### See Vendoring History

```bash
# See when hex was vendored and all changes
git log --oneline -- crates/universal-decoder-core/src/vendored/hex

# See our modifications
git log --patch -- crates/universal-decoder-core/src/vendored/hex

# See only our commits (exclude upstream)
git log --oneline --no-merges -- crates/universal-decoder-core/src/vendored/hex
```

### Audit Trail

```bash
# Show the exact commit that was vendored
git log --grep="Squashed 'hex'" --oneline

# Verify the upstream commit hash
git log --pretty=fuller -- crates/universal-decoder-core/src/vendored/hex | grep "commit"
```

## Security Benefits

### 1. Reproducible Builds

```bash
# Anyone can verify our vendored code matches upstream
git clone https://github.com/prasincs/universal-blockchain-decoder
cd universal-blockchain-decoder

# Check what version was vendored
git log --oneline -- crates/universal-decoder-core/src/vendored/hex | head -1
# Output: abc123 Squashed 'hex' content from commit 'v0.4.3'

# Verify it matches upstream
git remote add hex-upstream https://github.com/KokaKiwi/rust-hex.git
git fetch hex-upstream v0.4.3
git diff v0.4.3 -- crates/universal-decoder-core/src/vendored/hex
```

### 2. Supply Chain Security

```bash
# See ALL modifications we made to vendored code
git log --patch -- crates/universal-decoder-core/src/vendored/hex

# Any malicious changes would be visible in git history
```

### 3. Cryptographic Proof

```bash
# Generate checksums for audit
find crates/universal-decoder-core/src/vendored/hex/src -type f -name "*.rs" \
    -exec sha256sum {} \; > vendored-hex-checksums.txt

# Store in git for future verification
git add vendored-hex-checksums.txt
git commit -m "Add checksums for vendored hex v0.4.3"

# Anyone can verify later
sha256sum -c vendored-hex-checksums.txt
```

## Alternative: Git Submodule (Not Recommended)

### Why Not Submodules?

Git submodules are more complex and have drawbacks:

```bash
# Submodule approach (DON'T USE)
git submodule add https://github.com/KokaKiwi/rust-hex.git \
    crates/universal-decoder-core/src/vendored/hex

# Problems:
# 1. Users must run: git submodule update --init --recursive
# 2. Submodules are separate repositories (harder to manage)
# 3. Can't easily make local modifications
# 4. Detached HEAD state confusion
# 5. More complex to audit
```

**Verdict**: Use **git subtree**, not submodules, for vendoring.

## Comparison: Manual vs Subtree

### Manual Vendoring

```bash
# Manual copy
cp -r ~/.cargo/registry/src/*/hex-0.4.3/src/ \
    crates/universal-decoder-core/src/vendored/hex/

# ❌ No verification
# ❌ No update mechanism
# ❌ No audit trail
# ⚠️ Must trust the copy was done correctly
```

### Git Subtree Vendoring

```bash
# Git subtree
git subtree add --prefix path https://github.com/repo v0.4.3 --squash

# ✅ Git history proves what was vendored
# ✅ Easy updates: git subtree pull
# ✅ Full audit trail: git log
# ✅ Cryptographically verifiable: git diff
```

## Integration with CI/CD

### Verify Vendored Code in CI

```yaml
# .github/workflows/verify-vendored.yml

name: Verify Vendored Dependencies

on: [push, pull_request]

jobs:
  verify-hex:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need full history for git subtree

      - name: Add upstream remote
        run: |
          git remote add hex-upstream https://github.com/KokaKiwi/rust-hex.git
          git fetch hex-upstream v0.4.3

      - name: Verify vendored hex matches upstream
        run: |
          # Our modifications should be documented
          git diff v0.4.3 -- crates/universal-decoder-core/src/vendored/hex > modifications.diff

          # Check that modifications are only what we expect
          # (removal of benches/, tests/, serde feature)
          if grep -q "dangerous_change" modifications.diff; then
            echo "ERROR: Unexpected modifications to vendored hex!"
            exit 1
          fi

          echo "✓ Vendored hex verified against upstream v0.4.3"
```

## Documentation Requirements

Every vendored dependency must have:

1. **VENDORING.md** - Explains source, version, verification
2. **Git history** - Shows what was vendored (git subtree commit)
3. **Modifications documented** - All changes tracked in git
4. **Verification script** - How to verify against upstream

## Migration from Manual to Subtree

If you've already manually vendored:

```bash
# 1. Remove manually vendored files
git rm -r crates/universal-decoder-core/src/vendored/hex
git commit -m "Remove manually vendored hex (will re-add via subtree)"

# 2. Add via git subtree
git subtree add \
    --prefix crates/universal-decoder-core/src/vendored/hex \
    https://github.com/KokaKiwi/rust-hex.git \
    v0.4.3 \
    --squash

# 3. Reapply your modifications
# (remove benches, serde, etc.)
git commit -m "Reapply minimal modifications to vendored hex"
```

## Summary

**Use git subtree for vendoring because:**

1. ✅ **Verifiable**: Git history proves what was vendored
2. ✅ **Auditable**: Anyone can verify against upstream
3. ✅ **Updatable**: `git subtree pull` for updates
4. ✅ **Transparent**: All modifications tracked in git
5. ✅ **Single repository**: No submodule complexity
6. ✅ **Cryptographically verifiable**: Git hashes prove integrity

**Command Summary**:

```bash
# Initial vendoring
git subtree add --prefix path/to/vendor https://repo.git tag --squash

# Update
git subtree pull --prefix path/to/vendor https://repo.git new-tag --squash

# Verify
git diff upstream-tag -- path/to/vendor

# Audit
git log --oneline -- path/to/vendor
```

**Result**: Maximum verifiability + minimal complexity = **Use git subtree**

---

**Next Step**: Implement hex vendoring using git subtree instead of manual copy
