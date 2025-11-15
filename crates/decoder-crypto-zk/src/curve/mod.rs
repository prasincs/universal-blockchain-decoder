//! STARK curve primitives
//!
//! This module provides constants and parameters for the STARK elliptic curve
//! used in Starknet and related ZK systems.
//!
//! The STARK curve is defined by the equation:
//! ```text
//! y^2 = x^3 + alpha * x + beta
//! ```
//!
//! Where:
//! - Field prime: `p = 2^251 + 17 * 2^192 + 1`
//! - Curve order: `0x0800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f`
//! - Alpha: `1`
//! - Beta: `0x06f21413efbe40de150e596d72f7a8c5609ad26c15c915c1f4cdfcb99cee9e89`

pub mod stark;

pub use stark::*;
