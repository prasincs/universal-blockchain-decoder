# Migration to Borsh-Serialized Chain Registry

## Problem

Currently, we vendor 2,397 JSON files (~46MB) and either:
1. Parse them at build time (slow builds, all JSON in repo)
2. Generate Rust code (3-5MB of generated code, slow compiles)

## Solution: Borsh Binary Format

Serialize chain data once to Borsh binary format (~1-2MB), embed at compile time.

### Benefits

| Approach | Repo Size | Build Time | Runtime | Verifiable |
|----------|-----------|------------|---------|------------|
| **Current: JSON + build.rs** | 46MB | Slow (parse JSON) | Fast | ✅ |
| **Alt 1: Generated Rust** | 3-5MB | Very Slow (compile) | Fast | ✅ |
| **Alt 2: Borsh Binary** | **1-2MB** | **Fast** | **Fast** | ✅ |

### Size Comparison

```
JSON files:           46 MB (2,397 files)
Generated Rust code:  3-5 MB (70,000 lines)
Borsh binary:         1-2 MB (single file)   ← 95%+ reduction!
```

## Implementation

### 1. Add Borsh Serialization

Update `Cargo.toml`:
```toml
[dependencies]
borsh = { version = "1.3", features = ["derive"] }
```

Update types to support Borsh:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ChainInfo {
    // ... fields
}
```

### 2. Generate Binary File

```bash
# Generate chains.borsh from JSON files
./scripts/decoder-evm/generate-registry-borsh.sh

# Output: crates/decoder-evm/data/chains.borsh (~1-2MB)
```

### 3. Update Registry Code

Replace `include!()` with `include_bytes!()`:

```rust
// Old approach (generated code)
include!(concat!(env!("OUT_DIR"), "/chain_registry.rs"));

// New approach (Borsh binary)
const CHAINS_BORSH: &[u8] = include_bytes!("../data/chains.borsh");

impl ChainRegistry {
    pub fn new() -> Self {
        let registry: SerializedRegistry =
            SerializedRegistry::try_from_slice(CHAINS_BORSH)
                .expect("Failed to deserialize chain registry");
        // ... build indices
    }
}
```

### 4. Remove build.rs (Optional)

Since we're no longer parsing JSON at build time, `build.rs` can be simplified or removed.

### 5. Update .gitignore

```gitignore
# Don't track JSON files, only the binary
vendored/chainlist/_data/

# Do track the binary
!data/chains.borsh
!data/chains.metadata.txt
```

## Migration Steps

### Step 1: Generate Binary

```bash
# Ensure you have latest chain data
./scripts/decoder-evm/update-chains.sh

# Generate Borsh binary
./scripts/decoder-evm/generate-registry-borsh.sh
```

### Step 2: Update Code

1. Add `BorshSerialize` + `BorshDeserialize` derives to types
2. Update `registry.rs` to deserialize from binary
3. Remove or simplify `build.rs`

### Step 3: Test

```bash
cargo test -p decoder-evm --lib
```

### Step 4: Clean Up Git History (Optional)

Remove JSON files from git tracking:

```bash
# Add to .gitignore
echo "vendored/chainlist/_data/" >> crates/decoder-evm/.gitignore

# Remove from git (but keep on disk for verification)
git rm -r --cached crates/decoder-evm/vendored/chainlist/_data/

# Commit
git add crates/decoder-evm/data/
git commit -m "Migrate to Borsh-serialized chain registry"
```

## Verification

The binary file is still fully verifiable:

```bash
# 1. Clone upstream
git clone https://github.com/ethereum-lists/chains.git /tmp/chains
cd /tmp/chains

# 2. Checkout the commit from metadata
COMMIT=$(cat crates/decoder-evm/data/chains.metadata.txt | grep "Upstream Commit" | awk '{print $3}')
git checkout $COMMIT

# 3. Regenerate binary
./scripts/decoder-evm/generate-registry-borsh.sh

# 4. Compare (should be identical)
diff crates/decoder-evm/data/chains.borsh /tmp/old-binary
```

## Performance Comparison

### Build Time

```
JSON + build.rs:      ~15 seconds (parse 2,397 files)
Generated Rust code:  ~20 seconds (compile 70k lines)
Borsh binary:         ~2 seconds (just deserialize)
```

### Runtime Initialization

```
Generated code:       0 ms (already compiled)
Borsh binary:         ~5-10 ms (deserialize once)
```

### Memory

```
Generated code:       ~5 MB (static data)
Borsh binary:         ~5 MB (after deserialization)
```

## Future Updates

When updating chains:

```bash
# 1. Pull latest from upstream
./scripts/decoder-evm/update-chains.sh

# 2. Regenerate binary
./scripts/decoder-evm/generate-registry-borsh.sh

# 3. Commit the binary (not JSON files)
git add crates/decoder-evm/data/chains.borsh
git add crates/decoder-evm/data/chains.metadata.txt
git commit -m "Update chain registry to <commit>"
```

## Why Borsh?

1. **Already in dependencies**: Core uses Borsh for canonical serialization
2. **Efficient**: Designed for compact binary representation
3. **Deterministic**: Same data always produces same bytes
4. **Rust-native**: Excellent derive macro support
5. **Fast**: Optimized for serialization/deserialization

## Alternative Formats Considered

| Format | Size | Speed | Complexity |
|--------|------|-------|------------|
| JSON | Large | Slow | Simple |
| MessagePack | Medium | Fast | Medium |
| Protobuf | Medium | Fast | Complex (schema) |
| **Borsh** | **Small** | **Very Fast** | **Simple** |
| Bincode | Small | Very Fast | Simple |

We chose Borsh because it's already a dependency and used throughout the project.

## Rollback Plan

If issues arise, easy to rollback:

```bash
# Restore JSON parsing approach
git revert <migration-commit>

# Or manually:
# - Remove Borsh deserialization code
# - Restore build.rs to parse JSON
# - Remove .gitignore entries
```

## Open Questions

1. **Compression**: Should we gzip the Borsh binary for even smaller size?
   - Pro: ~30-40% smaller
   - Con: Decompression overhead, extra dependency

2. **Checksums**: Should we embed a SHA256 checksum?
   - Pro: Detect corruption
   - Con: Already verifiable via git commit hash

3. **Multiple files**: Split by region for lazy loading?
   - Pro: Only load needed chains
   - Con: Added complexity

## Conclusion

**Recommendation**: Migrate to Borsh binary format.

**Impact**:
- ✅ 95%+ size reduction (46MB → 1-2MB)
- ✅ Faster builds (15s → 2s)
- ✅ Still 100% verifiable
- ✅ Simpler maintenance
- ✅ Uses existing dependency (Borsh)

**Cost**:
- One-time migration effort (~2-4 hours)
- Small runtime deserialization cost (~5-10ms once)
