//! Field arithmetic for ZK systems
//!
//! This module provides field arithmetic implementations for different ZK systems:
//!
//! - **STARK Field**: 252-bit field for Starknet (p = 2^251 + 17 * 2^192 + 1)
//! - **Pallas Field**: 255-bit field for Mina Protocol (Pasta curves)
//!
//! Each field supports standard arithmetic operations and is designed for
//! zero-knowledge proof systems.

pub mod pallas;
pub mod stark;

// Re-export commonly used types
pub use pallas::PallasFieldElement;
pub use stark::{FieldElement, FieldExt};
