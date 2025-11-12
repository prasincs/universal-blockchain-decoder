# Using Universal Blockchain Decoder as a Library

This guide shows how to use the universal blockchain decoder in your Rust projects.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# Core library - always required
universal-decoder-core = { git = "https://github.com/prasincs/universal-blockchain-decoder", branch = "main" }

# Add specific decoders you need
decoder-bitcoin = { git = "https://github.com/prasincs/universal-blockchain-decoder", branch = "main" }
decoder-ethereum = { git = "https://github.com/prasincs/universal-blockchain-decoder", branch = "main" }
```

Or use a specific version once published:

```toml
[dependencies]
universal-decoder-core = "0.1"
decoder-bitcoin = "0.1"
decoder-ethereum = "0.1"
```

## Quick Start

### Decoding a Bitcoin Transaction

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

fn main() -> Result<()> {
    // Raw transaction bytes (hex decoded)
    let tx_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex)?;

    // Decode the transaction
    let tx: BitcoinTransaction = BitcoinDecoder::decode(&tx_bytes)?;

    // Access transaction data
    println!("TXID: {}", universal_decoder_core::hex::encode(&tx.txid()));
    println!("Version: {}", tx.version);
    println!("Inputs: {}", tx.inputs.len());
    println!("Outputs: {}", tx.outputs.len());
    println!("Is coinbase: {}", tx.is_coinbase());
    println!("Is SegWit: {}", tx.is_segwit());

    // Inspect outputs
    for (i, output) in tx.outputs.iter().enumerate() {
        let btc = output.value as f64 / 100_000_000.0;
        println!("Output {}: {:.8} BTC", i, btc);
    }

    Ok(())
}
```

### Converting to Universal IR (Canonical Representation)

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

fn decode_and_canonicalize(tx_bytes: &[u8]) -> Result<()> {
    // Decode chain-specific transaction
    let tx = BitcoinDecoder::decode(tx_bytes)?;

    // Convert to universal intermediate representation
    let tx_ir = tx.canonicalize()?;

    // Access canonical data (works the same for all chains)
    println!("Chain: {}", tx_ir.chain.chain_name);
    println!("Version: {}", tx_ir.version());
    println!("Operations: {}", tx_ir.operations.len());

    // Get canonical hash (deterministic across all implementations)
    let canonical_hash = tx_ir.canonical_hash()?;
    println!("Canonical Hash: {}", universal_decoder_core::hex::encode(&canonical_hash));

    // Serialize to canonical bytes (for hashing/storage)
    let canonical_bytes = tx_ir.to_canonical_bytes()?;
    println!("Canonical Size: {} bytes", canonical_bytes.len());

    Ok(())
}
```

### Working with Multiple Chains

```rust
use universal_decoder_core::prelude::*;
use decoder_bitcoin::BitcoinDecoder;
use decoder_ethereum::EthereumDecoder;

enum Chain {
    Bitcoin,
    Ethereum,
}

fn decode_transaction(chain: Chain, tx_bytes: &[u8]) -> Result<TxIR<'_, 1>> {
    match chain {
        Chain::Bitcoin => {
            let tx = BitcoinDecoder::decode(tx_bytes)?;
            tx.canonicalize()
        }
        Chain::Ethereum => {
            let tx = EthereumDecoder::decode(tx_bytes)?;
            tx.canonicalize()
        }
    }
}

fn main() -> Result<()> {
    // Bitcoin transaction
    let btc_bytes = universal_decoder_core::hex::decode("0100000001...")?;
    let btc_ir = decode_transaction(Chain::Bitcoin, &btc_bytes)?;
    println!("Bitcoin TX: {}", btc_ir.chain.chain_name);

    // Ethereum transaction
    let eth_bytes = universal_decoder_core::hex::decode("f86c...")?;
    let eth_ir = decode_transaction(Chain::Ethereum, &eth_bytes)?;
    println!("Ethereum TX: {}", eth_ir.chain.chain_name);

    Ok(())
}
```

## Common Use Cases

### 1. Transaction Indexer

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

struct TransactionIndex {
    txid: Vec<u8>,
    block_height: u64,
    inputs: usize,
    outputs: usize,
    total_value: u64,
}

fn index_transaction(tx_bytes: &[u8], block_height: u64) -> Result<TransactionIndex> {
    let tx = BitcoinDecoder::decode(tx_bytes)?;

    Ok(TransactionIndex {
        txid: tx.txid(),
        block_height,
        inputs: tx.inputs.len(),
        outputs: tx.outputs.len(),
        total_value: tx.total_output_value()?,
    })
}
```

### 2. Fee Calculator

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

fn calculate_fee(
    tx_bytes: &[u8],
    input_values: &[u64]  // Values of previous outputs being spent
) -> Result<u64> {
    let tx = BitcoinDecoder::decode(tx_bytes)?;

    // Sum input values (from previous outputs)
    let total_input: u64 = input_values.iter().sum();

    // Sum output values
    let total_output = tx.total_output_value()?;

    // Fee = inputs - outputs
    Ok(total_input.saturating_sub(total_output))
}
```

### 3. Address Extraction

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

fn extract_addresses(tx_bytes: &[u8]) -> Result<Vec<String>> {
    let tx = BitcoinDecoder::decode(tx_bytes)?;
    let mut addresses = Vec::new();

    // Extract from outputs (simplified - real implementation needs script parsing)
    for output in &tx.outputs {
        // Parse scriptPubKey to extract address
        // This is simplified - you'd need proper script parsing
        let script_type = guess_script_type(&output.script_pubkey);
        addresses.push(format!("{} script", script_type));
    }

    Ok(addresses)
}

fn guess_script_type(script: &[u8]) -> &'static str {
    match script.len() {
        25 if script.get(0..3) == Some(&[0x76, 0xa9, 0x14]) => "P2PKH",
        23 if script.get(0..2) == Some(&[0xa9, 0x14]) => "P2SH",
        22 if script.get(0..2) == Some(&[0x00, 0x14]) => "P2WPKH",
        34 if script.get(0..2) == Some(&[0x00, 0x20]) => "P2WSH",
        34 if script.get(0..2) == Some(&[0x51, 0x20]) => "P2TR",
        _ => "Unknown",
    }
}
```

