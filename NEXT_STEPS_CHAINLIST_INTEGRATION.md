# Next Steps: Chainlist.org Integration

**Status**: Planning Phase
**Priority**: High
**Estimated Timeline**: 2-3 weeks
**Assigned To**: TBD

---

## Problem Statement

Currently, we're creating a separate decoder crate for each EVM-compatible chain:

**Current Approach**:
- ✅ 5 EVM chains implemented (BNB, Polygon, Avalanche, Optimism, Arbitrum)
- Each is ~100 LOC of nearly identical code
- Workspace has 17 decoder crates (and growing)
- To support all EVM chains: 500+ crates needed

**Scalability Issue**:
- EVM ecosystem has 500+ chains (and growing daily)
- Each new chain requires: new crate, CI updates, docs, tests
- Workspace becomes unwieldy
- Maintenance burden scales linearly

---

## Proposed Solution: Generic EVM Decoder

### High-Level Architecture

```
Current (5 chains):
  decoder-bnb (100 LOC) ─┐
  decoder-polygon (100)  ├─> All nearly identical
  decoder-avalanche (100)│   Just different chain IDs
  decoder-optimism (100) │
  decoder-arbitrum (100) ┘

Proposed (500+ chains):
  decoder-evm (500 LOC) ──> Supports ALL EVM chains
    ├─ ChainRegistry (from chainlist.org)
    └─ Special case detection for non-standard chains
```

### Core Components

#### 1. Generic EVM Decoder (`decoder-evm`)

**Location**: `crates/decoder-evm/`

**Purpose**: Single decoder supporting all standard EVM-compatible chains

**Features**:
- Reuses `decoder-ethereum` for RLP parsing
- Validates against chain registry
- Returns chain metadata alongside decoded transaction
- Detects and handles special-case chains

**API**:
```rust
pub struct EvmDecoder {
    registry: ChainRegistry,
}

impl EvmDecoder {
    /// Create from chainlist.org data
    pub fn new() -> Result<Self>;

    /// Decode transaction for any EVM chain
    pub fn decode(
        &self,
        raw_bytes: &[u8],
        expected_chain_id: Option<u64>
    ) -> Result<(EthereumTransaction, ChainInfo)>;

    /// Get supported chains
    pub fn list_chains(&self) -> Vec<&ChainInfo>;

    /// Check if chain ID is supported
    pub fn is_supported(&self, chain_id: u64) -> bool;
}
```

#### 2. Chain Registry (`ChainRegistry`)

**Data Source**: https://chainid.network/chains.json

**Update Strategy**:
- Embedded at compile time (via `include_str!`)
- Optional runtime updates from URL
- Version pinning for reproducible builds

**Schema**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    pub short_name: String,
    pub network_id: u64,
    pub native_currency: CurrencyInfo,
    pub rpc: Vec<String>,
    pub explorers: Vec<ExplorerInfo>,
    pub is_testnet: bool,

    // Custom fields for special handling
    #[serde(default)]
    pub has_custom_tx_types: bool,
    #[serde(default)]
    pub decoder_override: Option<String>, // e.g., "optimism", "arbitrum"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerInfo {
    pub name: String,
    pub url: String,
    pub standard: String, // "EIP3091", "none", etc.
}

pub struct ChainRegistry {
    chains: HashMap<u64, ChainInfo>,
    special_cases: HashMap<u64, Box<dyn ChainDecoder>>,
}
```

#### 3. Special Case Handling

**Chains with Custom Transaction Types**:

| Chain | Chain ID | Custom Features | Decoder |
|-------|----------|-----------------|---------|
| Optimism | 10 | Deposit transactions (0x7E) | `decoder-optimism` |
| Arbitrum | 42161 | Retryable tickets, ArbOS internals | `decoder-arbitrum` |
| zkSync Era | 324 | Custom tx types, account abstraction | `decoder-zksync` (future) |
| Scroll | 534352 | Custom batch compression | `decoder-scroll` (future) |

**Detection Strategy**:
```rust
impl EvmDecoder {
    fn detect_special_case(&self, chain_id: u64, raw_bytes: &[u8]) -> Option<&str> {
        // Check registry for known special cases
        if let Some(chain) = self.registry.get_chain(chain_id) {
            if let Some(decoder) = &chain.decoder_override {
                return Some(decoder);
            }
        }

        // Auto-detect based on transaction type byte
        match raw_bytes.get(0) {
            Some(0x7E) => Some("optimism"), // Deposit transaction
            // Add more detection logic as needed
            _ => None
        }
    }
}
```

---

## Implementation Plan

### Phase 1: Create Generic EVM Decoder (Week 1)

**Tasks**:

1. **Create `decoder-evm` crate**
   ```bash
   cargo new --lib crates/decoder-evm
   ```

2. **Implement ChainRegistry**
   - Download chains.json from chainlist.org
   - Embed at compile time
   - Add parsing logic
   - Add chain lookup methods

3. **Implement EvmDecoder**
   - Delegate to `EthereumDecoder::decode()`
   - Validate chain ID against registry
   - Return `(EthereumTransaction, ChainInfo)` tuple

4. **Write tests**
   - Test with known chain IDs (1, 56, 137, etc.)
   - Test with unknown chain ID (should error)
   - Test chain listing
   - Test metadata lookup

**Deliverables**:
- [ ] `crates/decoder-evm/` created
- [ ] `chains.json` embedded
- [ ] Basic decoder working
- [ ] 20+ unit tests
- [ ] Documentation complete

---

### Phase 2: Migrate Existing EVM Decoders (Week 2)

**Strategy**: Keep existing decoders as thin wrappers

**Example Migration** (`decoder-bnb`):
```rust
// Before (current):
pub struct BnbDecoder;

