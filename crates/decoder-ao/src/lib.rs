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
//! use universal_decoder_core::traits::ChainDecoder;
//!
//! let decoder = AODecoder::new();
//! let tx_ir = decoder.decode(message_bytes)?;
//!
//! // Access AO-specific metadata
//! let action = tx_ir.metadata.extra["action"];
//! let target_process = tx_ir.metadata.extra["target"];
//! ```

pub mod parsing;
pub mod registry;
pub mod types;

use parsing::parse_ans104;
use registry::get_network_by_id;
use types::AOMessage;
use universal_decoder_core::{
    chain::{ChainFamily, ChainIdentity, ChainRef},
    error::{DecoderError, Result},
    ir::{
        AccountChange, Address, Amount, Authorization, Operation, Signature, StateDeltas, TxIR,
        TxMetadata,
    },
    traits::{Canonicalizer, ChainDecoder},
};

/// AO chain identity
#[derive(Debug, Clone)]
pub struct AOChain {
    network_id: u64,
}

impl AOChain {
    /// Create a new AO chain identity for mainnet
    pub fn mainnet() -> Self {
        Self {
            network_id: 1000000,
        }
    }

    /// Create a new AO chain identity for testnet
    pub fn testnet() -> Self {
        Self {
            network_id: 1000001,
        }
    }

    /// Create a new AO chain identity for a specific network
    pub fn new(network_id: u64) -> Self {
        Self { network_id }
    }
}

impl ChainIdentity for AOChain {
    fn chain_id(&self) -> u64 {
        self.network_id
    }

    fn chain_name(&self) -> &str {
        get_network_by_id(self.network_id)
            .map(|n| n.name.as_str())
            .unwrap_or("AO")
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Actor
    }

    fn network(&self) -> Option<&str> {
        get_network_by_id(self.network_id).map(|n| n.network_type.as_str())
    }
}

/// AO decoder implementing ChainDecoder trait
pub struct AODecoder {
    chain: AOChain,
}

impl AODecoder {
    /// Create a new AO decoder for mainnet
    pub fn new() -> Self {
        Self {
            chain: AOChain::mainnet(),
        }
    }

    /// Create a new AO decoder for a specific network
    pub fn with_network(network_id: u64) -> Self {
        Self {
            chain: AOChain::new(network_id),
        }
    }

    /// Decode an AOMessage into TxIR components
    fn decode_message(
        &self,
        msg: &AOMessage,
    ) -> Result<(TxMetadata, Authorization, Vec<Operation>, StateDeltas)> {
        // 1. Build metadata with AO-specific extras
        let message_id = msg.message_id();
        let message_id_hex = hex::encode(&message_id);

        let mut extra = serde_json::Map::new();
        extra.insert("message_type".to_string(), serde_json::json!("ao_message"));
        extra.insert("target".to_string(), serde_json::json!(msg.target_string()));
        extra.insert(
            "signature_type".to_string(),
            serde_json::json!(format!("{:?}", msg.signature_type)),
        );

        if let Some(epoch) = msg.epoch {
            extra.insert("epoch".to_string(), serde_json::json!(epoch));
        }
        if let Some(nonce) = msg.nonce {
            extra.insert("nonce".to_string(), serde_json::json!(nonce));
        }

        // Add tags to metadata
        let tags_json: Vec<serde_json::Value> = msg
            .tags
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "value": t.value,
                })
            })
            .collect();
        extra.insert("tags".to_string(), serde_json::json!(tags_json));

        let metadata = TxMetadata {
            tx_id: message_id_hex.clone(),
            timestamp: None, // AO messages don't have built-in timestamps
            version: None,
            fee: None, // AO doesn't have fees in messages
            extra: Some(serde_json::Value::Object(extra)),
        };

        // 2. Build authorization from signature
        let authorization = Authorization {
            signatures: vec![Signature {
                public_key: msg.owner.clone(),
                signature: msg.signature.clone(),
            }],
            required_signatures: 1,
        };

        // 3. Build operations from message action
        let operations = if let Some(action) = msg.action() {
            vec![Operation::ContractCall {
                contract: msg.target_string().unwrap_or_default(),
                function: action.to_string(),
                args: msg.data.clone(),
                gas_limit: None, // AO doesn't have gas limits
                value: None,     // No value transfer in AO messages
            }]
        } else {
            vec![]
        };

        // 4. Build state deltas (per-message state change)
        let state_deltas = StateDeltas {
            inputs: vec![],  // AO doesn't use UTXO model
            outputs: vec![], // AO doesn't use UTXO model
            account_changes: if let Some(target) = &msg.target {
                vec![AccountChange {
                    address: Address {
                        bytes: target.clone(),
                        human_readable: msg.target_string(),
                    },
                    nonce: msg.nonce,
                    balance_change: None, // AO doesn't track balances in messages
                    storage_changes: vec![], // State derived from message history
                }]
            } else {
                vec![]
            },
        };

        Ok((metadata, authorization, operations, state_deltas))
    }
}

impl Default for AODecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainDecoder for AODecoder {
    type TxSpecific = AOMessage;
    type ChainIdentity = AOChain;

    fn decode(&self, bytes: &[u8]) -> Result<TxIR> {
        // Parse ANS-104 DataItem
        let msg = parse_ans104(bytes)?;

        // Decode to TxIR components
        let (metadata, authorization, operations, state_deltas) = self.decode_message(&msg)?;

        // Create TxIR
        TxIR::new(
            &self.chain,
            metadata,
            authorization,
            operations,
            state_deltas,
        )
    }

    fn decode_specific(&self, bytes: &[u8]) -> Result<Self::TxSpecific> {
        parse_ans104(bytes)
    }

    fn chain(&self) -> &Self::ChainIdentity {
        &self.chain
    }
}

impl Canonicalizer for AOMessage {
    fn to_canonical_bytes(&self) -> Result<Vec<u8>> {
        // Canonical representation uses Borsh serialization
        // For ANS-104 messages, we use the message ID (hash of signature) as canonical ID
        Ok(self.message_id())
    }
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
        let chain = AOChain::mainnet();
        assert_eq!(chain.chain_id(), 1000000);
        assert_eq!(chain.chain_name(), "AO");
        assert_eq!(chain.chain_family(), ChainFamily::Actor);

        let testnet = AOChain::testnet();
        assert_eq!(testnet.chain_id(), 1000001);
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

        let decoder = AODecoder::new();
        let tx_ir = decoder.decode(&bytes).unwrap();

        assert_eq!(tx_ir.chain.family(), ChainFamily::Actor);
        assert!(tx_ir.metadata.tx_id.len() > 0);
        assert_eq!(tx_ir.authorization.signatures.len(), 1);
    }

    #[test]
    fn test_decode_message_with_action() {
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

        let decoder = AODecoder::new();
        let tx_ir = decoder.decode(&bytes).unwrap();

        assert_eq!(tx_ir.operations.len(), 1);

        if let Operation::ContractCall { function, .. } = &tx_ir.operations[0] {
            assert_eq!(function, "Transfer");
        } else {
            panic!("Expected ContractCall operation");
        }

        // Check metadata includes tags
        let extra = tx_ir.metadata.extra.unwrap();
        assert!(extra.get("tags").is_some());
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
