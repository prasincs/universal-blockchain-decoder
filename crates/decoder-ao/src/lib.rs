//! Arweave AO (Actor Oriented) blockchain decoder
//!
//! Decodes ANS-104 DataItems used for AO messages into the universal TxIR format.
//!
//! # Architecture
//!
//! AO uses the Actor Model for transaction semantics:
//! - Each message is decoded as a separate TxIR (per-message decoding)
//! - Messages are linked via parent/child references in `metadata.extra`
//! - State is derived from message history (event sourcing)
//!
//! # Example
//!
//! ```rust,ignore
//! use decoder_ao::AODecoder;
//! use decoder_primitives::prelude::*;
//!
//! let decoder = AODecoder;
//! let tx = AODecoder::decode(message_bytes)?;
//!
//! // Access AO-specific metadata
//! let extra_metadata = &tx.metadata.extra;
//! ```

pub mod parsing;
pub mod registry;
pub mod types;

use parsing::parse_ans104;
use types::AOMessage;

use decoder_primitives::prelude::*;

/// AO chain identity (Mainnet)
#[derive(Debug, Clone, Copy)]
pub struct AOChain;

impl ChainIdentity for AOChain {
    fn chain_id(&self) -> u64 {
        // Custom ID for AO mainnet
        1000000
    }

    fn chain_name(&self) -> &str {
        "AO"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Actor
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet")
    }
}

/// Parsed AO message
#[derive(Debug, Clone)]
pub struct AOTransaction {
    /// Parsed message
    pub message: AOMessage,
    /// Raw bytes
    pub raw_bytes: Vec<u8>,
    /// Message ID (hash of signature)
    pub message_id: Vec<u8>,
}

impl AOTransaction {
    /// Get action from tags
    pub fn action(&self) -> Option<&str> {
        self.message.action()
    }

    /// Get target process ID
    pub fn target(&self) -> Option<String> {
        self.message.target_string()
    }

    /// Get tags
    pub fn tags(&self) -> &[types::Tag] {
        &self.message.tags
    }
}

/// AO transaction decoder
pub struct AODecoder;

impl ChainDecoder for AODecoder {
    type TxSpecific = AOTransaction;
    type Chain = AOChain;

    fn chain() -> Self::Chain {
        AOChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;

        // Parse ANS-104 DataItem
        let message = parse_ans104(raw_bytes)?;
        let message_id = message.message_id();

        Ok(AOTransaction {
            message,
            raw_bytes: raw_bytes.to_vec(),
            message_id,
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "AO message cannot be empty",
            ));
        }

        // Minimum size check: signature_type (2 bytes) + signature + owner
        if raw_bytes.len() < 100 {
            return Err(DecoderError::invalid_structure(
                "AO message too short (minimum 100 bytes)",
            ));
        }

        Ok(())
    }
}

impl<'a> Canonicalizer<'a> for AOTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let operations = build_operations(&self.message)?;
        let authorization = build_authorization(&self.message)?;
        let state_deltas = build_state_deltas(&self.message)?;

        // Build extra metadata as JSON string
        let extra = format!(
            r#"{{"message_type":"ao_message","signature_type":"{:?}","target":"{}","epoch":{},"nonce":{},"tags_count":{}}}"#,
            self.message.signature_type,
            self.message.target_string().unwrap_or_default(),
            self.message
                .epoch
                .map(|e| e.to_string())
                .unwrap_or_else(|| "null".to_string()),
            self.message
                .nonce
                .map(|n| n.to_string())
                .unwrap_or_else(|| "null".to_string()),
            self.message.tags.len()
        );

        let metadata = TxMetadata {
            tx_hash: self.message_id.clone(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra,
        };

        Ok(TxIR::new(
            &AOChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Verify signature is present
        if self.message.signature.is_empty() {
            return Err(DecoderError::invalid_structure("No signature found"));
        }

        // Verify owner is present
        if self.message.owner.is_empty() {
            return Err(DecoderError::invalid_structure(
                "No owner (public key) found",
            ));
        }

        Ok(())
    }
}

/// Build operations from AO message
fn build_operations(msg: &AOMessage) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();

    if let Some(action) = msg.action() {
        let contract_address = if let Some(target) = &msg.target {
            Address {
                bytes: target.clone(),
                human_readable: msg.target_string(),
            }
        } else {
            Address {
                bytes: vec![],
                human_readable: None,
            }
        };

        operations.push(Operation::ContractCall(ContractCall {
            contract: contract_address,
            method: action.as_bytes().to_vec(),
            data: msg.data.clone(),
            value: None, // AO doesn't have value transfers in messages
            resource_limits: ResourceLimits {
                max_units: 0, // AO doesn't have gas limits
                unit_price: 0,
                resource_type: ResourceType::Custom(0), // AO uses custom resource model
            },
        }));
    }

    Ok(operations)
}

