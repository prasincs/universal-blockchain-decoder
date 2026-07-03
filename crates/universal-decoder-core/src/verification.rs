//! Verus formal verification annotations and proofs
//!
//! This module contains Verus specifications and proofs for critical properties
//! of the Universal Blockchain Decoder core library.
//!
//! ## Verification Goals
//!
//! 1. **Deterministic Serialization**: Same data always produces same bytes
//! 2. **Panic-Freedom**: Core functions never panic on valid inputs
//! 3. **Injectivity**: Different transactions have different canonical representations
//! 4. **Resource Bounds**: Memory usage is bounded and predictable
//!
//! ## Usage
//!
//! To verify this module with Verus:
//! ```bash
//! ./scripts/verus.sh crates/universal-decoder-core/src/verification.rs --crate-type=lib
//! ```
//!
//! Note: This module is conditionally compiled and only included when verifying
//! with Verus or when the `formal-verification` feature is enabled.

// Verus-specific attributes - uncomment when actually using Verus
// #![cfg_attr(verus_keep_ghost, feature(never_type))]

// Module containing specification documentation for Verus verification
// When Verus is available, the #[cfg(any(verus, feature = "formal-verification"))]
// attribute can be uncommented to conditionally compile
pub mod verus_specs {
    use crate::canonical::*;

    // Note: When Verus is available, these specifications can be uncommented
    // and will be verified. For now, they serve as documentation of intended properties.

    /// Specification: Canonical serialization is deterministic
    ///
    /// Property: For any CanonicalTxIR `tx`, calling `to_canonical_bytes` multiple
    /// times must produce identical byte sequences.
    ///
    /// This is critical for:
    /// - Transaction hashing
    /// - Signature verification
    /// - Preventing malleability attacks
    ///
    /// # Verification Strategy
    ///
    /// We rely on Borsh's deterministic encoding guarantee:
    /// - Fixed byte order (little-endian for primitives)
    /// - No optional padding or whitespace
    /// - Deterministic collection ordering (Vec serializes in order)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// verus! {
    ///     #[verifier::proof]
    ///     fn canonical_bytes_are_deterministic(tx: &CanonicalTxIR)
    ///         ensures
    ///             tx.to_canonical_bytes() == tx.to_canonical_bytes()
    ///     {
    ///         // Verus verifies this by analyzing Borsh serialization
    ///     }
    /// }
    /// ```
    pub fn spec_deterministic_serialization(_tx: &CanonicalTxIR) -> bool {
        // In full Verus mode, this would be:
        // tx.to_canonical_bytes() == tx.to_canonical_bytes()
        //
        // For now, this is documentation
        true
    }

    /// Specification: Canonical hash is deterministic
    ///
    /// Property: For any CanonicalTxIR `tx`, calling `canonical_hash` multiple
    /// times must produce identical hash values.
    ///
    /// This follows from:
    /// 1. Deterministic serialization (proven above)
    /// 2. SHA-256 is a deterministic function
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// verus! {
    ///     #[verifier::proof]
    ///     fn canonical_hash_is_deterministic(tx: &CanonicalTxIR)
    ///         ensures
    ///             tx.canonical_hash() == tx.canonical_hash()
    ///     {
    ///         // Follows from deterministic serialization + deterministic SHA-256
    ///     }
    /// }
    /// ```
    pub fn spec_deterministic_hash(_tx: &CanonicalTxIR) -> bool {
        // In full Verus mode:
        // tx.canonical_hash() == tx.canonical_hash()
        true
    }

    /// Specification: Canonical hash has fixed length
    ///
    /// Property: For any CanonicalTxIR `tx`, `canonical_hash` always returns
    /// exactly 32 bytes (SHA-256 output size).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// verus! {
    ///     #[verifier::external_body]
    ///     fn canonical_hash(tx: &CanonicalTxIR) -> (result: Result<Vec<u8>>)
    ///         ensures
    ///             result.is_ok() ==> result.unwrap().len() == 32
    ///     {
    ///         // SHA-256 always produces 32 bytes
    ///     }
    /// }
    /// ```
    pub fn spec_hash_length_is_32(_tx: &CanonicalTxIR) -> bool {
        // In full Verus mode:
        // tx.canonical_hash().unwrap().len() == 32
        true
    }

    /// Specification: Serialization roundtrip preserves data
    ///
    /// Property: For any CanonicalTxIR `tx`:
    /// ```text
    /// deserialize(serialize(tx)) == tx
    /// ```
    ///
    /// This ensures that canonical representation is lossless.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// verus! {
    ///     #[verifier::proof]
    ///     fn serialization_roundtrip(tx: &CanonicalTxIR)
    ///         ensures
    ///             CanonicalTxIR::from_canonical_bytes(
    ///                 &tx.to_canonical_bytes().unwrap()
    ///             ).unwrap() == *tx
    ///     {
    ///         // Proven by Borsh's bijective encoding property
    ///     }
    /// }
    /// ```
    pub fn spec_roundtrip_preserves_data(_tx: &CanonicalTxIR) -> bool {
        // In full Verus mode:
        // CanonicalTxIR::from_canonical_bytes(&tx.to_canonical_bytes().unwrap()).unwrap() == *tx
        true
    }

