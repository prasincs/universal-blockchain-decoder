# Blockchain ETL Datasets Research: Integration Approaches for Universal Blockchain Decoder

## Executive Summary

The Universal Blockchain Decoder project has a **critical constraint**: complete offline/airgapped operation with zero runtime network dependencies. This fundamentally shapes which ETL datasets are viable and how they integrate.

**Current Status**:
- ✅ Phase 3.5+ (Production decoders implemented: Bitcoin, Ethereum, Solana, Cosmos, etc.)
- ✅ Airgapped operation fully enforced (no network calls in production code)
- ✅ Git subtree vendoring established (chainlists, registries, test fixtures)
- ✅ Build-time data embedding pattern proven (decoder-evm uses Borsh binary registry)
- ⚠️ Testing infrastructure ready but needs more real transaction datasets

---

## Part 1: Available Datasets & Feasibility Analysis

### 1. Google BigQuery Public Datasets

**Dataset**: `crypto_bitcoin`, `crypto_ethereum`, etc.

**Characteristics**:
- Historical blockchain data (blocks, transactions, addresses)
- Indexed and normalized (easier for analysis than raw blocks)
- Query API (SQL-based)
- ~1TB+ per chain

**Integration Feasibility**: ⚠️ INDIRECT ONLY

**Why Not Direct Integration**:
- Requires runtime HTTP calls to BigQuery API
- Violates airgapped requirement
- Would need API keys (security liability)

**Valid Integration Pattern**:
```
OFFLINE WORKFLOW:
1. Developer runs BigQuery export once: SELECT ... FROM crypto_bitcoin.transactions
2. Export results to CSV/JSON (locally)
3. Transform CSV → compact test fixtures (Borsh or JSON)
4. Commit fixtures to git repository
5. All production builds use vendored fixtures (zero network)

CI/CD WORKFLOW:
1. Scheduled job (e.g., weekly) exports latest BigQuery data
2. Transform and validate
3. Update fixtures in repository (via PR)
4. All deployments use committed fixtures
```

**High-Value Export Strategy**:
```sql
-- Sample 1000 representative Bitcoin transactions
SELECT 
    hash,
    version,
    input_count,
    output_count,
    block_timestamp,
    block_number
FROM crypto_bitcoin.transactions
WHERE block_timestamp BETWEEN timestamp('2021-01-01') AND timestamp('2023-12-31')
LIMIT 1000;
```

**Pros**:
- ✅ Structured, normalized data
- ✅ Easy to filter (by type, date range, complexity)
- ✅ High-quality validation data

**Cons**:
- ❌ Requires manual export workflow
- ❌ Cannot use at runtime
- ❌ Costs money (BigQuery is metered)

---

### 2. Allium Blockchain Data Platform

**Dataset**: Comprehensive chain data with better filtering than BigQuery

**Characteristics**:
- Optimized for blockchain analytics
- Better filtering/labeling (transaction types, contract interactions)
- Web UI for easy data exploration
- Export API (can download datasets)

**Integration Feasibility**: ⚠️ INDIRECT ONLY (same as BigQuery)

**Valid Use Cases**:
- Export specific transaction types (e.g., "all Taproot transactions in 2023")
- Label training data (for ML-based decoder testing)
- Generate synthetic test cases

**Example Workflow**:
```
1. Use Allium to find:
   - All ERC-20 transfers on Ethereum (2023)
   - All Swap transactions on Uniswap
   - All Bridge transactions on OP Stack
2. Download JSON export
3. Transform to fixture format
4. Commit to test fixtures
5. Create integration tests with real-world transaction patterns
```

**Pros**:
- ✅ Better transaction type labeling
- ✅ Web UI for exploration
- ✅ Optimized for blockchain analysis

**Cons**:
- ❌ Also requires manual export
- ❌ Not free
- ❌ Cannot be used at runtime

---

### 3. Blockchain Explorer APIs (Etherscan, Blockchain.com, Solscan, etc.)

