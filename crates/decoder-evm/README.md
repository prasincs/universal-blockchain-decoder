# decoder-evm

Generic EVM decoder supporting 2,397 EVM-compatible blockchains through a single unified interface.

## Overview

The `decoder-evm` crate provides a comprehensive solution for decoding transactions from any EVM-compatible blockchain. Instead of creating individual decoders for each chain, this crate uses the ethereum-lists/chains registry to support thousands of chains with minimal code.

### Key Features

- **2,397+ Chains Supported**: Automatically supports all standard EVM-compatible chains
- **Airgapped Operation**: Chain data embedded at compile time via git subtree vendoring
- **Zero Runtime Dependencies**: No network calls, all data compiled into the binary
- **Verifiable Supply Chain**: Git subtree provides complete audit trail
- **Rich Metadata**: Returns chain information alongside decoded transactions
- **Special Chain Detection**: Identifies chains requiring custom decoders (Optimism, Arbitrum, zkSync)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
decoder-evm = "0.1.0"
```

## Usage

### Basic Decoding

```rust
use decoder_evm::EvmDecoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create decoder instance
    let decoder = EvmDecoder::new();

    // Decode transaction bytes
    let tx_bytes = hex::decode("f86c...")?;

    // Option 1: Auto-detect chain from transaction
    let (tx, chain_info) = decoder.decode(&tx_bytes, None)?;
    println!("Decoded {} transaction on {}",
        chain_info.native_currency.symbol,
        chain_info.name
    );

    // Option 2: Specify expected chain ID
    let (tx, chain_info) = decoder.decode(&tx_bytes, Some(1))?; // Ethereum mainnet
    assert_eq!(chain_info.chain_id, 1);

    Ok(())
}
```

### Chain Discovery

```rust
use decoder_evm::EvmDecoder;

fn main() {
    let decoder = EvmDecoder::new();

    // List all supported chains
    println!("Total chains: {}", decoder.count());

    // Get mainnet chains only
    for chain in decoder.list_mainnets().iter().take(10) {
        println!("{}: {} ({})",
            chain.chain_id,
            chain.name,
            chain.native_currency.symbol
        );
    }

    // Search for chains
    let polygon_chains = decoder.search("polygon");
    for chain in polygon_chains {
        println!("Found: {} (ID: {})", chain.name, chain.chain_id);
    }

    // Look up specific chain
    if let Some(bnb) = decoder.get_chain(56) {
        println!("BNB Chain: {}", bnb.name);
        if let Some(explorer) = bnb.primary_explorer() {
            println!("Explorer: {}", explorer.url);
        }
    }
}
```

### Chain Information

```rust
use decoder_evm::EvmDecoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let decoder = EvmDecoder::new();

    let chain = decoder.get_chain(1).unwrap(); // Ethereum

    // Chain metadata
    println!("Name: {}", chain.name);
    println!("Short name: {}", chain.short_name);
    println!("Network ID: {}", chain.network_id);
    println!("Currency: {} ({})",
        chain.native_currency.name,
        chain.native_currency.symbol
    );

    // Testnet detection
    if chain.is_testnet {
        println!("This is a testnet");
    }

    // Special chain detection
    if chain.requires_special_decoder() {
        println!("Note: This chain has custom transaction types");
    }

    // RPC endpoints
    if let Some(rpc) = chain.primary_rpc() {
        println!("Primary RPC: {}", rpc);
    }

    // Block explorer
    if let Some(tx_url) = chain.tx_url("0x123...") {
        println!("View transaction: {}", tx_url);
    }

    Ok(())
}
```

## Architecture

The decoder follows the chain family strategy outlined in `CHAIN_FAMILIES_GROUPING.md`:

### Design Principles

1. **Single Decoder, Multiple Chains**: One decoder supports all standard EVM chains
2. **Compile-Time Embedding**: All chain data embedded at build time (no runtime I/O)
3. **Verifiable Vendoring**: Chain registry vendored via git subtree for audit trail
4. **Pure Rust**: No production dependencies on blockchain-specific libraries
5. **Airgapped Compatible**: Works completely offline for secure environments

### Data Flow

```
Build Time:
  ethereum-lists/chains (git subtree)
    ↓
  build.rs (parse 2397 JSON files)
    ↓
  chain_registry.rs (generated Rust code)
    ↓
  Compiled into binary

Runtime:
  Transaction bytes
    ↓
  EthereumDecoder (pure Rust RLP parsing)
    ↓
  ChainRegistry (lookup chain metadata)
    ↓
  (EthereumTransaction, ChainInfo)
