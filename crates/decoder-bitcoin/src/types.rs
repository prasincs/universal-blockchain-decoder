//! Bitcoin-specific transaction types

use bitcoin::hashes::Hash;
use bitcoin::Transaction as BitcoinTx;
use universal_decoder_core::prelude::*;

/// Bitcoin-specific transaction representation
#[derive(Debug, Clone)]
pub struct BitcoinTransaction {
    /// The underlying bitcoin transaction
    pub inner: BitcoinTx,
    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl BitcoinTransaction {
    /// Create from a bitcoin transaction and raw bytes
    pub fn from_bitcoin_tx(tx: BitcoinTx, raw_bytes: &[u8]) -> Self {
        Self {
            inner: tx,
            raw_bytes: raw_bytes.to_vec(),
        }
    }

    /// Get the transaction ID
    pub fn txid(&self) -> Vec<u8> {
        self.inner.txid().as_byte_array().to_vec()
    }

    /// Get transaction version
    pub fn version(&self) -> i32 {
        self.inner.version.0
    }

    /// Get the number of inputs
    pub fn input_count(&self) -> usize {
        self.inner.input.len()
    }

    /// Get the number of outputs
    pub fn output_count(&self) -> usize {
        self.inner.output.len()
    }

    /// Calculate total input value (requires UTXO set information)
    pub fn calculate_fee(&self, input_values: &[u64]) -> Option<u64> {
        if input_values.len() != self.inner.input.len() {
            return None;
        }

        let total_input: u64 = input_values.iter().sum();
        let total_output: u64 = self.inner.output.iter().map(|o| o.value.to_sat()).sum();

        total_input.checked_sub(total_output)
    }
}

impl<'a> Canonicalizer<'a> for BitcoinTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let tx = &self.inner;

        // Build metadata
        let metadata = TxMetadata {
            tx_hash: self.txid(),
            block_height: None, // Not available from transaction alone
            timestamp: Some(tx.lock_time.to_consensus_u32() as u64),
            size: self.raw_bytes.len(),
            extra: serde_json::json!({
                "version": tx.version.0,
                "lock_time": tx.lock_time.to_consensus_u32(),
                "is_coinbase": tx.is_coinbase(),
            }),
        };

        // Build authorization package
        // Bitcoin transactions have signatures embedded in inputs (scriptSig)
        let mut signatures = Vec::new();
        let mut public_keys = Vec::new();

        for (idx, input) in tx.input.iter().enumerate() {
            // Extract signatures from scriptSig
            // This is simplified - real implementation would parse the script
            if !input.script_sig.is_empty() {
                signatures.push(Signature {
                    data: input.script_sig.as_bytes().to_vec(),
                    key_index: idx,
                    metadata: Some(serde_json::json!({
                        "input_index": idx,
                    })),
                });
            }

            // For witness transactions, also check witness data
            if !input.witness.is_empty() {
                for witness_item in input.witness.iter() {
                    signatures.push(Signature {
                        data: witness_item.to_vec(),
                        key_index: idx,
                        metadata: Some(serde_json::json!({
                            "input_index": idx,
                            "witness": true,
                        })),
                    });
                }
            }
        }

        let authorization = AuthorizationPackage {
            signatures,
            public_keys,
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Build operations (Bitcoin transactions are primarily transfers)
        let mut operations = Vec::new();

        // For each output, create a transfer operation
        for output in tx.output.iter() {
            operations.push(Operation::Transfer(Transfer {
                from: Address {
                    bytes: vec![],      // Input addresses require UTXO set
                    human_readable: None,
                },
                to: Address {
                    bytes: output.script_pubkey.as_bytes().to_vec(),
                    human_readable: output.script_pubkey.to_string().into(),
                },
                amount: Amount {
                    value: output.value.to_sat() as u128,
                    decimals: 8, // Bitcoin uses 8 decimal places (satoshis)
                },
                asset: AssetId::Native,
            }));
        }

        // Build state deltas
        let inputs: Vec<InputReference> = tx
            .input
            .iter()
            .enumerate()
            .map(|(idx, input)| InputReference {
                prev_tx: input.previous_output.txid.as_byte_array().to_vec(),
                output_index: input.previous_output.vout,
                value: Amount {
                    value: 0, // Would need UTXO set to determine actual value
                    decimals: 8,
                },
                script: input.script_sig.as_bytes().to_vec(),
            })
            .collect();

        let outputs: Vec<OutputValue> = tx
            .output
            .iter()
            .enumerate()
            .map(|(idx, output)| OutputValue {
                index: idx as u32,
                address: Address {
                    bytes: output.script_pubkey.as_bytes().to_vec(),
                    human_readable: output.script_pubkey.to_string().into(),
                },
                value: Amount {
                    value: output.value.to_sat() as u128,
                    decimals: 8,
                },
                script: output.script_pubkey.as_bytes().to_vec(),
            })
            .collect();

        let state_deltas = StateDeltas {
            inputs,
            outputs,
            account_changes: vec![], // Bitcoin uses UTXO model, not account model
        };

        Ok(TxIR::new(
            ChainId::Bitcoin,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        let tx = &self.inner;

        // Check version is valid
        if tx.version.0 < 1 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid Bitcoin transaction version: {}",
                tx.version.0
            )));
        }

        // Check inputs
        if tx.input.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction must have at least one input",
            ));
        }

        // Check outputs
        if tx.output.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction must have at least one output",
            ));
        }

        // Check for overflow in output values
        let total_output: Result<u64> = tx
            .output
            .iter()
            .try_fold(0u64, |acc, output| {
                acc.checked_add(output.value.to_sat())
                    .ok_or_else(|| DecoderError::overflow("Output value overflow"))
            });

        total_output?;

        Ok(())
    }
}

impl TxHashable for BitcoinTransaction {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        self.raw_bytes.clone()
    }

    fn compute_hash(&self) -> Vec<u8> {
        // Bitcoin uses double SHA-256
        self.compute_hash_with::<DoubleSha256>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_transaction_version() {
        assert_eq!(BitcoinTransaction::VERSION, 1);
    }
}
