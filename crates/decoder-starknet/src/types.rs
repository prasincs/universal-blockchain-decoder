//! Starknet transaction types
//!
//! Supports three main transaction types:
//! - INVOKE (v1, v3): Contract function calls
//! - DECLARE (v0, v3): Contract class registration
//! - DEPLOY_ACCOUNT (v1, v3): Account contract deployment

use decoder_crypto_zk::FieldElement;
use serde::{Deserialize, Serialize};

/// Starknet transaction type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarknetTxType {
    /// Contract function invocation
    Invoke,
    /// Contract class declaration
    Declare,
    /// Account contract deployment
    DeployAccount,
    /// L1 handler (messages from Ethereum)
    L1Handler,
}

/// Starknet transaction version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarknetVersion {
    /// Legacy version (Pedersen hash)
    V0,
    /// Standard version (Pedersen hash)
    V1,
    /// Current version (Poseidon hash, EIP-1559 fees)
    V3,
}

/// INVOKE transaction (v1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeTxV1 {
    /// Contract address being invoked
    pub sender_address: FieldElement,
    /// Function calldata
    pub calldata: Vec<FieldElement>,
    /// Maximum fee willing to pay
    pub max_fee: FieldElement,
    /// Transaction signature
    pub signature: Vec<FieldElement>,
    /// Transaction nonce
    pub nonce: FieldElement,
}

/// INVOKE transaction (v3 - with EIP-1559 fees)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeTxV3 {
    /// Contract address being invoked
    pub sender_address: FieldElement,
    /// Function calldata
    pub calldata: Vec<FieldElement>,
    /// Transaction signature
    pub signature: Vec<FieldElement>,
    /// Transaction nonce
    pub nonce: FieldElement,
    /// Resource bounds (gas limits)
    pub resource_bounds: ResourceBounds,
    /// Tip for priority
    pub tip: u64,
    /// Paymaster data (if using paymaster)
    pub paymaster_data: Vec<FieldElement>,
    /// Account deployment data (if deploying account)
    pub account_deployment_data: Vec<FieldElement>,
    /// Fee data availability mode
    pub nonce_data_availability_mode: DataAvailabilityMode,
    /// Fee data availability mode
    pub fee_data_availability_mode: DataAvailabilityMode,
}

/// DECLARE transaction (v0 - legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareTxV0 {
    /// Class hash of contract being declared
    pub class_hash: FieldElement,
    /// Sender address
    pub sender_address: FieldElement,
    /// Maximum fee
    pub max_fee: FieldElement,
    /// Transaction signature
    pub signature: Vec<FieldElement>,
}

/// DECLARE transaction (v3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareTxV3 {
    /// Class hash of contract being declared
    pub class_hash: FieldElement,
    /// Compiled class hash
    pub compiled_class_hash: FieldElement,
    /// Sender address
    pub sender_address: FieldElement,
    /// Transaction signature
    pub signature: Vec<FieldElement>,
    /// Transaction nonce
    pub nonce: FieldElement,
    /// Resource bounds
    pub resource_bounds: ResourceBounds,
    /// Tip for priority
    pub tip: u64,
    /// Paymaster data
    pub paymaster_data: Vec<FieldElement>,
    /// Account deployment data
    pub account_deployment_data: Vec<FieldElement>,
    /// Data availability modes
    pub nonce_data_availability_mode: DataAvailabilityMode,
    pub fee_data_availability_mode: DataAvailabilityMode,
}

/// DEPLOY_ACCOUNT transaction (v1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployAccountTxV1 {
    /// Class hash of account contract
    pub class_hash: FieldElement,
    /// Constructor calldata
    pub constructor_calldata: Vec<FieldElement>,
    /// Contract address salt
    pub contract_address_salt: FieldElement,
    /// Maximum fee
    pub max_fee: FieldElement,
    /// Transaction signature
    pub signature: Vec<FieldElement>,
    /// Transaction nonce (usually 0 for deployment)
    pub nonce: FieldElement,
}

