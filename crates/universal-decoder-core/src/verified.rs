//! Type-safe decoder verification module.
//!
//! This module provides the trait that enforces actual parsing of transactions
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
//! Transaction types must NOT store raw bytes. Instead they implement
//! [`ReconstructableTransaction`] and rebuild their byte representation from
//! parsed semantic fields. Combined with strict (canonical-only) decoding,
//! this guarantees the injective property is satisfied through genuine
//! parsing:
//!
//! ```text
//! ∀ tx_bytes: decode(tx_bytes) succeeds ⟹ decode(tx_bytes).to_bytes() == tx_bytes
//! ```
//!
//! The [`testing`] module provides utilities that detect lazy implementations
//! by checking that mutating parsed fields actually changes the output bytes.

use crate::error::{DecoderError, Result};

/// Trait for transactions that can reconstruct their byte representation from parsed fields.
///
/// This trait is the key to enforcing actual parsing. Implementors MUST reconstruct
/// bytes from their parsed semantic fields, NOT from stored raw bytes. Transaction
/// types implementing this trait must not have a raw-bytes field at all.
///
/// # Requirements
///
/// - MUST reconstruct bytes from semantic fields only
/// - MUST NOT rely on stored raw bytes
/// - MUST produce the canonical encoding, so that strict decoding of the
///   result yields equivalent fields
pub trait ReconstructableTransaction: Sized {
    /// Reconstruct the original byte representation from parsed fields.
    ///
    /// # Formal Property
    ///
    /// For a correct implementation together with a strict (canonical-only)
    /// decoder:
    /// ```text
    /// ∀ tx_bytes: parse(tx_bytes)?.reconstruct_bytes()? == tx_bytes
    /// ```
    fn reconstruct_bytes(&self) -> Result<Vec<u8>>;

    /// Verify that reconstruction reproduces the given original bytes exactly.
    ///
    /// This is the runtime check of the injective property for a single
    /// transaction. Decoder test suites should call this for every fixture.
    fn verify_reconstruction(&self, original_bytes: &[u8]) -> Result<()> {
        let reconstructed = self.reconstruct_bytes()?;
        if reconstructed != original_bytes {
            return Err(DecoderError::invalid_structure(format!(
                "Reconstruction mismatch: reconstructed {} bytes, original {} bytes",
                reconstructed.len(),
                original_bytes.len()
            )));
        }
        Ok(())
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
    /// * `parsed` - The parsed transaction to test
    /// * `mutate` - Function that modifies a parsed field
    ///
    /// # Returns
    ///
    /// `Ok(())` if the mutation changed the reconstructed bytes,
    /// `Err` if bytes are unchanged (indicating lazy parsing).
    pub fn verify_field_affects_output<P, F>(parsed: &P, mutate: F) -> Result<()>
    where
        P: ReconstructableTransaction + Clone,
        F: FnOnce(&mut P),
    {
        let original_reconstructed = parsed.reconstruct_bytes()?;

        let mut mutated = parsed.clone();
        mutate(&mut mutated);

        let mutated_reconstructed = mutated.reconstruct_bytes()?;

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
    /// Takes a list of named field mutators and verifies each one changes
    /// the reconstructed bytes.
    pub fn verify_all_fields_affect_output<P, I, F>(parsed: &P, mutators: I) -> Result<()>
    where
        P: ReconstructableTransaction + Clone,
        I: IntoIterator<Item = (&'static str, F)>,
        F: FnOnce(&mut P),
    {
        for (field_name, mutate) in mutators {
            let original_reconstructed = parsed.reconstruct_bytes()?;

            let mut mutated = parsed.clone();
            mutate(&mut mutated);

            let mutated_reconstructed = mutated.reconstruct_bytes()?;

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
    fn test_verify_reconstruction_matches() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };
        let original = parsed.reconstruct_bytes().unwrap();

        assert!(parsed.verify_reconstruction(&original).is_ok());
    }

    #[test]
    fn test_verify_reconstruction_mismatch() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };
        let wrong_bytes = vec![0xFF; 10];

        assert!(parsed.verify_reconstruction(&wrong_bytes).is_err());
    }

    #[test]
    fn test_field_mutation_detection() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };

        // Mutating value should change output
        testing::verify_field_affects_output(&parsed, |p| p.value = 999).unwrap();

        // Mutating data should change output
        testing::verify_field_affects_output(&parsed, |p| p.data = vec![9, 9, 9]).unwrap();
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
            _hidden_raw: raw,
        };

        // This should FAIL because mutating value doesn't change output
        let result = testing::verify_field_affects_output(&parsed, |p| p.value = 999);
        assert!(result.is_err());
    }

    type FieldMutator = fn(&mut MockParsedFields);

    #[test]
    fn test_all_fields_verification() {
        let parsed = MockParsedFields {
            value: 42,
            data: vec![1, 2, 3],
        };

        let mutators: Vec<(&'static str, FieldMutator)> = vec![
            ("value", |p| p.value = 999),
            ("data", |p| p.data = vec![9, 9, 9]),
        ];

        testing::verify_all_fields_affect_output(&parsed, mutators).unwrap();
    }
}
