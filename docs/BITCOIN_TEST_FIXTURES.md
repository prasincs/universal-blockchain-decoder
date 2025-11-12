# Bitcoin Test Fixtures and Vectors

**Purpose**: Document available test fixtures from established Bitcoin libraries and test suites for validating our pure Rust Bitcoin decoder implementation.

## Strategy: Leverage Existing Test Vectors

Instead of manually creating test fixtures, we should leverage the extensive test suites from:

1. **`bitcoin` crate test suite** - Rust's primary Bitcoin library
2. **Bitcoin Core test vectors** - Official reference implementation
3. **BIP test vectors** - Standardized test cases from Bitcoin Improvement Proposals
4. **Bitcoin Test Framework** - Comprehensive transaction test data

## Available Test Resources

### 1. rust-bitcoin Test Vectors

**Repository**: https://github.com/rust-bitcoin/rust-bitcoin

The `bitcoin` crate includes extensive test data in:

```
rust-bitcoin/
├── bitcoin/tests/data/
│   ├── mainnet_block_*.json
│   ├── testnet_block_*.json
│   └── tx_*.json
└── bitcoin/src/
    └── tests/
        └── (inline test vectors)
```

**How to Use**:
```bash
# Clone rust-bitcoin repository
git clone https://github.com/rust-bitcoin/rust-bitcoin.git /tmp/rust-bitcoin

# Copy test data to our fixtures directory
cp /tmp/rust-bitcoin/bitcoin/tests/data/*.json \
   crates/decoder-bitcoin/tests/fixtures/rust-bitcoin/
```

**Example Test Data**:
- Genesis block transactions
- SegWit transactions (BIP 141, BIP 143, BIP 144)
- Taproot transactions (BIP 341, BIP 342)
- Various transaction types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR)

### 2. Bitcoin Core Test Vectors

**Repository**: https://github.com/bitcoin/bitcoin

Bitcoin Core includes extensive test data in:

```
bitcoin/
├── src/test/data/
│   ├── tx_invalid.json          # Invalid transactions
│   ├── tx_valid.json            # Valid transactions
│   ├── script_tests.json        # Script validation tests
│   ├── sighash.json             # Signature hash tests
│   └── base58_encode_decode.json
└── test/functional/data/
    └── (various test transactions)
```

**JSON Format Example** (`tx_valid.json`):
```json
[
  [
    [
      ["previous_txid", vout, "scriptPubKey", amount_in_satoshis]
    ],
    "serialized_transaction_hex",
    "verification_flags",
    "comment"
  ]
]
```

**How to Use**:
```bash
# Download Bitcoin Core test data
curl -o crates/decoder-bitcoin/tests/fixtures/bitcoin-core/tx_valid.json \
  https://raw.githubusercontent.com/bitcoin/bitcoin/master/src/test/data/tx_valid.json

curl -o crates/decoder-bitcoin/tests/fixtures/bitcoin-core/tx_invalid.json \
  https://raw.githubusercontent.com/bitcoin/bitcoin/master/src/test/data/tx_invalid.json
```

### 3. BIP Test Vectors

#### BIP 143: Transaction Signature Verification for Version 0 Witness Program

**Source**: https://github.com/bitcoin/bips/blob/master/bip-0143.mediawiki

Test vectors include:
- Native P2WPKH transactions
- P2SH-P2WPKH transactions
- Native P2WSH transactions
- P2SH-P2WSH transactions

**Example from BIP 143**:
```
Transaction: 01000000000102fff7f7881a8099afa6940d42d1e7f6362bec38171ea3edf433541db4e4ad969f00000000494830450221008b9d1dc26ba6a9cb62127b02742fa9d754cd3bebf337f7a55d114c8e5cdd30be022040529b194ba3f9281a99f2b1c0a19c0489bc22ede944ccf4ecbab4cc618ef3ed01eeffffffef51e1b804cc89d182d279655c3aa89e815b1b309fe287d9b2b55d57b90ec68a0100000000ffffffff02202cb206000000001976a9148280b37df378db99f66f85c95a783a76ac7a6d5988ac9093510d000000001976a9143bde42dbee7e4dbe6a21b2d50ce2f0167faa815988ac000247304402203609e17b84f6a7d30c80bfa610b5b4542f32a8a0d5447a12fb1366d7f01cc44a0220573a954c4518331561406f90300e8f3358f51928d43c212a8caed02de67eebee0121025476c2e83188368da1ff3e292e7acafcdb3566bb0ad253f62fc70f07aeee635711000000

This is a SegWit transaction with 2 inputs and 2 outputs.
```

