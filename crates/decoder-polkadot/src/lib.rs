//! Polkadot transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PolkadotChain;

impl ChainIdentity for PolkadotChain {
    fn chain_id(&self) -> u64 {
        0
    }

    fn chain_name(&self) -> &str {
        "Polkadot"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

#[derive(Debug, Clone)]
pub struct PolkadotTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct PolkadotDecoder;

impl ChainDecoder for PolkadotDecoder {
    type TxSpecific = PolkadotTransaction;
    type Chain = PolkadotChain;

    fn chain() -> Self::Chain {
        PolkadotChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(PolkadotTransaction { raw_bytes: raw_bytes.to_vec() })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Polkadot transaction cannot be empty"));
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chain_identity() {
        let chain = PolkadotDecoder::chain();
        assert_eq!(chain.chain_id(), 0);
        assert_eq!(chain.chain_name(), "Polkadot");
    }
}

impl<'a> Canonicalizer<'a> for PolkadotTransaction {
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
            &PolkadotChain,
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
