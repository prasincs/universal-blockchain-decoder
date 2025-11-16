# Contributing to Universal Blockchain Decoder

Thank you for your interest in contributing to the Universal Blockchain Decoder! This document provides guidelines and information to help you contribute effectively.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Adding a New Chain Decoder](#adding-a-new-chain-decoder)
- [Areas for Contribution](#areas-for-contribution)
- [Communication](#communication)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Prerequisites

- **Rust 1.70+** - Install from [rust-lang.org](https://www.rust-lang.org/)
- **Git** - For version control
- **Cargo** - Comes with Rust

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/universal-blockchain-decoder.git
   cd universal-blockchain-decoder
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/prasincs/universal-blockchain-decoder.git
   ```

## Development Setup

### Build the Project

```bash
# Build all crates in the workspace
cargo build --workspace

# Build a specific crate
cargo build -p universal-decoder-core
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p decoder-bitcoin

# Run property-based tests with more iterations
PROPTEST_CASES=10000 cargo test --workspace
```

### Run Examples

```bash
# Run the simple decoder example
cargo run --example simple-decoder
```

### Check Your Code

Before committing, **always** run these commands:

```bash
# Format code
cargo fmt --all

# Check for linting issues
cargo clippy --all --all-targets --all-features -- -D warnings

# Run tests
cargo test --all
```

**IMPORTANT**: All CI checks must pass for your PR to be merged. Running these locally saves time!

## Code Style

We follow standard Rust conventions with some additional requirements:

### Formatting

- Use `cargo fmt` with the default configuration
- **Always run `cargo fmt --all` before committing**

### Linting

- **Zero clippy warnings** - We use `clippy` with `-D warnings` (warnings are errors)
- Common fixes:
  ```rust
  // ❌ Bad: Borrowed expression implements required traits
  Err(DecoderError::invalid(&format!("error: {}", x)))

  // ✅ Good: Direct ownership
  Err(DecoderError::invalid(format!("error: {}", x)))

  // ❌ Bad: Length comparison to zero
  if vec.len() > 0 { }

  // ✅ Good: Use is_empty()
  if !vec.is_empty() { }
  ```

### Documentation

- **All public APIs must have documentation comments**
- Include examples in doc comments when possible
- Use `///` for doc comments, `//` for inline comments
- Document all safety assumptions for `unsafe` code (though we avoid `unsafe` in core)

Example:
```rust
/// Decodes a Bitcoin transaction from raw bytes.
///
/// # Arguments
///
/// * `raw_bytes` - The raw transaction bytes in Bitcoin's wire format
///
/// # Returns
///
/// Returns a `Result` containing the decoded `BitcoinTransaction` or a `DecoderError`
///
/// # Examples
///
/// ```
/// use decoder_bitcoin::BitcoinDecoder;
/// use universal_decoder_core::ChainDecoder;
///
/// let raw_tx = &[/* transaction bytes */];
/// let tx = BitcoinDecoder::decode(raw_tx)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode(raw_bytes: &[u8]) -> Result<BitcoinTransaction> {
    // ...
}
```

### Design Principles

Follow the core design principles outlined in [CLAUDE.md](CLAUDE.md):

1. **Minimal Core** - Keep core library < 3000 LOC
2. **Trait-Based Extensibility** - Use traits, not enums for chains
3. **Canonical Serialization** - Use Borsh, never JSON for hashing
4. **No Unsafe Code** - Especially in core library
5. **Decoding Only** - This project does not support transaction encoding/construction

## Testing Requirements

We maintain world-class testing standards:

### Test Categories

1. **Unit Tests** - Test individual functions
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_decode_legacy_transaction() {
           let raw_tx = &[/* bytes */];
           let result = BitcoinDecoder::decode(raw_tx);
           assert!(result.is_ok());
       }
   }
   ```

2. **Property-Based Tests** - Using `proptest`
   ```rust
   use proptest::prelude::*;

   proptest! {
       #[test]
       fn canonicalize_is_deterministic(tx in arbitrary_tx()) {
           let bytes1 = tx.to_canonical_bytes()?;
           let bytes2 = tx.to_canonical_bytes()?;
           prop_assert_eq!(bytes1, bytes2);
       }
   }
   ```

3. **Integration Tests** - Using real blockchain data
   ```rust
   #[test]
   fn decode_real_bitcoin_transaction() {
       let raw_tx = include_bytes!("../fixtures/bitcoin/legacy/tx1.bin");
       let tx = BitcoinDecoder::decode(raw_tx).unwrap();
       assert_eq!(tx.version, 1);
   }
   ```

### Coverage Requirements

- **Unit tests**: All public APIs must have tests
- **Property tests**: Critical invariants must be tested:
  - Deterministic serialization
  - Roundtrip preservation
  - Panic-freedom
  - Hash determinism
- **Integration tests**: Use real blockchain data from `tests/fixtures/`

### Adding Test Fixtures

When adding integration tests:

1. Place fixtures in `tests/fixtures/{chain}/{category}/`
2. Add a README explaining the fixture
3. Include metadata (block height, tx hash, chain)
4. Prefer real mainnet transactions when possible

Example structure:
```
tests/fixtures/bitcoin/segwit/
├── README.md              # Describes the test case
├── tx1.bin                # Raw transaction bytes
└── tx1_expected.json      # Expected decoded values
```

## Pull Request Process

### Before Submitting

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the code style guidelines

3. **Run all checks**:
   ```bash
   # Format
   cargo fmt --all

   # Lint (must pass with zero warnings)
   cargo clippy --all --all-targets --all-features -- -D warnings

   # Test
   cargo test --all

   # Build docs
   cargo doc --all --no-deps
   ```

4. **Update documentation**:
   - Update relevant documentation in `docs/` if needed
   - Update README.md if adding new features
   - Add/update inline documentation
   - Update CHANGELOG.md

5. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add support for Dogecoin decoder"
   ```

   Use conventional commit messages:
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `docs:` - Documentation changes
   - `test:` - Adding/updating tests
   - `refactor:` - Code refactoring
   - `perf:` - Performance improvements
   - `chore:` - Maintenance tasks

### Submitting the PR

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request** on GitHub

3. **Fill out the PR template** with:
   - Description of changes
   - Related issues (if any)
   - Testing performed
   - Screenshots (if UI-related)

4. **Ensure CI passes** - All GitHub Actions workflows must succeed

5. **Respond to review feedback** - Maintainers may request changes

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] `cargo fmt --all` has been run
- [ ] `cargo clippy --all --all-targets --all-features -- -D warnings` passes with zero warnings
- [ ] All tests pass (`cargo test --all`)
- [ ] New/updated tests added for changes
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (for user-facing changes)
- [ ] Commit messages follow conventional commits format
- [ ] PR description clearly explains the changes

## Adding a New Chain Decoder

One of the most valuable contributions is adding support for new blockchains!

### Step-by-Step Guide

#### 1. Create a New Crate

```bash
# Create the crate directory
mkdir -p crates/decoder-yourchain
cd crates/decoder-yourchain

# Create Cargo.toml
cat > Cargo.toml <<EOF
[package]
name = "decoder-yourchain"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true

[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
decoder-primitives = { path = "../decoder-primitives" }
serde = { workspace = true }
borsh = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
# Add chain-specific libraries for validation only
yourchain = "x.y.z"
EOF

# Create src/lib.rs
mkdir src
touch src/lib.rs
```

#### 2. Define Your Transaction Type

```rust
// src/lib.rs
use serde::{Deserialize, Serialize};

/// YourChain transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourChainTransaction {
    pub version: u32,
    // Add your chain-specific fields
}
```

#### 3. Implement ChainDecoder Trait

```rust
use universal_decoder_core::{ChainDecoder, ChainId, DecoderError};

pub struct YourChainDecoder;

impl ChainDecoder for YourChainDecoder {
    type TxSpecific = YourChainTransaction;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific, DecoderError> {
        // Implement parsing logic
        // Return DecoderError::InvalidStructure if parsing fails
        todo!("Parse your chain's transaction format")
    }

    fn chain_id() -> ChainId {
        ChainId::Custom(YOUR_CHAIN_ID)
    }
}
```

#### 4. Implement Canonicalizer Trait

```rust
use universal_decoder_core::{Canonicalizer, TxIR, Metadata, Authorization, Operation};

impl<'a> Canonicalizer<'a> for YourChainTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, { Self::VERSION }>, DecoderError> {
        // Map your transaction to universal IR
        let metadata = Metadata {
            tx_hash: self.compute_hash()?,
            timestamp: None,
            block_height: None,
        };

        let authorization = Authorization {
            signatures: vec![/* extract signatures */],
            public_keys: vec![/* extract public keys */],
        };

        let operations = vec![/* map to operations */];

        TxIR::new(
            &Self::chain_id(),
            metadata,
            authorization,
            operations,
            vec![], // state_deltas
        )
    }
}
```

#### 5. Implement TxHashable Trait

```rust
use universal_decoder_core::TxHashable;

impl TxHashable for YourChainTransaction {
    fn tx_hash(&self) -> Result<Vec<u8>, DecoderError> {
        // Compute transaction hash using your chain's algorithm
        todo!("Implement chain-specific hashing")
    }
}
```

#### 6. Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_transaction() {
        let raw_tx = &[/* test transaction bytes */];
        let result = YourChainDecoder::decode(raw_tx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_canonicalize() {
        let tx = YourChainTransaction { /* ... */ };
        let tx_ir = tx.canonicalize();
        assert!(tx_ir.is_ok());
    }
}
```

#### 7. Add Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Define arbitrary transaction generator
    fn arbitrary_tx() -> impl Strategy<Value = YourChainTransaction> {
        // Generate random valid transactions
        todo!()
    }

    proptest! {
        #[test]
        fn canonicalize_is_deterministic(tx in arbitrary_tx()) {
            let tx_ir1 = tx.canonicalize()?;
            let tx_ir2 = tx.canonicalize()?;
            prop_assert_eq!(
                tx_ir1.to_canonical_bytes()?,
                tx_ir2.to_canonical_bytes()?
            );
        }

        #[test]
        fn decode_never_panics(bytes: Vec<u8>) {
            // Should return Err, not panic
            let _ = YourChainDecoder::decode(&bytes);
        }
    }
}
```

#### 8. Add to Workspace

Edit the root `Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing members
    "crates/decoder-yourchain",
]
```

#### 9. Create README

Create `crates/decoder-yourchain/README.md`:

```markdown
# decoder-yourchain