**Available APIs**:
- **Etherscan**: `eth_getTransactionByHash`, `eth_blockNumber`, etc.
- **Blockchain.com**: Bitcoin transaction lookup
- **Solscan**: Solana transaction data
- **BlockScout**: Open-source explorer for EVM chains

**Integration Feasibility**: ⚠️ INDIRECT WITH RATE LIMITS

**Why Not Direct**:
- Requires API keys (security issue for production)
- Rate-limited (10-15 req/sec typical)
- Terms of service restrict automated scraping
- Violates airgapped requirement for production

**Valid Use Case: One-Time Fixture Generation**

```rust
// tools/fetch_test_fixtures.rs (dev-dependencies only, build-time)
// NOT in production code path

use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
    
    // Fetch famous transactions for testing
    let etherscan_api_key = std::env::var("ETHERSCAN_API_KEY")?;
    
    let txs = vec![
        "0x123...", // EIP-1559 transaction
        "0x456...", // Contract creation
        "0x789...", // Token transfer
    ];
    
    for tx_hash in txs {
        let response = client
            .get(&format!(
                "https://api.etherscan.io/api?module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
                tx_hash, etherscan_api_key
            ))
            .send()
            .await?;
        
        let tx_data = response.text().await?;
        std::fs::write(
            format!("tests/fixtures/ethereum/{}.json", &tx_hash[2..10]),
            tx_data
        )?;
    }
    
    Ok(())
}
```

**Pros**:
- ✅ Real, authoritative transaction data
- ✅ Can fetch arbitrary transactions
- ✅ Free (with rate limits)

**Cons**:
- ❌ Rate-limited
- ❌ Requires API keys
- ❌ Depends on third-party uptime
- ❌ Terms of service restrictions

---

### 4. Chain-Specific Test Vector Repositories

**Examples**:
- **Bitcoin Core**: BIP 341 test vectors (Taproot)
- **Ethereum**: EIP test vectors
- **Cosmos**: SDK test vectors
- **Zcash**: Test vectors (different signature types)

**Integration Feasibility**: ✅ EXCELLENT (Already doing this!)

**Current Implementation**:
```
crates/decoder-bitcoin/tests/fixtures/bitcoin-core/
├── tx_valid.json       # From Bitcoin Core test suite
├── tx_invalid.json     # Invalid transaction tests
└── bip341_wallet_vectors.json  # Taproot test vectors
```

**How It Works**:
```rust
#[test]
fn test_bitcoin_core_vectors() {
    let fixtures = load_fixtures_dir("tests/fixtures/bitcoin-core");
    for fixture in fixtures {
        // Each fixture is a [tx_in, tx_out] pair from BIP 341
        // Validate decoder against reference test vectors
    }
}
```

**Where to Find These**:
- **Bitcoin**: https://github.com/bitcoin/bitcoin/tree/master/src/test/data
- **Ethereum**: https://github.com/ethereum/tests (JSON test vectors)
- **Solana**: https://github.com/solana-labs/solana/tree/master/programs/
- **Cosmos SDK**: https://github.com/cosmos/cosmos-sdk/tree/main/testutil

**Pros**:
- ✅ Official, authoritative test data
- ✅ Covers edge cases and spec compliance
- ✅ Easy to integrate (GitHub + git subtree)
- ✅ Well-documented (in BIPs, EIPs)

**Cons**:
- ❌ Limited to core chains
- ❌ Fewer real-world transaction examples
- ❌ Sometimes minimal metadata

---

### 5. Historical Blockchain Data (Git Subtree Vendoring)

**Current Implementation** (Already proven!):

```bash
# Ethereum chain registry (chainlist.org)
git subtree add \
    --prefix crates/decoder-evm/vendored/chainlist \
    https://github.com/ethereum-lists/chains.git \
    master --squash

# Cosmos chain registry
git subtree add \
    --prefix crates/decoder-cosmos/vendored/chain-registry \
    https://github.com/cosmos/chain-registry.git \
    master --squash

# OP Stack chains
git subtree add \
    --prefix crates/decoder-optimism/vendored/superchain-registry \
    https://github.com/ethereum-optimism/superchain-registry.git \
    main --squash
```

