//! Field arithmetic for ZK systems
//!
//! This module provides field arithmetic implementations for different ZK systems:
//!
//! - **STARK Field**: 252-bit field for Starknet (p = 2^251 + 17 * 2^192 + 1)
//! - **Pallas Field**: 255-bit field for Mina Protocol (Pasta curves)
//! - **Goldilocks Field**: 64-bit field for Polygon zkEVM (p = 2^64 - 2^32 + 1)
//!
//! Each field supports standard arithmetic operations and is designed for
//! zero-knowledge proof systems.

pub mod goldilocks;
pub mod pallas;
pub mod stark;

// Re-export commonly used types
pub use goldilocks::GoldilocksFieldElement;
pub use pallas::PallasFieldElement;
pub use stark::{FieldElement, FieldExt};
