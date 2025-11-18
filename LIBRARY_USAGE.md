# Library Usage Guide

How to use Universal Blockchain Decoder in your application.

---

## Installation

```toml
[dependencies]
universal-decoder-core = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
decoder-bitcoin = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
decoder-ethereum = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
```

---

## Basic Usage

### Decode a Bitcoin Transaction

```rust
use universal_decoder_core::prelude::*;
use decoder_bitcoin::BitcoinDecoder;

fn main() -> Result<()> {
    let tx_bytes = hex::decode("01000000...")?;

    // Decode to Bitcoin-specific type
    let btc_tx = BitcoinDecoder::decode(&tx_bytes)?;

    // Convert to universal TxIR
    let tx_ir = btc_tx.canonicalize()?;

    // Access normalized data
    println!("Chain: {}", tx_ir.chain.name);
    println!("Operations: {}", tx_ir.operations.len());

    Ok(())
}
```

### Decode Any Chain

```rust
fn decode_any_chain(chain_id: u64, raw_bytes: &[u8]) -> Result<TxIR<'_, 1>> {
    match chain_id {
        1 => decoder_bitcoin::BitcoinDecoder::decode(raw_bytes)?.canonicalize(),
        2 => decoder_ethereum::EthereumDecoder::decode(raw_bytes)?.canonicalize(),
        _ => Err(DecoderError::unsupported_chain("Unknown chain")),
    }
}
```

---

## Common Patterns

### Pattern 1: Simple Indexer

```rust
use sqlx::PgPool;

async fn index_transaction(
    db: &PgPool,
    tx_bytes: &[u8],
    block_height: u64,
) -> Result<()> {
    // Decode
    let tx = BitcoinDecoder::decode(tx_bytes)?;
    let tx_ir = tx.canonicalize()?;

    // Store
    let tx_hash = hex::encode(&tx_ir.metadata.tx_hash);
    sqlx::query!(
        "INSERT INTO transactions (tx_hash, block_height, data) VALUES ($1, $2, $3)",
        tx_hash,
        block_height as i64,
        serde_json::to_value(&tx_ir)?,
    )
    .execute(db)
    .await?;

    Ok(())
}
```

### Pattern 2: Extract Transfers

```rust
fn extract_transfers(tx_ir: &TxIR) -> Vec<(String, String, u128)> {
    tx_ir.operations
        .iter()
        .filter_map(|op| {
            if let Operation::Transfer(t) = op {
                Some((
                    hex::encode(&t.from.bytes),
                    hex::encode(&t.to.bytes),
                    t.amount.value,
                ))
            } else {
                None
            }
        })
        .collect()
}
```

### Pattern 3: Batch Processing

```rust
async fn process_block(block: Block, db: &PgPool) -> Result<()> {
    for tx_bytes in block.transactions {
        let tx_ir = decode_any_chain(block.chain_id, &tx_bytes)?;

        // Process each transaction
        for op in tx_ir.operations {
            match op {
                Operation::Transfer(t) => handle_transfer(db, &t).await?,
                Operation::ContractCall(c) => handle_contract_call(db, &c).await?,
                _ => {}
            }
        }
    }
    Ok(())
}
```

### Pattern 4: Using Hooks

```rust
use universal_decoder_core::prelude::*;

// Create hook registry
let registry = HookRegistryBuilder::new()
    .with_size_limit(1_000_000)  // 1MB max
    .with_logging("my-indexer".to_string(), vec![HookStage::PreDecode])
    .build();

// Decode with hooks
let tx = decoder_bitcoin::decode_with_hooks(&tx_bytes, &registry)?;
```

---

## Database Schema

Simple PostgreSQL schema for indexing:

```sql
-- Transactions
CREATE TABLE transactions (
    tx_hash VARCHAR(66) PRIMARY KEY,
    chain_id BIGINT NOT NULL,
    block_height BIGINT NOT NULL,
    timestamp BIGINT,
    data JSONB NOT NULL
);

CREATE INDEX idx_tx_block ON transactions(chain_id, block_height DESC);

-- Operations (for searching)
CREATE TABLE operations (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL REFERENCES transactions(tx_hash),
    op_type VARCHAR(50) NOT NULL,
    from_address VARCHAR(66),
    to_address VARCHAR(66),
    amount NUMERIC(78, 0)
);

CREATE INDEX idx_op_from ON operations(from_address);
CREATE INDEX idx_op_to ON operations(to_address);
```

---

## Working with TxIR

### Access Transaction Data

```rust
let tx_ir: TxIR = /* ... */;

// Metadata
println!("Hash: {}", hex::encode(&tx_ir.metadata.tx_hash));
println!("Block: {:?}", tx_ir.metadata.block_height);
println!("Size: {}", tx_ir.metadata.size);

// Chain info
println!("Chain: {} (ID: {})", tx_ir.chain.name, tx_ir.chain.id);

// Operations
for op in &tx_ir.operations {
    match op {
        Operation::Transfer(t) => {
            println!("Transfer: {} -> {}, amount: {}",
                hex::encode(&t.from.bytes[..8]),
                hex::encode(&t.to.bytes[..8]),
                t.amount.value
            );
        }
        Operation::ContractCall(c) => {
            println!("Contract call: {}", hex::encode(&c.contract_address.bytes[..8]));
        }
        _ => {}
    }
}

// State changes (UTXO chains)
println!("Inputs: {}", tx_ir.state_deltas.inputs.len());
println!("Outputs: {}", tx_ir.state_deltas.outputs.len());
```

