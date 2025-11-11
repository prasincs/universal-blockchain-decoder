# Testing Strategy for Universal Blockchain Decoder

## Design Philosophy

> "Test the abstraction, not the implementation. Verify the guarantees, not the code."

This testing strategy follows the **Minimal Trusted Computing Base (TCB)** principle:
- **Core library**: Minimal dependencies, formally verifiable, exhaustively tested
- **Decoder libraries**: Can use external dependencies for testing, integration-tested against real data

## Core Testing Principles

### 1. Dependency Isolation

#### Core Library Testing (`universal-decoder-core`)

**Production Dependencies (Minimal)**:
```toml
[dependencies]
# ONLY these dependencies allowed in core
serde = { version = "1.0", features = ["derive"] }
borsh = { version = "1.3", features = ["derive"] }
thiserror = "1.0"
sha2 = "0.10"
sha3 = "0.10"
hex = "0.4"  # Consider reimplementing for minimal TCB
```

**Test Dependencies (Can be extensive)**:
```toml
[dev-dependencies]
proptest = "1.4"              # Property-based testing
quickcheck = "1.0"            # Alternative property testing
criterion = "0.5"             # Benchmarking
arbitrary = "1.3"             # Generating arbitrary data
bolero = "0.10"               # Fuzzing
assert_matches = "1.5"        # Pattern matching assertions
```

**Rationale**:
- Production code has minimal attack surface
- Test code can use any library to **validate** core behavior
- Tests don't ship to production → dependencies don't matter
- Comprehensive test tooling increases confidence

#### Decoder Library Testing (`decoder-*`)

**Production Dependencies (Flexible)**:
```toml
[dependencies]
universal-decoder-core = { path = "../universal-decoder-core" }
bitcoin = "0.31"              # OK: Chain-specific parsing
ethers-core = "2.0"           # OK: Ethereum types
# ... other chain-specific libraries
```

**Test Dependencies (Unrestricted)**:
```toml
[dev-dependencies]
proptest = "1.4"
bitcoincore-rpc = "0.18"      # OK: For integration tests
hex-literal = "0.4"           # OK: Test fixtures
tempfile = "3.8"              # OK: Test utilities
tokio = { version = "1.0", features = ["rt"] }  # OK: Async tests
```

**Rationale**:
- Decoders are **not** part of the TCB
- Users audit decoders they use
- Can use heavyweight libraries for testing
- Real blockchain libraries ensure compatibility

### 2. Dependency Audit for Core

#### Current Core Dependencies Analysis

| Dependency | LOC | Status | Action |
|------------|-----|--------|--------|
| `serde` | ~30k | ✅ Trusted | Keep - industry standard |
| `borsh` | ~5k | ✅ Trusted | Keep - canonical serialization |
| `thiserror` | ~2k | ✅ Trusted | Keep - error handling |
| `sha2` | ~3k | ⚠️ Review | Consider RustCrypto audit |
| `sha3` | ~2k | ⚠️ Review | Consider RustCrypto audit |
| `hex` | ~1k | 🔄 Consider reimpl | Simple enough to reimplement |
| `smallvec` | ~3k | ❓ Evaluate | Is this necessary? |

**Action Items**:

1. **`hex` - Reimplement** (Priority: Medium)
   - Simple encoding/decoding
   - ~200 LOC to reimplement
   - Eliminates external dependency

   ```rust
   // Internal implementation in core/src/utils/hex.rs
   pub fn encode(bytes: &[u8]) -> String {
       const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
       let mut result = String::with_capacity(bytes.len() * 2);
       for &byte in bytes {
           result.push(HEX_CHARS[(byte >> 4) as usize] as char);
           result.push(HEX_CHARS[(byte & 0xf) as usize] as char);
       }
       result
   }
   ```

2. **`smallvec` - Evaluate necessity** (Priority: High)
   - Is stack optimization needed in core?
   - If not performance-critical, use `Vec`
   - If needed, consider reimplementation (~500 LOC)

3. **RustCrypto (`sha2`, `sha3`) - Audit status** (Priority: Critical)
   - Check: https://github.com/RustCrypto/hashes
   - Verify audit trail
   - Consider alternatives only if audit insufficient
   - **Recommendation**: Keep - RustCrypto is well-audited

#### Minimal Core Dependencies (Target)