impl ChainDecoder for BnbDecoder {
    type TxSpecific = EthereumTransaction;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        let tx = EthereumDecoder::decode(raw_bytes)?;

        if let Some(chain_id) = tx.chain_id {
            if chain_id != 56 && chain_id != 97 {
                return Err(/* ... */);
            }
        }

        Ok(tx)
    }
}

// After (wrapper):
pub struct BnbDecoder {
    evm: EvmDecoder,
}

impl BnbDecoder {
    pub fn new() -> Result<Self> {
        Ok(Self { evm: EvmDecoder::new()? })
    }
}

impl ChainDecoder for BnbDecoder {
    type TxSpecific = EthereumTransaction;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        let (tx, chain_info) = Self::new()?.evm.decode(raw_bytes, Some(56))?;

        // Optional: Add BNB-specific validation

        Ok(tx)
    }
}
```

**Tasks**:
1. Update `decoder-bnb` to use `decoder-evm`
2. Update `decoder-polygon` to use `decoder-evm`
3. Update `decoder-avalanche` to use `decoder-evm`
4. Update `decoder-optimism` (keep custom deposit tx handling)
5. Update `decoder-arbitrum` (keep custom retryable handling)

**Deliverables**:
- [ ] All 5 EVM decoders migrated
- [ ] All tests still passing
- [ ] No behavioral changes
- [ ] Reduced code duplication

---

### Phase 3: Special Case Handling (Week 3)

**Tasks**:

1. **Optimism Deposit Transactions**
   ```rust
   // decoder-optimism/src/lib.rs

   pub enum OptimismTransaction {
       Standard(EthereumTransaction),
       Deposit(DepositTransaction),
   }

   impl ChainDecoder for OptimismDecoder {
       type TxSpecific = OptimismTransaction;

       fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
           if raw_bytes[0] == 0x7E {
               Ok(OptimismTransaction::Deposit(
                   decode_deposit_transaction(raw_bytes)?
               ))
           } else {
               let (tx, _) = EvmDecoder::new()?.decode(raw_bytes, Some(10))?;
               Ok(OptimismTransaction::Standard(tx))
           }
       }
   }

   #[derive(Debug, Clone)]
   pub struct DepositTransaction {
       pub source_hash: [u8; 32],
       pub from: [u8; 20],
       pub to: Option<[u8; 20]>,
       pub mint: u128,
       pub value: u128,
       pub gas: u64,
       pub is_system_tx: bool,
       pub data: Vec<u8>,
   }
   ```

2. **Arbitrum Retryable Tickets**
   - Research Arbitrum custom transaction types
   - Implement retryable ticket parsing
   - Add validation logic

3. **Update EvmDecoder for auto-detection**
   - Add `detect_special_case()` method
   - Register special decoders
   - Route to appropriate decoder

**Deliverables**:
- [ ] Optimism deposit transactions supported
- [ ] Arbitrum retryables supported
- [ ] Auto-detection working
- [ ] Integration tests with real transactions

---

## Benefits Analysis

### Code Reduction

**Before**:
- 100 EVM chains = 100 crates × 100 LOC = 10,000 LOC
- Workspace members: 100+
- CI time: ~10 minutes (checking all crates)

**After**:
- 100 EVM chains = 1 crate × 500 LOC = 500 LOC
- Special cases: ~5 crates × 200 LOC = 1,000 LOC
- **Total**: 1,500 LOC (85% reduction)
- Workspace members: 6
- CI time: ~2 minutes

### Maintenance Reduction

**New EVM Chain**:
- Before: Create crate, write decoder, add tests, update CI, update docs (2-4 hours)
- After: Update chains.json (5 minutes)

**Ethereum Decoder Update**:
- Before: Manually update 100 crates
- After: Automatic propagation to all chains

### User Experience

**Discovery**:
```rust
// List all supported chains
let evm = EvmDecoder::new()?;
for chain in evm.list_chains() {
    if !chain.is_testnet {
        println!("{}: {}", chain.chain_id, chain.name);
    }
}
// Output: 1: Ethereum, 56: BNB Chain, 137: Polygon, ...
```

**Flexible Chain ID**:
```rust
// Decode without knowing chain ahead of time
let (tx, chain_info) = evm.decode(&tx_bytes, None)?;
println!("Decoded {} transaction on {}",
    chain_info.native_currency.symbol,
    chain_info.name
);
```

---

## Migration Strategy

### Backward Compatibility

**Option 1: Keep existing crates as aliases**
```rust
// decoder-bnb/src/lib.rs
pub use decoder_evm::{EvmDecoder, ChainInfo};