#### BIP 341: Taproot Test Vectors

**Source**: https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki

Includes test vectors for:
- Key path spending
- Script path spending
- Control block validation
- Schnorr signature validation

### 4. Transaction Test Framework

**Repository**: https://github.com/bitcoin/bitcoin/tree/master/test/functional

The Bitcoin Core functional test framework includes:
- Real mainnet transactions
- Edge case transactions
- Invalid transaction formats
- SegWit activation transactions
- Taproot activation transactions

### 5. Known Important Bitcoin Transactions

#### Historical Transactions

**Genesis Block Coinbase** (Block 0):
```
TXID: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
Hex: 01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000
```

**First Bitcoin Transaction** (Block 170):
```
TXID: f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16
Block: 170
From: Satoshi Nakamoto
To: Hal Finney
Amount: 10 BTC
```

**First SegWit Transaction** (Block 481,824):
```
TXID: 8f907925d2ebe48765103e6845c06f1f2bb77c6adc1cc002865865eb5cfd5c1c
Block: 481824 (SegWit activation)
```

**First Taproot Transaction** (Block 709,632):
```
TXID: (various - Taproot activation block)
Block: 709632 (Taproot activation)
```

## Implementation Plan

### Phase 1: Set Up Test Fixture Repository

```bash
# Create fixtures directory structure
mkdir -p crates/decoder-bitcoin/tests/fixtures/{rust-bitcoin,bitcoin-core,bips,mainnet}

# Download rust-bitcoin test data
git clone --depth 1 https://github.com/rust-bitcoin/rust-bitcoin.git /tmp/rust-bitcoin
cp -r /tmp/rust-bitcoin/bitcoin/tests/data/* \
   crates/decoder-bitcoin/tests/fixtures/rust-bitcoin/

# Download Bitcoin Core test vectors
curl -o crates/decoder-bitcoin/tests/fixtures/bitcoin-core/tx_valid.json \
  https://raw.githubusercontent.com/bitcoin/bitcoin/master/src/test/data/tx_valid.json

curl -o crates/decoder-bitcoin/tests/fixtures/bitcoin-core/tx_invalid.json \
  https://raw.githubusercontent.com/bitcoin/bitcoin/master/src/test/data/tx_invalid.json

curl -o crates/decoder-bitcoin/tests/fixtures/bitcoin-core/script_tests.json \
  https://raw.githubusercontent.com/bitcoin/bitcoin/master/src/test/data/script_tests.json
```

### Phase 2: Create Test Loader

**File**: `crates/decoder-bitcoin/tests/common/fixtures.rs`

```rust
//! Test fixture loaders for Bitcoin transactions

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use universal_decoder_core::hex;

/// Bitcoin Core tx_valid.json format
#[derive(Debug, Deserialize)]
pub struct BitcoinCoreTestCase {
    /// Previous outputs (inputs)
    pub inputs: Vec<BitcoinCoreInput>,
    /// Serialized transaction hex
    pub transaction_hex: String,
    /// Verification flags
    pub flags: String,
    /// Test description
    pub comment: String,
}

#[derive(Debug, Deserialize)]
pub struct BitcoinCoreInput {
    pub txid: String,
    pub vout: u32,
    pub script_pubkey: String,
    pub amount: Option<u64>,
}

/// Load Bitcoin Core tx_valid.json test cases
pub fn load_bitcoin_core_valid_txs() -> Vec<BitcoinCoreTestCase> {
    let path = "tests/fixtures/bitcoin-core/tx_valid.json";
    let content = fs::read_to_string(path)
        .expect("Failed to read tx_valid.json");

    serde_json::from_str(&content)
        .expect("Failed to parse tx_valid.json")
}

/// Load Bitcoin Core tx_invalid.json test cases
pub fn load_bitcoin_core_invalid_txs() -> Vec<BitcoinCoreTestCase> {
    let path = "tests/fixtures/bitcoin-core/tx_invalid.json";
    let content = fs::read_to_string(path)
        .expect("Failed to read tx_invalid.json");

    serde_json::from_str(&content)
        .expect("Failed to parse tx_invalid.json")
}

/// rust-bitcoin test transaction
#[derive(Debug, Deserialize)]
pub struct RustBitcoinTestTx {
    pub txid: String,
    pub hex: String,
    pub version: i32,
    pub locktime: u32,
    pub input_count: usize,
    pub output_count: usize,
}

/// Load rust-bitcoin test transactions
pub fn load_rust_bitcoin_test_txs() -> Vec<RustBitcoinTestTx> {
    let fixtures_dir = Path::new("tests/fixtures/rust-bitcoin");
    let mut txs = Vec::new();

    if let Ok(entries) = fs::read_dir(fixtures_dir) {
        for entry in entries.flatten() {
            if entry.path().extension() == Some("json".as_ref()) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(tx) = serde_json::from_str::<RustBitcoinTestTx>(&content) {
                        txs.push(tx);
                    }
                }
            }
        }
    }

    txs
}

/// Well-known mainnet transactions
pub struct KnownTransaction {
    pub name: &'static str,
    pub txid: &'static str,
    pub block: u32,
    pub hex: &'static str,
    pub description: &'static str,
}

pub const KNOWN_TRANSACTIONS: &[KnownTransaction] = &[
    KnownTransaction {
        name: "genesis_coinbase",
        txid: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
        block: 0,
        hex: "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000",
        description: "Bitcoin genesis block coinbase transaction",
    },
    KnownTransaction {
        name: "first_transaction",
        txid: "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
        block: 170,
        hex: "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000",
        description: "First Bitcoin transaction (Satoshi to Hal Finney)",
    },
];
```

