# decoder-chains-common

Common utilities and patterns for blockchain decoder implementations.

## Overview

This crate provides shared functionality used across multiple decoder crates, reducing code duplication and ensuring consistency. It is part of the Universal Blockchain Decoder project's strategy to maintain a minimal, auditable core while maximizing code reuse.

## Modules

### 📋 `validation` - Standard Validation Functions

Common validation patterns for transaction format checking:

```rust
use decoder_chains_common::validation;

// Validate not empty
validation::validate_not_empty(raw_bytes, "Bitcoin")?;

// Validate size bounds
validation::validate_size_bounds(raw_bytes, min, max, "Bitcoin")?;

// Combined validation (not empty + size bounds)
validation::validate_format(raw_bytes, 10, 100_000, "Bitcoin")?;
```

**Functions:**
- `validate_not_empty()` - Ensures input is not empty
- `validate_min_size()` - Validates minimum size requirement
- `validate_max_size()` - Validates maximum size requirement
- `validate_size_bounds()` - Validates size is within min/max bounds
- `validate_format()` - Combined validation (not empty + size bounds)

### 🔐 `hashing` - Cryptographic Hash Functions

Standardized hash operations for consistent behavior across decoders:

```rust
use decoder_chains_common::hashing;

// Single SHA-256
let hash = hashing::sha256(data);

// Double SHA-256 (Bitcoin-style)
let hash = hashing::sha256_double(data);

// Keccak-256 (Ethereum-style)
let hash = hashing::keccak256(data);

// Fixed-size array versions
let hash: [u8; 32] = hashing::sha256_array(data);
let hash: [u8; 32] = hashing::keccak256_array(data);
```

**Functions:**
- `sha256()` - Single SHA-256 hash
- `sha256_double()` - Double SHA-256 (Bitcoin, Litecoin, Dogecoin)
- `keccak256()` - Keccak-256 (Ethereum, EVM chains)
- `sha256_array()`, `sha256_double_array()`, `keccak256_array()` - Fixed-size array versions

### 🔌 `hooks` - Hook Execution Helpers

Helper functions for standardized hook execution patterns:

```rust
use decoder_chains_common::hooks;
use universal_decoder_core::HookRegistry;

fn decode(raw_bytes: &[u8], registry: &HookRegistry) -> Result<Transaction> {
    // Execute pre-decode hooks
    hooks::execute_pre_decode_hooks(registry, raw_bytes)?;

    // Perform decoding
    let tx = parse_transaction(raw_bytes)?;

    // Execute post-decode hooks
    hooks::execute_post_decode_hooks(registry, raw_bytes, Some(&tx as &dyn std::any::Any))?;

    Ok(tx)
}

// Or use the convenience function
fn decode_with_hooks(raw_bytes: &[u8], registry: &HookRegistry) -> Result<Transaction> {
    hooks::decode_with_hooks(raw_bytes, registry, MyDecoder::decode)
}
```

**Functions:**
- `execute_pre_decode_hooks()` - Execute hooks before decoding
- `execute_post_decode_hooks()` - Execute hooks after decoding
- `execute_post_canonicalize_hooks()` - Execute hooks after canonicalization
- `decode_with_hooks()` - Generic function combining all hook stages

### 🌐 `chains` - Pre-defined Chain Identities

Registry of well-known blockchain identities, eliminating boilerplate:

```rust
use decoder_chains_common::chains;
use universal_decoder_core::prelude::ChainIdentity;

// Use pre-defined chain identities
let bitcoin = chains::BITCOIN;
let ethereum = chains::ETHEREUM;
let solana = chains::SOLANA;

assert_eq!(bitcoin.chain_id(), 0);
assert_eq!(bitcoin.chain_name(), "Bitcoin");
assert_eq!(bitcoin.chain_family(), ChainFamily::Utxo);

// Lookup by ID or name
let chain = chains::lookup_by_id(1)?; // Ethereum
let chain = chains::lookup_by_name("Bitcoin")?; // Case-insensitive
```

**Available Chains:**

| Chain | Constant | ID | Family |
|-------|----------|-----|--------|
| Bitcoin | `BITCOIN` | 0 | UTXO |
| Ethereum | `ETHEREUM` | 1 | Account |
| Litecoin | `LITECOIN` | 2 | UTXO |
| Dogecoin | `DOGECOIN` | 3 | UTXO |
| Solana | `SOLANA` | 101 | Instruction |
| Polygon | `POLYGON` | 137 | Account |
| Aptos | `APTOS` | 1001 | Account |
| Cosmos | `COSMOS` | 118 | Account |
| ... and 14 more |

**Functions:**
- `lookup_by_id(id)` - Find chain by numeric ID
- `lookup_by_name(name)` - Find chain by name (case-insensitive)
- `ALL_CHAINS` - Array of all registered chains

## Usage in Decoders

### Before (Without Common Library)

