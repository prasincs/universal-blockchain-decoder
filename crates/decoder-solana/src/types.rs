//! Solana-specific transaction types

use universal_decoder_core::prelude::*;

/// Solana-specific transaction representation
///
/// Note: This is a stub implementation. Full Solana support coming soon.
#[derive(Debug, Clone)]
pub struct SolanaTransaction {
    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl SolanaTransaction {
    /// Create from raw bytes
    pub fn from_raw_bytes(raw_bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            raw_bytes: raw_bytes.to_vec(),
        })
    }
}

impl<'a> Canonicalizer<'a> for SolanaTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Stub implementation
        let metadata = TxMetadata {
            tx_hash: vec![],
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: serde_json::json!({}),
        };

        let authorization = AuthorizationPackage {
            signatures: vec![],
            public_keys: vec![],
            signature_scheme: SignatureScheme::EdDsa,
        };

        let state_deltas = StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes: vec![],
        };

        Ok(TxIR::new(
            ChainId::Solana,
            metadata,
            authorization,
            vec![],
            state_deltas,
        ))
    }
}

impl TxHashable for SolanaTransaction {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.raw_bytes.clone()
    }
}