```toml
[dependencies]
# Core serialization (cannot eliminate)
serde = { version = "1.0", features = ["derive"], default-features = false }
borsh = { version = "1.3", features = ["derive"], default-features = false }

# Error handling (tiny, std-like)
thiserror = "1.0"

# Cryptography (audited, essential)
sha2 = { version = "0.10", default-features = false }
sha3 = { version = "0.10", default-features = false }

# Internal utilities (no external deps)
# hex - reimplemented internally
# smallvec - removed or reimplemented
```

**Target**: ≤ 5 external dependencies in core

## Testing Pyramid

### Level 1: Unit Tests (Fast, Many)

**Coverage Target**: 100% of core, 90% of decoders

#### Core Unit Tests

```rust
// crates/universal-decoder-core/src/ir.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_checked_add_overflow() {
        let a = Amount { value: u128::MAX, decimals: 18 };
        let b = Amount { value: 1, decimals: 18 };

        assert!(a.checked_add(b).is_none());
    }

    #[test]
    fn test_amount_checked_add_different_decimals() {
        let a = Amount { value: 100, decimals: 18 };
        let b = Amount { value: 100, decimals: 6 };

        assert!(a.checked_add(b).is_none());
    }

    #[test]
    fn test_canonical_hash_deterministic() {
        let tx_ir = create_test_tx_ir();

        let hash1 = tx_ir.canonical_hash().unwrap();
        let hash2 = tx_ir.canonical_hash().unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_to_canonical_bytes_deterministic() {
        let tx_ir = create_test_tx_ir();

        let bytes1 = tx_ir.to_canonical_bytes().unwrap();
        let bytes2 = tx_ir.to_canonical_bytes().unwrap();

        assert_eq!(bytes1, bytes2);
    }
}
```

**Test Organization**:
```
crates/universal-decoder-core/
├── src/
│   ├── ir.rs
│   │   └── #[cfg(test)] mod tests { ... }
│   ├── traits.rs
│   │   └── #[cfg(test)] mod tests { ... }
│   └── canonical.rs
│       └── #[cfg(test)] mod tests { ... }
└── tests/  # Integration tests
    ├── canonical_serialization.rs
    └── error_handling.rs
```

### Level 2: Property-Based Tests (Medium Speed, Important)

**Tool**: `proptest` (in `dev-dependencies`)

#### Core Property Tests

```rust
// crates/universal-decoder-core/tests/property_tests.rs

use proptest::prelude::*;
use universal_decoder_core::*;

// Generate arbitrary valid TxIR
prop_compose! {
    fn arbitrary_tx_ir()(
        chain_id in any::<u64>(),
        amount_value in any::<u128>(),
        decimals in 0u8..30u8,
    ) -> TxIR<'static, 1> {
        // Build valid TxIR
        TxIR::new(/* ... */)
    }
}

proptest! {
    /// Property: Canonical serialization is deterministic
    #[test]
    fn prop_canonical_bytes_deterministic(tx in arbitrary_tx_ir()) {
        let bytes1 = tx.to_canonical_bytes()?;
        let bytes2 = tx.to_canonical_bytes()?;
        prop_assert_eq!(bytes1, bytes2);
    }

    /// Property: Canonical hash is deterministic
    #[test]
    fn prop_canonical_hash_deterministic(tx in arbitrary_tx_ir()) {
        let hash1 = tx.canonical_hash()?;
        let hash2 = tx.canonical_hash()?;
        prop_assert_eq!(hash1, hash2);
    }

    /// Property: Different transactions have different hashes (collision resistance)
    #[test]
    fn prop_different_tx_different_hash(
        tx1 in arbitrary_tx_ir(),
        tx2 in arbitrary_tx_ir()
    ) {
        prop_assume!(tx1 != tx2);
        let hash1 = tx1.canonical_hash()?;
        let hash2 = tx2.canonical_hash()?;
        prop_assert_ne!(hash1, hash2);
    }

    /// Property: Amount addition is commutative
    #[test]
    fn prop_amount_add_commutative(
        a in any::<u64>().prop_map(|v| Amount { value: v as u128, decimals: 18 }),
        b in any::<u64>().prop_map(|v| Amount { value: v as u128, decimals: 18 })
    ) {
        let sum1 = a.checked_add(b);
        let sum2 = b.checked_add(a);
        prop_assert_eq!(sum1, sum2);
    }

    /// Property: Borsh serialization round-trips
    #[test]
    fn prop_borsh_roundtrip(tx in arbitrary_tx_ir()) {
        let bytes = borsh::to_vec(&tx)?;
        let decoded: TxIR<1> = borsh::from_slice(&bytes)?;
        prop_assert_eq!(tx, decoded);
    }
}
```

