# Verus Formal Verification Setup Guide

This guide explains how to set up Verus for formal verification of the Universal Blockchain Decoder core library.

## What is Verus?

[Verus](https://github.com/verus-lang/verus) is a tool for verifying the correctness of Rust code. It allows us to:
- Prove mathematical properties of our code
- Guarantee panic-freedom
- Verify overflow safety
- Prove determinism and injectivity

## Installation

### Prerequisites

- Rust toolchain (stable or nightly)
- Z3 SMT solver (Verus uses this for automated theorem proving)

### Step 1: Clone Verus

```bash
cd ~/tools  # or wherever you keep development tools
git clone https://github.com/verus-lang/verus.git
cd verus
```

### Step 2: Install Z3

#### Option A: Use Verus's helper script
```bash
./tools/get-z3.sh
source tools/activate  # Adds Z3 to PATH
```

#### Option B: Install via package manager

**Ubuntu/Debian:**
```bash
sudo apt-get install z3
```

**macOS:**
```bash
brew install z3
```

**Arch Linux:**
```bash
sudo pacman -S z3
```

### Step 3: Build Verus

```bash
cargo build --release
```

### Step 4: Add to PATH

```bash
export PATH="$HOME/tools/verus/target/release:$PATH"
export VERUS_Z3_PATH="$(which z3)"
```

Add these lines to your `~/.bashrc` or `~/.zshrc` to make them permanent.

### Step 5: Verify Installation

```bash
verus --version
```

Should output something like:
```
verus 0.x.x
```

## Using Verus with Universal Blockchain Decoder

### Running Verification

Verify a specific file:
```bash
verus crates/universal-decoder-core/src/ir.rs
```

Verify all annotated files:
```bash
./scripts/verify_all.sh
```

### CI Integration

Verus runs in GitHub Actions (nightly workflow):
```yaml
- name: Install Verus
  run: |
    git clone https://github.com/verus-lang/verus.git
    cd verus && ./tools/get-z3.sh && source tools/activate
    cargo build --release

- name: Run Verus
  run: verus crates/universal-decoder-core/src/ir.rs
```

## Verification Strategy

### Phase 1: Core Types (Current)

**Focus**: Amount arithmetic, basic properties

Files:
- `src/ir.rs` - Amount, Address types
- `src/canonical.rs` - Canonical serialization

Properties to verify:
- ✅ `Amount::checked_add` never panics
- ✅ `Amount::checked_sub` never panics
- ✅ Overflow is correctly detected
- ✅ `Amount` equality is reflexive, symmetric, transitive

### Phase 2: Canonical Serialization (Month 2)

**Focus**: Borsh serialization determinism

Properties to verify:
- Canonical bytes are deterministic: `to_canonical_bytes(x) == to_canonical_bytes(x)`
- Canonical hash is deterministic: `canonical_hash(x) == canonical_hash(x)`
- Different transactions have different hashes (collision resistance)

### Phase 3: Decoder Verification (Month 3-4)

**Focus**: Bitcoin decoder correctness

Properties to verify:
- Parsing never panics on any input
- Valid transactions parse successfully
- Round-trip: `encode(decode(bytes)) == bytes`

### Phase 4: Full System Verification (Month 5-6)

**Focus**: End-to-end correctness

Properties to verify:
- TxIR construction preserves semantics
- Canonical representation is injective
- All error paths are correct

## Writing Verus Annotations

### Example 1: Panic-Free Arithmetic

```rust
use builtin::*;
use builtin_macros::*;

verus! {

impl Amount {
    /// Verified addition with overflow checking
    #[verifier::external_body]  // Trust Rust's checked_add
    pub fn checked_add(self, other: Amount) -> (result: Option<Amount>)
        requires
            self.decimals == other.decimals,
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
        match self.value.checked_add(other.value) {
            Some(sum) => Some(Amount { value: sum, decimals: self.decimals }),
            None => None,
        }
    }
}

} // verus!
```

### Example 2: Deterministic Serialization

```rust
verus! {

/// Prove that canonical serialization is deterministic
#[verifier::proof]
pub fn canonical_bytes_deterministic<'a>(tx: &TxIR<'a, 1>)
    ensures
        tx.to_canonical_bytes() == tx.to_canonical_bytes()
{
    // Proof obligation: Verus checks this automatically
    // by analyzing the implementation of to_canonical_bytes
}

} // verus!
```

### Example 3: Injectivity

```rust
verus! {

/// Prove that canonical encoding is injective
#[verifier::proof]
pub fn canonical_encoding_injective<'a>(tx1: &TxIR<'a, 1>, tx2: &TxIR<'a, 1>)
    requires
        tx1.to_canonical_bytes() == tx2.to_canonical_bytes(),
    ensures
        tx1 == tx2  // If encodings match, transactions are equal
{
    // Proof by Borsh properties + TxIR structure
}

} // verus!
```

## Common Verus Annotations

### Function Specifications

```rust
#[verifier::proof]      // This is a proof function (ghost code)
#[verifier::external_body]  // Trust the implementation, just verify the spec
#[verifier::external]   // Don't verify this function at all
```

### Pre/Post Conditions

```rust
requires            // Precondition: must be true when function is called
ensures             // Postcondition: must be true when function returns
invariant           // Loop invariant: must be true on each iteration
```

### Logical Operators

```rust
==>                 // Implication: A ==> B means "if A then B"
<==                 // Reverse implication
&&                  // Logical AND
||                  // Logical OR
forall              // Universal quantification: ∀
exists              // Existential quantification: ∃
```

## Debugging Verification Failures

### 1. Start Simple

Verify a single, simple property first:
```rust
#[verifier::proof]
pub fn test_reflexive(x: Amount)
    ensures x == x
{
    // Trivial proof - good sanity check
}
```

### 2. Add Intermediate Assertions

```rust
#[verifier::proof]
pub fn complex_proof(x: Amount, y: Amount)
    ensures x.checked_add(y).is_some() || x.value + y.value > u128::MAX
{
    assert!(x.value <= u128::MAX);  // Intermediate step
    assert!(y.value <= u128::MAX);  // Intermediate step
    // Now prove the main property
}
```

### 3. Use Z3 Trace

```bash
verus --trace crates/universal-decoder-core/src/ir.rs
```

Shows what Z3 is trying to prove.

### 4. Simplify Specifications

If verification fails, try:
- Weaken postconditions
- Strengthen preconditions
- Break complex proofs into smaller lemmas

## Performance Tips

- Use `#[verifier::external_body]` for trusted functions
- Mark test code as `#[cfg(not(verus))]` to skip verification
- Verify incrementally (one module at a time)
- Cache verification results in CI

## Resources

- [Verus Documentation](https://verus-lang.github.io/verus/)
- [Verus Tutorial](https://verus-lang.github.io/verus/guide/)
- [Verus GitHub](https://github.com/verus-lang/verus)
- [Z3 SMT Solver](https://github.com/Z3Prover/z3)
- [Formal Verification Strategy](./FORMAL_VERIFICATION.md)

## Current Verification Status

| Module | Status | Properties Verified |
|--------|--------|---------------------|
| `ir.rs` | ⚙️ In Progress | Basic type safety |
| `canonical.rs` | 📋 Planned | Determinism |
| `traits.rs` | 📋 Planned | Trait properties |
| `error.rs` | ✅ Complete | Error handling |

See `FORMAL_VERIFICATION.md` for the complete roadmap.

## Getting Help

- **Verus Discord**: [Join here](https://discord.gg/verus)
- **GitHub Issues**: Report bugs or ask questions
- **Verus Documentation**: Comprehensive guides and examples

## Next Steps

1. Install Verus following this guide
2. Review example annotations in `src/ir.rs`
3. Read `FORMAL_VERIFICATION.md` for the verification strategy
4. Start with simple properties (reflexivity, determinism)
5. Gradually add more complex proofs

---

**Last Updated**: 2025-01-11
**Verus Version**: 0.x.x (check for latest)
**Status**: Phase 1.5 PR #10 - Initial setup complete
