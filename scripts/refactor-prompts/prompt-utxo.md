# UTXO Chain Family Analysis

You are analyzing a UTXO-based blockchain decoder (Bitcoin, Litecoin, Dogecoin, Cardano).

## UTXO-Specific Focus Areas

### 1. Transaction Structure
- **Inputs (UTXOs consumed)**:
  - Previous transaction hash + output index
  - Unlocking script (scriptSig / witness data)
  - Sequence number
- **Outputs (new UTXOs created)**:
  - Value (satoshis)
  - Locking script (scriptPubKey)
  - Output type detection (P2PKH, P2SH, P2WPKH, P2WSH, P2TR)

### 2. Script Parsing & Validation
- **Script opcodes**: Correct parsing of Bitcoin Script
- **SegWit support**: Witness data handling
- **Taproot support** (if Bitcoin/Litecoin): Schnorr signatures, MAST
- **Script type detection**: Standard vs non-standard outputs
- **Security**: Script validation without executing untrusted code

### 3. UTXO-Specific Challenges
- **Double-spend detection**: Not applicable at decoder level (consensus)
- **UTXO set management**: Decoders should NOT maintain state
- **Transaction malleability**: SegWit transactions vs legacy
- **Fee calculation**: sum(inputs) - sum(outputs)

### 4. Chain-Specific Differences
- **Bitcoin**: Full SegWit + Taproot support
- **Litecoin**: SegWit + MWEB (MimbleWimble Extension Block)
- **Dogecoin**: No SegWit (legacy format only)
- **Cardano**: eUTXO model (extended UTXO with smart contracts + datum)

### 5. Dependency Strategy (CRITICAL)
- **Pure Rust implementation required**
- `bitcoin` crate → dev-dependencies ONLY (for validation tests)
- Implement custom parsing for:
  - VarInt encoding
  - CompactSize integers
  - Script deserialization
  - Transaction hash calculation (double SHA-256)

### 6. Latest Protocol Updates
- **Bitcoin**: Taproot (BIP 340-342), Ordinals/Inscriptions
- **Litecoin**: MWEB activation
- **Cardano**: Plutus V2/V3 scripts, Babel fees

### 7. Security Checklist
- [ ] Input validation: Check for oversized scripts
- [ ] Overflow protection: Satoshi values (u64), fee calculation
- [ ] Canonical encoding: Use Borsh for TxIR, NOT JSON
- [ ] No unsafe code blocks
- [ ] Script parsing: Bounds checking on opcodes

### 8. Performance Optimizations
- [ ] Zero-copy parsing where possible
- [ ] Avoid unnecessary allocations
- [ ] Use `SmallVec` for typical script sizes (if benchmarked)
- [ ] Lazy parsing: Don't parse unused fields

### 9. Testing Requirements
- [ ] Unit tests: Each script type (P2PKH, P2SH, SegWit, Taproot)
- [ ] Property tests: Transaction serialization round-trip
- [ ] Integration tests: Real transactions from mainnet/testnet
- [ ] Fixtures: Include at least 10 diverse transactions
  - Coinbase transaction
  - Multi-input, multi-output
  - SegWit native (P2WPKH, P2WSH)
  - P2SH-wrapped SegWit
  - Taproot (if applicable)

## Analysis Instructions

Provide specific, actionable suggestions in the following categories:

1. **Dependency**: Pure Rust implementation gaps, unnecessary dependencies
2. **Security**: Script parsing vulnerabilities, overflow risks, canonical encoding
3. **Performance**: Parsing inefficiencies, allocation hotspots
4. **Testing**: Missing test coverage, need for property tests
5. **Architecture**: Separation of parsing vs validation, trait implementations

Focus on alignment with project goals:
- Minimal trusted computing base
- Formally verifiable code (no unsafe)
- Canonical serialization (Borsh)
- Pure Rust decoders (blockchain libs in dev-deps only)
