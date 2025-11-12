# Borsh Transformation Plan

## Goal

Transform vendored chain registries from JSON to compact Borsh binary format:
- **Current**: 14.5MB raw JSON (cosmos 7.4MB + superchain 7.1MB)
- **Target**: <2MB Borsh binaries (~1MB cosmos + ~200KB superchain)
- **Improvement**: ~85% size reduction

## Why This Matters

1. **Repository Size**: Reduce git clone size by 12.5MB
2. **Load Performance**: Borsh deserialization is faster than JSON parsing
3. **Consistency**: All registries use the same format (EVM already uses Borsh: 551KB)
4. **Airgapped Operation**: Compile-time embedded data, zero runtime I/O

## Implementation Strategy

### Phase 1: Refactor registry-generator Tool

**Create unified tool with subcommands** (following Cargo/Git pattern):

```bash
# Rename existing tool
mv tools/chain-registry-generator tools/registry-generator

# New usage:
cargo run -p registry-generator -- evm     # Already working
cargo run -p registry-generator -- cosmos  # New
cargo run -p registry-generator -- superchain  # New
```

### Phase 2: Cosmos Registry Types

**File**: `tools/registry-generator/src/cosmos.rs`

```rust
use borsh::{BorshSerialize, BorshDeserialize};
use serde::Deserialize;

/// Cosmos chain information (minimal subset)
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CosmosChainInfo {
    pub chain_name: String,
    pub chain_id: String,
    pub pretty_name: String,
    pub bech32_prefix: String,
    pub slip44: u32,
    pub network_type: String, // "mainnet" | "testnet"
}

/// Upstream chain.json schema (full)
#[derive(Debug, Deserialize)]
struct CosmosChainJson {
    chain_name: String,
    chain_id: String,
    pretty_name: Option<String>,
    bech32_prefix: String,
    slip44: u32,
    network_type: Option<String>,
    // ... other fields we don't need
}

impl From<CosmosChainJson> for CosmosChainInfo {
    fn from(json: CosmosChainJson) -> Self {
        CosmosChainInfo {
            chain_name: json.chain_name.clone(),
            chain_id: json.chain_id,
            pretty_name: json.pretty_name.unwrap_or(json.chain_name),
            bech32_prefix: json.bech32_prefix,
            slip44: json.slip44,
            network_type: json.network_type.unwrap_or_else(|| "mainnet".to_string()),
        }
    }
}
```

### Phase 3: Superchain Registry Types

**File**: `tools/registry-generator/src/superchain.rs`

```rust
use borsh::{BorshSerialize, BorshDeserialize};
use serde::Deserialize;

/// OP Stack chain information (minimal subset)
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct SuperchainInfo {
    pub name: String,
    pub chain_id: u64,
    pub rpc: Vec<String>,
    pub explorers: Vec<String>,
    pub superchain_level: u8,
    pub data_availability_type: String, // "eth-da" | "alt-da"
}

/// Upstream chainList.json schema (full)
#[derive(Debug, Deserialize)]
struct SuperchainJson {
    name: String,
    #[serde(rename = "chainId")]
    chain_id: u64,
    rpc: Vec<String>,
    explorers: Vec<String>,
    #[serde(rename = "superchainLevel")]
    superchain_level: u8,
    #[serde(rename = "dataAvailabilityType")]
    data_availability_type: String,
    // ... other fields we don't need
}
```

### Phase 4: Subcommand Implementation

**File**: `tools/registry-generator/src/main.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "registry-generator")]
#[command(about = "Generate compact Borsh-serialized chain registries")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate EVM chain registry (ethereum-lists/chains)
    Evm {
        #[arg(short, long, default_value = "crates/decoder-evm/vendored/chainlist/_data/chains")]
        input: PathBuf,
        #[arg(short, long, default_value = "crates/decoder-evm/data/chains.borsh")]
        output: PathBuf,
    },
    /// Generate Cosmos chain registry (cosmos/chain-registry)
    Cosmos {
        #[arg(short, long, default_value = "crates/decoder-cosmos/vendored/chain-registry")]
        input: PathBuf,
        #[arg(short, long, default_value = "crates/decoder-cosmos/data/cosmos-chains.borsh")]
        output: PathBuf,
    },
    /// Generate Superchain registry (ethereum-optimism/superchain-registry)
    Superchain {
        #[arg(short, long, default_value = "crates/decoder-optimism/vendored/superchain-registry/chainList.json")]
        input: PathBuf,
        #[arg(short, long, default_value = "crates/decoder-optimism/data/op-chains.borsh")]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Evm { input, output } => generate_evm_registry(input, output),
        Commands::Cosmos { input, output } => generate_cosmos_registry(input, output),
        Commands::Superchain { input, output } => generate_superchain_registry(input, output),
    }
}
```

### Phase 5: Cosmos Registry Generation

