# [BUG] Invalid Cardano Transaction CBOR Examples in Documentation

## TL;DR

Two commonly-cited Cardano transaction CBOR examples **cannot be decoded by the official pallas library** and violate CDDL specifications. We need to remove/replace them.

## The Problem

I was implementing a Cardano transaction decoder and found that example transactions from various documentation sources fail to decode:

### Example 1: "Alonzo Era Transaction"
```
CBOR Hex (truncated): 86a500818258201db731b2349b2a16745512809c86756ed42e2547f8cf7b93d8803bd88bd68338...
```

**Issues**:
- ❌ Has **6 CBOR elements** (byte `0x86`)
- ❌ CDDL spec requires **3-4 elements**: `[tx_body, witness_set, bool, auxiliary_data/null]`
- ❌ Pallas error: `unexpected type indefinite array at position 182: expected map`

### Example 2: "Byron/Shelley Era Transaction"
```
CBOR Hex (truncated): 839f8200d8185826825820e981442c2be40475bb42193ca35907861d90715854de6fcba...
```

**Issues**:
- ❌ Uses **indefinite-length arrays** (`0x9f`)
- ❌ CDDL spec requires **definite-length arrays**
- ❌ Violates CBOR RFC 8949 Section 3.9
- ❌ Pallas error: `unexpected type indefinite array at position 1: expected map`

## Verification

Tested with **pallas** (official Cardano Rust library):

```rust
use pallas_codec::minicbor;
use pallas_primitives::alonzo::MintedTx;
use pallas_primitives::babbage::MintedTx as BabbageTx;

let tx_bytes = hex::decode(tx_hex)?;

// Both fail with same error
let alonzo = minicbor::decode::<MintedTx>(&tx_bytes);  // ❌ FAIL
let babbage = minicbor::decode::<BabbageTx>(&tx_bytes); // ❌ FAIL
```

**Result**: Neither example can be decoded by pallas.

## Impact

- ❌ Developers waste time debugging "why won't this work?"
- ❌ Creates incorrect parser implementations
- ❌ Fragments ecosystem with incompatible decoders
- ❌ Erodes trust in official documentation

## Solution

Replace with **validated examples** from:

1. **Pallas test fixtures** (recommended):
   - Repository: https://github.com/txpipe/pallas
   - Location: `pallas-applying/tests/`
   - Example: Mainnet tx `a06e5a0150e09f8983be2deafab9e04afc60d92e7110999eb672c903343f1e26`
   - Status: ✅ Verified against mainnet

2. **Cardano-ledger test suite**:
   - Repository: https://github.com/IntersectMBO/cardano-ledger
   - Location: `eras/{era}/test-suite/`

3. **Export from cardano-cli**:
   ```bash
   cardano-cli transaction view --tx-file tx.signed --output-json
   # Use the "cborHex" field
   ```

## Correct Transaction Format

According to [Conway CDDL spec](https://github.com/IntersectMBO/cardano-ledger/blob/master/eras/conway/impl/cddl-files/conway.cddl):

```cddl
transaction =
  [ transaction_body
  , transaction_witness_set
  , bool              ; validity flag
  , auxiliary_data / null
  ]
```

**Required**: Exactly **4 elements** (or 3 if auxiliary_data is null), all using **definite-length encoding**.

## Where These Examples Appear

I found these examples in:
- Various Cardano documentation sites (please comment if you know specific URLs)
- Stack Overflow / StackExchange answers
- Tutorial blog posts
- Code example repositories

If you know where these appear, please:
1. File issues on those repositories
2. Link to this report for technical details
3. Suggest pallas test fixtures as replacements

## References

- **Full bug report**: [CARDANO_DOCUMENTATION_BUG_REPORT.md](https://github.com/prasincs/universal-blockchain-decoder/blob/claude/add-cardano-support-01DLdQNNf1mM84bC9hfCdMY6/docs/CARDANO_DOCUMENTATION_BUG_REPORT.md)
- **Test code**: [decoder-cardano/tests/integration_tests.rs](https://github.com/prasincs/universal-blockchain-decoder/blob/claude/add-cardano-support-01DLdQNNf1mM84bC9hfCdMY6/crates/decoder-cardano/tests/integration_tests.rs#L267-L333)
- **CDDL spec**: [cardano-ledger CDDL files](https://github.com/IntersectMBO/cardano-ledger/tree/master/eras)
- **RFC 8949**: [CBOR Specification](https://www.rfc-editor.org/rfc/rfc8949.html)

## Request for Help

1. **Documentation maintainers**: Please audit examples and replace with validated ones
2. **Community**: Help identify where these invalid examples appear
3. **Pallas maintainers**: Consider adding a "validated examples" section to docs

---

**Reported by**: @prasincs (via universal-blockchain-decoder project)
**Date**: 2025-11-17
**Severity**: High - Affects ecosystem compatibility
**Status**: Documented, awaiting fixes
