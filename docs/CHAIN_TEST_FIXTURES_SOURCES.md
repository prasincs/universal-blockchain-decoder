# Chain Test Fixtures - Official Sources

This document maps each supported blockchain to its official repository and test fixture locations.

**Status**: 🚧 Work in Progress
**Last Updated**: 2025-11-18
**Purpose**: Identify official test fixtures from each chain's repository for comprehensive integration testing

---

## Priority Tiers

### Tier 1: High-Priority Chains (Production-Ready Decoders)
These chains have complete decoder implementations and need comprehensive test fixtures.

| Chain | Status | Repository | Test Fixtures Location |
|-------|--------|------------|------------------------|
| **Bitcoin** | ✅ Has fixtures (123 vectors) | https://github.com/bitcoin/bitcoin | `src/test/data/tx_valid.json`, `src/test/data/tx_invalid.json` |
| **Ethereum** | ✅ Has fixtures (12 files) | https://github.com/ethereum/tests | `GeneralStateTests/`, `TransactionTests/` |
| **Solana** | 🔴 Needs fixtures | https://github.com/solana-labs/solana | `runtime/tests/`, `sdk/tests/` |
| **Cosmos SDK** | 🔴 Needs fixtures | https://github.com/cosmos/cosmos-sdk | `simapp/`, `x/*/testdata/` |

### Tier 2: Advanced Implementations (Specialized Decoders)
These chains have good implementations but need real transaction examples.

| Chain | Status | Repository | Test Fixtures Location |
|-------|--------|------------|------------------------|
| **Bittensor** | 🟡 Has synthetic | https://github.com/opentensor/subtensor | `pallets/*/tests/`, `node/tests/` |
| **Cardano** | 🔴 Needs fixtures | https://github.com/input-output-hk/cardano-node | `cardano-api/test/`, `cardano-ledger/*/test/` |
| **Aptos** | 🔴 Needs fixtures | https://github.com/aptos-labs/aptos-core | `aptos-move/aptos-vm/tests/`, `testsuite/` |
| **Polkadot** | 🔴 Needs fixtures | https://github.com/paritytech/polkadot | `runtime/*/src/tests/`, `node/test/` |
| **Starknet** | 🔴 Needs fixtures | https://github.com/starkware-libs/starknet | `crates/blockifier/tests/`, `crates/papyrus_storage/testdata/` |
| **Mina** | 🔴 Needs fixtures | https://github.com/MinaProtocol/mina | `src/lib/*/test/`, `frontend/wallet/test/` |
| **Algorand** | 🔴 Needs fixtures | https://github.com/algorand/go-algorand | `data/transactions/logic/testdata/`, `test/` |
| **Zcash** | 🔴 Needs fixtures | https://github.com/zcash/zcash | `src/gtest/test_*.cpp`, `qa/rpc-tests/` |

### Tier 3: Framework/Scaffolded Decoders
These have minimal implementations and need both code and fixtures.

| Chain | Status | Repository | Test Fixtures Location |
|-------|--------|------------|------------------------|
| **Optimism** | 🔴 Needs fixtures | https://github.com/ethereum-optimism/optimism | `op-node/testdata/`, `op-geth/core/types/testdata/` |
| **Arbitrum** | 🔴 Needs fixtures | https://github.com/OffchainLabs/nitro | `arbos/programs/testdata/`, `system_tests/` |
| **Avalanche** | 🔴 Needs fixtures | https://github.com/ava-labs/avalanchego | `vms/platformvm/txs/testdata/`, `vms/avm/txs/testdata/` |
| **Sui** | 🔴 Needs fixtures | https://github.com/MystenLabs/sui | `crates/sui-types/src/unit_tests/`, `crates/sui-core/src/unit_tests/` |
| **Tron** | 🔴 Needs fixtures | https://github.com/tronprotocol/java-tron | `framework/src/test/resources/`, `actuator/src/test/` |
| **XRP Ledger** | 🔴 Needs fixtures | https://github.com/XRPLF/rippled | `src/test/app/`, `src/test/jtx/` |
| **BNB Chain** | 🔴 Needs fixtures | https://github.com/bnb-chain/bsc | `core/types/testdata/`, `tests/` |
| **Polygon** | 🔴 Needs fixtures | https://github.com/maticnetwork/bor | Based on go-ethereum, use Ethereum fixtures |
| **Near** | 🔴 Needs fixtures | https://github.com/near/nearcore | `core/primitives/src/test_utils/`, `runtime/runtime/tests/` |
| **Stellar** | 🔴 Needs fixtures | https://github.com/stellar/stellar-core | `src/transactions/test/`, `src/testdata/` |

