//! Dash transaction decoder
//!
//! Dash is a Bitcoin fork with additional features like InstantSend, PrivateSend,
//! and masternodes. Most transactions use the same format as Bitcoin.
//!
//! ## Differences from Bitcoin
//!
//! - Different address prefix (X for standard, 7 for P2SH)
//! - X11 PoW algorithm (doesn't affect transaction format)
//! - Special transaction types (v3+): ProRegTx, ProUpServTx, etc.
//! - Most regular transactions identical to Bitcoin
//!
//! ## Implementation Strategy
//!
//! For now, we implement basic Dash transaction decoding that handles regular
//! transactions (v1-v2) which are identical to Bitcoin. Special transaction types
//! (v3+) with extra payloads can be added in future versions.

use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use decoder_primitives::prelude::*;

/// Dash chain identity (re-export from common library)
pub use decoder_chains_common::chains::DASH as DashChain;

/// Dash decoder - wrapper around Bitcoin decoder
///
/// For regular Dash transactions (v1-v2), we use the Bitcoin decoder.
/// Special transaction types (v3+) with mastern node/governance payloads
/// will be added in future versions.
pub struct DashDecoder;

impl ChainDecoder for DashDecoder {
    type TxSpecific = BitcoinTransaction;
    type Chain = decoder_chains_common::chains::ChainInfo;

    fn chain() -> Self::Chain {
        decoder_chains_common::chains::DASH
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // For regular Dash transactions (v1-v2), format is identical to Bitcoin
        // TODO: Add support for special transaction types (v3+) with extra payloads
        BitcoinDecoder::decode(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Use Bitcoin's validation logic
        BitcoinDecoder::validate_format(raw_bytes)
    }
}

/// Dash-specific canonicalizer wrapper
pub struct DashCanonicalizer<'a> {
    tx: &'a BitcoinTransaction,
}

impl<'a> DashCanonicalizer<'a> {
    pub fn new(tx: &'a BitcoinTransaction) -> Self {
        Self { tx }
    }

    pub fn canonicalize(&self) -> Result<TxIR<'a, 1>> {
        // Reuse Bitcoin's canonicalization logic
        let mut tx_ir = self.tx.canonicalize()?;

        // Override chain to Dash
        tx_ir.chain = (&decoder_chains_common::chains::DASH).into();

        Ok(tx_ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = DashDecoder::chain();
        assert_eq!(chain.chain_id(), 5);
        assert_eq!(chain.chain_name(), "Dash");
        assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    }

    #[test]
    fn test_decode_minimal_legacy_transaction() {
        let mut tx_bytes = vec![];
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());
        tx_bytes.push(0x01);
        tx_bytes.extend_from_slice(&[0u8; 32]);
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        tx_bytes.push(0x01);
        tx_bytes.push(0x00);
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        tx_bytes.push(0x01);
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes());
        tx_bytes.push(0x00);
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());

        let decoded = DashDecoder::decode(&tx_bytes).expect("Failed to decode minimal transaction");

        assert_eq!(decoded.version(), 1);
        assert_eq!(decoded.input_count(), 1);
        assert_eq!(decoded.output_count(), 1);
        assert!(decoded.is_coinbase());
    }

    #[test]
    fn test_canonicalize_uses_dash_chain() {
        let mut tx_bytes = vec![];
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());
        tx_bytes.push(0x01);
        tx_bytes.extend_from_slice(&[0u8; 32]);
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        tx_bytes.push(0x01);
        tx_bytes.push(0x00);
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        tx_bytes.push(0x01);
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes());
        tx_bytes.push(0x00);
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());

        let tx = DashDecoder::decode(&tx_bytes).unwrap();
        let canonicalizer = DashCanonicalizer::new(&tx);
        let tx_ir = canonicalizer.canonicalize().unwrap();

        assert_eq!(tx_ir.chain.id, 5);
        assert_eq!(tx_ir.chain.name, "Dash");
    }
}
