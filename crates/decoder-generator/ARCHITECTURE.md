# decoder-generator Architecture: Extract, Don't Specify

## The Problem

TOML specs have a fundamental issue: **sync drift**.

```
Manual TOML spec → Generated code → Developer edits code → Spec outdated
```

## Solution: Code as Source of Truth

### Phase 1: Template-Based Generation (Current)

**Instead of TOML**, use the **actual working decoder as the template**:

```rust
// crates/decoder-generator/src/templates/bitcoin.rs
// This IS the Bitcoin decoder - used as a template for similar chains

use decoder_primitives::prelude::*;

pub struct BitcoinTemplate {
    pub chain_name: &'static str,
    pub chain_id: u64,
    pub has_segwit: bool,
    pub hash_algorithm: HashAlgorithm,
}

impl BitcoinTemplate {
    pub fn generate(&self) -> String {
        // Generate code by substituting values in actual working code
        include_str!("../../decoder-bitcoin/src/lib.rs")
            .replace("Bitcoin", self.chain_name)
            .replace("chain_id: 0", &format!("chain_id: {}", self.chain_id))
            // ... smart replacements
    }
}
```

**Usage**:
```bash
# Generate Litecoin from Bitcoin template
cargo run -p decoder-generator -- from-template bitcoin \
    --name Litecoin \
    --chain-id 2 \
    --segwit true \
    --output crates/decoder-litecoin
```

### Phase 2: Trait-Based Extension (Better)

**Make Bitcoin decoder generic**:

```rust
// crates/decoder-bitcoin/src/lib.rs
pub trait UtxoChainConfig {
    const CHAIN_ID: u64;
    const CHAIN_NAME: &'static str;
    const HAS_SEGWIT: bool;
    const HASH_ALG: HashAlgorithm;

    // Override only if different from Bitcoin
    fn parse_witness<R: Read>(reader: &mut R) -> Result<Witness> {
        if Self::HAS_SEGWIT {
            parse_witness_default(reader)
        } else {
            Ok(Witness::None)
        }
    }
}

// Generic UTXO decoder - works for any config
pub struct UtxoDecoder<C: UtxoChainConfig>(PhantomData<C>);

impl<C: UtxoChainConfig> ChainDecoder for UtxoDecoder<C> {
    // Uses C::CHAIN_ID, C::parse_witness, etc.
}
```

**Add Litecoin** (just configuration):
```rust
// crates/decoder-litecoin/src/lib.rs
use decoder_bitcoin::{UtxoChainConfig, UtxoDecoder};

struct LitecoinConfig;

impl UtxoChainConfig for LitecoinConfig {
    const CHAIN_ID: u64 = 2;
    const CHAIN_NAME: &'static str = "Litecoin";
    const HAS_SEGWIT: bool = true;  // Only difference from Bitcoin
    const HASH_ALG: HashAlgorithm = HashAlgorithm::DoubleSha256;
}

// That's it! Litecoin decoder is done
pub type LitecoinDecoder = UtxoDecoder<LitecoinConfig>;
```

**Dogecoin** (different from Bitcoin):
```rust
struct DogecoinConfig;

impl UtxoChainConfig for DogecoinConfig {
    const CHAIN_ID: u64 = 3;
    const CHAIN_NAME: &'static str = "Dogecoin";
    const HAS_SEGWIT: bool = false;  // Override
    const HASH_ALG: HashAlgorithm = HashAlgorithm::DoubleSha256;
}

pub type DogecoinDecoder = UtxoDecoder<DogecoinConfig>;
```

### Phase 3: Macro-Based DSL (Even Better)

