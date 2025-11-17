#!/usr/bin/env python3
"""
Fetch chain metrics (TVL, volume, market cap) for blockchain visualization.

Data Sources:
- CoinGecko API (free tier): Market cap, 24h volume, price
- DefiLlama API: TVL by chain
- L2Beat: L2-specific metrics

Usage:
    python fetch_metrics.py --output data/snapshot_2025-01-16.json
"""

import argparse
import json
import time
from datetime import datetime
from typing import Dict, List, Optional
import sys

try:
    import requests
except ImportError:
    print("Error: requests library not installed. Install with: pip install requests")
    sys.exit(1)


# Chain ID mappings to CoinGecko IDs
COINGECKO_CHAIN_MAP = {
    # Major chains
    "bitcoin": {"chain_id": 0, "family": "utxo"},
    "ethereum": {"chain_id": 1, "family": "account", "evm": True},
    "binancecoin": {"chain_id": 56, "family": "account", "evm": True},
    "cardano": {"chain_id": 1010, "family": "utxo"},
    "solana": {"chain_id": 101, "family": "account", "svm": True},
    "polkadot": {"chain_id": 1009, "family": "account"},
    "dogecoin": {"chain_id": 3, "family": "utxo"},
    "polygon": {"chain_id": 137, "family": "account", "evm": True},
    "avalanche-2": {"chain_id": 43114, "family": "account", "evm": True},
    "cosmos": {"chain_id": 118, "family": "account", "cosmos": True},
    "tron": {"chain_id": 1007, "family": "account"},
    "litecoin": {"chain_id": 2, "family": "utxo"},
    "chainlink": {"chain_id": 1, "family": "account", "evm": True},  # Token on Ethereum
    "near": {"chain_id": 1003, "family": "account"},
    "stellar": {"chain_id": 1004, "family": "account"},
    "algorand": {"chain_id": 1006, "family": "account"},
    "aptos": {"chain_id": 1001, "family": "account", "movevm": True},
    "sui": {"chain_id": 1002, "family": "account", "movevm": True},
    "optimism": {"chain_id": 10, "family": "account", "evm": True, "opstack": True},
    "arbitrum": {"chain_id": 42161, "family": "account", "evm": True, "arbitrum": True},
    "base": {"chain_id": 8453, "family": "account", "evm": True, "opstack": True},
    "zksync": {"chain_id": 324, "family": "account", "evm": True, "zkevm": True},
    "starknet": {"chain_id": "starknet", "family": "account", "zkevm": True},
    "fantom": {"chain_id": 250, "family": "account", "evm": True},
    "celo": {"chain_id": 42220, "family": "account", "evm": True},
    "gnosis": {"chain_id": 100, "family": "account", "evm": True},
    "moonbeam": {"chain_id": 1284, "family": "account", "evm": True},
    "zcash": {"chain_id": 133, "family": "privacy"},
}


def fetch_coingecko_top_coins(limit: int = 100) -> List[Dict]:
    """
    Fetch top coins by market cap from CoinGecko.

    Rate limit: 10-50 calls/minute on free tier
    """
    url = "https://api.coingecko.com/api/v3/coins/markets"
    params = {
        "vs_currency": "usd",
        "order": "market_cap_desc",
        "per_page": limit,
        "page": 1,
        "sparkline": False,
        "price_change_percentage": "24h"
    }

    try:
        response = requests.get(url, params=params, timeout=30)
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Warning: CoinGecko API failed: {e}")
        return []


def map_coin_to_chain(coin_data: Dict) -> Optional[Dict]:
    """Map CoinGecko coin data to chain metrics."""
    coin_id = coin_data.get("id")

    if coin_id not in COINGECKO_CHAIN_MAP:
        return None

    chain_info = COINGECKO_CHAIN_MAP[coin_id]

    return {
        "chain_id": chain_info["chain_id"],
        "name": coin_data.get("name"),
        "symbol": coin_data.get("symbol", "").upper(),
        "family": chain_info.get("family", "account"),
        "market_cap": coin_data.get("market_cap", 0),
        "volume_24h": coin_data.get("total_volume", 0),
        "price": coin_data.get("current_price", 0),
        "price_change_24h": coin_data.get("price_change_percentage_24h", 0),
        "evm": chain_info.get("evm", False),
        "opstack": chain_info.get("opstack", False),
        "arbitrum": chain_info.get("arbitrum", False),
        "zkevm": chain_info.get("zkevm", False),
        "cosmos": chain_info.get("cosmos", False),
        "svm": chain_info.get("svm", False),
        "movevm": chain_info.get("movevm", False),
    }


def create_snapshot(output_path: str, dry_run: bool = False) -> Dict:
    """Create a metrics snapshot and save to file."""
    print("Fetching chain metrics from CoinGecko...")

    # Fetch data
    coins = fetch_coingecko_top_coins(limit=100)
    print(f"Fetched {len(coins)} coins from CoinGecko")

    # Map to chains
    chains = {}
    for coin in coins:
        chain = map_coin_to_chain(coin)
        if chain:
            chain_id = str(chain["chain_id"])
            chains[chain_id] = chain
            print(f"  ✓ {chain['name']} (chain_id={chain_id})")

    print(f"\nMapped {len(chains)} chains")

    # Create snapshot
    snapshot = {
        "version": "1.0",
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "snapshot_date": datetime.utcnow().strftime("%Y-%m-%d"),
        "source": "CoinGecko API v3",
        "description": "Chain metrics snapshot (market cap, 24h volume, price)",
        "chain_count": len(chains),
        "chains": chains,
        "metadata": {
            "fetched_at": datetime.utcnow().isoformat() + "Z",
            "api": "CoinGecko",
            "rate_limit": "10-50 calls/minute (free tier)",
            "metrics": ["market_cap", "volume_24h", "price", "price_change_24h"],
            "note": "Market cap used as proxy for TVL. Real TVL would come from DefiLlama."
        }
    }

    if dry_run:
        print("\n[DRY RUN] Would save snapshot to:", output_path)
        print(json.dumps(snapshot, indent=2))
        return snapshot

    # Save to file
    with open(output_path, 'w') as f:
        json.dump(snapshot, f, indent=2)

    print(f"\n✓ Snapshot saved to: {output_path}")
    print(f"  Chains: {len(chains)}")
    print(f"  Timestamp: {snapshot['timestamp']}")

    return snapshot


def main():
    parser = argparse.ArgumentParser(description="Fetch chain metrics for visualization")
    parser.add_argument(
        "--output",
        default="data/snapshot_latest.json",
        help="Output path for snapshot JSON"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print snapshot without saving"
    )

    args = parser.parse_args()

    create_snapshot(args.output, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
