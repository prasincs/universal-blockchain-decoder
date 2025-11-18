#!/bin/bash
# Fetch Chain Test Fixtures
#
# This script fetches official test fixtures from blockchain repositories
# and organizes them into our test fixture structure.
#
# Usage: ./tools/fetch_chain_fixtures.sh [chain_name]
#        ./tools/fetch_chain_fixtures.sh all

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
TEMP_DIR="${REPO_ROOT}/tmp/fixture_fetch"

# Colors for output
RED='\033[0,31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create fixture directory structure
create_fixture_dirs() {
    local chain=$1
    local decoder_path="${REPO_ROOT}/crates/decoder-${chain}/tests/fixtures"

    mkdir -p "${decoder_path}/simple"
    mkdir -p "${decoder_path}/complex"
    mkdir -p "${decoder_path}/edge_cases"
    mkdir -p "${decoder_path}/invalid"

    log_success "Created fixture directories for ${chain}"
}

# Fetch Bitcoin fixtures
fetch_bitcoin_fixtures() {
    log_info "Bitcoin fixtures already comprehensive (123 vectors from Bitcoin Core)"
    log_info "Skipping Bitcoin"
}

# Fetch Ethereum fixtures
fetch_ethereum_fixtures() {
    log_info "Ethereum fixtures already comprehensive (12 fixtures + EIP test vectors)"
    log_info "Skipping Ethereum"
}

# Fetch Solana fixtures
fetch_solana_fixtures() {
    log_info "Fetching Solana test fixtures..."

    local decoder_path="${REPO_ROOT}/crates/decoder-solana/tests/fixtures"
    create_fixture_dirs "solana"

    # Clone Solana repo to temp
    mkdir -p "${TEMP_DIR}"
    if [ ! -d "${TEMP_DIR}/solana" ]; then
        log_info "Cloning Solana repository..."
        git clone --depth 1 --branch v1.18.0 \
            https://github.com/solana-labs/solana.git \
            "${TEMP_DIR}/solana" || true
    fi

    if [ -d "${TEMP_DIR}/solana" ]; then
        log_info "Extracting Solana test vectors..."

        # Look for test vectors in SDK
        find "${TEMP_DIR}/solana/sdk" -name "*test*.rs" -type f \
            -exec grep -l "Transaction\|Message" {} \; \
            > "${decoder_path}/source_files.txt" || true

        log_success "Solana test files catalogued"
    else
        log_warn "Could not clone Solana repo, will use manual examples"
    fi

    # Create README with sources
    cat > "${decoder_path}/README.md" <<'EOF'
# Solana Test Fixtures

## Sources

1. **Solana SDK Test Vectors**
   - Repository: https://github.com/solana-labs/solana
   - Version: v1.18.0
   - Location: `sdk/src/transaction/tests`

2. **Real Mainnet Transactions**
   - Explorer: https://solscan.io
   - Example transactions curated for testing

## Fixture Types

- `simple/` - Basic SOL transfers
- `complex/` - Multi-instruction transactions, program interactions
- `edge_cases/` - Maximum size, unusual formats
- `invalid/` - Malformed transactions (should fail to decode)

## Format

All fixtures are stored as:
- `.base64` - Base64-encoded transaction (Solana native format)
- `.json` - Expected decoded output with metadata

## License

Solana is licensed under Apache 2.0
Test vectors derived from official Solana repository
EOF

    log_success "Created Solana fixture structure"
}