/// DEPLOY_ACCOUNT transaction (v3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployAccountTxV3 {
    /// Class hash of account contract
    pub class_hash: FieldElement,
    /// Constructor calldata
    pub constructor_calldata: Vec<FieldElement>,
    /// Contract address salt
    pub contract_address_salt: FieldElement,
    /// Transaction signature
    pub signature: Vec<FieldElement>,
    /// Transaction nonce
    pub nonce: FieldElement,
    /// Resource bounds
    pub resource_bounds: ResourceBounds,
    /// Tip for priority
    pub tip: u64,
    /// Paymaster data
    pub paymaster_data: Vec<FieldElement>,
    /// Data availability modes
    pub nonce_data_availability_mode: DataAvailabilityMode,
    pub fee_data_availability_mode: DataAvailabilityMode,
}

/// Resource bounds for v3 transactions (EIP-1559 style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBounds {
    /// L1 gas bounds
    pub l1_gas: ResourceBound,
    /// L2 gas bounds
    pub l2_gas: ResourceBound,
}

/// Single resource bound
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBound {
    /// Maximum amount of resource
    pub max_amount: u64,
    /// Maximum price per unit
    pub max_price_per_unit: u128,
}

/// Data availability mode (v3 transactions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataAvailabilityMode {
    /// On-chain data availability (L1)
    L1 = 0,
    /// Off-chain data availability (L2/DA layer)
    L2 = 1,
}

/// Unified Starknet transaction envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StarknetTxVariant {
    InvokeV1(InvokeTxV1),
    InvokeV3(InvokeTxV3),
    DeclareV0(DeclareTxV0),
    DeclareV3(DeclareTxV3),
    DeployAccountV1(DeployAccountTxV1),
    DeployAccountV3(DeployAccountTxV3),
}

impl StarknetTxVariant {
    /// Get transaction type
    pub fn tx_type(&self) -> StarknetTxType {
        match self {
            StarknetTxVariant::InvokeV1(_) | StarknetTxVariant::InvokeV3(_) => {
                StarknetTxType::Invoke
            }
            StarknetTxVariant::DeclareV0(_) | StarknetTxVariant::DeclareV3(_) => {
                StarknetTxType::Declare
            }
            StarknetTxVariant::DeployAccountV1(_) | StarknetTxVariant::DeployAccountV3(_) => {
                StarknetTxType::DeployAccount
            }
        }
    }

    /// Get transaction version
    pub fn version(&self) -> StarknetVersion {
        match self {
            StarknetTxVariant::InvokeV1(_) => StarknetVersion::V1,
            StarknetTxVariant::InvokeV3(_) => StarknetVersion::V3,
            StarknetTxVariant::DeclareV0(_) => StarknetVersion::V0,
            StarknetTxVariant::DeclareV3(_) => StarknetVersion::V3,
            StarknetTxVariant::DeployAccountV1(_) => StarknetVersion::V1,
            StarknetTxVariant::DeployAccountV3(_) => StarknetVersion::V3,
        }
    }

    /// Get sender address
    pub fn sender_address(&self) -> FieldElement {
        match self {
            StarknetTxVariant::InvokeV1(tx) => tx.sender_address,
            StarknetTxVariant::InvokeV3(tx) => tx.sender_address,
            StarknetTxVariant::DeclareV0(tx) => tx.sender_address,
            StarknetTxVariant::DeclareV3(tx) => tx.sender_address,
            StarknetTxVariant::DeployAccountV1(_) => {
                // For deploy_account, sender is computed from constructor args
                FieldElement::ZERO
            }
            StarknetTxVariant::DeployAccountV3(_) => FieldElement::ZERO,
        }
    }

    /// Get signature
    pub fn signature(&self) -> &[FieldElement] {
        match self {
            StarknetTxVariant::InvokeV1(tx) => &tx.signature,
            StarknetTxVariant::InvokeV3(tx) => &tx.signature,
            StarknetTxVariant::DeclareV0(tx) => &tx.signature,
            StarknetTxVariant::DeclareV3(tx) => &tx.signature,
            StarknetTxVariant::DeployAccountV1(tx) => &tx.signature,
            StarknetTxVariant::DeployAccountV3(tx) => &tx.signature,
        }
    }
}