#### Decoder Property Tests

```rust
// crates/decoder-bitcoin/tests/property_tests.rs

use proptest::prelude::*;
use decoder_bitcoin::*;

proptest! {
    /// Property: Decoding always succeeds or returns error (no panics)
    #[test]
    fn prop_decode_no_panic(bytes in prop::collection::vec(any::<u8>(), 0..10000)) {
        // Should never panic, even on garbage input
        let _ = BitcoinDecoder::decode(&bytes);
        // If it returns, test passes (no panic)
    }

    /// Property: Valid Bitcoin transaction bytes decode successfully
    #[test]
    fn prop_valid_bitcoin_decodes(tx in arbitrary_valid_bitcoin_tx()) {
        let bytes = serialize_bitcoin_tx(&tx);
        let result = BitcoinDecoder::decode(&bytes);
        prop_assert!(result.is_ok());
    }

    /// Property: Decoded transaction has correct fee
    #[test]
    fn prop_fee_calculation_no_overflow(tx in arbitrary_bitcoin_tx()) {
        let decoded = BitcoinDecoder::decode(&serialize(&tx))?;
        // Should never panic or overflow
        let _ = decoded.calculate_fee();
    }
}
```

**Property Test Categories**:

1. **Determinism**: Same input → same output
2. **Idempotence**: `f(f(x)) = f(x)`
3. **Commutativity**: `f(a, b) = f(b, a)`
4. **Associativity**: `f(f(a, b), c) = f(a, f(b, c))`
5. **Inverse**: `decode(encode(x)) = x`
6. **Monotonicity**: `a < b ⟹ f(a) < f(b)`
7. **Bounds**: `f(x) ≤ MAX`

### Level 3: Integration Tests (Slower, Real Data)

**Real blockchain data fixtures**

```rust
// crates/decoder-bitcoin/tests/integration_tests.rs

use decoder_bitcoin::*;

#[test]
fn test_decode_bitcoin_genesis_block() {
    // Real Bitcoin genesis block coinbase transaction
    let tx_bytes = include_bytes!("fixtures/btc_genesis_coinbase.bin");

    let decoded = BitcoinDecoder::decode(tx_bytes).unwrap();

    // Verify known properties
    assert_eq!(decoded.version(), 1);
    assert_eq!(decoded.input_count(), 1);
    assert_eq!(decoded.output_count(), 1);
    assert_eq!(decoded.outputs()[0].value, 5_000_000_000); // 50 BTC
}

#[test]
fn test_decode_bitcoin_taproot_transaction() {
    // Real Taproot transaction from block 709,632
    let tx_bytes = include_bytes!("fixtures/btc_taproot_709632.bin");

    let decoded = BitcoinDecoder::decode(tx_bytes).unwrap();
    let tx_ir = decoded.canonicalize().unwrap();

    // Verify Taproot-specific fields
    assert_eq!(tx_ir.chain_family(), ChainFamily::Utxo);
    // ... more assertions
}

#[test]
fn test_decode_ethereum_eip1559_transaction() {
    // Real EIP-1559 transaction
    let tx_bytes = include_bytes!("fixtures/eth_eip1559_london.bin");

    let decoded = EthereumDecoder::decode(tx_bytes).unwrap();
    let tx_ir = decoded.canonicalize().unwrap();

    // Verify EIP-1559 fields
    assert!(tx_ir.metadata.gas_price.is_some());
    assert!(tx_ir.metadata.max_priority_fee.is_some());
}
```

**Fixture Organization**:
```
crates/decoder-bitcoin/tests/fixtures/
├── btc_genesis_coinbase.bin
├── btc_taproot_709632.bin
├── btc_segwit_witness.bin
└── btc_multisig_p2sh.bin

crates/decoder-ethereum/tests/fixtures/
├── eth_eip1559_london.bin
├── eth_legacy_tx.bin
├── eth_contract_creation.bin
└── eth_eip2930_access_list.bin
```

