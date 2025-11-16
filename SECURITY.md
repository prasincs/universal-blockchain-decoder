# Security Policy

## Overview

The Universal Blockchain Decoder is designed with security as a core principle. However, this is experimental software (v0.1.0-alpha) and **has not yet been audited for production use**.

## Supported Versions

We provide security updates for the following versions:

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| 0.1.x   | :white_check_mark: | Alpha - Active Development |
| < 0.1   | :x:                | Not Released |

**Note**: As this is an alpha release, we may make breaking changes to address security issues.

## Security Priorities

The Universal Blockchain Decoder prioritizes security through:

1. **Memory Safety** - Leveraging Rust's ownership model to prevent common vulnerabilities:
   - No buffer overflows
   - No use-after-free
   - No data races (in safe code)
   - No null pointer dereferences

2. **Canonical Encoding** - Prevents transaction malleability attacks:
   - Deterministic serialization using Borsh
   - Consistent hashing across all transaction types
   - No JSON for cryptographic operations

3. **Supply Chain Security** - Minimal attack surface:
   - Only 5 production dependencies (serde, borsh, thiserror, sha2, sha3)
   - Dependency vendoring via git subtree for verifiable supply chain
   - `cargo-audit` runs on every commit in CI
   - Designed for airgapped operation (no runtime network dependencies)

4. **Type Safety** - Compile-time guarantees:
   - Const generics for version constraints
   - Associated types for chain-specific logic
   - Zero-cost abstractions via static dispatch

5. **Formal Verification** - Currently in progress:
   - Verus annotations for critical paths
   - Property-based testing (100+ tests with proptest)
   - Panic-freedom verification
   - Determinism proofs

## Known Limitations

### Alpha Software Warnings

⚠️ **DO NOT USE IN PRODUCTION** without thorough review and additional testing.

This software is in active development (v0.1.0-alpha) and has the following limitations:

1. **Not Audited** - No professional security audit has been conducted
2. **API Instability** - Public APIs may change without notice
3. **Incomplete Coverage** - Some decoders are still in development
4. **Limited Testing** - While we have 322 unit tests and 100+ property tests, real-world coverage is incomplete

### Scope Limitations

This project is **decoding-only** and does NOT support:
- ❌ Transaction encoding/construction
- ❌ Transaction signing
- ❌ Key management
- ❌ Wallet functionality

For these features, use established, audited libraries like:
- Bitcoin: `bitcoin` crate, BDK (Bitcoin Dev Kit)
- Ethereum: `ethers-rs`, `alloy`
- Solana: `solana-sdk`

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please follow responsible disclosure:

### DO NOT

- ❌ Open a public GitHub issue
- ❌ Post about it on social media or public forums
- ❌ Share details with anyone not involved in fixing it

### DO

1. **Email** the security team: **[INSERT SECURITY EMAIL HERE]**
   - Use subject line: `[SECURITY] Brief description`
   - Include "Universal Blockchain Decoder" in the body

2. **Provide Details**:
   - Description of the vulnerability
   - Steps to reproduce (proof of concept)
   - Affected versions
   - Potential impact assessment
   - Any suggested fixes (optional)

3. **Use Encryption** (optional but recommended):
   - PGP Key: [INSERT PGP KEY ID/FINGERPRINT HERE]
   - Public key available at: [INSERT KEY SERVER URL]

### What to Report

Please report any issues that could lead to:

**High Severity:**
- Memory safety violations (unsafe code bugs)
- Denial of service through malformed input
- Transaction malleability (hash collisions, non-deterministic encoding)
- Incorrect decoding that could lead to loss of funds
- Supply chain attacks (compromised dependencies)

**Medium Severity:**
- Resource exhaustion (CPU, memory)
- Parser bugs causing incorrect interpretation
- Cryptographic implementation flaws
- Privacy leaks

**Low Severity:**
- Minor correctness issues
- Performance degradation
- Documentation errors about security properties

