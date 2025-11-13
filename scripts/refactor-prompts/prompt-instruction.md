# Instruction-Based Chain Family Analysis

You are analyzing an instruction-based blockchain decoder (Solana, Aptos, Sui).

## Instruction Model-Specific Focus Areas

### 1. Transaction Structure

#### Solana
- **Message**:
  - Account keys (array of public keys)
  - Recent blockhash (PoH reference)
  - Instructions (array of program invocations)
- **Instruction**:
  - Program ID (which program to execute)
  - Account indices (which accounts to pass)
  - Data (instruction-specific parameters)
- **Signatures**: Array of signatures (one per signer)

#### Aptos (Move-based)
- **Transaction**:
  - Sender address
  - Sequence number
  - Payload (script, module, or entry function)
  - Max gas, gas price
  - Expiration timestamp
  - Chain ID
- **Signature**: Ed25519 or multi-sig
- **BCS encoding**: Binary Canonical Serialization

#### Sui (Move-based)
- **Transaction**:
  - Transaction kind (Move call, transfer, publish, etc.)
  - Sender address
  - Gas payment (object references)
  - Gas budget, gas price
  - Expiration epoch
- **Signature**: Ed25519 or multi-sig
- **BCS encoding**: Binary Canonical Serialization

### 2. Encoding Formats

#### Solana: Compact-Array Encoding
- **CompactU16 length prefix** for arrays
- **Base58 encoding** for addresses/signatures (display only)
- **Bincode-like serialization** (custom format)

#### Aptos/Sui: BCS (Binary Canonical Serialization)
- **ULEB128** for lengths
- **Little-endian** for integers
- **Deterministic** (canonical property)
- Similar to Borsh but Move-specific

### 3. Chain-Specific Challenges

#### Solana
- **Account model**: Transactions reference accounts by index
- **Program execution**: Complex cross-program invocations (CPI)
- **Versioned transactions**: Legacy vs v0 (address lookup tables)
- **Durable nonces**: Alternative to recent blockhash

#### Aptos
- **Move VM**: Bytecode verification required for full validation
- **Object model**: Resources and modules
- **Parallel execution**: Transaction dependencies affect ordering
- **Gas schedule**: Complex, version-dependent

#### Sui
- **Object-centric**: Transactions operate on objects (not accounts)
- **Ownership model**: Owned, shared, and immutable objects
- **Move VM**: Similar to Aptos but different standard library
- **Sponsored transactions**: Gas payer can be different from sender

### 4. Dependency Strategy (CRITICAL)
- **Pure Rust implementation required**
- `solana-sdk` / `aptos-sdk` / `sui-sdk` → dev-dependencies ONLY
- Implement custom parsing for:
  - **Solana**: Compact-array encoding, message deserialization
  - **Aptos/Sui**: BCS deserialization (or vendor minimal BCS library)
  - Signature verification (Ed25519)
  - Address encoding/decoding

### 5. Latest Protocol Updates

#### Solana
- **Versioned transactions (v0)**: Address lookup tables for lower fees
- **QUIC protocol**: Improved transaction submission (not decoder-relevant)
- **Fee markets**: Priority fees (compute budget program)
- **State compression**: Compressed NFTs using Merkle trees

#### Aptos
- **Aptos Framework v1.x**: Ongoing module updates
- **Parallel execution**: BlockSTM consensus
- **Keyless accounts**: OAuth-based authentication

#### Sui
- **Sui Move**: Diverging from Aptos Move
- **Mysticeti consensus**: High throughput
- **Object versioning**: Changes affect transaction structure

### 6. Security Checklist
- [ ] Input validation: Account indices (Solana), object references (Sui)
- [ ] Overflow protection: Gas calculations, sequence numbers
- [ ] Canonical encoding: BCS (Aptos/Sui) is canonical; Borsh for TxIR
- [ ] No unsafe code blocks
- [ ] Deserialization: Max depth limits (recursive structures)
- [ ] Signature verification: Ed25519 curve validation

### 7. Performance Optimizations
- [ ] Zero-copy deserialization where possible (Solana account keys)
- [ ] Efficient Ed25519 verification (use `ed25519-dalek`)
- [ ] Avoid base58 conversions in hot paths (work with raw bytes)
- [ ] Pre-allocate instruction/account arrays
- [ ] BCS decoding: ULEB128 optimization

### 8. Testing Requirements
- [ ] Unit tests: Each instruction type, transaction version
- [ ] Property tests: Serialization round-trip, signature verification
- [ ] Integration tests: Real transactions from mainnet
- [ ] Fixtures: Include at least 12 diverse transactions
  - **Solana**:
    - Legacy transaction
    - Versioned transaction (v0)
    - Multi-signature
    - Token transfer (SPL token program)
    - NFT mint/transfer
    - Durable nonce transaction
  - **Aptos**:
    - Entry function call
    - Script payload
    - Module deployment
    - Multi-agent transaction
  - **Sui**:
    - Move call
    - Object transfer
    - Package publish
    - Sponsored transaction

### 9. Move Language Considerations (Aptos/Sui)
- **Bytecode verification**: Out of scope for decoder (validation layer)
- **Module dependencies**: Track for metadata, but don't execute
- **Type parameters**: Generic types in Move functions
- **Resources**: Linear types (consume-once semantics)

### 10. Instruction Decoding Depth
- **Level 1 (Required)**: Parse transaction structure, extract instructions
- **Level 2 (Optional)**: Decode well-known program instructions (token, NFT)
- **Level 3 (Out of scope)**: Full program execution simulation

## Analysis Instructions

Provide specific, actionable suggestions in the following categories:

1. **Dependency**: Pure Rust implementation gaps (BCS, compact-array), unnecessary SDK dependencies
2. **Security**: Deserialization vulnerabilities, signature verification, overflow protection
3. **Performance**: Encoding/decoding efficiency, signature verification performance
4. **Testing**: Missing transaction types (versioned, sponsored), property tests
5. **Architecture**: Separation of parsing vs instruction decoding, trait implementations

Focus on alignment with project goals:
- Minimal trusted computing base
- Formally verifiable code (no unsafe)
- Canonical serialization (BCS for Aptos/Sui, custom for Solana; Borsh for TxIR)
- Pure Rust decoders (SDK dependencies in dev-deps only)
- Clear separation: transaction parsing vs instruction semantics
