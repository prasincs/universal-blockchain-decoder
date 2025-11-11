//! Ethereum-specific transaction types

use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Bytes, NameOrAddress, Transaction as EthTx,
    TransactionRequest, U256, U64,
};
use universal_decoder_core::prelude::*;

/// Ethereum-specific transaction representation
///
/// Supports different transaction types (Legacy, EIP-2930, EIP-1559)
#[derive(Debug, Clone)]
pub struct EthereumTransaction {
    /// Transaction data
    pub nonce: u64,
    pub gas_price: Option<U256>,
    pub gas_limit: U256,
    pub to: Option<ethers_core::types::Address>,
    pub value: U256,
    pub data: Vec<u8>,
    pub chain_id: Option<U64>,

    // EIP-1559 fields
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    // Signature
    pub v: u64,
    pub r: U256,
    pub s: U256,

    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl EthereumTransaction {
    /// Create from raw RLP-encoded bytes
    pub fn from_raw_bytes(raw_bytes: &[u8]) -> Result<Self> {
        // For a production implementation, we would parse RLP here
        // This is a simplified version that handles the structure

        // Check if it's a typed transaction (EIP-2718)
        let is_typed = raw_bytes.first().map_or(false, |&b| b < 0x7f);

        if is_typed {
            // Parse typed transaction
            Self::parse_typed_transaction(raw_bytes)
        } else {
            // Parse legacy transaction
            Self::parse_legacy_transaction(raw_bytes)
        }
    }

    fn parse_typed_transaction(raw_bytes: &[u8]) -> Result<Self> {
        // Simplified parsing - in production, use proper RLP decoding

        // For now, create a minimal transaction structure
        Ok(Self {
            nonce: 0,
            gas_price: None,
            gas_limit: U256::from(21000),
            to: None,
            value: U256::zero(),
            data: vec![],
            chain_id: Some(U64::from(1)), // Mainnet
            max_fee_per_gas: Some(U256::from(1_000_000_000u64)),
            max_priority_fee_per_gas: Some(U256::from(1_000_000_000u64)),
            v: 0,
            r: U256::zero(),
            s: U256::zero(),
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn parse_legacy_transaction(raw_bytes: &[u8]) -> Result<Self> {
        // Simplified parsing - in production, use proper RLP decoding

        Ok(Self {
            nonce: 0,
            gas_price: Some(U256::from(20_000_000_000u64)), // 20 gwei
            gas_limit: U256::from(21000),
            to: None,
            value: U256::zero(),
            data: vec![],
            chain_id: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            v: 0,
            r: U256::zero(),
            s: U256::zero(),
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    /// Check if this is an EIP-1559 transaction
    pub fn is_eip1559(&self) -> bool {
        self.max_fee_per_gas.is_some() && self.max_priority_fee_per_gas.is_some()
    }

    /// Check if this is a contract creation
    pub fn is_contract_creation(&self) -> bool {
        self.to.is_none()
    }

    /// Get the sender address (would require signature recovery)
    pub fn sender(&self) -> Option<ethers_core::types::Address> {
        // In production, recover from signature
        None
    }

    /// Calculate transaction hash
    pub fn hash(&self) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        Keccak256::digest(&self.raw_bytes).to_vec()
    }
}

impl<'a> Canonicalizer<'a> for EthereumTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        // Build metadata
        let metadata = TxMetadata {
            tx_hash: self.hash(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: serde_json::json!({
                "nonce": self.nonce,
                "gas_limit": self.gas_limit.as_u64(),
                "gas_price": self.gas_price.map(|p| p.as_u64()),
                "max_fee_per_gas": self.max_fee_per_gas.map(|p| p.as_u64()),
                "max_priority_fee_per_gas": self.max_priority_fee_per_gas.map(|p| p.as_u64()),
                "is_eip1559": self.is_eip1559(),
                "chain_id": self.chain_id.map(|c| c.as_u64()),
            }),
        };

        // Build authorization package
        let signature = Signature {
            data: {
                let mut sig_bytes = Vec::new();
                // Encode r, s, v
                let mut r_bytes = [0u8; 32];
                self.r.to_big_endian(&mut r_bytes);
                let mut s_bytes = [0u8; 32];
                self.s.to_big_endian(&mut s_bytes);
                sig_bytes.extend_from_slice(&r_bytes);
                sig_bytes.extend_from_slice(&s_bytes);
                sig_bytes.push(self.v as u8);
                sig_bytes
            },
            key_index: 0,
            metadata: Some(serde_json::json!({
                "v": self.v,
            })),
        };

        let authorization = AuthorizationPackage {
            signatures: vec![signature],
            public_keys: vec![], // Would be recovered from signature
            signature_scheme: SignatureScheme::Ecdsa,
        };

        // Build operations
        let mut operations = Vec::new();

        if self.is_contract_creation() {
            // Contract deployment
            operations.push(Operation::ContractDeploy(ContractDeploy {
                bytecode: self.data.clone(),
                constructor_args: vec![],
                value: Amount {
                    value: self.value.as_u128(),
                    decimals: 18, // ETH has 18 decimals
                },
            }));
        } else if !self.data.is_empty() {
            // Contract call
            let method = if self.data.len() >= 4 {
                self.data[0..4].to_vec()
            } else {
                vec![]
            };

            operations.push(Operation::ContractCall(ContractCall {
                contract: Address {
                    bytes: self.to.map(|a| a.0.to_vec()).unwrap_or_default(),
                    human_readable: self.to.map(|a| format!("{:?}", a)),
                },
                method,
                data: self.data.clone(),
                value: Some(Amount {
                    value: self.value.as_u128(),
                    decimals: 18,
                }),
                resource_limits: ResourceLimits {
                    max_units: self.gas_limit.as_u64(),
                    unit_price: self
                        .gas_price
                        .or(self.max_fee_per_gas)
                        .unwrap_or(U256::zero())
                        .as_u64(),
                    resource_type: ResourceType::Gas,
                },
            }));
        } else {
            // Simple transfer
            operations.push(Operation::Transfer(Transfer {
                from: Address {
                    bytes: vec![],
                    human_readable: None,
                },
                to: Address {
                    bytes: self.to.map(|a| a.0.to_vec()).unwrap_or_default(),
                    human_readable: self.to.map(|a| format!("{:?}", a)),
                },
                amount: Amount {
                    value: self.value.as_u128(),
                    decimals: 18,
                },
                asset: AssetId::Native,
            }));
        }

        // Build state deltas (account model)
        let mut account_changes = vec![
            // Sender account change
            AccountChange {
                address: Address {
                    bytes: vec![],
                    human_readable: None,
                },
                nonce: Some(self.nonce),
                balance_change: -(self.value.as_u128() as i128),
                storage_changes: vec![],
            },
        ];

        if let Some(recipient) = self.to {
            // Recipient account change
            account_changes.push(AccountChange {
                address: Address {
                    bytes: recipient.0.to_vec(),
                    human_readable: Some(format!("{:?}", recipient)),
                },
                nonce: None,
                balance_change: self.value.as_u128() as i128,
                storage_changes: vec![],
            });
        }

        let state_deltas = StateDeltas {
            inputs: vec![],  // Ethereum uses account model, not UTXO
            outputs: vec![], // Ethereum uses account model, not UTXO
            account_changes,
        };

        Ok(TxIR::new(
            ChainId::Ethereum,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Check gas limit
        if self.gas_limit.is_zero() {
            return Err(DecoderError::invalid_structure(
                "Gas limit cannot be zero",
            ));
        }

        // Check that either gas_price or EIP-1559 fields are set
        if self.gas_price.is_none()
            && (self.max_fee_per_gas.is_none() || self.max_priority_fee_per_gas.is_none())
        {
            return Err(DecoderError::invalid_structure(
                "Must have either gas_price or EIP-1559 fee fields",
            ));
        }

        // Validate signature components
        if self.r.is_zero() || self.s.is_zero() {
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
        // Ethereum uses Keccak-256
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
    fn test_is_eip1559() {
        let tx = EthereumTransaction {
            nonce: 0,
            gas_price: None,
            gas_limit: U256::from(21000),
            to: None,
            value: U256::zero(),
            data: vec![],
            chain_id: Some(U64::from(1)),
            max_fee_per_gas: Some(U256::from(1_000_000_000u64)),
            max_priority_fee_per_gas: Some(U256::from(1_000_000_000u64)),
            v: 0,
            r: U256::zero(),
            s: U256::zero(),
            raw_bytes: vec![],
        };

        assert!(tx.is_eip1559());
    }

    #[test]
    fn test_is_contract_creation() {
        let tx = EthereumTransaction {
            nonce: 0,
            gas_price: Some(U256::from(20_000_000_000u64)),
            gas_limit: U256::from(21000),
            to: None,
            value: U256::zero(),
            data: vec![0x60, 0x80], // Contract bytecode
            chain_id: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            v: 0,
            r: U256::zero(),
            s: U256::zero(),
            raw_bytes: vec![],
        };

        assert!(tx.is_contract_creation());
    }
}
