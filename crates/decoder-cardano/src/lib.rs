//! Cardano transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct CardanoChain;

impl ChainIdentity for CardanoChain {
    fn chain_id(&self) -> u64 {
        1815
    }

    fn chain_name(&self) -> &str {
        "Cardano"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Utxo
    }
}

#[derive(Debug, Clone)]
pub struct CardanoTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct CardanoDecoder;

impl ChainDecoder for CardanoDecoder {
    type TxSpecific = CardanoTransaction;
    type Chain = CardanoChain;

    fn chain() -> Self::Chain {
        CardanoChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(CardanoTransaction { raw_bytes: raw_bytes.to_vec() })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Cardano transaction cannot be empty"));
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chain_identity() {
        let chain = CardanoDecoder::chain();
        assert_eq!(chain.chain_id(), 1815);
        assert_eq!(chain.chain_name(), "Cardano");
    }
}

impl<'a> Canonicalizer<'a> for CardanoTransaction {
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
            &CardanoChain,
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
