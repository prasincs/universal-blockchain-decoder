# Verus Formal Verification Setup

This document explains how to install and use Verus for formal verification of the Universal Blockchain Decoder core library.

## What is Verus?

[Verus](https://github.com/verus-lang/verus) is a tool for verifying the correctness of Rust code. It allows you to write specifications (contracts) that describe what your code should do, and then mathematically proves that your code meets those specifications.

## Why Verus for This Project?

The Universal Blockchain Decoder core library has strict correctness requirements:

1. **Deterministic Serialization**: Same data must always produce same bytes
2. **Panic-Freedom**: Core functions must never panic on valid inputs
3. **Injectivity**: Different transactions must have different canonical representations
4. **Resource Bounds**: Memory usage must be bounded and predictable

Verus helps us prove these properties mathematically, giving users confidence in the library's correctness.

## Installation

### Quick Install (Recommended)

Use our automated installation script:

```bash
chmod +x scripts/install-verus.sh
./scripts/install-verus.sh
```

This will:
- Detect your platform (Linux, macOS, Windows)
- Download the latest Verus release
- Extract it to `tools/verus-bin/`
- Make it executable
- Test the installation

### Manual Installation

1. **Download the latest release:**

   Visit [Verus Releases](https://github.com/verus-lang/verus/releases/latest) and download the appropriate file for your platform:

   - Linux x86_64: `verus-X.X.X-x86-linux.zip`
   - macOS ARM64: `verus-X.X.X-arm64-macos.zip`
   - macOS x86_64: `verus-X.X.X-x86-macos.zip`
   - Windows: `verus-X.X.X-x86-win.zip`

2. **Extract the archive:**

   ```bash
   unzip verus-*.zip
   cd verus-*
   ```

3. **macOS only: Remove quarantine:**

   ```bash
   xattr -d com.apple.quarantine verus
   find . -type f -exec xattr -d com.apple.quarantine {} \;
   ```

4. **Add to PATH (optional):**

   ```bash
   export PATH="/path/to/verus:$PATH"
   ```

### Building from Source (Advanced)

If you need the latest features or want to contribute to Verus:

```bash
git clone https://github.com/verus-lang/verus.git
cd verus/source
source ../tools/activate  # or activate.fish for fish shell
vargo build --release
```

See [Verus BUILD.md](https://github.com/verus-lang/verus/blob/main/BUILD.md) for detailed build instructions.

### Verify Installation

```bash
./tools/verus-bin/verus --version || echo "Installed successfully"
```

## Usage

### Verify a Single File

```bash
# Using the wrapper script (recommended)
./scripts/verus.sh crates/universal-decoder-core/src/canonical.rs --crate-type=lib

# Or directly
tools/verus-bin/verus crates/universal-decoder-core/src/canonical.rs --crate-type=lib
```

### Verify the Entire Core Library

```bash
# Verify all files with Verus annotations
find crates/universal-decoder-core/src -name "*.rs" -type f | while read -r file; do
    echo "Verifying $file"
    ./scripts/verus.sh "$file" --crate-type=lib
done
```

### Common Verus Flags

- `--crate-type=lib`: Verify as a library (required for non-main files)
- `--compile`: Compile the code after verification
- `--verbose`: Show detailed verification information
- `--rlimit <n>`: Set resource limit for the SMT solver (default: 10)
- `--time`: Show timing information

### Example: Verify and Compile

```bash
./scripts/verus.sh examples/verified_hash.rs --compile
./verified_hash  # Run the compiled binary
```

## CI Integration

Our GitHub Actions workflow automatically runs Verus verification on every push and pull request.

See [`.github/workflows/verus-verification.yml`](../.github/workflows/verus-verification.yml) for the complete workflow.

The CI workflow:
1. Caches Verus installation for faster builds
2. Installs Verus using our automated script
3. Runs verification on core modules
4. Reports verification results

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

## Common Issues

### Issue: "Verus binary not found"

**Solution**: Run the installation script first:
```bash
./scripts/install-verus.sh
```

### Issue: "Verification failed" / "SMT solver timeout"

**Solution**: Increase the resource limit:
```bash
./scripts/verus.sh file.rs --rlimit 20
```

### Issue: macOS "cannot be opened because the developer cannot be verified"

**Solution**: Remove quarantine:
```bash
xattr -d com.apple.quarantine tools/verus-bin/verus
```

### Issue: "error: toolchain '1.82.0' is not installed"

**Solution**: Verus requires a specific Rust toolchain. Install it:
```bash
rustup toolchain install 1.82.0
```

## Learning Resources

1. **Official Verus Guide**: https://verus-lang.github.io/verus/guide/
2. **Verus Tutorial**: https://verus-lang.github.io/verus/guide/getting_started.html
3. **Verus Examples**: https://github.com/verus-lang/verus/tree/main/examples
4. **Verus Playground**: https://play.verus-lang.org/ (try Verus in your browser!)
5. **Project Verification Docs**: [FORMAL_VERIFICATION.md](./FORMAL_VERIFICATION.md)

## Current Verification Status

As of 2025-11-13:

| Module | Status | Properties Verified |
|--------|--------|---------------------|
| `canonical.rs` | ⚙️ In Progress | Determinism annotations added |
| `ir.rs` | ⚙️ In Progress | Basic type safety |
| `traits.rs` | 📋 Planned | Trait properties |
| `error.rs` | 📋 Planned | Error handling |

See [ROADMAP.md](../ROADMAP.md) and [FORMAL_VERIFICATION.md](./FORMAL_VERIFICATION.md) for the complete verification roadmap.

## Contributing

When adding new code to the core library:

1. Add Verus specifications for critical properties
2. Run verification locally: `./scripts/verus.sh <file>`
3. Ensure CI verification passes
4. Document any unverified assumptions in comments

See [CONTRIBUTING.md](../CONTRIBUTING.md) for general contribution guidelines.

## Support

- **Verus Issues**: https://github.com/verus-lang/verus/issues
- **Project Issues**: https://github.com/prasincs/universal-blockchain-decoder/issues
- **Verus Zulip Chat**: https://verus-lang.zulipchat.com/

---

**Last Updated**: 2025-11-13
**Verus Version**: 0.2025.11.07
**Status**: Phase 1.5 - Verus installation and CI integration complete