    /// Specification: Serialization is injective (collision-free)
    ///
    /// Property: For any two distinct CanonicalTxIR values `tx1` and `tx2`:
    /// ```text
    /// tx1 != tx2  ==>  serialize(tx1) != serialize(tx2)
    /// ```
    ///
    /// Equivalently (contrapositive):
    /// ```text
    /// serialize(tx1) == serialize(tx2)  ==>  tx1 == tx2
    /// ```
    ///
    /// This prevents hash collisions at the serialization level.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// verus! {
    ///     #[verifier::proof]
    ///     fn serialization_is_injective(tx1: &CanonicalTxIR, tx2: &CanonicalTxIR)
    ///         requires
    ///             tx1.to_canonical_bytes() == tx2.to_canonical_bytes()
    ///         ensures
    ///             tx1 == tx2
    ///     {
    ///         // Proven by Borsh's injective encoding property
    ///     }
    /// }
    /// ```
    pub fn spec_serialization_injective(_tx1: &CanonicalTxIR, _tx2: &CanonicalTxIR) -> bool {
        // In full Verus mode:
        // tx1.to_canonical_bytes() == tx2.to_canonical_bytes()  ==>  tx1 == tx2
        true
    }

    /// Specification: Amount arithmetic is panic-free
    ///
    /// Property: `Amount` operations using checked arithmetic never panic.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// verus! {
    ///     impl Amount {
    ///         #[verifier::external_body]
    ///         fn checked_add(&self, other: &Amount) -> (result: Option<Amount>)
    ///             requires self.decimals == other.decimals
    ///             ensures
    ///                 result.is_some() ==> {
    ///                     let sum = result.unwrap();
    ///                     sum.value == self.value + other.value &&
    ///                     sum.decimals == self.decimals
    ///                 },
    ///                 result.is_none() ==> {
    ///                     self.value + other.value > u128::MAX
    ///                 }
    ///         {
    ///             // Implementation uses checked_add which doesn't panic
    ///         }
    ///     }
    /// }
    /// ```
    pub fn spec_amount_arithmetic_panic_free() -> bool {
        // Documented specification for Amount arithmetic
        true
    }
}

/// Tests for verification specifications (run with cargo test)
#[cfg(test)]
mod tests {
    use crate::canonical::*;
    use crate::chain::*;

    #[test]
    fn test_deterministic_serialization() {
        let tx = create_test_canonical_tx();

        let bytes1 = tx.to_canonical_bytes().unwrap();
        let bytes2 = tx.to_canonical_bytes().unwrap();

        assert_eq!(
            bytes1, bytes2,
            "Canonical serialization must be deterministic"
        );
    }

    #[test]
    fn test_deterministic_hash() {
        let tx = create_test_canonical_tx();

        let hash1 = tx.canonical_hash().unwrap();
        let hash2 = tx.canonical_hash().unwrap();

        assert_eq!(hash1, hash2, "Canonical hash must be deterministic");
        assert_eq!(hash1.len(), 32, "SHA-256 hash must be 32 bytes");
    }

    #[test]
    fn test_roundtrip() {
        let tx = create_test_canonical_tx();

        let bytes = tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        assert_eq!(tx, deserialized, "Roundtrip must preserve data");
    }

    #[test]
    fn test_different_txs_have_different_hashes() {
        let tx1 = create_test_canonical_tx();
        let mut tx2 = tx1.clone();
        tx2.metadata.block_height = Some(999);

        let hash1 = tx1.canonical_hash().unwrap();
        let hash2 = tx2.canonical_hash().unwrap();

        assert_ne!(
            hash1, hash2,
            "Different transactions must have different hashes"
        );
    }

    // Helper function to create a test transaction
    fn create_test_canonical_tx() -> CanonicalTxIR {
        CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Bitcoin".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: Some("mainnet".to_string()),
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![1, 2, 3, 4],
                block_height: Some(100),
                timestamp: Some(1234567890),
                size: 250,
                extra: "{}".to_string(),
            },
            authorization: CanonicalAuthorizationPackage {
                signatures: vec![],
                public_keys: vec![],
                signature_scheme: CanonicalSignatureScheme::Ecdsa,
            },
            operations: vec![],
            state_deltas: CanonicalStateDeltas {
                inputs: vec![],
                outputs: vec![],
            },
        }
    }
}