### Response Timeline

We strive for the following response times:

- **Initial Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Regular Updates**: Every 7-14 days until resolved
- **Fix Timeline**: Depends on severity
  - Critical: 1-7 days
  - High: 7-30 days
  - Medium: 30-90 days
  - Low: Best effort

### Disclosure Policy

We follow **coordinated disclosure**:

1. **Private Fix** - We develop and test a fix privately
2. **Embargo Period** - 90 days or until fix is released (whichever is sooner)
3. **Public Disclosure** - After fix is released, we publish:
   - Security advisory (GitHub Security Advisories)
   - CVE (if applicable)
   - Credit to reporter (unless anonymity requested)
   - Detailed write-up in release notes

### Bug Bounty

Currently, we do **not** have a formal bug bounty program. However:

- We will publicly acknowledge security researchers (unless anonymity requested)
- We may offer rewards for significant vulnerabilities on a case-by-case basis
- All reporters will be credited in:
  - Security advisories
  - Release notes
  - AUTHORS.md
  - Blog posts about the vulnerability (if written)

## Security Best Practices for Users

If you're using Universal Blockchain Decoder:

### Development

1. **Dependency Management**:
   ```bash
   # Regular security audits
   cargo audit

   # Keep dependencies updated
   cargo update
   ```

2. **Enable Security Features**:
   ```toml
   [dependencies]
   universal-decoder-core = { version = "0.1", features = [] }

   [profile.release]
   overflow-checks = true
   ```

3. **Input Validation**:
   ```rust
   // Always handle errors, never unwrap() in production
   match BitcoinDecoder::decode(untrusted_bytes) {
       Ok(tx) => { /* process */ },
       Err(e) => { /* handle error safely */ },
   }
   ```

### Production Deployment

1. **Isolate Decoder**:
   - Run in sandboxed environment
   - Limit resource usage (CPU, memory, time)
   - Use process isolation for untrusted input

2. **Monitor for Issues**:
   - Watch GitHub repository for security advisories
   - Subscribe to release notifications
   - Join security mailing list (when available)

3. **Defense in Depth**:
   - Don't rely solely on the decoder for validation
   - Implement additional checks for critical operations
   - Use established libraries for signing/broadcasting

4. **Incident Response**:
   - Have a plan for security updates
   - Test updates in staging before production
   - Monitor for unexpected behavior after updates

## Security Roadmap

Planned security improvements:

### v0.2.0 (Q2 2025)
- [ ] Professional security audit (external firm)
- [ ] Fuzzing infrastructure improvements
- [ ] Increased test coverage (80%+ target)
- [ ] More Verus formal verification annotations

### v0.3.0 (Q3 2025)
- [ ] Complete formal verification of core library
- [ ] Security documentation improvements
- [ ] Threat model documentation
- [ ] Security benchmarks

### v1.0.0 (Q4 2025)
- [ ] Second external audit
- [ ] Bug bounty program
- [ ] Production-ready security certification
- [ ] Comprehensive security guide

## References

- **Rust Security Guidelines**: https://anssi-fr.github.io/rust-guide/
- **OWASP**: https://owasp.org/
- **CVE Database**: https://cve.mitre.org/
- **Cargo Audit**: https://github.com/rustsec/rustsec
- **Verus**: https://github.com/verus-lang/verus

## Security Team

- **Current Maintainer**: [INSERT NAME/GITHUB]
- **Security Contact**: [INSERT SECURITY EMAIL]
- **PGP Key**: [INSERT PGP FINGERPRINT]

## Acknowledgments

We thank the security research community for helping keep Universal Blockchain Decoder secure. Security researchers who have helped:

- [Will be listed here as vulnerabilities are reported and fixed]

---

**Questions about security?** Please contact: [INSERT SECURITY EMAIL]

**General questions?** Open a GitHub Discussion (not for security issues!)
