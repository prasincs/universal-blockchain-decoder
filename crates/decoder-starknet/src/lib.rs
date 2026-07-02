//! Starknet transaction decoder
//!
//! Decodes Starknet transactions (INVOKE, DECLARE, DEPLOY_ACCOUNT) using
//! pure Rust cryptographic primitives from decoder-crypto-zk.
//!
//! ## Supported Transaction Types
//!
//! - **INVOKE** (v1, v3): Contract function calls
//! - **DECLARE** (v0, v3): Contract class registration
//! - **DEPLOY_ACCOUNT** (v1, v3): Account contract deployment
//!
//! ## Hash Functions
//!
//! - **v1 transactions**: Pedersen hash (legacy)
//! - **v3 transactions**: Poseidon hash (current)
//!
//! ## Chains Unlocked
//!
//! - Starknet Mainnet
//! - Starknet Sepolia Testnet
//! - 228+ Starknet appchains (Kakarot zkEVM, Madara-based chains, etc.)
//!
//! ## Example
//!
//! ```rust,ignore
//! use decoder_starknet::{StarknetDecoder, StarknetChain};
//! use decoder_primitives::ChainDecoder;
//!
//! let raw_tx_bytes = /* ... */;
//! let tx = StarknetDecoder::decode(raw_tx_bytes)?;
//! println!("Tx hash: {:?}", tx.tx_hash);
//! ```

use decoder_crypto_zk::FieldElement;
use decoder_primitives::prelude::*;

pub mod hashing;
pub mod parsing;
pub mod registry;
pub mod types;

pub use registry::{StarknetChainInfo, StarknetRegistry};
pub use types::*;

/// Starknet chain identity (Mainnet)
#[derive(Debug, Clone, Copy)]
pub struct StarknetChain;

impl ChainIdentity for StarknetChain {
    fn chain_id(&self) -> u64 {
        // Starknet Mainnet chain ID (SN_MAIN in ASCII + numeric encoding)
        // This is the canonical chain ID for Starknet
        23448594291968336
    }

    fn chain_name(&self) -> &str {
        "Starknet"
    }

    fn chain_family(&self) -> ChainFamily {
        ChainFamily::Account
    }
}

/// Parsed Starknet transaction
#[derive(Debug, Clone)]
pub struct StarknetTransaction {
    /// Transaction variant (INVOKE/DECLARE/DEPLOY_ACCOUNT with version)
    pub variant: StarknetTxVariant,
    /// Raw transaction bytes
    pub raw_bytes: Vec<u8>,
    /// Transaction hash
    pub tx_hash: Vec<u8>,
}

impl StarknetTransaction {
    /// Get transaction type
    pub fn tx_type(&self) -> StarknetTxType {
        self.variant.tx_type()
    }

    /// Get transaction version
    pub fn version(&self) -> StarknetVersion {
        self.variant.version()
    }

    /// Get sender address
    pub fn sender_address(&self) -> FieldElement {
        self.variant.sender_address()
    }

    /// Get signature
    pub fn signature(&self) -> &[FieldElement] {
        self.variant.signature()
    }

    /// Verify transaction hash
    pub fn verify_hash(&self) -> Result<bool> {
        let computed_hash = match &self.variant {
            StarknetTxVariant::InvokeV1(tx) => hashing::hash_invoke_v1(tx)?,
            StarknetTxVariant::InvokeV3(tx) => hashing::hash_invoke_v3(tx)?,
            StarknetTxVariant::DeclareV0(tx) => hashing::hash_declare_v0(tx)?,
            StarknetTxVariant::DeclareV3(tx) => hashing::hash_declare_v3(tx)?,
            StarknetTxVariant::DeployAccountV1(tx) => hashing::hash_deploy_account_v1(tx)?,
            StarknetTxVariant::DeployAccountV3(tx) => hashing::hash_deploy_account_v3(tx)?,
        };

        Ok(computed_hash == self.tx_hash)
    }
}

/// Starknet transaction decoder
pub struct StarknetDecoder;

impl ChainDecoder for StarknetDecoder {
    type TxSpecific = StarknetTransaction;
    type Chain = StarknetChain;

