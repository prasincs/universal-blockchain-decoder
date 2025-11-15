# Decoder Generator Improvements

## Status: Planned Enhancements

The `decoder-generator` crate exists but is currently **underutilized**. This document outlines improvements to make it the **standard way** to add new chains.

---

## Current State

**Crate**: `crates/decoder-generator/`

**Files**:
- `src/lib.rs` - Main generator logic
- `src/spec.rs` - TOML specification parser
- `src/codegen.rs` - Code generation engine
- `src/templates.rs` - Boilerplate templates
- `src/bin/decoder_gen.rs` - CLI tool

**Usage** (documented but not actively used):
```bash
cargo run -p decoder-generator -- generate specs/dogecoin.toml
```

**Problem**: All 30 decoders were created manually, not with the generator.

---

## Proposed Enhancements

### 1. Add `new` Command (Quick Scaffold)

**Goal**: Generate a complete decoder scaffold in < 5 minutes

**Command**:
```bash
cargo run -p decoder-generator -- new \
    --chain "Avalanche" \
    --chain-id 43114 \
    --family Account \
    --template evm-compatible
```

**What It Generates**:
```
crates/decoder-avalanche/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # ChainIdentity + ChainDecoder + Canonicalizer
│   ├── types.rs            # Transaction structure (template)
│   └── registry.rs         # (optional, for multi-chain families)
├── tests/
│   ├── unit_tests.rs       # Chain identity test
│   ├── property_tests.rs   # 8 property tests (from template)
│   ├── integration_tests.rs # Fixture loading + batch tests
│   └── fixtures/
│       ├── README.md       # Fixture documentation template
│       └── mainnet/
│           └── .gitkeep
└── .gitignore
```

**Implementation** (`src/commands/new.rs`):
```rust
pub fn generate_new_decoder(args: NewDecoderArgs) -> Result<()> {
    // 1. Create directory structure
    create_decoder_directories(&args.chain)?;

    // 2. Generate Cargo.toml from template
    generate_cargo_toml(&args)?;

    // 3. Generate lib.rs with chain-specific values
    generate_lib_rs(&args)?;

    // 4. Copy test templates with substitution
    copy_test_templates(&args)?;

    // 5. Add to workspace Cargo.toml
    add_to_workspace(&args.chain)?;

    // 6. Run cargo fmt on generated files
    format_generated_code(&args.chain)?;

    Ok(())
}
```

### 2. Add `add-tests` Command (Test Generation)

**Goal**: Add comprehensive tests to existing decoder

**Command**:
```bash
cargo run -p decoder-generator -- add-tests \
    --decoder avalanche \
    --fixtures 3 \
    --property-tests 10 \
    --fuzz
```

**What It Generates**:
- `tests/property_tests.rs` - 10 property-based tests
- `tests/integration_tests.rs` - 3 fixture test stubs
- `fuzz/fuzz_targets/fuzz_decode.rs` - Fuzzing target
- `tests/fixtures/README.md` - Fixture documentation

**Implementation**:
```rust
pub fn add_tests(args: AddTestsArgs) -> Result<()> {
    let decoder_path = format!("crates/decoder-{}", args.decoder);

    // 1. Generate property tests
    if args.property_tests > 0 {
        generate_property_tests(&decoder_path, args.property_tests)?;
    }

    // 2. Generate integration test stubs
    if args.fixtures > 0 {
        generate_integration_tests(&decoder_path, args.fixtures)?;
    }

    // 3. Generate fuzzing targets
    if args.fuzz {
        generate_fuzz_targets(&decoder_path)?;
    }

    Ok(())
}
```

### 3. Add `validate` Command (Check Completeness)

**Goal**: Verify decoder meets maturity level requirements

**Command**:
```bash
cargo run -p decoder-generator -- validate \
    --decoder avalanche \
    --maturity advanced
```

**Output**:
```
Validating decoder-avalanche against "Advanced" maturity level...

✅ PASS: Chain identity tests present
✅ PASS: Format validation implemented
✅ PASS: Canonicalizer implemented
⚠️  WARN: Only 3/10 property tests present
❌ FAIL: No integration tests found
❌ FAIL: No test fixtures found
✅ PASS: README.md exists
⚠️  WARN: README.md missing "Test Fixtures" section

Score: 4/8 requirements met

Next steps to reach "Advanced" maturity:
1. Add 7 more property tests (see docs/templates/PROPERTY_TEST_TEMPLATE.rs)
2. Add integration tests (see docs/templates/INTEGRATION_TEST_TEMPLATE.rs)
3. Add at least 3 test fixtures to tests/fixtures/mainnet/
4. Update README.md with fixture documentation
```

**Maturity Levels**:

| Level | Requirements |
|-------|-------------|
| **Scaffold** | Chain identity test |
| **Core** | + Format validation, + 1 fixture, + Integration tests |
| **Advanced** | + 10 property tests, + 3 fixtures, + Canonicalizer |
| **Production** | + Reference validation, + Fuzzing, + 95% coverage |

### 4. Add `upgrade` Command (Batch Improvements)

**Goal**: Upgrade existing decoder to next maturity level

**Command**:
```bash
cargo run -p decoder-generator -- upgrade \
    --decoder avalanche \
    --from scaffold \
    --to core
```

**What It Does**:
1. Runs `validate` to identify gaps
2. Generates missing components (tests, fixtures, docs)
3. Provides TODO comments in code for manual work
4. Updates README.md with new status

**Example Output**:
```
Upgrading decoder-avalanche from Scaffold → Core...

Generated:
✅ tests/integration_tests.rs (with 1 fixture stub)
✅ tests/fixtures/mainnet/tx_001_simple.hex (empty, needs real data)
✅ tests/fixtures/README.md
✅ Updated src/lib.rs with validate_format() TODO

Manual steps required:
1. Add real transaction data to tests/fixtures/mainnet/tx_001_simple.hex
   - Get from: https://snowtrace.io/txs
   - Format: Hex-encoded transaction bytes (one line, no 0x prefix)

2. Implement validate_format() in src/lib.rs:72
   - Check transaction structure
   - Verify version, lengths, signatures

3. Run: cargo test -p decoder-avalanche

Upgrade progress: 60% complete (automation), 40% manual work needed
```

### 5. Add `batch-upgrade` Command (Workspace-Wide)

**Goal**: Upgrade all decoders matching criteria

**Command**:
```bash
# Upgrade all Scaffold decoders to Core
cargo run -p decoder-generator -- batch-upgrade \
    --from scaffold \
    --to core \
    --dry-run

# Actually perform upgrades
cargo run -p decoder-generator -- batch-upgrade \
    --from scaffold \
    --to core
```

**Output**:
```
Found 12 decoders at Scaffold maturity:
- decoder-algorand
- decoder-tron
- decoder-stellar
- decoder-polkadot
- decoder-cardano
- decoder-near
- decoder-dash
- decoder-bnb
- decoder-polygon
- ...

Upgrading to Core maturity (adds integration tests + 1 fixture)...

[1/12] decoder-algorand... ✅ Done
[2/12] decoder-tron... ✅ Done
...
[12/12] decoder-polygon... ✅ Done

Summary:
- 12 decoders upgraded
- 24 files generated
- 36 manual TODOs created

Next steps:
1. Add real transaction fixtures (see tests/fixtures/README.md in each decoder)
2. Implement validate_format() (search for TODO in src/lib.rs)
3. Run: cargo test --workspace
```

---

## Implementation Plan

### Phase 1: Basic Commands (1 week)

**PR #1**: Implement `new` command
- Create directory structure
- Generate Cargo.toml, lib.rs, README.md
- Copy basic test templates
- Add to workspace

**PR #2**: Implement `add-tests` command
- Copy property test template with substitution
- Copy integration test template
- Generate fuzzing targets (optional)

**PR #3**: Implement `validate` command
- Check file existence
- Count tests
- Verify maturity requirements
- Generate report

### Phase 2: Advanced Commands (1 week)

**PR #4**: Implement `upgrade` command
- Run validate
- Generate missing components
- Insert TODO comments
- Update README

**PR #5**: Implement `batch-upgrade` command
- Find decoders by maturity
- Batch apply upgrades
- Summary reporting

### Phase 3: Templates & Registry (3 days)

**PR #6**: Improve templates
- Add chain family templates (EVM, Bitcoin, Cosmos, Account, UTXO)
- Add registry-based decoder template
- Add multi-chain decoder template (like decoder-evm)

**PR #7**: Registry integration
- Auto-detect if chain belongs to existing family
- Suggest using registry instead of new decoder
- Provide instructions for adding to registry

### Phase 4: CI Integration (2 days)

**PR #8**: Add CI checks
- Run `validate` on all decoders in CI
- Fail if any decoder regresses in maturity
- Generate maturity dashboard (markdown)

---

## Templates System

### Template Structure

```
crates/decoder-generator/templates/
├── common/
│   ├── Cargo.toml.template
│   ├── README.md.template
│   ├── lib.rs.template
│   ├── types.rs.template
│   └── .gitignore.template
├── families/
│   ├── evm/
│   │   ├── lib.rs.template
│   │   └── types.rs.template
│   ├── bitcoin/
│   │   ├── lib.rs.template
│   │   └── parsing.rs.template
│   ├── cosmos/
│   │   ├── lib.rs.template
│   │   └── registry.rs.template
│   └── account/
│       └── lib.rs.template
├── tests/
│   ├── unit_tests.rs.template
│   ├── property_tests.rs.template
│   └── integration_tests.rs.template
└── fuzz/
    └── fuzz_decode.rs.template
```