Decoder for YourChain blockchain transactions.

## Usage

\`\`\`rust
use decoder_yourchain::YourChainDecoder;
use universal_decoder_core::ChainDecoder;

let raw_tx = &[/* transaction bytes */];
let tx = YourChainDecoder::decode(raw_tx)?;
let tx_ir = tx.canonicalize()?;
\`\`\`

## Implementation Status

- [x] Basic decoding
- [x] Canonicalization to TxIR
- [ ] All transaction types
- [ ] Property tests

## Chain-Specific Details

- Chain ID: YOUR_CHAIN_ID
- Transaction Format: [Brief description]
- Hashing Algorithm: [e.g., SHA-256]
```

#### 10. Update Documentation

- Update the main README.md "Supported Blockchains" table
- Add any chain-specific documentation to `docs/`
- Update CHANGELOG.md

### Example: See Existing Decoders

For reference implementations:
- **Bitcoin**: `crates/decoder-bitcoin/`
- **Ethereum**: `crates/decoder-ethereum/`
- **Cosmos SDK**: `crates/decoder-cosmos-sdk/`

## Areas for Contribution

We welcome contributions in these areas:

### 1. New Blockchain Support

**High Priority Chains:**
- Cardano (UTXO model with extended features)
- Polkadot (relay chain + parachains)
- Tezos (account model with smart contracts)
- Algorand (pure proof-of-stake)
- Near Protocol (sharded account model)

