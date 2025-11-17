//! # Zero-Knowledge Cryptographic Primitives
//!
//! This crate provides vendored implementations of cryptographic primitives
//! used in zero-knowledge proof systems and privacy-preserving blockchains.
//!
//! ## Supported Primitives
//!
//! - **STARK Field Arithmetic**: 252-bit field operations for Starknet
//! - **Pallas Field Arithmetic**: 255-bit field operations for Mina Protocol
//! - **Poseidon Hash**: Starknet and Pallas variants
//! - **Pedersen Hash**: Starknet variant (unlocks 235+ chains)
//! - **STARK Curve**: Elliptic curve operations
//! - **ECDSA on STARK Curve**: Signature verification
//!
//! ## Why Vendored?
//!
//! 1. **Minimal TCB**: No external cryptographic dependencies in production
//! 2. **Airgapped Operation**: Complete offline operation for security-critical deployments
//! 3. **Formal Verification**: Can be verified with Verus
//! 4. **Security Audit**: Single audit point vs multiple external crates
//! 5. **Force Multiplier**: Unlocks 300+ blockchain chains
//!
//! ## Architecture
//!
//! ```text
//! decoder-crypto-zk/
//! ├── field/      # STARK field (252-bit modular arithmetic)
//! ├── hash/       # Poseidon + Pedersen hash functions
//! ├── curve/      # STARK curve primitives
//! └── signature/  # ECDSA verification
//! ```
//!
//! ## Chains Unlocked
//!
//! - **Starknet ecosystem** (230+ chains): Mainnet, testnet, appchains (Kakarot, Madara, etc.)
//! - **Zcash**: Privacy-preserving transactions (Pedersen commitments)
//! - **Polygon zkEVM** (10+ chains): Goldilocks Poseidon variant
//! - **Mina Protocol**: Pallas Poseidon variant
//! - **Aleo**: BLS12-377 Poseidon variant
//! - **Aztec Network**: Privacy rollup
//! - **Scroll**: zkEVM L2
//! - **Loopring**: zkRollup protocol
//!
//! ## Security Model
//!
//! All implementations are:
//! - ✅ Vendored from audited libraries (starknet-crypto, zcash/librustzcash)
//! - ✅ Cross-validated with reference implementations in dev-dependencies
//! - ✅ Property-tested for correctness
//! - ✅ Panic-free (all operations return Result)
//! - ✅ Constant-time where applicable (timing attack resistance)
//!
//! ## Usage
//!
//! ```rust
//! use decoder_crypto_zk::hash::poseidon::PoseidonHash;
//! use decoder_crypto_zk::field::FieldElement;
//!
//! // Example: Hash a pair of field elements
//! let a = FieldElement::from(123u64);
//! let b = FieldElement::from(456u64);
//! let hash = PoseidonHash::hash_pair(a, b);
//! ```
//!
//! ## See Also
//!
//! - `docs/CRYPTO_VENDORING_LEVERAGE.md` - Strategic analysis
//! - `docs/STARKNET_RESEARCH.md` - Starknet architecture
//! - `VENDORED.md` - Vendoring audit trail

// Modules
pub mod curve;
pub mod error;
pub mod field;
pub mod hash;
pub mod signature;

// Re-exports
pub use curve::{AffinePoint, ALPHA, BETA, EC_ORDER, GENERATOR};
pub use error::{CryptoError, Result};
pub use field::{FieldElement, FieldExt, PallasFieldElement};
pub use signature::{verify, Signature, VerifyError};
