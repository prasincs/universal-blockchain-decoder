//! Bitcoin-specific transaction types

use universal_decoder_core::prelude::*;
use crate::parsing::{TxInput, TxOutput, Witness};
use crate::BitcoinChain;

/// Bitcoin-specific transaction representation
///
/// This struct represents a fully parsed Bitcoin transaction with all fields
/// decoded from the raw transaction bytes.
#[derive(Debug, Clone)]
pub struct BitcoinTransaction {
    /// Transaction version
    pub version: u32,
    /// Transaction inputs
    pub inputs: Vec<TxInput>,
    /// Transaction outputs
    pub outputs: Vec<TxOutput>,
    /// Witness data (if SegWit)
    pub witnesses: Vec<Witness>,
    /// Lock time
    pub locktime: u32,
    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl BitcoinTransaction {
    /// Get transaction version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get the number of inputs
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Get the number of outputs
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Check if transaction is coinbase
    ///
    /// A coinbase transaction has exactly 1 input with:
    /// - prev_hash: all zeros
    /// - prev_index: 0xFFFFFFFF
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1
            && self.inputs[0].prev_hash == [0u8; 32]
            && self.inputs[0].prev_index == 0xFFFFFFFF
    }

    /// Check if transaction uses SegWit
    pub fn is_segwit(&self) -> bool {
        self.witnesses.iter().any(|w| !w.is_empty())
    }

    /// Calculate transaction ID (TXID)
    ///
    /// For SegWit transactions, this is the hash of the non-witness serialization.
    /// For legacy transactions, this is the hash of the entire transaction.
    ///
    /// TODO: Implement proper non-witness serialization for SegWit transactions.
    /// For now, we use the raw bytes (which is correct for legacy, but includes
    /// witness data for SegWit transactions).
    pub fn txid(&self) -> Vec<u8> {
        use sha2::{Sha256, Digest};

        // TODO: For SegWit, serialize without witness data
        let bytes_to_hash = if self.is_segwit() {
            // For now, use raw bytes (this is INCORRECT for SegWit)
            // Should serialize without marker, flag, and witness data
            &self.raw_bytes
        } else {
            &self.raw_bytes
        };

        // Double SHA-256
        let hash1 = Sha256::digest(bytes_to_hash);
        let hash2 = Sha256::digest(hash1);

        hash2.to_vec()
    }

    /// Calculate witness transaction ID (WTXID)
    ///
    /// This includes witness data (same as TXID for non-SegWit).
    pub fn wtxid(&self) -> Vec<u8> {
        use sha2::{Sha256, Digest};

        // WTXID includes all data including witness
        let hash1 = Sha256::digest(&self.raw_bytes);
        let hash2 = Sha256::digest(hash1);

        hash2.to_vec()
    }

    /// Calculate total output value
    ///
    /// Returns error if overflow occurs.
    pub fn total_output_value(&self) -> Result<u64> {
        self.outputs
            .iter()
            .try_fold(0u64, |acc, output| {
                acc.checked_add(output.value)
                    .ok_or_else(|| DecoderError::invalid_structure("Output value overflow"))
            })
    }

    /// Calculate fee (requires input values from UTXO set)
    ///
    /// Returns None if:
    /// - input_values length doesn't match inputs length
    /// - overflow occurs
    pub fn calculate_fee(&self, input_values: &[u64]) -> Option<u64> {
        if input_values.len() != self.inputs.len() {
            return None;
        }

        let total_input: u64 = input_values.iter().sum();
        let total_output: u64 = self.outputs.iter().map(|o| o.value).sum();

        total_input.checked_sub(total_output)
    }
}

