//! Zcash Viewing Key Infrastructure
//!
//! This module provides viewing key types and decryption logic for Zcash shielded transactions.
//!
//! ## Overview
//!
//! Viewing keys allow holders to decrypt shielded transaction data without having spending authority.
//! This enables:
//! - Transaction monitoring (balance tracking, incoming payments)
//! - Auditing and compliance (selective disclosure)
//! - Block explorers (with user-provided keys)
//!
//! ## Supported Protocols
//!
//! - **Sapling**: Full support for incoming viewing keys (IVK) and note decryption
//! - **Orchard**: Future support (Phase 4)
//!
//! ## Security Model
//!
//! **Viewing keys are sensitive data**:
//! - **Incoming Viewing Key (IVK)**: Reveals all received transactions and amounts
//! - **Outgoing Viewing Key (OVK)**: Reveals all sent transactions and amounts
//! - **Full Viewing Key (FVK)**: Combines IVK + OVK + nullifier deriving key
//!
//! This module **never** handles spending keys. All decryption is read-only.
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use decoder_zcash::viewing_key::{SaplingIncomingViewingKey, NotePlaintext};
//! use decoder_zcash::sapling::OutputDescription;
//!
//! // Parse a shielded output from transaction
//! let output: OutputDescription = /* ... */;
//!
//! // User provides their incoming viewing key
//! let ivk = SaplingIncomingViewingKey::from_bytes(&ivk_bytes)?;
//!
//! // Attempt decryption
//! match output.try_decrypt(&ivk)? {
//!     Some(plaintext) => {
//!         println!("Decrypted! Value: {} zatoshis", plaintext.value);
//!         println!("Memo: {}", String::from_utf8_lossy(&plaintext.memo));
//!     }
//!     None => {
//!         println!("Not addressed to this viewing key");
//!     }
//! }
//! ```
//!
//! ## Cryptographic Dependencies
//!
//! - **jubjub**: Sapling elliptic curve (ECDH key agreement)
//! - **chacha20poly1305**: ChaCha20-Poly1305 AEAD (note decryption)
//! - **blake2b**: Key derivation and hashing
//!
//! ## References
//!
//! - [ZIP-32: Shielded Hierarchical Deterministic Wallets](https://zips.z.cash/zip-0032)
//! - [ZIP-316: Unified Addresses and Unified Viewing Keys](https://zips.z.cash/zip-0316)
//! - [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf) - Section 4.19 (Note Encryption)

mod decrypt;
mod types;

pub use types::{NotePlaintext, SaplingFullViewingKey, SaplingIncomingViewingKey};

pub use decrypt::{decrypt_sapling_note, DecryptionError};