### Canonical Serialization

```rust
// Serialize to deterministic bytes (for hashing)
let canonical_bytes = borsh::to_vec(&tx_ir)?;
let hash = sha2::Sha256::digest(&canonical_bytes);

// Note: Don't use JSON for hashing (not deterministic)
// JSON is only for display/storage
```

---

## Async Processing

```rust
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Decoding is sync, wrap if needed
    let tx_bytes = fetch_from_rpc().await?;

    let tx_ir = tokio::task::spawn_blocking(move || {
        BitcoinDecoder::decode(&tx_bytes)?.canonicalize()
    })
    .await??;

    // Store async
    store_in_db(&tx_ir).await?;

    Ok(())
}
```

---

## Error Handling

```rust
match BitcoinDecoder::decode(&tx_bytes) {
    Ok(tx) => {
        // Process transaction
        let tx_ir = tx.canonicalize()?;
    }
    Err(DecoderError::InvalidStructure(msg)) => {
        eprintln!("Invalid transaction format: {}", msg);
    }
    Err(e) => {
        eprintln!("Decode error: {:?}", e);
    }
}
```

---

## Multi-Chain Support

### Supported Chains

- **UTXO**: Bitcoin, Litecoin, Dogecoin
- **Account**: Ethereum (legacy + EIP-1559), 500+ EVM chains
- **Cosmos SDK**: 100+ Cosmos chains
- **OP Stack**: Optimism, Base, etc.
- **Others**: TON, Starknet (230+ chains)

See ROADMAP.md for upcoming chains.

### Example: Multi-Chain Explorer

```rust
async fn handle_transaction(
    chain_family: ChainFamily,
    tx_bytes: &[u8],
    db: &PgPool,
) -> Result<String> {
    let tx_ir = match chain_family {
        ChainFamily::Utxo => {
            decoder_bitcoin::BitcoinDecoder::decode(tx_bytes)?.canonicalize()?
        }
        ChainFamily::Account => {
            decoder_ethereum::EthereumDecoder::decode(tx_bytes)?.canonicalize()?
        }
        _ => return Err(DecoderError::unsupported_chain("Not supported")),
    };

    let tx_hash = hex::encode(&tx_ir.metadata.tx_hash);

    // Store in database
    store_transaction(db, &tx_ir).await?;

    Ok(tx_hash)
}
```

---

## Performance Tips

1. **Use connection pooling**:
```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;
```

2. **Batch inserts**:
```rust
// Use sqlx::QueryBuilder for bulk inserts
let mut query_builder = sqlx::QueryBuilder::new(
    "INSERT INTO operations (tx_hash, op_type) "
);
query_builder.push_values(operations, |mut b, op| {
    b.push_bind(op.tx_hash).push_bind(op.op_type);
});
```

3. **Process in parallel**:
```rust
use futures::stream::{self, StreamExt};

stream::iter(blocks)
    .map(|block| process_block(block))
    .buffer_unordered(10)  // Process 10 blocks concurrently
    .collect::<Vec<_>>()
    .await;
```

---

## Custom Hooks

```rust
use universal_decoder_core::prelude::*;

struct MySizeChecker;

impl Hook for MySizeChecker {
    fn name(&self) -> &str {
        "size_checker"
    }

    fn stages(&self) -> Vec<HookStage> {
        vec![HookStage::PreDecode]
    }

    fn execute(&self, context: &HookContext) -> Result<HookResult> {
        if context.raw_bytes.len() > 500_000 {
            return Ok(HookResult::Abort("Too large".to_string()));
        }
        Ok(HookResult::Continue)
    }
}

// Use it
let mut registry = HookRegistry::new();
registry.register(MySizeChecker);
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bitcoin() {
        let tx_hex = "01000000...";
        let tx_bytes = hex::decode(tx_hex).unwrap();

        let tx = BitcoinDecoder::decode(&tx_bytes).unwrap();
        assert_eq!(tx.version, 1);

        let tx_ir = tx.canonicalize().unwrap();
        assert!(!tx_ir.operations.is_empty());
    }
}
```

---

## Examples

See:
- `examples/simple-decoder/` - Basic usage
- Live demo: https://trustless-txir.netlify.app

---

## Resources

- **API Docs**: See code comments in `universal-decoder-core/src/`
- **More chains**: See `ROADMAP.md` for upcoming support
- **Architecture**: See `CLAUDE.md` for design philosophy
- **Comparison**: See `INDEXER_COMPARISON.md` vs other tools

---

**Version**: 1.0.0
**Last Updated**: 2025-11-18
