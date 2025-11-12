//! Litecoin transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct LitecoinChain;

impl ChainIdentity for LitecoinChain {
    fn chain_id(&self) -> u64 {
        2
    }

    fn chain_name(&self) -> &str {
        "Litecoin"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

#[derive(Debug, Clone)]
pub struct LitecoinTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct LitecoinDecoder;

impl ChainDecoder for LitecoinDecoder {
    type TxSpecific = LitecoinTransaction;
    type Chain = LitecoinChain;

    fn chain() -> Self::Chain {
        LitecoinChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(LitecoinTransaction { raw_bytes: raw_bytes.to_vec() })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Litecoin transaction cannot be empty"));
        }
        Ok(())
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
    }
}

impl<'a> Canonicalizer<'a> for LitecoinTransaction {
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
            &LitecoinChain,
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
