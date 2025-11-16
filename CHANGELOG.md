# Changelog

All notable changes to the Universal Blockchain Decoder project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Additional blockchain decoders (Cardano, Polkadot, Tezos)
- Professional security audit
- Complete formal verification with Verus
- crates.io publication
- v1.0.0 stable release

## [0.1.0-alpha] - 2025-01-XX

### Overview

Initial alpha release of the Universal Blockchain Decoder - a compile-time safe, universal transaction decoder architecture for heterogeneous blockchains. This release establishes the core architecture, trait system, and reference implementations for Bitcoin, Ethereum, and other major chains.

⚠️ **ALPHA SOFTWARE**: This release is for testing and feedback only. Do not use in production without thorough review.

### Added - Core Architecture

#### Type System & Traits
- **TxIR (Transaction Intermediate Representation)**: Canonical format normalizing transactions across blockchain models
  - UTXO model (Bitcoin)
  - Account model (Ethereum)
  - Instruction model (Solana)
- **ChainDecoder trait**: Parse chain-specific raw bytes
- **Canonicalizer trait**: Transform chain-specific types into universal TxIR
- **TxHashable trait**: Compute transaction hashes deterministically
- **Hook system**: Extensible processing at various pipeline stages
  - Pre-decode, post-decode, pre-canonicalize, post-canonicalize stages
  - Hook registry with configurable size limits and logging

#### Core Library Features
- Const generic version constraints (`TxIR<'a, const V: u8>`)
- Associated types for type-safe chain-specific logic
- Zero-cost abstractions via static dispatch
- Canonical serialization using Borsh (deterministic, non-malleable)
- Minimal dependencies (5 production deps: serde, borsh, thiserror, sha2, sha3)
- Supply chain security via git subtree vendoring (hex crate)

### Added - Blockchain Decoders

#### Bitcoin Family
- **Bitcoin** (`decoder-bitcoin`)
  - Legacy transactions (P2PKH, P2SH)
  - SegWit transactions (P2WPKH, P2WSH)
  - Witness data parsing
  - Script decoding
  - 47 tests (unit + property)

- **Litecoin** (`decoder-litecoin`)
  - Full transaction support
  - MWEB (MimbleWimble Extension Blocks) awareness

- **Bitcoin Cash** (`decoder-bitcoin-cash`)
  - CashAddr format support

- **Dogecoin** (`decoder-dogecoin`)
  - AuxPoW awareness

- **Dash** (`decoder-dash`)
  - InstantSend and ChainLocks support

- **Zcash** (`decoder-zcash`)
  - Sapling transaction support
  - Viewing key decryption
  - ZIP-243 test vectors
  - Privacy-preserving features

#### Ethereum & EVM Ecosystem
- **Ethereum** (`decoder-ethereum`)
  - Legacy transactions (pre-EIP-1559)
  - EIP-1559 transactions (dynamic fees)
  - EIP-2930 (access lists)
  - RLP encoding/decoding
  - 28 property tests

- **EVM Multi-Chain** (`decoder-evm`)
  - Support for 500+ EVM-compatible chains
  - Chain registry vendored via git subtree
  - Compile-time chain data embedding
  - 7 property tests

- **Arbitrum** (`decoder-arbitrum`)
  - L2 transaction support
  - Nitro-specific features

- **Optimism** (`decoder-optimism`)
  - Optimistic rollup transaction support

- **OP Stack** (`decoder-op-stack`)
  - Universal decoder for OP Stack chains
  - Superchain registry vendored

- **Polygon** (`decoder-polygon`)
  - Plasma and PoS chain support

- **Avalanche** (`decoder-avalanche`)
  - C-Chain transaction support

- **BNB Chain** (`decoder-bnb-chain`)
  - BSC transaction support

#### Other Blockchains
- **Solana** (`decoder-solana`)
  - Compact-u16 encoding
  - Instruction-based model
  - 15 tests

- **Cosmos SDK** (`decoder-cosmos-sdk`)
  - Protobuf transaction decoding
  - Multi-chain support (100+ Cosmos chains)
  - Chain registry vendored
  - 15 property tests

- **Stellar** (`decoder-stellar`)
  - XDR transaction support

### Added - Testing Infrastructure

#### Test Coverage
- **322 unit tests** across all crates (all passing ✅)
- **100+ property-based tests** using proptest
  - 1,000 iterations per test in CI
  - 10,000 iterations in nightly builds