**Data Transformation Pipeline**:
```
Raw JSON (14.5MB) ──> Transform ──> Borsh Binary (2MB)
   ↓
Load at startup (0 network calls)
   ↓
Used for chain validation in decoders
```

**Pros**:
- ✅ Completely airgapped (git history proves authenticity)
- ✅ 85% size reduction (JSON → Borsh)
- ✅ Verifiable via git commit hashes
- ✅ Zero runtime dependencies

**Cons**:
- ❌ Limited to data in public GitHub repos
- ❌ Requires git subtree management
- ❌ Size adds to repository (partially mitigated by Borsh)

---

## Part 2: Test Fixture Generation Strategy

### Current Testing Infrastructure

**5-Level Testing Pyramid**:
```
Level 5: Formal Verification (Verus proofs)
  └─ Core library panic-freedom, determinism

Level 4: Fuzz Testing (cargo-fuzz with libFuzzer)
  └─ Random input generation

Level 3: Integration Tests (Real blockchain data)
  └─ 100+ test fixtures per chain

Level 2: Property Tests (proptest)
  └─ Determinism, idempotence, commutativity

Level 1: Unit Tests (100% core coverage)
  └─ Individual functions, error cases
```

**Existing Fixture Counts**:
```
Bitcoin: 47 tests + fixtures
Ethereum: 6 tests + fixtures  
Solana: 13 tests + fixtures
Cosmos: 31 tests + fixtures
Zcash: 16 comprehensive mainnet tests
Total: 100+ real transaction tests
```

### Recommended Fixture Generation Pipeline

**Phase 1: Curate Representative Transactions** (Week 1-2)

```bash
# For each chain, collect:
# 1. Genesis/initial transactions (structural validation)
# 2. Standard transactions (happy path)
# 3. Complex transactions (edge cases)
# 4. Unusual but valid transactions (error resilience)
# 5. Known problematic cases (regression testing)

BITCOIN:
  - Genesis coinbase (✅ already have)
  - SegWit P2WPKH (✅ already have)
  - Taproot (✅ already have)
  - Multisig P2SH (✅ already have)
  - Lightning channel opens (NEW)
  - Stale blocks (NEW)

ETHEREUM:
  - Legacy transactions (✅ already have)
  - EIP-1559 transactions (✅ already have)
  - Contract creation (✅ already have)
  - ERC-20 transfers (NEW)
  - Failed transactions (NEW)
  - 0-value transactions (NEW)

SOLANA:
  - Simple SOL transfers (✅ already have)
  - Token transfers (✅ already have)
  - Complex instruction chains (NEW)
  - Failed transactions (NEW)
```

**Phase 2: Automated Fixture Download** (Week 2-3)

```rust
// tools/fetch_test_fixtures.rs (dev tool, not production)

use universal_decoder_core::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct TestFixture {
    description: String,
    chain: String,
    raw_hex: String,
    expected: ExpectedProperties,
    metadata: FixtureMetadata,
}

// Source 1: Bitcoin Core test vectors
fn fetch_bitcoin_core_vectors() -> Result<Vec<TestFixture>> {
    // Read from vendored tests/fixtures/bitcoin-core/*.json
    // Already in repository
}

// Source 2: Official chain test vectors (EIP, BIP)
fn fetch_official_test_vectors() -> Result<Vec<TestFixture>> {
    // Convert BIP 341, EIP-2718, etc. to fixture format
    // One-time conversion, store in git
}

// Source 3: Explorer APIs (rate-limited, dev-only)
fn fetch_from_explorers(api_keys: &ApiKeys) -> Result<Vec<TestFixture>> {
    // For well-known transactions only
    // Genesis blocks, famous transactions, etc.
}

// Source 4: Synthetic/generated transactions
fn generate_synthetic_fixtures() -> Result<Vec<TestFixture>> {
    // Use `proptest` to generate valid edge cases
    // Verify with reference implementations
}
```

**Phase 3: Validate Against Reference Implementations** (Week 3-4)

