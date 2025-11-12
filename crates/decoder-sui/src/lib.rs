//! Sui transaction decoder

use decoder_primitives::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct SuiChain;

impl ChainIdentity for SuiChain {
    fn chain_id(&self) -> u64 {
        0
    }

    fn chain_name(&self) -> &str {
        "Sui"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Instruction
    }
}

#[derive(Debug, Clone)]
pub struct SuiTransaction {
    pub raw_bytes: Vec<u8>,
}

pub struct SuiDecoder;

impl ChainDecoder for SuiDecoder {
    type TxSpecific = SuiTransaction;
    type Chain = SuiChain;

    fn chain() -> Self::Chain {
        SuiChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;
        Ok(SuiTransaction {
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Sui transaction cannot be empty",
            ));
        }
        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for SuiTransaction {
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
            &SuiChain,
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
        let chain = SuiDecoder::chain();
        assert_eq!(chain.chain_id(), 0);
        assert_eq!(chain.chain_name(), "Sui");
    }
}