```rust
// decoder-bitcoin/src/lib.rs - 25+ lines of boilerplate

#[derive(Debug, Clone, Copy)]
pub struct BitcoinChain;

impl ChainIdentity for BitcoinChain {
    fn chain_id(&self) -> u64 { 0 }
    fn chain_name(&self) -> &str { "Bitcoin" }
    fn chain_family(&self) -> ChainFamily { ChainFamily::Utxo }
}

impl ChainDecoder for BitcoinDecoder {
    type Chain = BitcoinChain;
    fn chain() -> Self::Chain { BitcoinChain }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("..."));
        }
        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure("..."));
        }
        if raw_bytes.len() > MAX_SIZE {
            return Err(DecoderError::invalid_structure("..."));
        }
        Ok(())
    }
}
```

### After (With Common Library)

```rust
// decoder-bitcoin/src/lib.rs - 5 lines, simplified

use decoder_chains_common::prelude::*;

impl ChainDecoder for BitcoinDecoder {
    type Chain = decoder_chains_common::chains::ChainInfo;
    fn chain() -> Self::Chain { chains::BITCOIN }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        validation::validate_format(raw_bytes, 10, MAX_SIZE, "Bitcoin")
    }
}
```

**Savings:** ~80% reduction in boilerplate code per decoder!

## Design Principles

1. **No `unsafe` code** - All functions are panic-free and memory-safe
2. **Minimal dependencies** - Only depends on `universal-decoder-core` and crypto primitives
3. **Trait-based** - Works seamlessly with trait-based decoder architecture
4. **Well-tested** - 26 unit tests, comprehensive property tests
5. **Documented** - Every public function has examples and doc tests

## Benefits

### For Decoder Implementers

- ✅ **Less code to write** - Pre-built validation, hashing, chain identities
- ✅ **Less code to test** - Common logic is already tested
- ✅ **Less code to audit** - Reduced attack surface per decoder
- ✅ **Consistency** - Standard patterns across all decoders

### For the Project

- ✅ **Reduced LOC** - Eliminates 500-800 lines of duplication
- ✅ **Easier maintenance** - Fix once, benefit everywhere
- ✅ **Faster development** - New decoders require minimal boilerplate
- ✅ **Better quality** - Common code is more thoroughly tested

## Examples

### Complete Decoder Using Common Library

```rust
use decoder_chains_common::prelude::*;
use decoder_primitives::prelude::*;
use universal_decoder_core::prelude::*;

pub struct MyChainDecoder;

impl ChainDecoder for MyChainDecoder {
    type TxSpecific = MyTransaction;
    type Chain = decoder_chains_common::chains::ChainInfo;

    fn chain() -> Self::Chain {
        chains::lookup_by_name("MyChain").expect("Chain registered")
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate using common library
        validation::validate_format(raw_bytes, 10, 1_000_000, "MyChain")?;

        // Parse transaction
        let tx = parse_my_transaction(raw_bytes)?;

        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        validation::validate_format(raw_bytes, 10, 1_000_000, "MyChain")
    }
}

impl Canonicalizer for MyTransaction {
    fn canonicalize(&self) -> Result<TxIR> {
        // Calculate hash using common library
        let hash = hashing::sha256(&self.raw_bytes);

        let metadata = TxMetadata {
            hash: hash.clone(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len() as u64,
            fee: self.fee,
        };

        // ... build TxIR ...

        Ok(TxIR::new(
            &Self::chain(),
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }
}
```

### With Hook Support

```rust
pub fn decode_with_hooks(
    raw_bytes: &[u8],
    registry: &HookRegistry,
) -> Result<MyTransaction> {
    // Use common hook helper
    hooks::decode_with_hooks(raw_bytes, registry, MyChainDecoder::decode)
}
```

## Integration

Add to your decoder's `Cargo.toml`:

```toml
[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
decoder-chains-common = { path = "../decoder-chains-common" }
```

Import in your decoder:

```rust
use decoder_chains_common::prelude::*;
```

The prelude re-exports:
- `chains` - Chain identity registry
- `validation` - Validation functions
- `hashing` - Hash functions
- `hooks` - Hook helpers

## Testing

```bash
# Run all tests
cargo test -p decoder-chains-common

# Run with verbose output
cargo test -p decoder-chains-common -- --nocapture

# Run specific test module
cargo test -p decoder-chains-common validation::tests
```

## Contributing

When adding new chains to the registry:

1. Add the chain constant to `src/chains.rs`
2. Add to `ALL_CHAINS` array
3. Run tests to ensure unique IDs and names
4. Update this README with the new chain

## See Also

- [universal-decoder-core](../universal-decoder-core/README.md) - Core traits and types
- [decoder-primitives](../decoder-primitives/README.md) - Low-level byte readers
- [decoder-encodings](../decoder-encodings/README.md) - Chain-specific encodings
- [CLAUDE.md](../../CLAUDE.md) - Project design philosophy

## License

MIT OR Apache-2.0
