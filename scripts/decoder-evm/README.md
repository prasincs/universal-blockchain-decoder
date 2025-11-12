# Decoder-EVM Maintenance Scripts

This directory contains maintenance scripts for the decoder-evm crate.

## update-chains.sh

Automates the process of updating the vendored ethereum-lists/chains registry.

### Usage

```bash
# Update to latest master
./scripts/decoder-evm/update-chains.sh

# Update to specific commit or branch
./scripts/decoder-evm/update-chains.sh v1.2.3
./scripts/decoder-evm/update-chains.sh abc123def
```

### What it does

1. **Pulls from upstream** using `git subtree pull`
2. **Cleans up** unnecessary files (CI configs, build tools, etc.)
3. **Updates documentation** (VENDORED.md) with commit hash and date
4. **Verifies** essential files are present
5. **Tests** that build and tests still pass
6. **Stages changes** for review
7. **Generates** commit message

### Example Output

```
=== Updating vendored chainlist ===
Repository: https://github.com/ethereum-lists/chains.git
Target: master

==> Pulling from upstream...
Squash commit -- not updating HEAD
  ...

==> Getting upstream commit hash...
Upstream commit: abc123def...
Date: 2025-11-12

==> Cleaning up unnecessary files...
  Removing: .github
  Removing: gradle
  ...

==> Verifying essential files...
  Chain files found: 2397

==> Updating VENDORED.md...
  Updated VENDORED.md with commit abc123def

==> Testing build...
  Finished dev [unoptimized + debuginfo] target(s) in 2.5s

==> Testing...
  test result: ok. 21 passed; 0 failed

==> Suggested commit message:
  Update vendored chainlist to abc123def
  ...

=== Update complete! ===
```

### Manual Process (for reference)

If you need to update manually:

```bash
# 1. Pull from upstream
cd /path/to/universal-blockchain-decoder
git subtree pull \
    --prefix crates/decoder-evm/vendored/chainlist \
    https://github.com/ethereum-lists/chains.git \
    master \
    --squash

# 2. Clean up
cd crates/decoder-evm/vendored/chainlist
rm -rf .ci .github gradle build.gradle gradlew* \
       httpsloader model processor tools website \
       .gitignore .prettierrc* package.json settings.gradle.kts

# 3. Get commit hash
git log --oneline | head -1

# 4. Update VENDORED.md with commit hash

# 5. Test
cargo test -p decoder-evm

# 6. Commit
git add crates/decoder-evm/vendored/
git commit -m "Update vendored chainlist to <commit>"
```

### Troubleshooting

**Build fails after update:**
```bash
# Check what changed
git diff crates/decoder-evm/vendored/chainlist/_data/chains/

# Verify JSON schema matches
cd crates/decoder-evm/vendored/chainlist
cat _data/chains/eip155-1.json | python3 -m json.tool
```

**Tests fail:**
```bash
# Run with verbose output
RUST_MIN_STACK=8388608 cargo test -p decoder-evm --lib -- --nocapture

# Check for breaking changes in chain data
git diff --stat crates/decoder-evm/vendored/chainlist/_data/
```

**Git subtree conflicts:**
```bash
# If subtree pull fails, you may need to manually merge
git subtree pull --prefix crates/decoder-evm/vendored/chainlist \
    https://github.com/ethereum-lists/chains.git master

# Resolve conflicts, then:
git commit
```

### CI Integration

This script can be run in CI to check for upstream updates:

```yaml
# .github/workflows/check-chain-updates.yml
name: Check Chain Registry Updates

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  workflow_dispatch:

jobs:
  check-updates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Check for updates
        run: |
          cd /tmp
          git clone https://github.com/ethereum-lists/chains.git
          cd chains
          LATEST=$(git rev-parse HEAD)
          CURRENT=$(grep "Upstream Commit" \
            $GITHUB_WORKSPACE/crates/decoder-evm/vendored/VENDORED.md | \
            cut -d'`' -f2)
          if [ "$LATEST" != "$CURRENT" ]; then
            echo "::notice::New chains available: $LATEST"
          fi
```

### See Also

- `crates/decoder-evm/vendored/VENDORED.md` - Documentation with commit hashes
- `crates/decoder-evm/README.md` - Decoder documentation
- `CHAIN_FAMILIES_GROUPING.md` - Chain family strategy
