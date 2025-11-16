## Description

<!-- Provide a brief description of the changes in this PR -->

## Type of Change

<!-- Mark the relevant option with an 'x' -->

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update
- [ ] 🔧 Refactoring (no functional changes)
- [ ] ⚡ Performance improvement
- [ ] ✅ Test addition/improvement
- [ ] 🔒 Security fix
- [ ] 🎨 Style/formatting change
- [ ] 🔨 Build/CI change
- [ ] 🏗️ New blockchain decoder

## Related Issues

<!-- Link to related issues using #issue_number -->

Closes #
Related to #

## Changes Made

<!-- Provide a detailed list of changes -->

-
-
-

## Blockchain Support (if applicable)

<!-- If adding a new blockchain decoder, fill this section -->

- **Blockchain**:
- **Chain ID**:
- **Transaction Format**:
- **Hashing Algorithm**:
- **Reference Implementation**:
- **Test Coverage**: % (unit tests) + % (property tests)

## Testing

### Pre-Submission Checklist

**Required checks (must all pass):**

- [ ] `cargo fmt --all` has been run (code is formatted)
- [ ] `cargo clippy --all --all-targets --all-features -- -D warnings` passes with zero warnings
- [ ] `cargo test --all` passes (all tests pass)
- [ ] `cargo build --all` succeeds (project builds)
- [ ] `cargo doc --all --no-deps` builds without errors

### Test Coverage

<!-- Mark what types of tests you've added -->

- [ ] Unit tests added/updated
- [ ] Property-based tests added (using proptest)
- [ ] Integration tests added (with real transaction fixtures)
- [ ] Existing tests still pass
- [ ] Test coverage is adequate (aim for 80%+)

### Test Details

<!-- Describe the tests you've added -->

**New tests:**
-
-

**Modified tests:**
-
-

**Test fixtures added:**
- Location: `tests/fixtures/...`
- Description:

## Documentation

<!-- Mark all that apply -->

- [ ] Code changes include appropriate inline documentation
- [ ] Public APIs have doc comments with examples
- [ ] README.md updated (if applicable)
- [ ] CHANGELOG.md updated
- [ ] Documentation in `docs/` updated (if applicable)
- [ ] Examples added/updated (if applicable)

## Breaking Changes

<!-- If this is a breaking change, describe what breaks and how to migrate -->

- [ ] This PR includes breaking changes
- [ ] Migration guide provided below

**Migration Guide:**

<!-- If breaking changes, explain how users should update their code -->

```rust
// Old API
// ...

// New API
// ...
```

## Performance Impact

<!-- Describe any performance implications -->

- [ ] No performance impact
- [ ] Performance improved (provide benchmarks if available)
- [ ] Performance degraded (justify why)

**Benchmarks:**

<!-- If applicable, show before/after benchmarks -->

```
Before: ...
After: ...
```

## Security Considerations

<!-- Consider security implications -->

- [ ] No security implications
- [ ] Reviewed for common vulnerabilities (buffer overflows, panics, etc.)
- [ ] Cryptographic operations use established libraries
- [ ] Input validation added for untrusted data
- [ ] Potential security impact (describe below)

**Security Notes:**

<!-- Describe any security considerations -->

## Design Principles Compliance

<!-- Ensure your changes align with project design principles (see CLAUDE.md) -->

- [ ] Maintains minimal core (< 3000 LOC for core library)
- [ ] Uses trait-based extensibility (not enums for chains)
- [ ] Uses canonical serialization (Borsh, not JSON for hashing)
- [ ] No `unsafe` code in core library
- [ ] Follows decoding-only scope (no encoding/signing/broadcasting)
- [ ] Maintains supply chain security (minimal dependencies)
- [ ] Code is reviewable and audit-friendly

## Checklist

<!-- Final checklist before submission -->

- [ ] I have read [CONTRIBUTING.md](https://github.com/prasincs/universal-blockchain-decoder/blob/main/CONTRIBUTING.md)
- [ ] My code follows the project's code style guidelines
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published

## Screenshots/Examples

<!-- If applicable, add screenshots or usage examples -->

```rust
// Example usage of new feature

```

## Additional Context

<!-- Add any other context about the PR here -->

## Reviewer Notes

<!-- Any specific areas you'd like reviewers to focus on? -->

**Please review:**
-
-

**Questions for reviewers:**
-
-

---

**By submitting this pull request, I confirm that my contribution is made under the terms of the MIT OR Apache-2.0 license.**
