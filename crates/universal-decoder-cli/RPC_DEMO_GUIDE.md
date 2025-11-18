# RPC Transaction Decoder Demo Guide

This guide shows you how to use the **RPC fetching** and **explorer URL parsing** features to quickly demo the Universal Blockchain Decoder with real transactions.

## Quick Start: Copy-Paste a Transaction URL

The easiest way to demo the decoder is to copy a transaction URL from your phone or browser and paste it directly:

```bash
# Ethereum transaction from Etherscan
universal-tx-decoder --from-url "https://etherscan.io/tx/0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060" --fetch -v

# Bitcoin transaction from Mempool.space
universal-tx-decoder --from-url "https://mempool.space/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b" --fetch -v

# Polygon transaction
universal-tx-decoder --from-url "https://polygonscan.com/tx/0xabc..." --fetch -v
```

The `--from-url` flag automatically:
1. ✅ Detects the blockchain from the explorer domain
2. ✅ Extracts the transaction ID
3. ✅ Fetches the raw transaction via public RPC (with `--fetch`)
4. ✅ Decodes and displays it

## Supported Explorer URLs

| Blockchain | Supported Explorers |
|------------|-------------------|
| **Bitcoin** | blockchain.com, mempool.space, blockstream.info, blockchair.com/bitcoin |
| **Ethereum** | etherscan.io |
| **BSC** | bscscan.com |
| **Polygon** | polygonscan.com |
| **Avalanche** | snowtrace.io, avascan.info |
| **Optimism** | optimistic.etherscan.io |
| **Arbitrum** | arbiscan.io |
| **Litecoin** | blockchair.com/litecoin, litecoinspace.org |
| **Dogecoin** | blockchair.com/dogecoin, dogechain.info |
| **Zcash** | blockchair.com/zcash, zcha.in |
| **Solana** | solscan.io, explorer.solana.com |

## Usage Modes

### Mode 1: Explorer URL + Auto-Fetch (Easiest!)

```bash
# Just paste the URL and add --fetch
universal-tx-decoder --from-url "PASTE_URL_HERE" --fetch -v
```

**Example**:
```bash
universal-tx-decoder \
  --from-url "https://etherscan.io/tx/0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060" \
  --fetch \
  -v
```

**Output**:
```
Parsed explorer URL:
  Chain:  eth
  TxID:   0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060
Fetching transaction 0x5c504... from RPC endpoint...
Warning: Using public RPC endpoint (rate-limited)
Successfully fetched 219 bytes

=== Universal Blockchain Transaction Decoder ===

Chain:                  Ethereum
Chain ID:               1
Family:                 Account
Raw transaction size:   219 bytes
...
```

### Mode 2: Manual Chain + TxID + Fetch

If the explorer isn't supported, manually specify the chain:

```bash
universal-tx-decoder -c eth --fetch --txid 0xabc123... -v
```

**Example**:
```bash
universal-tx-decoder \
  -c btc \
  --fetch \
  --txid 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b \
  -v
```

### Mode 3: Custom RPC Endpoint (Production)

For production use or higher rate limits, provide your own RPC endpoint:

```bash
# Ethereum with Infura
universal-tx-decoder \
  -c eth \
  --fetch \
  --txid 0xabc... \
  --rpc-endpoint "https://mainnet.infura.io/v3/YOUR-API-KEY"

# Bitcoin with custom node
universal-tx-decoder \
  -c btc \
  --fetch \
  --txid abc123... \
  --rpc-endpoint "https://your-bitcoin-node.com:8332"
```

You can also set the endpoint via environment variable:

```bash
export RPC_ENDPOINT="https://mainnet.infura.io/v3/YOUR-KEY"
universal-tx-decoder -c eth --fetch --txid 0xabc...
```

## Demo Workflow: Phone to CLI

Perfect for live demos at conferences or presentations:

1. **On your phone**: Open a blockchain explorer (Etherscan, Mempool.space, etc.)
2. **Find a transaction**: Browse to any interesting transaction
3. **Copy the URL**: Long-press the URL bar and copy
4. **On your laptop**: Paste into terminal:
   ```bash
   universal-tx-decoder --from-url "PASTE_HERE" --fetch -v
   ```
5. **Watch it decode!** ✨

## Show Canonical IR Representation

Add `--canonical` or `-C` to show the chain-agnostic intermediate representation:

```bash
universal-tx-decoder \
  --from-url "https://etherscan.io/tx/0x5c504ed4..." \
  --fetch \
  -v \
  -C
```

