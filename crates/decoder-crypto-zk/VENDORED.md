# Vendored Dependencies Audit Trail

This document tracks all vendored code in the `decoder-crypto-zk` crate.

## Purpose

Vendoring cryptographic primitives serves multiple critical security goals:

1. **Minimal TCB (Trusted Computing Base)**: Reduces external dependencies to absolute minimum
2. **Airgapped Operation**: Enables complete offline operation for security-critical deployments
3. **Formal Verification**: Allows Verus verification of vendored code
4. **Security Audit**: Single audit point vs multiple external crates
5. **Supply Chain Security**: Verifiable git commit audit trail
6. **Reproducible Builds**: All code in repository, no runtime network dependencies

## Vendoring Strategy

We use **git subtree** for verifiable vendoring:

```bash
git subtree add \
    --prefix crates/decoder-crypto-zk/vendored/starknet-crypto \
    https://github.com/xJonathanLEI/starknet-rs.git \
    master --squash
```

This provides:
- ✅ Exact upstream commit tracked in git history
- ✅ Can verify: `git diff <commit> -- vendored/starknet-crypto`
- ✅ Reproducible builds (all data in repo)
- ✅ No TOCTOU attacks (data can't change at runtime)

## Vendored Libraries

### 1. starknet-rs (starknet-crypto subsystem)

**Source**: https://github.com/xJonathanLEI/starknet-rs
**Commit**: See git history (`git log --oneline -- crates/decoder-crypto-zk/vendored/`)
**Date Vendored**: 2025-11-15
**License**: MIT OR Apache-2.0
**Components Used**:
- `starknet-crypto/`: Poseidon, Pedersen, ECDSA
- `starknet-curve/`: STARK curve parameters

**What We Extract**:
- ✅ Poseidon hash (Hades permutation)
- 🚧 Pedersen hash (elliptic curve based)
- 📋 ECDSA verification (STARK curve)
- 📋 Curve primitives

**Dependencies of Vendored Code**:
The vendored code depends on:
- `starknet-types-core`: Field arithmetic (252-bit STARK field)
  - **Decision**: Use as dependency (not vendored)
  - **Rationale**: Foundation library, battle-tested, minimal
  - **Future**: Can vendor if needed for formal verification
- `crypto-bigint`, `num-bigint`: Big integer arithmetic
  - **Status**: Production dependencies (required for crypto)
- `hmac`, `rfc6979`, `sha2`: ECDSA dependencies
  - **Status**: Workspace dependencies (standard crypto primitives)

**Verification**:
```bash
# Check original source
cd crates/decoder-crypto-zk/vendored/starknet-crypto
git log --oneline | head -5

# Verify no modifications
git diff <upstream-commit> -- crates/decoder-crypto-zk/vendored/starknet-crypto/starknet-crypto/
```

**Audit Status**:
- 🟢 **starknet-crypto**: Widely used in Starknet ecosystem, audited
- 🟢 **starknet-types-core**: Foundation library, reviewed by community

## Implementation Approach

### Hybrid Vendoring Strategy

We use a **pragmatic hybrid approach**:

1. **Vendor the Algorithm Implementations**:
   - Poseidon permutation logic ✅
   - Pedersen hash computation 🚧
   - ECDSA verification logic 📋

2. **Use Well-Audited Foundation Libraries**:
   - `starknet-types-core` for STARK field arithmetic
   - Reason: Like using `std` or `serde` - foundational, battle-tested
   - Can vendor later if formal verification requires it

3. **Create Clean Public API**:
   - Our modules: `decoder_crypto_zk::field`, `decoder_crypto_zk::hash`
   - Re-exports: Clean, documented interface
   - Tests: Property tests + cross-validation with reference impl

### Benefits of This Approach

1. **Faster Implementation**: Leverage existing field arithmetic (1 week vs 3 weeks)
2. **Battle-Tested**: starknet-types-core used by entire Starknet ecosystem
3. **Verification Path**: Can verify wrapper code + foundation separately
4. **Flexibility**: Can vendor foundation later if TCB requirements change

## Cross-Validation Strategy

All vendored implementations are cross-validated against reference implementations:

```toml
[dev-dependencies]
# For cross-validation ONLY, not in production
starknet-crypto = "0.7"
```

**Test Strategy**:
1. **Unit Tests**: Test our wrapper API
2. **Property Tests**: Verify mathematical properties (commutativity, determinism)
3. **Test Vectors**: 100+ test vectors from Starknet docs
4. **Cross-Validation**: Compare outputs with `starknet-crypto` crate

Example cross-validation test:
```rust
#[cfg(test)]
mod cross_validation {
    use starknet_crypto as reference;
    use decoder_crypto_zk::hash::PoseidonHash;

    #[test]
    fn validate_against_reference() {
        let a = FieldElement::from(123u64);
        let b = FieldElement::from(456u64);

        let our_hash = PoseidonHash::hash_pair(a, b);
        let ref_hash = reference::poseidon_hash(a, b);

        assert_eq!(our_hash, ref_hash);
    }
}
```

## Update Procedure

To update vendored code to newer version:

```bash
# 1. Fetch new version
git subtree pull \
    --prefix crates/decoder-crypto-zk/vendored/starknet-crypto \
    https://github.com/xJonathanLEI/starknet-rs.git \
    <new-version-tag> --squash

# 2. Review changes
git diff HEAD~1 -- crates/decoder-crypto-zk/vendored/

# 3. Update tests if API changed
cargo test -p decoder-crypto-zk

# 4. Update this document
# - New commit hash
# - Date updated
# - Breaking changes (if any)

# 5. Commit
git commit -m "chore(crypto): Update vendored starknet-crypto to <version>"
```

## Security Audit Checklist

When auditing vendored cryptographic code:

- [ ] Verify git subtree commit matches upstream
- [ ] Check for unsafe code blocks
- [ ] Review arithmetic operations for overflow
- [ ] Validate constant-time operations where required
- [ ] Test against known test vectors
- [ ] Cross-validate with reference implementation
- [ ] Review error handling (no panics in production)
- [ ] Check for side-channel vulnerabilities

## License Compliance

All vendored code is licensed under **MIT OR Apache-2.0**, compatible with this project's license.

**Attribution**:
- starknet-rs by Jonathan LEI: https://github.com/xJonathanLEI/starknet-rs
- Licensed under MIT OR Apache-2.0

## Version History

### v0.1.0 (2025-11-15) - Initial Vendoring

**Added**:
- Vendored `starknet-rs` repository (full monorepo)
- Extracted `starknet-crypto/` crate
- Extracted `starknet-curve/` crate

**Components Integrated**:
- ✅ Poseidon hash (fully working)
- 🚧 Pedersen hash (API defined, implementation in progress)
- 📋 ECDSA verification (planned)
- 📋 Curve primitives (planned)

**Dependencies**:
- `starknet-types-core 0.2.4`: Field arithmetic
- `crypto-bigint 0.5.1`: Big integer support
- `num-bigint 0.4.3`, `num-integer 0.1.45`, `num-traits 0.2.18`: Number utilities
- `hmac 0.12.1`, `rfc6979 0.4.0`, `sha2 0.10.9`: ECDSA dependencies

**Status**: Week 1 of Phase 3.6a complete
- Crate infrastructure: ✅ Complete
- Field arithmetic wrapper: ✅ Complete
- Poseidon hash: ✅ Complete
- Pedersen hash: 🚧 In Progress
- STARK curve: 📋 Planned (Week 2)
- ECDSA: 📋 Planned (Week 2)

---

**Maintained By**: Universal Blockchain Decoder Team
**Last Updated**: 2025-11-15
**Next Review**: When adding new vendored components or updating existing ones
