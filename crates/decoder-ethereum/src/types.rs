//! Ethereum-specific transaction types
//!
//! Pure Rust implementation using custom RLP decoder.
//! Supports Legacy, EIP-2930, EIP-1559, and EIP-4844 transactions.

use borsh::{BorshDeserialize, BorshSerialize};
use decoder_encodings::rlp::RlpItem;
use decoder_encodings::rlp_encoder::RlpEncoder;
use serde::{Deserialize, Serialize};
use universal_decoder_core::prelude::*;

// ECDSA signature recovery
use k256::ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

/// Ethereum transaction type indicator (EIP-2718)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxType {
    /// Legacy transaction (pre-EIP-2718)
    Legacy = 0,
    /// EIP-2930: Optional access lists
    Eip2930 = 1,
    /// EIP-1559: Fee market change
    Eip1559 = 2,
    /// EIP-4844: Blob transactions (Proto-Danksharding)
    Eip4844 = 3,
}

impl BorshSerialize for TxType {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let value: u8 = match self {
            TxType::Legacy => 0,
            TxType::Eip2930 => 1,
            TxType::Eip1559 => 2,
            TxType::Eip4844 => 3,
        };
        BorshSerialize::serialize(&value, writer)
    }
}

impl BorshDeserialize for TxType {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let value = u8::deserialize_reader(reader)?;
        match value {
            0 => Ok(TxType::Legacy),
            1 => Ok(TxType::Eip2930),
            2 => Ok(TxType::Eip1559),
            3 => Ok(TxType::Eip4844),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown transaction type: {}", value),
            )),
        }
    }
}

impl TxType {
    /// Parse transaction type from byte
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            1 => Ok(TxType::Eip2930),
            2 => Ok(TxType::Eip1559),
            3 => Ok(TxType::Eip4844),
            _ => Err(DecoderError::invalid_structure(format!(
                "Unknown transaction type: {}",
                byte
            ))),
        }
    }
}

/// Ethereum-specific transaction representation
///
/// Pure Rust implementation supporting all transaction types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EthereumTransaction {
    /// Transaction type
    pub tx_type: TxType,
    /// Nonce
    pub nonce: u64,
    /// Gas price (legacy transactions)
    pub gas_price: Option<u128>,
    /// Gas limit
    pub gas_limit: u128,
    /// Recipient address (None for contract creation)
    pub to: Option<[u8; 20]>,
    /// Value in Wei
    pub value: u128,
    /// Call data or contract bytecode
    pub data: Vec<u8>,
    /// Chain ID
    pub chain_id: Option<u64>,

    // EIP-1559 fields
    /// Max fee per gas (EIP-1559)
    pub max_fee_per_gas: Option<u128>,
    /// Max priority fee per gas (EIP-1559)
    pub max_priority_fee_per_gas: Option<u128>,

    // EIP-2930 fields
    /// Access list (EIP-2930)
    pub access_list: Vec<AccessListItem>,

    // Signature
    /// Signature v component
    pub v: u64,
    /// Signature r component (32 bytes)
    pub r: [u8; 32],
    /// Signature s component (32 bytes)
    pub s: [u8; 32],
    // NOTE: No raw_bytes field - bytes must be reconstructed from fields
}

/// Access list item (EIP-2930)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AccessListItem {
    /// Address
    pub address: [u8; 20],
    /// Storage keys
    pub storage_keys: Vec<[u8; 32]>,
}

