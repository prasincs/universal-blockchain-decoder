//! Cosmos transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct CosmosChain;

impl ChainIdentity for CosmosChain {
    fn chain_id(&self) -> u64 {
        118
    }

    fn chain_name(&self) -> &str {
        "Cosmos"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

#[derive(Debug, Clone)]
pub struct CosmosTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct CosmosDecoder;

impl ChainDecoder for CosmosDecoder {
    type TxSpecific = CosmosTransaction;
    type Chain = CosmosChain;

    fn chain() -> Self::Chain {
        CosmosChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(CosmosTransaction {
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Cosmos transaction cannot be empty",
            ));
        }
        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for CosmosTransaction {
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
            &CosmosChain,
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
        let chain = CosmosDecoder::chain();
        assert_eq!(chain.chain_id(), 118);
        assert_eq!(chain.chain_name(), "Cosmos");
    }
}
