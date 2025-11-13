//! Common utilities and patterns for blockchain decoder implementations.
//!
//! This crate provides shared functionality used across multiple decoder crates,
//! reducing code duplication and ensuring consistency.
//!
//! # Modules
//!
//! - [`validation`] - Standard validation functions for transaction format
//! - [`hooks`] - Helper functions for hook execution
//! - [`hashing`] - Common cryptographic hash functions
//! - [`chains`] - Pre-defined chain identity registry
//!
//! # Example
//!
//! ```rust
//! use decoder_chains_common::prelude::*;
//! use universal_decoder_core::prelude::*;
//!
//! # fn example() -> Result<()> {
//! # let raw_bytes = b"some transaction data";
//! # let data = b"some data";
//! // Use standard validation
//! validation::validate_not_empty(raw_bytes, "Bitcoin")?;
//! validation::validate_size_bounds(raw_bytes, 10, 100_000, "Bitcoin")?;
//!
//! // Use standard hashing
//! let hash = hashing::sha256_double(data);
//!
//! // Use pre-defined chain identities
//! let bitcoin = chains::BITCOIN;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod chains;
pub mod hashing;
pub mod hooks;
pub mod validation;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::chains;
    pub use crate::hashing;
    pub use crate::hooks;
    pub use crate::validation;
}
