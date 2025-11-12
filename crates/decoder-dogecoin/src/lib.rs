//! Dogecoin transaction decoder
//!
//! Dogecoin is a Bitcoin fork that uses identical transaction format (no SegWit).
//! This decoder reuses the Bitcoin decoder with Dogecoin-specific chain ID.
//!
//! ## Phase 1 (Current): Scaffolding
//! ## Phase 2 (Future): Reuse Bitcoin decoder (no SegWit)

use decoder_primitives::prelude::*;

/// Dogecoin chain identity
#[derive(Debug, Clone, Copy)]
pub struct DogecoinChain;

impl ChainIdentity for DogecoinChain {
    fn chain_id(&self) -> u64 {
        3 // Dogecoin chain ID
    }

    fn chain_name(&self) -> &str {
        "Dogecoin"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

/// Dogecoin transaction (will reuse BitcoinTransaction in Phase 2)
#[derive(Debug, Clone)]
pub struct DogecoinTransaction {
    pub raw_bytes: Vec<u8>,
}

/// Dogecoin decoder
pub struct DogecoinDecoder;

impl ChainDecoder for DogecoinDecoder {
    type TxSpecific = DogecoinTransaction;
    type Chain = DogecoinChain;

    fn chain() -> Self::Chain {
        DogecoinChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        // Phase 2: Will use BitcoinDecoder::decode_legacy(raw_bytes)
        Ok(DogecoinTransaction {
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() || raw_bytes.len() < 10 {
            return Err(DecoderError::invalid_structure(
                "Invalid Dogecoin transaction",
            ));
        }
        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for DogecoinTransaction {
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
            &DogecoinChain,
            metadata,
            authorization,
            vec![], // operations
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = DogecoinDecoder::chain();
        assert_eq!(chain.chain_id(), 3);
        assert_eq!(chain.chain_name(), "Dogecoin");
        assert_eq!(chain.chain_family(), ChainFamily::Utxo);
    }
}
