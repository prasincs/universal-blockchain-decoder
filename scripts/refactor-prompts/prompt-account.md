# Account-Based Chain Family Analysis (EVM)

You are analyzing an account-based blockchain decoder, primarily EVM-compatible chains (Ethereum, Polygon, BSC, Avalanche, Optimism, Arbitrum).

## Account Model-Specific Focus Areas

### 1. Transaction Structure
- **Basic fields**:
  - Nonce (account transaction counter)
  - Gas price / Gas limit
  - To address (or null for contract creation)
  - Value (wei)
  - Data (contract call / deployment bytecode)
- **Signature**:
  - v, r, s (ECDSA signature)
  - Chain ID recovery (EIP-155)

### 2. Transaction Types (EIP-2718)
- **Type 0 (Legacy)**: Pre-EIP-155 and EIP-155 formats
- **Type 1 (EIP-2930)**: Access list transactions
- **Type 2 (EIP-1559)**: Fee market with max fee + priority fee
- **Type 3 (EIP-4844)**: Blob transactions (Cancun upgrade)
- **Future types**: Extensible envelope format

### 3. RLP Encoding (Recursive Length Prefix)
- **Pure Rust implementation**: Custom RLP parser (no `alloy-rlp` in production)
- **Canonical encoding**: RLP is deterministic
- **Edge cases**:
  - Empty lists vs empty strings
  - Single-byte values (< 0x80)
  - Length encoding for strings > 55 bytes
  - Nested lists

### 4. EVM-Specific Challenges
- **Contract interaction decoding**:
  - Function selector (first 4 bytes of keccak256(signature))
  - ABI decoding (optional, not required for TxIR)
- **Event logs**: Topic0 = event signature hash
- **Internal transactions**: Not visible at transaction level
- **Gas estimation**: Complex, depends on state

### 5. Chain-Specific Differences
- **Ethereum**: All transaction types, latest upgrades (Cancun)
- **Polygon**: EIP-1559, lower gas costs
- **BSC**: Similar to Ethereum, centralized validator set
- **Avalanche C-Chain**: EVM-compatible, different consensus
- **Optimism/Arbitrum**: L2 rollups, additional L1 deposit transactions

### 6. Dependency Strategy (CRITICAL)
- **Pure Rust implementation required**
- `ethers` / `alloy` → dev-dependencies ONLY (for validation tests)
- Implement custom parsing for:
  - RLP encoding/decoding
  - Keccak-256 hashing
  - ECDSA signature verification
  - EIP-2718 transaction envelope
  - Address checksum (EIP-55)

### 7. Latest Protocol Updates
- **Ethereum Cancun (2024)**: EIP-4844 blob transactions, transient storage (EIP-1153)
- **Dencun**: Proto-danksharding for L2 data availability
- **Future**: Account abstraction (EIP-4337), Verkle trees
- **alloy-rs**: Modern Ethereum library (successor to ethers), version 0.7+

### 8. Security Checklist
- [ ] Input validation: Address format (20 bytes), signature (v/r/s)
- [ ] Overflow protection: Nonce, gas, value (u64/U256)
- [ ] Canonical encoding: RLP for transaction hash, Borsh for TxIR
- [ ] No unsafe code blocks
- [ ] RLP parsing: Bounds checking, max depth limits (prevent stack overflow)
- [ ] Chain ID validation (EIP-155)

### 9. Performance Optimizations
- [ ] Zero-copy RLP decoding where possible
- [ ] Efficient Keccak hashing (use `sha3` crate)
- [ ] Avoid hex string conversions in hot paths
- [ ] Pre-allocate buffers for known sizes
- [ ] Use `SmallVec` for transaction signatures (3 elements: v, r, s)

### 10. Testing Requirements
- [ ] Unit tests: Each transaction type (0, 1, 2, 3)
- [ ] Property tests: RLP round-trip, signature verification
- [ ] Integration tests: Real transactions from mainnet
- [ ] Fixtures: Include at least 15 diverse transactions
  - Legacy (pre-EIP-155)
  - Legacy (post-EIP-155)
  - EIP-2930 (access list)
  - EIP-1559 (dynamic fee)
  - EIP-4844 (blob transaction)
  - Contract creation
  - Contract call with complex data
  - Token transfer (ERC-20)
  - NFT transfer (ERC-721)
  - Failed transactions (still valid)

### 11. Multi-Chain Support
- [ ] Generic EVM decoder for 2000+ chains
- [ ] Chain-specific decoders only for significant differences
- [ ] Shared RLP/signature logic in `decoder-evm`
- [ ] Chain registry for chain ID → name mapping

### 12. L2-Specific Considerations
- **Optimism**: Additional L1 attributes in transaction metadata
- **Arbitrum**: Retryable tickets, delayed inbox messages
- **zkEVM (Polygon, zkSync)**: Different gas accounting

## Analysis Instructions

Provide specific, actionable suggestions in the following categories:

1. **Dependency**: Pure Rust RLP implementation, unnecessary dependencies
2. **Security**: RLP parsing vulnerabilities, signature verification, overflow protection
3. **Performance**: RLP decoding efficiency, hashing performance
4. **Testing**: Missing transaction types, property tests, L2-specific tests
5. **Architecture**: Separation of generic EVM vs chain-specific logic, trait implementations

Focus on alignment with project goals:
- Minimal trusted computing base
- Formally verifiable code (no unsafe)
- Canonical serialization (Borsh for TxIR, RLP for transaction hashing)
- Pure Rust decoders (blockchain libs in dev-deps only)
- Support 2000+ EVM chains through generic decoder
