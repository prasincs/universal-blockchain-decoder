## Description

Invalid Cardano transaction CBOR examples in documentation cannot be decoded by the official pallas library and violate CDDL specifications.

## Examples Affected

### Example 1: 6-Element Array
```
86a500818258201db731b2349b2a16745512809c86756ed42e2547f8cf7b93d8803bd88bd68338000d8001828258390003a114d3cf5200be6d1990493620ce2155a756582ec92ae4c532942902053e0c6f434f19a205fb5c118500cc77dc32c4e0b2a7ccbc6046861a0591fd6482583900cd0f6f5ee5b34c5af9aa40fc99fa788996f27e464a96d58f3e524bce53e163da5b1d71b783f037a745e1250b9df2450754ffed78fbf592291a000f4240021a000290f90e809fff8080f5f6
```

**Issue**: Has 6 CBOR elements; spec requires 3-4

### Example 2: Indefinite-Length Arrays
```
839f8200d8185826825820e981442c2be40475bb42193ca35907861d90715854de6fcba767b98f1789b51219439aff9f8282d818584a83581ce7fe8e468d2249f18cd7bf9aec0d4374b7d3e18609ede8589f82f7f0a20058208200581c240596b9b63fc010c06fbe92cf6f820587406534795958c411e662dc014443c0688e001a6768cc861b0037699e3ea6d064ffa0
```

**Issue**: Uses indefinite-length arrays; violates CBOR RFC 8949 Section 3.9

## Steps to Reproduce

```rust
use pallas_codec::minicbor;
use pallas_primitives::alonzo::MintedTx;

let tx_hex = "86a500818258201db731b2349b2a16745512809c86756ed42e2547f8cf7b93d8803bd88bd68338...";
let tx_bytes = hex::decode(tx_hex).unwrap();
let result = minicbor::decode::<MintedTx>(&tx_bytes);

// Expected: Ok(tx)
// Actual: Err("unexpected type indefinite array at position 182: expected map")
```

## Expected Behavior

Transaction examples should:
- Decode successfully with pallas library
- Follow CDDL specification: `[tx_body, witness_set, bool, auxiliary_data/null]`
- Use definite-length encoding per CBOR RFC 8949

## Actual Behavior

Both examples fail to decode with errors:
- Example 1: `unexpected type indefinite array at position 182: expected map`
- Example 2: `unexpected type indefinite array at position 1: expected map`

## Impact

- **Severity**: High
- **Affected**: Developers implementing Cardano parsers/decoders
- **Risk**: Incompatible implementations, wasted development time

## Proposed Solution

Replace invalid examples with validated ones from:

1. **Pallas test fixtures** (recommended):
   - Path: `pallas-applying/tests/`
   - Example: Mainnet tx `a06e5a0150e09f8983be2deafab9e04afc60d92e7110999eb672c903343f1e26`

2. **Cardano-ledger test suite**:
   - Path: `eras/{era}/test-suite/`

3. **cardano-cli export**:
   ```bash
   cardano-cli transaction view --tx-file tx.signed --output-json
   ```

## Additional Context

- **Full technical report**: [CARDANO_DOCUMENTATION_BUG_REPORT.md](https://github.com/prasincs/universal-blockchain-decoder/blob/main/docs/CARDANO_DOCUMENTATION_BUG_REPORT.md)
- **Test code**: [integration_tests.rs#L267-L333](https://github.com/prasincs/universal-blockchain-decoder/blob/main/crates/decoder-cardano/tests/integration_tests.rs#L267-L333)
- **CDDL specification**: [cardano-ledger CDDL](https://github.com/IntersectMBO/cardano-ledger/blob/master/eras/conway/impl/cddl-files/conway.cddl)

## Verification

Tested with:
- ✅ Pallas 0.30 (Alonzo and Babbage decoders)
- ✅ Custom decoder using minicbor 0.20
- ✅ Manual CBOR hex analysis

All tests confirm: **Examples are invalid**.

## Environment

- **Pallas version**: 0.30
- **Minicbor version**: 0.20
- **Test date**: 2025-11-17

---

**Labels**: bug, documentation, cbor, high-priority
**Assignees**: @documentation-team
**Projects**: Documentation Cleanup
