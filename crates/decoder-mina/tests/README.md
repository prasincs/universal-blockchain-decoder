# Mina Protocol Test Vectors

This directory contains test vectors for the Mina Protocol decoder, with a focus on compatibility with the **o1js** (SnarkyJS) reference implementation.

## Test Structure

### `o1js_test_vectors.rs`

Comprehensive test vectors derived from o1js to ensure our implementation is compatible with the Mina Protocol ecosystem.

**Test Categories**:

1. **Pallas Field Tests**
   - Field modulus validation
   - Arithmetic operations (add, sub, mul, inv)
   - Field properties (associativity, commutativity)

2. **Poseidon Hash Tests**
   - Hash determinism
   - Hash avalanche effect (security property)
   - Merkle tree construction (hash of hashes)
   - zkApp state hashing (8 field elements)

3. **Public Key Tests**
   - Key creation and validation
   - Address encoding (B62q... format)
   - Key equality and compression

4. **Signature Tests**
   - Signature structure validation
   - Determinism tests

5. **Transaction Decoding Tests** (TODO: Phase 3.9)
   - Payment transactions
   - zkApp transactions
   - Delegation transactions

6. **Property-Based Tests**
   - Field arithmetic properties
   - Hash determinism
   - Avalanche effect

## Current Status

### ✅ Implemented
- Pallas field arithmetic tests
- Poseidon hash structural tests (determinism, non-commutativity)
- Public key and signature type tests
- Property-based tests for field operations

### ⏳ Pending (Phase 3.9 Full Implementation)
- Actual Poseidon round constants from o1js
- Actual MDS matrix from o1js
- Real transaction test vectors from Mina mainnet
- zkApp transaction parsing
- Signature verification
- Merkle tree verification

## o1js Compatibility

### What We Need from o1js

To achieve full compatibility with o1js, we need to extract:

1. **Poseidon Constants** (`src/lib/provable/crypto/poseidon.ts`)
   - Round constants (63 rounds × 3 state elements = 189 constants)
   - MDS matrix (3×3 matrix of field elements)
   - Domain separation tags

2. **Test Vectors** (`src/lib/provable/test/`)
   - Known hash outputs for specific inputs
   - Transaction serialization examples
   - Signature verification examples

3. **Encoding Specifications**
   - Transaction binary format
   - Field element serialization (big-endian/little-endian)
   - Public key compression format
   - Signature encoding

### Reference Implementation Sources

```typescript
// o1js Poseidon hash
import { Poseidon } from 'o1js';
const hash = Poseidon.hash([Field(1), Field(2)]);
// hash.toString() => "0x..."

// o1js Public key
import { PublicKey } from 'o1js';
const pk = PublicKey.fromBase58('B62qr...');
// pk.toFields() => [Field, Field]

// o1js Signature
import { Signature } from 'o1js';
const sig = Signature.create(privateKey, [Field(123)]);
// sig.toFields() => [Field, Field]
```

## Extracting Test Vectors

### Method 1: Use o1js Directly

```bash
npm install o1js

node << 'EOF'
const { Field, Poseidon } = require('o1js');

// Generate test vector
const a = Field(123);
const b = Field(456);
const hash = Poseidon.hash([a, b]);

console.log({
  input_a: a.toString(),
  input_b: b.toString(),
  output: hash.toString()
});
EOF
```

### Method 2: Extract from o1js Source

```bash
# Clone o1js repository
git clone https://github.com/o1-labs/o1js.git
cd o1js

# Find Poseidon constants
cat src/lib/provable/crypto/poseidon.ts | grep -A 200 "ROUND_CONSTANTS"

# Find test vectors
find src/lib/provable/test -name "*.test.ts" -exec grep -l "Poseidon" {} \;
```

### Method 3: Query Mina GraphQL API

```bash
# Get real transaction data
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"query": "{ bestChain(maxLength: 1) { transactions { userCommands { hash amount } } } }"}' \
  https://graphql.minaexplorer.com/
```

## Test Vector Format

When adding test vectors, use this format:

```rust
#[test]
fn test_o1js_vector_NAME() {
    // Source: o1js/src/lib/provable/test/poseidon.test.ts:L42
    // Input: Field(123), Field(456)
    // Expected output: 0x1234...abcd

    let a = PallasFieldElement::from(123u64);
    let b = PallasFieldElement::from(456u64);
    let hash = PoseidonPallasHash::hash_pair(a, b);

    let expected = PallasFieldElement::from_hex(
        "0x1234...abcd"  // Actual value from o1js
    ).unwrap();

    assert_eq!(hash, expected);
}
```

## Running Tests

```bash
# Run all Mina tests
cargo test -p decoder-mina

# Run only o1js test vectors
cargo test -p decoder-mina o1js_test_vectors

# Run with detailed output
cargo test -p decoder-mina -- --nocapture

# Run ignored tests (when ready)
cargo test -p decoder-mina -- --ignored
```

## Contributing Test Vectors

When adding new test vectors:

1. **Source**: Document where the vector came from (o1js version, file, line)
2. **Format**: Use consistent hex formatting (0x prefix, lowercase)
3. **Description**: Explain what the test verifies
4. **Reference**: Link to o1js documentation or code

Example:

```rust
/// Test vector from o1js v0.15.0
/// Source: https://github.com/o1-labs/o1js/blob/v0.15.0/src/lib/provable/test/poseidon.test.ts#L42
/// Verifies: Poseidon hash of two sequential numbers
#[test]
fn test_o1js_v0_15_poseidon_sequential() {
    // ... test implementation
}
```

## Phase 3.9 Implementation Checklist

- [x] Basic Pallas field tests
- [x] Basic Poseidon structure tests
- [x] Public key type tests
- [x] Signature type tests
- [x] Property-based tests
- [ ] Extract Poseidon round constants from o1js
- [ ] Extract MDS matrix from o1js
- [ ] Add exact Poseidon hash test vectors
- [ ] Add payment transaction test vectors
- [ ] Add zkApp transaction test vectors
- [ ] Add signature verification test vectors
- [ ] Add Merkle tree test vectors
- [ ] Cross-validate all tests with o1js

## References

- [o1js Documentation](https://docs.minaprotocol.com/zkapps/o1js-reference)
- [o1js GitHub](https://github.com/o1-labs/o1js)
- [Mina Protocol Specification](https://o1-labs.github.io/proof-systems/)
- [Poseidon Hash Paper](https://eprint.iacr.org/2019/458.pdf)
- [Pasta Curves](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/)
- [Mina GraphQL API](https://graphql.minaexplorer.com/)