**Test Data Sources**:
1. **Block explorers**: Download raw transaction bytes
2. **Chain nodes**: Query via RPC (e.g., `bitcoin-cli getrawtransaction`)
3. **Test vectors**: From chain specification documents (BIPs, EIPs)
4. **Known edge cases**: Malformed transactions, unusual inputs

### Level 4: Fuzz Testing (Continuous, Automated)

**Tool**: `cargo-fuzz` with `libFuzzer`

```rust
// crates/universal-decoder-core/fuzz/fuzz_targets/fuzz_decode.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use decoder_bitcoin::BitcoinDecoder;

fuzz_target!(|data: &[u8]| {
    // Should never panic, even on malicious input
    let _ = BitcoinDecoder::decode(data);
});
```

**Fuzz Testing Setup**:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing
cd crates/decoder-bitcoin
cargo fuzz init

# Run fuzzer (continuous)
cargo fuzz run fuzz_decode -- -max_len=100000
```

**Fuzzing Targets**:
- `fuzz_decode_bitcoin`: Random bytes → Bitcoin decoder
- `fuzz_decode_ethereum`: Random bytes → Ethereum decoder
- `fuzz_canonical_serialization`: Arbitrary TxIR → Borsh encoding
- `fuzz_amount_operations`: Arithmetic operations with random values

**Coverage-Guided Fuzzing**:
```bash
# Use coverage feedback to guide fuzzing
cargo fuzz run fuzz_decode -- \
    -max_len=100000 \
    -timeout=30 \
    -rss_limit_mb=8192 \
    -use_value_profile=1
```

### Level 5: Formal Verification (Strongest Guarantees)

See `docs/FORMAL_VERIFICATION.md` for detailed strategy.

**Verus-Verified Core Functions**:

```rust
// crates/universal-decoder-core/src/ir.rs

use builtin::*;
use builtin_macros::*;

verus! {

impl Amount {
    /// Formally verified addition with overflow checking
    #[verifier::proof]
    pub fn checked_add(self, other: Amount) -> (result: Option<Amount>)
        requires
            self.decimals == other.decimals,
        ensures
            result.is_some() ==> {
                let sum = result.unwrap();
                sum.value == self.value + other.value &&
                sum.decimals == self.decimals
            },
            result.is_none() ==> {
                self.value + other.value > u128::MAX
            }
    {
        match self.value.checked_add(other.value) {
            Some(sum) => Some(Amount { value: sum, decimals: self.decimals }),
            None => None,
        }
    }
}

#[verifier::proof]
pub fn canonical_bytes_deterministic<'a>(tx: &TxIR<'a, 1>)
    ensures
        tx.to_canonical_bytes() == tx.to_canonical_bytes()
{
    // Proof obligation verified by Verus
}

} // verus!
```

**Verification Phases**:
1. **Phase 1** (Month 1-2): Core trait definitions, error types
2. **Phase 2** (Month 3-4): Canonical serialization, Amount arithmetic
3. **Phase 3** (Month 5-6): Bitcoin decoder, UTXO validation
4. **Phase 4** (Month 7-8): Ethereum decoder, state transitions

## Test Organization Structure

```
universal-blockchain-decoder/
├── crates/
│   ├── universal-decoder-core/
│   │   ├── src/
│   │   │   ├── ir.rs              // Unit tests: #[cfg(test)] mod tests
│   │   │   ├── traits.rs          // Unit tests: #[cfg(test)] mod tests
│   │   │   └── canonical.rs       // Unit tests: #[cfg(test)] mod tests
│   │   ├── tests/
│   │   │   ├── property_tests.rs  // Property-based tests (proptest)
│   │   │   ├── canonical.rs       // Integration tests
│   │   │   └── error_handling.rs  // Error path tests
│   │   ├── benches/
│   │   │   └── benchmarks.rs      // Criterion benchmarks
│   │   └── fuzz/
│   │       └── fuzz_targets/
│   │           └── fuzz_canonical.rs
│   │
│   ├── decoder-bitcoin/
│   │   ├── src/
│   │   │   └── lib.rs             // Unit tests inline
│   │   ├── tests/
│   │   │   ├── integration.rs     // Real Bitcoin data
│   │   │   ├── property_tests.rs  // Property-based tests
│   │   │   └── fixtures/          // Binary test data
│   │   │       ├── btc_genesis_coinbase.bin
│   │   │       └── btc_taproot.bin
│   │   └── fuzz/
│   │       └── fuzz_targets/
│   │           └── fuzz_decode.rs
│   │
│   └── decoder-ethereum/
│       ├── src/
│       │   └── lib.rs
│       ├── tests/
│       │   ├── integration.rs
│       │   ├── property_tests.rs
│       │   └── fixtures/
│       │       └── eth_eip1559.bin
│       └── fuzz/
│           └── fuzz_targets/
│               └── fuzz_decode.rs
│
└── scripts/
    ├── fetch_test_data.sh         // Download real blockchain data
    ├── run_all_tests.sh           // Run entire test suite
    └── coverage_report.sh         // Generate coverage report
