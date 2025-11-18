# Blockchain Indexer Comparison

## How Universal Blockchain Decoder differs from existing tools

This is an honest comparison focused on **architectural differences**, not marketing claims.

---

## What Universal Decoder Actually Is

A **Rust library** that:
- Parses raw transaction bytes from 40+ blockchain families (Bitcoin, Ethereum, Cosmos, Solana, etc.)
- Converts them to a unified `TxIR` (Transaction Intermediate Representation)
- Runs in-process (no network calls, no external services)
- Is open source

**What it's NOT**:
- Not a hosted service or API
- Not a complete indexer (you build the database/storage layer)
- Not a GraphQL endpoint (you build the query layer)
- Not a compliance platform (you build risk scoring)

---

## Comparison with Alternatives

### The Graph

**What it is**: Decentralized protocol for building GraphQL APIs on top of blockchain data

**Key difference**:
- **The Graph**: Full stack (subgraph + Graph Node + GraphQL API + IPFS)
- **Universal Decoder**: Just the parsing layer (you build the rest)

**The Graph approach**:
```typescript
// Define schema + handlers, deploy to Graph Network
export function handleTransfer(event: TransferEvent): void {
  let entity = new Transfer(...)
  entity.save() // Stored in Graph Node's PostgreSQL
}

// Query via GraphQL
query { transfers(where: {to: "0x..."}) { from, to, amount } }
```

**Universal Decoder approach**:
```rust
// Decode transaction
let tx = EthereumDecoder::decode(raw_bytes)?;
let tx_ir = tx.canonicalize()?;

// YOU decide where to store it (PostgreSQL, ClickHouse, etc.)
// YOU build the query API (REST, GraphQL, gRPC, whatever)
```

**Choose The Graph if**: You want a complete GraphQL indexing solution, especially for dApps

**Choose Universal Decoder if**: You want control over storage/queries, need non-EVM chains, or building an explorer from scratch

---

### Subsquid

**What it is**: TypeScript framework for blockchain indexing with pre-indexed archives

**Key difference**:
- **Subsquid**: Framework + archive service + processor runtime
- **Universal Decoder**: Just a parsing library

**Subsquid approach**:
```typescript
const processor = new EvmBatchProcessor()
  .setDataSource({ archive: 'https://eth.archive.subsquid.io' })
  .addTransaction({...})

processor.run(new TypeormDatabase(), async ctx => {
  // Process batches of blocks from Subsquid archive
})
```

**Universal Decoder approach**:
```rust
// You fetch from RPC/node however you want
let raw_tx = your_rpc_client.get_transaction(txid)?;
let tx_ir = decode_transaction(&raw_tx)?;
// You store it however you want
```

**Choose Subsquid if**: TypeScript shop, EVM-focused, want managed archives

**Choose Universal Decoder if**: Rust shop, need non-EVM chains, want full control

---

### Chain-Specific Libraries (bitcoin, alloy, etc.)

**What they are**: Native libraries for each blockchain (e.g., `bitcoin` crate for Bitcoin, `alloy` for Ethereum)

**Key difference**:
- **Chain-specific**: Different API per chain, optimized for that chain
- **Universal Decoder**: Same API for all chains, normalized output

**Chain-specific approach**:
```rust
// Bitcoin
use bitcoin::Transaction as BtcTx;
let btc_tx: BtcTx = deserialize(&bytes)?;

// Ethereum (different API)
use alloy::Transaction as EthTx;
let eth_tx: EthTx = decode(&bytes)?;

// Problem: Can't write generic code across chains
```

**Universal Decoder approach**:
```rust
// Same API, same output type
fn process(chain_id: u64, bytes: &[u8]) -> Result<TxIR> {
    match chain_id {
        1 => BitcoinDecoder::decode(bytes)?.canonicalize(),
        2 => EthereumDecoder::decode(bytes)?.canonicalize(),
        _ => todo!()
    }
}
```

**Trade-off**: Chain-specific libraries might be slightly faster (no canonicalization overhead), but you can't write generic multi-chain code.

**Choose chain-specific if**: Single-chain app, need absolute max performance

**Choose Universal Decoder if**: Multi-chain app (10+ chains)

---

### Blockchair / Etherscan API

**What they are**: Hosted APIs that return pre-decoded transaction data

**Key difference**:
- **APIs**: Network call to their servers, rate limited, costs money
- **Universal Decoder**: Local parsing, no network, no rate limits, free