**Declarative, but in Rust** (type-safe, can't drift):

```rust
use decoder_dsl::chain;

chain! {
    name: Litecoin,
    id: 2,
    extends: Bitcoin,

    // Only specify differences
    differences {
        // Nothing! Litecoin is identical to Bitcoin
    }
}

chain! {
    name: Dogecoin,
    id: 3,
    extends: Bitcoin,

    differences {
        segwit: false,  // Compiler ensures this is valid
    }
}
```

**Benefits**:
- ✅ Type-safe (won't compile if wrong)
- ✅ Can't drift (spec IS the code)
- ✅ IDE support (autocomplete, refactoring)
- ✅ Tested by compiling
- ✅ Versioned with code

## Keeping Things in Sync

### Current Problem (TOML)
```
specs/dogecoin.toml  ←→  crates/decoder-dogecoin/  (can diverge)
```

### Solution 1: Tests Enforce Sync
```rust
#[test]
fn test_spec_matches_implementation() {
    let spec = load_spec("specs/dogecoin.toml");
    let actual = DogecoinDecoder::chain();

    assert_eq!(spec.chain_id, actual.chain_id());  // Fails if out of sync
    assert_eq!(spec.chain_name, actual.chain_name());
}
```

### Solution 2: Generate Specs FROM Code (Reverse)
```bash
# Extract spec from working decoder
cargo run -p decoder-generator -- extract decoder-dogecoin > specs/dogecoin.toml

# This becomes DOCUMENTATION, not source of truth
```

### Solution 3: No Specs (Best)
**Just use Rust traits** - the code IS the spec.

```rust
// No TOML needed!
pub trait ChainConfig {
    const ID: u64;
    const NAME: &'static str;
    // ... all config here, type-checked
}
```

## Recommended Architecture

### For This Project

1. **Short term**: Keep TOML for exploration
   - Add sync tests (spec ↔ code validation)
   - Treat specs as documentation

2. **Medium term**: Trait-based extension
   ```rust
   // decoder-bitcoin becomes generic
   pub struct UtxoDecoder<C: UtxoChainConfig>;

   // Other chains just provide config
   impl UtxoChainConfig for Dogecoin { ... }
   ```

3. **Long term**: Proc macro DSL
   ```rust
   #[derive_decoder(extends = Bitcoin, segwit = false)]
   struct Dogecoin;
   ```

## Why This Is Better

| Approach | Sync Risk | Type Safety | Maintainability |
|----------|-----------|-------------|-----------------|
| **TOML specs** | ❌ High | ❌ None | ❌ Two sources of truth |
| **Trait configs** | ✅ Low | ✅ Full | ✅ Code IS spec |
| **Proc macros** | ✅ None | ✅ Full | ✅ DRY, compiler-enforced |

## Implementation Plan

### Week 1: Refactor Bitcoin to be Generic
```rust
// Make BitcoinDecoder generic over config
pub struct UtxoDecoder<C: UtxoChainConfig> {
    _phantom: PhantomData<C>,
}

// Tests ensure it still works
#[test]
fn test_bitcoin_decoder_unchanged() {
    // Existing tests pass with generic version
}
```

### Week 2: Add Litecoin/Dogecoin via Config
```rust
// No code duplication - just config
impl UtxoChainConfig for Litecoin { ... }
impl UtxoChainConfig for Dogecoin { ... }
```

### Week 3: Extract to decoder-families Crate
```
crates/decoder-families/
├── utxo/       # UtxoDecoder<C>
├── account/    # AccountDecoder<C> (Ethereum-like)
├── instruction/ # InstructionDecoder<C> (Solana-like)
└── move/       # MoveDecoder<C> (Aptos/Sui)
```

## Migration Path from TOML

If we keep TOML for now:

1. **Add validation tests**:
```rust
#[test]
fn specs_match_implementations() {
    for spec in glob("specs/*.toml") {
        let spec: DecoderSpec = load(spec);
        let decoder = get_decoder(&spec.chain.name);

        // Fail fast if out of sync
        assert_eq!(spec.chain.id, decoder.chain_id());
    }
}
```

2. **Use TOML for CI testing only**:
```yaml
# .github/workflows/test.yml
- name: Validate specs match code
  run: cargo test --test spec_validation
```

3. **Eventually remove TOML**:
   - Once trait-based approach works
   - TOML becomes optional docs/examples

## Conclusion

**Don't fight sync issues** - eliminate them by making code the source of truth.

The best spec is **working, tested code** that others can extend via traits/configs.
