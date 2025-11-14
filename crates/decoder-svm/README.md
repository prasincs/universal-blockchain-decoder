# decoder-svm

SVM (Solana Virtual Machine) family decoder for the universal blockchain decoder.

## Overview

This decoder supports the entire SVM ecosystem, including:
- **Solana** (mainnet-beta, devnet, testnet)
- **Eclipse** (Ethereum-Solana hybrid)
- **Pyth Network** (oracle network)
- **Drift Protocol** (derivatives DEX)
- **Jito** (MEV infrastructure)
- Future SVM-based chains

## Architecture

The `decoder-svm` wraps the `decoder-solana` implementation and adds:
- Chain-specific identification and validation
- SVM chain registry (compile-time embedded)
- Support for SVM variants with different features

## Usage

```rust
use decoder_svm::{SvmDecoder, SvmChainId};
use universal_decoder_core::prelude::*;

// Decode a Solana mainnet transaction
let tx_bytes = &[...];
let tx = SvmDecoder::decode_with_chain(tx_bytes, SvmChainId::SolanaMainnet)?;

// Auto-detect chain (uses Solana mainnet by default)
let tx = SvmDecoder::decode(tx_bytes)?;
```

## Supported Chains

| Chain | ID | Status |
|-------|-----|--------|
| Solana Mainnet | 101 | ✅ Fully supported |
| Solana Devnet | 102 | ✅ Fully supported |
| Solana Testnet | 103 | ✅ Fully supported |
| Eclipse Mainnet | 201 | 🚧 Planned |
| Pyth Network | 301 | 🚧 Planned |
| Drift Protocol | 401 | 🚧 Planned |
| Jito | 501 | 🚧 Planned |

## Features

- ✅ Pure Rust implementation (zero runtime dependencies)
- ✅ Compile-time embedded chain registry
- ✅ Support for legacy and versioned transactions
- ✅ Instruction parsing
- 🚧 Address lookup tables (planned)
- 🚧 Chain-specific program validation (planned)

## Testing

```bash
cargo test -p decoder-svm
```

## Documentation

For more information, see:
- [Solana Documentation](https://docs.solana.com/)
- [SVM Specification](https://docs.solana.com/developing/programming-model/overview)
