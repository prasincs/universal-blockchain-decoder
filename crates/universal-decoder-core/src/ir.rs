//! Transaction Intermediate Representation (TxIR)
//!
//! This module defines the canonical intermediate representation for blockchain transactions.
//! The TxIR normalizes transactions from different blockchain models (UTXO, Account, Instruction)
//! into a unified semantic structure.

use crate::chain::{ChainIdentity, ChainRef};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Transaction version with const generic parameter for compile-time version enforcement
///
/// Using const generics allows different versions to be treated as different types,
/// ensuring version-specific logic cannot be accidentally mixed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxIR<'a, const V: u8> {
    /// Chain reference (trait-based, extensible)
    pub chain: ChainRef,

    /// Transaction metadata
    pub metadata: TxMetadata,

    /// Authorization information (signatures, public keys)
    pub authorization: AuthorizationPackage,

    /// Operations performed by this transaction
    pub operations: Vec<Operation>,

    /// Expected state deltas (inputs consumed / outputs created)
    pub state_deltas: StateDeltas,

    /// PhantomData to bind the IR to the source data lifetime
    /// This ensures proper lifetime tracking and influences Send/Sync traits
    _phantom: PhantomData<&'a [u8]>,
}

impl<'a, const V: u8> TxIR<'a, V> {
    /// Creates a new TxIR instance from a chain identity
    ///
    /// # Arguments
    ///
    /// * `chain` - A type implementing `ChainIdentity` to identify the blockchain
    /// * `metadata` - Transaction metadata
    /// * `authorization` - Authorization information (signatures, public keys)
    /// * `operations` - Operations performed by this transaction
    /// * `state_deltas` - Expected state changes
    ///
    /// # Example
    ///
    /// ```ignore
    /// use universal_decoder_core::prelude::*;
    ///
    /// let tx_ir = TxIR::new(
    ///     &BitcoinChain,
    ///     metadata,
    ///     authorization,
    ///     operations,
    ///     state_deltas,
    /// );
    /// ```
    pub fn new<C: ChainIdentity>(
        chain: &C,
        metadata: TxMetadata,
        authorization: AuthorizationPackage,
        operations: Vec<Operation>,
        state_deltas: StateDeltas,
    ) -> Self {
        Self {
            chain: ChainRef::from(chain),
            metadata,
            authorization,
            operations,
            state_deltas,
            _phantom: PhantomData,
        }
    }

    /// Gets the transaction version (encoded in the type)
    pub const fn version(&self) -> u8 {
        V
    }
}

/// Metadata about the transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxMetadata {
    /// Transaction hash/ID
    pub tx_hash: Vec<u8>,

    /// Block height (if known)
    pub block_height: Option<u64>,

    /// Timestamp (if available)
    pub timestamp: Option<u64>,

    /// Transaction size in bytes
    pub size: usize,

    /// Additional chain-specific metadata (JSON string)
    pub extra: String,
}

/// Authorization package containing signatures and public keys
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationPackage {
    /// List of signatures
    pub signatures: Vec<Signature>,

    /// List of public keys (for verification)
    pub public_keys: Vec<PublicKey>,

    /// Signature scheme used
    pub signature_scheme: SignatureScheme,
}

/// Signature data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// Raw signature bytes
    pub data: Vec<u8>,

    /// Index of the corresponding public key
    pub key_index: usize,

    /// Additional signature metadata (JSON string)
    pub metadata: Option<String>,
}

/// Public key data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    /// Raw public key bytes
    pub data: Vec<u8>,

    /// Key format/type
    pub key_type: KeyType,
}

/// Signature scheme used for transaction authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureScheme {
    Ecdsa,
    EdDsa,
    Schnorr,
    Custom(u32),
}

/// Public key type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    Secp256k1,
    Ed25519,
    P256,
    Custom(u32),
}

/// Operations performed by the transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Transfer of value
    Transfer(Transfer),

    /// Contract/Program call
    ContractCall(ContractCall),

    /// Contract deployment
    ContractDeploy(ContractDeploy),

    /// Stake operation
    Stake(Stake),

    /// Generic operation
    Generic(GenericOperation),
}

/// Transfer operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transfer {
    /// Source address/account
    pub from: Address,

    /// Destination address/account
    pub to: Address,

    /// Amount transferred
    pub amount: Amount,

    /// Asset/token identifier
    pub asset: AssetId,
}

/// Contract/Program call operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractCall {
    /// Contract address
    pub contract: Address,

    /// Function selector/method
    pub method: Vec<u8>,

    /// Call data/arguments
    pub data: Vec<u8>,

    /// Value sent with call (if applicable)
    pub value: Option<Amount>,

    /// Resource limits (gas, compute units, etc.)
    pub resource_limits: ResourceLimits,
}

