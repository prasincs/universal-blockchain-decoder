# Canonical Serialization: Why JSON is Dangerous

## ⚠️ Critical Security Issue: JSON is NOT Canonical

### The Problem

JSON serialization is **fundamentally unsafe** for blockchain transaction encoding because it is **NOT deterministic**:

```rust
// WRONG: JSON can produce different bytes for the same data!
let tx_ir = /* ... */;
let json1 = serde_json::to_string(&tx_ir)?; // {"a":1, "b":2}
let json2 = serde_json::to_string_pretty(&tx_ir)?; // {
                                                     //   "a": 1,
                                                     //   "b": 2
                                                     // }
// json1 != json2  BUT THEY REPRESENT THE SAME DATA!
```

### Why This Breaks Security

1. **Signature Verification Fails**
   ```
   Original TX bytes  →  decode  →  TxIR  →  JSON  →  Different bytes!
                                                    ↓
                                            Signature invalid ❌
   ```

2. **Transaction Malleability**
   - Attacker modifies whitespace/key order
   - Logical transaction unchanged
   - Signature becomes invalid
   - Double-spend possible

3. **Violates Injectivity**
   ```
   encode(decode(bytes)) ≠ bytes  ❌
   ```

## ✅ The Solution: Borsh (Binary Canonical Serialization)

### Why Borsh?

Borsh provides:
- ✅ **Deterministic**: Same data → always same bytes
- ✅ **Bijective**: One-to-one mapping
- ✅ **Efficient**: Binary format (no overhead)
- ✅ **No ambiguity**: Fixed encoding rules
- ✅ **Battle-tested**: Used by NEAR, Solana

### Comparison: JSON vs Borsh

| Property | JSON | Borsh |
|----------|------|-------|
| Deterministic | ❌ No | ✅ Yes |
| Bijective | ❌ No | ✅ Yes |
| Key ordering | ❌ Undefined | ✅ Fixed |
| Whitespace | ❌ Variable | ✅ N/A (binary) |
| Size | 🐌 Large | ⚡ Compact |
| Parse speed | 🐌 Slow | ⚡ Fast |
| Human readable | ✅ Yes | ❌ No |
| Signature safe | ❌ **NO** | ✅ **YES** |

## 🔐 Correct Usage

### For Signature Verification (MUST USE BORSH)

```rust
use universal_decoder_core::prelude::*;

let tx_ir = /* ... */;

// CORRECT: Use Borsh for canonical bytes
let canonical_bytes = tx_ir.to_canonical_bytes()?;
let hash = tx_ir.canonical_hash()?;  // SHA-256 of Borsh bytes

// Verify signature against canonical hash
verify_signature(&hash, &signature, &public_key)?;
```

### For Human Display (JSON is OK)

```rust
// JSON is ONLY acceptable for human-readable display
let json = serde_json::to_string_pretty(&tx_ir)?;
println!("Transaction for display:\n{}", json);

// But NEVER use this for:
// - Hashing ❌
// - Signature verification ❌
// - Canonical representation ❌
// - Storage keys ❌
```

## 📊 Concrete Example: Malleability Attack

### Scenario: Transaction Hash Mismatch

```rust
// Original transaction from blockchain
let bitcoin_tx_bytes = hex::decode("0100000001...")?;
let tx = BitcoinDecoder::decode(&bitcoin_tx_bytes)?;
let tx_ir = tx.canonicalize()?;

// WRONG: Use JSON for hash
let json_bytes = serde_json::to_vec(&tx_ir)?;
let json_hash = sha256(&json_bytes);

// Attacker uses different JSON formatting
let pretty_json_bytes = serde_json::to_vec_pretty(&tx_ir)?;
let attacker_hash = sha256(&pretty_json_bytes);

// DISASTER: Same transaction, different hashes!
assert_ne!(json_hash, attacker_hash); // ❌ SECURITY FAILURE

// CORRECT: Use canonical serialization
let canonical_bytes1 = tx_ir.to_canonical_bytes()?;
let canonical_bytes2 = tx_ir.to_canonical_bytes()?;
let canonical_hash1 = sha256(&canonical_bytes1);
let canonical_hash2 = sha256(&canonical_bytes2);

// ✅ Always produces the same hash
assert_eq!(canonical_hash1, canonical_hash2);
assert_eq!(canonical_bytes1, canonical_bytes2);
```

## 🏗️ Implementation Architecture

### Canonical Type Hierarchy

```
TxIR<'a, V>  (with lifetimes, PhantomData)
     ↓
to_canonical()
     ↓
CanonicalTxIR  (pure data, Borsh-serializable)
     ↓
to_canonical_bytes()
     ↓
Vec<u8>  (deterministic binary)
     ↓
SHA-256
     ↓
Transaction Hash (canonical)
```

### Key Design Decisions

1. **Separate Types**: `TxIR` vs `CanonicalTxIR`
   - `TxIR`: Has lifetimes, used during processing
   - `CanonicalTxIR`: Owned data, used for serialization

2. **JSON as String**: Nested data (like `extra` field)
   - Cannot serialize arbitrary `serde_json::Value` to Borsh
   - Solution: Store JSON as string in canonical form
   - Trade-off: Lose structure but maintain determinism

3. **Conversion Layer**: From `TxIR` → `CanonicalTxIR`
   - Strips lifetimes
   - Removes `PhantomData`
   - Converts all types to Borsh-compatible equivalents

## 🔬 Verification

### Property: Determinism

