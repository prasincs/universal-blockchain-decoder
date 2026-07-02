//! Algorand transaction decoder
//!
//! This decoder parses Algorand transactions from their canonical MessagePack encoding.
//! Algorand uses Ed25519 signatures and account-based model.
//!
//! Reference: <https://developer.algorand.org/docs/get-details/transactions/>

use decoder_primitives::prelude::*;
use decoder_primitives::{
    Address, Amount, AssetId, ContractCall, GenericOperation, KeyType, Operation, PublicKey,
    ResourceLimits, ResourceType, Signature, Transfer,
};
use serde::{Deserialize, Serialize};

/// Algorand blockchain identity
#[derive(Debug, Clone, Copy)]
pub struct AlgorandChain;

impl ChainIdentity for AlgorandChain {
    fn chain_id(&self) -> u64 {
        4160 // Algorand mainnet chain ID
    }

    fn chain_name(&self) -> &str {
        "Algorand"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }

    fn network(&self) -> Option<&str> {
        Some("mainnet-v1.0")
    }
}

/// Algorand transaction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlgorandTxType {
    #[serde(rename = "pay")]
    Payment,
    #[serde(rename = "keyreg")]
    KeyRegistration,
    #[serde(rename = "acfg")]
    AssetConfig,
    #[serde(rename = "axfer")]
    AssetTransfer,
    #[serde(rename = "afrz")]
    AssetFreeze,
    #[serde(rename = "appl")]
    ApplicationCall,
}

/// Algorand signed transaction wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    #[serde(rename = "sig", skip_serializing_if = "Option::is_none")]
    pub signature: Option<Vec<u8>>,

    #[serde(rename = "txn")]
    pub transaction: RawTransaction,

    #[serde(rename = "sgnr", skip_serializing_if = "Option::is_none")]
    pub auth_addr: Option<Vec<u8>>,
}

/// Algorand raw transaction structure
///
/// Field names follow the canonical MessagePack encoding specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    // Common fields (all transaction types)
    #[serde(rename = "type")]
    pub tx_type: AlgorandTxType,

    #[serde(rename = "snd")]
    pub sender: Vec<u8>,

    #[serde(rename = "fee")]
    pub fee: u64,

    #[serde(rename = "fv")]
    pub first_valid: u64,

    #[serde(rename = "lv")]
    pub last_valid: u64,

    #[serde(rename = "gen", skip_serializing_if = "Option::is_none")]
    pub genesis_id: Option<String>,

    #[serde(rename = "gh")]
    pub genesis_hash: Vec<u8>,

    #[serde(rename = "note", skip_serializing_if = "Option::is_none")]
    pub note: Option<Vec<u8>>,

    #[serde(rename = "grp", skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<u8>>,

    #[serde(rename = "lx", skip_serializing_if = "Option::is_none")]
    pub lease: Option<Vec<u8>>,

    #[serde(rename = "rekey", skip_serializing_if = "Option::is_none")]
    pub rekey_to: Option<Vec<u8>>,

    // Payment fields
    #[serde(rename = "rcv", skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Vec<u8>>,

    #[serde(rename = "amt", skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,

    #[serde(rename = "close", skip_serializing_if = "Option::is_none")]
    pub close_remainder_to: Option<Vec<u8>>,

    // Asset transfer fields
    #[serde(rename = "xaid", skip_serializing_if = "Option::is_none")]
    pub xfer_asset: Option<u64>,

    #[serde(rename = "aamt", skip_serializing_if = "Option::is_none")]
    pub asset_amount: Option<u64>,

    #[serde(rename = "asnd", skip_serializing_if = "Option::is_none")]
    pub asset_sender: Option<Vec<u8>>,

    #[serde(rename = "arcv", skip_serializing_if = "Option::is_none")]
    pub asset_receiver: Option<Vec<u8>>,

    #[serde(rename = "aclose", skip_serializing_if = "Option::is_none")]
    pub asset_close_to: Option<Vec<u8>>,

    // Application call fields
    #[serde(rename = "apid", skip_serializing_if = "Option::is_none")]
    pub application_id: Option<u64>,

    #[serde(rename = "apan", skip_serializing_if = "Option::is_none")]
    pub on_completion: Option<u64>,

    #[serde(rename = "apaa", skip_serializing_if = "Option::is_none")]
    pub app_arguments: Option<Vec<Vec<u8>>>,

    #[serde(rename = "apat", skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<Vec<u8>>>,

    #[serde(rename = "apfa", skip_serializing_if = "Option::is_none")]
    pub foreign_apps: Option<Vec<u64>>,

    #[serde(rename = "apas", skip_serializing_if = "Option::is_none")]
    pub foreign_assets: Option<Vec<u64>>,

    // Asset config fields
    #[serde(rename = "caid", skip_serializing_if = "Option::is_none")]
    pub config_asset: Option<u64>,

    // Key registration fields
    #[serde(rename = "votekey", skip_serializing_if = "Option::is_none")]
    pub vote_pk: Option<Vec<u8>>,

    #[serde(rename = "selkey", skip_serializing_if = "Option::is_none")]
    pub selection_pk: Option<Vec<u8>>,

    #[serde(rename = "votefst", skip_serializing_if = "Option::is_none")]
    pub vote_first: Option<u64>,

    #[serde(rename = "votelst", skip_serializing_if = "Option::is_none")]
    pub vote_last: Option<u64>,

    #[serde(rename = "votekd", skip_serializing_if = "Option::is_none")]
    pub vote_key_dilution: Option<u64>,
}