pub type BnbDecoder = EvmDecoder;

pub fn decode_bnb_transaction(raw_bytes: &[u8]) -> Result<EthereumTransaction> {
    let (tx, _) = EvmDecoder::new()?.decode(raw_bytes, Some(56))?;
    Ok(tx)
}
```

**Option 2: Deprecate and guide migration**
```rust
#[deprecated(
    since = "0.2.0",
    note = "Use decoder_evm::EvmDecoder instead. \
            Example: EvmDecoder::new()?.decode(bytes, Some(56))"
)]
pub struct BnbDecoder;
```

### Version Plan

**v0.2.0** (Breaking changes allowed):
- Add `decoder-evm` crate
- Migrate existing EVM decoders to use it
- Keep old APIs for compatibility

**v0.3.0**:
- Remove deprecated chain-specific decoders
- `decoder-evm` is the standard way

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_decode_ethereum_mainnet() {
    let evm = EvmDecoder::new().unwrap();
    let tx_bytes = include_bytes!("../tests/fixtures/eth_mainnet_tx.bin");

    let (tx, chain) = evm.decode(tx_bytes, Some(1)).unwrap();

    assert_eq!(chain.chain_id, 1);
    assert_eq!(chain.name, "Ethereum Mainnet");
    assert_eq!(tx.chain_id, Some(1));
}

#[test]
fn test_decode_unknown_chain() {
    let evm = EvmDecoder::new().unwrap();
    let mut tx_bytes = vec![/* valid RLP */];

    // Modify to have unknown chain ID 999999
    // ...

    let result = evm.decode(&tx_bytes, Some(999999));
    assert!(result.is_err());
}

#[test]
fn test_chain_registry_loading() {
    let evm = EvmDecoder::new().unwrap();

    // Should have 500+ chains
    assert!(evm.list_chains().len() > 500);

    // Should include major chains
    assert!(evm.is_supported(1));   // Ethereum
    assert!(evm.is_supported(56));  // BNB
    assert!(evm.is_supported(137)); // Polygon
}
```

### Integration Tests

**Test with Real Transactions**:
```rust
#[test]
fn test_real_polygon_transaction() {
    let evm = EvmDecoder::new().unwrap();

    // Real Polygon transaction from block explorer
    let tx_hex = "f86d...";
    let tx_bytes = hex::decode(tx_hex).unwrap();

    let (tx, chain) = evm.decode(&tx_bytes, None).unwrap();

    assert_eq!(chain.chain_id, 137);
    assert_eq!(chain.name, "Polygon Mainnet");
}
```

### Property Tests

```rust
proptest! {
    #[test]
    fn test_any_valid_evm_transaction(
        chain_id in 1u64..100000,
        tx in arbitrary_ethereum_tx()
    ) {
        let evm = EvmDecoder::new().unwrap();

        let encoded = encode_rlp(&tx);

        match evm.decode(&encoded, Some(chain_id)) {
            Ok((decoded, _)) => {
                // Should successfully decode
                prop_assert_eq!(decoded.chain_id, Some(chain_id));
            }
            Err(_) => {
                // Chain not in registry is acceptable
            }
        }
    }
}
```

---

## Documentation Updates

### README Update