- **Zero clippy warnings** with `-D warnings` enforcement
- **Zero formatting issues** with strict `cargo fmt` checks

#### Property Tests
Verified invariants:
- ✅ Deterministic serialization: `encode(x) = encode(x)`
- ✅ Roundtrip preservation: `decode(encode(x)) = x`
- ✅ Panic-freedom: Decoder never panics on arbitrary input
- ✅ Hash determinism: `hash(x) = hash(x)`
- ✅ Hash collision resistance: `x ≠ y ⟹ hash(x) ≠ hash(y)`
- ✅ Boundary value handling: `u64::MAX`, empty vectors, etc.

#### Test Organization
- Shared test utilities (`decoder-test-utils`)
- Real transaction fixtures in `tests/fixtures/`
- Integration tests with mainnet data
- Fuzzing infrastructure (cargo-fuzz)

### Added - CI/CD

**8 GitHub Actions workflows:**
1. **test.yml** - Comprehensive test suite
   - Unit tests (stable + beta Rust)
   - Property tests (1,000 iterations)
   - Integration tests
   - Format checks
   - Clippy linting (-D warnings)
   - Security audit (cargo-audit)
   - Documentation builds
   - Minimal versions check

2. **nightly.yml** - Extended testing
   - Property tests with 10,000 iterations
   - Scheduled nightly runs

3. **coverage.yml** - Code coverage
   - cargo-llvm-cov integration
   - Codecov upload
   - 80% coverage threshold (lenient during alpha)

4. **verus.yml** - Formal verification
   - Verus integration for formal proofs

5. **deploy-wasm-demo.yml** - WASM deployment
   - Builds WASM bindings
   - Deploys to GitHub Pages

6. **auto-update-docs-smart.yml** - Documentation automation

7. **ai-refactor-suggest.yml** - AI-assisted refactoring

8. Additional workflow for security audits

### Added - Developer Tools

#### CLI
- **universal-decoder-cli** (`crates/universal-decoder-cli`)
  - Multi-chain transaction decoding from command line
  - Dynamic chain registry
  - Privacy support for sensitive transactions
  - JSON output format

#### WASM
- **universal-decoder-wasm** (`crates/universal-decoder-wasm`)
  - Browser-based transaction decoder
  - Comprehensive README with embedding guide
  - Performance metrics
  - Deployment to GitHub Pages
  - Interactive demo interface

#### Build Tools
- Git subtree vendoring tools
- Chain registry update scripts
- Automated documentation generators

### Added - Documentation

#### Project Documentation
- `README.md` - Comprehensive overview with architecture diagrams
- `CLAUDE.md` - Core design principles and development guide (1,487 lines)
- `ROADMAP.md` - Phased development plan (98KB)
- `PRODUCT_VISION.md` - Long-term vision (73KB)
- `LIBRARY_USAGE.md` - Library usage patterns
- `CLI.md` - CLI documentation
- `CONTRIBUTING.md` - Contribution guidelines
- `CODE_OF_CONDUCT.md` - Community standards
- `SECURITY.md` - Security policy
- `CHANGELOG.md` - This file

#### Technical Documentation (docs/)
**47 comprehensive markdown files** including:
- `TESTING_STRATEGY.md` - 5-level testing pyramid
- `FORMAL_VERIFICATION.md` - Verus integration plan
- `GIT_SUBTREE_VENDORING.md` - Verifiable dependency vendoring
- `DECODER_DEPENDENCY_STRATEGY.md` - Pure Rust decoder pattern
- `CANONICAL_SERIALIZATION.md` - Borsh usage guidelines
- `TRAIT_BASED_ARCHITECTURE.md` - Trait design patterns
- `WASM_DEMO.md` - Interactive demo documentation
- Chain-specific implementation guides
- Dependency audits
- Architecture refactoring plans

### Added - Examples

- `examples/simple-decoder/` - Complete working example (211 lines)
  - Bitcoin transaction decoding
  - Ethereum transaction decoding
  - Custom hook creation
  - TxIR canonicalization
  - JSON serialization

### Added - Shared Libraries

- **decoder-primitives** - Common types and utilities
- **decoder-encodings** - Encoding utilities (RLP, Borsh, etc.)
- **decoder-chains-common** - Shared chain patterns
- **decoder-test-utils** - Test infrastructure
- **decoder-crypto-zk** - ZK cryptography (Starknet, etc.)
  - Vendored starknet-crypto

