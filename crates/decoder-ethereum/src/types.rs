//! Ethereum-specific transaction types
//!
//! Pure Rust implementation using custom RLP decoder.
//! Supports Legacy, EIP-2930, EIP-1559, and EIP-4844 transactions.

use universal_decoder_core::prelude::*;
use decoder_encodings::rlp::RlpItem;
use crate::EthereumChain;

/// Ethereum transaction type indicator (EIP-2718)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl TxType {
    /// Parse transaction type from byte
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            1 => Ok(TxType::Eip2930),
            2 => Ok(TxType::Eip1559),
            3 => Ok(TxType::Eip4844),
            _ => Err(DecoderError::invalid_structure(
                &format!("Unknown transaction type: {}", byte)
            )),
        }
    }
}

/// Ethereum-specific transaction representation
///
/// Pure Rust implementation supporting all transaction types.
#[derive(Debug, Clone)]
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

    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

/// Access list item (EIP-2930)
#[derive(Debug, Clone)]
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
            return Err(DecoderError::invalid_structure(
                "Empty transaction bytes"
            ));
        }

        // Check if it's a typed transaction (EIP-2718)
        let first_byte = raw_bytes[0];

        if first_byte <= 0x7f {
            // Typed transaction: first byte is type
            let tx_type = TxType::from_byte(first_byte)?;
            Self::parse_typed_transaction(tx_type, &raw_bytes[1..], raw_bytes)
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
            return Err(DecoderError::invalid_structure(
                &format!("Legacy transaction must have 9 fields, got {}", items.len())
            ));
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
                "Invalid address length (must be 20 bytes or empty)"
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
        let chain_id = if v >= 35 {
            Some((v - 35) / 2)
        } else {
            None
        };

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
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    /// Parse typed transaction (EIP-2718)
    fn parse_typed_transaction(tx_type: TxType, payload: &[u8], raw_bytes: &[u8]) -> Result<Self> {
        let rlp = RlpItem::decode(payload)?;
        let items = rlp.as_list()?;

        match tx_type {
            TxType::Eip2930 => Self::parse_eip2930(items, raw_bytes),
            TxType::Eip1559 => Self::parse_eip1559(items, raw_bytes),
            TxType::Eip4844 => Self::parse_eip4844(items, raw_bytes),
            TxType::Legacy => Err(DecoderError::invalid_structure(
                "Legacy type should not be in typed transaction"
            )),
        }
    }

    /// Parse EIP-2930 transaction
    fn parse_eip2930(items: &[RlpItem], raw_bytes: &[u8]) -> Result<Self> {
        // EIP-2930: [chainId, nonce, gasPrice, gasLimit, to, value, data, accessList, signatureYParity, signatureR, signatureS]
        if items.len() != 11 {
            return Err(DecoderError::invalid_structure(
                &format!("EIP-2930 transaction must have 11 fields, got {}", items.len())
            ));
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
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    /// Parse EIP-1559 transaction
    fn parse_eip1559(items: &[RlpItem], raw_bytes: &[u8]) -> Result<Self> {
        // EIP-1559: [chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gasLimit, to, value, data, accessList, signatureYParity, signatureR, signatureS]
        if items.len() != 12 {
            return Err(DecoderError::invalid_structure(
                &format!("EIP-1559 transaction must have 12 fields, got {}", items.len())
            ));
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
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    /// Parse EIP-4844 transaction (blob transactions)
    fn parse_eip4844(items: &[RlpItem], raw_bytes: &[u8]) -> Result<Self> {
        // EIP-4844: Similar to EIP-1559 but with additional blob fields
        // For minimal implementation, we'll parse the core fields
        if items.len() < 12 {
            return Err(DecoderError::invalid_structure(
                &format!("EIP-4844 transaction must have at least 12 fields, got {}", items.len())
            ));
        }

        // Parse similar to EIP-1559 (blob fields can be ignored for basic decoding)
        Self::parse_eip1559(items, raw_bytes).map(|mut tx| {
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
        Keccak256::digest(&self.raw_bytes).to_vec()
    }

    /// Get effective gas price
    pub fn effective_gas_price(&self) -> u128 {
        if let Some(gas_price) = self.gas_price {
            gas_price
        } else if let Some(max_fee) = self.max_fee_per_gas {
            max_fee
        } else {
            0
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
            "Invalid address length (must be 20 bytes or empty)"
        ))
    }
}

/// Parse signature component (r or s) from RLP data
fn parse_signature_component(data: &[u8], name: &str) -> Result<[u8; 32]> {
    if data.len() > 32 {
        return Err(DecoderError::invalid_structure(
            &format!("Signature component {} too large (max 32 bytes)", name)
        ));
    }

    let mut component = [0u8; 32];
    // Right-pad with zeros if less than 32 bytes
    let offset = 32 - data.len();
    component[offset..].copy_from_slice(data);

    Ok(component)
}

/// Parse access list from RLP item
fn parse_access_list(item: &RlpItem) -> Result<Vec<AccessListItem>> {
    let list = item.as_list()?;
    let mut access_list = Vec::new();

    for entry in list {
        let entry_items = entry.as_list()?;
        if entry_items.len() != 2 {
            return Err(DecoderError::invalid_structure(
                "Access list entry must have 2 fields [address, storageKeys]"
            ));
        }

        let addr_data = entry_items[0].as_data()?;
        if addr_data.len() != 20 {
            return Err(DecoderError::invalid_structure(
                "Access list address must be 20 bytes"
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
                    "Storage key must be 32 bytes"
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

impl<'a> Canonicalizer<'a> for EthereumTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Build metadata
        let extra = format!(
            r#"{{"tx_type":{:?},"nonce":{},"gas_limit":{},"gas_price":{},"max_fee_per_gas":{},"max_priority_fee_per_gas":{},"chain_id":{}}}"#,
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
                .unwrap_or_else(|| "null".to_string())
        );

        let metadata = TxMetadata {
            tx_hash: self.hash(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
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
                    human_readable: self.to.map(|a| format!("0x{}", universal_decoder_core::hex::encode(a))),
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
                    bytes: vec![],
                    human_readable: None,
                },
                to: Address {
                    bytes: self.to.map(|a| a.to_vec()).unwrap_or_default(),
                    human_readable: self.to.map(|a| format!("0x{}", universal_decoder_core::hex::encode(a))),
                },
                amount: Amount {
                    value: self.value,
                    decimals: 18,
                },
                asset: AssetId::Native,
            }));
        }

        // Build state deltas
        let mut account_changes = vec![
            AccountChange {
                address: Address {
                    bytes: vec![],
                    human_readable: None,
                },
                nonce: Some(self.nonce),
                balance_change: -(self.value as i128),
                storage_changes: vec![],
            },
        ];

        if let Some(recipient) = self.to {
            account_changes.push(AccountChange {
                address: Address {
                    bytes: recipient.to_vec(),
                    human_readable: Some(format!("0x{}", universal_decoder_core::hex::encode(recipient))),
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

        Ok(TxIR::new(
            &EthereumChain,
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
        self.raw_bytes.clone()
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