**Moderate Priority:**
- Bitcoin forks (Litecoin, Dogecoin, Bitcoin Cash)
- EVM-compatible chains (already have multi-chain support)
- Layer 2 solutions (Arbitrum, Optimism - some implemented)

### 2. Testing & Quality

- Add more property-based tests
- Add real mainnet transaction fixtures
- Improve test coverage (target: 80%+)
- Add fuzzing for parsers
- Benchmark performance

### 3. Optimization

- Profile hot paths (decoding, hashing)
- Optimize memory allocations
- Reduce binary size
- WASM performance improvements

### 4. Formal Verification

- Verus annotations for core library
- Prove key invariants (determinism, injectivity)
- Verify panic-freedom
- Resource bounds proofs

### 5. Documentation

- More usage examples
- Integration guides (block explorers, indexers)
- Video tutorials
- Blog posts about architecture
- Comparison with other decoders

### 6. Tooling

- CLI improvements
- WASM demo enhancements
- VS Code extension for transaction visualization
- Performance profiling tools

### 7. Core Library

- Improve error messages
- Add more hook stages
- Enhanced privacy features
- Better type-level constraints

## Communication

### Where to Ask Questions

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, design discussions, ideas
- **Pull Requests**: Code reviews, specific implementation questions

### Response Time

- We aim to respond to issues/PRs within 48 hours
- Complex contributions may require multiple review rounds
- Please be patient and constructive in discussions

### Getting Help

If you're stuck:
1. Check existing documentation in `docs/`
2. Look at similar implementations (e.g., Bitcoin decoder)
3. Ask in GitHub Discussions
4. Reference the [Rust Book](https://doc.rust-lang.org/book/) for Rust questions

## Recognition

Contributors will be:
- Listed in AUTHORS.md
- Credited in release notes
- Mentioned in the project README

Thank you for contributing to Universal Blockchain Decoder! 🙏

---

**Questions?** Open an issue or start a discussion on GitHub.
