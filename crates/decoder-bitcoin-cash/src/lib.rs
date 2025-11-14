//! Bitcoin Cash transaction decoder
//!
//! Bitcoin Cash is a Bitcoin fork that removed SegWit support and increased block size.
//! This decoder reuses the Bitcoin decoder with Bitcoin Cash-specific chain ID.
//!
//! ## Differences from Bitcoin
//!
//! - **No SegWit support** (removed in fork)
//! - Larger block size (8 MB → 32 MB)
//! - CashAddr address format (different from Bitcoin)
//! - Some script opcode differences (OP_CHECKDATASIG, OP_REVERSEBYTES)
//! - Transaction format is identical to Bitcoin legacy transactions
//!
//! ## Implementation Strategy
//!
//! Bitcoin Cash transactions use the **exact same format** as Bitcoin legacy transactions.
//! We reuse `BitcoinDecoder` and only override the chain identity.

use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use decoder_primitives::prelude::*;

/// Bitcoin Cash chain identity (re-export from common library)
pub use decoder_chains_common::chains::BITCOIN_CASH as BitcoinCashChain;

/// Bitcoin Cash decoder - wrapper around Bitcoin decoder
///
/// Since Bitcoin Cash uses identical transaction format to Bitcoin legacy transactions
/// (no SegWit), we simply delegate to the Bitcoin decoder and override the chain identity.
pub struct BitcoinCashDecoder;

impl ChainDecoder for BitcoinCashDecoder {
    type TxSpecific = BitcoinTransaction;
    type Chain = decoder_chains_common::chains::ChainInfo;

    fn chain() -> Self::Chain {
        decoder_chains_common::chains::BITCOIN_CASH
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Bitcoin Cash uses identical transaction format to Bitcoin (legacy only, no SegWit)
        BitcoinDecoder::decode(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Use Bitcoin's validation logic
        BitcoinDecoder::validate_format(raw_bytes)
    }
}

/// Bitcoin Cash-specific canonicalizer wrapper
///
/// This creates a Bitcoin Cash-specific canonicalizer that uses the Bitcoin Cash chain identity
/// instead of Bitcoin's chain identity.
pub struct BitcoinCashCanonicalizer<'a> {
    tx: &'a BitcoinTransaction,
}

impl<'a> BitcoinCashCanonicalizer<'a> {
    pub fn new(tx: &'a BitcoinTransaction) -> Self {
        Self { tx }
    }

    pub fn canonicalize(&self) -> Result<TxIR<'a, 1>> {
        // Reuse Bitcoin's canonicalization logic
        let mut tx_ir = self.tx.canonicalize()?;

        // Override chain to Bitcoin Cash
        tx_ir.chain = (&decoder_chains_common::chains::BITCOIN_CASH).into();

        Ok(tx_ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = BitcoinCashDecoder::chain();
        assert_eq!(chain.chain_id(), 145);
        assert_eq!(chain.chain_name(), "Bitcoin Cash");
        assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    }

    #[test]
    fn test_decode_minimal_legacy_transaction() {
        // Minimal valid legacy transaction: coinbase with 1 input, 1 output
        let mut tx_bytes = vec![];

        // Version: 1
        tx_bytes.extend_from_slice(&1u32.to_le_bytes());

        // Input count: 1
        tx_bytes.push(0x01);

        // Input 0 (coinbase):
        // - prev_hash (32 bytes, all zeros)
        tx_bytes.extend_from_slice(&[0u8; 32]);
        // - prev_index (0xFFFFFFFF for coinbase)
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        // - script_sig length: 1
        tx_bytes.push(0x01);
        // - script_sig data: [0x00]
        tx_bytes.push(0x00);
        // - sequence
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());

        // Output count: 1
        tx_bytes.push(0x01);

        // Output 0:
        // - value (50 BCH in satoshis)
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes());
        // - script_pubkey length: 0
        tx_bytes.push(0x00);

        // Locktime: 0
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());

        let decoded =
            BitcoinCashDecoder::decode(&tx_bytes).expect("Failed to decode minimal transaction");

        assert_eq!(decoded.version(), 1);
        assert_eq!(decoded.input_count(), 1);
        assert_eq!(decoded.output_count(), 1);
        assert_eq!(decoded.locktime, 0);
        assert!(decoded.is_coinbase());
        assert!(!decoded.is_segwit()); // Bitcoin Cash doesn't use SegWit
    }

    #[test]
    fn test_validate_empty() {
        let result = BitcoinCashDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_too_small() {
        let result = BitcoinCashDecoder::validate_format(&[0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn test_canonicalize_uses_bitcoin_cash_chain() {
        // Create minimal transaction
        let mut tx_bytes = vec![];
        tx_bytes.extend_from_slice(&1u32.to_le_bytes()); // version
        tx_bytes.push(0x01); // 1 input
        tx_bytes.extend_from_slice(&[0u8; 32]); // prev_hash
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // prev_index
        tx_bytes.push(0x01); // script_sig len
        tx_bytes.push(0x00); // script_sig data
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // sequence
        tx_bytes.push(0x01); // 1 output
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes()); // value
        tx_bytes.push(0x00); // script_pubkey len
        tx_bytes.extend_from_slice(&0u32.to_le_bytes()); // locktime

        let tx = BitcoinCashDecoder::decode(&tx_bytes).unwrap();
        let canonicalizer = BitcoinCashCanonicalizer::new(&tx);
        let tx_ir = canonicalizer.canonicalize().unwrap();

        // Verify it uses Bitcoin Cash chain
        assert_eq!(tx_ir.chain.id, 145);
        assert_eq!(tx_ir.chain.name, "Bitcoin Cash");
    }

    #[test]
    fn test_bitcoin_cash_no_segwit() {
        // Bitcoin Cash removed SegWit, so all transactions should be legacy format

        // Create a legacy transaction (no SegWit)
        let mut tx_bytes = vec![];
        tx_bytes.extend_from_slice(&1u32.to_le_bytes()); // version
        tx_bytes.push(0x01); // 1 input
        tx_bytes.extend_from_slice(&[0u8; 32]); // prev_hash
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // prev_index
        tx_bytes.push(0x01); // script_sig len
        tx_bytes.push(0x00); // script_sig data
        tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // sequence
        tx_bytes.push(0x01); // 1 output
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes()); // value
        tx_bytes.push(0x00); // script_pubkey len
        tx_bytes.extend_from_slice(&0u32.to_le_bytes()); // locktime

        let tx = BitcoinCashDecoder::decode(&tx_bytes).unwrap();

        // Bitcoin Cash transactions should not have SegWit
        assert!(!tx.is_segwit());
    }
}