/// Build authorization from AO message signature
fn build_authorization(msg: &AOMessage) -> Result<AuthorizationPackage> {
    let signatures = vec![Signature {
        data: msg.signature.clone(),
        key_index: 0,
        metadata: Some(format!("AO {:?} signature", msg.signature_type)),
    }];

    let public_keys = vec![PublicKey {
        data: msg.owner.clone(),
        key_type: match msg.signature_type {
            types::SignatureType::Arweave => KeyType::Custom(1), // RSA 4096-bit
            types::SignatureType::Ethereum => KeyType::Secp256k1,
            types::SignatureType::Solana => KeyType::Ed25519,
            types::SignatureType::Unknown(n) => KeyType::Custom(n as u32),
        },
    }];

    Ok(AuthorizationPackage {
        signatures,
        public_keys,
        signature_scheme: match msg.signature_type {
            types::SignatureType::Arweave => SignatureScheme::Custom(1), // RSA-PSS
            types::SignatureType::Ethereum => SignatureScheme::Ecdsa,
            types::SignatureType::Solana => SignatureScheme::EdDsa,
            types::SignatureType::Unknown(n) => SignatureScheme::Custom(n as u32),
        },
    })
}

/// Build state deltas from AO message
fn build_state_deltas(msg: &AOMessage) -> Result<StateDeltas> {
    let mut account_changes = Vec::new();

    // Add target process state change
    if let Some(target) = &msg.target {
        account_changes.push(AccountChange {
            address: Address {
                bytes: target.clone(),
                human_readable: msg.target_string(),
            },
            nonce: msg.nonce,
            balance_change: 0,       // AO doesn't track balances in messages
            storage_changes: vec![], // State derived from message history
        });
    }

    Ok(StateDeltas {
        inputs: vec![],  // AO doesn't use UTXO model
        outputs: vec![], // AO doesn't use UTXO model
        account_changes,
    })
}

/// Helper function to get message ID from bytes without full parsing
pub fn get_message_id(bytes: &[u8]) -> Result<Vec<u8>> {
    let msg = parse_ans104(bytes)?;
    Ok(msg.message_id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ao_chain_identity() {
        let chain = AODecoder::chain();
        assert_eq!(chain.chain_id(), 1000000);
        assert_eq!(chain.chain_name(), "AO");
        assert_eq!(chain.chain_family(), ChainFamily::Actor);
    }

    #[test]
    fn test_decode_minimal_message() {
        // Construct minimal ANS-104 message
        let mut bytes = Vec::new();

        // Signature type (Solana = 4)
        bytes.extend_from_slice(&4u16.to_be_bytes());

        // Signature (64 bytes)
        bytes.extend_from_slice(&[0xAB; 64]);

        // Owner (32 bytes)
        bytes.extend_from_slice(&[0xCD; 32]);

        // No target
        bytes.push(0);

        // No anchor
        bytes.push(0);

        // No tags
        bytes.extend_from_slice(&0u64.to_be_bytes());
        bytes.extend_from_slice(&0u64.to_be_bytes());

        // Data
        bytes.extend_from_slice(b"Hello, AO!");

        let tx = AODecoder::decode(&bytes).unwrap();

        assert!(tx.message_id.len() == 32); // SHA-256
        assert_eq!(tx.message.signature_type, types::SignatureType::Solana);
    }

    #[test]
    fn test_canonicalize_message() {
        let mut bytes = Vec::new();

        // Signature type (Ethereum = 3)
        bytes.extend_from_slice(&3u16.to_be_bytes());

        // Signature (65 bytes)
        bytes.extend_from_slice(&[0x11; 65]);

        // Owner (65 bytes)
        bytes.extend_from_slice(&[0x22; 65]);

        // Target present
        bytes.push(1);
        bytes.extend_from_slice(&[0x33; 32]);

        // No anchor
        bytes.push(0);

        // 1 tag: Action=Transfer
        bytes.extend_from_slice(&1u64.to_be_bytes());

        let mut tag_bytes = Vec::new();
        tag_bytes.push(6); // "Action"
        tag_bytes.extend_from_slice(b"Action");
        tag_bytes.push(8); // "Transfer"
        tag_bytes.extend_from_slice(b"Transfer");

        bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
        bytes.extend_from_slice(&tag_bytes);

        // Data
        bytes.extend_from_slice(b"transfer_payload");

        let tx = AODecoder::decode(&bytes).unwrap();
        let tx_ir = tx.canonicalize().unwrap();

        assert_eq!(tx_ir.chain.family(), ChainFamily::Actor);
        assert!(!tx_ir.operations.is_empty());
        assert!(tx_ir.metadata.extra.contains("ao_message"));
    }

    #[test]
    fn test_message_id_deterministic() {
        let bytes = vec![
            0, 4, // Signature type: Solana
        ];
        let mut full_bytes = bytes;
        full_bytes.extend_from_slice(&[0xAA; 64]); // Signature
        full_bytes.extend_from_slice(&[0xBB; 32]); // Owner
        full_bytes.push(0); // No target
        full_bytes.push(0); // No anchor
        full_bytes.extend_from_slice(&0u64.to_be_bytes()); // No tags
        full_bytes.extend_from_slice(&0u64.to_be_bytes());
        full_bytes.extend_from_slice(b"data");

        let id1 = get_message_id(&full_bytes).unwrap();
        let id2 = get_message_id(&full_bytes).unwrap();

        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 32); // SHA-256 hash
    }
}
