//! Litecoin transaction decoder
//!
//! Litecoin is a Bitcoin fork that uses identical transaction format with SegWit support.
//! This decoder reuses the Bitcoin decoder with Litecoin-specific chain ID.
//!
//! ## Differences from Bitcoin
//!
//! - Different address prefixes (L for P2PKH, M for P2SH)
//! - Scrypt PoW algorithm (doesn't affect transaction format)
//! - 2.5 minute block time (doesn't affect transaction format)
//! - Transaction format is identical to Bitcoin
//!
//! ## Implementation Strategy
//!
//! Litecoin transactions use the **exact same format** as Bitcoin transactions,
//! so we reuse `BitcoinTransaction` and only override the chain identity.

use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use decoder_primitives::prelude::*;

/// Litecoin chain identity (re-export from common library)
pub use decoder_chains_common::chains::LITECOIN as LitecoinChain;

/// Litecoin decoder - wrapper around Bitcoin decoder
///
/// Since Litecoin uses identical transaction format to Bitcoin (including SegWit),
/// we simply delegate to the Bitcoin decoder and override the chain identity.
pub struct LitecoinDecoder;

impl ChainDecoder for LitecoinDecoder {
    type TxSpecific = BitcoinTransaction;
    type Chain = decoder_chains_common::chains::ChainInfo;

    fn chain() -> Self::Chain {
        decoder_chains_common::chains::LITECOIN
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        // Litecoin uses identical transaction format to Bitcoin (including SegWit)
        BitcoinDecoder::decode(raw_bytes)
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Use Bitcoin's validation logic
        BitcoinDecoder::validate_format(raw_bytes)
    }
}

/// Litecoin-specific canonicalizer wrapper
///
/// This creates a Litecoin-specific canonicalizer that uses the Litecoin chain identity
/// instead of Bitcoin's chain identity.
pub struct LitecoinCanonicalizer<'a> {
    tx: &'a BitcoinTransaction,
}

impl<'a> LitecoinCanonicalizer<'a> {
    pub fn new(tx: &'a BitcoinTransaction) -> Self {
        Self { tx }
    }

    pub fn canonicalize(&self) -> Result<TxIR<'a, 1>> {
        // Reuse Bitcoin's canonicalization logic
        let mut tx_ir = self.tx.canonicalize()?;

        // Override chain to Litecoin
        tx_ir.chain = (&decoder_chains_common::chains::LITECOIN).into();

        Ok(tx_ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = LitecoinDecoder::chain();
        assert_eq!(chain.chain_id(), 2);
        assert_eq!(chain.chain_name(), "Litecoin");
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
        // - value (50 LTC in satoshis)
        tx_bytes.extend_from_slice(&5_000_000_000u64.to_le_bytes());
        // - script_pubkey length: 0
        tx_bytes.push(0x00);

        // Locktime: 0
        tx_bytes.extend_from_slice(&0u32.to_le_bytes());

        let decoded =
            LitecoinDecoder::decode(&tx_bytes).expect("Failed to decode minimal transaction");

        assert_eq!(decoded.version(), 1);
        assert_eq!(decoded.input_count(), 1);
        assert_eq!(decoded.output_count(), 1);
        assert_eq!(decoded.locktime, 0);
        assert!(decoded.is_coinbase());
        assert!(!decoded.is_segwit());
    }

    #[test]
    fn test_validate_empty() {
        let result = LitecoinDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_too_small() {
        let result = LitecoinDecoder::validate_format(&[0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn test_canonicalize_uses_litecoin_chain() {
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

        let tx = LitecoinDecoder::decode(&tx_bytes).unwrap();
        let canonicalizer = LitecoinCanonicalizer::new(&tx);
        let tx_ir = canonicalizer.canonicalize().unwrap();

        // Verify it uses Litecoin chain
        assert_eq!(tx_ir.chain.id, 2);
        assert_eq!(tx_ir.chain.name, "Litecoin");
    }
}