impl<'a> Canonicalizer<'a> for BitcoinTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Build metadata
        let extra = format!(
            r#"{{"version":{},"lock_time":{},"is_coinbase":{},"is_segwit":{}}}"#,
            self.version, self.locktime, self.is_coinbase(), self.is_segwit()
        );

        let metadata = TxMetadata {
            tx_hash: self.txid(),
            block_height: None, // Not available from transaction alone
            timestamp: Some(self.locktime as u64),
            size: self.raw_bytes.len(),
            extra,
        };

        // Build authorization package (extract signatures from inputs and witnesses)
        let mut signatures = Vec::new();

        // Extract signatures from scriptSig (legacy inputs)
        for (idx, input) in self.inputs.iter().enumerate() {
            if !input.script_sig.is_empty() {
                signatures.push(Signature {
                    data: input.script_sig.clone(),
                    key_index: idx,
                    metadata: Some(format!(r#"{{"input_index":{}}}"#, idx)),
                });
            }
        }

        // Extract signatures from witness data (SegWit)
        for (idx, witness) in self.witnesses.iter().enumerate() {
            for (item_idx, item) in witness.items.iter().enumerate() {
                signatures.push(Signature {
                    data: item.clone(),
                    key_index: idx,
                    metadata: Some(format!(
                        r#"{{"input_index":{},"witness_index":{}}}"#,
                        idx, item_idx
                    )),
                });
            }
        }

        let authorization = AuthorizationPackage {
            signatures,
            public_keys: vec![], // TODO: Extract from scripts
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Build operations (transfers from outputs)
        let operations = self
            .outputs
            .iter()
            .map(|output| {
                Operation::Transfer(Transfer {
                    from: Address {
                        bytes: vec![],
                        human_readable: None,
                    },
                    to: Address {
                        bytes: output.script_pubkey.clone(),
                        human_readable: None, // TODO: Decode address from script
                    },
                    amount: Amount {
                        value: output.value as u128,
                        decimals: 8, // Bitcoin uses 8 decimal places (satoshis)
                    },
                    asset: AssetId::Native,
                })
            })
            .collect();

        // Build state deltas
        let inputs = self
            .inputs
            .iter()
            .map(|input| InputReference {
                prev_tx: input.prev_hash.to_vec(),
                output_index: input.prev_index,
                value: Amount {
                    value: 0, // Requires UTXO set
                    decimals: 8,
                },
                script: input.script_sig.clone(),
            })
            .collect();

        let outputs = self
            .outputs
            .iter()
            .enumerate()
            .map(|(idx, output)| OutputValue {
                index: idx as u32,
                address: Address {
                    bytes: output.script_pubkey.clone(),
                    human_readable: None,
                },
                value: Amount {
                    value: output.value as u128,
                    decimals: 8,
                },
                script: output.script_pubkey.clone(),
            })
            .collect();

        let state_deltas = StateDeltas {
            inputs,
            outputs,
            account_changes: vec![], // Bitcoin uses UTXO model, not account model
        };

        Ok(TxIR::new(
            &BitcoinChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Check version
        if self.version < 1 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid Bitcoin transaction version: {}",
                self.version
            )));
        }

        // Check inputs
        if self.inputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction must have at least one input",
            ));
        }

        // Check outputs
        if self.outputs.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Bitcoin transaction must have at least one output",
            ));
        }

        // Check for overflow in output values
        self.total_output_value()?;

        // Check witness data consistency
        if self.is_segwit() && self.witnesses.len() != self.inputs.len() {
            return Err(DecoderError::invalid_structure(format!(
                "Witness count ({}) must match input count ({}) for SegWit transactions",
                self.witnesses.len(),
                self.inputs.len()
            )));
        }

        Ok(())
    }
}

impl TxHashable for BitcoinTransaction {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        // Use raw bytes as canonical representation
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

    #[test]
    fn test_is_coinbase() {
        let coinbase_tx = BitcoinTransaction {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0u8; 32],
                prev_index: 0xFFFFFFFF,
                script_sig: vec![],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![],
            witnesses: vec![],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(coinbase_tx.is_coinbase());

        let regular_tx = BitcoinTransaction {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0xAAu8; 32],
                prev_index: 0,
                script_sig: vec![],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![],
            witnesses: vec![],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(!regular_tx.is_coinbase());
    }

    #[test]
    fn test_is_segwit() {
        let legacy_tx = BitcoinTransaction {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            witnesses: vec![Witness::empty()],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(!legacy_tx.is_segwit());

        let segwit_tx = BitcoinTransaction {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            witnesses: vec![Witness {
                items: vec![vec![0xAA, 0xBB]],
            }],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(segwit_tx.is_segwit());
    }

    #[test]
    fn test_total_output_value() {
        let tx = BitcoinTransaction {
            version: 1,
            inputs: vec![],
            outputs: vec![
                TxOutput {
                    value: 1_000_000,
                    script_pubkey: vec![],
                },
                TxOutput {
                    value: 2_000_000,
                    script_pubkey: vec![],
                },
                TxOutput {
                    value: 3_000_000,
                    script_pubkey: vec![],
                },
            ],
            witnesses: vec![],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert_eq!(tx.total_output_value().unwrap(), 6_000_000);
    }

    #[test]
    fn test_total_output_value_overflow() {
        let tx = BitcoinTransaction {
            version: 1,
            inputs: vec![],
            outputs: vec![
                TxOutput {
                    value: u64::MAX,
                    script_pubkey: vec![],
                },
                TxOutput {
                    value: 1,
                    script_pubkey: vec![],
                },
            ],
            witnesses: vec![],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(tx.total_output_value().is_err());
    }

    #[test]
    fn test_calculate_fee() {
        let tx = BitcoinTransaction {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0u8; 32],
                prev_index: 0,
                script_sig: vec![],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![TxOutput {
                value: 4_000_000,
                script_pubkey: vec![],
            }],
            witnesses: vec![],
            locktime: 0,
            raw_bytes: vec![],
        };

        // Input: 5 BTC, Output: 4 BTC, Fee: 1 BTC
        let input_values = vec![5_000_000];
        assert_eq!(tx.calculate_fee(&input_values), Some(1_000_000));

        // Wrong number of input values
        assert_eq!(tx.calculate_fee(&[]), None);
    }

    #[test]
    fn test_validate_success() {
        let tx = BitcoinTransaction {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0u8; 32],
                prev_index: 0,
                script_sig: vec![],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![TxOutput {
                value: 1_000_000,
                script_pubkey: vec![],
            }],
            witnesses: vec![Witness::empty()],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_validate_no_inputs() {
        let tx = BitcoinTransaction {
            version: 1,
            inputs: vec![],
            outputs: vec![TxOutput {
                value: 1_000_000,
                script_pubkey: vec![],
            }],
            witnesses: vec![],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(tx.validate().is_err());
    }

    #[test]
    fn test_validate_no_outputs() {
        let tx = BitcoinTransaction {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0u8; 32],
                prev_index: 0,
                script_sig: vec![],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![],
            witnesses: vec![],
            locktime: 0,
            raw_bytes: vec![],
        };

        assert!(tx.validate().is_err());
    }
}