```rust
// tests/integration_tests.rs

#[test]
fn validate_fixtures_against_reference() {
    let fixtures = load_fixtures_dir("tests/fixtures");
    
    for fixture in fixtures {
        match fixture.chain.as_str() {
            "bitcoin" => {
                // Validate with `bitcoin` crate (dev-dependency)
                let ref_tx = bitcoin::consensus::decode::<bitcoin::Transaction>(
                    &fixture.raw_bytes()
                ).expect("Reference implementation should decode");
                
                let our_tx = decoder_bitcoin::BitcoinDecoder::decode(
                    &fixture.raw_bytes()
                ).expect("Our decoder should decode");
                
                // Compare key properties
                assert_eq!(our_tx.version, ref_tx.version);
                assert_eq!(our_tx.inputs.len(), ref_tx.input.len());
                // ... more assertions
            },
            "ethereum" => {
                // Validate with `alloy` or `ethers-core` (dev-dependency)
                // ...
            },
            _ => panic!("Unknown chain"),
        }
    }
}
```

---

## Part 3: Airgapped Operation Implementation

### Pattern: Build-Time Data Fetching

The project has already pioneered this with chain registries. Here's how to extend it:

**Directory Structure**:
```
tools/
├── fixture-generator/        # NEW: Generate test fixtures
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # CLI entry point
│   │   ├── bitcoin.rs       # Bitcoin fixture generation
│   │   ├── ethereum.rs      # Ethereum fixture generation
│   │   └── cosmos.rs        # Cosmos fixture generation
│   └── vendored/
│       ├── bitcoin-core/    # git subtree: bitcoin/bitcoin
│       ├── ethereum-tests/  # git subtree: ethereum/tests
│       └── bips/            # git subtree: bitcoin/bips
└── registry-generator/      # EXISTING: Chain registry generation
    ├── src/
    │   ├── main.rs
    │   └── evm.rs
    └── vendored/
        └── chainlist/       # git subtree: ethereum-lists/chains
```

**Build Script Verification Pattern**:

```rust
// crates/universal-decoder-core/build.rs

use std::path::Path;

fn main() {
    // Verify all vendored test fixtures exist
    verify_fixture("tests/fixtures/bitcoin/genesis_coinbase.json");
    verify_fixture("tests/fixtures/ethereum/legacy_tx.json");
    verify_fixture("tests/fixtures/solana/simple_transfer.json");
    
    // Rerun if fixtures change
    println!("cargo:rerun-if-changed=tests/fixtures");
}

fn verify_fixture(path: &str) {
    if !Path::new(path).exists() {
        eprintln!("\nMISSING TEST FIXTURE: {}", path);
        eprintln!("\nTo generate fixtures, run:");
        eprintln!("  cargo run -p fixture-generator -- --all");
        eprintln!("\nOr for specific chain:");
        eprintln!("  cargo run -p fixture-generator -- --chain bitcoin");
        panic!("Missing required test fixture");
    }
    println!("cargo:rerun-if-changed={}", path);
}
```

### Privacy Considerations

**Issue**: Some test fixtures may reveal personal information (sender/receiver addresses)

**Solution Pattern**:

```rust
// tools/fixture-generator/src/privacy.rs

fn anonymize_fixture(fixture: TestFixture) -> TestFixture {
    // 1. Replace addresses with test vectors (all zeros, all ones, random)
    // 2. Keep structural properties (length, type)
    // 3. Preserve validation properties (signatures still valid)
    
    let mut anon = fixture.clone();
    anon.expected.from_address = Some("0x0000000000000000000000000000000000000001".to_string());
    anon.expected.to_address = Some("0x0000000000000000000000000000000000000002".to_string());
    anon
}
```

---

## Part 4: Recommended Implementation Plan

### Phase 1: Expand Test Fixtures (2 weeks)

**Goal**: Grow from 100 to 500+ integration test fixtures

**Priority Order**:
1. **Bitcoin** (Complete chain test coverage):
   - 50 fixtures (currently have 10+)
   - All transaction types: legacy, segwit, taproot, multisig
   - Edge cases: empty inputs, oversized scripts