### Tier 4: UTXO Chains (Bitcoin Forks)
These can often reuse Bitcoin test infrastructure with minor modifications.

| Chain | Status | Repository | Test Fixtures Location |
|-------|--------|------------|------------------------|
| **Litecoin** | 🔴 Needs fixtures | https://github.com/litecoin-project/litecoin | Based on Bitcoin, similar structure |
| **Dogecoin** | 🔴 Needs fixtures | https://github.com/dogecoin/dogecoin | Based on Bitcoin, similar structure |
| **Bitcoin Cash** | 🔴 Needs fixtures | https://github.com/bitcoin-cash-node/bitcoin-cash-node | `src/test/data/` |
| **Bitcoin SV** | 🔴 Needs fixtures | https://github.com/bitcoin-sv/bitcoin-sv | `src/test/data/` |
| **Dash** | 🔴 Needs fixtures | https://github.com/dashpay/dash | `src/test/data/` |

---

## Repository Analysis Plan

For each chain, we need to:

### 1. **Locate Test Fixtures** 🔍
Search for:
- `testdata/` directories
- `fixtures/` directories
- `test_vectors/` files
- JSON/CBOR/binary test files in test directories
- Official test suites (e.g., Ethereum's ethereum/tests repo)

### 2. **Identify Fixture Formats** 📋
Document:
- File format (JSON, binary, hex-encoded, etc.)
- Structure (raw transactions, full blocks, test vectors)
- What fields are included
- Expected outputs (hashes, parsed results, etc.)

### 3. **Extraction Strategy** 📦
Decide:
- **Copy directly**: Small, essential fixtures
- **Git subtree**: Large test suites (like Ethereum tests)
- **Download script**: Generated or frequently updated fixtures
- **Synthetic**: Where official fixtures don't exist

### 4. **Integration Plan** 🔧
For each fixture:
- Create `crates/decoder-{chain}/tests/fixtures/` directory
- Add fixture files
- Write test cases that:
  - Load fixture
  - Decode transaction
  - Validate against expected output
  - Compare with reference implementation (in dev-dependencies)

---

## Specific Fixture Locations (Detailed)

### Bitcoin ✅ (Already Integrated)
**Repository**: https://github.com/bitcoin/bitcoin
**Fixtures Used**:
- `src/test/data/tx_valid.json` - Valid transaction test vectors
- `src/test/data/tx_invalid.json` - Invalid transaction test vectors
- `src/wallet/test/scriptpubkeyman_tests.cpp` - BIP341 (Taproot) vectors
**Status**: 123 vectors integrated ✅

### Ethereum ✅ (Already Integrated)
**Repository**: https://github.com/ethereum/tests
**Fixtures Used**:
- `TransactionTests/ttVValue/V_equals27.json`
- `TransactionTests/ttVValue/V_equals28.json`
- Custom fixtures for EIP-1559, EIP-2930
**Status**: 12 fixtures integrated ✅

### Solana 🔴
**Repository**: https://github.com/solana-labs/solana
**Potential Fixture Sources**:
1. `sdk/src/transaction/test.rs` - Transaction parsing tests
2. `runtime/src/bank/tests.rs` - Full transaction examples
3. `ledger/src/blockstore_processor.rs` - Historical transactions
4. `programs/*/tests/` - Program-specific transaction examples

**Action Items**:
- [ ] Clone Solana repo
- [ ] Extract transaction test vectors from `sdk/src/transaction/test.rs`
- [ ] Find real mainnet transactions (e.g., simple transfer, token transfer, stake)
- [ ] Convert to hex format for our fixtures

### Cosmos SDK 🔴
**Repository**: https://github.com/cosmos/cosmos-sdk
**Potential Fixture Sources**:
1. `simapp/` - Simulated app with example transactions
2. `x/bank/testdata/` - Bank module test data
3. `x/staking/testdata/` - Staking module test data
4. `x/gov/testdata/` - Governance module test data
5. `types/tx/testdata/` - Transaction type test data

**Action Items**:
- [ ] Clone Cosmos SDK repo
- [ ] Extract Protobuf-encoded transaction examples
- [ ] Get real chain data from chains like Cosmos Hub, Osmosis
- [ ] Document message type coverage

### Polkadot 🔴
**Repository**: https://github.com/paritytech/polkadot
**Potential Fixture Sources**:
1. `runtime/polkadot/src/tests.rs` - Runtime test transactions
2. `node/test/` - Node integration tests
3. `rpc/src/transaction/tests.rs` - Transaction RPC tests
4. Substrate repo: https://github.com/paritytech/substrate
   - `frame/*/src/tests.rs` - Pallet tests with extrinsics

**Action Items**:
- [ ] Extract SCALE-encoded extrinsic examples
- [ ] Get signed vs unsigned extrinsics
- [ ] Document era, nonce, tip structure

### Aptos 🔴
**Repository**: https://github.com/aptos-labs/aptos-core
**Potential Fixture Sources**:
1. `aptos-move/aptos-vm/tests/` - VM test transactions
2. `testsuite/smoke-test/` - Integration test transactions
3. `crates/aptos-rosetta/src/types/fixtures/` - Rosetta fixtures
4. `execution/executor-test-helpers/src/` - Transaction helpers

**Action Items**:
- [ ] Extract BCS-encoded transaction examples
- [ ] Get multi-sig transaction examples
- [ ] Document transaction types (script, module, entry function)

### Starknet 🔴
**Repository**: https://github.com/starkware-libs/cairo (Cairo VM)
**Alternative**: https://github.com/starknet-io/starknet-addresses (Network data)
**Potential Fixture Sources**:
1. `crates/blockifier/tests/` - Transaction execution tests
2. `crates/papyrus_storage/testdata/` - Storage test data
3. Starknet.js tests: https://github.com/starknet-io/starknet.js
4. Real transactions from Starknet explorers (Voyager, Starkscan)

**Action Items**:
- [ ] Get examples of all 6 transaction types
- [ ] Extract STARK field arithmetic test vectors
- [ ] Document signature verification

### Cardano 🔴
**Repository**: https://github.com/input-output-hk/cardano-node
**Potential Fixture Sources**:
1. `cardano-api/test/Test/Cardano/Api/Typed/` - Typed API tests
2. `cardano-ledger/eras/shelley/test-suite/` - Shelley era tests
3. `cardano-ledger/eras/alonzo/test-suite/` - Alonzo (Plutus) tests
4. `cardano-node/cardano-cli/test/` - CLI test data

**Action Items**:
- [ ] Extract CBOR-encoded transaction examples
- [ ] Get multi-asset transactions
- [ ] Get Plutus script transactions
- [ ] Document different eras (Byron, Shelley, Alonzo, Babbage)

### Mina Protocol 🔴
**Repository**: https://github.com/MinaProtocol/mina
**Potential Fixture Sources**:
1. `src/lib/mina_base/test/` - Base layer tests
2. `src/lib/transaction_snark/test/` - Transaction SNARK tests
3. `frontend/wallet/test/` - Wallet tests
4. Archive node data (real transactions)

**Action Items**:
- [ ] Extract transaction test vectors
- [ ] Get zkSNARK proof examples
- [ ] Document Pallas curve operations

### OP Stack (Optimism, Arbitrum) 🔴
**Optimism Repository**: https://github.com/ethereum-optimism/optimism
**Potential Fixture Sources**:
1. `op-node/testdata/` - Node test data
2. `op-geth/core/types/testdata/` - Type test data
3. `packages/contracts-bedrock/test/` - Contract tests
4. Deposit transaction examples

**Arbitrum Repository**: https://github.com/OffchainLabs/nitro
**Potential Fixture Sources**:
1. `arbos/programs/testdata/` - ArbOS test data
2. `system_tests/` - System tests
3. Retryable ticket examples

**Action Items**:
- [ ] Extract L1→L2 deposit transactions
- [ ] Get L2→L1 withdrawal transactions
- [ ] Document OP Stack-specific fields

### Avalanche 🔴
**Repository**: https://github.com/ava-labs/avalanchego
**Potential Fixture Sources**:
1. `vms/platformvm/txs/testdata/` - Platform chain transactions
2. `vms/avm/txs/testdata/` - X-chain transactions
3. `vms/evm/core/types/` - C-chain (EVM) transactions

**Action Items**:
- [ ] Get examples from all 3 chains (X, P, C)
- [ ] Extract UTXO-based transactions (X/P chains)
- [ ] Get subnet transaction examples

---

## Fixture Organization Structure

For each chain, create the following structure:

```
crates/decoder-{chain}/tests/fixtures/
├── README.md                    # Source attribution, license info
├── simple/                      # Basic transactions
│   ├── transfer.hex            # Raw transaction bytes
│   ├── transfer.json           # Expected decoded output
│   └── metadata.json           # Block height, tx hash, etc.
├── complex/                     # Advanced features
│   ├── multi_sig.hex
│   ├── contract_call.hex
│   └── ...
├── edge_cases/                  # Boundary conditions
│   ├── empty_payload.hex
│   ├── max_size.hex
│   └── ...
└── invalid/                     # Should fail to decode
    ├── bad_signature.hex
    ├── invalid_format.hex
    └── ...
```

---

## Test Template

For each fixture, create a test following this pattern:

```rust
#[test]
fn test_fixture_{chain}_{type}() {
    // Load fixture
    let tx_bytes = include_bytes!("fixtures/simple/transfer.hex");
    let expected: ExpectedOutput = serde_json::from_str(
        include_str!("fixtures/simple/transfer.json")
    ).unwrap();

    // Decode
    let decoder = {Chain}Decoder::new();
    let tx_ir = decoder.decode(tx_bytes).expect("Should decode successfully");

    // Validate
    assert_eq!(tx_ir.version, expected.version);
    assert_eq!(tx_ir.operations.len(), expected.operations_count);
    assert_eq!(tx_ir.canonical_hash().unwrap(), expected.tx_hash);

    // Cross-validate with reference implementation (in dev-dependencies)
    #[cfg(test)]
    {
        let reference_tx = {chain_lib}::Transaction::decode(tx_bytes).unwrap();
        assert_eq!(tx_ir.hash(), reference_tx.hash());
    }
}
```

---

## Automation Strategy

### Phase 1: Manual Collection (This Session)
1. Identify top 10 priority chains
2. Clone repositories
3. Find and extract 3-5 fixtures per chain
4. Write basic integration tests

### Phase 2: Scripted Extraction (Future)
Create `tools/fetch_fixtures.sh`:
```bash
#!/bin/bash
# Fetch official test fixtures from chain repositories

CHAINS=("bitcoin" "ethereum" "solana" "cosmos" ...)

for chain in "${CHAINS[@]}"; do
    echo "Fetching fixtures for $chain..."
    # Clone repo to temp dir
    # Extract fixtures
    # Convert to standard format
    # Copy to crates/decoder-$chain/tests/fixtures/
done
```

### Phase 3: CI/CD Integration (Future)
- Monthly job to check for new fixtures in upstream repos
- Automated PR if new test vectors found
- Regression testing against reference implementations

---

## Next Steps

1. **Immediate** (This Session):
   - [ ] Start with Tier 1 chains (Solana, Cosmos)
   - [ ] Clone repositories
   - [ ] Extract 3-5 real transaction examples per chain
   - [ ] Write integration tests

2. **Short-term** (This Week):
   - [ ] Complete Tier 2 chains (Polkadot, Aptos, Cardano)
   - [ ] Document all fixture sources
   - [ ] Add to CI/CD pipeline

3. **Medium-term** (This Month):
   - [ ] Complete all 35+ chains
   - [ ] Reach 100+ fixture target
   - [ ] Automated fixture fetching

---

**Last Updated**: 2025-11-18
**Contributors**: Claude Code Agent
**Status**: 🚧 Work in Progress