/// Parsed Algorand transaction
#[derive(Debug, Clone)]
pub struct AlgorandTransaction {
    pub raw_bytes: Vec<u8>,
    pub signed_tx: SignedTransaction,
}

impl AlgorandTransaction {
    /// Compute transaction ID (hash of the transaction)
    pub fn tx_id(&self) -> Vec<u8> {
        use sha2::{Digest, Sha512_256};

        // Algorand uses SHA-512/256 with "TX" prefix
        let mut hasher = Sha512_256::new();
        hasher.update(b"TX");

        // Hash the canonical msgpack encoding of the transaction (not the signed transaction)
        if let Ok(tx_bytes) = rmp_serde::to_vec(&self.signed_tx.transaction) {
            hasher.update(&tx_bytes);
        }

        hasher.finalize().to_vec()
    }

    /// Get sender address as base32 string (Algorand address format)
    pub fn sender_address(&self) -> String {
        encode_address(&self.signed_tx.transaction.sender)
    }

    /// Get receiver address (if payment transaction)
    pub fn receiver_address(&self) -> Option<String> {
        self.signed_tx
            .transaction
            .receiver
            .as_ref()
            .map(|r| encode_address(r))
    }
}

/// Encode Algorand address (32 bytes + 4 byte checksum, base32 encoded)
fn encode_address(public_key: &[u8]) -> String {
    use sha2::{Digest, Sha512_256};

    if public_key.len() != 32 {
        return format!(
            "INVALID_ADDRESS_{}",
            universal_decoder_core::hex::encode(public_key)
        );
    }

    // Compute checksum (last 4 bytes of SHA-512/256)
    let mut hasher = Sha512_256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();
    let checksum = &hash[28..32];

    // Combine public key + checksum
    let mut addr_bytes = public_key.to_vec();
    addr_bytes.extend_from_slice(checksum);

    // Base32 encode
    base32_encode(&addr_bytes)
}

/// Simple base32 encoding (RFC 4648)
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();

    let mut bits = 0u32;
    let mut bit_count = 0;

    for &byte in data {
        bits = (bits << 8) | (byte as u32);
        bit_count += 8;

        while bit_count >= 5 {
            bit_count -= 5;
            let index = ((bits >> bit_count) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
    }

    if bit_count > 0 {
        let index = ((bits << (5 - bit_count)) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }

    result
}

/// Algorand transaction decoder
pub struct AlgorandDecoder;

impl ChainDecoder for AlgorandDecoder {
    type TxSpecific = AlgorandTransaction;
    type Chain = AlgorandChain;

    fn chain() -> Self::Chain {
        AlgorandChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;

        // Parse MessagePack encoded transaction
        let signed_tx: SignedTransaction = rmp_serde::from_slice(raw_bytes).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to decode MessagePack: {}", e))
        })?;

        Ok(AlgorandTransaction {
            raw_bytes: raw_bytes.to_vec(),
            signed_tx,
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Algorand transaction cannot be empty",
            ));
        }

        // MessagePack can start with:
        // - Map types: 0x80-0x8f (fixmap), 0xde (map16), 0xdf (map32)
        // - Array types: 0x90-0x9f (fixarray), 0xdc (array16), 0xdd (array32)
        // rmp_serde may serialize structs as either maps or arrays depending on configuration
        let first_byte = raw_bytes[0];
        let is_valid = (0x80..=0x8f).contains(&first_byte)  // fixmap
            || (0x90..=0x9f).contains(&first_byte)           // fixarray
            || first_byte == 0xde                              // map16
            || first_byte == 0xdf                              // map32
            || first_byte == 0xdc                              // array16
            || first_byte == 0xdd; // array32

        if !is_valid {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid MessagePack format: got 0x{:02x}",
                first_byte
            )));
        }

        Ok(())
    }
}