impl EthereumTransaction {
    /// Create from raw RLP-encoded bytes
    pub fn from_raw_bytes(raw_bytes: &[u8]) -> Result<Self> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure("Empty transaction bytes"));
        }

        // Check if it's a typed transaction (EIP-2718)
        let first_byte = raw_bytes[0];

        if first_byte <= 0x7f {
            // Typed transaction: first byte is type
            let tx_type = TxType::from_byte(first_byte)?;
            Self::parse_typed_transaction(tx_type, &raw_bytes[1..])
        } else {
            // Legacy transaction: starts with RLP list prefix
            Self::parse_legacy_transaction(raw_bytes)
        }
    }

    /// Parse legacy transaction (pre-EIP-2718)
    fn parse_legacy_transaction(raw_bytes: &[u8]) -> Result<Self> {
        let rlp = RlpItem::decode(raw_bytes)?;
        let items = rlp.as_list()?;

        // Legacy transaction has 9 fields:
        // [nonce, gasPrice, gasLimit, to, value, data, v, r, s]
        if items.len() != 9 {
            return Err(DecoderError::invalid_structure(format!(
                "Legacy transaction must have 9 fields, got {}",
                items.len()
            )));
        }

        let nonce = items[0].as_u64()?;
        let gas_price = items[1].as_u128()?;
        let gas_limit = items[2].as_u128()?;

        // Parse 'to' address (empty for contract creation)
        let to_data = items[3].as_data()?;
        let to = if to_data.is_empty() {
            None
        } else if to_data.len() == 20 {
            let mut addr = [0u8; 20];
            addr.copy_from_slice(to_data);
            Some(addr)
        } else {
            return Err(DecoderError::invalid_structure(
                "Invalid address length (must be 20 bytes or empty)",
            ));
        };

        let value = items[4].as_u128()?;
        let data = items[5].as_data()?.to_vec();
        let v = items[6].as_u64()?;

        // Parse r and s (32 bytes each)
        let r_data = items[7].as_data()?;
        let s_data = items[8].as_data()?;

        let r = parse_signature_component(r_data, "r")?;
        let s = parse_signature_component(s_data, "s")?;

        // Extract chain_id from v (EIP-155)
        let chain_id = if v >= 35 { Some((v - 35) / 2) } else { None };

        Ok(Self {
            tx_type: TxType::Legacy,
            nonce,
            gas_price: Some(gas_price),
            gas_limit,
            to,
            value,
            data,
            chain_id,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: vec![],
            v,
            r,
            s,
        })
    }

    /// Parse typed transaction (EIP-2718)
    fn parse_typed_transaction(tx_type: TxType, payload: &[u8]) -> Result<Self> {
        let rlp = RlpItem::decode(payload)?;
        let items = rlp.as_list()?;

        match tx_type {
            TxType::Eip2930 => Self::parse_eip2930(items),
            TxType::Eip1559 => Self::parse_eip1559(items),
            TxType::Eip4844 => Self::parse_eip4844(items),
            TxType::Legacy => Err(DecoderError::invalid_structure(
                "Legacy type should not be in typed transaction",
            )),
        }
    }

    /// Parse EIP-2930 transaction
    fn parse_eip2930(items: &[RlpItem]) -> Result<Self> {
        // EIP-2930: [chainId, nonce, gasPrice, gasLimit, to, value, data, accessList, signatureYParity, signatureR, signatureS]
        if items.len() != 11 {
            return Err(DecoderError::invalid_structure(format!(
                "EIP-2930 transaction must have 11 fields, got {}",
                items.len()
            )));
        }

        let chain_id = items[0].as_u64()?;
        let nonce = items[1].as_u64()?;
        let gas_price = items[2].as_u128()?;
        let gas_limit = items[3].as_u128()?;
        let to = parse_address_field(&items[4])?;
        let value = items[5].as_u128()?;
        let data = items[6].as_data()?.to_vec();
        let access_list = parse_access_list(&items[7])?;
        let v = items[8].as_u64()?;
        let r = parse_signature_component(items[9].as_data()?, "r")?;
        let s = parse_signature_component(items[10].as_data()?, "s")?;

        Ok(Self {
            tx_type: TxType::Eip2930,
            nonce,
            gas_price: Some(gas_price),
            gas_limit,
            to,
            value,
            data,
            chain_id: Some(chain_id),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list,
            v,
            r,
            s,
        })
    }

    /// Parse EIP-1559 transaction
    fn parse_eip1559(items: &[RlpItem]) -> Result<Self> {
        // EIP-1559: [chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, signatureYParity, signatureR, signatureS]
        if items.len() != 12 {
            return Err(DecoderError::invalid_structure(format!(
                "EIP-1559 transaction must have 12 fields, got {}",
                items.len()
            )));
        }

        let chain_id = items[0].as_u64()?;
        let nonce = items[1].as_u64()?;
        let max_priority_fee_per_gas = items[2].as_u128()?;
        let max_fee_per_gas = items[3].as_u128()?;
        let gas_limit = items[4].as_u128()?;
        let to = parse_address_field(&items[5])?;
        let value = items[6].as_u128()?;
        let data = items[7].as_data()?.to_vec();
        let access_list = parse_access_list(&items[8])?;
        let v = items[9].as_u64()?;
        let r = parse_signature_component(items[10].as_data()?, "r")?;
        let s = parse_signature_component(items[11].as_data()?, "s")?;

        Ok(Self {
            tx_type: TxType::Eip1559,
            nonce,
            gas_price: None,
            gas_limit,
            to,
            value,
            data,
            chain_id: Some(chain_id),
            max_fee_per_gas: Some(max_fee_per_gas),
            max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
            access_list,
            v,
            r,
            s,
        })
    }

    /// Parse EIP-4844 transaction (blob transactions)
    fn parse_eip4844(items: &[RlpItem]) -> Result<Self> {
        // EIP-4844: Similar to EIP-1559 but with additional blob fields
        // For minimal implementation, we'll parse the core fields
        if items.len() < 12 {
            return Err(DecoderError::invalid_structure(format!(
                "EIP-4844 transaction must have at least 12 fields, got {}",
                items.len()
            )));
        }

        // Parse similar to EIP-1559 (blob fields can be ignored for basic decoding)
        Self::parse_eip1559(items).map(|mut tx| {
            tx.tx_type = TxType::Eip4844;
            tx
        })
    }

    /// Check if this is an EIP-1559 transaction
    pub fn is_eip1559(&self) -> bool {
        self.tx_type == TxType::Eip1559 || self.tx_type == TxType::Eip4844
    }

    /// Check if this is a contract creation
    pub fn is_contract_creation(&self) -> bool {
        self.to.is_none()
    }

    /// Calculate transaction hash using Keccak-256
    pub fn hash(&self) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        // Reconstruct bytes from fields - no stored raw_bytes
        let bytes = self.to_bytes().unwrap_or_default();
        Keccak256::digest(&bytes).to_vec()
    }

    /// Strip leading zeros from signature component for RLP encoding
    fn encode_signature_r(&self) -> Vec<u8> {
        strip_leading_zeros(&self.r)
    }

    /// Strip leading zeros from signature component for RLP encoding
    fn encode_signature_s(&self) -> Vec<u8> {
        strip_leading_zeros(&self.s)
    }

    /// Reconstruct legacy transaction RLP bytes
    fn reconstruct_legacy(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // [nonce, gasPrice, gasLimit, to, value, data, v, r, s]
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.gas_price)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;
        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;
        Ok(encoder.finalize())
    }

    /// Reconstruct EIP-2930 transaction bytes
    fn reconstruct_eip2930(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // [chainId, nonce, gasPrice, gasLimit, to, value, data, accessList, v, r, s]
        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.gas_price)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list
        self.append_access_list_rlp(&mut list)?;

        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;

        // Prepend type byte
        let mut result = vec![0x01];
        result.extend(encoder.finalize());
        Ok(result)
    }

    /// Reconstruct EIP-1559 transaction bytes
    fn reconstruct_eip1559(&self) -> Result<Vec<u8>> {
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // [chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, v, r, s]
        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.max_priority_fee_per_gas)?;
        list.append_optional_u128(self.max_fee_per_gas)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list
        self.append_access_list_rlp(&mut list)?;

        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;

        // Prepend type byte
        let mut result = vec![0x02];
        result.extend(encoder.finalize());
        Ok(result)
    }

    /// Reconstruct EIP-4844 transaction bytes
    fn reconstruct_eip4844(&self) -> Result<Vec<u8>> {
        // Same structure as EIP-1559, different type byte
        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.max_priority_fee_per_gas)?;
        list.append_optional_u128(self.max_fee_per_gas)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        self.append_access_list_rlp(&mut list)?;

        list.append_u64(self.v)?;
        list.append_bytes(&self.encode_signature_r())?;
        list.append_bytes(&self.encode_signature_s())?;

        list.finalize()?;

        let mut result = vec![0x03];
        result.extend(encoder.finalize());
        Ok(result)
    }

    /// Append access list to RLP encoder
    fn append_access_list_rlp(
        &self,
        list: &mut decoder_encodings::rlp_encoder::ListEncoder<'_>,
    ) -> Result<()> {
        list.append_list(|access_list_encoder| {
            for item in &self.access_list {
                access_list_encoder.append_list(|entry| {
                    entry.append_bytes(&item.address)?;
                    entry.append_list(|keys| {
                        for key in &item.storage_keys {
                            keys.append_bytes(key)?;
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
            Ok(())
        })?;
        Ok(())
    }

    /// Get effective gas price
    pub fn effective_gas_price(&self) -> u128 {
        self.gas_price.or(self.max_fee_per_gas).unwrap_or_default()
    }

    /// Get the sender address by recovering from the ECDSA signature
    ///
    /// This performs ECDSA public key recovery using the (v, r, s) signature
    /// components and the transaction's signing hash.
    ///
    /// Returns zero address if recovery fails (for compatibility).
    pub fn get_from(&self) -> [u8; 20] {
        self.recover_sender().unwrap_or([0u8; 20])
    }

    /// Recover the sender address from the signature
    ///
    /// This is the full ECDSA recovery implementation that:
    /// 1. Computes the signing hash
    /// 2. Recovers the public key from (signature, recovery_id, hash)
    /// 3. Derives the Ethereum address from the public key
    ///
    /// # Returns
    ///
    /// The 20-byte Ethereum address, or an error if recovery fails
    pub fn recover_sender(&self) -> Result<[u8; 20]> {
        // Step 1: Compute the signing hash
        let signing_hash = self.signing_hash()?;

        // Step 2: Extract recovery ID from v
        let recovery_id = self.get_recovery_id()?;

        // Step 3: Construct signature from r and s
        let mut sig_bytes = [0u8; 64];
        sig_bytes[0..32].copy_from_slice(&self.r);
        sig_bytes[32..64].copy_from_slice(&self.s);

        let signature = K256Signature::from_bytes(&sig_bytes.into()).map_err(|e| {
            DecoderError::signature_verification(format!("Invalid signature: {}", e))
        })?;

        // Step 4: Recover public key
        let verifying_key =
            VerifyingKey::recover_from_prehash(&signing_hash, &signature, recovery_id).map_err(
                |e| DecoderError::signature_verification(format!("Recovery failed: {}", e)),
            )?;

        // Step 5: Derive Ethereum address from public key
        // Address = keccak256(uncompressed_pubkey)[12..32]
        let pubkey_bytes = verifying_key.to_encoded_point(false); // Uncompressed format
        let pubkey = &pubkey_bytes.as_bytes()[1..]; // Remove 0x04 prefix

        let hash = Keccak256::digest(pubkey);
        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..32]);

        Ok(address)
    }

    /// Compute the signing hash for this transaction
    ///
    /// This is the hash that was actually signed by the sender.
    fn signing_hash(&self) -> Result<[u8; 32]> {
        match self.tx_type {
            TxType::Legacy => self.legacy_signing_hash(),
            TxType::Eip2930 => self.typed_signing_hash(0x01),
            TxType::Eip1559 | TxType::Eip4844 => self.typed_signing_hash(0x02),
        }
    }

    /// Legacy transaction signing hash
    ///
    /// For EIP-155: hash(rlp([nonce, gasPrice, gas, to, value, data, chainId, 0, 0]))
    /// For pre-EIP-155: hash(rlp([nonce, gasPrice, gas, to, value, data]))
    fn legacy_signing_hash(&self) -> Result<[u8; 32]> {
        use decoder_encodings::rlp_encoder::RlpEncoder;
        use sha3::{Digest, Keccak256};

        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        list.append_u64(self.nonce)?;
        list.append_optional_u128(self.gas_price)?;
        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // EIP-155: append chain_id, 0, 0
        if let Some(chain_id) = self.chain_id {
            list.append_u64(chain_id)?;
            list.append_u64(0)?;
            list.append_u64(0)?;
        }

        list.finalize()?;
        let rlp_bytes = encoder.finalize();

        Ok(Keccak256::digest(&rlp_bytes).into())
    }

    /// Typed transaction signing hash (EIP-2930, EIP-1559, EIP-4844)
    ///
    /// hash(type_byte || rlp([...transaction_fields...]))
    fn typed_signing_hash(&self, type_byte: u8) -> Result<[u8; 32]> {
        use decoder_encodings::rlp_encoder::RlpEncoder;
        use sha3::{Digest, Keccak256};

        let mut encoder = RlpEncoder::new();
        let mut list = encoder.begin_list();

        // All typed transactions include chain_id first
        list.append_u64(self.chain_id.unwrap_or(1))?;
        list.append_u64(self.nonce)?;

        // EIP-1559/4844 use max fees, EIP-2930 uses gas_price
        if type_byte == 0x02 {
            list.append_optional_u128(self.max_priority_fee_per_gas)?;
            list.append_optional_u128(self.max_fee_per_gas)?;
        } else {
            list.append_optional_u128(self.gas_price)?;
        }

        list.append_u128(self.gas_limit)?;
        list.append_address(self.to)?;
        list.append_u128(self.value)?;
        list.append_bytes(&self.data)?;

        // Access list (for EIP-2930 and EIP-1559)
        list.append_list(|access_list| {
            for item in &self.access_list {
                access_list.append_list(|entry| {
                    entry.append_bytes(&item.address)?;
                    entry.append_list(|keys| {
                        for key in &item.storage_keys {
                            keys.append_bytes(key)?;
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
            }
            Ok(())
        })?;

        list.finalize()?;
        let rlp_bytes = encoder.finalize();

        // Prepend type byte and hash
        let mut payload = vec![type_byte];
        payload.extend_from_slice(&rlp_bytes);

        Ok(Keccak256::digest(&payload).into())
    }

    /// Extract ECDSA recovery ID from v
    ///
    /// For legacy: v = chain_id * 2 + 35 + recovery_id (EIP-155)
    ///          or v = 27 + recovery_id (pre-EIP-155)
    /// For typed: v = recovery_id (0 or 1)
    fn get_recovery_id(&self) -> Result<RecoveryId> {
        let recovery_id = match self.tx_type {
            TxType::Legacy => {
                if self.chain_id.is_some() {
                    // EIP-155: v = chain_id * 2 + 35 + recovery_id
                    ((self.v - 35) % 2) as u8
                } else {
                    // Pre-EIP-155: v = 27 + recovery_id
                    (self.v - 27) as u8
                }
            }
            _ => {
                // EIP-2930/EIP-1559: v is recovery_id directly (0 or 1)
                self.v as u8
            }
        };

        RecoveryId::try_from(recovery_id).map_err(|e| {
            DecoderError::signature_verification(format!(
                "Invalid recovery ID {}: {}",
                recovery_id, e
            ))
        })
    }

    /// Get transaction type as u8
    pub fn tx_type_u8(&self) -> u8 {
        match self.tx_type {
            TxType::Legacy => 0,
            TxType::Eip2930 => 1,
            TxType::Eip1559 => 2,
            TxType::Eip4844 => 3,
        }
    }
}

/// Parse address field from RLP item
fn parse_address_field(item: &RlpItem) -> Result<Option<[u8; 20]>> {
    let data = item.as_data()?;

    if data.is_empty() {
        Ok(None)
    } else if data.len() == 20 {
        let mut addr = [0u8; 20];
        addr.copy_from_slice(data);
        Ok(Some(addr))
    } else {
        Err(DecoderError::invalid_structure(
            "Invalid address length (must be 20 bytes or empty)",
        ))
    }
}

/// Parse signature component (r or s) from RLP data
fn parse_signature_component(data: &[u8], name: &str) -> Result<[u8; 32]> {
    if data.len() > 32 {
        return Err(DecoderError::invalid_structure(format!(
            "Signature component {} too large (max 32 bytes)",
            name
        )));
    }

    let mut component = [0u8; 32];
    // Right-pad with zeros if less than 32 bytes
    let offset = 32 - data.len();
    component[offset..].copy_from_slice(data);

    Ok(component)
}

/// Strip leading zeros from a byte array for RLP encoding
fn strip_leading_zeros(bytes: &[u8]) -> Vec<u8> {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    bytes[start..].to_vec()
}

/// Parse access list from RLP item
fn parse_access_list(item: &RlpItem) -> Result<Vec<AccessListItem>> {
    let list = item.as_list()?;
    let mut access_list = Vec::new();

    for entry in list {
        let entry_items = entry.as_list()?;
        if entry_items.len() != 2 {
            return Err(DecoderError::invalid_structure(
                "Access list entry must have 2 fields [address, storageKeys]",
            ));
        }

        let addr_data = entry_items[0].as_data()?;
        if addr_data.len() != 20 {
            return Err(DecoderError::invalid_structure(
                "Access list address must be 20 bytes",
            ));
        }

        let mut address = [0u8; 20];
        address.copy_from_slice(addr_data);

        let storage_keys_list = entry_items[1].as_list()?;
        let mut storage_keys = Vec::new();

        for key_item in storage_keys_list {
            let key_data = key_item.as_data()?;
            if key_data.len() != 32 {
                return Err(DecoderError::invalid_structure(
                    "Storage key must be 32 bytes",
                ));
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(key_data);
            storage_keys.push(key);
        }

        access_list.push(AccessListItem {
            address,
            storage_keys,
        });
    }

    Ok(access_list)
}

impl ChainEncoder for EthereumTransaction {
    /// Re-encode the Ethereum transaction back to RLP-encoded bytes.
    ///
    /// This reconstructs the transaction from parsed fields, guaranteeing
    /// that the decoder actually parsed the data rather than just storing bytes.
    ///
    /// # Formal Properties
    ///
    /// The injective property is satisfied through actual reconstruction:
    /// ```text
    /// ∀ tx_bytes: EthereumDecoder::decode(tx_bytes)?.to_bytes()? == tx_bytes
    /// ```
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self.tx_type {
            TxType::Legacy => self.reconstruct_legacy(),
            TxType::Eip2930 => self.reconstruct_eip2930(),
            TxType::Eip1559 => self.reconstruct_eip1559(),
            TxType::Eip4844 => self.reconstruct_eip4844(),
        }
    }
}

impl ReconstructableTransaction for EthereumTransaction {
    fn reconstruct_bytes(&self) -> Result<Vec<u8>> {
        self.to_bytes()
    }
}

impl<'a> Canonicalizer<'a> for EthereumTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Recover sender address from ECDSA signature
        let sender_address = self.recover_sender()?;

        // Build metadata with access list information
        let access_list_json = if self.access_list.is_empty() {
            "[]".to_string()
        } else {
            let items: Vec<String> = self
                .access_list
                .iter()
                .map(|item| {
                    let storage_keys: Vec<String> = item
                        .storage_keys
                        .iter()
                        .map(|key| format!("\"0x{}\"", universal_decoder_core::hex::encode(key)))
                        .collect();
                    format!(
                        r#"{{"address":"0x{}","storage_keys":[{}]}}"#,
                        universal_decoder_core::hex::encode(item.address),
                        storage_keys.join(",")
                    )
                })
                .collect();
            format!("[{}]", items.join(","))
        };

        let extra = format!(
            r#"{{"tx_type":{:?},"nonce":{},"gas_limit":{},"gas_price":{},"max_fee_per_gas":{},"max_priority_fee_per_gas":{},"chain_id":{},"access_list":{}}}"#,
            self.tx_type,
            self.nonce,
            self.gas_limit,
            self.gas_price
                .map(|p| p.to_string())
                .unwrap_or_else(|| "null".to_string()),
            self.max_fee_per_gas
                .map(|p| p.to_string())
                .unwrap_or_else(|| "null".to_string()),
            self.max_priority_fee_per_gas
                .map(|p| p.to_string())
                .unwrap_or_else(|| "null".to_string()),
            self.chain_id
                .map(|c| c.to_string())
                .unwrap_or_else(|| "null".to_string()),
            access_list_json
        );

        // Get size from reconstructed bytes
        let tx_bytes = self.to_bytes()?;
        let metadata = TxMetadata {
            tx_hash: self.hash(),
            block_height: None,
            timestamp: None,
            size: tx_bytes.len(),
            extra,
        };

        // Build authorization package
        let signature = Signature {
            data: {
                let mut sig_bytes = Vec::new();
                sig_bytes.extend_from_slice(&self.r);
                sig_bytes.extend_from_slice(&self.s);
                sig_bytes.push(self.v as u8);
                sig_bytes
            },
            key_index: 0,
            metadata: Some(format!(r#"{{"v":{}}}"#, self.v)),
        };

        let authorization = AuthorizationPackage {
            signatures: vec![signature],
            public_keys: vec![],
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Build operations
        let mut operations = Vec::new();

        if self.is_contract_creation() {
            operations.push(Operation::ContractDeploy(ContractDeploy {
                bytecode: self.data.clone(),
                constructor_args: vec![],
                value: Amount {
                    value: self.value,
                    decimals: 18,
                },
            }));
        } else if !self.data.is_empty() {
            let method = if self.data.len() >= 4 {
                self.data[0..4].to_vec()
            } else {
                vec![]
            };

            operations.push(Operation::ContractCall(ContractCall {
                contract: Address {
                    bytes: self.to.map(|a| a.to_vec()).unwrap_or_default(),
                    human_readable: self
                        .to
                        .map(|a| format!("0x{}", universal_decoder_core::hex::encode(a))),
                },
                method,
                data: self.data.clone(),
                value: Some(Amount {
                    value: self.value,
                    decimals: 18,
                }),
                resource_limits: ResourceLimits {
                    max_units: self.gas_limit as u64,
                    unit_price: self.effective_gas_price() as u64,
                    resource_type: ResourceType::Gas,
                },
            }));
        } else {
            operations.push(Operation::Transfer(Transfer {
                from: Address {
                    bytes: sender_address.to_vec(),
                    human_readable: Some(format!(
                        "0x{}",
                        universal_decoder_core::hex::encode(sender_address)
                    )),
                },
                to: Address {
                    bytes: self.to.map(|a| a.to_vec()).unwrap_or_default(),
                    human_readable: self
                        .to
                        .map(|a| format!("0x{}", universal_decoder_core::hex::encode(a))),
                },
                amount: Amount {
                    value: self.value,
                    decimals: 18,
                },
                asset: AssetId::Native,
            }));
        }

        // Build state deltas
        let mut account_changes = vec![AccountChange {
            address: Address {
                bytes: sender_address.to_vec(),
                human_readable: Some(format!(
                    "0x{}",
                    universal_decoder_core::hex::encode(sender_address)
                )),
            },
            nonce: Some(self.nonce),
            balance_change: -(self.value as i128),
            storage_changes: vec![],
        }];

        if let Some(recipient) = self.to {
            account_changes.push(AccountChange {
                address: Address {
                    bytes: recipient.to_vec(),
                    human_readable: Some(format!(
                        "0x{}",
                        universal_decoder_core::hex::encode(recipient)
                    )),
                },
                nonce: None,
                balance_change: self.value as i128,
                storage_changes: vec![],
            });
        }

        let state_deltas = StateDeltas {
            inputs: vec![],
            outputs: vec![],
            account_changes,
        };

        // Use the actual chain ID from the transaction, not hardcoded EthereumChain
        // This fixes the bug where Polygon (chain_id=137) was incorrectly showing as Ethereum (chain_id=1)
        let chain = crate::get_evm_chain_by_id(self.chain_id.unwrap_or(1));

        Ok(TxIR::new(
            chain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        if self.gas_limit == 0 {
            return Err(DecoderError::invalid_structure("Gas limit cannot be zero"));
        }

        if self.gas_price.is_none()
            && (self.max_fee_per_gas.is_none() || self.max_priority_fee_per_gas.is_none())
        {
            return Err(DecoderError::invalid_structure(
                "Must have either gas_price or EIP-1559 fee fields",
            ));
        }

        if self.r == [0u8; 32] || self.s == [0u8; 32] {
            return Err(DecoderError::signature_verification(
                "Invalid signature: r or s is zero",
            ));
        }

        Ok(())
    }
}

impl TxHashable for EthereumTransaction {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        // Reconstruct bytes from parsed fields
        self.to_bytes().unwrap_or_default()
    }

    fn compute_hash(&self) -> Vec<u8> {
        self.compute_hash_with::<Keccak256Hash>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_transaction_version() {
        assert_eq!(EthereumTransaction::VERSION, 1);
    }

    #[test]
    fn test_tx_type_from_byte() {
        assert_eq!(TxType::from_byte(1).unwrap(), TxType::Eip2930);
        assert_eq!(TxType::from_byte(2).unwrap(), TxType::Eip1559);
        assert_eq!(TxType::from_byte(3).unwrap(), TxType::Eip4844);
        assert!(TxType::from_byte(99).is_err());
    }

    #[test]
    fn test_parse_signature_component() {
        // Test with 32 bytes
        let data = [0x42u8; 32];
        let result = parse_signature_component(&data, "r").unwrap();
        assert_eq!(result, data);

        // Test with less than 32 bytes (should be left-padded with zeros)
        let data = vec![0x01, 0x02, 0x03];
        let result = parse_signature_component(&data, "r").unwrap();
        assert_eq!(&result[29..], &[0x01, 0x02, 0x03]);
        assert_eq!(&result[..29], &[0u8; 29]);

        // Test with more than 32 bytes (should error)
        let data = vec![0u8; 33];
        assert!(parse_signature_component(&data, "r").is_err());
    }
}