```

## Supported Chains

### Major Networks (Sample)

| Chain ID | Name | Symbol | Type |
|----------|------|--------|------|
| 1 | Ethereum Mainnet | ETH | Mainnet |
| 56 | BNB Smart Chain | BNB | Mainnet |
| 137 | Polygon Mainnet | POL | Mainnet |
| 43114 | Avalanche C-Chain | AVAX | Mainnet |
| 250 | Fantom Opera | FTM | Mainnet |
| 42161 | Arbitrum One | ETH | L2 (Special) |
| 10 | Optimism | ETH | L2 (Special) |
| 8453 | Base | ETH | L2 |
| 324 | zkSync Era | ETH | L2 (Special) |

###Special Chains

Some chains have custom transaction types requiring specialized decoders:

- **Optimism (10)**: Deposit transactions (0x7E) → Use `decoder-op-stack`
- **Arbitrum (42161)**: Retryable tickets → Use `decoder-arbitrum-orbit`
- **zkSync Era (324)**: Custom tx types, account abstraction → Use `decoder-zksync-era`

The generic EVM decoder will warn when encountering these chains and suggest using the appropriate specialized decoder.

## Vendor Chain Registry

The chain registry is vendored using git subtree for maximum security and verifiability:

```bash
# Vendored location
crates/decoder-evm/vendored/chainlist/

# View vendoring commit
git log --oneline crates/decoder-evm/vendored/chainlist | head -1

# Update chain registry (when needed)
git subtree pull \
    --prefix crates/decoder-evm/vendored/chainlist \
    https://github.com/ethereum-lists/chains.git \
    master \
    --squash
```

### Benefits of Vendoring

- ✅ **Offline Operation**: Works in airgapped environments
- ✅ **Verifiable**: Complete git history and audit trail
- ✅ **Reproducible**: Pinned to specific upstream commit
- ✅ **No TOCTOU**: Data cannot change at runtime
- ✅ **Faster**: No network I/O during execution
- ✅ **Secure**: Reviewed before integration

## Performance

- **Build Time**: ~15 seconds (parsing 2,397 chains)
- **Binary Size**: +~2MB (embedded chain data)
- **Runtime Overhead**: < 1μs per decode (HashMap lookup)
- **Memory**: ~5MB (static chain registry)

## Testing

```bash
# Run all tests
cargo test -p decoder-evm

# Run with increased stack size (recommended)
RUST_MIN_STACK=8388608 cargo test -p decoder-evm

# Run specific test
cargo test -p decoder-evm test_list_chains
```

### Test Coverage

- ✅ 21 unit tests covering all functionality
- ✅ Registry initialization and singleton pattern
- ✅ Chain lookup by ID and name
- ✅ Mainnet/testnet filtering
- ✅ Search functionality
- ✅ Special chain detection
- ✅ Sorting and ordering

## Comparison

### Before: Individual Chain Decoders

```rust
// decoder-bnb (100 LOC)
pub struct BnbDecoder;
impl ChainDecoder for BnbDecoder { /* ... */ }

// decoder-polygon (100 LOC)
pub struct PolygonDecoder;
impl ChainDecoder for PolygonDecoder { /* ... */ }

// ... 500+ more crates needed
```

### After: Generic EVM Decoder

```rust
// decoder-evm (1,500 LOC)
pub struct EvmDecoder { /* supports 2,397 chains */ }

// Workspace reduction: 500+ crates → 1 crate (99.8% reduction!)
```

## Limitations

1. **Standard EVM Only**: Only supports chains using standard EVM transaction format
2. **No Custom Types**: Special transaction types require specialized decoders
3. **EIP-155 Required**: Pre-EIP-155 transactions (no chain ID) not supported
4. **Binary Size**: Embedded chain data adds ~2MB to binary

## Roadmap

- [ ] Add caching for frequently accessed chains
- [ ] Support pre-EIP-155 transactions with chain hint
- [ ] Generate TypeScript types for web integration
- [ ] Add chain categorization (L1, L2, sidechain, etc.)
- [ ] Performance optimization for initialization

## Contributing

To add a new EVM chain:

1. Submit chain to [ethereum-lists/chains](https://github.com/ethereum-lists/chains)
2. Wait for merge
3. Update vendored registry: `git subtree pull ...`
4. Rebuild: `cargo build -p decoder-evm`

No code changes needed!

## License

MIT OR Apache-2.0

## See Also

- [CHAIN_FAMILIES_GROUPING.md](../../CHAIN_FAMILIES_GROUPING.md) - Chain family strategy
- [NEXT_STEPS_CHAINLIST_INTEGRATION.md](../../NEXT_STEPS_CHAINLIST_INTEGRATION.md) - Implementation plan
- [ethereum-lists/chains](https://github.com/ethereum-lists/chains) - Upstream chain registry
- [chainlist.org](https://chainlist.org/) - User-friendly chain browser
