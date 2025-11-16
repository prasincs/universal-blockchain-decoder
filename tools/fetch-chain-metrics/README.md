# Chain Metrics Fetcher

Fetch chain metrics (TVL, market cap, volume) for the WASM treemap visualization.

## Overview

This tool creates **snapshots** of blockchain metrics that are:
- ✅ **Committed to the repository** (no runtime network dependencies)
- ✅ **Verifiable** (git commit shows source & timestamp)
- ✅ **Reproducible** (can regenerate and compare)
- ✅ **Historical** (supports multiple snapshots for animation)

Aligns with CLAUDE.md's principle of **airgapped operation**.

## Data Sources

### Primary: CoinGecko API (Free Tier)
- **Endpoint**: `https://api.coingecko.com/api/v3/coins/markets`
- **Rate Limit**: 10-50 calls/minute
- **CORS**: ✅ Enabled
- **Metrics**: Market cap (proxy for TVL), 24h volume, price

### Future: DefiLlama API
- **Endpoint**: `https://api.llama.fi/v2/chains`
- **Metrics**: Real TVL per chain
- TODO: Add when API access stabilizes

## Usage

### Install Dependencies

```bash
pip install requests
```

### Fetch Latest Snapshot

```bash
# Fetch to local data directory
python fetch_metrics.py --output data/snapshot_$(date +%Y-%m-%d).json

# Also create/update snapshot_latest.json
python fetch_metrics.py --output data/snapshot_latest.json

# For deployment (Netlify), copy to www directory:
cp data/snapshot_latest.json ../../crates/universal-decoder-wasm/www/data/snapshot_latest.json
```

### Dry Run (Preview Only)

```bash
python fetch_metrics.py --dry-run
```

## Snapshot Format

```json
{
  "version": "1.0",
  "timestamp": "2025-01-16T10:30:00Z",
  "snapshot_date": "2025-01-16",
  "source": "CoinGecko API v3",
  "chain_count": 30,
  "chains": {
    "1": {
      "chain_id": 1,
      "name": "Ethereum",
      "symbol": "ETH",
      "family": "account",
      "market_cap": 450000000000,
      "volume_24h": 15000000000,
      "price": 3500.00,
      "evm": true
    }
  },
  "metadata": {
    "fetched_at": "2025-01-16T10:30:00Z",
    "api": "CoinGecko",
    "metrics": ["market_cap", "volume_24h", "price"]
  }
}
```

## Update Schedule

**Recommended**: Monthly or when preparing for demos/presentations

```bash
# Create dated snapshot
./update_snapshot.sh

# Commit to repo
git add data/snapshot_*.json
git commit -m "data: Update chain metrics snapshot ($(date +%Y-%m))"
git push
```

## Historical Snapshots

Keep multiple snapshots for:
- **Comparison** over time
- **Animation** (future feature)
- **Verification** (cross-check data sources)

Naming convention: `snapshot_YYYY-MM-DD.json`

Example:
```
data/
├── snapshot_2025-01-16.json
├── snapshot_2024-12-15.json
├── snapshot_2024-11-10.json
└── snapshot_latest.json → snapshot_2025-01-16.json (symlink)
```

## Verification

```bash
# Check snapshot integrity
jq '.chain_count' data/snapshot_latest.json

# Verify timestamp
jq '.timestamp, .source' data/snapshot_latest.json

# List all chains
jq '.chains | keys' data/snapshot_latest.json
```

## Integration

Snapshots are loaded by `treemap.js`:

```javascript
// Load snapshot
const snapshot = await fetch('../../tools/fetch-chain-metrics/data/snapshot_latest.json');
const metrics = await snapshot.json();

// Apply to chain data
applyMetrics(CHAIN_DATA, metrics.chains);
```

## Future Enhancements

- [ ] Add DefiLlama TVL data
- [ ] Add L2Beat metrics for rollups
- [ ] Add transaction count from block explorers
- [ ] Snapshot comparison tool
- [ ] Animated treemap transitions between snapshots