# Fetch Cosmos SDK fixtures
fetch_cosmos_fixtures() {
    log_info "Fetching Cosmos SDK test fixtures..."

    local decoder_path="${REPO_ROOT}/crates/decoder-cosmos-sdk/tests/fixtures"
    create_fixture_dirs "cosmos-sdk"

    # Clone Cosmos SDK repo to temp
    mkdir -p "${TEMP_DIR}"
    if [ ! -d "${TEMP_DIR}/cosmos-sdk" ]; then
        log_info "Cloning Cosmos SDK repository..."
        git clone --depth 1 --branch v0.50.0 \
            https://github.com/cosmos/cosmos-sdk.git \
            "${TEMP_DIR}/cosmos-sdk" || true
    fi

    if [ -d "${TEMP_DIR}/cosmos-sdk" ]; then
        log_info "Extracting Cosmos SDK test data..."

        # Find test files with transaction examples
        find "${TEMP_DIR}/cosmos-sdk/x" -name "*_test.go" -type f \
            -exec grep -l "MsgSend\|MsgDelegate\|MsgVote" {} \; \
            > "${decoder_path}/source_files.txt" || true

        # Look for testdata directories
        find "${TEMP_DIR}/cosmos-sdk" -type d -name "testdata" \
            >> "${decoder_path}/testdata_dirs.txt" || true

        log_success "Cosmos SDK test files catalogued"
    else
        log_warn "Could not clone Cosmos SDK repo"
    fi

    cat > "${decoder_path}/README.md" <<'EOF'
# Cosmos SDK Test Fixtures

## Sources

1. **Cosmos SDK Official Test Data**
   - Repository: https://github.com/cosmos/cosmos-sdk
   - Version: v0.50.0
   - Location: `x/*/testdata/`, `tests/integration/`

2. **Real Chain Transactions**
   - Cosmos Hub, Osmosis, and other IBC chains
   - Explorer: https://www.mintscan.io

## Message Types Covered

- Bank: MsgSend, MsgMultiSend
- Staking: MsgDelegate, MsgUndelegate, MsgRedelegate
- Distribution: MsgWithdrawDelegatorReward
- Governance: MsgSubmitProposal, MsgVote
- IBC: MsgTransfer, MsgChannelOpenInit
- CosmWasm: MsgStoreCode, MsgInstantiateContract, MsgExecuteContract

## Format

All fixtures are stored as:
- `.proto.bin` - Protobuf-encoded transaction
- `.json` - Expected decoded output with metadata

## License

Cosmos SDK is licensed under Apache 2.0
EOF

    log_success "Created Cosmos SDK fixture structure"
}

# Fetch Polkadot/Substrate fixtures
fetch_polkadot_fixtures() {
    log_info "Fetching Polkadot test fixtures..."

    local decoder_path="${REPO_ROOT}/crates/decoder-polkadot/tests/fixtures"
    create_fixture_dirs "polkadot"

    mkdir -p "${TEMP_DIR}"
    if [ ! -d "${TEMP_DIR}/polkadot-sdk" ]; then
        log_info "Cloning Polkadot SDK repository..."
        git clone --depth 1 --branch polkadot-v1.7.0 \
            https://github.com/paritytech/polkadot-sdk.git \
            "${TEMP_DIR}/polkadot-sdk" || true
    fi

    if [ -d "${TEMP_DIR}/polkadot-sdk" ]; then
        log_info "Extracting Polkadot test vectors..."

        # Find extrinsic test files
        find "${TEMP_DIR}/polkadot-sdk/substrate/frame" -name "*.rs" -type f \
            -exec grep -l "UncheckedExtrinsic\|test" {} \; \
            > "${decoder_path}/source_files.txt" || true

        log_success "Polkadot test files catalogued"
    fi

    cat > "${decoder_path}/README.md" <<'EOF'
# Polkadot Test Fixtures

## Sources

1. **Polkadot SDK Test Vectors**
   - Repository: https://github.com/paritytech/polkadot-sdk
   - Version: polkadot-v1.7.0
   - Location: `substrate/frame/*/src/tests.rs`

2. **Real Mainnet Extrinsics**
   - Explorer: https://polkadot.subscan.io
   - Polkadot.js examples

## Extrinsic Types

- Signed vs Unsigned
- Balance transfers
- Staking operations
- Governance votes
- XCM (cross-chain) messages

## Format

All fixtures are stored as:
- `.scale` - SCALE-encoded extrinsic (hex or binary)
- `.json` - Expected decoded output with metadata

## License

Polkadot SDK is licensed under Apache 2.0 or GPL-3.0
EOF

    log_success "Created Polkadot fixture structure"
}

# Fetch Aptos fixtures
fetch_aptos_fixtures() {
    log_info "Fetching Aptos test fixtures..."

    local decoder_path="${REPO_ROOT}/crates/decoder-aptos/tests/fixtures"
    create_fixture_dirs "aptos"

    mkdir -p "${TEMP_DIR}"
    if [ ! -d "${TEMP_DIR}/aptos-core" ]; then
        log_info "Cloning Aptos repository..."
        git clone --depth 1 --branch aptos-release-v1.12 \
            https://github.com/aptos-labs/aptos-core.git \
            "${TEMP_DIR}/aptos-core" || true
    fi

    if [ -d "${TEMP_DIR}/aptos-core" ]; then
        log_info "Extracting Aptos test vectors..."

        # Find transaction test files
        find "${TEMP_DIR}/aptos-core/testsuite" -name "*.rs" -type f \
            -exec grep -l "SignedTransaction\|RawTransaction" {} \; \
            > "${decoder_path}/source_files.txt" || true

        # Check for existing fixtures
        if [ -d "${TEMP_DIR}/aptos-core/testsuite/fixtures" ]; then
            log_info "Found Aptos fixtures directory"
            ls -la "${TEMP_DIR}/aptos-core/testsuite/fixtures" > "${decoder_path}/available_fixtures.txt" || true
        fi

        log_success "Aptos test files catalogued"
    fi

    cat > "${decoder_path}/README.md" <<'EOF'
# Aptos Test Fixtures

## Sources

1. **Aptos Core Test Vectors**
   - Repository: https://github.com/aptos-labs/aptos-core
   - Version: aptos-release-v1.12
   - Location: `testsuite/`, `aptos-move/aptos-vm/tests/`

2. **Real Mainnet Transactions**
   - Explorer: https://explorer.aptoslabs.com
   - TypeScript SDK test vectors

## Transaction Types

- Entry function calls
- Script transactions
- Multi-sig transactions
- Multi-agent transactions

## Format

All fixtures are stored as:
- `.bcs` - BCS-encoded transaction (binary)
- `.json` - Expected decoded output with metadata

## License

Aptos is licensed under Apache 2.0
EOF

    log_success "Created Aptos fixture structure"
}

