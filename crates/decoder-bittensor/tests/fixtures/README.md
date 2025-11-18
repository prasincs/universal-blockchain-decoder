# Bittensor Transaction Test Fixtures

This directory is reserved for **real Bittensor transaction data** from the mainnet or testnet.

## Current Status

Currently, we use **synthetic SCALE-encoded transactions** defined in `tests/fixtures.rs`. These are properly formatted and test all decoder functionality, but they're not from the actual Bittensor blockchain.

## Why Synthetic Fixtures?

Real transaction data from Bittensor explorers (like Taostats.io) wasn't readily accessible during initial development due to:
- API rate limiting / access restrictions
- Need for RPC node access
- Time constraints

However, the synthetic fixtures are:
- ✅ Properly SCALE-encoded according to Substrate spec
- ✅ Cover all transaction types (transfers, staking, set_weights, etc.)
- ✅ Test all signature types (Sr25519, Ed25519, ECDSA)
- ✅ Test various nonces, tips, and eras
- ✅ Fully valid and parseable

## How to Add Real Bittensor Transactions

### Method 1: Using Bittensor Explorer (Taostats.io)

1. **Find a transaction**:
   - Visit https://taostats.io
   - Search for a recent transaction
   - Example: `https://taostats.io/extrinsic/4408774-0020`

2. **Extract raw extrinsic data**:
   - Use Substrate RPC: `chain.getBlock(blockHash)`
   - Or use `subxt` library to query the chain
   - Raw data is SCALE-encoded bytes

3. **Save as binary file**:
   ```bash
   # Convert hex string to binary
   echo "YOUR_HEX_STRING" | xxd -r -p > mainnet_block_4408774_tx0.bin
   ```

### Method 2: Using Substrate RPC

```bash
# Connect to Bittensor node
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getBlock", "params":["BLOCK_HASH"]}' \
     http://YOUR_BITTENSOR_NODE:9933

# Extract extrinsic from response
# Convert hex to binary and save
```

### Method 3: Using `subxt` (Rust)

```rust
use subxt::{OnlineClient, PolkadotConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Bittensor node
    let api = OnlineClient::<PolkadotConfig>::new().await?;

    // Get block
    let block_hash = "...";
    let block = api.blocks().at(block_hash).await?;

    // Get extrinsics
    let extrinsics = block.extrinsics().await?;
    for ext in extrinsics.iter() {
        let ext = ext?;
        println!("Extrinsic bytes: {:?}", ext.bytes());
        // Save to file
    }

    Ok(())
}
```

## Fixture File Naming Convention

```
<network>_<block_number>_<tx_index>_<type>.bin
```

Examples:
- `mainnet_4408774_00_set_weights.bin` - Set weights transaction from mainnet block 4408774
- `mainnet_6618343_17_unstake.bin` - Unstake transaction
- `testnet_1234567_05_transfer.bin` - TAO transfer on testnet

## Metadata Files

For each `.bin` file, create a corresponding `.json` metadata file:

```json
{
  "network": "mainnet",
  "block_number": 4408774,
  "tx_index": 0,
  "tx_hash": "0x3a56c3f4fc252e1b211f190735b826c06d9f72b5691ad4f8243421a121b12118",
  "timestamp": "2025-10-09T12:34:56Z",
  "pallet": "SubtensorModule",
  "call": "set_weights",
  "signer": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "nonce": 42,
  "tip": 0,
  "success": true,
  "description": "Set weights for subnet 1, neuron UID 123"
}
```

## Adding Tests for Real Fixtures

Once you have real fixture files, add integration tests:

```rust
// In tests/bittensor_integration.rs

#[test]
fn test_real_mainnet_set_weights() {
    let tx_bytes = include_bytes!("fixtures/mainnet_4408774_00_set_weights.bin");
    let metadata: serde_json::Value = serde_json::from_str(
        include_str!("fixtures/mainnet_4408774_00_set_weights.json")
    ).unwrap();

    let tx = BittensorDecoder::decode(tx_bytes).unwrap();

    // Verify against metadata
    assert!(tx.extrinsic.is_signed());

    let call = tx.call().unwrap();
    assert_eq!(call.pallet_name(), metadata["pallet"].as_str().unwrap());
    assert_eq!(call.call_name(), metadata["call"].as_str().unwrap());

    // Verify nonce
    if let Extrinsic::Signed(signed) = &tx.extrinsic {
        assert_eq!(signed.extension.nonce, metadata["nonce"].as_u64().unwrap());
    }

    // Canonicalize and verify
    let tx_ir = tx.canonicalize().unwrap();
    assert!(!tx_ir.operations.is_empty());
}
```

## Recommended Test Cases

When adding real fixtures, prioritize these transaction types:

### High Priority
- [ ] **Balances::transfer** - Basic TAO transfer
- [ ] **SubtensorModule::set_weights** - Core Bittensor operation
- [ ] **SubtensorModule::add_stake** - Staking TAO
- [ ] **SubtensorModule::register** - Neuron registration

### Medium Priority
- [ ] **SubtensorModule::remove_stake** - Unstaking
- [ ] **SubtensorModule::serve_axon** - Serve axon endpoint
- [ ] **Utility::batch** - Batch operations
- [ ] **System::remark** - Unsigned extrinsic

### Low Priority
- [ ] **Multisig** operations
- [ ] **Proxy** operations
- [ ] **Registry** operations

## Verification

Before committing fixtures, verify they decode correctly:

```bash
# Quick test
cargo test --package decoder-bittensor test_real_mainnet

# Verbose output
cargo test --package decoder-bittensor test_real_mainnet -- --nocapture
```

## Resources

- **Taostats Explorer**: https://taostats.io
- **Bittensor Docs**: https://docs.bittensor.com
- **Substrate RPC**: https://docs.substrate.io/reference/command-line-tools/subxt/
- **SCALE Codec**: https://docs.substrate.io/reference/scale-codec/

## Contributing

When adding real fixtures:

1. ✅ Use descriptive filenames
2. ✅ Include metadata JSON files
3. ✅ Add integration tests
4. ✅ Verify tests pass
5. ✅ Document the transaction purpose
6. ✅ Ensure fixtures are from public, non-sensitive transactions

---

**Status**: 🔨 Synthetic fixtures in use, real fixtures welcome!
**Last Updated**: 2025-11-18