impl ChainEncoder for AlgorandTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for AlgorandTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let tx = &self.signed_tx.transaction;

        // Metadata
        let metadata = TxMetadata {
            tx_hash: self.tx_id(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!(
                "first_valid={} last_valid={}",
                tx.first_valid, tx.last_valid
            ),
        };

        // Authorization (Ed25519 signatures)
        let mut signatures = Vec::new();
        let mut public_keys = Vec::new();

        if let Some(ref sig) = self.signed_tx.signature {
            signatures.push(Signature {
                data: sig.clone(),
                key_index: 0,
                metadata: None,
            });
        }

        public_keys.push(PublicKey {
            data: tx.sender.clone(),
            key_type: KeyType::Ed25519,
        });

        let authorization = AuthorizationPackage {
            signatures,
            public_keys,
            signature_scheme: SignatureScheme::EdDsa, // Algorand uses Ed25519
        };

        // Operations based on transaction type
        let mut operations = Vec::new();

        match tx.tx_type {
            AlgorandTxType::Payment => {
                if let (Some(receiver), Some(amount)) = (&tx.receiver, tx.amount) {
                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: tx.sender.clone(),
                            human_readable: Some(encode_address(&tx.sender)),
                        },
                        to: Address {
                            bytes: receiver.clone(),
                            human_readable: Some(encode_address(receiver)),
                        },
                        amount: Amount {
                            value: amount as u128,
                            decimals: 6, // ALGO has 6 decimals (microALGOs)
                        },
                        asset: AssetId::Native,
                    }));
                }
            }
            AlgorandTxType::AssetTransfer => {
                if let (Some(receiver), Some(amount)) = (&tx.asset_receiver, tx.asset_amount) {
                    let asset_id = tx
                        .xfer_asset
                        .map(|id| AssetId::Token(id.to_le_bytes().to_vec()))
                        .unwrap_or(AssetId::Native);

                    operations.push(Operation::Transfer(Transfer {
                        from: Address {
                            bytes: tx.sender.clone(),
                            human_readable: Some(encode_address(&tx.sender)),
                        },
                        to: Address {
                            bytes: receiver.clone(),
                            human_readable: Some(encode_address(receiver)),
                        },
                        amount: Amount {
                            value: amount as u128,
                            decimals: 0, // Asset decimals unknown, use 0
                        },
                        asset: asset_id,
                    }));
                }
            }
            AlgorandTxType::ApplicationCall => {
                if let Some(app_id) = tx.application_id {
                    // Flatten app_arguments into single byte vector
                    let call_data = tx
                        .app_arguments
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .flatten()
                        .collect();

                    operations.push(Operation::ContractCall(ContractCall {
                        contract: Address {
                            bytes: app_id.to_le_bytes().to_vec(),
                            human_readable: Some(format!("app-{}", app_id)),
                        },
                        method: tx
                            .on_completion
                            .map(|oc| vec![oc as u8])
                            .unwrap_or_default(),
                        data: call_data,
                        value: None,
                        resource_limits: ResourceLimits {
                            max_units: 700, // Algorand app call budget
                            unit_price: 0,
                            resource_type: ResourceType::ComputeUnits,
                        },
                    }));
                }
            }
            AlgorandTxType::AssetConfig => {
                operations.push(Operation::Generic(GenericOperation {
                    op_type: "asset_config".to_string(),
                    data: format!("asset_id={:?}", tx.config_asset).into_bytes(),
                    metadata: String::new(),
                }));
            }
            AlgorandTxType::KeyRegistration => {
                operations.push(Operation::Generic(GenericOperation {
                    op_type: "key_registration".to_string(),
                    data: format!(
                        "vote_first={:?} vote_last={:?}",
                        tx.vote_first, tx.vote_last
                    )
                    .into_bytes(),
                    metadata: String::new(),
                }));
            }
            AlgorandTxType::AssetFreeze => {
                operations.push(Operation::Generic(GenericOperation {
                    op_type: "asset_freeze".to_string(),
                    data: vec![],
                    metadata: String::new(),
                }));
            }
        }

        // Balance-effect guesses are NOT byte-derivable and were removed
        // from TxIR (docs/CONCEPTS_REVIEW.md C1).
        let state_deltas = StateDeltas {
            inputs: vec![],
            outputs: vec![],
        };

        Ok(TxIR::new(
            &AlgorandChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        let tx = &self.signed_tx.transaction;

        // Validate sender address
        if tx.sender.len() != 32 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid sender address length: {}",
                tx.sender.len()
            )));
        }

        // Validate genesis hash
        if tx.genesis_hash.len() != 32 {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid genesis hash length: {}",
                tx.genesis_hash.len()
            )));
        }

        // Validate first_valid < last_valid
        if tx.first_valid >= tx.last_valid {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid round range: {} >= {}",
                tx.first_valid, tx.last_valid
            )));
        }

        // Type-specific validation
        match tx.tx_type {
            AlgorandTxType::Payment => {
                if tx.receiver.is_none() {
                    return Err(DecoderError::invalid_structure(
                        "Payment transaction must have receiver",
                    ));
                }
                if tx.amount.is_none() {
                    return Err(DecoderError::invalid_structure(
                        "Payment transaction must have amount",
                    ));
                }
            }
            AlgorandTxType::AssetTransfer => {
                if tx.xfer_asset.is_none() {
                    return Err(DecoderError::invalid_structure(
                        "Asset transfer must have asset ID",
                    ));
                }
            }
            _ => {} // Other types have different validation rules
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = AlgorandDecoder::chain();
        assert_eq!(chain.chain_id(), 4160);
        assert_eq!(chain.chain_name(), "Algorand");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_empty_transaction_rejected() {
        let result = AlgorandDecoder::decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_msgpack_rejected() {
        // Not a valid MessagePack map
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
        let result = AlgorandDecoder::decode(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_base32_encoding() {
        // Test vector: empty input
        assert_eq!(base32_encode(&[]), "");

        // Test vector: "f" -> "MY======"
        assert_eq!(base32_encode(b"f").trim_end_matches('='), "MY");

        // Test vector: "fo" -> "MZXQ===="
        assert_eq!(base32_encode(b"fo").trim_end_matches('='), "MZXQ");
    }

    #[test]
    fn test_encode_address() {
        // 32-byte public key (all zeros for test)
        let pubkey = vec![0u8; 32];
        let addr = encode_address(&pubkey);

        // Should be 58 characters (32 bytes + 4 checksum = 36 bytes * 8/5 = 57.6)
        assert!(addr.len() >= 58 && addr.len() <= 60);
        assert!(addr
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_signature_scheme() {
        let tx = AlgorandTransaction {
            raw_bytes: vec![],
            signed_tx: SignedTransaction {
                signature: Some(vec![0u8; 64]),
                transaction: create_minimal_payment_tx(),
                auth_addr: None,
            },
        };

        let tx_ir = tx.canonicalize().unwrap();
        assert_eq!(tx_ir.authorization.signature_scheme, SignatureScheme::EdDsa);
    }

    fn create_minimal_payment_tx() -> RawTransaction {
        RawTransaction {
            tx_type: AlgorandTxType::Payment,
            sender: vec![0u8; 32],
            fee: 1000,
            first_valid: 1000,
            last_valid: 2000,
            genesis_id: Some("mainnet-v1.0".to_string()),
            genesis_hash: vec![0u8; 32],
            note: None,
            group: None,
            lease: None,
            rekey_to: None,
            receiver: Some(vec![1u8; 32]),
            amount: Some(1000000),
            close_remainder_to: None,
            xfer_asset: None,
            asset_amount: None,
            asset_sender: None,
            asset_receiver: None,
            asset_close_to: None,
            application_id: None,
            on_completion: None,
            app_arguments: None,
            accounts: None,
            foreign_apps: None,
            foreign_assets: None,
            config_asset: None,
            vote_pk: None,
            selection_pk: None,
            vote_first: None,
            vote_last: None,
            vote_key_dilution: None,
        }
    }
}
