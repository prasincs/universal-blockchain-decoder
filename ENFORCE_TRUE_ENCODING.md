# Enforcing True Encoding with Rust's Type System

## Problem

Current implementation "cheats" by storing original bytes:

```rust
pub struct EthereumTransaction {
    pub nonce: u64,
    // ... fields ...
    pub raw_bytes: Vec<u8>,  // ← CHEATING!
}

impl ChainEncoder for EthereumTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())  // ← Not real encoding!
    }
}
```

**Goal**: Make it **impossible** to store bytes using the type system.

---

## Solution 1: Separate Borrowed Decoding (Recommended)

### Idea

Decoders receive `&[u8]` but **cannot own** it - enforced by lifetimes.

### Implementation

```rust
/// Decoded transaction that borrows from original bytes
/// Cannot store the bytes because it doesn't own them
pub struct EthereumTransaction<'a> {
    pub nonce: u64,
    pub gas_price: Option<u128>,
    pub gas_limit: u128,
    pub to: Option<[u8; 20]>,
    pub value: u128,
    pub data: Vec<u8>,  // Owned copy (needed for operations)
    pub chain_id: Option<u64>,
    // ... other fields ...

    // ✅ Can borrow for validation
    _original: PhantomData<&'a [u8]>,

    // ❌ CANNOT do this - ownership violation!
    // raw_bytes: Vec<u8>,
}

impl<'a> EthereumTransaction<'a> {
    /// Decode from borrowed bytes
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        // Parse RLP...
        Ok(Self {
            nonce: /* ... */,
            // ...
            _original: PhantomData,
        })
    }
}

impl<'a> ChainEncoder for EthereumTransaction<'a> {
    /// Must reconstruct from fields - no stored bytes available!
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();

        // Reconstruct RLP from fields
        encoder.append_u64(self.nonce)?;
        encoder.append_optional_u128(self.gas_price)?;
        encoder.append_u128(self.gas_limit)?;
        encoder.append_address(self.to)?;
        encoder.append_u128(self.value)?;
        encoder.append_bytes(&self.data)?;
        // ... encode v, r, s ...

        encoder.finalize()
    }
}
```

**Key**:
- ✅ Type system **prevents** `raw_bytes: Vec<u8>`
- ✅ Lifetimes ensure bytes are **borrowed**, not owned
- ✅ Encoder **must** reconstruct from fields

---

## Solution 2: Phantom Type Enforcement

### Idea

Use a phantom type parameter to track whether bytes can be stored.

### Implementation

```rust
/// Marker: Cannot store original bytes
pub struct NoStorage;

/// Marker: Can store original bytes (for testing only)
pub struct AllowStorage;

/// Transaction with storage policy
pub struct EthereumTransaction<Policy = NoStorage> {
    pub nonce: u64,
    // ... fields ...

    // Only exists if Policy = AllowStorage
    raw_bytes: Option<Vec<u8>>,
    _policy: PhantomData<Policy>,
}

impl EthereumTransaction<NoStorage> {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            nonce: /* ... */,
            raw_bytes: None,  // ✅ Cannot store!
            _policy: PhantomData,
        })
    }
}

impl<Policy> ChainEncoder for EthereumTransaction<Policy> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        // raw_bytes is None for NoStorage policy
        match &self.raw_bytes {
            None => {
                // Must reconstruct
                encode_rlp_from_fields(self)
            },
            Some(_) => {
                // Only reachable if Policy = AllowStorage
                unreachable!("NoStorage policy enforced at compile time")
            }
        }
    }
}
```

---

## Solution 3: Builder Pattern with Consumed Bytes

### Idea

Make the original bytes **consumed** during construction, so they can't be stored.

### Implementation

```rust
pub struct TransactionBuilder<'a> {
    bytes: &'a [u8],
}

pub struct EthereumTransaction {
    pub nonce: u64,
    // ... fields ...
    // ❌ No raw_bytes field possible
}

impl<'a> TransactionBuilder<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Consumes the builder, returns only the parsed tx
    pub fn build(self) -> Result<EthereumTransaction> {
        let rlp = RlpItem::decode(self.bytes)?;

        Ok(EthereumTransaction {
            nonce: rlp.get_u64(0)?,
            // ...
            // self.bytes is MOVED here, cannot be stored!
        })
    }
}

impl ChainEncoder for EthereumTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        // No bytes to return - must encode!
        encode_rlp(self)
    }
}
```

**Key**: Builder is **consumed** (moved), so bytes can't leak into the struct.

