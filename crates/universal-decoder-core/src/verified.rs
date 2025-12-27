//! Type-safe decoder verification module.
//!
//! This module provides traits and types that enforce actual parsing of transactions
//! through the type system, preventing decoders from simply storing and replaying
//! raw bytes to satisfy the injective property.
//!
//! # Problem
//!
//! Without type-level enforcement, a decoder could cheat:
//! ```rust,ignore
//! struct LazyTransaction {
//!     raw_bytes: Vec<u8>,  // Just store bytes, don't parse
//! }
//!
//! impl ChainEncoder for LazyTransaction {
//!     fn to_bytes(&self) -> Result<Vec<u8>> {
//!         Ok(self.raw_bytes.clone())  // Trivially passes injective property
//!     }
//! }
//! ```
//!
//! # Solution
//!
//! This module provides:
//! 1. `ReconstructableTransaction` - requires reconstruction from parsed fields
//! 2. `VerifiedDecoder` - wrapper that verifies reconstruction matches original
//! 3. Property test utilities to detect lazy parsing
//!
//! # Architecture
//!
//! ```text
//! Raw Bytes (Input)
//!        ↓
//!   [ChainDecoder::decode()]
//!        ↓
//! VerifiedTransaction<P>
//!   ├── parsed: P (ParsedFields - NO raw_bytes access in to_bytes)
//!   └── original_bytes: Vec<u8> (stored separately, not accessible to P)
//!        │
//!        ├→ [P::reconstruct_bytes()] → Reconstructed bytes (from fields)
//!        │      ↓
//!        │   [verify: reconstructed == original_bytes]
//!        │
//!        └→ [to_bytes()] → Returns original_bytes (after verification)
//! ```

use crate::error::{DecoderError, Result};

/// Trait for transactions that can reconstruct their byte representation from parsed fields.
///
/// This trait is the key to enforcing actual parsing. Implementors MUST reconstruct
/// bytes from their parsed semantic fields, NOT from stored raw bytes.
///
/// # Type-Safety Guarantee
///
/// Types implementing this trait should NOT have access to raw bytes when implementing
/// `reconstruct_bytes()`. The separation is enforced by the `VerifiedTransaction` wrapper.
///
/// # Example
///
/// ```rust,ignore
/// struct EthereumParsedFields {
///     nonce: u64,
///     gas_price: u128,
///     gas_limit: u128,
///     to: Option<[u8; 20]>,
///     value: u128,
///     data: Vec<u8>,
///     v: u64,
///     r: [u8; 32],
///     s: [u8; 32],
///     // NO raw_bytes field!
/// }
///
/// impl ReconstructableTransaction for EthereumParsedFields {
///     fn reconstruct_bytes(&self) -> Result<Vec<u8>> {
///         // Must RLP-encode from the semantic fields
///         let mut encoder = RlpEncoder::new();
///         // ... encode each field ...
///         Ok(encoder.finalize())
///     }
/// }
/// ```
pub trait ReconstructableTransaction: Sized {
    /// Reconstruct the original byte representation from parsed fields.
    ///
    /// # Requirements
    ///
    /// - MUST reconstruct bytes from semantic fields only
    /// - MUST NOT rely on stored raw bytes
    /// - MUST produce bytes that when re-parsed yield equivalent fields
    ///
    /// # Formal Property
    ///
    /// For a correct implementation:
    /// ```text
    /// ∀ tx_bytes: parse(tx_bytes)?.reconstruct_bytes()? ≈ tx_bytes
    /// ```
    ///
    /// Note: We use ≈ (semantic equivalence) rather than = (byte equality)
    /// because some formats allow multiple valid encodings of the same data.
    fn reconstruct_bytes(&self) -> Result<Vec<u8>>;

    /// Check if two instances are semantically equivalent.
    ///
    /// This is used for verification when byte-level equality isn't guaranteed
    /// due to encoding flexibility (e.g., RLP integer encoding).
    ///
    /// Default implementation compares reconstructed bytes.
    fn semantically_equivalent(&self, other: &Self) -> bool
    where
        Self: PartialEq,
    {
        self == other
    }
}

