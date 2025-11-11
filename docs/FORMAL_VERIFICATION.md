# Formal Verification Plan for Universal Blockchain Decoder

## Overview

This document outlines the formal verification strategy for the universal blockchain decoder, focusing on proving critical security properties using Verus.

## Verification Targets

### 1. Core Properties (Priority: CRITICAL)

#### 1.1 Injectivity of Canonicalization

**Property**: For all valid transaction bytes `tx_bytes`:
```
encode(canonicalize(decode(tx_bytes))) == tx_bytes
```

**Why it matters**: Ensures signature verification integrity and prevents transaction malleability attacks.

**Verification approach**:
```rust
// In Verus syntax (pseudo-code)
#[verifier::proof]
fn canonicalization_is_injective(tx_bytes: &[u8])
    requires valid_transaction_bytes(tx_bytes)
    ensures {
        let tx = decode(tx_bytes)?;
        let ir = tx.canonicalize()?;
        let reconstructed = ir.to_canonical_bytes();
        reconstructed == tx_bytes
    }
```

#### 1.2 Panic-Freedom

**Properties**:
- No array index out of bounds
- No integer overflow in arithmetic operations
- No division by zero
- No unwrap() failures on Result/Option

**Verification approach**:
```rust
#[verifier::proof]
fn decode_is_panic_free(tx_bytes: &[u8])
    requires tx_bytes.len() < MAX_TX_SIZE
    ensures no_panic()
{
    BitcoinDecoder::decode(tx_bytes)
}
```

#### 1.3 Resource Bounds

**Properties**:
- Gas calculations don't overflow
- Amount transfers don't exceed u128::MAX
- Transaction size within limits

### 2. Structural Invariants (Priority: HIGH)

#### 2.1 Transaction Version Constraints

Using const generics, prove version isolation:

```rust
#[verifier::proof]
fn version_isolation<const V1: u8, const V2: u8>(tx: TxIR<V1>)
    requires V1 != V2
    ensures cannot_cast_to::<TxIR<V2>>(tx)
```

#### 2.2 Input/Output Validity

For UTXO models:
```rust
#[verifier::invariant]
fn valid_utxo_set(tx: &BitcoinTransaction) -> bool {
    tx.input_count() > 0 &&
    tx.output_count() > 0 &&
    tx.inputs().all(|i| i.prev_tx.len() == 32)
}
```

### 3. Cryptographic Properties (Priority: MEDIUM)

#### 3.1 Hash Determinism

```rust
#[verifier::proof]
fn hash_is_deterministic(tx_bytes: &[u8])
    ensures {
        let h1 = DoubleSha256::hash(tx_bytes);
        let h2 = DoubleSha256::hash(tx_bytes);
        h1 == h2
    }
```

## Implementation Strategy

### Phase 1: Core Module Verification (4-6 weeks)

**Target**: `universal-decoder-core`

1. Add Verus annotations to `TxIR`
2. Verify `DecoderError` exhaustiveness
3. Prove panic-freedom in error propagation
4. Verify hook execution ordering

**Files to annotate**:
- `crates/universal-decoder-core/src/ir.rs`
- `crates/universal-decoder-core/src/traits.rs`
- `crates/universal-decoder-core/src/error.rs`

### Phase 2: Bitcoin Decoder Verification (6-8 weeks)

**Target**: `decoder-bitcoin`

1. Verify UTXO parsing bounds
2. Prove output value overflow checks
3. Verify canonicalization injectivity
4. Prove script parsing safety

**Critical functions**:
- `BitcoinTransaction::from_bitcoin_tx`
- `BitcoinTransaction::canonicalize`
- `BitcoinTransaction::calculate_fee`

### Phase 3: Ethereum Decoder Verification (6-8 weeks)

**Target**: `decoder-ethereum`

1. Verify RLP decoding bounds
2. Prove gas calculation safety
3. Verify account state transitions
4. Prove EIP-1559 fee calculations