### Template Variables

All templates support these substitutions:

```rust
{{CHAIN}}           // "Avalanche"
{{chain}}           // "avalanche"
{{CHAIN_UPPER}}     // "AVALANCHE"
{{chain_id}}        // "43114"
{{chain_family}}    // "Account"
{{year}}            // "2025"
{{version}}         // "0.1.0"
```

### Template Example

**File**: `templates/common/lib.rs.template`

```rust
//! {{CHAIN}} transaction decoder

use decoder_primitives::prelude::*;
use decoder_chains_common::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct {{CHAIN}}Chain;

impl ChainIdentity for {{CHAIN}}Chain {
    fn chain_id(&self) -> u64 {
        {{chain_id}}
    }

    fn chain_name(&self) -> &str {
        "{{CHAIN}}"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::{{chain_family}}
    }
}

// ... rest of template
```

---

## Usage Examples

### Example 1: Add New EVM-Compatible Chain

```bash
# Generate scaffold
cargo run -p decoder-generator -- new \
    --chain "Avalanche" \
    --chain-id 43114 \
    --family Account \
    --template evm-compatible

# Add comprehensive tests
cargo run -p decoder-generator -- add-tests \
    --decoder avalanche \
    --property-tests 10 \
    --fixtures 3

# Build and test
cargo test -p decoder-avalanche

# Validate maturity
cargo run -p decoder-generator -- validate \
    --decoder avalanche \
    --maturity core
```

### Example 2: Upgrade Existing Decoder

```bash
# Check current status
cargo run -p decoder-generator -- validate \
    --decoder algorand \
    --maturity advanced

# Upgrade to Advanced maturity
cargo run -p decoder-generator -- upgrade \
    --decoder algorand \
    --from scaffold \
    --to advanced

# Add missing components manually (follow TODOs)
# ...

# Verify upgrade succeeded
cargo run -p decoder-generator -- validate \
    --decoder algorand \
    --maturity advanced
```

### Example 3: Batch Upgrade All Scaffold Decoders

```bash
# Preview changes
cargo run -p decoder-generator -- batch-upgrade \
    --from scaffold \
    --to core \
    --dry-run

# Execute
cargo run -p decoder-generator -- batch-upgrade \
    --from scaffold \
    --to core

# Test all upgraded decoders
cargo test --workspace
```

---

## Benefits

1. **Consistency**: All decoders follow same structure
2. **Speed**: 5 min to scaffold vs 30 min manual
3. **Quality**: Tests included by default
4. **Discoverability**: Templates show best practices
5. **Maintainability**: Batch operations for workspace-wide changes
6. **Metrics**: Track maturity across all decoders

---

## Metrics Dashboard (Future)

After implementing `validate` command, generate a dashboard:

```markdown
# Decoder Maturity Dashboard

Generated: 2025-11-15 14:32:00

## Summary

| Maturity | Count | Percentage |
|----------|-------|------------|
| Production | 6 | 20% |
| Advanced | 4 | 13% |
| Core | 8 | 27% |
| Scaffold | 12 | 40% |

## Production Decoders (6)

- ✅ decoder-bitcoin (100% coverage, fuzzing, validation)
- ✅ decoder-ethereum (100% coverage, fuzzing, validation)
- ✅ decoder-solana (95% coverage, fuzzing, validation)
- ✅ decoder-cosmos (90% coverage, validation)
- ✅ decoder-evm (85% coverage, fuzzing)
- ✅ decoder-optimism (90% coverage, validation)

## Needs Improvement (12 Scaffold)

- 📋 decoder-algorand (only chain_identity test)
- 📋 decoder-tron (only chain_identity test)
...

## Next Actions

1. Run batch-upgrade to move 12 Scaffold → Core: `cargo run -p decoder-generator -- batch-upgrade --from scaffold --to core`
2. Add fixtures for Core decoders (see tests/fixtures/README.md)
3. Implement validate_format() for all decoders (search for TODO)
```

---

## See Also

- `docs/templates/PROPERTY_TEST_TEMPLATE.rs` - Property test template
- `docs/templates/INTEGRATION_TEST_TEMPLATE.rs` - Integration test template
- `docs/CHAIN_ADDITION_STRATEGY.md` - Manual chain addition guide
- `docs/TESTING_STRATEGY.md` - Testing philosophy

---

**Status**: Planned (not yet implemented)
**Priority**: HIGH (would significantly speed up development)
**Effort**: 2-3 weeks for full implementation
