//! Bitcoin transaction decoder - Pure Rust implementation
//!
//! This module provides a decoder for Bitcoin transactions, transforming them
//! from their native format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! This decoder is implemented in **pure Rust** with **zero production dependencies**
//! on external blockchain libraries. The `bitcoin` crate is used only in
//! `dev-dependencies` for validation testing.
//!
//! ## Transaction Format Support
//!
//! - ✅ Legacy transactions (pre-SegWit)
//! - ✅ SegWit transactions (BIP 141, 143, 144)
//! - ✅ Coinbase transactions
//! - ✅ P2PKH, P2SH, P2WPKH, P2WSH scripts
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_bitcoin::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "01000000...";
//! let tx_bytes = hex::decode(tx_hex)?;
//!
//! let decoded = BitcoinDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

use decoder_primitives::prelude::*;
use std::io::Cursor;
use universal_decoder_core::prelude::*;

pub mod parsing;
pub mod types;

use parsing::*;
use types::BitcoinTransaction;

/// Bitcoin chain identity
#[derive(Debug, Clone, Copy)]
pub struct BitcoinChain;

impl ChainIdentity for BitcoinChain {
    fn chain_id(&self) -> u64 {
        0 // Bitcoin chain ID
    }

    fn chain_name(&self) -> &str {
        "Bitcoin"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

/// Bitcoin decoder implementing the ChainDecoder trait
///
/// This decoder uses a pure Rust implementation to parse Bitcoin transactions
/// without depending on external blockchain libraries in production.
pub struct BitcoinDecoder;

impl ChainDecoder for BitcoinDecoder {
    type TxSpecific = BitcoinTransaction;
    type Chain = BitcoinChain;

    fn chain() -> Self::Chain {
        BitcoinChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Validate format first
        Self::validate_format(raw_bytes)?;

        let mut cursor = Cursor::new(raw_bytes);

        // Parse version (4 bytes, little-endian)
        let version = read_u32_le(&mut cursor)?;

        // Detect SegWit by checking for marker and flag
        let is_segwit = detect_segwit(raw_bytes, cursor.position() as usize)?;

        // Skip marker (0x00) and flag (0x01) if SegWit
        if is_segwit {
            let _marker = read_u8(&mut cursor)?;
            let _flag = read_u8(&mut cursor)?;
        }

        // Parse input count (varint)
        let input_count = read_varint(&mut cursor)?;
        if input_count > MAX_INPUTS_OUTPUTS as u64 {
            return Err(DecoderError::invalid_structure(format!(
                "Too many inputs: {}",
                input_count
            )));
        }

        // Parse inputs
        let mut inputs = Vec::with_capacity(input_count as usize);
        for i in 0..input_count {
            inputs.push(parse_input(&mut cursor).map_err(|e| {
                DecoderError::chain_decoding(format!("Failed to parse input {}: {}", i, e))
            })?);
        }

        // Parse output count (varint)
        let output_count = read_varint(&mut cursor)?;
        if output_count > MAX_INPUTS_OUTPUTS as u64 {
            return Err(DecoderError::invalid_structure(format!(
                "Too many outputs: {}",
                output_count
            )));
        }

        // Parse outputs
        let mut outputs = Vec::with_capacity(output_count as usize);
        for i in 0..output_count {
            outputs.push(parse_output(&mut cursor).map_err(|e| {
                DecoderError::chain_decoding(format!("Failed to parse output {}: {}", i, e))
            })?);
        }

        // Parse witness data (if SegWit)
        let witnesses = if is_segwit {
            parse_witnesses(&mut cursor, inputs.len())?
        } else {
            vec![Witness::empty(); inputs.len()]
        };

        // Parse locktime (4 bytes, little-endian)
        let locktime = read_u32_le(&mut cursor)?;

        // Verify we consumed all bytes
        let consumed = cursor.position() as usize;
        if consumed != raw_bytes.len() {
            return Err(DecoderError::invalid_structure(format!(
                "Transaction has {} trailing bytes (consumed {}, total {})",
                raw_bytes.len() - consumed,
                consumed,
                raw_bytes.len()
            )));
        }

        Ok(BitcoinTransaction {
            version,
            inputs,
            outputs,
            witnesses,
            locktime,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction cannot be empty",
            ));
        }

        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(format!(
                "Bitcoin transaction too small: {} bytes (minimum 10 bytes)",
                raw_bytes.len()
            )));
        }

