//! Common testing utilities for blockchain decoder implementations
//!
//! This crate provides reusable test assertions, property-based testing helpers,
//! and fixture loading utilities to reduce boilerplate and standardize testing
//! across all decoder implementations.
//!
//! # Modules
//!
//! - `assertions` - Common test assertions (decode safety, roundtrip testing)
//! - `proptest_helpers` - Property-based testing utilities
//! - `fixtures` - Test fixture loading and management
//!
//! # Examples
//!
//! ## Testing Decode Safety
//!
//! ```rust,no_run
//! use decoder_test_utils::assertions::assert_decode_never_panics;
//! use universal_decoder_core::prelude::*;
//!
//! #[test]
//! fn test_decoder_never_panics() {
//!     let random_bytes = vec![0xFF; 1000];
//!     // This will catch_unwind and ensure decode returns Result::Err
//!     // instead of panicking
//!     assert_decode_never_panics::<MyChainDecoder>(&random_bytes);
//! }
//! ```
//!
//! ## Testing Canonical Serialization
//!
//! ```rust,no_run
//! use decoder_test_utils::assertions::assert_canonical_roundtrip;
//!
//! #[test]
//! fn test_canonical_serialization() {
//!     let tx = create_test_transaction();
//!     // Ensures: to_canonical_bytes() is deterministic
//!     // and from_canonical_bytes(to_canonical_bytes(tx)) == tx
//!     assert_canonical_roundtrip(&tx);
//! }
//! ```
//!
//! ## Loading Test Fixtures
//!
//! ```rust,no_run
//! use decoder_test_utils::fixtures::load_fixture;
//!
//! #[test]
//! fn test_with_fixture() {
//!     let fixture = load_fixture("tests/fixtures/bitcoin/genesis_coinbase.json");
//!     let tx_bytes = fixture.raw_bytes();
//!     // ... test with real transaction data
//! }
//! ```

pub mod assertions;
pub mod fixtures;
pub mod proptest_helpers;

// Re-export commonly used items
pub use assertions::{
    assert_canonical_roundtrip, assert_decode_encode_roundtrip, assert_decode_never_panics,
};
pub use fixtures::{load_fixture, load_fixtures_dir, TestFixture};
pub use proptest_helpers::{arbitrary_transaction_bytes, canonical_serialization_properties};