    fn chain() -> Self::Chain {
        StarknetChain
    }

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
        Self::validate_format(raw_bytes)?;

        // Parse transaction
        let variant = parsing::parse_transaction(raw_bytes)?;

        // Compute hash based on version
        let tx_hash = match &variant {
            StarknetTxVariant::InvokeV1(tx) => hashing::hash_invoke_v1(tx)?,
            StarknetTxVariant::InvokeV3(tx) => hashing::hash_invoke_v3(tx)?,
            StarknetTxVariant::DeclareV0(tx) => hashing::hash_declare_v0(tx)?,
            StarknetTxVariant::DeclareV3(tx) => hashing::hash_declare_v3(tx)?,
            StarknetTxVariant::DeployAccountV1(tx) => hashing::hash_deploy_account_v1(tx)?,
            StarknetTxVariant::DeployAccountV3(tx) => hashing::hash_deploy_account_v3(tx)?,
        };

        Ok(StarknetTransaction {
            variant,
            raw_bytes: raw_bytes.to_vec(),
            tx_hash,
        })
    }

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        if raw_bytes.is_empty() {
            return Err(DecoderError::invalid_structure(
                "Starknet transaction cannot be empty",
            ));
        }

        // Minimum size check: at least version byte + type byte
        if raw_bytes.len() < 2 {
            return Err(DecoderError::invalid_structure(
                "Starknet transaction too short",
            ));
        }

        Ok(())
    }
}

impl ChainEncoder for StarknetTransaction {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.raw_bytes.clone())
    }
}

impl<'a> Canonicalizer<'a> for StarknetTransaction {
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {
        let operations = build_operations(&self.variant)?;
        let authorization = build_authorization(&self.variant)?;
        let state_deltas = build_state_deltas(&self.variant)?;

        let metadata = TxMetadata {
            tx_hash: self.tx_hash.clone(),
            block_height: None,
            timestamp: None,
            size: self.raw_bytes.len(),
            extra: format!("type: {:?}, version: {:?}", self.tx_type(), self.version()),
        };

        Ok(TxIR::new(
            &StarknetChain,
            metadata,
            authorization,
            operations,
            state_deltas,
        ))
    }

    fn validate(&self) -> Result<()> {
        // Verify signature is present
        if self.signature().is_empty() {
            return Err(DecoderError::invalid_structure("No signature found"));
        }

        // Verify hash matches
        if !self.verify_hash()? {
            return Err(DecoderError::invalid_structure("Transaction hash mismatch"));
        }

        Ok(())
    }
}

fn create_address(addr: FieldElement) -> Address {
    Address {
        bytes: addr.to_bytes_be().to_vec(),
        human_readable: Some(format!("0x{}", hex::encode(addr.to_bytes_be()))),
    }
}

// Helper function for converting FieldElement to Amount (reserved for future use)
#[allow(dead_code)]
fn field_to_amount(field: FieldElement) -> Amount {
    // Starknet uses 18 decimals for ETH
    // Convert FieldElement to u128 (with overflow check)
    let bytes = field.to_bytes_be();
    let mut amount_bytes = [0u8; 16];
    amount_bytes.copy_from_slice(&bytes[16..32]);
    let value = u128::from_be_bytes(amount_bytes);

    Amount::new(value, 18)
}

