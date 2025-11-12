//! Arbitrum transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct ArbitrumChain;

impl ChainIdentity for ArbitrumChain {
    fn chain_id(&self) -> u64 {
        42161
    }

    fn chain_name(&self) -> &str {
        "Arbitrum"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

#[derive(Debug, Clone)]
pub struct ArbitrumTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct ArbitrumDecoder;

impl ChainDecoder for ArbitrumDecoder {
    type TxSpecific = ArbitrumTransaction;
    type Chain = ArbitrumChain;

    fn chain() -> Self::Chain {
        ArbitrumChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(ArbitrumTransaction { raw_bytes: raw_bytes.to_vec() })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Arbitrum transaction cannot be empty"));
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chain_identity() {
        let chain = ArbitrumDecoder::chain();
        assert_eq!(chain.chain_id(), 42161);
        assert_eq!(chain.chain_name(), "Arbitrum");
    }
}

impl<'a> Canonicalizer<'a> for ArbitrumTransaction {
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
            &ArbitrumChain,
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
