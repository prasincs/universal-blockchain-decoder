# Aleo Blockchain Decoder

Pure Rust decoder for Aleo blockchain transactions supporting zero-knowledge computations with Leo VM.

## Overview

Aleo is a privacy-focused blockchain using zkSNARKs (BLS12-377 curve) for decentralized private computations. This decoder provides full support for:

- **Transaction Types**:
  - Deploy: Program deployment to the blockchain
  - Execute: Program execution with transitions
  - Fee: Network fee payments

- **Privacy Features**:
  - Encrypted records (UTXO-like)
  - Zero-knowledge proofs (Varuna zkSNARKs)
  - Private inputs/outputs
  - Viewing keys for selective disclosure

- **Aleo-Specific Concepts**:
  - Transitions: Atomic state changes
  - Records: Encrypted state with commitments
  - Finalize operations: On-chain state updates
  - Leo VM program execution

## Architecture

```
decoder-aleo/
├── src/
│   ├── lib.rs              # Main decoder implementation
│   ├── types.rs            # Transaction type definitions
│   ├── parsing.rs          # Binary parsing logic
│   └── error.rs            # Error types
├── tests/
│   ├── mainnet_tests.rs    # Real mainnet transaction tests (8+ tests)
│   └── property_tests.rs    # Property-based tests (12+ properties, 1000+ cases)
└── README.md
```

## Features

### Implemented ✅

- ✅ Pure Rust implementation (no snarkVM dependency in production)
- ✅ Complete transaction parsing (Deploy, Execute, Fee)
- ✅ Transition parsing with inputs/outputs
- ✅ Record handling (serial numbers, commitments, ciphertexts)
- ✅ Finalize operation parsing (mapping operations)
- ✅ Privacy metadata extraction
- ✅ Canonical TxIR conversion
- ✅ Comprehensive mainnet tests (8 test scenarios)
- ✅ Property-based tests (12 properties, 1000+ test cases)
- ✅ Never-panic guarantee on arbitrary input

###Future Work ⏳

- ⏳ zkSNARK proof verification (uses BLS12-377)
- ⏳ Viewing key decryption
- ⏳ Full Leo VM semantics
- ⏳ Cross-validation with snarkVM (in dev-dependencies)

## Usage

```rust
use decoder_aleo::AleoDecoder;
use universal_decoder_core::prelude::*;

// Decode an Aleo transaction
let tx_bytes = /* ... transaction bytes ... */;
let decoded_tx = AleoDecoder::decode(&tx_bytes)?;

// Convert to canonical IR
let tx_ir = decoded_tx.canonicalize()?;

// Access transaction details
match &decoded_tx.transaction_type {
    TransactionType::Deploy(deploy) => {
        println!("Program: {}", deploy.program_id);
    }
    TransactionType::Execute(exec) => {
        println!("Transitions: {}", exec.transitions.len());
    }
    TransactionType::Fee(fee) => {
        println!("Fee: {} gates", fee.amount);
    }
}
```

## Test Coverage

### Mainnet Tests (8 scenarios)
1. ✅ Fee transaction decoding
2. ✅ Deployment transaction (with Leo program)
3. ✅ Execution transaction (public inputs/outputs)
4. ✅ Private execution (encrypted records)
5. ✅ Finalize operations (on-chain state)
6. ✅ Validation (rejects invalid transactions)
7. ✅ Transaction hash calculation
8. ✅ Complex multi-transition executions

### Property-Based Tests (12 properties, ~1000 cases each)
1. ✅ Decoder never panics on arbitrary input
2. ✅ Transaction ID is deterministic
3. ✅ Canonical serialization is deterministic
4. ✅ Transaction type detection is consistent
5. ✅ Privacy metadata is correctly set
6. ✅ State deltas generated for finalize operations
7. ✅ Fee amounts are validated
8. ✅ Record inputs have valid serial numbers (32 bytes)
9. ✅ Program IDs are non-empty in valid transactions
10. ✅ Full decode-canonicalize-hash pipeline never panics
11. ✅ Valid executions have at least one transition
12. ✅ Global state roots are always 32 bytes

**Total test cases**: 8 mainnet + 12,000+ property tests = **12,008+ test cases**

## Chain Information

- **Chain ID**: 368 (SLIP-44 registered)
- **Chain Family**: Privacy
- **Consensus**: Proof of Stake (PoSW - Proof of Succinct Work)
- **Curve**: BLS12-377 (zkSNARK-friendly)
- **Hash**: Poseidon (ZK-friendly)
- **VM**: Leo VM (domain-specific language for zero-knowledge programs)

## Dependencies

### Production
- `universal-decoder-core` - Core traits and types
- `decoder-primitives` - Common decoder utilities
- `decoder-chains-common` - Chain registry
- `decoder-crypto-zk` - ZK cryptography (BLS12-377 Poseidon)
- `serde`, `borsh` - Serialization
- `bs58` - Address encoding

### Development (test validation only)
- `proptest` - Property-based testing
- `hex` - Test fixtures
- Future: `snarkvm` for cross-validation

## Privacy Model

Aleo transactions support three observability levels:

1. **Fully Observable** (Public transactions)
   - All inputs/outputs are public
   - Function calls visible
   - No encryption

2. **Partially Observable** (Mixed transactions)
   - Some public, some private data
   - Selective disclosure via viewing keys

3. **Fully Private** (Private transactions)
   - Encrypted records
   - Hidden amounts, senders, recipients
   - Zero-knowledge proofs

## References

- [Aleo Official](https://aleo.org)
- [snarkVM Repository](https://github.com/ProvableHQ/snarkVM)
- [Leo Language](https://leo-lang.org)
- [Aleo VM Spec](https://developer.aleo.org/specs/aleovm.pdf)
- [BLS12-377 Curve](https://eprint.iacr.org/2018/962.pdf)

## License

MIT OR Apache-2.0
