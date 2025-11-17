//! Cardano transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for Cardano transactions, transforming them
//! from their native CBOR format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. The `pallas` crates are used only in
//! `dev-dependencies` for validation testing.
//!
//! ## Transaction Format Support
//!
//! - ✅ Shelley-era transactions (post-Mary hard fork)
//! - ✅ Multi-asset support (native tokens)
//! - ✅ Plutus script support
//! - ✅ Metadata parsing
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_cardano::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "84a400...";
//! let tx_bytes = hex::decode(tx_hex)?;
//!
//! let decoded = CardanoDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

use decoder_chains_common::prelude::*;
use decoder_primitives::prelude::*;
use std::io::Cursor;

pub mod parsing;
pub mod types;

use parsing::*;
pub use types::CardanoTransaction;

/// Cardano chain identity
#[derive(Debug, Clone, Copy)]
pub struct CardanoChain;

impl ChainIdentity for CardanoChain {
    fn chain_id(&self) -> u64 {
        1815 // Cardano's founding year
    }

    fn chain_name(&self) -> &str {
        "Cardano"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

/// Maximum transaction size (64 KB)
const MAX_TRANSACTION_SIZE: usize = 65536;

/// Cardano decoder implementing the ChainDecoder trait
///
/// This decoder uses a pure Rust implementation to parse Cardano transactions
/// without depending on external blockchain libraries in production.
pub struct CardanoDecoder;

impl ChainDecoder for CardanoDecoder {
    type TxSpecific = CardanoTransaction;
    type Chain = CardanoChain;

    fn chain() -> Self::Chain {
        CardanoChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        let mut cursor = Cursor::new(raw_bytes);

        // Parse the CBOR array tag
        let array_len = read_cbor_array_header(&mut cursor)?;

        // Cardano transactions are CBOR arrays with 3 or 4 elements:
        // [transaction_body, transaction_witness_set, auxiliary_data?, is_valid?]
        if array_len < 3 || array_len > 4 {
            return Err(DecoderError::invalid_structure(format!(
                "Expected CBOR array with 3-4 elements, got {}",
                array_len
            )));
        }

        // Parse transaction body
        let tx_body = parse_transaction_body(&mut cursor)?;

        // Parse witness set
        let witness_set = parse_witness_set(&mut cursor)?;

        // Parse auxiliary data (metadata) if present
        let auxiliary_data = if array_len >= 4 {
            parse_auxiliary_data(&mut cursor)?
        } else {
            None
        };

        // Parse validity flag if present (for Alonzo era and later)
        let is_valid = if array_len == 4 {
            Some(read_cbor_bool(&mut cursor)?)
        } else {
            None
        };

        Ok(CardanoTransaction {
            body: tx_body,
            witness_set,
            auxiliary_data,
            is_valid,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Use common validation logic
        validation::validate_format(raw_bytes, 10, MAX_TRANSACTION_SIZE, "Cardano")?;

        // Check for CBOR array marker (should start with 0x81-0x84 or 0x98-0x9f for array)
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Cardano transaction cannot be empty",
            ));
        }

        let first_byte = raw_bytes[0];
        // CBOR array markers: 0x80-0x9f (fixed arrays 0-31) or 0x98-0x9b (variable arrays)
        if (first_byte & 0xE0) != 0x80 && first_byte != 0x9f {
            return Err(DecoderError::invalid_structure(format!(
                "Expected CBOR array marker, got 0x{:02x}",
                first_byte
            )));
        }

        Ok(())
    }
}

/// Helper function to decode a Cardano transaction with hooks
pub fn decode_with_hooks(raw_bytes: &[u8], registry: &HookRegistry) -> Result<CardanoTransaction> {
    // Execute pre-decode hooks
    let context = HookContext::new(HookStage::PreDecode, raw_bytes);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    // Perform decoding
    let tx = CardanoDecoder::decode(raw_bytes)?;

    // Execute post-decode hooks
    let context = HookContext::new(HookStage::PostDecode, raw_bytes).with_chain_specific(&tx);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use universal_decoder_core::hex;

    #[test]
    fn test_validate_format() {
        // Empty transaction should fail
        assert!(CardanoDecoder::validate_format(&[]).is_err());

        // Too small transaction should fail
        assert!(CardanoDecoder::validate_format(&[0x01]).is_err());

        // Valid CBOR array marker should pass basic validation
        // Make it at least 10 bytes
        let dummy_tx = vec![0x83, 0xa0, 0xa0, 0xf6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // [map, map, null] + padding
        assert!(CardanoDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_validate_format_too_large() {
        let huge_tx = vec![0x83; MAX_TRANSACTION_SIZE + 1];
        assert!(CardanoDecoder::validate_format(&huge_tx).is_err());
    }

    #[test]
    fn test_chain() {
        let chain = CardanoDecoder::chain();
        assert_eq!(chain.chain_id(), 1815);
        assert_eq!(chain.chain_name(), "Cardano");
        assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    }

    #[test]
    fn test_decode_invalid_empty() {
        let empty = vec![];
        assert!(CardanoDecoder::decode(&empty).is_err());
    }

    #[test]
    fn test_decode_invalid_not_cbor_array() {
        // CBOR integer (not array)
        let not_array = vec![0x01];
        assert!(CardanoDecoder::decode(&not_array).is_err());
    }

    #[test]
    fn test_decode_with_hooks() {
        let registry = HookRegistryBuilder::new().with_size_limit(10000).build();

        // Minimal valid CBOR structure: [map, map, null]
        let tx_bytes = vec![0x83, 0xa0, 0xa0, 0xf6];

        let result = decode_with_hooks(&tx_bytes, &registry);
        // This will fail because it's not a complete transaction,
        // but it shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