### 4. Transaction Validator

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

fn validate_transaction(tx_bytes: &[u8]) -> Result<bool> {
    // Decode
    let tx = BitcoinDecoder::decode(tx_bytes)?;

    // Basic validation
    if tx.inputs.is_empty() {
        return Ok(false); // No inputs
    }

    if tx.outputs.is_empty() {
        return Ok(false); // No outputs
    }

    // Check for overflow
    let _total = tx.total_output_value()?;

    // More validation...
    // - Check coinbase rules
    // - Verify signatures (requires additional implementation)
    // - Check script validity

    Ok(true)
}
```

## Advanced Usage

### Generic Decoder Pattern

```rust
use universal_decoder_core::prelude::*;

fn process_transaction<D: ChainDecoder>(tx_bytes: &[u8]) -> Result<()>
where
    D::TxSpecific: Canonicalizer,
{
    // Decode using any decoder
    let tx = D::decode(tx_bytes)?;

    // Convert to canonical IR
    let tx_ir = tx.canonicalize()?;

    // Process using universal format
    println!("Chain: {}", tx_ir.chain.chain_name);
    println!("Operations: {}", tx_ir.operations.len());

    Ok(())
}

// Use with any chain
fn main() -> Result<()> {
    let btc_bytes = vec![/* ... */];
    process_transaction::<BitcoinDecoder>(&btc_bytes)?;

    let eth_bytes = vec![/* ... */];
    process_transaction::<EthereumDecoder>(&eth_bytes)?;

    Ok(())
}
```

### Custom Error Handling

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

fn safe_decode(tx_bytes: &[u8]) -> std::result::Result<BitcoinTransaction, String> {
    BitcoinDecoder::decode(tx_bytes)
        .map_err(|e| format!("Decode failed: {:?}", e))
}

fn main() {
    let tx_hex = "0100000001...";
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex).unwrap();

    match safe_decode(&tx_bytes) {
        Ok(tx) => println!("Successfully decoded: {:?}", tx.txid()),
        Err(e) => eprintln!("Failed: {}", e),
    }
}
```

## Performance Considerations

### Zero-Copy Where Possible

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

// Avoid unnecessary cloning
fn process_efficiently(tx_bytes: &[u8]) -> Result<()> {
    let tx = BitcoinDecoder::decode(tx_bytes)?;

    // Reference the data instead of cloning
    for input in &tx.inputs {
        // Process without copying
        let _ = &input.prev_hash;
    }

    Ok(())
}
```

### Batch Processing

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;

fn decode_block(transactions: &[Vec<u8>]) -> Result<Vec<BitcoinTransaction>> {
    transactions
        .iter()
        .map(|tx_bytes| BitcoinDecoder::decode(tx_bytes))
        .collect()
}
```

## Integration Examples

### With Tokio (Async)

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let tx_bytes = fetch_transaction_async().await?;

    // Decoding is sync, but can be wrapped
    let tx = tokio::task::spawn_blocking(move || {
        BitcoinDecoder::decode(&tx_bytes)
    }).await??;

    println!("Decoded: {:?}", tx.txid());
    Ok(())
}

async fn fetch_transaction_async() -> Result<Vec<u8>> {
    // Your async code here
    Ok(vec![])
}
```

### With Serde (JSON Export)

Note: `serde_json` is in dev-dependencies only. For production JSON export, add it to your dependencies:

```rust
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use universal_decoder_core::prelude::*;
use serde_json;

fn export_to_json(tx_bytes: &[u8]) -> Result<String> {
    let tx = BitcoinDecoder::decode(tx_bytes)?;
    let tx_ir = tx.canonicalize()?;

    // TxIR implements Serialize
    let json = serde_json::to_string_pretty(&tx_ir)?;
    Ok(json)
}
```

## Testing Your Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
    use universal_decoder_core::prelude::*;

    #[test]
    fn test_decode_genesis_coinbase() {
        let tx_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex).unwrap();

        let tx = BitcoinDecoder::decode(&tx_bytes).unwrap();

        assert_eq!(tx.version, 1);
        assert!(tx.is_coinbase());
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].value, 5_000_000_000);
    }
}
```

## Feature Flags

None currently, but future versions may support:

```toml
[dependencies]
decoder-bitcoin = { version = "0.1", features = ["async", "serde_json"] }
```

## Minimum Supported Rust Version (MSRV)

Rust 1.70 or later

## Dependencies

The library has minimal dependencies:
- **Core**: `serde`, `borsh`, `thiserror`, `sha2`, `sha3`
- **Decoders**: Only depend on `universal-decoder-core` (no external blockchain libraries in production)

## Support

- **Documentation**: https://docs.rs/universal-decoder-core
- **Examples**: See `examples/` directory
- **Issues**: https://github.com/prasincs/universal-blockchain-decoder/issues
- **Discussions**: https://github.com/prasincs/universal-blockchain-decoder/discussions

## License

MIT OR Apache-2.0