2. **Ethereum** (Complete EIP coverage):
   - 50 fixtures (currently have 5+)
   - All types: legacy, EIP-1559, EIP-2930, EIP-4844
   - Special cases: contract creation, failed tx, 0-value

3. **Solana** (Instruction coverage):
   - 30 fixtures (currently have 13)
   - All instruction types: transfers, token, NFT, DeFi

4. **Cosmos** (Message type coverage):
   - 40 fixtures (currently have 31)
   - All message types from SDK

5. **Other chains** (Representative):
   - 20 fixtures each for: Zcash, Cardano, Polkadot, etc.

**Execution**:
```bash
# Step 1: Use official test vectors
cargo run -p fixture-generator -- --source official-test-vectors

# Step 2: Use explorer APIs (with rate limiting)
ETHERSCAN_API_KEY=xxx cargo run -p fixture-generator -- --source explorers

# Step 3: Generate synthetic edge cases
cargo run -p fixture-generator -- --source synthetic --chain bitcoin

# Step 4: Validate all fixtures
cargo test --all --test '*fixture*'
```

### Phase 2: Automated Maintenance (3 weeks)

**Goal**: Keep fixtures fresh and representative

**Approach**:
```yaml
# .github/workflows/update_fixtures.yml
name: Update Test Fixtures

on:
  schedule:
    - cron: "0 0 * * 0"  # Weekly
  workflow_dispatch:

jobs:
  update-fixtures:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Generate new fixtures
        env:
          ETHERSCAN_API_KEY: ${{ secrets.ETHERSCAN_API_KEY }}
          ALCHEMY_API_KEY: ${{ secrets.ALCHEMY_API_KEY }}
        run: |
          cargo run -p fixture-generator -- \
            --source explorers \
            --all-chains \
            --save-to tests/fixtures
      
      - name: Validate fixtures
        run: cargo test --test '*fixture*'
      
      - name: Create PR if changed
        uses: peter-evans/create-pull-request@v5
        with:
          title: "chore: Update test fixtures"
          body: "Weekly update of test fixtures from explorers"
          branch: "update-fixtures-${{ github.run_number }}"
```

### Phase 3: Privacy-First Integration (2 weeks)

**Goal**: Ensure test data doesn't leak sensitive information

**Approach**:
1. Audit all existing fixtures for PII
2. Implement fixture sanitization
3. Document privacy policy
4. Add privacy tests

```rust
#[test]
fn test_fixtures_are_anonymized() {
    let fixtures = load_fixtures_dir("tests/fixtures");
    
    for fixture in fixtures {
        // Verify no obvious wallets/addresses used
        assert!(!fixture.description.contains("my wallet"));
        assert!(!fixture.description.contains("personal"));
        
        // Verify metadata doesn't contain PII
        if let Some(url) = &fixture.metadata.explorer_url {
            // Only public blockchain explorer URLs allowed
            assert!(is_public_explorer(url));
        }
    }
}
```

---

## Part 5: Integration with Current Testing Strategy

### Mapping to 5-Level Pyramid

```
LEVEL 1: Unit Tests
├─ Small, isolated functions
├─ No fixture dependencies
└─ Run in <1 second per test

LEVEL 2: Property Tests (proptest)
├─ Generate arbitrary valid transactions
├─ Verify properties hold for all cases
├─ Uses synthetic data (no real fixtures needed)
└─ Run in <5 seconds

LEVEL 3: Integration Tests ← NEW FIXTURE FOCUS
├─ Load real transaction fixtures
├─ Validate decoder output
├─ Compare against reference implementations
├─ 500+ test fixtures by Phase completion
└─ Run in <30 seconds total

LEVEL 4: Fuzz Testing (cargo-fuzz)
├─ Random byte sequences
├─ No fixtures needed (pure fuzzing)
├─ Runs continuously in CI
└─ Detects panics and crashes

LEVEL 5: Formal Verification (Verus)
├─ Mathematical proofs
├─ No fixtures needed
├─ Proves core properties
└─ Runs weekly (computationally intensive)
```

