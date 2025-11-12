//! BNB Chain (Binance Smart Chain) transaction decoder
//!
//! This module provides a decoder for BNB Chain transactions, transforming them
//! from their native RLP format into the universal TxIR representation.
//!
//! ## Implementation Strategy
//!
//! BNB Chain is EVM-compatible and uses the same transaction format as Ethereum.
//! This decoder will **reuse the Ethereum decoder** with BNB-specific validation.
//!
//! ## Phase 1 (Current): Scaffolding
//! - Chain identity implementation
//! - Basic structure
//! - Stub decoder
//!
//! ## Phase 2 (Future): Pure Rust Implementation
//! - Reuse Ethereum RLP parser
//! - Add BNB-specific validation (chain ID 56)
//! - Support PoSA consensus-specific fields (if needed)
//!
//! ## Transaction Format
//!
//! - RLP-encoded (identical to Ethereum)
//! - EIP-2718 transaction types (legacy, EIP-2930, EIP-1559)
//! - Chain ID: 56 (mainnet), 97 (testnet)
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_bnb::*;
//! use universal_decoder_core::prelude::*;
//!
//! let tx_hex = "f86c...";
//! let tx_bytes = hex::decode(tx_hex)?;
//!
//! let decoded = BnbDecoder::decode(&tx_bytes)?;
//! let tx_ir = decoded.canonicalize()?;
//! ```

use decoder_primitives::prelude::*;

/// BNB Chain identity
#[derive(Debug, Clone, Copy)]
pub struct BnbChain;

impl ChainIdentity for BnbChain {
    fn chain_id(&self) -> u64 {
        56 // BNB Chain mainnet ID
    }

    fn chain_name(&self) -> &str {
        "BNB Chain"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// BNB Chain transaction (currently a stub, will reuse Ethereum in Phase 2)
#[derive(Debug, Clone)]
pub struct BnbTransaction {
    pub chain_id: u64,
    pub raw_bytes: Vec<u8>,
}

/// BNB Chain decoder implementing the ChainDecoder trait
///
/// **Phase 1**: Stub implementation
/// **Phase 2**: Will reuse Ethereum decoder with BNB-specific chain ID validation
pub struct BnbDecoder;

impl ChainDecoder for BnbDecoder {
    type TxSpecific = BnbTransaction;
    type Chain = BnbChain;

    fn chain() -> Self::Chain {
        BnbChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;

        // Phase 1: Stub implementation
        // Phase 2: Will use Ethereum RLP decoder
        Ok(BnbTransaction {
            chain_id: 56,
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "BNB Chain transaction cannot be empty",
            ));
        }

        if raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(format!(
                "BNB Chain transaction too small: {} bytes",
                raw_bytes.len()
            )));
        }

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = BnbDecoder::chain();
        assert_eq!(chain.chain_id(), 56);
        assert_eq!(chain.chain_name(), "BNB Chain");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_format() {
        assert!(BnbDecoder::validate_format(&[]).is_err());
        assert!(BnbDecoder::validate_format(&[0x01]).is_err());
        assert!(BnbDecoder::validate_format(&vec![0u8; 100]).is_ok());
    }

    #[test]
    fn test_decode_stub() {
        let dummy_tx = vec![0u8; 100];
        let result = BnbDecoder::decode(&dummy_tx);
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.chain_id, 56);
    }
}

impl<'a> Canonicalizer<'a> for BnbTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let metadata = TxMetadata {
            tx_hash: vec![],
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: String::new(),
        };

        let authorization = AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        };

        let state_deltas = StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        };

        Ok(TxIR::new(
            &BnbChain,
            metadata,
            authorization,
            vec![],  // operations
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        Ok(())
    }
}
