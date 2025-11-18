# decoder-bittensor

Bittensor (TAO) transaction decoder for the Universal Blockchain Decoder.

## Overview

This crate provides pure Rust decoding for Bittensor blockchain transactions. Bittensor is a Substrate-based blockchain that implements a proof-of-intelligence consensus mechanism for decentralized machine learning.

## Features

- **Pure Rust Implementation**: No external blockchain libraries in production dependencies
- **SCALE Encoding**: Full support for Substrate's SCALE codec
- **Comprehensive Testing**: Unit, property-based, and integration tests
- **Canonical Serialization**: Deterministic Borsh encoding for hashing and verification
- **Bittensor-Specific Pallets**: Support for SubtensorModule, Registry, and other TAO pallets

## Usage

```rust
use decoder_bittensor::{BittensorDecoder, BittensorChain};
use decoder_primitives::prelude::*;

// Decode a raw SCALE-encoded extrinsic
let tx_bytes = /* your transaction bytes */;
let tx = BittensorDecoder::decode(&tx_bytes)?;

// Get transaction hash (Blake2b-512)
println!("TX Hash: {:?}", tx.tx_hash);

// Parse the call
let call = tx.call()?;
println!("Pallet: {}", call.pallet_name());
println!("Call: {}", call.call_name());

// Canonicalize to TxIR
let tx_ir = tx.canonicalize()?;
println!("Operations: {}", tx_ir.operations.len());
```

## Bittensor Architecture

### Chain Identity

- **Chain Name**: Bittensor
- **Chain Family**: Account-based (Substrate)
- **Native Token**: TAO (9 decimals)
- **Address Format**: SS58 encoding (32-byte account IDs)

### Supported Pallets

| Pallet Index | Pallet Name | Description |
|--------------|-------------|-------------|
| 0 | System | System-level operations |
| 4 | Balances | Token transfers and balance management |
| 7 | SubtensorModule | Bittensor-specific: set_weights, staking, registration |
| 11 | Utility | Batch operations |
| 13 | Multisig | Multi-signature accounts |
| 15 | Registry | Bittensor subnet registry |

### Common Operations

1. **Balances::transfer** - Transfer TAO tokens
2. **SubtensorModule::set_weights** - Update neuron weights
3. **SubtensorModule::add_stake** - Stake TAO
4. **SubtensorModule::remove_stake** - Unstake TAO
5. **SubtensorModule::register** - Register a neuron
6. **SubtensorModule::serve_axon** - Serve an axon endpoint

## Transaction Structure

Bittensor transactions follow Substrate's extrinsic format:

```
[CompactLength][Version][Address][Signature][Era][Nonce][Tip][PalletCall]
```

- **CompactLength**: SCALE-encoded length of the extrinsic
- **Version**: Version byte (bit 7 = signed flag)
- **Address**: SS58-encoded account (32 bytes)
- **Signature**: Sr25519, Ed25519, or ECDSA (64-65 bytes)
- **Era**: Transaction mortality (immortal or mortal)
- **Nonce**: Account nonce (compact-encoded)
- **Tip**: Block producer tip (compact-encoded u128)
- **PalletCall**: Pallet index + call index + parameters

## Testing

```bash
# Run all tests
cargo test --all

# Run property-based tests
cargo test --test bittensor_property

# Run integration tests
cargo test --test bittensor_integration

# Run with verbose output
cargo test -- --nocapture
```

## Dependencies

### Production
- `universal-decoder-core` - Core decoder traits and types
- `decoder-primitives` - Shared decoder utilities
- `blake2` - Blake2b hashing (Substrate standard)
- `borsh` - Canonical serialization
- `serde` - Serialization framework
- `thiserror` - Error handling

### Development
- `proptest` - Property-based testing
- `serde_json` - JSON serialization for tests

## Architecture

This decoder follows the Universal Blockchain Decoder's trait-based architecture:

```
┌─────────────────────────────────────┐
│  BittensorDecoder                  │
│  - Implements ChainDecoder         │
│  - SCALE parsing                   │
│  - Blake2b hashing                 │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  BittensorTransaction              │
│  - Implements Canonicalizer        │
│  - Converts to TxIR                │
│  - Borsh serialization             │
└─────────────────────────────────────┘
```

## Resources

- **Bittensor Documentation**: https://docs.bittensor.com
- **Bittensor GitHub**: https://github.com/opentensor/subtensor
- **TAO Stats Explorer**: https://taostats.io
- **SCALE Codec Spec**: https://docs.substrate.io/reference/scale-codec/
- **Substrate Docs**: https://docs.substrate.io

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test --all`
2. Code is formatted: `cargo fmt --all`
3. No clippy warnings: `cargo clippy --all --all-targets --all-features -- -D warnings`
4. Add tests for new features
5. Update documentation

## License

MIT OR Apache-2.0

## See Also

- `decoder-polkadot` - Similar Substrate-based decoder
- `universal-decoder-core` - Core decoder traits
- `decoder-primitives` - Shared utilities
