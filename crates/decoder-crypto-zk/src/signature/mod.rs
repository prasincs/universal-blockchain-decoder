//! ECDSA signature verification on the STARK curve
//!
//! This module provides signature verification for Starknet transactions.
//! For decoding purposes, we only implement verification (not signing).

pub mod ecdsa;

pub use ecdsa::{verify, Signature, VerifyError};
