//! Aptos transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct AptosChain;

impl ChainIdentity for AptosChain {
    fn chain_id(&self) -> u64 {
        1
    }

    fn chain_name(&self) -> &str {
        "Aptos"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

#[derive(Debug, Clone)]
pub struct AptosTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct AptosDecoder;

impl ChainDecoder for AptosDecoder {
    type TxSpecific = AptosTransaction;
    type Chain = AptosChain;

    fn chain() -> Self::Chain {
        AptosChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(AptosTransaction { raw_bytes: raw_bytes.to_vec() })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Aptos transaction cannot be empty"));
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chain_identity() {
        let chain = AptosDecoder::chain();
        assert_eq!(chain.chain_id(), 1);
        assert_eq!(chain.chain_name(), "Aptos");
    }
}

impl<'a> Canonicalizer<'a> for AptosTransaction {
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
            &AptosChain,
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