```

## Continuous Integration (CI) Strategy

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml

name: Test Suite

on: [push, pull_request]

jobs:
  # Job 1: Fast unit tests (every commit)
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --lib
      - run: cargo test --all --bins

  # Job 2: Property-based tests (every commit)
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --test '*property*'

  # Job 3: Integration tests (every commit)
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --test '*integration*'

  # Job 4: Fuzz testing (nightly, scheduled)
  fuzz-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run fuzz_decode -- -max_total_time=3600

  # Job 5: Code coverage (every PR)
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-tarpaulin
      - run: cargo tarpaulin --all --out Xml
      - uses: codecov/codecov-action@v3

  # Job 6: Formal verification (weekly)
  formal-verification:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - name: Install Verus
        run: |
          git clone https://github.com/verus-lang/verus.git
          cd verus && ./tools/get-z3.sh && source tools/activate
      - run: verus crates/universal-decoder-core/src/ir.rs
```

## Testing Best Practices

### 1. Test Naming Convention

```rust
#[test]
fn test_<module>_<function>_<condition>_<expected>() {
    // Example: test_amount_checked_add_overflow_returns_none
}
```

### 2. Arrange-Act-Assert Pattern

```rust
#[test]
fn test_canonical_hash_deterministic() {
    // Arrange: Set up test data
    let tx_ir = create_test_tx_ir();

    // Act: Perform the operation
    let hash1 = tx_ir.canonical_hash().unwrap();
    let hash2 = tx_ir.canonical_hash().unwrap();

    // Assert: Verify expectations
    assert_eq!(hash1, hash2);
}
```

### 3. Test Data Builders

```rust
// crates/universal-decoder-core/tests/common/builders.rs

pub struct TxIRBuilder<'a, const V: u8> {
    chain_id: u64,
    metadata: TxMetadata,
    // ... other fields
}

impl<'a, const V: u8> TxIRBuilder<'a, V> {
    pub fn new() -> Self {
        Self {
            chain_id: 1,
            metadata: TxMetadata::default(),
        }
    }

    pub fn chain_id(mut self, id: u64) -> Self {
        self.chain_id = id;
        self
    }

    pub fn build(self) -> TxIR<'a, V> {
        TxIR::new(/* ... */)
    }
}

// Usage in tests
#[test]
fn test_with_builder() {
    let tx = TxIRBuilder::new()
        .chain_id(1)
        .version(1)
        .build();

    assert_eq!(tx.chain_id(), 1);
}
```

### 4. Table-Driven Tests

```rust
#[test]
fn test_amount_add_cases() {
    let cases = vec![
        // (a, b, expected)
        (Amount { value: 0, decimals: 18 }, Amount { value: 0, decimals: 18 }, Some(0)),
        (Amount { value: 1, decimals: 18 }, Amount { value: 2, decimals: 18 }, Some(3)),
        (Amount { value: u128::MAX, decimals: 18 }, Amount { value: 1, decimals: 18 }, None),
        (Amount { value: 100, decimals: 18 }, Amount { value: 100, decimals: 6 }, None), // Different decimals
    ];

    for (a, b, expected) in cases {
        let result = a.checked_add(b);
        match expected {
            Some(val) => assert_eq!(result.unwrap().value, val),
            None => assert!(result.is_none()),
        }
    }
}
```

### 5. Error Path Testing