### Phase 4: Cross-Chain Properties (4 weeks)

1. Verify hook system properties
2. Prove decoder composition safety
3. Verify batch decoding correctness

## Verus-Specific Implementation

### Setting Up Verus

```bash
# Install Verus
git clone https://github.com/verus-lang/verus.git
cd verus && ./tools/get-z3.sh && source tools/activate

# Add to Cargo.toml
[dependencies]
builtin = { git = "https://github.com/verus-lang/verus" }
builtin_macros = { git = "https://github.com/verus-lang/verus" }
```

### Example Annotated Code

```rust
use builtin::*;
use builtin_macros::*;

verus! {

// Specify the function's mathematical specification
pub fn decode_transaction_length(bytes: &[u8]) -> (result: usize)
    requires
        bytes.len() >= 4,  // Precondition: minimum header size
    ensures
        result <= bytes.len(),  // Postcondition: length is valid
        result >= 4,  // Postcondition: at least header
{
    // Implementation with verification conditions
    let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

    proof {
        // Manual proof that array access is safe
        assert(0 < bytes.len());
        assert(3 < bytes.len());
    }

    // Return verified result
    4 + calculate_body_length(version)
}

#[verifier::invariant]
pub fn valid_bitcoin_transaction(tx: &BitcoinTransaction) -> bool {
    tx.inner.version.0 >= 1 &&
    tx.inner.input.len() > 0 &&
    tx.inner.output.len() > 0 &&
    tx.raw_bytes.len() >= 10 &&
    tx.raw_bytes.len() <= 1_000_000  // Max 1MB
}

} // verus!
```

### Proving Canonicalization Injectivity

```rust
verus! {

#[verifier::proof]
pub fn canonicalization_preserves_hash(
    tx_bytes: &[u8]
) -> (result: Result<(), DecoderError>)
    requires
        BitcoinDecoder::validate_format(tx_bytes).is_ok(),
    ensures
        result.is_ok() ==> {
            let tx = BitcoinDecoder::decode(tx_bytes).unwrap();
            let ir = tx.canonicalize().unwrap();
            let hash1 = tx.compute_hash();
            let hash2 = DoubleSha256::hash(&ir.to_canonical_bytes());
            hash1 == hash2
        }
{
    // Verification proof
    let tx = BitcoinDecoder::decode(tx_bytes)?;

    proof {
        // Prove that canonicalize preserves the transaction bytes
        assert(tx.raw_bytes == tx_bytes);
        assert(tx.to_canonical_bytes() == tx.raw_bytes);
    }

    let ir = tx.canonicalize()?;

    proof {
        // Prove that TxIR reconstructs the same bytes
        assert(ir.metadata.tx_hash == tx.compute_hash());
    }

    Ok(())
}

} // verus!
```

## Alternative: F* Approach

### When to use F*:

F* is better if you want to:
1. **Extract to multiple languages** (OCaml, F#, C)
2. **Prove deep cryptographic properties**
3. **Verify protocol-level security** (not just implementation)

### F* Implementation Strategy:

1. **Extract Core Logic to F***:
   - Define TxIR as F* types
   - Implement canonicalization in F*
   - Verify and extract to Rust via C FFI

```fstar
module UniversalDecoder.TxIR

type tx_ir = {
  chain_id: chain_id;
  metadata: tx_metadata;
  operations: list operation;
  state_deltas: state_deltas;
}

// Prove injectivity as a lemma
let lemma_canonicalization_injective
  (tx1 tx2: raw_transaction)
  : Lemma
    (requires (canonicalize tx1 == canonicalize tx2))
    (ensures (tx1 == tx2))
  = admit() // Proof here
```

2. **FFI Boundary**:
   - Keep parsing in Rust (complex, not critical)
   - Verify canonicalization in F*
   - Extract verified F* to C
   - Call from Rust via FFI

**Pros**: Stronger guarantees, proven at higher level
**Cons**: FFI overhead, more complex setup, dual codebase

## Practical Recommendation

### **Start with Verus** (Recommended)

**Why**:
1. ✅ Native Rust - no FFI overhead
2. ✅ Verify actual production code
3. ✅ Better tooling for systems programming
4. ✅ Can verify lifetimes and ownership
5. ✅ Active community support

**Prioritized Roadmap**:

**Month 1-2: Foundation**
- Set up Verus toolchain
- Annotate core traits
- Prove basic panic-freedom

**Month 3-4: Bitcoin Decoder**
- Verify UTXO parsing
- Prove overflow safety
- Verify canonicalization

**Month 5-6: Ethereum + Integration**
- Verify Ethereum decoder
- Prove cross-chain properties
- CI/CD integration

### **Consider F* if**:
- You need protocol-level security proofs
- You want to extract to multiple languages
- You're proving cryptographic primitives
- You have F* expertise on team

## Cost-Benefit Analysis

### Benefits:
- **Security Assurance**: Mathematical proof of correctness
- **Bug Prevention**: Catch errors at verification time
- **Documentation**: Specifications serve as formal docs
- **Confidence**: Provable supply chain security

### Costs:
- **Time**: 4-6 months for comprehensive verification
- **Learning Curve**: Team needs Verus/F* expertise
- **Maintenance**: Keep specs in sync with code

### ROI Calculation:
For a blockchain decoder used in:
- Exchanges: $100M+ at risk → **HIGH ROI**
- Indexers: Critical infrastructure → **HIGH ROI**
- Research: Academic contribution → **MEDIUM ROI**
- Personal project: Learning value → **LOW-MEDIUM ROI**

## Incremental Approach

Don't verify everything at once:

### Quick Wins (1-2 weeks):
1. Prove panic-freedom in amount calculations
2. Verify array bounds in parsing
3. Prove overflow checks in fee calculations

### Medium Effort (1-2 months):
1. Verify Bitcoin canonicalization injectivity
2. Prove hook execution properties
3. Verify batch decoding correctness

### Full Verification (4-6 months):
1. Complete Bitcoin + Ethereum verification
2. Cross-chain property proofs
3. Cryptographic property verification

## Getting Started

### Step 1: Install Verus (Today)

```bash
cd /tmp
git clone https://github.com/verus-lang/verus.git
cd verus
./tools/get-z3.sh
source tools/activate
```

### Step 2: Create Verification Branch

```bash
cd /home/user/universal-blockchain-decoder
git checkout -b feature/verus-verification
```

### Step 3: Add First Annotation (1 hour)

Start with a simple function to learn Verus:

```rust
// In crates/universal-decoder-core/src/ir.rs
verus! {

impl Amount {
    #[verifier::proof]
    pub fn checked_add(self, other: Amount) -> (result: Option<Amount>)
        ensures
            result.is_some() ==> {
                let sum = result.unwrap();
                sum.value == self.value + other.value &&
                sum.decimals == self.decimals
            },
            result.is_none() ==> {
                self.value + other.value > u128::MAX
            }
    {
        if self.decimals != other.decimals {
            return None;
        }

        match self.value.checked_add(other.value) {
            Some(sum) => Some(Amount {
                value: sum,
                decimals: self.decimals,
            }),
            None => None,
        }
    }
}

} // verus!
```

### Step 4: Verify It

```bash
verus crates/universal-decoder-core/src/ir.rs
```

## Conclusion

**Recommendation**: **Start with Verus for incremental verification**

1. Begin with critical arithmetic operations (quick wins)
2. Move to canonicalization injectivity (core property)
3. Expand to full decoder verification (comprehensive)
4. Consider F* only if extracting to multiple targets

The project is **well-suited for formal verification** because:
- Clear security properties to prove
- Modular architecture with trait boundaries
- Type-driven design already enforces many invariants
- High-value target (blockchain security)

**Next Action**: Install Verus and verify a simple function (Amount arithmetic) to evaluate feasibility.
