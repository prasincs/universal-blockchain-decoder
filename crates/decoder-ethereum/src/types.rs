//! Ethereum-specific transaction types
//!
//! NOTE: This is a placeholder implementation for Phase 1.5.
//! Phase 2 will implement pure Rust RLP parsing using alloy-rs.

use universal_decoder_core::prelude::*;

use crate::EthereumChain;

/// Ethereum-specific transaction representation
///
/// Supports different transaction types (Legacy, EIP-2930, EIP-1559)
#[derive(Debug, Clone)]
pub struct EthereumTransaction {
    /// Transaction data
    pub nonce: u64,
    pub gas_price: Option<u128>,
    pub gas_limit: u128,
    pub to: Option<[u8; 20]>, // Ethereum address (20 bytes)
    pub value: u128,
    pub data: Vec<u8>,
    pub chain_id: Option<u64>,

    // EIP-1559 fields
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,

    // Signature
    pub v: u64,
    pub r: [u8; 32], // 256-bit signature component
    pub s: [u8; 32], // 256-bit signature component

    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
}

impl EthereumTransaction {
    /// Create from raw RLP-encoded bytes
    pub fn from_raw_bytes(raw_bytes: &[u8]) -> Result<Self> {
        // For a production implementation, we would parse RLP here
        // This is a simplified version that handles the structure

        // Check if it's a typed transaction (EIP-2718)
        let is_typed = raw_bytes.first().is_some_and(|&b| b < 0x7f);

        if is_typed {
            // Parse typed transaction
            Self::parse_typed_transaction(raw_bytes)
        } else {
            // Parse legacy transaction
            Self::parse_legacy_transaction(raw_bytes)
        }
    }

    fn parse_typed_transaction(raw_bytes: &[u8]) -> Result<Self> {
        // Simplified parsing - in production, use proper RLP decoding with alloy-rs

        // For now, create a minimal transaction structure
        Ok(Self {
            nonce: 0,
            gas_price: None,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: vec![],
            chain_id: Some(1), // Mainnet
            max_fee_per_gas: Some(1_000_000_000),
            max_priority_fee_per_gas: Some(1_000_000_000),
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
            raw_bytes: raw_bytes.to_vec(),
        })
    }

    fn parse_legacy_transaction(raw_bytes: &[u8]) -> Result<Self> {
        // Simplified parsing - in production, use proper RLP decoding with alloy-rs

        Ok(Self {
            nonce: 0,
            gas_price: Some(20_000_000_000), // 20 gwei
            gas_limit: 21000,
            to: None,
            value: 0,
            data: vec![],
            chain_id: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
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
    pub fn sender(&self) -> Option<[u8; 20]> {
        // In production, recover from signature using ECDSA recovery
        // This would use alloy-rs in Phase 2
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
        let extra = format!(
            r#"{{"nonce":{},"gas_limit":{},"gas_price":{},"max_fee_per_gas":{},"max_priority_fee_per_gas":{},"is_eip1559":{},"chain_id":{}}}"#,
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
            self.is_eip1559(),
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
                // Encode r, s, v
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
                    value: self.value,
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
                    bytes: self.to.map(|a| a.to_vec()).unwrap_or_default(),
                    human_readable: self.to.map(|a| format!("{:?}", a)),
                },
                method,
                data: self.data.clone(),
                value: Some(Amount {
                    value: self.value,
                    decimals: 18,
                }),
                resource_limits: ResourceLimits {
                    max_units: self.gas_limit as u64,
                    unit_price: self.gas_price.or(self.max_fee_per_gas).unwrap_or(0) as u64,
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
                    bytes: self.to.map(|a| a.to_vec()).unwrap_or_default(),
                    human_readable: self.to.map(|a| format!("{:?}", a)),
                },
                amount: Amount {
                    value: self.value,
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
                balance_change: -(self.value as i128),
                storage_changes: vec![],
            },
        ];

        if let Some(recipient) = self.to {
            // Recipient account change
            account_changes.push(AccountChange {
                address: Address {
                    bytes: recipient.to_vec(),
                    human_readable: Some(format!("{:?}", recipient)),
                },
                nonce: None,
                balance_change: self.value as i128,
                storage_changes: vec![],
            });
        }

        let state_deltas = StateDeltas {
            inputs: vec![],  // Ethereum uses account model, not UTXO
            outputs: vec![], // Ethereum uses account model, not UTXO
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
        // Check gas limit
        if self.gas_limit == 0 {
            return Err(DecoderError::invalid_structure("Gas limit cannot be zero"));
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
            gas_limit: 21000,
            to: None,
            value: 0,
            data: vec![],
            chain_id: Some(1),
            max_fee_per_gas: Some(1_000_000_000),
            max_priority_fee_per_gas: Some(1_000_000_000),
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
            raw_bytes: vec![],
        };

        assert!(tx.is_eip1559());
    }

    #[test]
    fn test_is_contract_creation() {
        let tx = EthereumTransaction {
            nonce: 0,
            gas_price: Some(20_000_000_000),
            gas_limit: 21000,
            to: None,
            value: 0,
            data: vec![0x60, 0x80], // Contract bytecode
            chain_id: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
            raw_bytes: vec![],
        };

        assert!(tx.is_contract_creation());
    }
}