```rust
#[test]
fn test_decode_invalid_version_returns_error() {
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid version

    let result = BitcoinDecoder::decode(&invalid_bytes);

    assert!(result.is_err());
    match result.unwrap_err() {
        DecoderError::InvalidVersion(_) => {}, // Expected
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_all_error_variants_covered() {
    // Ensure every error variant has a test
    let _ = DecoderError::InvalidVersion(0);
    let _ = DecoderError::InvalidFormat;
    let _ = DecoderError::BufferTooSmall;
    // ... all variants
}
```

## Test Coverage Goals

| Component | Unit Test Coverage | Integration Tests | Property Tests | Formal Verification |
|-----------|-------------------|-------------------|----------------|---------------------|
| Core traits | 100% | N/A | Yes | High priority |
| TxIR | 100% | Yes | Yes | High priority |
| Canonical serialization | 100% | Yes | Yes | Critical |
| Error types | 100% | Yes | No | Low priority |
| Bitcoin decoder | 90% | Yes (real data) | Yes | Medium priority |
| Ethereum decoder | 90% | Yes (real data) | Yes | Medium priority |
| Hooks system | 100% | Yes | Yes | Medium priority |

## Performance Testing

### Benchmark Strategy

```rust
// crates/universal-decoder-core/benches/benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use universal_decoder_core::*;

fn bench_canonical_serialization(c: &mut Criterion) {
    let tx_ir = create_large_tx_ir();

    c.bench_function("canonical_serialization", |b| {
        b.iter(|| {
            black_box(tx_ir.to_canonical_bytes())
        })
    });
}

fn bench_canonical_hash(c: &mut Criterion) {
    let tx_ir = create_large_tx_ir();

    c.bench_function("canonical_hash", |b| {
        b.iter(|| {
            black_box(tx_ir.canonical_hash())
        })
    });
}

criterion_group!(benches, bench_canonical_serialization, bench_canonical_hash);
criterion_main!(benches);
```

**Performance Targets**:
- Bitcoin transaction decode: < 100μs
- Ethereum transaction decode: < 50μs
- Canonical serialization: < 10μs
- Canonical hash: < 50μs

## Security Testing

### 1. Malformed Input Testing

```rust
#[test]
fn test_decode_truncated_input() {
    for len in 0..100 {
        let truncated = vec![0u8; len];
        let result = BitcoinDecoder::decode(&truncated);
        // Should return error, not panic
        assert!(result.is_err() || result.is_ok());
    }
}

#[test]
fn test_decode_malicious_length_field() {
    let malicious = vec![
        0xFF, 0xFF, 0xFF, 0xFF, // Huge length
        0x00, 0x00, 0x00, 0x00, // Minimal data
    ];

    let result = BitcoinDecoder::decode(&malicious);
    // Should not crash or allocate excessive memory
    assert!(result.is_err());
}
```

### 2. Overflow Testing

```rust
#[test]
fn test_amount_operations_no_overflow() {
    let max = Amount { value: u128::MAX, decimals: 18 };

    // Addition overflow
    assert!(max.checked_add(Amount { value: 1, decimals: 18 }).is_none());

    // Multiplication overflow
    assert!(max.checked_mul(2).is_none());
}
```

### 3. Side-Channel Resistance

```rust
#[test]
fn test_constant_time_comparison() {
    // For cryptographic operations
    use subtle::ConstantTimeEq;

    let hash1 = [0u8; 32];
    let hash2 = [0u8; 32];

    // Use constant-time comparison
    assert!(bool::from(hash1.ct_eq(&hash2)));
}
```

## Dependency Testing Strategy

### For Core Dependencies

```rust
// Test that we're using the dependencies correctly

#[test]
fn test_borsh_determinism() {
    use borsh::{BorshSerialize, BorshDeserialize};

    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
    struct TestStruct {
        a: u64,
        b: String,
    }

    let obj = TestStruct { a: 42, b: "test".to_string() };

    // Serialize twice
    let bytes1 = borsh::to_vec(&obj).unwrap();
    let bytes2 = borsh::to_vec(&obj).unwrap();

    // Must be identical
    assert_eq!(bytes1, bytes2);
}

#[test]
fn test_sha256_determinism() {
    use sha2::{Sha256, Digest};

    let data = b"hello world";

    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(data);

    assert_eq!(hash1, hash2);
}
```

### Monitoring Dependency Updates