**Logic**:
1. Walk `crates/decoder-cosmos/vendored/chain-registry/`
2. For each directory (except `_*` and `testnets`):
   - Read `{dir}/chain.json`
   - Parse to `CosmosChainJson`
   - Convert to `CosmosChainInfo`
   - Add to HashMap
3. Serialize HashMap to Borsh
4. Write to `crates/decoder-cosmos/data/cosmos-chains.borsh`

**Expected Output**: ~1MB (406 chains * ~2.5KB each)

### Phase 6: Superchain Registry Generation

**Logic**:
1. Read `chainList.json`
2. Parse array of `SuperchainJson`
3. Convert to Vec<SuperchainInfo>
4. Serialize to Borsh
5. Write to `crates/decoder-optimism/data/op-chains.borsh`

**Expected Output**: ~200KB (35 chains * ~6KB each)

### Phase 7: Update Decoder Crates

**decoder-cosmos**:

```rust
// crates/decoder-cosmos/src/registry.rs

use borsh::BorshDeserialize;
use std::collections::HashMap;

pub struct CosmosRegistry {
    chains: HashMap<String, CosmosChainInfo>,
}

impl CosmosRegistry {
    pub fn new() -> Self {
        // Embed Borsh binary at compile time
        const CHAINS_BORSH: &[u8] = include_bytes!("../data/cosmos-chains.borsh");

        let chains: HashMap<String, CosmosChainInfo> =
            BorshDeserialize::try_from_slice(CHAINS_BORSH)
                .expect("Failed to deserialize cosmos chains");

        Self { chains }
    }

    pub fn get_chain(&self, chain_id: &str) -> Option<&CosmosChainInfo> {
        self.chains.get(chain_id)
    }
}
```

**decoder-optimism**:

```rust
// crates/decoder-optimism/src/registry.rs

use borsh::BorshDeserialize;

pub struct SuperchainRegistry {
    chains: Vec<SuperchainInfo>,
}

impl SuperchainRegistry {
    pub fn new() -> Self {
        // Embed Borsh binary at compile time
        const CHAINS_BORSH: &[u8] = include_bytes!("../data/op-chains.borsh");

        let chains: Vec<SuperchainInfo> =
            BorshDeserialize::try_from_slice(CHAINS_BORSH)
                .expect("Failed to deserialize OP Stack chains");

        Self { chains }
    }

    pub fn get_by_chain_id(&self, chain_id: u64) -> Option<&SuperchainInfo> {
        self.chains.iter().find(|c| c.chain_id == chain_id)
    }
}
```

### Phase 8: Cleanup

**Remove raw JSON files** (after Borsh generation succeeds):

```bash
# Cosmos: Keep only LICENSE + VENDORED.md + data/cosmos-chains.borsh
find crates/decoder-cosmos/vendored/chain-registry -type f -name "*.json" -delete
rm -rf crates/decoder-cosmos/vendored/chain-registry/*/  # All chain dirs

# Superchain: Keep only LICENSE + VENDORED.md + data/op-chains.borsh
rm -f crates/decoder-optimism/vendored/superchain-registry/chainList.json
rm -rf crates/decoder-optimism/vendored/superchain-registry/superchain/
```

**Final sizes**:
- `crates/decoder-cosmos/vendored/chain-registry/`: 7.4MB → ~50KB (LICENSE + docs)
- `crates/decoder-cosmos/data/cosmos-chains.borsh`: ~1MB
- `crates/decoder-optimism/vendored/superchain-registry/`: 7.1MB → ~50KB
- `crates/decoder-optimism/data/op-chains.borsh`: ~200KB

**Total**: 14.5MB → **1.3MB** (~91% reduction!)

## Testing Strategy

1. **Unit Tests**: Verify Borsh serialization roundtrip
2. **Integration Tests**: Load registries and verify chain count
3. **Validation Tests**: Compare with original JSON (before cleanup)
4. **Performance Tests**: Benchmark load time vs JSON

## Implementation Timeline

- **Phase 1-4**: Tool refactoring (2 hours)
- **Phase 5-6**: Registry generation (2 hours)
- **Phase 7**: Decoder integration (1 hour)
- **Phase 8**: Cleanup & testing (1 hour)

**Total**: ~6 hours

## Success Criteria

- ✅ All three registries use Borsh binary format
- ✅ Repository size reduced by >10MB
- ✅ All tests pass
- ✅ Decoder load time < 10ms (vs ~50ms for JSON)
- ✅ Zero runtime file I/O (compile-time embedding)

## Follow-up

After this is complete, consider:
1. **Compression**: Add zstd compression for additional 30-50% reduction
2. **Incremental Updates**: Script to update registries from upstream
3. **Build Verification**: CI check to ensure Borsh binaries are up-to-date

---

**Status**: Planned, not yet implemented
**Priority**: High (size optimization)
**Blocked by**: None (Phase 1.5.1 complete)
**Estimated effort**: 6 hours
