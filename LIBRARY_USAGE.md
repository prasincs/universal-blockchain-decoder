# Universal Blockchain Decoder - Library Usage Guide

## For Indexers, Explorers, and Forensic Analysis

This comprehensive guide provides practical patterns and examples for using the Universal Blockchain Decoder in production applications: block explorers, transaction indexers, forensic tools, and analytics platforms.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Architecture Overview](#architecture-overview)
3. [Use Case: Block Explorer](#use-case-block-explorer)
4. [Use Case: Transaction Indexer](#use-case-transaction-indexer)
5. [Use Case: Forensic Analysis](#use-case-forensic-analysis)
6. [Use Case: Multi-Chain Analytics](#use-case-multi-chain-analytics)
7. [Database Schemas](#database-schemas)
8. [Batch Processing Patterns](#batch-processing-patterns)
9. [Performance Optimization](#performance-optimization)
10. [Error Handling Strategies](#error-handling-strategies)
11. [Production Deployment](#production-deployment)
12. [Basic Usage Examples](#basic-usage-examples)

---

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
universal-decoder-core = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
decoder-bitcoin = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
decoder-ethereum = { git = "https://github.com/prasincs/universal-blockchain-decoder" }
decoder-evm = { git = "https://github.com/prasincs/universal-blockchain-decoder" }

# Common production dependencies
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
serde_json = "1.0"
borsh = "1.3"
```

### Basic Decoding Example

```rust
use universal_decoder_core::prelude::*;
use decoder_bitcoin::BitcoinDecoder;

fn decode_transaction(raw_tx: &[u8]) -> Result<TxIR<'_, 1>> {
    // 1. Decode chain-specific format
    let btc_tx = BitcoinDecoder::decode(raw_tx)?;

    // 2. Convert to universal TxIR
    let tx_ir = btc_tx.canonicalize()?;

    Ok(tx_ir)
}
```

---

## Architecture Overview

### Three-Layer Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│  1. Raw Transaction Bytes (from blockchain node/RPC)        │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  2. Chain-Specific Decoder (Bitcoin, Ethereum, Solana...)   │
│     • Validates format                                       │
│     • Parses chain-specific structures                       │
│     • Extracts signatures, scripts, data                     │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  3. Universal TxIR (Intermediate Representation)             │
│     • Normalized operations (Transfer, ContractCall, etc.)   │
│     • State deltas (inputs/outputs/account changes)          │
│     • Authorization (signatures, public keys)                │
│     • Chain-agnostic queries and analysis                    │
└──────────────────────────────────────────────────────────────┘
```

### Key Components

- **ChainDecoder**: Parses chain-specific transaction format
- **Canonicalizer**: Transforms to universal TxIR
- **TxIR**: Chain-agnostic intermediate representation
- **Hooks**: Custom processing pipeline (validation, logging, metrics)

---

## Use Case: Block Explorer

Block explorers need to decode, display, and search transactions from multiple blockchains.

### Multi-Chain Explorer Backend

```rust
use universal_decoder_core::prelude::*;
use sqlx::PgPool;

pub struct ExplorerService {
    db: PgPool,
    hook_registry: HookRegistry,
}

impl ExplorerService {
    pub fn new(db: PgPool) -> Self {
        let hook_registry = HookRegistryBuilder::new()
            .with_size_limit(10_000_000) // 10MB max
            .with_logging("explorer".to_string(), vec![
                HookStage::PreDecode,
                HookStage::PostDecode,
            ])
            .build();

        Self { db, hook_registry }
    }

    /// Index a transaction from any blockchain
    pub async fn index_transaction(
        &self,
        chain_family: ChainFamily,
        raw_tx: &[u8],
        block_height: u64,
        block_timestamp: u64,
    ) -> Result<String> {
        // Decode based on chain family
        let tx_ir = match chain_family {
            ChainFamily::Utxo => {
                let tx = decoder_bitcoin::decode_with_hooks(raw_tx, &self.hook_registry)?;
                tx.canonicalize()?
            }
            ChainFamily::Account => {
                let tx = decoder_ethereum::decode_with_hooks(raw_tx, &self.hook_registry)?;
                tx.canonicalize()?
            }
            _ => return Err(DecoderError::unsupported_chain("Chain not supported")),
        };

        // Store in database
        let tx_hash = hex::encode(&tx_ir.metadata.tx_hash);
        self.store_transaction(&tx_ir, block_height, block_timestamp).await?;

        Ok(tx_hash)
    }

    /// Store transaction in normalized schema
    async fn store_transaction(
        &self,
        tx_ir: &TxIR<'_, 1>,
        block_height: u64,
        block_timestamp: u64,
    ) -> Result<()> {
        let tx_hash = hex::encode(&tx_ir.metadata.tx_hash);
        let chain_id = tx_ir.chain.id;

        // Serialize to JSON for display
        let tx_json = serde_json::to_value(tx_ir)
            .map_err(|e| DecoderError::serialization_failed(e.to_string()))?;

        // Serialize to canonical Borsh bytes for hashing
        let canonical_bytes = borsh::to_vec(tx_ir)
            .map_err(|e| DecoderError::serialization_failed(e.to_string()))?;

        // Insert transaction
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                tx_hash, chain_id, block_height, timestamp,
                size, num_operations, tx_data, canonical_bytes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (tx_hash, chain_id) DO UPDATE
            SET block_height = EXCLUDED.block_height
            "#,
            tx_hash,
            chain_id as i64,
            block_height as i64,
            block_timestamp as i64,
            tx_ir.metadata.size as i32,
            tx_ir.operations.len() as i32,
            tx_json,
            canonical_bytes,
        )
        .execute(&self.db)
        .await
        .map_err(|e| DecoderError::custom(format!("Database error: {}", e)))?;

        // Index operations for searching
        for (idx, op) in tx_ir.operations.iter().enumerate() {
            self.index_operation(&tx_hash, chain_id, idx, op).await?;
        }

        // Index state changes
        self.index_state_deltas(&tx_hash, chain_id, &tx_ir.state_deltas).await?;

        Ok(())
    }

    /// Index individual operations
    async fn index_operation(
        &self,
        tx_hash: &str,
        chain_id: u64,
        index: usize,
        operation: &Operation,
    ) -> Result<()> {
        match operation {
            Operation::Transfer(transfer) => {
                let from_addr = hex::encode(&transfer.from.bytes);
                let to_addr = hex::encode(&transfer.to.bytes);
                let amount = transfer.amount.value.to_string();

                sqlx::query!(
                    r#"
                    INSERT INTO operations (
                        tx_hash, chain_id, op_index, op_type,
                        from_address, to_address, amount
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                    tx_hash,
                    chain_id as i64,
                    index as i32,
                    "transfer",
                    from_addr,
                    to_addr,
                    amount,
                )
                .execute(&self.db)
                .await
                .map_err(|e| DecoderError::custom(format!("DB error: {}", e)))?;
            }

            Operation::ContractCall(call) => {
                let contract_addr = hex::encode(&call.contract_address.bytes);
                let function_sig = call.function_signature.clone().unwrap_or_default();

                sqlx::query!(
                    r#"
                    INSERT INTO operations (
                        tx_hash, chain_id, op_index, op_type,
                        to_address, function_signature
                    ) VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    tx_hash,
                    chain_id as i64,
                    index as i32,
                    "contract_call",
                    contract_addr,
                    function_sig,
                )
                .execute(&self.db)
                .await
                .map_err(|e| DecoderError::custom(format!("DB error: {}", e)))?;
            }

            _ => {} // Handle other operation types as needed
        }

        Ok(())
    }

    /// Index state deltas
    async fn index_state_deltas(
        &self,
        tx_hash: &str,
        chain_id: u64,
        deltas: &StateDeltas,
    ) -> Result<()> {
        // Index inputs (UTXOs consumed)
        for (idx, input) in deltas.inputs.iter().enumerate() {
            let prev_txid = hex::encode(&input.previous_output.tx_hash);

            sqlx::query!(
                r#"
                INSERT INTO transaction_inputs (
                    tx_hash, chain_id, input_index,
                    prev_tx_hash, prev_output_index
                ) VALUES ($1, $2, $3, $4, $5)
                "#,
                tx_hash,
                chain_id as i64,
                idx as i32,
                prev_txid,
                input.previous_output.output_index as i32,
            )
            .execute(&self.db)
            .await
            .map_err(|e| DecoderError::custom(format!("DB error: {}", e)))?;
        }

        // Index outputs (UTXOs created)
        for (idx, output) in deltas.outputs.iter().enumerate() {
            let address = output.address.as_ref()
                .map(|addr| hex::encode(&addr.bytes));
            let amount = output.value.value.to_string();

            sqlx::query!(
                r#"
                INSERT INTO transaction_outputs (
                    tx_hash, chain_id, output_index,
                    address, amount, spent
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                tx_hash,
                chain_id as i64,
                idx as i32,
                address,
                amount,
                false,
            )
            .execute(&self.db)
            .await
            .map_err(|e| DecoderError::custom(format!("DB error: {}", e)))?;
        }

        Ok(())
    }

    /// Query transactions by address
    pub async fn get_transactions_by_address(
        &self,
        address: &str,
        chain_id: u64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TransactionSummary>> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT t.tx_hash, t.block_height, t.timestamp,
                   t.num_operations, t.size
            FROM transactions t
            JOIN operations o ON t.tx_hash = o.tx_hash AND t.chain_id = o.chain_id
            WHERE (o.from_address = $1 OR o.to_address = $1)
              AND t.chain_id = $2
            ORDER BY t.block_height DESC
            LIMIT $3 OFFSET $4
            "#,
            address,
            chain_id as i64,
            limit,
            offset,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| DecoderError::custom(format!("DB error: {}", e)))?;

        Ok(rows.into_iter().map(|row| TransactionSummary {
            tx_hash: row.tx_hash,
            block_height: row.block_height as u64,
            timestamp: row.timestamp as u64,
            num_operations: row.num_operations as usize,
            size: row.size as usize,
        }).collect())
    }
}

#[derive(Debug)]
pub struct TransactionSummary {
    pub tx_hash: String,
    pub block_height: u64,
    pub timestamp: u64,
    pub num_operations: usize,
    pub size: usize,
}
```

---

## Use Case: Transaction Indexer

Indexers need to process large volumes of historical transactions efficiently.

### Batch Block Processor

```rust
use universal_decoder_core::prelude::*;
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct BatchIndexer {
    explorer: Arc<ExplorerService>,
    concurrency: usize,
}

impl BatchIndexer {
    pub fn new(explorer: Arc<ExplorerService>, concurrency: usize) -> Self {
        Self { explorer, concurrency }
    }

    /// Index a range of blocks in parallel
    pub async fn index_block_range(
        &self,
        chain_family: ChainFamily,
        start_block: u64,
        end_block: u64,
        rpc_client: Arc<dyn BlockchainRpcClient>,
    ) -> Result<IndexingStats> {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mut handles = vec![];
        let mut stats = IndexingStats::default();

        for block_height in start_block..=end_block {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let explorer = self.explorer.clone();
            let rpc = rpc_client.clone();

            let handle = tokio::spawn(async move {
                let result = Self::index_block(explorer, chain_family, block_height, rpc).await;
                drop(permit);
                result
            });

            handles.push(handle);
        }

        // Wait for all blocks
        for handle in handles {
            match handle.await {
                Ok(Ok(block_stats)) => {
                    stats.blocks_processed += 1;
                    stats.transactions_indexed += block_stats.tx_count;
                }
                Ok(Err(e)) => {
                    eprintln!("Block error: {}", e);
                    stats.errors += 1;
                }
                Err(e) => {
                    eprintln!("Join error: {}", e);
                    stats.errors += 1;
                }
            }
        }

        Ok(stats)
    }

    async fn index_block(
        explorer: Arc<ExplorerService>,
        chain_family: ChainFamily,
        block_height: u64,
        rpc_client: Arc<dyn BlockchainRpcClient>,
    ) -> Result<BlockStats> {
        let block = rpc_client.get_block(block_height).await
            .map_err(|e| DecoderError::custom(format!("RPC error: {}", e)))?;

        let mut tx_count = 0;

        for tx_bytes in block.transactions {
            match explorer.index_transaction(
                chain_family,
                &tx_bytes,
                block_height,
                block.timestamp,
            ).await {
                Ok(_) => tx_count += 1,
                Err(e) => eprintln!("TX error: {}", e),
            }
        }

        Ok(BlockStats { tx_count })
    }
}

#[derive(Debug, Default)]
pub struct IndexingStats {
    pub blocks_processed: u64,
    pub transactions_indexed: u64,
    pub errors: u64,
}

#[derive(Debug)]
struct BlockStats {
    tx_count: u64,
}

#[async_trait::async_trait]
pub trait BlockchainRpcClient: Send + Sync {
    async fn get_block(&self, height: u64) -> std::result::Result<Block, String>;
}

#[derive(Debug)]
pub struct Block {
    pub height: u64,
    pub timestamp: u64,
    pub transactions: Vec<Vec<u8>>,
}
```

---

## Use Case: Forensic Analysis

Forensic tools trace funds, identify patterns, and analyze transaction graphs.

### Transaction Graph Analyzer

```rust
use universal_decoder_core::prelude::*;
use std::collections::HashMap;

pub struct TransactionGraph {
    nodes: HashMap<String, Node>,
    edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub address: String,
    pub total_received: u128,
    pub total_sent: u128,
    pub transaction_count: usize,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub tx_hash: String,
    pub timestamp: u64,
}

impl TransactionGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add a transaction to the graph
    pub fn add_transaction(&mut self, tx_ir: &TxIR<'_, 1>) -> Result<()> {
        let tx_hash = hex::encode(&tx_ir.metadata.tx_hash);
        let timestamp = tx_ir.metadata.timestamp.unwrap_or(0);

        for operation in &tx_ir.operations {
            if let Operation::Transfer(transfer) = operation {
                let from = hex::encode(&transfer.from.bytes);
                let to = hex::encode(&transfer.to.bytes);
                let amount = transfer.amount.value;

                // Update nodes
                self.nodes.entry(from.clone())
                    .and_modify(|n| {
                        n.total_sent += amount;
                        n.transaction_count += 1;
                    })
                    .or_insert(Node {
                        address: from.clone(),
                        total_received: 0,
                        total_sent: amount,
                        transaction_count: 1,
                    });

                self.nodes.entry(to.clone())
                    .and_modify(|n| {
                        n.total_received += amount;
                        n.transaction_count += 1;
                    })
                    .or_insert(Node {
                        address: to.clone(),
                        total_received: amount,
                        total_sent: 0,
                        transaction_count: 1,
                    });

                // Add edge
                self.edges.push(Edge {
                    from,
                    to,
                    amount,
                    tx_hash: tx_hash.clone(),
                    timestamp,
                });
            }
        }

        Ok(())
    }

    /// Trace funds forward from an address (BFS)
    pub fn trace_forward(
        &self,
        start_address: &str,
        max_hops: usize,
    ) -> HashMap<String, (u128, usize)> {
        let mut visited: HashMap<String, (u128, usize)> = HashMap::new();
        let mut queue = vec![(start_address.to_string(), 0u128, 0usize)];
        visited.insert(start_address.to_string(), (0, 0));

        while let Some((current, cumulative_amount, hops)) = queue.pop() {
            if hops >= max_hops {
                continue;
            }

            for edge in &self.edges {
                if edge.from == current {
                    let new_amount = cumulative_amount + edge.amount;
                    let new_hops = hops + 1;

                    visited.entry(edge.to.clone())
                        .or_insert_with(|| {
                            queue.push((edge.to.clone(), new_amount, new_hops));
                            (new_amount, new_hops)
                        });
                }
            }
        }

        visited
    }

    /// Detect potential mixers
    pub fn detect_mixers(&self, min_connections: usize) -> Vec<String> {
        let mut connection_counts: HashMap<String, (usize, usize)> = HashMap::new();

        for edge in &self.edges {
            connection_counts.entry(edge.from.clone())
                .or_insert((0, 0))
                .1 += 1;
            connection_counts.entry(edge.to.clone())
                .or_insert((0, 0))
                .0 += 1;
        }

        connection_counts.into_iter()
            .filter(|(_, (incoming, outgoing))| {
                incoming + outgoing >= min_connections
            })
            .map(|(addr, _)| addr)
            .collect()
    }
}
```

---

## Use Case: Multi-Chain Analytics

### Cross-Chain Volume Analyzer

```rust
use universal_decoder_core::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct MultiChainAnalyzer {
    analytics: HashMap<u64, ChainAnalytics>,
}

#[derive(Debug, Clone)]
pub struct ChainAnalytics {
    pub chain_id: u64,
    pub chain_name: String,
    pub total_transactions: u64,
    pub total_volume: u128,
    pub unique_addresses: HashSet<String>,
    pub operation_counts: HashMap<String, u64>,
}

impl MultiChainAnalyzer {
    pub fn new() -> Self {
        Self {
            analytics: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, tx_ir: &TxIR<'_, 1>) -> Result<()> {
        let chain_id = tx_ir.chain.id;

        let analytics = self.analytics.entry(chain_id)
            .or_insert_with(|| ChainAnalytics {
                chain_id,
                chain_name: tx_ir.chain.name.clone(),
                total_transactions: 0,
                total_volume: 0,
                unique_addresses: HashSet::new(),
                operation_counts: HashMap::new(),
            });

        analytics.total_transactions += 1;

        for operation in &tx_ir.operations {
            let op_type = match operation {
                Operation::Transfer(t) => {
                    analytics.total_volume += t.amount.value;
                    analytics.unique_addresses.insert(hex::encode(&t.from.bytes));
                    analytics.unique_addresses.insert(hex::encode(&t.to.bytes));
                    "transfer"
                }
                Operation::ContractCall(_) => "contract_call",
                Operation::ContractDeploy(_) => "contract_deploy",
                Operation::Stake(_) => "stake",
                Operation::Generic(_) => "generic",
            };

            *analytics.operation_counts.entry(op_type.to_string()).or_insert(0) += 1;
        }

        Ok(())
    }

    pub fn top_chains_by_volume(&self, limit: usize) -> Vec<&ChainAnalytics> {
        let mut chains: Vec<_> = self.analytics.values().collect();
        chains.sort_by(|a, b| b.total_volume.cmp(&a.total_volume));
        chains.into_iter().take(limit).collect()
    }
}
```

---

## Database Schemas

### PostgreSQL Schema for Block Explorer

```sql
-- Main transactions table
CREATE TABLE transactions (
    tx_hash VARCHAR(66) NOT NULL,
    chain_id BIGINT NOT NULL,
    block_height BIGINT NOT NULL,
    timestamp BIGINT NOT NULL,
    size INTEGER NOT NULL,
    num_operations INTEGER NOT NULL,
    tx_data JSONB NOT NULL,
    canonical_bytes BYTEA NOT NULL,
    PRIMARY KEY (tx_hash, chain_id)
);

CREATE INDEX idx_transactions_block ON transactions(chain_id, block_height DESC);
CREATE INDEX idx_transactions_timestamp ON transactions(chain_id, timestamp DESC);

-- Operations table
CREATE TABLE operations (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    chain_id BIGINT NOT NULL,
    op_index INTEGER NOT NULL,
    op_type VARCHAR(50) NOT NULL,
    from_address VARCHAR(66),
    to_address VARCHAR(66),
    amount NUMERIC(78, 0),
    function_signature VARCHAR(256),
    FOREIGN KEY (tx_hash, chain_id) REFERENCES transactions(tx_hash, chain_id)
);

CREATE INDEX idx_operations_from ON operations(chain_id, from_address);
CREATE INDEX idx_operations_to ON operations(chain_id, to_address);

-- UTXO inputs
CREATE TABLE transaction_inputs (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    chain_id BIGINT NOT NULL,
    input_index INTEGER NOT NULL,
    prev_tx_hash VARCHAR(66) NOT NULL,
    prev_output_index INTEGER NOT NULL,
    FOREIGN KEY (tx_hash, chain_id) REFERENCES transactions(tx_hash, chain_id)
);

-- UTXO outputs
CREATE TABLE transaction_outputs (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    chain_id BIGINT NOT NULL,
    output_index INTEGER NOT NULL,
    address VARCHAR(66),
    amount NUMERIC(78, 0) NOT NULL,
    spent BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (tx_hash, chain_id) REFERENCES transactions(tx_hash, chain_id)
);

CREATE INDEX idx_outputs_address ON transaction_outputs(chain_id, address);
CREATE INDEX idx_outputs_unspent ON transaction_outputs(chain_id) WHERE NOT spent;
```

---

## Batch Processing Patterns

### Parallel Block Processing

```rust
use futures::stream::{self, StreamExt};

pub async fn process_blocks_parallel(
    blocks: Vec<u64>,
    chain_family: ChainFamily,
    rpc: Arc<dyn BlockchainRpcClient>,
    explorer: Arc<ExplorerService>,
    concurrency: usize,
) -> Result<()> {
    stream::iter(blocks)
        .map(|block_height| {
            let rpc = rpc.clone();
            let explorer = explorer.clone();
            async move {
                let block = rpc.get_block(block_height).await?;
                for tx_bytes in block.transactions {
                    explorer.index_transaction(
                        chain_family,
                        &tx_bytes,
                        block_height,
                        block.timestamp,
                    ).await?;
                }
                Ok::<_, DecoderError>(())
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    Ok(())
}
```

### Checkpoint-based Resumable Processing

```rust
use std::fs;

pub struct CheckpointProcessor {
    checkpoint_file: String,
}

impl CheckpointProcessor {
    pub fn new(checkpoint_file: String) -> Self {
        Self { checkpoint_file }
    }

    fn load_checkpoint(&self) -> Result<u64> {
        if std::path::Path::new(&self.checkpoint_file).exists() {
            let content = fs::read_to_string(&self.checkpoint_file)
                .map_err(|e| DecoderError::custom(format!("Checkpoint read: {}", e)))?;
            content.trim().parse()
                .map_err(|e| DecoderError::custom(format!("Checkpoint parse: {}", e)))
        } else {
            Ok(0)
        }
    }

    fn save_checkpoint(&self, block_height: u64) -> Result<()> {
        fs::write(&self.checkpoint_file, block_height.to_string())
            .map_err(|e| DecoderError::custom(format!("Checkpoint write: {}", e)))
    }

    pub async fn process_with_checkpoints(
        &self,
        end_block: u64,
        chain_family: ChainFamily,
        rpc: Arc<dyn BlockchainRpcClient>,
        explorer: Arc<ExplorerService>,
    ) -> Result<()> {
        let start_block = self.load_checkpoint()?;

        for block_height in start_block..=end_block {
            let block = rpc.get_block(block_height).await
                .map_err(|e| DecoderError::custom(format!("RPC: {}", e)))?;

            for tx_bytes in block.transactions {
                explorer.index_transaction(
                    chain_family,
                    &tx_bytes,
                    block_height,
                    block.timestamp,
                ).await?;
            }

            if block_height % 100 == 0 {
                self.save_checkpoint(block_height)?;
                println!("Checkpoint: {}", block_height);
            }
        }

        self.save_checkpoint(end_block)?;
        Ok(())
    }
}
```

---

## Performance Optimization

### 1. Connection Pooling

```rust
use sqlx::postgres::PgPoolOptions;

pub async fn create_db_pool(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| DecoderError::custom(format!("DB connection: {}", e)))
}
```

### 2. Batch Database Inserts

```rust
pub async fn batch_insert_operations(
    db: &PgPool,
    operations: &[(String, u64, usize, Operation)],
) -> Result<()> {
    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO operations (tx_hash, chain_id, op_index, op_type)"
    );

    query_builder.push_values(operations, |mut b, (tx_hash, chain_id, index, op)| {
        if let Operation::Transfer(_) = op {
            b.push_bind(tx_hash)
             .push_bind(*chain_id as i64)
             .push_bind(*index as i32)
             .push_bind("transfer");
        }
    });

    query_builder.build()
        .execute(db)
        .await
        .map_err(|e| DecoderError::custom(format!("Batch insert: {}", e)))?;

    Ok(())
}
```

### 3. Caching

```rust
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CachedExplorer {
    explorer: Arc<ExplorerService>,
    tx_cache: Arc<RwLock<LruCache<String, Vec<u8>>>>,
}

impl CachedExplorer {
    pub fn new(explorer: Arc<ExplorerService>, cache_size: usize) -> Self {
        Self {
            explorer,
            tx_cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(cache_size).unwrap()
            ))),
        }
    }
}
```

---

## Error Handling Strategies

### Retry with Exponential Backoff

```rust
use tokio::time::{sleep, Duration};

pub async fn retry_with_backoff<F, Fut, T>(
    mut f: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) if retries < max_retries => {
                retries += 1;
                let backoff = Duration::from_millis(100 * 2u64.pow(retries));
                eprintln!("Retry {} after {:?}", retries, backoff);
                sleep(backoff).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Production Deployment

### Docker Compose Setup

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: blockchain_explorer
      POSTGRES_USER: explorer
      POSTGRES_PASSWORD: secret
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  indexer:
    build: .
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://explorer:secret@postgres/blockchain_explorer
      CONCURRENCY: 10
    command: cargo run --release --bin indexer

volumes:
  postgres_data:
```

---

## Basic Usage Examples

### Decoding Bitcoin

```rust
use decoder_bitcoin::BitcoinDecoder;
use universal_decoder_core::prelude::*;

fn main() -> Result<()> {
    let tx_hex = "01000000010000...";
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex)?;

    let tx = BitcoinDecoder::decode(&tx_bytes)?;
    println!("TXID: {}", hex::encode(tx.txid()));

    let tx_ir = tx.canonicalize()?;
    println!("Operations: {}", tx_ir.operations.len());

    Ok(())
}
```

### Decoding Ethereum

```rust
use decoder_ethereum::EthereumDecoder;
use universal_decoder_core::prelude::*;

fn main() -> Result<()> {
    let tx_hex = "f86c...";
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex)?;

    let tx = EthereumDecoder::decode(&tx_bytes)?;
    println!("Hash: {}", hex::encode(tx.hash()));

    let tx_ir = tx.canonicalize()?;
    println!("Chain: {}", tx_ir.chain.name);

    Ok(())
}
```

---

## Conclusion

The Universal Blockchain Decoder provides a **type-safe, performant, and extensible** foundation for building blockchain indexers, explorers, and analytics tools.

**Key Benefits:**

✅ **Chain-Agnostic**: Process 40+ blockchain families with unified API
✅ **Production-Ready**: Minimal dependencies, formally verified core
✅ **Performant**: Zero-cost abstractions, batch processing support
✅ **Extensible**: Trait-based architecture, custom hooks
✅ **Verifiable**: Canonical serialization (Borsh), deterministic hashing

**Resources:**

- **Architecture**: See `CLAUDE.md`
- **Roadmap**: See `ROADMAP.md`
- **Testing**: See `docs/TESTING_STRATEGY.md`
- **Live Demo**: https://trustless-txir.netlify.app

---

**Last Updated**: 2025-11-18
**Version**: 1.0.0