        if raw_bytes.len() > MAX_TRANSACTION_SIZE {
            return Err(DecoderError::invalid_structure(format!(
                "Bitcoin transaction too large: {} bytes (maximum {} bytes)",
                raw_bytes.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
    }
}

/// Detect if transaction uses SegWit format
///
/// SegWit transactions have a marker (0x00) and flag (0x01) immediately after the version.
/// However, we need to be careful because a legacy transaction could have 0x00 as the
/// first byte of input count varint (meaning 0 inputs, which is invalid but we should
/// handle gracefully).
///
/// The combination of marker=0x00 and flag=0x01 is the definitive indicator of SegWit.
fn detect_segwit(bytes: &[u8], offset: usize) -> Result<bool> {
    if offset + 2 > bytes.len() {
        return Ok(false);
    }

    let marker = bytes[offset];
    let flag = bytes[offset + 1];

    // SegWit has marker=0x00, flag=0x01
    Ok(marker == 0x00 && flag == 0x01)
}

/// Helper function to decode a Bitcoin transaction with hooks
pub fn decode_with_hooks(raw_bytes: &[u8], registry: &HookRegistry) -> Result<BitcoinTransaction> {
    // Execute pre-decode hooks
    let context = HookContext::new(HookStage::PreDecode, raw_bytes);
    match registry.execute_stage(&context)? {
        HookResult::Abort(msg) => {
            return Err(DecoderError::hook_execution(msg));
        }
        HookResult::Skip | HookResult::Continue | HookResult::ContinueWithMetadata(_) => {}
    }

    // Perform decoding
    let tx = BitcoinDecoder::decode(raw_bytes)?;

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
        assert!(BitcoinDecoder::validate_format(&[]).is_err());

        // Too small transaction should fail
        assert!(BitcoinDecoder::validate_format(&[0x01]).is_err());

        // Reasonable size should pass basic validation
        let dummy_tx = vec![0u8; 100];
        assert!(BitcoinDecoder::validate_format(&dummy_tx).is_ok());
    }

    #[test]
    fn test_validate_format_too_large() {
        let huge_tx = vec![0u8; MAX_TRANSACTION_SIZE + 1];
        assert!(BitcoinDecoder::validate_format(&huge_tx).is_err());
    }

    #[test]
    fn test_chain() {
        let chain = BitcoinDecoder::chain();
        assert_eq!(chain.chain_id(), 0);
        assert_eq!(chain.chain_name(), "Bitcoin");
        assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    }

    #[test]
    fn test_decode_minimal_legacy_transaction() {
        // Minimal valid legacy transaction: 1 input, 1 output
        let mut tx_bytes = vec![];

        // Version: 1
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());

        // Input count: 1
        tx_bytes.push(0x01);

        // Input 0:
        // - prev_hash (32 bytes, all zeros for coinbase)
        tx_bytes.extend_from_slice(&[0u8; 32]);
        // - prev_index (0xFFFFFFFF for coinbase)
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        // - script_sig length: 0
        tx_bytes.push(0x00);
        // - sequence
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());

        // Output count: 1
        tx_bytes.push(0x01);

        // Output 0:
        // - value (50 BTC in satoshis)
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes());
        // - script_pubkey length: 0
        tx_bytes.push(0x00);

        // Locktime: 0
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());

        let decoded =
            BitcoinDecoder::decode(&tx_bytes).expect("Failed to decode minimal transaction");

        assert_eq!(decoded.version(), 1);
        assert_eq!(decoded.input_count(), 1);
        assert_eq!(decoded.output_count(), 1);
        assert_eq!(decoded.locktime, 0);
        assert!(decoded.is_coinbase());
        assert!(!decoded.is_segwit());
    }

    #[test]
    fn test_decode_with_hooks() {
        let registry = HookRegistryBuilder::new().with_size_limit(10000).build();

        // Use genesis coinbase transaction
        let tx_hex = include_str!("../tests/fixtures/btc_genesis_coinbase.hex");
        let tx_bytes = hex::decode(tx_hex.trim()).unwrap();

        let result = decode_with_hooks(&tx_bytes, &registry);
        assert!(
            result.is_ok(),
            "decode_with_hooks failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_decode_invalid_empty() {
        let empty = vec![];
        assert!(BitcoinDecoder::decode(&empty).is_err());
    }

    #[test]
    fn test_decode_invalid_truncated() {
        let truncated = vec![0x01, 0x00, 0x00]; // Only 3 bytes
        assert!(BitcoinDecoder::decode(&truncated).is_err());
    }

    #[test]
    fn test_detect_segwit() {
        // Legacy transaction (no marker/flag)
        let legacy = vec![0x01, 0x00, 0x00, 0x00, 0x01]; // version=1, input_count=1
        assert!(!detect_segwit(&legacy, 4).unwrap());

        // SegWit transaction (marker=0x00, flag=0x01)
        let segwit = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x01]; // version=1, marker=0x00, flag=0x01
        assert!(detect_segwit(&segwit, 4).unwrap());

        // Edge case: 0x00 but not followed by 0x01
        let not_segwit = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x02];
        assert!(!detect_segwit(&not_segwit, 4).unwrap());
    }
}