This displays:
- Version and operation count
- State deltas (inputs/outputs)
- **Canonical hash** (deterministic, cross-chain comparable)
- Operations breakdown (transfers, contract calls, etc.)

## Public RPC Endpoints (Demo Use Only)

When you don't provide `--rpc-endpoint`, the CLI uses these public endpoints:

| Chain | Public Endpoint | Rate Limit |
|-------|----------------|------------|
| Bitcoin | https://blockstream.info/api | Low |
| Ethereum | https://eth.llamarpc.com | Low |
| BSC | https://bsc-dataseed.binance.org | Medium |
| Polygon | https://polygon-rpc.com | Low |
| Avalanche | https://api.avax.network/ext/bc/C/rpc | Low |
| Optimism | https://mainnet.optimism.io | Low |
| Arbitrum | https://arb1.arbitrum.io/rpc | Low |
| Solana | https://api.mainnet-beta.solana.com | Very Low |

**⚠️ Warning**: Public endpoints are rate-limited and may be unreliable. For production or heavy use, provide your own RPC endpoint.

## Output Formats

```bash
# Human-readable (default)
universal-tx-decoder --from-url "..." --fetch

# JSON output
universal-tx-decoder --from-url "..." --fetch -o json

# Hex output (raw bytes)
universal-tx-decoder --from-url "..." --fetch -o hex
```

## Troubleshooting

### "Failed to fetch transaction"

**Cause**: Public RPC is rate-limited or transaction not found.

**Solution**:
1. Wait a few seconds and retry
2. Use `--rpc-endpoint` with your own API key
3. Verify the transaction ID is correct

### "Unsupported explorer domain"

**Cause**: Explorer not recognized by URL parser.

**Solution**: Use manual mode:
```bash
universal-tx-decoder -c CHAIN --fetch --txid TXID
```

### "No public endpoint available"

**Cause**: No public RPC endpoint configured for that chain.

**Solution**: Provide your own:
```bash
universal-tx-decoder -c CHAIN --fetch --txid TXID --rpc-endpoint "YOUR_RPC_URL"
```

## Advanced Examples

### Decode and Save Canonical IR

```bash
# Fetch, decode, and show canonical IR
universal-tx-decoder \
  --from-url "https://etherscan.io/tx/0xabc..." \
  --fetch \
  -C \
  -o json > transaction.json
```

### Batch Processing (from file)

```bash
# URLs in urls.txt, one per line
while read url; do
  echo "Processing: $url"
  universal-tx-decoder --from-url "$url" --fetch -o json
done < urls.txt
```

### Privacy Chains with Viewing Keys

```bash
# Zcash shielded transaction
universal-tx-decoder \
  -c zec \
  --fetch \
  --txid abc123... \
  --viewing-key-file ~/.zcash/viewkey \
  --decrypt
```

## Architecture: Network Code is CLI-Only

**Important**: The RPC fetching functionality is **only in the CLI binary**, not in the core library. This preserves the core library's ability to operate in airgapped environments.

```
┌─────────────────────────────────────┐
│  CLI Binary (network code OK)      │
│  ├─ rpc_fetcher.rs (HTTP requests)  │  ← Network code HERE
│  ├─ explorer_parser.rs (URL parse)  │
│  └─ main.rs (orchestration)         │
└──────────────┬──────────────────────┘
               │ uses
               ▼
┌─────────────────────────────────────┐
│  Core Library (NO network code)     │  ← Airgapped operation
│  - ChainDecoder trait                │
│  - TxIR types                        │
│  - Canonical serialization           │
└─────────────────────────────────────┘
```

This design allows:
- ✅ **Security**: Core library has zero network dependencies
- ✅ **Airgapped deployments**: Banks, enterprises can use core library offline
- ✅ **Convenience**: CLI users get easy RPC fetching for demos
- ✅ **Flexibility**: Users can choose: airgapped OR networked mode

## Real Transaction Examples

Try these real transactions:

```bash
# Ethereum: Uniswap V3 swap
universal-tx-decoder --from-url "https://etherscan.io/tx/0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060" --fetch -v

# Bitcoin: First Pizza transaction
universal-tx-decoder --from-url "https://mempool.space/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b" --fetch -v

# Polygon: High-value transfer
universal-tx-decoder -c matic --fetch --txid 0x123... --rpc-endpoint "https://polygon-rpc.com"
```

## Next Steps

- See `CLI.md` for full CLI documentation
- See `LIBRARY_USAGE.md` for using the decoder as a library
- See `ROADMAP.md` for upcoming features

---

**Questions?** Open an issue at https://github.com/prasincs/universal-blockchain-decoder/issues
