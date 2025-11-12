# Bitcoin Core Test Vectors

This directory contains test vectors from Bitcoin Core's test suite.

## Files

- `tx_valid.json` - Valid Bitcoin transactions (500+ test cases)
- `tx_invalid.json` - Invalid Bitcoin transactions (200+ test cases)

## Source

These files are fetched from the official Bitcoin Core repository:
- **Repository**: https://github.com/bitcoin/bitcoin
- **Path**: `src/test/data/`
- **Branch**: `master` (configurable)

## Updating Test Vectors

To fetch the latest test vectors from Bitcoin Core:

```bash
./scripts/update-bitcoin-test-vectors.sh
```

This ensures our decoder is tested against the most current Bitcoin Core test suite.

### Pinning to a Specific Version

To test against a specific Bitcoin Core version:

```bash
BITCOIN_BRANCH=v25.0 ./scripts/update-bitcoin-test-vectors.sh
```

### CI Integration

Our CI pipeline automatically validates against the latest Bitcoin Core test vectors:

```yaml
# .github/workflows/test.yml
- name: Update Bitcoin Core test vectors
  run: ./scripts/update-bitcoin-test-vectors.sh

- name: Run Bitcoin Core vector tests
  run: cargo test -p decoder-bitcoin --test bitcoin_core_vectors
```

## Test Format

Test vectors are in JSON format:

```json
[
  [
    [[prevout_hash, prevout_index, prevout_scriptPubKey, amount?], ...],
    serialized_transaction_hex,
    verify_flags
  ],
  ...
]
```

### Valid Transactions (tx_valid.json)

These transactions should decode successfully. Our decoder validates:
- Transaction structure is correct
- Parsing matches `bitcoin` crate (in dev-dependencies)
- TXID calculation matches
- Input/output counts match

### Invalid Transactions (tx_invalid.json)

These transactions are structurally invalid. Our decoder should:
- Return an error (not panic)
- Handle malformed inputs gracefully
- Reject transactions that fail basic structural validation

**Note**: Some "invalid" transactions are only invalid due to script verification
(not structural issues). Our decoder focuses on structural validation.

## License

Test vectors are from Bitcoin Core (MIT License).
See: https://github.com/bitcoin/bitcoin/blob/master/COPYING