**API approach**:
```python
# Network call, rate limited
response = requests.get('https://api.etherscan.io/api?...')
tx_data = response.json()['result']
```

**Universal Decoder approach**:
```rust
// Local parsing, no network
let tx_bytes = ...; // From your own node
let tx = EthereumDecoder::decode(&tx_bytes)?;
```

**Trade-off**: APIs are easier for prototyping, but don't scale to high volume and create external dependencies.

**Choose APIs if**: Prototyping, low volume (< 1000 tx/day)

**Choose Universal Decoder if**: Production system, high volume, or need offline operation

---

### Chainalysis

**What it is**: Enterprise compliance platform with risk scoring and attribution

**Key difference**:
- **Chainalysis**: Proprietary compliance tool with pre-built risk models (costs $$$)
- **Universal Decoder**: Open source parsing library (you build risk models)

Chainalysis provides things we don't:
- Risk scoring (which addresses are mixers, sanctioned entities, etc.)
- Attribution database (which exchange owns which address)
- Regulatory compliance reports

Universal Decoder provides things they don't:
- Open source (can audit)
- Privacy chain support (Zcash, Monero parsers)
- Full transaction data access

**Choose Chainalysis if**: Need off-the-shelf compliance solution, have budget

**Choose Universal Decoder if**: Want to build custom compliance logic, need transparency

---

### Dune Analytics

**What it is**: SQL-based SaaS for querying pre-indexed blockchain data

**Key difference**:
- **Dune**: Managed service, SQL interface, pre-indexed data
- **Universal Decoder**: Library you integrate into your own system

Not really comparable - different use cases. Dune is for analysts running queries, Universal Decoder is for engineers building systems.

---

## What We Know vs. Don't Know

### We Know:
- ✅ Universal Decoder supports 40+ blockchain families
- ✅ It's open source (MIT/Apache-2.0)
- ✅ Written in Rust (can call from other languages via FFI/WASM)
- ✅ Outputs a unified `TxIR` type across all chains
- ✅ Runs in-process (no external services)
- ✅ Core is designed for formal verification (~2,500 LOC)

### We Don't Know (needs benchmarking):
- ❓ Actual performance vs chain-specific libraries
- ❓ Memory usage at scale
- ❓ Real-world adoption/production usage
- ❓ How much code reduction in practice (depends on use case)

---

## Decision Guide

**Use Universal Decoder if you're building:**
- Multi-chain explorer (need to support 10+ chains)
- Multi-chain indexer (need unified data model)
- Compliance tool that needs privacy chains (Zcash, Monero)
- System that must run offline/airgapped (no external API calls)
- Rust-based infrastructure

**Don't use it if:**
- Building a dApp → use The Graph
- Need GraphQL API → use The Graph or Subsquid
- TypeScript shop with no Rust expertise → use Subsquid
- Single-chain app → use chain-specific library
- Just need quick queries → use Dune or Etherscan API

---

## Architectural Philosophy

**The "Pandoc for Blockchains" analogy**:

Just like Pandoc converts between document formats (Markdown → HTML → LaTeX), Universal Decoder converts between blockchain transaction formats (Bitcoin → TxIR ← Ethereum).

It's a **building block**, not a complete solution. You combine it with:
- Your choice of database (PostgreSQL, ClickHouse, etc.)
- Your choice of API layer (REST, GraphQL, gRPC)
- Your choice of processing logic (batch, streaming, real-time)

---

## Honest Limitations

**What's missing** (as of 2025-11-18):
1. Some blockchains not yet implemented (see ROADMAP.md)
2. No hosted service (you must self-host)
3. No pre-built GraphQL/REST API (you build it)
4. Rust-only (FFI bindings for other languages are WIP)
5. No built-in risk scoring or compliance features
6. Documentation could be better (we're working on it)

**What we're working on**:
- More chain support (Phase 3 in ROADMAP.md)
- Formal verification with Verus (Phase 1.5 complete)
- Better examples and documentation (in progress)

---

## Try It Yourself

**Live WASM demo**: https://trustless-txir.netlify.app
Decode transactions from 500+ EVM chains, Bitcoin, and Cosmos in your browser.

**Code examples**: See `LIBRARY_USAGE.md`

**Questions?** Open an issue: https://github.com/prasincs/universal-blockchain-decoder/issues

---

**Version**: 1.1.0 (Honest Edition)
**Last Updated**: 2025-11-18
