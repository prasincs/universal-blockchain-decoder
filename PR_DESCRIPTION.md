## Summary

This PR completes **Phase 3.4: SVM (Solana Virtual Machine) Family Decoder** from the roadmap and includes a critical fix for Bitcoin property tests.

### Key Achievements

- ✅ **Phase 3.4 Complete**: Full SVM ecosystem decoder implementation
- ✅ **11 SVM chains supported**: Solana, Eclipse, Pyth, Drift, Jito, Sonic, Firedancer, NeonEvm
- ✅ **41 comprehensive tests**: 13 unit + 16 integration + 12 property tests
- ✅ **Property test coverage**: 28/50 (56%) - +75% progress toward 50-test goal
- ✅ **Bitcoin test fix**: Eliminated 1024 rejections in property tests

### Changes Included

#### 1. Bitcoin Property Test Fix (`b3059a5`)

**Problem**: `prop_fee_calculation_properties` was rejecting ~50% of generated inputs due to independent value generation followed by `prop_assume!(input_value >= output_value)`.

**Solution**: Changed to dependent value generation using `prop_flat_map`:
```rust
// BEFORE: 50% rejection rate
(input_value, output_value) with prop_assume!(input >= output)

// AFTER: 0% rejection rate
(input_value, output_value) in input_range
    .prop_flat_map(|input| (Just(input), 0u64..=input))
```

**Result**: Zero rejections, all 186 Bitcoin tests pass

#### 2. SVM Family Decoder Implementation (`dce1250`)

**Architecture**: Wrapper pattern around `decoder-solana` with chain-specific context

**New Crates**:
- `crates/decoder-svm/` - SVM family decoder
- Wraps `decoder-solana` for chain identification
- Compile-time chain registry (no runtime I/O)

**Core Components**:
- `SvmChainId` enum: 11 chain identifiers
- `SvmChainRegistry`: Metadata for all SVM chains
- `SvmDecoder`: Implements `ChainDecoder` trait
- `SvmTransaction`: Wraps `SolanaTransaction` with chain context

**Initial Chains** (8):
- Solana Mainnet (101), Devnet (102), Testnet (103)
- Eclipse Mainnet (201), Testnet (202)
- Pyth Network (301)
- Drift Protocol (401)
- Jito (501)

**Tests**: 29 tests (13 unit + 16 integration)

#### 3. SVM Ecosystem Enhancements (`19e4e28`)

**Added Property Tests** (12 tests):
- Chain ID conversion roundtrips
- Property consistency (is_solana, is_mainnet, is_testnet)
- Registry completeness and lookup correctness
- Chain identity implementation validation
- Decoder creation safety
- Format validation
- Name stability
- RPC endpoint consistency
- Explorer URL format validation

**Added Usage Example**:
- `examples/decode_svm_transaction.rs` (125 lines)
- Demonstrates multi-chain decoding
- Shows chain selection patterns
- Displays chain properties

**Expanded Chain Support** (+3 chains):
- Sonic SVM (601) - gaming-focused L2
- Firedancer (701) - high-performance validator
- Neon EVM (801) - Ethereum compatibility

**Final Stats**: 41 tests (13 unit + 16 integration + 12 property)

### Testing

All tests passing on rebased branch:

```bash
# Bitcoin decoder: 186 tests
cargo test -p decoder-bitcoin
# ✅ All tests pass (includes fixed property test)

# SVM decoder: 41 tests
cargo test -p decoder-svm
# ✅ 13 unit tests
# ✅ 16 integration tests
# ✅ 12 property tests

# Example runs successfully
cargo run --example decode_svm_transaction
```

### Files Changed

```
 Cargo.toml                                         |   1 +
 crates/decoder-bitcoin/tests/property_tests.rs     |   9 +-
 crates/decoder-svm/Cargo.toml                      |  20 ++
 crates/decoder-svm/README.md                       |  67 ++++
 crates/decoder-svm/examples/decode_svm_transaction.rs | 125 ++++++++
 crates/decoder-svm/src/lib.rs                      | 271 ++++++++++++++++
 crates/decoder-svm/src/registry.rs                 | 311 ++++++++++++++++++
 crates/decoder-svm/tests/integration_tests.rs      | 349 +++++++++++++++++++++
 crates/decoder-svm/tests/property_tests.rs         | 338 ++++++++++++++++++++

 9 files changed, 1486 insertions(+), 5 deletions(-)
```

### Roadmap Impact

**Phase 3.4: SVM Family Decoder** ✅ COMPLETE
- Status: 100% complete
- All deliverables met:
  - [x] SVM chain registry with 11+ chains
  - [x] Wrapper pattern around decoder-solana
  - [x] Multi-chain support (Solana, Eclipse, Pyth, etc.)
  - [x] Comprehensive testing (41 tests)
  - [x] Property-based tests (12 tests)
  - [x] Usage examples
  - [x] Documentation

**Property Test Progress**:
- Before: 16/50 (32%)
- After: 28/50 (56%)
- **+75% progress** toward 50-test goal

### Design Compliance

✅ Follows `CLAUDE.md` design principles:
- Minimal core (wrapper pattern, no core changes)
- Trait-based extensibility (implements `ChainDecoder`)
- Zero-cost abstractions (static dispatch)
- Canonical serialization (Borsh)
- Comprehensive testing (unit + integration + property)
- Pure Rust implementation
- Compile-time chain registry (no runtime I/O)

### Next Steps

After merging, recommended next actions:
1. **Phase 3.6**: Move VM decoder (Aptos/Sui)
2. **Phase 3.10**: WASM demo (high visibility for papers/conferences)
3. **Property tests**: Continue toward 50/50 goal (need 22 more)

## Test Plan

- [x] All Bitcoin tests pass (186 tests)
- [x] All SVM tests pass (41 tests)
- [x] Property tests run without rejections
- [x] Example runs successfully
- [x] Branch rebased on latest main
- [x] Pre-commit checks pass (`cargo fmt`, `cargo clippy`)
- [x] No merge conflicts
- [x] Git history is clean
