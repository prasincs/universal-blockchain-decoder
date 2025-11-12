# Invalid Transactions

Malformed transactions that should fail to decode.

## Purpose

Test that decoders handle invalid input gracefully:
- Return `Result::Err` (never panic)
- Provide meaningful error messages
- Reject at validation stage when possible

## Test Cases

### Structural Errors
- Empty transaction bytes
- Truncated transactions
- Invalid RLP/Bincode encoding
- Malformed length fields
- Integer overflow attempts

### Semantic Errors
- Invalid signatures
- Negative values
- Exceeds maximum transaction size
- Non-canonical encodings

## Naming Convention

- `invalid_{chain}_{error_type}_{id}.json`
- Examples:
  - `invalid_btc_truncated_001.json`
  - `invalid_eth_bad_rlp_002.json`
  - `invalid_sol_overflow_003.json`

## Expected Behavior

All fixtures in this directory have:
```json
{
  "expected": {
    "should_decode": false
  }
}
```

Decoders MUST return error, never panic.
