# Universal Blockchain Decoder

A Rust library for decoding transactions from different blockchain protocols (Bitcoin, Ethereum, Solana, etc.) into a unified intermediate representation (TxIR).

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Live Demo](https://img.shields.io/badge/🌐_Live_Demo-trustless--txir-blueviolet.svg)](https://trustless-txir.netlify.app)

## Overview

The Universal Blockchain Decoder provides a unified framework for parsing blockchain transactions. Like Pandoc for documents, it converts different blockchain formats into a canonical intermediate representation.

**What it does:**
- Parses raw transaction bytes from Bitcoin, Ethereum, Cosmos, Solana, etc.
- Converts to a unified `TxIR` (Transaction Intermediate Representation)
- Provides type-safe API using Rust traits and const generics
- Runs deterministic canonicalization for hashing/analysis

**What it doesn't do:**
- Transaction construction/encoding
- Transaction signing
- Fee estimation
- Network operations (broadcasting, RPC calls)

For transaction construction, use chain-specific SDKs (`bitcoin` crate, `alloy`, `solana-sdk`).

## Live Demo

[https://trustless-txir.netlify.app](https://trustless-txir.netlify.app)

Try decoding transactions from 500+ EVM chains, Bitcoin, and Cosmos in your browser. All processing is local.

## Quick Start

### Installation

```toml
[dependencies]
universal-decoder-core = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
decoder-bitcoin = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
decoder-ethereum = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
```

### Basic Usage

```rust
use universal_decoder_core::prelude::*;
use decoder_bitcoin::BitcoinDecoder;

// Decode Bitcoin transaction
let tx = BitcoinDecoder::decode(raw_tx_bytes)?;

// Convert to universal IR
let tx_ir = tx.canonicalize()?;

// Access normalized data
println!("Chain: {:?}", tx_ir.chain.name);
println!("Operations: {}", tx_ir.operations.len());
```

### With Hooks

```rust
// Set up validation/logging
let registry = HookRegistryBuilder::new()
    .with_size_limit(1_000_000)
    .with_logging("my-app".to_string(), vec![HookStage::PreDecode])
    .build();

let tx = decoder_bitcoin::decode_with_hooks(raw_bytes, &registry)?;
```

See [LIBRARY_USAGE.md](LIBRARY_USAGE.md) for patterns and examples.

## Architecture

Three-layer pipeline:

```
Raw Bytes → ChainDecoder → Chain-Specific Type → Canonicalizer → TxIR
```

**Core traits:**
- `ChainDecoder`: Parse chain-specific format
- `Canonicalizer`: Transform to TxIR
- `Hook`: Custom processing pipeline

**TxIR structure:**
```rust
pub struct TxIR<'a, const V: u8> {
    pub chain: ChainRef,              // Chain info
    pub metadata: TxMetadata,          // Hash, block, size
    pub authorization: AuthorizationPackage,  // Signatures
    pub operations: Vec<Operation>,    // Transfers, calls, etc.
    pub state_deltas: StateDeltas,     // Inputs/outputs
}
```

## Supported Blockchains

| Chain | Status | Model |
|-------|--------|-------|
| Bitcoin | ✅ Implemented | UTXO |
| Ethereum | ✅ Implemented | Account (legacy + EIP-1559) |
| 500+ EVM chains | ✅ Implemented | Account |
| 100+ Cosmos chains | ✅ Implemented | Account |
| OP Stack | ✅ Implemented | Account |
| TON | ✅ Implemented | Actor |
| Starknet | ✅ Implemented | Account |
| Solana | 🚧 In progress | Instruction |

See [ROADMAP.md](ROADMAP.md) for upcoming chains.

## Testing

- 322 unit tests
- 100+ property-based tests (proptest)
- Zero clippy warnings (`-D warnings`)
- CI runs on stable + beta Rust

Property tests verify:
- Deterministic serialization: `encode(x) = encode(x)`
- Panic-freedom: No panics on arbitrary input
- Hash determinism: `hash(x) = hash(x)`

See [docs/TESTING_STRATEGY.md](docs/TESTING_STRATEGY.md) for details.

## Documentation

- [LIBRARY_USAGE.md](LIBRARY_USAGE.md) - How to use as a library
- [INDEXER_COMPARISON.md](INDEXER_COMPARISON.md) - vs The Graph, Subsquid, etc.
- [CLAUDE.md](CLAUDE.md) - Design philosophy and architecture
- [ROADMAP.md](ROADMAP.md) - Implementation roadmap

## Project Structure

```
crates/
├── universal-decoder-core/    # Core traits and TxIR
├── decoder-bitcoin/           # Bitcoin decoder
├── decoder-ethereum/          # Ethereum decoder
├── decoder-evm/              # 500+ EVM chains
├── decoder-cosmos-sdk/       # 100+ Cosmos chains
└── decoder-op-stack/         # OP Stack L2s
```

## Use Cases

- Multi-chain indexers
- Block explorers
- Transaction analytics
- Forensic analysis
- Cross-chain monitoring

## Development

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Format + Lint
cargo fmt --all
cargo clippy --all --all-targets --all-features -- -D warnings
```

## Contributing

Contributions welcome. Focus areas:
- Additional blockchain support
- Performance optimization
- Documentation improvements
- More test coverage

To add a new chain:
1. Create `decoder-yourchain` crate
2. Implement `ChainDecoder` trait
3. Implement `Canonicalizer` trait
4. Add tests

## Security

**Status**: Experimental. Not audited for production use.

The library prioritizes:
- Memory safety (Rust ownership)
- Type safety (const generics, traits)
- Canonical encoding (prevents malleability)
- Minimal dependencies

## License

MIT OR Apache-2.0

## Links

- Issues: https://github.com/prasincs/universal-blockchain-decoder/issues
- Discussions: https://github.com/prasincs/universal-blockchain-decoder/discussions

---

**Status**: Active Development | **Version**: 0.1.0 | **Rust**: 1.70+
