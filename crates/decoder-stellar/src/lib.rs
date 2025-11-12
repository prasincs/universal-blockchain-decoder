//! Stellar transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct StellarChain;

impl ChainIdentity for StellarChain {
    fn chain_id(&self) -> u64 {
        144
    }

    fn chain_name(&self) -> &str {
        "Stellar"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

#[derive(Debug, Clone)]
pub struct StellarTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct StellarDecoder;

impl ChainDecoder for StellarDecoder {
    type TxSpecific = StellarTransaction;
    type Chain = StellarChain;

    fn chain() -> Self::Chain {
        StellarChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(StellarTransaction { raw_bytes: raw_bytes.to_vec() })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Stellar transaction cannot be empty"));
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chain_identity() {
        let chain = StellarDecoder::chain();
        assert_eq!(chain.chain_id(), 144);
        assert_eq!(chain.chain_name(), "Stellar");
    }
}

impl<'a> Canonicalizer<'a> for StellarTransaction {
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
            &StellarChain,
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
