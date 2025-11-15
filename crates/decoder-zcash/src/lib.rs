//! Zcash transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for Zcash transactions, transforming them
//! from their native format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. The `zcash_primitives` crate is used only in
//! `dev-dependencies` for validation testing.
//!
//! ## Transaction Format Support
//!
//! ### Phase 1 (Current): Transparent Transactions
//! - ✅ Transparent-to-transparent (t→t) transactions
//! - ✅ Bitcoin-compatible UTXO model
//! - ✅ Zcash-specific fields (version_group_id, expiry_height)
//!
//! ### Phase 2 (Planned): Sapling Shielded Transactions
//! - ⏳ Sapling spend descriptions
//! - ⏳ Sapling output descriptions
//! - ⏳ zk-SNARK proofs (parsing, not verification)
//! - ⏳ Shielding (t→z), deshielding (z→t), fully shielded (z→z)
//!
//! ### Phase 3 (Planned): Viewing Key Decryption
//! - ⏳ Sapling viewing key decryption
//! - ⏳ Note plaintext parsing
//!
//! ### Phase 4 (Planned): Orchard Support
//! - ⏳ Orchard action descriptions
//! - ⏳ Halo2 proof structures
//! - ⏳ Orchard viewing key decryption
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_zcash::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "0400008085202f89...";  // Zcash transparent transaction
//! let tx_bytes = universal_decoder_core::hex::decode(tx_hex)?;
//!
//! let decoded = ZcashDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```
//!
//! ## Privacy Features
//!
//! For transparent transactions, privacy metadata will indicate `FullyObservable`.
//! For shielded transactions (Phase 2+), privacy metadata will be populated with:
//! - `HiddenSender` (for shielded spends)
//! - `HiddenRecipient` (for shielded outputs)
//! - `HiddenAmount` (for confidential amounts)
//! - `ObservabilityLevel::FullyPrivate` or `PartiallyObservable`

use decoder_primitives::prelude::*;
use std::io::Cursor;

pub mod parsing;
pub mod transparent;
pub mod types;

use parsing::*;
pub use types::ZcashTransaction;

/// Zcash mainnet chain identity
pub use decoder_chains_common::chains::ZCASH as ZcashChain;

/// Zcash decoder implementing the ChainDecoder trait
///
/// This decoder uses a pure Rust implementation to parse Zcash transactions
/// without depending on external blockchain libraries in production.
///
/// Currently supports: Transparent transactions (Phase 1)
/// Planned: Sapling, Orchard, Viewing Keys (Phases 2-4)
pub struct ZcashDecoder;

impl ChainDecoder for ZcashDecoder {
    type TxSpecific = ZcashTransaction;
    type Chain = decoder_chains_common::chains::ChainInfo;

    fn chain() -> Self::Chain {
        decoder_chains_common::chains::ZCASH
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        let mut cursor = Cursor::new(raw_bytes);

        // Parse transaction header (version + version_group_id detection)
        let (version, version_group_id) = parse_zcash_header(&mut cursor)?;

        // Determine transaction type based on version
        let tx = match version {
            1..=3 => {
                // Pre-Sapling (Sprout) - Not supported in Phase 1
                return Err(DecoderError::chain_specific(format!(
                    "Zcash Sprout transactions (version {}) not yet supported. Supported: v4 (Sapling), v5 (Orchard)",
                    version
                )));
            }
            4 => {
                // Sapling or later (Overwinter bit set)
                // Phase 1: Only transparent transactions
                // Phase 2+: Full Sapling support
                parse_zcash_v4_transaction(&mut cursor, version, version_group_id)?
            }
            5 => {
                // Orchard (NU5+)
                // Phase 4: Full Orchard support
                return Err(DecoderError::chain_specific(
                    "Zcash Orchard transactions (version 5) not yet supported. Phase 4 feature."
                        .to_string(),
                ));
            }
            _ => {
                return Err(DecoderError::chain_specific(format!(
                    "Unknown Zcash transaction version: {}",
                    version
                )));
            }
        };

        Ok(tx)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Minimum transaction size: version(4) + vin_count(1) + vout_count(1) + locktime(4) + expiry_height(4) = 14 bytes
        if raw_bytes.len() < 14 {
            return Err(DecoderError::invalid_structure(format!(
                "Zcash transaction too small: {} bytes (minimum 14)",
                raw_bytes.len()
            )));
        }

        // Maximum transaction size (100KB per BIP consensus)
        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Zcash transaction too large: {} bytes (maximum {})",
                raw_bytes.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
    }
}

// Maximum transaction size (100KB)
const MAX_TRANSACTION_SIZE: usize = 100_000;