/// Contract deployment operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractDeploy {
    /// Contract bytecode
    pub bytecode: Vec<u8>,

    /// Constructor arguments
    pub constructor_args: Vec<u8>,

    /// Initial value
    pub value: Amount,
}

/// Stake operation (for PoS chains)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stake {
    /// Validator address
    pub validator: Address,

    /// Amount to stake
    pub amount: Amount,

    /// Stake operation type
    pub operation_type: StakeOperationType,
}

/// Type of stake operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StakeOperationType {
    Delegate,
    Undelegate,
    Redelegate,
    Claim,
}

/// Generic operation for chain-specific actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericOperation {
    /// Operation type identifier
    pub op_type: String,

    /// Operation data
    pub data: Vec<u8>,

    /// Additional metadata (JSON string)
    pub metadata: String,
}

/// State deltas representing inputs consumed and outputs created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateDeltas {
    /// Inputs consumed (for UTXO model)
    pub inputs: Vec<InputReference>,

    /// Outputs created (for UTXO model)
    pub outputs: Vec<OutputValue>,

    /// Account state changes (for Account model)
    pub account_changes: Vec<AccountChange>,
}

/// Reference to an input (UTXO or account state)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputReference {
    /// Previous transaction hash
    pub prev_tx: Vec<u8>,

    /// Output index
    pub output_index: u32,

    /// Value consumed
    pub value: Amount,

    /// Script/conditions
    pub script: Vec<u8>,
}

/// Output value created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputValue {
    /// Output index
    pub index: u32,

    /// Recipient address
    pub address: Address,

    /// Value
    pub value: Amount,

    /// Script/conditions
    pub script: Vec<u8>,
}

/// Account state change (for account-based models)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountChange {
    /// Account address
    pub address: Address,

    /// Sequence number/nonce
    pub nonce: Option<u64>,

    /// Balance change
    pub balance_change: i128,

    /// Storage changes
    pub storage_changes: Vec<StorageChange>,
}

/// Storage change in an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageChange {
    /// Storage key
    pub key: Vec<u8>,

    /// New value (None for deletion)
    pub value: Option<Vec<u8>>,
}

/// Address representation (normalized across chains)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    /// Raw address bytes
    pub bytes: Vec<u8>,

    /// Human-readable representation
    pub human_readable: Option<String>,
}

/// Amount representation (with support for large numbers)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    /// Value in smallest unit
    pub value: u128,

    /// Decimal places (for display)
    pub decimals: u8,
}

/// Asset/Token identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetId {
    Native,
    Token(Vec<u8>),
    Custom(String),
}

/// Resource limits (gas, compute units, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum units
    pub max_units: u64,

    /// Unit price
    pub unit_price: u64,

    /// Resource type
    pub resource_type: ResourceType,
}

/// Type of computational resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Gas,
    ComputeUnits,
    Weight,
    Custom(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{ChainFamily, ChainIdentity};

    #[derive(Debug)]
    struct TestChain;

    impl ChainIdentity for TestChain {
        fn chain_id(&self) -> u64 {
            0
        }
        fn chain_name(&self) -> &str {
            "Bitcoin"
        }
        fn chain_family(&self) -> ChainFamily {
            ChainFamily::Utxo
        }
    }

    #[test]
    fn test_txir_version_const_generic() {
        let chain = TestChain;
        let tx_v1 = TxIR::<1>::new(
            &chain,
            TxMetadata {
                tx_hash: vec![0; 32],
                block_height: Some(800000),
                timestamp: Some(1699999999),
                size: 250,
                extra: "{}".to_string(),
            },
            AuthorizationPackage {
                signatures: vec![],
                public_keys: vec![],
                signature_scheme: SignatureScheme::Ecdsa,
            },
            vec![],
            StateDeltas {
                inputs: vec![],
                outputs: vec![],
                account_changes: vec![],
            },
        );

        assert_eq!(tx_v1.version(), 1);
    }

    #[test]
    fn test_address_creation() {
        let addr = Address {
            bytes: vec![1, 2, 3, 4],
            human_readable: Some("0x01020304".to_string()),
        };

        assert_eq!(addr.bytes.len(), 4);
        assert!(addr.human_readable.is_some());
    }

    #[test]
    fn test_amount_representation() {
        let amount = Amount {
            value: 1_000_000_000, // 1 BTC in satoshis
            decimals: 8,
        };

        assert_eq!(amount.value, 1_000_000_000);
        assert_eq!(amount.decimals, 8);
    }
}