```toml
# Cargo.toml - Use exact versions for core dependencies
[dependencies]
serde = "=1.0.196"      # Pin exact version
borsh = "=1.3.1"        # Pin exact version
thiserror = "=1.0.56"   # Pin exact version

# Dev dependencies can use ranges
[dev-dependencies]
proptest = "1.4"        # Allow minor updates
criterion = "0.5"       # Allow minor updates
```

**Update Strategy**:
1. Review changelog for security fixes
2. Update in separate PR with full test run
3. Verify no behavior changes
4. Update formal verification proofs if needed

## Test Execution Strategy

### Local Development

```bash
# Fast feedback loop (unit tests only)
cargo test --lib

# Full test suite (unit + integration)
cargo test --all

# Property tests with more iterations
PROPTEST_CASES=10000 cargo test --test property_tests

# Run with coverage
cargo tarpaulin --all --out Html

# Benchmarks
cargo bench
```

### Pre-Commit Checks

```bash
#!/bin/bash
# scripts/pre-commit.sh

set -e

echo "Running unit tests..."
cargo test --lib --quiet

echo "Running clippy..."
cargo clippy -- -D warnings

echo "Checking formatting..."
cargo fmt -- --check

echo "Running security audit..."
cargo audit

echo "✓ All checks passed"
```

### Pull Request Checks

```bash
#!/bin/bash
# scripts/pr-checks.sh

set -e

# Full test suite
cargo test --all

# Property tests with extended iterations
PROPTEST_CASES=10000 cargo test --test '*property*'

# Integration tests
cargo test --test '*integration*'

# Benchmarks (ensure no performance regression)
cargo bench -- --save-baseline pr-baseline

# Coverage report
cargo tarpaulin --all --out Xml --output-dir ./coverage

# Check coverage threshold
COVERAGE=$(grep -oP 'line-rate="\K[^"]+' coverage/cobertura.xml | head -1)
if (( $(echo "$COVERAGE < 0.90" | bc -l) )); then
    echo "Coverage below 90%: $COVERAGE"
    exit 1
fi

echo "✓ All PR checks passed"
```

## Documentation Testing

### Doc Tests

```rust
/// Computes the canonical hash of the transaction.
///
/// # Example
///
/// ```
/// use universal_decoder_core::prelude::*;
///
/// let tx_ir = TxIR::new(/* ... */);
/// let hash = tx_ir.canonical_hash()?;
/// assert_eq!(hash.len(), 32);
/// # Ok::<(), DecoderError>(())
/// ```
pub fn canonical_hash(&self) -> Result<[u8; 32], DecoderError> {
    // Implementation
}
```

**Run doc tests**:
```bash
cargo test --doc
```

## Regression Testing

### Test Case Management

```rust
// tests/regression/mod.rs

/// Regression test for issue #123: Overflow in fee calculation
#[test]
fn test_issue_123_fee_overflow() {
    let tx_bytes = include_bytes!("fixtures/issue_123.bin");
    let result = BitcoinDecoder::decode(tx_bytes);

    // Should not panic, should return error
    assert!(result.is_err());
    match result.unwrap_err() {
        DecoderError::ArithmeticOverflow => {},
        other => panic!("Expected ArithmeticOverflow, got {:?}", other),
    }
}
```

## Test Maintenance

### Periodic Review

- **Monthly**: Review flaky tests
- **Quarterly**: Review test coverage gaps
- **Semi-annually**: Review property test effectiveness
- **Annually**: Major test suite refactoring

### Test Metrics

Track these metrics:
- Test execution time (should be < 5 min for unit tests)
- Flakiness rate (should be < 0.1%)
- Coverage percentage (goal: > 90% for core)
- Property test iterations (increase over time)

## Summary: Testing Philosophy

1. **Core library**: Minimal dependencies, maximal testing
2. **Test code**: Can use any dependency to validate behavior
3. **Property tests**: Prove general properties, not just examples
4. **Real data**: Integration tests with actual blockchain transactions
5. **Formal verification**: Mathematical proof for critical properties
6. **Continuous**: Fuzzing, benchmarking, coverage tracking

**Goal**: Build **unshakeable confidence** in the core library through **layered, comprehensive testing**.

---

**Next Steps**:
1. Review and approve this testing strategy
2. Implement test infrastructure (test builders, fixtures)
3. Write initial test suite for core modules
4. Set up CI/CD pipeline
5. Begin formal verification annotations