```rust
#[test]
fn test_deterministic_encoding() {
    let tx_ir = create_test_transaction();

    // Encode twice
    let bytes1 = tx_ir.to_canonical_bytes()?;
    let bytes2 = tx_ir.to_canonical_bytes()?;

    // MUST be identical
    assert_eq!(bytes1, bytes2);
}
```

### Property: Bijection (for owned types)

```rust
#[test]
fn test_bijection() {
    let canonical_tx = create_canonical_transaction();

    // Encode
    let bytes = canonical_tx.to_canonical_bytes()?;

    // Decode
    let decoded = CanonicalTxIR::from_canonical_bytes(&bytes)?;

    // MUST round-trip perfectly
    assert_eq!(canonical_tx, decoded);
}
```

### Property: Collision Resistance

```rust
#[test]
fn test_different_transactions_different_hashes() {
    let tx1 = create_transaction_with_value(100);
    let tx2 = create_transaction_with_value(101);

    let hash1 = tx1.canonical_hash()?;
    let hash2 = tx2.canonical_hash()?;

    // Different transactions MUST have different hashes
    assert_ne!(hash1, hash2);
}
```

## 🎯 Formal Specification

### Canonical Encoding Requirements

For a serialization format to be canonical, it must satisfy:

1. **Determinism**:
   ```
   ∀ x : TxIR. encode(x) = encode(x)
   ```

2. **Injectivity**:
   ```
   ∀ x,y : TxIR. encode(x) = encode(y) ⟹ x = y
   ```

3. **Totality**:
   ```
   ∀ x : TxIR. ∃ b : Bytes. encode(x) = b
   ```

4. **Surjectivity (for valid encodings)**:
   ```
   ∀ b : ValidBytes. ∃ x : TxIR. encode(x) = b
   ```

### Why JSON Fails

JSON violates **Determinism** and **Injectivity**:

```
Counter-example for determinism:
  let x = {"a": 1, "b": 2}
  encode(x) with compact formatting = {"a":1,"b":2}
  encode(x) with pretty formatting  = {\n  "a": 1,\n  "b": 2\n}
  Therefore: encode(x) ≠ encode(x)  ❌

Counter-example for injectivity:
  let b1 = ["{\"a\":1,\"b\":2}"]
  let b2 = ["{\"b\":2,\"a\":1}"]  // different key order
  decode(b1) = decode(b2) = {"a": 1, "b": 2}
  But b1 ≠ b2
  Therefore: encode is not injective  ❌
```

### Why Borsh Succeeds

Borsh satisfies all properties:

```
Borsh encoding rules (simplified):
  - Integers: little-endian bytes
  - Strings: length-prefixed UTF-8
  - Vectors: length-prefixed elements
  - Structs: fields in declaration order
  - Enums: discriminant + payload

These rules are:
  ✅ Deterministic (fixed format)
  ✅ Injective (one-to-one)
  ✅ Total (handles all data)
  ✅ Surjective (all valid bytes decode)
```

## 📝 Best Practices

### DO ✅

1. **Always use Borsh for canonical operations**:
   ```rust
   let canonical_bytes = tx_ir.to_canonical_bytes()?;
   let hash = tx_ir.canonical_hash()?;
   ```

2. **Use JSON for display only**:
   ```rust
   let display = serde_json::to_string_pretty(&tx_ir)?;
   println!("{}", display);
   ```

3. **Test determinism**:
   ```rust
   #[test]
   fn test_canonical_determinism() {
       let tx = create_test_tx();
       let b1 = tx.to_canonical_bytes()?;
       let b2 = tx.to_canonical_bytes()?;
       assert_eq!(b1, b2);
   }
   ```

### DON'T ❌

1. **Never hash JSON**:
   ```rust
   // WRONG
   let json = serde_json::to_vec(&tx_ir)?;
   let hash = sha256(&json); // ❌ INSECURE
   ```

2. **Never use JSON for storage keys**:
   ```rust
   // WRONG
   let key = serde_json::to_string(&tx_id)?; // ❌ NON-DETERMINISTIC
   db.put(&key, &value);
   ```

3. **Never compare JSON strings**:
   ```rust
   // WRONG
   let json1 = serde_json::to_string(&tx1)?;
   let json2 = serde_json::to_string(&tx2)?;
   if json1 == json2 { /* ... */ } // ❌ UNRELIABLE
   ```

## 🔗 Related Standards

- **Borsh**: https://borsh.io/
- **SCALE**: https://docs.substrate.io/reference/scale-codec/
- **BCS**: https://github.com/diem/bcs
- **Protobuf (deterministic)**: https://protobuf.dev/programming-guides/encoding/

## 📚 References

1. Yakovenko et al., "Solana: A new architecture for a high performance blockchain" (uses Borsh)
2. NEAR Protocol Specification (canonical Borsh encoding)
3. Bitcoin BIP-0340: "Schnorr Signatures" (canonical signature encoding)
4. Ethereum EIP-155: "Simple replay attack protection" (canonical transaction signing)

## ✅ Migration Checklist

If your code currently uses JSON for canonical operations:

- [ ] Replace JSON serialization with Borsh
- [ ] Update hash computation to use `canonical_hash()`
- [ ] Update signature verification to use canonical bytes
- [ ] Add determinism tests
- [ ] Keep JSON only for human-readable display
- [ ] Update documentation to warn against JSON for canonical use
- [ ] Review all `serde_json::to_*` calls
- [ ] Add CI checks for canonical encoding tests

---

**Remember**: JSON is for humans 👨‍💻, Borsh is for machines 🤖!