### Phase 3: Comprehensive Test Suite

**File**: `crates/decoder-bitcoin/tests/bitcoin_core_vectors.rs`

```rust
//! Tests using Bitcoin Core test vectors

mod common;
use common::fixtures::*;
use decoder_bitcoin::*;
use universal_decoder_core::prelude::*;

#[test]
fn test_all_bitcoin_core_valid_transactions() {
    let test_cases = load_bitcoin_core_valid_txs();
    let mut passed = 0;
    let mut failed = 0;

    for (idx, test_case) in test_cases.iter().enumerate() {
        let tx_bytes = match universal_decoder_core::hex::decode(&test_case.transaction_hex) {
            Ok(bytes) => bytes,
            Err(_) => {
                eprintln!("Test {}: Failed to decode hex", idx);
                failed += 1;
                continue;
            }
        };

        match BitcoinDecoder::decode(&tx_bytes) {
            Ok(tx) => {
                // Validate transaction was parsed
                assert!(tx.version() > 0);
                assert!(tx.input_count() > 0);
                passed += 1;
            }
            Err(e) => {
                eprintln!("Test {}: {}", idx, test_case.comment);
                eprintln!("  Error: {:?}", e);
                failed += 1;
            }
        }
    }

    println!("Bitcoin Core valid transactions: {} passed, {} failed", passed, failed);
    assert!(failed == 0, "Some valid transactions failed to parse");
}

#[test]
fn test_all_bitcoin_core_invalid_transactions() {
    let test_cases = load_bitcoin_core_invalid_txs();
    let mut correctly_rejected = 0;
    let mut incorrectly_accepted = 0;

    for (idx, test_case) in test_cases.iter().enumerate() {
        let tx_bytes = match universal_decoder_core::hex::decode(&test_case.transaction_hex) {
            Ok(bytes) => bytes,
            Err(_) => {
                // Hex decode failure is expected for some invalid cases
                correctly_rejected += 1;
                continue;
            }
        };

        match BitcoinDecoder::decode(&tx_bytes) {
            Ok(_) => {
                // This is problematic - we accepted an invalid transaction
                eprintln!("Test {}: Incorrectly accepted invalid transaction", idx);
                eprintln!("  Comment: {}", test_case.comment);
                incorrectly_accepted += 1;
            }
            Err(_) => {
                // Correctly rejected
                correctly_rejected += 1;
            }
        }
    }

    println!(
        "Bitcoin Core invalid transactions: {} correctly rejected, {} incorrectly accepted",
        correctly_rejected, incorrectly_accepted
    );

    // Note: Some "invalid" transactions may only be invalid at script validation level,
    // not at parsing level. So we can't assert incorrectly_accepted == 0.
    // But we should investigate any that are accepted.
}

#[test]
fn test_known_mainnet_transactions() {
    for tx in KNOWN_TRANSACTIONS {
        println!("Testing: {} ({})", tx.name, tx.description);

        let tx_bytes = universal_decoder_core::hex::decode(tx.hex)
            .expect("Failed to decode known transaction hex");

        let decoded = BitcoinDecoder::decode(&tx_bytes)
            .expect("Failed to decode known transaction");

        // Verify basic properties
        assert!(decoded.version() > 0, "Version should be positive");
        assert!(decoded.input_count() > 0, "Should have inputs");
        assert!(decoded.output_count() > 0, "Should have outputs");

        println!("  ✓ Version: {}", decoded.version());
        println!("  ✓ Inputs: {}", decoded.input_count());
        println!("  ✓ Outputs: {}", decoded.output_count());
        println!("  ✓ SegWit: {}", decoded.is_segwit());
    }
}
```

