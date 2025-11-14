//! Bitcoin SV transaction decoder
//!
//! Bitcoin SV (Satoshi Vision) is a Bitcoin Cash fork that further increased block sizes
//! and restored some original Bitcoin opcodes. This decoder reuses the Bitcoin decoder.
//!
//! ## Differences from Bitcoin
//!
//! - **No SegWit support** (removed in Bitcoin Cash fork)
//! - Unlimited block size (removed 32 MB cap from Bitcoin Cash)
//! - Restored original Bitcoin opcodes
//! - Transaction format is identical to Bitcoin legacy transactions
//!
//! ## Implementation Strategy
//!
//! Bitcoin SV transactions use the **exact same format** as Bitcoin legacy transactions.
//! We reuse `BitcoinDecoder` and only override the chain identity.

use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use decoder_primitives::prelude::*;

/// Bitcoin SV chain identity (re-export from common library)
pub use decoder_chains_common::chains::BITCOIN_SV as BitcoinSvChain;

/// Bitcoin SV decoder - wrapper around Bitcoin decoder
///
/// Since Bitcoin SV uses identical transaction format to Bitcoin legacy transactions
/// (no SegWit), we simply delegate to the Bitcoin decoder and override the chain identity.
pub struct BitcoinSvDecoder;

impl ChainDecoder for BitcoinSvDecoder {
    type TxSpecific = BitcoinTransaction;
    type Chain = decoder_chains_common::chains::ChainInfo;

    fn chain() -> Self::Chain {
        decoder_chains_common::chains::BITCOIN_SV
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Bitcoin SV uses identical transaction format to Bitcoin (legacy only, no SegWit)
        BitcoinDecoder::decode(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Use Bitcoin's validation logic
        BitcoinDecoder::validate_format(raw_bytes)
    }
}

/// Bitcoin SV-specific canonicalizer wrapper
pub struct BitcoinSvCanonicalizer<'a> {
    tx: &'a BitcoinTransaction,
}

impl<'a> BitcoinSvCanonicalizer<'a> {
    pub fn new(tx: &'a BitcoinTransaction) -> Self {
        Self { tx }
    }

    pub fn canonicalize(&self) -> Result<TxIR<'a, 1>> {
        // Reuse Bitcoin's canonicalization logic
        let mut tx_ir = self.tx.canonicalize()?;

        // Override chain to Bitcoin SV
        tx_ir.chain = (&decoder_chains_common::chains::BITCOIN_SV).into();

        Ok(tx_ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = BitcoinSvDecoder::chain();
        assert_eq!(chain.chain_id(), 236);
        assert_eq!(chain.chain_name(), "Bitcoin SV");
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

        let decoded =
            BitcoinSvDecoder::decode(&tx_bytes).expect("Failed to decode minimal transaction");

        assert_eq!(decoded.version(), 1);
        assert_eq!(decoded.input_count(), 1);
        assert_eq!(decoded.output_count(), 1);
        assert!(decoded.is_coinbase());
        assert!(!decoded.is_segwit());
    }

    #[test]
    fn test_canonicalize_uses_bitcoin_sv_chain() {
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

        let tx = BitcoinSvDecoder::decode(&tx_bytes).unwrap();
        let canonicalizer = BitcoinSvCanonicalizer::new(&tx);
        let tx_ir = canonicalizer.canonicalize().unwrap();

        assert_eq!(tx_ir.chain.id, 236);
        assert_eq!(tx_ir.chain.name, "Bitcoin SV");
    }
}