### Fixture Coverage Goals

```toml
# crates/universal-decoder-core/Cargo.toml

[package.metadata.test-coverage]
# Target: 100% of common transaction types tested

bitcoin = { types = 20, fixtures = 50, coverage = "95%" }
ethereum = { types = 15, fixtures = 50, coverage = "90%" }
solana = { types = 10, fixtures = 30, coverage = "85%" }
cosmos = { types = 12, fixtures = 40, coverage = "80%" }
```

---

## Part 6: Recommended Dataset Selection for Best ROI

### Tier 1: Official (Do These First - Highest Value)

| Dataset | Integration | ROI | Effort |
|---------|-------------|-----|--------|
| Bitcoin Core test vectors | Git subtree | ★★★★★ | 1 day |
| Ethereum JSON test suite | Git subtree | ★★★★★ | 1 day |
| BIP/EIP specifications | Manual parse | ★★★★☆ | 2 days |
| Cosmos SDK vectors | Git subtree | ★★★★☆ | 1 day |

### Tier 2: Vendor APIs (Do These Second)

| Dataset | Integration | ROI | Effort |
|---------|-------------|-----|--------|
| Etherscan (dev-only) | Fetch script | ★★★★☆ | 3 days |
| Blockchain.com (dev-only) | Fetch script | ★★★☆☆ | 2 days |
| Blockchair (dev-only) | Fetch script | ★★★☆☆ | 2 days |

### Tier 3: Analytics (Do These Third)

| Dataset | Integration | ROI | Effort |
|---------|-------------|-----|--------|
| BigQuery exports | ETL script | ★★★☆☆ | 1 week |
| Allium exports | Manual export | ★★☆☆☆ | 1 week |

### NOT Recommended (Don't Use)

- ❌ Runtime BigQuery access (violates airgapped)
- ❌ Runtime Allium API (violates airgapped)
- ❌ Explorer APIs in production (violates airgapped)
- ❌ Custom scraping (too much maintenance)

---

## Part 7: Specific Action Items

### Immediate (This Week)

- [ ] Audit existing 100 fixtures for quality/coverage
- [ ] Document fixture sources in metadata
- [ ] Create privacy audit for sensitive data

### Short-term (This Month)

- [ ] Build `fixture-generator` tool with official test vector support
- [ ] Integrate Bitcoin Core test vectors (tx_valid.json, tx_invalid.json)
- [ ] Integrate Ethereum JSON test suite
- [ ] Expand to 200+ fixtures

### Medium-term (Next Month)

- [ ] Add explorer API fetch support (with rate limiting)
- [ ] Implement privacy/anonymization
- [ ] Set up weekly fixture update CI job
- [ ] Reach 500+ fixtures

### Long-term (Next Quarter)

- [ ] BigQuery export workflow (if needed)
- [ ] Allium integration (if needed)
- [ ] Formal verification of fixture authenticity

---

## Conclusion

**Best approach for Universal Blockchain Decoder**:

1. **Primary**: Official test vectors + git subtree vendoring (Bitcoin Core, Ethereum, etc.)
   - ✅ Airgapped
   - ✅ Authoritative
   - ✅ Verifiable via git history
   - ✅ Already proven effective

2. **Secondary**: Explorer APIs for fixture generation (dev-tool only, not production)
   - ✅ Real-world transaction data
   - ✅ Community use cases
   - ⚠️ Rate limited, requires API keys
   - ✅ Committed to git after fetching

3. **Avoid**: Runtime network access
   - ❌ Violates airgapped requirement
   - ❌ Security/latency issues
   - ❌ Fails in offline deployments (banks/enterprise)

**Next Steps**:
1. Create `fixture-generator` tool (modeled after existing `registry-generator`)
2. Expand test fixtures from 100 → 500+
3. Document fixture sources and privacy considerations
4. Set up automated weekly fixture updates

This keeps the decoder lightweight, verifiable, and suitable for security-critical deployments.