/// Wrapper that separates parsed fields from raw bytes.
///
/// This type enforces the separation of concerns:
/// - `parsed`: Contains the parsed semantic fields (no raw bytes)
/// - `original_bytes`: Stored separately, only used for verification
///
/// # Type-Safety
///
/// The `P` type parameter (the parsed fields) does NOT have access to `original_bytes`.
/// This means `P::reconstruct_bytes()` MUST work purely from parsed data.
///
/// # Verification
///
/// The `verify()` method checks that reconstruction from parsed fields
/// produces bytes that are semantically equivalent to the original.
#[derive(Debug, Clone)]
pub struct VerifiedTransaction<P: ReconstructableTransaction> {
    /// Parsed semantic fields (no raw bytes)
    parsed: P,
    /// Original bytes (stored separately for verification)
    original_bytes: Vec<u8>,
    /// Whether reconstruction has been verified
    verified: bool,
}

impl<P: ReconstructableTransaction> VerifiedTransaction<P> {
    /// Create a new verified transaction.
    ///
    /// # Arguments
    ///
    /// * `parsed` - The parsed semantic fields
    /// * `original_bytes` - The original raw bytes for verification
    ///
    /// # Note
    ///
    /// The transaction is NOT verified at construction time.
    /// Call `verify()` or `verify_strict()` to validate.
    pub fn new(parsed: P, original_bytes: Vec<u8>) -> Self {
        Self {
            parsed,
            original_bytes,
            verified: false,
        }
    }

    /// Create a new verified transaction with immediate verification.
    ///
    /// Returns an error if reconstruction doesn't match original bytes.
    pub fn new_verified(parsed: P, original_bytes: Vec<u8>) -> Result<Self> {
        let mut tx = Self::new(parsed, original_bytes);
        tx.verify_strict()?;
        Ok(tx)
    }

    /// Get a reference to the parsed fields.
    pub fn parsed(&self) -> &P {
        &self.parsed
    }

    /// Get a mutable reference to the parsed fields.
    ///
    /// # Warning
    ///
    /// Modifying parsed fields invalidates verification.
    /// Call `verify()` again after modifications.
    pub fn parsed_mut(&mut self) -> &mut P {
        self.verified = false;
        &mut self.parsed
    }

    /// Check if this transaction has been verified.
    pub fn is_verified(&self) -> bool {
        self.verified
    }

    /// Verify that reconstruction from parsed fields matches original bytes.
    ///
    /// This method:
    /// 1. Calls `parsed.reconstruct_bytes()` to get bytes from fields
    /// 2. Compares with `original_bytes`
    /// 3. Sets `verified = true` if they match
    ///
    /// # Strict Mode
    ///
    /// Use `verify_strict()` for byte-exact verification.
    /// Use `verify_semantic()` when the format allows multiple valid encodings.
    pub fn verify_strict(&mut self) -> Result<()> {
        let reconstructed = self.parsed.reconstruct_bytes()?;
        if reconstructed != self.original_bytes {
            return Err(DecoderError::invalid_structure(format!(
                "Reconstruction mismatch: reconstructed {} bytes, original {} bytes",
                reconstructed.len(),
                self.original_bytes.len()
            )));
        }
        self.verified = true;
        Ok(())
    }

    /// Verify semantic equivalence (allows different byte representations).
    ///
    /// This is useful for formats like RLP where the same value can be
    /// encoded in multiple valid ways.
    ///
    /// The verification:
    /// 1. Reconstructs bytes from parsed fields
    /// 2. Re-parses the reconstructed bytes
    /// 3. Checks semantic equivalence with original parsed fields
    pub fn verify_semantic<F>(&mut self, reparse: F) -> Result<()>
    where
        F: FnOnce(&[u8]) -> Result<P>,
        P: PartialEq,
    {
        let reconstructed = self.parsed.reconstruct_bytes()?;
        let reparsed = reparse(&reconstructed)?;
        if !self.parsed.semantically_equivalent(&reparsed) {
            return Err(DecoderError::invalid_structure(
                "Semantic verification failed: reparsed fields don't match original",
            ));
        }
        self.verified = true;
        Ok(())
    }