Add section:
```markdown
## Supported Chains

The Universal Blockchain Decoder supports **500+ EVM-compatible chains** through
a single generic decoder powered by [chainlist.org](https://chainlist.org/).

### Using the Generic EVM Decoder

```rust
use universal_decoder::evm::EvmDecoder;

// Initialize decoder
let evm = EvmDecoder::new()?;

// Decode transaction for any EVM chain
let tx_bytes = hex::decode("f86c...")?;
let (tx, chain_info) = evm.decode(&tx_bytes, None)?;

println!("Chain: {} (ID: {})", chain_info.name, chain_info.chain_id);
```

### Supported Chains

To see all supported chains:
```rust
for chain in evm.list_chains() {
    println!("{}: {}", chain.chain_id, chain.name);
}
```
```

### Migration Guide

Create `docs/MIGRATION_TO_EVM_DECODER.md`:
```markdown
# Migration Guide: Chain-Specific Decoders → Generic EVM Decoder

## For BNB Chain

### Before
```rust
use decoder_bnb::BnbDecoder;

let tx = BnbDecoder::decode(&tx_bytes)?;
```

### After
```rust
use decoder_evm::EvmDecoder;

let evm = EvmDecoder::new()?;
let (tx, chain_info) = evm.decode(&tx_bytes, Some(56))?;
// chain_info.name == "BNB Chain"
```

## Benefits
- Support for 500+ chains instead of just 5
- Automatic updates when new chains are added
- Chain metadata included in response
```

---

## Risks and Mitigation

### Risk 1: Chainlist.org Data Quality

**Risk**: Incorrect or outdated chain information

**Mitigation**:
- Pin specific version of chains.json in git
- Add validation tests for known chains
- Allow manual overrides via config file
- Regular audits of chain data

### Risk 2: Breaking Changes in Chainlist Schema

**Risk**: chains.json format changes unexpectedly

**Mitigation**:
- Version pinning
- Schema validation at compile time
- Fallback to embedded version
- CI tests against latest chainlist.org

### Risk 3: Performance with 500+ Chains

**Risk**: HashMap lookup might be slow

**Mitigation**:
- Benchmark shows HashMap lookup is O(1), negligible overhead
- Lazy loading of chain metadata
- Caching frequently-used chains
- Binary search for ordered lookups

### Risk 4: Missing Special Cases

**Risk**: Some chains have undocumented custom tx types

**Mitigation**:
- Community reporting
- Comprehensive testing with real transactions
- Graceful fallback to standard Ethereum decoding
- Error messages guide users to report issues

---

## Success Metrics

**Phase 1 Complete**:
- [ ] `decoder-evm` crate created and tested
- [ ] Supports 500+ chains from chainlist.org
- [ ] Performance: <1ms overhead vs direct Ethereum decoder
- [ ] 100% test coverage on core logic

**Phase 2 Complete**:
- [ ] All 5 existing EVM decoders migrated
- [ ] Zero regressions in functionality
- [ ] CI pipeline passes
- [ ] Documentation updated

**Phase 3 Complete**:
- [ ] Special cases (Optimism, Arbitrum) working
- [ ] Auto-detection functional
- [ ] Integration tests with real transactions
- [ ] Migration guide published

---

## Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1 | Generic Decoder | `decoder-evm` crate, chainlist integration |
| 2 | Migration | Migrate 5 existing decoders |
| 3 | Special Cases | Optimism deposits, Arbitrum retryables |
| 4 | Polish | Docs, examples, CI updates |

**Total**: 4 weeks to full completion

---

## Open Questions

1. **Chain ID conflicts**: What if two chains claim same ID?
   - **Resolution**: Use chainlist.org as source of truth, allow manual overrides

2. **Testnet handling**: Should testnets be separate?
   - **Resolution**: Include in same registry, filter by `is_testnet` flag

3. **Custom RPC endpoints**: Should decoder know about RPCs?
   - **Resolution**: Yes, include in ChainInfo for convenience, but decoder doesn't use them

4. **Automatic updates**: Should chains.json auto-update?
   - **Resolution**: No for security. Manual update process via PR.

---

## References

- [Chainlist.org](https://chainlist.org/)
- [Chainlist GitHub](https://github.com/ethereum-lists/chains)
- [chains.json API](https://chainid.network/chains.json)
- [EIP-155: Simple replay attack protection](https://eips.ethereum.org/EIPS/eip-155)

---

**Status**: Ready for implementation
**Next Action**: Create `decoder-evm` crate
**Owner**: TBD
**Review Date**: TBD
