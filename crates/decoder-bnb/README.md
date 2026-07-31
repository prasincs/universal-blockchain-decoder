# decoder-bnb

BNB Chain (Binance Smart Chain) transaction decoder for the Universal Blockchain Decoder.

## Status

**Phase**: 1 (Scaffolding) ✅
**Production Ready**: ❌ (Phase 2 in progress)

## Overview

BNB Chain is an EVM-compatible blockchain that uses the same transaction format as Ethereum. This decoder will reuse the Ethereum decoder with BNB-specific validation.

## Chain Specification

- **Chain ID**: 56 (mainnet), 97 (testnet)
- **Consensus**: Proof of Staked Authority (PoSA)
- **Chain Family**: Account (EVM)
- **Transaction Format**: RLP-encoded (identical to Ethereum)
- **Address Format**: 0x-prefixed hex (same as Ethereum)

## Implementation Plan

### Phase 1: Scaffolding ✅

- [x] Chain identity implementation
- [x] Basic decoder structure
- [x] Cargo.toml with dependencies
- [x] Stub implementation

### Phase 2: Pure Rust Implementation (Planned)

**Complexity**: Very Low (reuse Ethereum decoder)

**Timeline**: Week 1-2

**Tasks**:
1. Import Ethereum RLP parser
2. Add chain ID validation (56 for mainnet, 97 for testnet)
3. Handle PoSA-specific validation (if needed)
4. Add integration tests with real BSC transactions

**Implementation Strategy**:
```rust
// Reuse Ethereum decoder:
pub struct BnbDecoder;

impl ChainDecoder for BnbDecoder {
    type TxSpecific = EthereumTransaction;  // Reuse!
    type Chain = BnbChain;

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Use Ethereum RLP decoder
        let tx = EthereumDecoder::decode(raw_bytes)?;

        // Validate chain ID
        if let Some(chain_id) = tx.chain_id() {
            if chain_id != 56 && chain_id != 97 {
                return Err(DecoderError::invalid_structure(
                    format!("Invalid BNB Chain ID: {}", chain_id)
                ));
            }
        }

        Ok(tx)
    }
}
```

### Phase 3: Testing & Validation

**Test Plan**:
1. Unit tests for chain ID validation
2. Integration tests with real BSC transactions
3. Property tests (fuzz RLP encoding)
4. Validation of the RLP parse is inherited from `decoder-ethereum`'s
   `alloy_differential` suite, since BNB's `decode()` delegates entirely to
   `EthereumDecoder` (no separate alloy oracle here — it would only re-test the
   identical parser).

**Test Fixtures**:
- BSC mainnet transactions (chain ID 56)
- BSC testnet transactions (chain ID 97)
- BEP-20 token transfers
- Pancakeswap transactions
- Validator staking operations

### Phase 4: Documentation

- API documentation
- Migration guide from web3.js/ethers.js
- Example usage with BSC DEX transactions

## Dependencies

### Production Dependencies

Will share dependencies with Ethereum decoder:
- `universal-decoder-core` - Core traits and types
- `decoder-primitives` - Shared primitives
- RLP parser (from Ethereum decoder)

### Dev Dependencies

- `serde_json` - For test fixtures
- `proptest` - Property-based testing

RLP-decode validation is covered by `decoder-ethereum` (BNB reuses the
Ethereum decoder), so no `alloy-*` oracle dev-deps are declared here.

## Key Differences from Ethereum

1. **Chain ID**: 56 (mainnet), 97 (testnet)
2. **Consensus**: PoSA instead of PoW/PoS
   - 21 validators
   - Epoch-based validator selection
   - Faster block times (3 seconds)
3. **Staking**: Validators stake BNB
4. **Token Standards**: BEP-2, BEP-20 (similar to ERC-20)

**Note**: These differences are mostly consensus-level and don't affect transaction decoding. The transaction format is identical to Ethereum.

## Resources

- [BNB Chain Docs](https://docs.bnbchain.org/)
- [BSC Transaction Format](https://docs.bnbchain.org/docs/learn/transactions)
- [BEP Standards](https://github.com/bnb-chain/BEPs)
- [BSC Explorer](https://bscscan.com/)

## License

MIT OR Apache-2.0
