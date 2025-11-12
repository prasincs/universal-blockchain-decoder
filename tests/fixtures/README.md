# Test Fixtures

This directory contains real blockchain transaction data for integration testing.

## Structure

```
fixtures/
├── bitcoin/          # Bitcoin mainnet transactions
│   ├── legacy/       # Pre-SegWit transactions
│   ├── segwit/       # SegWit (BIP 141, 143, 144) transactions
│   ├── taproot/      # Taproot (BIP 341, 342) transactions
│   └── special/      # Coinbase, multisig, etc.
│
├── ethereum/         # Ethereum mainnet transactions
│   ├── legacy/       # Pre-EIP-1559 transactions
│   ├── eip1559/      # EIP-1559 (London fork) transactions
│   ├── eip2930/      # EIP-2930 (access list) transactions
│   └── special/      # Contract creation, ERC-20, etc.
│
├── solana/           # Solana mainnet transactions
│   ├── simple/       # Simple SOL transfers
│   ├── instructions/ # Complex instruction-based transactions
│   └── special/      # Token transfers, NFT mints, etc.
│
└── common/           # Shared test utilities
    └── invalid/      # Invalid transactions (should fail to decode)
```

## Fixture Format

All fixtures are JSON files with the following structure:

```json
{
  "description": "Human-readable description",
  "chain": "bitcoin|ethereum|solana",
  "raw_hex": "hex-encoded transaction bytes",
  "expected": {
    "should_decode": true,
    "tx_hash": "expected transaction hash (hex)",
    "version": 1,
    "num_inputs": 2,
    "num_outputs": 2,
    "value": "1000000",
    "fee": "10000",
    "is_segwit": true,
    "tx_type": "legacy|eip1559|eip2930"
  },
  "metadata": {
    "source": "bitcoin-core|etherscan|solscan",
    "block_number": 100000,
    "block_hash": "block hash (hex)",
    "network": "mainnet|testnet",
    "explorer_url": "https://...",
    "tags": ["segwit", "multisig"]
  }
}
```

## Naming Convention

- `{chain}_{type}_{id}.json` (e.g., `btc_segwit_001.json`)
- Use sequential numbering for similar transaction types
- Keep filenames under 50 characters

## Adding New Fixtures

1. **Source**: Get transaction data from:
   - Bitcoin: Bitcoin Core test vectors, blockchain explorers
   - Ethereum: Etherscan, alloy test vectors
   - Solana: Solscan, solana-test-validator

2. **Validation**: Verify with reference implementation (in dev-dependencies)

3. **Documentation**: Add comprehensive `expected` properties

4. **Tags**: Use descriptive tags for categorization

## Test Integration

Load fixtures in tests using `decoder-test-utils`:

```rust
use decoder_test_utils::fixtures::{load_fixture, load_fixtures_dir};

#[test]
fn test_bitcoin_segwit_transactions() {
    let fixtures = load_fixtures_dir("tests/fixtures/bitcoin/segwit");

    for fixture in fixtures {
        let tx_bytes = fixture.raw_bytes();
        let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

        // Validate against expected properties
        if let Some(expected_hash) = fixture.expected_tx_hash() {
            let actual_hash = decoded.hash();
            assert_eq!(actual_hash, expected_hash);
        }
    }
}
```

## Coverage Goals

**Phase 1.5.2 Target**: 100+ fixtures

- Bitcoin: 40+ fixtures
  - 10 legacy
  - 15 SegWit
  - 5 Taproot
  - 10 special cases

- Ethereum: 40+ fixtures
  - 15 legacy
  - 15 EIP-1559
  - 5 EIP-2930
  - 5 special cases

- Solana: 20+ fixtures
  - 10 simple transfers
  - 10 instruction-based

## Source Attribution

All fixtures sourced from:
- Bitcoin Core test vectors (MIT License)
- Ethereum test vectors (MIT License)
- Public blockchain explorers (factual data)

Each fixture's `metadata.source` field indicates origin.
