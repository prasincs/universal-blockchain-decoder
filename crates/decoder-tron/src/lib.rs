//! Tron transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct TronChain;

impl ChainIdentity for TronChain {
    fn chain_id(&self) -> u64 {
        195
    }

    fn chain_name(&self) -> &str {
        "Tron"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

#[derive(Debug, Clone)]
pub struct TronTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct TronDecoder;

impl ChainDecoder for TronDecoder {
    type TxSpecific = TronTransaction;
    type Chain = TronChain;

    fn chain() -> Self::Chain {
        TronChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(TronTransaction {
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Tron transaction cannot be empty",
            ));
        }
        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for TronTransaction {
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
            &TronChain,
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
        let chain = TronDecoder::chain();
        assert_eq!(chain.chain_id(), 195);
        assert_eq!(chain.chain_name(), "Tron");
    }
}
