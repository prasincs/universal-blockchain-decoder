//! Verified Bitcoin transaction types.
//!
//! This module provides type-safe Bitcoin transaction types that enforce
//! actual parsing through the type system by requiring reconstruction
//! from parsed fields.

use decoder_encodings::varint::encode_varint;
use universal_decoder_core::prelude::*;

use crate::parsing::{TxInput, TxOutput, Witness};

/// Bitcoin transaction parsed fields (no raw_bytes).
///
/// This struct contains ONLY the semantic fields parsed from a transaction.
/// It does NOT contain raw bytes, ensuring that `reconstruct_bytes()` must
/// actually serialize from the parsed fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitcoinParsedFields {
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
    // NOTE: No raw_bytes field! This is intentional.
}

impl BitcoinParsedFields {
    /// Check if transaction uses SegWit
    pub fn is_segwit(&self) -> bool {
        self.witnesses.iter().any(|w| !w.is_empty())
    }

    /// Serialize a single input to bytes
    fn serialize_input(input: &TxInput, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&input.prev_hash);
        buf.extend_from_slice(&input.prev_index.to_le_bytes());
        encode_varint(buf, input.script_sig.len() as u64);
        buf.extend_from_slice(&input.script_sig);
        buf.extend_from_slice(&input.sequence.to_le_bytes());
    }

    /// Serialize a single output to bytes
    fn serialize_output(output: &TxOutput, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&output.value.to_le_bytes());
        encode_varint(buf, output.script_pubkey.len() as u64);
        buf.extend_from_slice(&output.script_pubkey);
    }

    /// Serialize a single witness to bytes
    fn serialize_witness(witness: &Witness, buf: &mut Vec<u8>) {
        encode_varint(buf, witness.items.len() as u64);
        for item in &witness.items {
            encode_varint(buf, item.len() as u64);
            buf.extend_from_slice(item);
        }
    }

    /// Reconstruct as legacy (non-SegWit) transaction
    fn reconstruct_legacy(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Version (4 bytes, little-endian)
        buf.extend_from_slice(&self.version.to_le_bytes());

        // Input count
        encode_varint(&mut buf, self.inputs.len() as u64);

        // Inputs
        for input in &self.inputs {
            Self::serialize_input(input, &mut buf);
        }

        // Output count
        encode_varint(&mut buf, self.outputs.len() as u64);

        // Outputs
        for output in &self.outputs {
            Self::serialize_output(output, &mut buf);
        }

        // Locktime (4 bytes, little-endian)
        buf.extend_from_slice(&self.locktime.to_le_bytes());

        buf
    }

    /// Reconstruct as SegWit transaction
    fn reconstruct_segwit(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Version (4 bytes, little-endian)
        buf.extend_from_slice(&self.version.to_le_bytes());

        // SegWit marker and flag
        buf.push(0x00); // Marker
        buf.push(0x01); // Flag

        // Input count
        encode_varint(&mut buf, self.inputs.len() as u64);

        // Inputs
        for input in &self.inputs {
            Self::serialize_input(input, &mut buf);
        }

        // Output count
        encode_varint(&mut buf, self.outputs.len() as u64);

        // Outputs
        for output in &self.outputs {
            Self::serialize_output(output, &mut buf);
        }

        // Witness data
        for witness in &self.witnesses {
            Self::serialize_witness(witness, &mut buf);
        }

        // Locktime (4 bytes, little-endian)
        buf.extend_from_slice(&self.locktime.to_le_bytes());

        buf
    }
}

impl ReconstructableTransaction for BitcoinParsedFields {
    /// Reconstruct the transaction bytes from parsed fields.
    ///
    /// This method MUST NOT rely on any stored raw bytes - it reconstructs
    /// the transaction purely from the semantic fields.
    fn reconstruct_bytes(&self) -> Result<Vec<u8>> {
        if self.is_segwit() {
            Ok(self.reconstruct_segwit())
        } else {
            Ok(self.reconstruct_legacy())
        }
    }
}

/// Verified Bitcoin decoder that enforces actual parsing.
pub struct VerifiedBitcoinDecoder;

impl VerifiedChainDecoder for VerifiedBitcoinDecoder {
    type ParsedFields = BitcoinParsedFields;

