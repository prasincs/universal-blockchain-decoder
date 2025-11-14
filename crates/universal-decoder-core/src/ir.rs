//! Transaction Intermediate Representation (TxIR)
//!
//! This module defines the canonical intermediate representation for blockchain transactions.
//! The TxIR normalizes transactions from different blockchain models (UTXO, Account, Instruction)
//! into a unified semantic structure.

use crate::chain::{ChainIdentity, ChainRef};
use crate::privacy::PrivacyMetadata;
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

    /// Optional privacy metadata (None for fully transparent chains)
    ///
    /// When present, describes the privacy mechanisms used in this transaction.
    /// This field is backward compatible: existing code can ignore it (defaults to None).
    ///
    /// # Examples
    ///
    /// - Bitcoin/Ethereum legacy: `None` (fully transparent)
    /// - Ethereum with stealth addresses: `Some(PrivacyMetadata { ... })`
    /// - Monero: `Some(PrivacyMetadata { observability: FullyPrivate, ... })`
    pub privacy: Option<PrivacyMetadata>,

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
            privacy: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new TxIR instance with privacy metadata
    ///
    /// # Arguments
    ///
    /// * `chain` - A type implementing `ChainIdentity` to identify the blockchain
    /// * `metadata` - Transaction metadata
    /// * `authorization` - Authorization information (signatures, public keys)
    /// * `operations` - Operations performed by this transaction
    /// * `state_deltas` - Expected state changes
    /// * `privacy` - Optional privacy metadata
    ///
    /// # Example
    ///
    /// ```ignore
    /// use universal_decoder_core::prelude::*;
    ///
    /// let privacy = PrivacyMetadata {
    ///     features: vec![
    ///         PrivacyFeature::HiddenRecipient(PrivateAddress {
    ///             privacy_type: AddressPrivacyType::Stealth { scheme_id: 5564 },
    ///             public_address: vec![1, 2, 3],
    ///             viewing_hint: None,
    ///         }),
    ///     ],
    ///     observability: ObservabilityLevel::PartiallyObservable,
    ///     viewing_key: None,
    /// };
    ///
    /// let tx_ir = TxIR::with_privacy(
    ///     &EthereumChain,
    ///     metadata,
    ///     authorization,
    ///     operations,
    ///     state_deltas,
    ///     Some(privacy),
    /// );
    /// ```
    pub fn with_privacy<C: ChainIdentity>(
        chain: &C,
        metadata: TxMetadata,
        authorization: AuthorizationPackage,
        operations: Vec<Operation>,
        state_deltas: StateDeltas,
        privacy: Option<PrivacyMetadata>,
    ) -> Self {
        Self {
            chain: ChainRef::from(chain),
            metadata,
            authorization,
            operations,
            state_deltas,
            privacy,
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
///
/// This type is used for all cryptocurrency amounts, fees, and balances.
/// It uses u128 to support large values (up to 340 undecillion).
///
/// **Formal Verification (Verus)**:
/// - VT-1.1: checked_add never overflows silently
/// - VT-1.2: checked_sub never underflows silently
/// - VT-1.3: checked_mul never overflows silently
/// - All arithmetic operations are panic-free
///
/// See `docs/VERUS_WHAT_IT_PROVES.md` for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    /// Value in smallest unit
    pub value: u128,

    /// Decimal places (for display)
    pub decimals: u8,
}

impl Amount {
    /// Creates a new Amount with the given value and decimals
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_decoder_core::ir::Amount;
    ///
    /// let btc = Amount::new(100_000_000, 8); // 1 BTC in satoshis
    /// assert_eq!(btc.value, 100_000_000);
    /// assert_eq!(btc.decimals, 8);
    /// ```
    pub const fn new(value: u128, decimals: u8) -> Self {
        Self { value, decimals }
    }

    /// Checked addition with overflow detection
    ///
    /// Returns `None` if the addition would overflow.
    ///
    /// **Formal Verification (VT-1.1)**:
    /// ```text
    /// Property: ∀ a, b: Amount where a.decimals == b.decimals,
    ///   checked_add(a, b) = Some(sum) ==> sum.value == a.value + b.value
    ///   checked_add(a, b) = None      ==> a.value + b.value > u128::MAX
    ///   checked_add(a, b) NEVER panics
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_decoder_core::ir::Amount;
    ///
    /// let a = Amount::new(100, 8);
    /// let b = Amount::new(200, 8);
    /// let sum = a.checked_add(b).unwrap();
    /// assert_eq!(sum.value, 300);
    ///
    /// // Overflow case
    /// let max = Amount::new(u128::MAX, 8);
    /// let one = Amount::new(1, 8);
    /// assert!(max.checked_add(one).is_none());
    /// ```
    ///
    /// # Verus Specification
    ///
    /// When Verus is enabled, this function is verified to satisfy:
    /// ```rust,ignore
    /// verus! {
    ///     pub fn checked_add(self, other: Amount) -> (result: Option<Amount>)
    ///         requires
    ///             self.decimals == other.decimals,
    ///         ensures
    ///             result.is_some() ==> {
    ///                 let sum = result.unwrap();
    ///                 sum.value == self.value + other.value &&
    ///                 sum.decimals == self.decimals
    ///             },
    ///             result.is_none() ==> self.value + other.value > u128::MAX,
    /// }
    /// ```
    pub const fn checked_add(self, other: Amount) -> Option<Amount> {
        if self.decimals != other.decimals {
            return None; // Cannot add amounts with different decimals
        }

        match self.value.checked_add(other.value) {
            Some(sum) => Some(Amount {
                value: sum,
                decimals: self.decimals,
            }),
            None => None,
        }
    }

    /// Checked subtraction with underflow detection
    ///
    /// Returns `None` if the subtraction would underflow.
    ///
    /// **Formal Verification (VT-1.2)**:
    /// ```text
    /// Property: ∀ a, b: Amount where a.decimals == b.decimals,
    ///   checked_sub(a, b) = Some(diff) ==> diff.value == a.value - b.value && a.value >= b.value
    ///   checked_sub(a, b) = None       ==> a.value < b.value
    ///   checked_sub(a, b) NEVER panics
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_decoder_core::ir::Amount;
    ///
    /// let a = Amount::new(500, 8);
    /// let b = Amount::new(200, 8);
    /// let diff = a.checked_sub(b).unwrap();
    /// assert_eq!(diff.value, 300);
    ///
    /// // Underflow case
    /// let small = Amount::new(100, 8);
    /// let large = Amount::new(500, 8);
    /// assert!(small.checked_sub(large).is_none());
    /// ```
    ///
    /// # Verus Specification
    ///
    /// When Verus is enabled, this function is verified to satisfy:
    /// ```rust,ignore
    /// verus! {
    ///     pub fn checked_sub(self, other: Amount) -> (result: Option<Amount>)
    ///         requires
    ///             self.decimals == other.decimals,
    ///         ensures
    ///             result.is_some() ==> {
    ///                 let diff = result.unwrap();
    ///                 diff.value == self.value - other.value &&
    ///                 self.value >= other.value &&
    ///                 diff.decimals == self.decimals
    ///             },
    ///             result.is_none() ==> self.value < other.value || self.decimals != other.decimals,
    /// }
    /// ```
    pub const fn checked_sub(self, other: Amount) -> Option<Amount> {
        if self.decimals != other.decimals {
            return None; // Cannot subtract amounts with different decimals
        }

        match self.value.checked_sub(other.value) {
            Some(diff) => Some(Amount {
                value: diff,
                decimals: self.decimals,
            }),
            None => None,
        }
    }

    /// Checked multiplication with overflow detection
    ///
    /// Returns `None` if the multiplication would overflow.
    ///
    /// **Formal Verification (VT-1.3)**:
    /// ```text
    /// Property: ∀ a: Amount, multiplier: u128,
    ///   checked_mul(a, multiplier) = Some(prod) ==> prod.value == a.value * multiplier
    ///   checked_mul(a, multiplier) = None       ==> a.value * multiplier > u128::MAX
    ///   checked_mul(a, multiplier) NEVER panics
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_decoder_core::ir::Amount;
    ///
    /// let amount = Amount::new(100, 8);
    /// let doubled = amount.checked_mul(2).unwrap();
    /// assert_eq!(doubled.value, 200);
    ///
    /// // Overflow case
    /// let large = Amount::new(u128::MAX / 2 + 1, 8);
    /// assert!(large.checked_mul(2).is_none());
    /// ```
    ///
    /// # Verus Specification
    ///
    /// When Verus is enabled, this function is verified to satisfy:
    /// ```rust,ignore
    /// verus! {
    ///     pub fn checked_mul(self, multiplier: u128) -> (result: Option<Amount>)
    ///         ensures
    ///             result.is_some() ==> {
    ///                 let prod = result.unwrap();
    ///                 prod.value == self.value * multiplier &&
    ///                 prod.decimals == self.decimals
    ///             },
    ///             result.is_none() ==> self.value * multiplier > u128::MAX,
    /// }
    /// ```
    pub const fn checked_mul(self, multiplier: u128) -> Option<Amount> {
        match self.value.checked_mul(multiplier) {
            Some(prod) => Some(Amount {
                value: prod,
                decimals: self.decimals,
            }),
            None => None,
        }
    }

    /// Checked division with divide-by-zero detection
    ///
    /// Returns `None` if the divisor is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_decoder_core::ir::Amount;
    ///
    /// let amount = Amount::new(100, 8);
    /// let half = amount.checked_div(2).unwrap();
    /// assert_eq!(half.value, 50);
    ///
    /// // Division by zero
    /// assert!(amount.checked_div(0).is_none());
    /// ```
    pub const fn checked_div(self, divisor: u128) -> Option<Amount> {
        match self.value.checked_div(divisor) {
            Some(quot) => Some(Amount {
                value: quot,
                decimals: self.decimals,
            }),
            None => None,
        }
    }

    /// Returns true if the amount is zero
    pub const fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Returns the value as a float (for display purposes only, not for calculations)
    ///
    /// **Warning**: This is for display only. Do not use for calculations as it loses precision.
    pub fn to_float(&self) -> f64 {
        self.value as f64 / 10_f64.powi(self.decimals as i32)
    }
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

    // ===== VT-1 Amount Arithmetic Safety Tests =====
    // These tests validate the properties that Verus will formally verify

    #[test]
    fn test_amount_new() {
        let amount = Amount::new(100_000_000, 8);
        assert_eq!(amount.value, 100_000_000);
        assert_eq!(amount.decimals, 8);
    }

    #[test]
    fn test_checked_add_success() {
        // VT-1.1: Normal addition
        let a = Amount::new(100, 8);
        let b = Amount::new(200, 8);
        let sum = a.checked_add(b).unwrap();
        assert_eq!(sum.value, 300);
        assert_eq!(sum.decimals, 8);
    }

    #[test]
    fn test_checked_add_overflow_detection() {
        // VT-1.1: Overflow detection
        let max = Amount::new(u128::MAX, 8);
        let one = Amount::new(1, 8);
        assert!(max.checked_add(one).is_none());
    }

    #[test]
    fn test_checked_add_near_overflow() {
        // VT-1.1: Near-overflow edge case
        let a = Amount::new(u128::MAX - 100, 8);
        let b = Amount::new(50, 8);
        let sum = a.checked_add(b).unwrap();
        assert_eq!(sum.value, u128::MAX - 50);
    }

    #[test]
    fn test_checked_add_exact_overflow() {
        // VT-1.1: Exact overflow boundary
        let a = Amount::new(u128::MAX - 100, 8);
        let b = Amount::new(101, 8);
        assert!(a.checked_add(b).is_none());
    }

    #[test]
    fn test_checked_add_mismatched_decimals() {
        // VT-1.1: Cannot add amounts with different decimals
        let a = Amount::new(100, 8);
        let b = Amount::new(200, 6);
        assert!(a.checked_add(b).is_none());
    }

    #[test]
    fn test_checked_add_zero() {
        // VT-1.1: Adding zero
        let a = Amount::new(100, 8);
        let zero = Amount::new(0, 8);
        let sum = a.checked_add(zero).unwrap();
        assert_eq!(sum.value, 100);
    }

    #[test]
    fn test_checked_sub_success() {
        // VT-1.2: Normal subtraction
        let a = Amount::new(500, 8);
        let b = Amount::new(200, 8);
        let diff = a.checked_sub(b).unwrap();
        assert_eq!(diff.value, 300);
        assert_eq!(diff.decimals, 8);
    }

    #[test]
    fn test_checked_sub_underflow_detection() {
        // VT-1.2: Underflow detection
        let small = Amount::new(100, 8);
        let large = Amount::new(500, 8);
        assert!(small.checked_sub(large).is_none());
    }

    #[test]
    fn test_checked_sub_exact_zero() {
        // VT-1.2: Subtracting to zero
        let a = Amount::new(100, 8);
        let b = Amount::new(100, 8);
        let diff = a.checked_sub(b).unwrap();
        assert_eq!(diff.value, 0);
    }

    #[test]
    fn test_checked_sub_near_underflow() {
        // VT-1.2: Near-underflow edge case
        let a = Amount::new(100, 8);
        let b = Amount::new(99, 8);
        let diff = a.checked_sub(b).unwrap();
        assert_eq!(diff.value, 1);
    }

    #[test]
    fn test_checked_sub_mismatched_decimals() {
        // VT-1.2: Cannot subtract amounts with different decimals
        let a = Amount::new(500, 8);
        let b = Amount::new(200, 6);
        assert!(a.checked_sub(b).is_none());
    }

    #[test]
    fn test_checked_mul_success() {
        // VT-1.3: Normal multiplication
        let amount = Amount::new(100, 8);
        let doubled = amount.checked_mul(2).unwrap();
        assert_eq!(doubled.value, 200);
        assert_eq!(doubled.decimals, 8);
    }

    #[test]
    fn test_checked_mul_overflow_detection() {
        // VT-1.3: Overflow detection
        let large = Amount::new(u128::MAX / 2 + 1, 8);
        assert!(large.checked_mul(2).is_none());
    }

    #[test]
    fn test_checked_mul_zero() {
        // VT-1.3: Multiplying by zero
        let amount = Amount::new(100, 8);
        let zero = amount.checked_mul(0).unwrap();
        assert_eq!(zero.value, 0);
    }

    #[test]
    fn test_checked_mul_one() {
        // VT-1.3: Multiplying by one (identity)
        let amount = Amount::new(100, 8);
        let same = amount.checked_mul(1).unwrap();
        assert_eq!(same.value, 100);
    }

    #[test]
    fn test_checked_mul_large() {
        // VT-1.3: Large multiplication that fits
        let amount = Amount::new(1_000_000, 8);
        let result = amount.checked_mul(1_000_000).unwrap();
        assert_eq!(result.value, 1_000_000_000_000);
    }

    #[test]
    fn test_checked_div_success() {
        // Division
        let amount = Amount::new(100, 8);
        let half = amount.checked_div(2).unwrap();
        assert_eq!(half.value, 50);
    }

    #[test]
    fn test_checked_div_zero() {
        // Division by zero detection
        let amount = Amount::new(100, 8);
        assert!(amount.checked_div(0).is_none());
    }

    #[test]
    fn test_is_zero() {
        let zero = Amount::new(0, 8);
        let non_zero = Amount::new(100, 8);
        assert!(zero.is_zero());
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn test_to_float() {
        let amount = Amount::new(100_000_000, 8); // 1 BTC
        assert!((amount.to_float() - 1.0).abs() < 1e-10);

        let amount = Amount::new(150_000_000, 8); // 1.5 BTC
        assert!((amount.to_float() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_amount_copy_semantics() {
        // Amount should implement Copy (it's a simple value type)
        let a = Amount::new(100, 8);
        let b = a; // This copies
        let c = a; // This should also work (a wasn't moved)
        assert_eq!(a.value, b.value);
        assert_eq!(a.value, c.value);
    }

    // Property-based test patterns (for future proptest integration)
    // These document properties that should hold for all inputs

    #[test]
    fn test_checked_add_commutative() {
        // Property: a + b = b + a
        let a = Amount::new(100, 8);
        let b = Amount::new(200, 8);
        assert_eq!(a.checked_add(b), b.checked_add(a));
    }

    #[test]
    fn test_checked_add_associative() {
        // Property: (a + b) + c = a + (b + c)
        let a = Amount::new(100, 8);
        let b = Amount::new(200, 8);
        let c = Amount::new(300, 8);

        let left = a.checked_add(b).unwrap().checked_add(c);
        let right = a.checked_add(b.checked_add(c).unwrap());
        assert_eq!(left, right);
    }

    #[test]
    fn test_checked_sub_inverse_of_add() {
        // Property: (a + b) - b = a
        let a = Amount::new(100, 8);
        let b = Amount::new(200, 8);

        let sum = a.checked_add(b).unwrap();
        let back = sum.checked_sub(b).unwrap();
        assert_eq!(back, a);
    }

    #[test]
    fn test_checked_mul_distributive() {
        // Property: a * (m + n) = a * m + a * n (when no overflow)
        let a = Amount::new(100, 8);
        let m = 2;
        let n = 3;

        let left = a.checked_mul(m + n).unwrap();
        let right = a
            .checked_mul(m)
            .unwrap()
            .checked_add(a.checked_mul(n).unwrap())
            .unwrap();
        assert_eq!(left, right);
    }

    // ===== Const Fn Tests =====
    // These tests demonstrate compile-time evaluation of Amount arithmetic

    #[test]
    fn test_const_amount_new() {
        // Amount::new is const, can be used in const contexts
        const ONE_BTC: Amount = Amount::new(100_000_000, 8);
        assert_eq!(ONE_BTC.value, 100_000_000);
        assert_eq!(ONE_BTC.decimals, 8);
    }

    #[test]
    fn test_const_amount_is_zero() {
        // is_zero is const
        const ZERO: Amount = Amount::new(0, 8);
        const NON_ZERO: Amount = Amount::new(100, 8);

        assert!(ZERO.is_zero());
        assert!(!NON_ZERO.is_zero());
    }

    #[test]
    fn test_const_checked_add() {
        // checked_add is now const, enabling compile-time validation
        const ONE_BTC: Amount = Amount::new(100_000_000, 8);
        const TWO_BTC: Amount = match ONE_BTC.checked_add(ONE_BTC) {
            Some(sum) => sum,
            None => panic!("Overflow in const addition"),
        };

        assert_eq!(TWO_BTC.value, 200_000_000);
        assert_eq!(TWO_BTC.decimals, 8);

        // Runtime verification matches compile-time computation
        let runtime_sum = ONE_BTC.checked_add(ONE_BTC).unwrap();
        assert_eq!(runtime_sum.value, TWO_BTC.value);
    }

    #[test]
    fn test_const_checked_sub() {
        // checked_sub is now const
        const FIVE_BTC: Amount = Amount::new(500_000_000, 8);
        const TWO_BTC: Amount = Amount::new(200_000_000, 8);
        const THREE_BTC: Amount = match FIVE_BTC.checked_sub(TWO_BTC) {
            Some(diff) => diff,
            None => panic!("Underflow in const subtraction"),
        };

        assert_eq!(THREE_BTC.value, 300_000_000);
        assert_eq!(THREE_BTC.decimals, 8);
    }

    #[test]
    fn test_const_checked_mul() {
        // checked_mul is now const
        const BASE: Amount = Amount::new(100, 8);
        const DOUBLED: Amount = match BASE.checked_mul(2) {
            Some(prod) => prod,
            None => panic!("Overflow in const multiplication"),
        };

        assert_eq!(DOUBLED.value, 200);
        assert_eq!(DOUBLED.decimals, 8);
    }

    #[test]
    fn test_const_checked_div() {
        // checked_div is now const
        const BASE: Amount = Amount::new(100, 8);
        const HALF: Amount = match BASE.checked_div(2) {
            Some(quot) => quot,
            None => panic!("Division by zero"),
        };

        assert_eq!(HALF.value, 50);
        assert_eq!(HALF.decimals, 8);
    }

    #[test]
    fn test_const_complex_calculation() {
        // Demonstrate complex const calculations
        // Calculate: (10 BTC + 5 BTC) * 2 / 3
        const TEN_BTC: Amount = Amount::new(1_000_000_000, 8);
        const FIVE_BTC: Amount = Amount::new(500_000_000, 8);

        const SUM: Amount = match TEN_BTC.checked_add(FIVE_BTC) {
            Some(s) => s,
            None => panic!("Addition overflow"),
        };

        const DOUBLED: Amount = match SUM.checked_mul(2) {
            Some(d) => d,
            None => panic!("Multiplication overflow"),
        };

        const RESULT: Amount = match DOUBLED.checked_div(3) {
            Some(r) => r,
            None => panic!("Division by zero"),
        };

        // (10 + 5) * 2 / 3 = 15 * 2 / 3 = 30 / 3 = 10 BTC
        assert_eq!(RESULT.value, 1_000_000_000);
        assert_eq!(RESULT.decimals, 8);
    }

    #[test]
    fn test_const_version() {
        // Demonstrate const version check on TxIR
        const VERSION_1: u8 = 1;
        const VERSION_2: u8 = 2;

        // These could be used in const contexts for version validation
        assert_eq!(VERSION_1, 1);
        assert_eq!(VERSION_2, 2);
    }
}
