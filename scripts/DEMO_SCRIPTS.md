# Demo Scripts

This directory contains demonstration scripts that showcase the Universal Blockchain Decoder's testing and verification infrastructure.

## 📊 Verification Coverage Dashboard ⭐ RECOMMENDED

**Script**: `./scripts/verification-dashboard.sh`

**Purpose**: **Comprehensive dashboard showing coverage, impact, and remaining work** - Best for presentations!

**What it shows**:
- ✅ **Part 1: COVERAGE** - What's been verified (67/67 VCs in Phase 4.1)
- 💎 **Part 2: IMPACT** - Security guarantees proven (5 critical properties)
- 📋 **Part 3: WHAT'S LEFT** - Remaining work (118 VCs across Phase 4.2 & 4.3)
- 🎯 Overall progress with visual bars (36% complete)
- ⏱️ Timeline and effort estimates

**Usage**:
```bash
./scripts/verification-dashboard.sh
```

**Output**: Beautiful, comprehensive dashboard with:
- Progress bars for each verification target
- Tree-view breakdown of VCs
- Security impact analysis
- Timeline estimates
- File references

**Perfect for**: Papers, presentations, demos, stakeholder updates

---

## 🔬 Verus Formal Verification Demo

**Script**: `./scripts/demo-verus.sh`

**Purpose**: Demonstrates Verus formal verification infrastructure and proven properties.

**What it shows**:
- ✅ 67 Verification Conditions (VCs) across 5 modules
- ✅ Critical security properties proven mathematically
- ✅ Verus annotations in the codebase
- ✅ Runtime property tests that validate verified properties
- ✅ CI/CD integration for continuous verification

**Key Properties Proven**:
1. **Panic-Freedom**: Core library never panics on valid inputs
2. **Deterministic Serialization**: Same TX → same bytes (critical for signatures)
3. **Injectivity**: Different TX → different bytes (no collisions)
4. **Overflow Safety**: Arithmetic never overflows silently
5. **Type Safety**: Version isolation at compile time

**Usage**:
```bash
./scripts/demo-verus.sh
```

**Output**: Colorful, informative display of:
- Verus installation status
- Verification targets and their locations in code
- Example proof annotations
- Property test results
- Verification documentation references
- CI/CD integration details

**Requirements**: None! Works with or without Verus installed.
- If Verus is installed: Runs actual formal verification
- If not installed: Shows annotations and documentation

**Install Verus** (optional):
```bash
./scripts/install-verus.sh
```

---

## 🧪 Comprehensive Test Suite Demo

**Script**: `./scripts/demo-tests.sh`

**Purpose**: Showcases all test types in the project (unit, property, integration, etc.).

**What it shows**:
- 📊 Test organization and statistics
- 🧪 Unit tests (individual components)
- 🎲 Property-based tests (1000s of random cases)
- 🔗 Integration tests (real blockchain data)
- 📚 Documentation tests (examples in docs)
- ⚡ Performance benchmarks
- 📊 Code coverage analysis
- 🔒 Security audits

**Usage**:
```bash
./scripts/demo-tests.sh
```

**Output**: Comprehensive test execution with:
- Test statistics by crate
- Real-time test execution results
- Coverage percentages
- Code quality checks (fmt, clippy)
- Security audit results

**Requirements**: Standard Rust toolchain
- Optional: `cargo-llvm-cov` for coverage
- Optional: `cargo-audit` for security scanning

---

## 🔄 Comparison: Verus vs Property Tests

| Aspect | Verus Formal Verification | Property-Based Tests |
|--------|--------------------------|---------------------|
| **Guarantees** | Mathematical proof | High confidence |
| **Coverage** | All possible inputs | Random sampling |
| **Speed** | Slower (minutes) | Fast (seconds) |
| **When it runs** | Weekly in CI | Every commit |
| **What it proves** | "Provably correct" | "Probably correct" |
| **Dependencies** | Requires Verus | Built-in (proptest) |

**Bottom line**: Verus gives mathematical guarantees, property tests give practical confidence. We use both!

---

## 📋 Demo Quick Reference

```bash
# Formal verification demo (recommended!)
./scripts/demo-verus.sh

# Comprehensive test suite
./scripts/demo-tests.sh

# Install Verus (optional)
./scripts/install-verus.sh

# Run Verus verification manually
./scripts/verify_all.sh
```

---

## 🎯 Use Cases

### For Presentations / Papers / Conferences

Run `./scripts/demo-verus.sh` to showcase:
- Formal verification capabilities
- Security guarantees proven mathematically
- Modern software engineering practices

### For Code Reviews / Audits

Run `./scripts/demo-tests.sh` to demonstrate:
- Comprehensive test coverage
- Multiple testing strategies (unit, property, integration)
- Code quality enforcement (clippy, fmt)
- Security scanning (audit)

### For CI/CD Validation

Both scripts are designed to work in CI/CD:
- Color output can be disabled with `NO_COLOR=1`
- Exit codes indicate success/failure
- Detailed logs for debugging

---

## 📖 Related Documentation

- **Formal Verification**: See `docs/FORMAL_VERIFICATION.md`
- **Testing Strategy**: See `docs/TESTING_STRATEGY.md`
- **Verus Setup**: See `docs/VERUS_SETUP.md`
- **Verification Targets**: See `docs/VERIFICATION_TARGETS.md`

---

## 🤝 Contributing

When adding new verification targets or tests:

1. Update the demo scripts to showcase them
2. Add examples to this README
3. Update the verification count if adding VCs
4. Test the demo scripts locally before committing

---

**Last Updated**: 2025-11-17
**Status**: Production-ready demos for formal verification and testing