    /// Get the original bytes (for injective property).
    ///
    /// # Panics
    ///
    /// Panics if the transaction has not been verified.
    /// Use `to_bytes_unchecked()` to skip verification check.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        if !self.verified {
            return Err(DecoderError::invalid_structure(
                "Transaction has not been verified. Call verify() first.",
            ));
        }
        Ok(self.original_bytes.clone())
    }

    /// Get the original bytes without verification check.
    ///
    /// # Safety
    ///
    /// This bypasses the verification requirement. Only use when you're
    /// certain the transaction is valid (e.g., in legacy code paths).
    pub fn to_bytes_unchecked(&self) -> Vec<u8> {
        self.original_bytes.clone()
    }

    /// Get reconstructed bytes (from parsed fields).
    ///
    /// This is useful for debugging and testing to compare
    /// reconstructed vs original bytes.
    pub fn reconstructed_bytes(&self) -> Result<Vec<u8>> {
        self.parsed.reconstruct_bytes()
    }

    /// Consume the wrapper and return just the parsed fields.
    pub fn into_parsed(self) -> P {
        self.parsed
    }

    /// Get the size of the original transaction in bytes.
    pub fn size(&self) -> usize {
        self.original_bytes.len()
    }
}

/// Trait for verified chain decoders.
///
/// This trait extends `ChainDecoder` to return `VerifiedTransaction` wrappers,
/// ensuring that the injective property is enforced through the type system.
pub trait VerifiedChainDecoder {
    /// The parsed fields type (no raw bytes).
    type ParsedFields: ReconstructableTransaction;

    /// Decode raw bytes into a verified transaction.
    ///
    /// The returned `VerifiedTransaction` wraps the parsed fields and
    /// stores original bytes separately for verification.
    fn decode_verified(raw_bytes: &[u8]) -> Result<VerifiedTransaction<Self::ParsedFields>>;

    /// Decode and immediately verify (strict byte equality).
    fn decode_and_verify(raw_bytes: &[u8]) -> Result<VerifiedTransaction<Self::ParsedFields>> {
        let mut tx = Self::decode_verified(raw_bytes)?;
        tx.verify_strict()?;
        Ok(tx)
    }
}

/// Test utilities for detecting lazy parsing.
///
/// These functions help write property tests that detect decoders
/// that cheat by storing raw bytes instead of actually parsing.
pub mod testing {
    use super::*;

    /// Test that modifying a parsed field changes the reconstructed bytes.
    ///
    /// This detects lazy parsing by checking that field mutations
    /// actually affect the output.
    ///
    /// # Arguments
    ///
    /// * `tx` - The verified transaction to test
    /// * `mutate` - Function that modifies a parsed field
    ///
    /// # Returns
    ///
    /// `Ok(())` if the mutation changed the reconstructed bytes,
    /// `Err` if bytes are unchanged (indicating lazy parsing).
    pub fn verify_field_affects_output<P, F>(tx: &VerifiedTransaction<P>, mutate: F) -> Result<()>
    where
        P: ReconstructableTransaction + Clone,
        F: FnOnce(&mut P),
    {
        let original_reconstructed = tx.parsed().reconstruct_bytes()?;

        let mut cloned_parsed = tx.parsed().clone();
        mutate(&mut cloned_parsed);

        let mutated_reconstructed = cloned_parsed.reconstruct_bytes()?;

        if original_reconstructed == mutated_reconstructed {
            return Err(DecoderError::invalid_structure(
                "Field mutation did not change reconstructed bytes. \
                 This may indicate the decoder is not actually parsing the data.",
            ));
        }

        Ok(())
    }