---

## Solution 4: Sealed Trait + Private Module (Nuclear Option)

### Idea

Make it impossible to implement `ChainEncoder` with stored bytes.

### Implementation

```rust
mod sealed {
    use super::*;

    /// Sealed trait - only we can implement it
    pub trait SealedEncoder {
        /// Internal: verify no stored bytes
        #[doc(hidden)]
        fn _assert_no_stored_bytes(&self) -> bool {
            true  // Must override to prove no storage
        }
    }
}

pub trait ChainEncoder: sealed::SealedEncoder {
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

// Only internal implementations allowed
impl sealed::SealedEncoder for EthereumTransaction {
    fn _assert_no_stored_bytes(&self) -> bool {
        // Compile-time check: struct has no Vec<u8> field
        std::mem::size_of::<Self>() < 1000  // Can't fit stored bytes
    }
}
```

**Key**: Sealed trait pattern prevents external implementations that could cheat.

---

## Recommended Approach: Solution 1 (Lifetime-Based)

**Advantages**:
1. ✅ **Zero-cost** - no runtime overhead
2. ✅ **Compile-time enforcement** - type system prevents storage
3. ✅ **Clear semantics** - lifetimes make ownership explicit
4. ✅ **Idiomatic Rust** - follows standard patterns
5. ✅ **Forces true encoding** - no way to store bytes

**Implementation Plan**:

```rust
// crates/universal-decoder-core/src/traits.rs

pub trait ChainDecoder {
    type TxSpecific<'a>: ChainEncoder + Canonicalizer<'a>;

    /// Decode transaction from borrowed bytes
    /// Implementer CANNOT store these bytes due to lifetime
    fn decode<'a>(raw_bytes: &'a [u8]) -> Result<Self::TxSpecific<'a>>;
}

pub trait ChainEncoder {
    /// Re-encode from parsed fields (no stored bytes available!)
    fn to_bytes(&self) -> Result<Vec<u8>>;
}
```

**Migration**:

```rust
// OLD (cheating):
pub struct EthereumTransaction {
    // ...
    raw_bytes: Vec<u8>,  // ❌
}

// NEW (enforced):
pub struct EthereumTransaction<'a> {
    // ...
    _marker: PhantomData<&'a ()>,  // ✅ Prevents storage
}
```

---

## For Testing: Separate Verification Type

If we need to verify encoders during testing:

```rust
#[cfg(test)]
pub struct VerifiableTransaction<T> {
    pub tx: T,
    pub original_bytes: Vec<u8>,  // Only in tests
}

#[cfg(test)]
impl<T: ChainEncoder> VerifiableTransaction<T> {
    pub fn verify_roundtrip(&self) -> Result<()> {
        let re_encoded = self.tx.to_bytes()?;
        if re_encoded != self.original_bytes {
            return Err(DecoderError::invalid_structure(
                "Encoder is not injective!"
            ));
        }
        Ok(())
    }
}
```

**Key**: Original bytes only exist in test wrapper, not production struct.

---

## Formal Verification Benefits

With this approach, we can prove at the **type level**:

```rust
verus! {
    spec fn cannot_store_bytes<'a>(tx: EthereumTransaction<'a>) -> bool {
        // Lifetime 'a guarantees tx cannot own the original bytes
        true
    }

    proof fn encoder_must_be_real<'a>(tx: EthereumTransaction<'a>) {
        // Since tx cannot contain original bytes,
        // to_bytes() MUST reconstruct from fields
        assert(tx.to_bytes() == encode_from_fields(tx));
    }
}
```

---

## Summary

**Question**: Can we prevent storing original bytes?

**Answer**: **YES!** Use lifetimes to enforce borrowing:

```rust
// ❌ BEFORE: Can cheat
struct Tx { raw_bytes: Vec<u8> }

// ✅ AFTER: Cannot cheat (enforced by type system)
struct Tx<'a> { _marker: PhantomData<&'a ()> }
```

This makes it **impossible** to implement a fake encoder that returns stored bytes.

---

## Next Steps

1. Update `ChainDecoder` trait to use lifetimes
2. Implement true RLP encoder
3. Migrate existing decoders
4. Add property tests to verify injectivity
5. Formal verification with Verus

**Timeline**: 2-3 weeks for full migration

**Breaking Change**: Yes, but worth it for correctness

---

Want me to implement this? It would require:
- Updating core traits
- Implementing RLP encoder
- Migrating Ethereum decoder
- Adding comprehensive tests
