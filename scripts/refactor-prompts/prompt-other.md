# Other Chain Family Analysis

You are analyzing a blockchain decoder for chains with unique transaction models (XRP, Tron, Polkadot, NEAR, Cosmos, Stellar, Algorand).

## Chain-Specific Focus Areas

### XRP Ledger
- **Binary Codec**: Custom binary serialization (not Protobuf, not JSON)
- **Transaction Types**: Payment, OfferCreate, TrustSet, AccountSet, etc. (25+ types)
- **Canonical Field Encoding**: Field ID (type + field) + value
- **Signing**: ECDSA (secp256k1) or Ed25519
- **Sequence numbers**: Account-based transaction ordering
- **Latest**: XLS-20 NFTs, Hooks amendments

### Tron
- **Protobuf Encoding**: Google Protocol Buffers v3
- **Transaction Structure**: Similar to Ethereum but different encoding
- **Contract types**: Transfer, TriggerSmartContract, etc.
- **Resource model**: Energy + Bandwidth instead of gas
- **Latest**: TRC-20 (token standard), TRON 4.x upgrades

### Polkadot (and Parachains)
- **SCALE Encoding**: Simple Concatenated Aggregate Little-Endian
- **Extrinsics**: Signed vs unsigned, inherent
- **Pallet calls**: Modular runtime with different call types
- **Multi-signature**: Complex account schemes
- **Metadata**: Runtime-dependent (different for each parachain)
- **Latest**: XCM v3 (cross-chain messaging), Async backing

### NEAR Protocol
- **Borsh Encoding**: Binary Object Representation Serializer for Hashing
- **Transaction Structure**: Actions (CreateAccount, Transfer, FunctionCall, etc.)
- **Account model**: Named accounts (not addresses)
- **Gas**: Attached gas for function calls
- **Latest**: NEAR Protocol 1.x, meta-transactions

### Cosmos SDK
- **Protobuf Encoding**: Amino (legacy) or Protobuf (modern)
- **Transaction Structure**: Msgs (array of messages), Fee, Memo
- **Signing**: Multiple signature modes (Direct, Amino, Textual)
- **Modules**: Bank, Staking, Gov, IBC, etc.
- **IBC**: Inter-Blockchain Communication protocol
- **Latest**: Cosmos SDK v0.47+, ABCI 2.0

### Stellar
- **XDR Encoding**: External Data Representation (RFC 4506)
- **Transaction Structure**: Operations (Payment, CreateAccount, etc.)
- **Sequence numbers**: Account-based ordering
- **Multi-signature**: Threshold signatures
- **Latest**: Protocol 20 (Soroban smart contracts), CAP-40

### Algorand
- **MessagePack Encoding**: Efficient binary JSON-like format
- **Transaction Types**: Payment, AssetTransfer, ApplicationCall, etc.
- **Account model**: Addresses are 32-byte hashes
- **Atomic transfers**: Transaction groups
- **Latest**: AVM 1.1 (AlgoVM), state proofs

## Common Patterns Across "Other" Chains

### 1. Encoding Formats
- **Binary serialization**: Most use custom formats (not JSON)
- **Canonical property**: Check if encoding is deterministic
- **Versioning**: Handle multiple format versions

### 2. Transaction Structure
- **Header**: Sender, nonce/sequence, fee/gas, expiration
- **Payload**: Chain-specific (operations, calls, actions)
- **Signature**: Various schemes (ECDSA, Ed25519, BLS, multi-sig)

### 3. Dependency Strategy (CRITICAL)
- **Pure Rust implementation required**
- Chain-specific SDKs → dev-dependencies ONLY
- Implement or vendor parsing libraries:
  - XRP: Custom binary codec
  - Tron: Protobuf (prost crate acceptable if vendored)
  - Polkadot: SCALE (parity-scale-codec - consider vendoring)
  - NEAR: Borsh (already in workspace deps!)
  - Cosmos: Protobuf + Amino codec
  - Stellar: XDR codec
  - Algorand: MessagePack (rmp crate)

### 4. Security Checklist
- [ ] Input validation: Format-specific (field IDs, types, lengths)
- [ ] Overflow protection: Integer fields, fee calculations
- [ ] Canonical encoding: Use format's canonical form; Borsh for TxIR
- [ ] No unsafe code blocks
- [ ] Deserialization: Max depth limits, bounds checking
- [ ] Signature verification: Curve-specific validation

### 5. Performance Optimizations
- [ ] Zero-copy deserialization where possible
- [ ] Efficient encoding libraries (avoid unnecessary allocations)
- [ ] Pre-allocate for known sizes
- [ ] Lazy parsing: Don't decode unused fields

### 6. Testing Requirements
- [ ] Unit tests: Each transaction/operation type
- [ ] Property tests: Serialization round-trip
- [ ] Integration tests: Real transactions from mainnet
- [ ] Fixtures: At least 10 diverse transactions per chain
  - Common operations (transfer, contract call)
  - Complex transactions (multi-sig, atomic)
  - Edge cases (empty fields, max values)

### 7. Chain-Specific Nuances

#### XRP
- **Field ordering**: Must follow canonical order
- **Type system**: 16 types (UInt32, Amount, Blob, etc.)
- **Amendments**: Protocol features enabled by validator voting

#### Tron
- **Protobuf definitions**: Use official `.proto` files
- **Contract addresses**: Base58 encoded (start with 'T')
- **Resource freezing**: Bandwidth/energy mechanics

#### Polkadot
- **Metadata**: Changes with runtime upgrades
- **Call indices**: Not stable across versions
- **SS58 address format**: Network-specific prefix

#### NEAR
- **Account IDs**: UTF-8 strings (not hashes)
- **Action types**: 8 different action kinds
- **Borsh compatibility**: Ensure version compatibility

#### Cosmos
- **Any type**: Protobuf Any for extensibility
- **Sign modes**: SIGN_MODE_DIRECT, SIGN_MODE_LEGACY_AMINO_JSON
- **IBC packets**: Complex cross-chain message format

#### Stellar
- **XDR limitations**: No optional fields (use discriminated unions)
- **Operation types**: 23 different operation kinds
- **Stroop**: Smallest unit (1/10^7 XLM)

#### Algorand
- **Compact encoding**: MessagePack is space-efficient
- **Transaction groups**: Atomic execution
- **LogicSigs**: Program-signed transactions

## Analysis Instructions

Provide specific, actionable suggestions in the following categories:

1. **Dependency**: Pure Rust encoding implementations, unnecessary SDK dependencies
2. **Security**: Format-specific parsing vulnerabilities, signature verification
3. **Performance**: Encoding/decoding efficiency, allocation patterns
4. **Testing**: Missing transaction types, property tests, edge cases
5. **Architecture**: Separation of parsing vs validation, trait implementations

Focus on alignment with project goals:
- Minimal trusted computing base
- Formally verifiable code (no unsafe)
- Canonical serialization (chain-native format for hashing; Borsh for TxIR)
- Pure Rust decoders (SDK dependencies in dev-deps only)
- Handle chain-specific uniqueness without bloating core