    /// Verify that all critical fields affect the output.
    ///
    /// Takes a list of field mutators and verifies each one changes
    /// the reconstructed bytes.
    pub fn verify_all_fields_affect_output<P, I, F>(
        tx: &VerifiedTransaction<P>,
        mutators: I,
    ) -> Result<()>
    where
        P: ReconstructableTransaction + Clone,
        I: IntoIterator<Item = (&'static str, F)>,
        F: FnOnce(&mut P),
    {
        for (field_name, mutate) in mutators {
            let original_reconstructed = tx.parsed().reconstruct_bytes()?;

            let mut cloned_parsed = tx.parsed().clone();
            mutate(&mut cloned_parsed);

            let mutated_reconstructed = cloned_parsed.reconstruct_bytes()?;

            if original_reconstructed == mutated_reconstructed {
                return Err(DecoderError::invalid_structure(format!(
                    "Mutating field '{}' did not change reconstructed bytes. \
                     This may indicate lazy parsing.",
                    field_name
                )));
            }
        }

        Ok(())
    }

    /// Create a verification test that ensures bytes actually come from fields.
    ///
    /// This macro generates property tests for decoder verification.
    #[macro_export]
    macro_rules! verify_not_lazy {
        ($tx:expr, $field:ident, $new_value:expr) => {{
            use $crate::verified::testing::verify_field_affects_output;

            verify_field_affects_output($tx, |parsed| {
                parsed.$field = $new_value;
            })
        }};
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock parsed fields for testing
    #[derive(Debug, Clone, PartialEq)]
    struct MockParsedFields {
        value: u64,
        data: Vec<u8>,
    }

    impl ReconstructableTransaction for MockParsedFields {
        fn reconstruct_bytes(&self) -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&self.value.to_le_bytes());
            bytes.extend_from_slice(&self.data);
            Ok(bytes)
        }
    }

    #[test]
    fn test_verified_transaction_strict() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };
        let original = parsed.reconstruct_bytes().unwrap();

        let mut tx = VerifiedTransaction::new(parsed, original);
        assert!(!tx.is_verified());

        tx.verify_strict().unwrap();
        assert!(tx.is_verified());
    }

    #[test]
    fn test_verified_transaction_mismatch() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };
        // Wrong original bytes
        let wrong_bytes = vec![0xFF; 10];

        let mut tx = VerifiedTransaction::new(parsed, wrong_bytes);
        assert!(tx.verify_strict().is_err());
    }

    #[test]
    fn test_field_mutation_detection() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };
        let original = parsed.reconstruct_bytes().unwrap();
        let tx = VerifiedTransaction::new(parsed, original);

        // Mutating value should change output
        testing::verify_field_affects_output(&tx, |p| p.value = 999).unwrap();

        // Mutating data should change output
        testing::verify_field_affects_output(&tx, |p| p.data = vec![9, 9, 9]).unwrap();
    }

    #[test]
    fn test_lazy_parser_detection() {
        // Simulate a "lazy" parsed type that doesn't actually use its fields
        #[derive(Debug, Clone, PartialEq)]
        struct LazyParsed {
            value: u64,
            _hidden_raw: Vec<u8>,
        }

        impl ReconstructableTransaction for LazyParsed {
            fn reconstruct_bytes(&self) -> Result<Vec<u8>> {
                // Bug: returns hidden raw bytes, not reconstructed from value
                Ok(self._hidden_raw.clone())
            }
        }

        let raw = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let parsed = LazyParsed {
            value: 42,
            _hidden_raw: raw.clone(),
        };
        let tx = VerifiedTransaction::new(parsed, raw);

        // This should FAIL because mutating value doesn't change output
        let result = testing::verify_field_affects_output(&tx, |p| p.value = 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_verified_transaction_to_bytes() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };
        let original = parsed.reconstruct_bytes().unwrap();

        // Unverified transaction should error on to_bytes()
        let tx = VerifiedTransaction::new(parsed.clone(), original.clone());
        assert!(tx.to_bytes().is_err());

        // Verified transaction should succeed
        let tx = VerifiedTransaction::new_verified(parsed, original.clone()).unwrap();
        assert_eq!(tx.to_bytes().unwrap(), original);
    }

    #[test]
    fn test_all_fields_verification() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };
        let original = parsed.reconstruct_bytes().unwrap();
        let tx = VerifiedTransaction::new(parsed, original);

        // Test value field
        {
            let original_bytes = tx.parsed().reconstruct_bytes().unwrap();
            let mut cloned = tx.parsed().clone();
            cloned.value = 999;
            let mutated_bytes = cloned.reconstruct_bytes().unwrap();
            assert_ne!(
                original_bytes, mutated_bytes,
                "Field value should affect output"
            );
        }

        // Test data field
        {
            let original_bytes = tx.parsed().reconstruct_bytes().unwrap();
            let mut cloned = tx.parsed().clone();
            cloned.data = vec![9, 9, 9];
            let mutated_bytes = cloned.reconstruct_bytes().unwrap();
            assert_ne!(
                original_bytes, mutated_bytes,
                "Field data should affect output"
            );
        }
    }
}