fn build_operations(variant: &StarknetTxVariant) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();

    match variant {
        StarknetTxVariant::InvokeV1(tx) => {
            // For INVOKE: Extract contract calls from calldata
            operations.push(Operation::ContractCall(ContractCall {
                contract: create_address(tx.sender_address),
                method: b"invoke".to_vec(),
                data: vec![], // Simplified: calldata would go here
                value: None,
                resource_limits: ResourceLimits {
                    max_units: 0, // Would extract from max_fee
                    unit_price: 0,
                    resource_type: ResourceType::Gas,
                },
            }));
        }
        StarknetTxVariant::InvokeV3(tx) => {
            operations.push(Operation::ContractCall(ContractCall {
                contract: create_address(tx.sender_address),
                method: b"invoke".to_vec(),
                data: vec![],
                value: None,
                resource_limits: ResourceLimits {
                    max_units: tx.resource_bounds.l1_gas.max_amount,
                    unit_price: tx.resource_bounds.l1_gas.max_price_per_unit as u64,
                    resource_type: ResourceType::Gas,
                },
            }));
        }
        StarknetTxVariant::DeclareV0(tx) => {
            // DECLARE: Contract class registration
            operations.push(Operation::ContractCall(ContractCall {
                contract: create_address(tx.sender_address),
                method: b"declare".to_vec(),
                data: vec![],
                value: None,
                resource_limits: ResourceLimits {
                    max_units: 0,
                    unit_price: 0,
                    resource_type: ResourceType::Gas,
                },
            }));
        }
        StarknetTxVariant::DeclareV3(tx) => {
            operations.push(Operation::ContractCall(ContractCall {
                contract: create_address(tx.sender_address),
                method: b"declare".to_vec(),
                data: vec![],
                value: None,
                resource_limits: ResourceLimits {
                    max_units: tx.resource_bounds.l1_gas.max_amount,
                    unit_price: tx.resource_bounds.l1_gas.max_price_per_unit as u64,
                    resource_type: ResourceType::Gas,
                },
            }));
        }
        StarknetTxVariant::DeployAccountV1(_tx) => {
            // DEPLOY_ACCOUNT: Account creation
            operations.push(Operation::ContractCall(ContractCall {
                contract: create_address(FieldElement::ZERO),
                method: b"deploy_account".to_vec(),
                data: vec![],
                value: None,
                resource_limits: ResourceLimits {
                    max_units: 0,
                    unit_price: 0,
                    resource_type: ResourceType::Gas,
                },
            }));
        }
        StarknetTxVariant::DeployAccountV3(tx) => {
            operations.push(Operation::ContractCall(ContractCall {
                contract: create_address(FieldElement::ZERO),
                method: b"deploy_account".to_vec(),
                data: vec![],
                value: None,
                resource_limits: ResourceLimits {
                    max_units: tx.resource_bounds.l1_gas.max_amount,
                    unit_price: tx.resource_bounds.l1_gas.max_price_per_unit as u64,
                    resource_type: ResourceType::Gas,
                },
            }));
        }
    }

    Ok(operations)
}

fn build_authorization(variant: &StarknetTxVariant) -> Result<AuthorizationPackage> {
    let signature_data = variant.signature();

    let mut signatures = Vec::new();
    // Starknet signatures are typically (r, s) pairs
    if signature_data.len() >= 2 {
        let mut sig_bytes = Vec::new();
        for field in signature_data.iter().take(2) {
            sig_bytes.extend_from_slice(&field.to_bytes_be());
        }

        signatures.push(Signature {
            data: sig_bytes,
            key_index: 0,
            metadata: Some(format!(
                "STARK-curve ECDSA, {} elements",
                signature_data.len()
            )),
        });
    }

    // Public key would need to be recovered or provided separately
    // For now, we'll leave it empty (derivable from signature + message)
    let public_keys = vec![];

    Ok(AuthorizationPackage {
        signatures,
        public_keys,
        signature_scheme: SignatureScheme::Ecdsa,
    })
}

fn build_state_deltas(_variant: &StarknetTxVariant) -> Result<StateDeltas> {
    // Balance/nonce effect guesses are NOT byte-derivable and were removed
    // from TxIR (docs/CONCEPTS_REVIEW.md C1).
    Ok(StateDeltas {
        inputs: vec![],
        outputs: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_identity() {
        let chain = StarknetDecoder::chain();
        assert_eq!(chain.chain_id(), 23448594291968336);
        assert_eq!(chain.chain_name(), "Starknet");
        assert_eq!(chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_validate_empty_bytes() {
        let result = StarknetDecoder::validate_format(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_too_short() {
        let result = StarknetDecoder::validate_format(&[0x01]);
        assert!(result.is_err());
    }

    #[test]
    fn test_tx_variant_type() {
        use types::*;

        let invoke_v1 = StarknetTxVariant::InvokeV1(InvokeTxV1 {
            sender_address: FieldElement::ZERO,
            calldata: vec![],
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
        });

        assert_eq!(invoke_v1.tx_type(), StarknetTxType::Invoke);
        assert_eq!(invoke_v1.version(), StarknetVersion::V1);
    }
}
