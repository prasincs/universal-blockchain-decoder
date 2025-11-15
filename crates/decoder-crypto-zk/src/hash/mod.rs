//! Cryptographic hash functions for ZK systems

pub mod pedersen;
pub mod poseidon;

pub use pedersen::PedersenHash;
pub use poseidon::PoseidonHash;