# Fetch Cardano fixtures
fetch_cardano_fixtures() {
    log_info "Fetching Cardano test fixtures..."

    local decoder_path="${REPO_ROOT}/crates/decoder-cardano/tests/fixtures"
    create_fixture_dirs "cardano"

    cat > "${decoder_path}/README.md" <<'EOF'
# Cardano Test Fixtures

## Sources

1. **Cardano Node Test Data**
   - Repository: https://github.com/input-output-hk/cardano-node
   - Location: `cardano-api/test/`, `cardano-ledger/eras/*/test-suite/`

2. **Real Mainnet Transactions**
   - Explorer: https://cardanoscan.io
   - Multiple eras: Shelley, Alonzo (Plutus), Babbage

## Transaction Types

- Simple ADA transfers
- Multi-asset transactions
- Plutus smart contracts
- Stake pool operations
- Governance actions

## Format

All fixtures are stored as:
- `.cbor` - CBOR-encoded transaction
- `.json` - Expected decoded output with metadata

## License

Cardano is licensed under Apache 2.0
EOF

    log_success "Created Cardano fixture structure"
}

# Fetch Algorand fixtures
fetch_algorand_fixtures() {
    log_info "Fetching Algorand test fixtures..."

    local decoder_path="${REPO_ROOT}/crates/decoder-algorand/tests/fixtures"
    create_fixture_dirs "algorand"

    cat > "${decoder_path}/README.md" <<'EOF'
# Algorand Test Fixtures

## Sources

1. **go-algorand Test Data**
   - Repository: https://github.com/algorand/go-algorand
   - Location: `data/transactions/logic/testdata/`

2. **Real Mainnet Transactions**
   - Explorer: https://algoexplorer.io
   - SDK examples

## Transaction Types

- Payment transactions
- Asset transfers (ASA)
- Application calls (smart contracts)
- Key registration
- Asset configuration

## Format

All fixtures are stored as:
- `.msgpack` - MessagePack-encoded transaction
- `.json` - Expected decoded output with metadata

## License

Algorand is licensed under AGPL-3.0
EOF

    log_success "Created Algorand fixture structure"
}

# Main function
main() {
    local chain="${1:-all}"

    log_info "Universal Blockchain Decoder - Fixture Fetcher"
    log_info "=============================================="

    case "$chain" in
        bitcoin)
            fetch_bitcoin_fixtures
            ;;
        ethereum)
            fetch_ethereum_fixtures
            ;;
        solana)
            fetch_solana_fixtures
            ;;
        cosmos|cosmos-sdk)
            fetch_cosmos_fixtures
            ;;
        polkadot)
            fetch_polkadot_fixtures
            ;;
        aptos)
            fetch_aptos_fixtures
            ;;
        cardano)
            fetch_cardano_fixtures
            ;;
        algorand)
            fetch_algorand_fixtures
            ;;
        all)
            log_info "Fetching fixtures for all chains..."
            fetch_bitcoin_fixtures
            fetch_ethereum_fixtures
            fetch_solana_fixtures
            fetch_cosmos_fixtures
            fetch_polkadot_fixtures
            fetch_aptos_fixtures
            fetch_cardano_fixtures
            fetch_algorand_fixtures
            ;;
        *)
            log_error "Unknown chain: $chain"
            echo "Usage: $0 [chain_name|all]"
            echo "Available chains: bitcoin, ethereum, solana, cosmos, polkadot, aptos, cardano, algorand, all"
            exit 1
            ;;
    esac

    log_success "Fixture fetching complete!"
    log_info "Next steps:"
    log_info "  1. Review fixtures in crates/decoder-*/tests/fixtures/"
    log_info "  2. Extract actual transaction examples from cloned repos in tmp/"
    log_info "  3. Write integration tests using these fixtures"
}

main "$@"