    fn decode_verified(raw_bytes: &[u8]) -> Result<VerifiedTransaction<Self::ParsedFields>> {
        use crate::BitcoinDecoder;

        // Parse using existing decoder
        let tx = BitcoinDecoder::decode(raw_bytes)?;

        // Convert to parsed fields (no raw_bytes)
        let parsed = BitcoinParsedFields {
            version: tx.version,
            inputs: tx.inputs,
            outputs: tx.outputs,
            witnesses: tx.witnesses,
            locktime: tx.locktime,
        };

        Ok(VerifiedTransaction::new(parsed, raw_bytes.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use universal_decoder_core::verified::testing::verify_field_affects_output;

    #[test]
    fn test_parsed_fields_no_raw_bytes() {
        let parsed = BitcoinParsedFields {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            witnesses: vec![],
            locktime: 0,
        };

        // This compiles because there's no raw_bytes field
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.locktime, 0);
    }

    #[test]
    fn test_reconstruct_minimal_legacy() {
        let parsed = BitcoinParsedFields {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0u8; 32],
                prev_index: 0xFFFFFFFF,
                script_sig: vec![],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![TxOutput {
                value: 5_000_000_000,
                script_pubkey: vec![],
            }],
            witnesses: vec![Witness::empty()],
            locktime: 0,
        };

        let bytes = parsed.reconstruct_bytes().unwrap();

        // Verify structure:
        // - version (4) + input_count (1) + input (41) + output_count (1) + output (9) + locktime (4)
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..4], &1u32.to_le_bytes()); // version
    }

    #[test]
    fn test_field_mutation_changes_output() {
        let parsed = BitcoinParsedFields {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0xAA; 32],
                prev_index: 0,
                script_sig: vec![1, 2, 3],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![TxOutput {
                value: 1_000_000,
                script_pubkey: vec![4, 5, 6],
            }],
            witnesses: vec![Witness::empty()],
            locktime: 500000,
        };

        let original_bytes = parsed.reconstruct_bytes().unwrap();

        // Mutate version
        let mut mutated = parsed.clone();
        mutated.version = 2;
        let mutated_bytes = mutated.reconstruct_bytes().unwrap();

        assert_ne!(
            original_bytes, mutated_bytes,
            "Changing version should change output bytes"
        );
    }

    #[test]
    fn test_verified_transaction_detects_mutation() {
        let parsed = BitcoinParsedFields {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0xBB; 32],
                prev_index: 0,
                script_sig: vec![0xDE, 0xAD],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![TxOutput {
                value: 500_000,
                script_pubkey: vec![0xBE, 0xEF],
            }],
            witnesses: vec![Witness::empty()],
            locktime: 0,
        };

        let original_bytes = parsed.reconstruct_bytes().unwrap();
        let tx = VerifiedTransaction::new(parsed, original_bytes);

        // Verify critical fields affect output
        verify_field_affects_output(&tx, |p| p.version = 999).unwrap();
        verify_field_affects_output(&tx, |p| p.locktime = 999999).unwrap();
        verify_field_affects_output(&tx, |p| p.inputs[0].script_sig = vec![0xFF, 0xFF, 0xFF])
            .unwrap();
        verify_field_affects_output(&tx, |p| p.outputs[0].value = 999).unwrap();
    }

    #[test]
    fn test_reconstruct_segwit() {
        let parsed = BitcoinParsedFields {
            version: 2,
            inputs: vec![TxInput {
                prev_hash: [0xCC; 32],
                prev_index: 0,
                script_sig: vec![],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![TxOutput {
                value: 1_000_000,
                script_pubkey: vec![0x00, 0x14, 0xAA, 0xBB], // P2WPKH-like
            }],
            witnesses: vec![Witness {
                items: vec![vec![0x30, 0x44], vec![0x02, 0x21]], // Mock signature and pubkey
            }],
            locktime: 0,
        };

        let bytes = parsed.reconstruct_bytes().unwrap();

        // Verify SegWit marker and flag
        assert_eq!(bytes[4], 0x00); // marker
        assert_eq!(bytes[5], 0x01); // flag
    }

    #[test]
    fn test_reconstruction_determinism() {
        let parsed = BitcoinParsedFields {
            version: 1,
            inputs: vec![TxInput {
                prev_hash: [0x11; 32],
                prev_index: 0,
                script_sig: vec![0x48],
                sequence: 0xFFFFFFFF,
            }],
            outputs: vec![TxOutput {
                value: 100_000,
                script_pubkey: vec![0x76, 0xa9],
            }],
            witnesses: vec![Witness::empty()],
            locktime: 12345,
        };

        let bytes1 = parsed.reconstruct_bytes().unwrap();
        let bytes2 = parsed.reconstruct_bytes().unwrap();
        let bytes3 = parsed.reconstruct_bytes().unwrap();

        assert_eq!(bytes1, bytes2);
        assert_eq!(bytes2, bytes3);
    }
}