### Security

#### Features
- Memory-safe by design (Rust's ownership model)
- Canonical encoding prevents transaction malleability
- Minimal attack surface (5 production dependencies)
- Supply chain security via git subtree vendoring
- Airgapped operation (no runtime network dependencies)
- cargo-audit on every commit

#### Limitations
- ⚠️ **Not audited** - No professional security audit yet
- ⚠️ **Alpha software** - API may change
- ⚠️ **Experimental** - Do not use in production

See [SECURITY.md](SECURITY.md) for vulnerability reporting.

### Project Scope

**In Scope** ✅:
- Transaction decoding (chain-specific bytes → TxIR)
- Canonical serialization (TxIR → Borsh bytes)
- Transaction validation and analysis
- Signature verification (checking existing signatures)

**Out of Scope** ❌:
- Transaction encoding/construction
- Transaction signing
- Fee estimation
- UTXO selection or nonce management
- Transaction broadcasting

See [CLAUDE.md](CLAUDE.md#project-scope-decoding-only-) for rationale.

### Design Principles

1. **Minimal Core** - Core library < 3000 LOC (currently ~2700)
2. **Formally Verifiable** - Verus annotations for critical paths
3. **Reviewable** - Audit-friendly code
4. **Trait-Based Extensibility** - Add chains without modifying core
5. **Canonical Serialization** - Borsh for determinism
6. **Zero-Cost Abstractions** - Static dispatch
7. **Layered Security** - Core is trusted, decoders are pluggable
8. **Supply Chain Security** - Minimal dependencies, vendoring
9. **Comprehensive Testing** - Unit, property, integration, fuzz
10. **Documentation as Code** - Inline docs, examples

### Dependencies

#### Production Dependencies (Core)
```toml
borsh = "1.3"        # Canonical serialization
serde = "1.0.127"    # Serialization framework
thiserror = "1.0.30" # Error handling
sha2 = "0.10"        # SHA-256 hashing
sha3 = "0.10"        # Keccak hashing
```

#### Vendored Dependencies
- `hex` - Vendored via git subtree for verifiable supply chain
- Starknet crypto - Vendored in decoder-crypto-zk

#### Dev Dependencies
- `proptest` - Property-based testing
- `criterion` - Benchmarking
- `serde_json` - Display only (not for canonical encoding)
- Chain-specific libraries for validation (bitcoin, alloy, etc.)

### Performance

- Zero-cost abstractions via static dispatch
- Minimal allocations
- Efficient parsers
- WASM-compatible (runs in browser)

### Breaking Changes

N/A - Initial release

### Deprecated

None

### Removed

None

### Fixed

None (initial release)

### Known Issues

- Some decoder integration tests are disabled pending Phase 2 pure Rust implementations
- Coverage not yet at 80% target (expected during active development)
- Some TODOs in code for features planned in future phases
- SECURITY.md and CODE_OF_CONDUCT.md have placeholder contact information

See [GitHub Issues](https://github.com/prasincs/universal-blockchain-decoder/issues) for tracked issues.

### Contributors

- **Lead Developer**: [INSERT NAME]
- **Built with**: Claude (Anthropic) - $1000 credit program
- Special thanks to all contributors (see [AUTHORS.md](AUTHORS.md))

### Acknowledgments

- Inspired by Pandoc's universal document conversion approach
- Built on canonical serialization research (SCALE, BCS, Borsh)
- Leverages Rust's advanced type system features
- Test vectors from blockchain reference implementations

---

## Release Versioning

We follow [Semantic Versioning](https://semver.org/):

- **0.x.x** - Pre-1.0 releases (breaking changes allowed)
- **1.0.0** - First stable release (breaking changes require major version bump)
- **1.x.x** - Minor releases (new features, backward compatible)
- **1.x.y** - Patch releases (bug fixes, backward compatible)

### Alpha/Beta Releases

- **0.1.0-alpha** - Initial alpha (current)
- **0.2.0-beta** - Beta with pure Rust decoders
- **0.3.0-rc.1** - Release candidate
- **1.0.0** - Stable release (post-audit)

---

[Unreleased]: https://github.com/prasincs/universal-blockchain-decoder/compare/v0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/prasincs/universal-blockchain-decoder/releases/tag/v0.1.0-alpha