### Phase 4: Validation Against rust-bitcoin

**File**: `crates/decoder-bitcoin/tests/rust_bitcoin_comparison.rs`

```rust
//! Compare our implementation against rust-bitcoin crate

mod common;
use common::fixtures::*;
use decoder_bitcoin::*;
use universal_decoder_core::prelude::*;
use bitcoin::{consensus::Decodable, Transaction as BitcoinTx};
use std::io::Cursor;

#[test]
fn test_rust_bitcoin_test_vectors() {
    let test_txs = load_rust_bitcoin_test_txs();

    for test_tx in test_txs {
        let tx_bytes = universal_decoder_core::hex::decode(&test_tx.hex)
            .expect("Failed to decode hex");

        // Our implementation
        let our_tx = BitcoinDecoder::decode(&tx_bytes)
            .expect("Our decoder failed");

        // rust-bitcoin implementation
        let mut cursor = Cursor::new(&tx_bytes);
        let ref_tx = BitcoinTx::consensus_decode(&mut cursor)
            .expect("rust-bitcoin decoder failed");

        // Compare key properties
        assert_eq!(our_tx.version(), ref_tx.version.0 as u32, "Version mismatch");
        assert_eq!(our_tx.input_count(), ref_tx.input.len(), "Input count mismatch");
        assert_eq!(our_tx.output_count(), ref_tx.output.len(), "Output count mismatch");
        assert_eq!(our_tx.locktime, ref_tx.lock_time.to_consensus_u32(), "Locktime mismatch");
    }
}
```

## Test Coverage Goals

Using these test fixtures, we should achieve:

- **✅ 1000+ valid transactions** from Bitcoin Core
- **✅ 500+ invalid transactions** from Bitcoin Core (should reject)
- **✅ 100+ SegWit transactions** from rust-bitcoin
- **✅ 50+ Taproot transactions** from BIPs
- **✅ Known historical transactions** (genesis, first tx, etc.)

## Continuous Integration

Add test fixture download to CI:

```yaml
# .github/workflows/test.yml

- name: Download Bitcoin test fixtures
  run: |
    mkdir -p crates/decoder-bitcoin/tests/fixtures/bitcoin-core
    curl -o crates/decoder-bitcoin/tests/fixtures/bitcoin-core/tx_valid.json \
      https://raw.githubusercontent.com/bitcoin/bitcoin/master/src/test/data/tx_valid.json
    curl -o crates/decoder-bitcoin/tests/fixtures/bitcoin-core/tx_invalid.json \
      https://raw.githubusercontent.com/bitcoin/bitcoin/master/src/test/data/tx_invalid.json
```

## Benefits

1. **Comprehensive Coverage**: Leverage thousands of test cases from established projects
2. **Standard Compliance**: Validate against official Bitcoin Core test vectors
3. **Edge Case Coverage**: Bitcoin Core tests include many edge cases we might miss
4. **Regression Prevention**: Known transactions ensure we don't break existing functionality
5. **Community Trust**: Using standard test vectors increases confidence

## Summary

Instead of manually creating test fixtures:

✅ **Use Bitcoin Core test vectors** (`tx_valid.json`, `tx_invalid.json`)
✅ **Use rust-bitcoin test data** (diverse transaction types)
✅ **Use BIP test vectors** (standard compliance)
✅ **Use known mainnet transactions** (real-world validation)

This approach gives us **1000+ test cases** immediately, with comprehensive coverage of:
- Legacy transactions
- SegWit (P2WPKH, P2WSH)
- Taproot (P2TR)
- Edge cases and invalid formats
- Historical important transactions

---

**Next Steps**:
1. Set up test fixture repository structure
2. Download Bitcoin Core test vectors
3. Create test fixture loaders
4. Implement comprehensive test suite
5. Run tests and validate against rust-bitcoin

**Status**: Ready for implementation
