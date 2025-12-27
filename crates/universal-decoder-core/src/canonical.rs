//! Canonical serialization for TxIR
//!
//! This module provides canonical binary serialization using Borsh to ensure
//! deterministic encoding, which is critical for signature verification and
//! preventing transaction malleability.
//!
//! ## Why Borsh?
//!
//! Borsh (Binary Object Representation Serializer for Hashing) guarantees:
//! - **Deterministic**: Same data always produces same bytes
//! - **Bijective**: One-to-one mapping between data and bytes
//! - **Efficient**: Binary format, no overhead
//! - **No ambiguity**: Fixed encoding rules
//!
//! ## JSON is NOT canonical!
//!
//! JSON should ONLY be used for:
//! - Human-readable display
//! - Debugging
//! - API responses
//!
//! JSON must NEVER be used for:
//! - Transaction hashing
//! - Signature verification
//! - Canonical representation

use crate::chain::ChainRef;
use crate::error::{DecoderError, Result};
use crate::ir::*;
use borsh::{BorshDeserialize, BorshSerialize};

/// Trait for types that can be canonically serialized
///
/// This trait ensures that a type can be serialized to a deterministic
/// byte representation suitable for hashing and signature verification.
pub trait CanonicalSerialize {
    /// Serialize to canonical bytes using Borsh
    ///
    /// This method MUST be deterministic: calling it multiple times
    /// on the same value must produce identical bytes.
    fn to_canonical_bytes(&self) -> Result<Vec<u8>>;

    /// Deserialize from canonical bytes
    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// Compute the canonical hash (SHA-256 of canonical bytes)
    fn canonical_hash(&self) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha256};
        let bytes = self.to_canonical_bytes()?;
        Ok(Sha256::digest(&bytes).to_vec())
    }
}

/// Wrapper for canonical serialization of TxIR
///
/// Since TxIR uses PhantomData and lifetimes, we need a serialization-friendly
/// representation. This struct strips away the lifetime and phantom data.
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalTxIR {
    pub version: u8,
    pub chain: ChainRef, // Now uses ChainRef directly (already Borsh-serializable)
    pub metadata: CanonicalTxMetadata,
    pub authorization: CanonicalAuthorizationPackage,
    pub operations: Vec<CanonicalOperation>,
    pub state_deltas: CanonicalStateDeltas,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalTxMetadata {
    pub tx_hash: Vec<u8>,
    pub block_height: Option<u64>,
    pub timestamp: Option<u64>,
    pub size: usize,
    pub extra: String, // JSON string for extra data
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalAuthorizationPackage {
    pub signatures: Vec<CanonicalSignature>,
    pub public_keys: Vec<CanonicalPublicKey>,
    pub signature_scheme: CanonicalSignatureScheme,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalSignature {
    pub data: Vec<u8>,
    pub key_index: usize,
    pub metadata: Option<String>, // JSON string
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalPublicKey {
    pub data: Vec<u8>,
    pub key_type: CanonicalKeyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum CanonicalSignatureScheme {
    Ecdsa,
    EdDsa,
    Schnorr,
    Custom(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum CanonicalKeyType {
    Secp256k1,
    Ed25519,
    P256,
    Custom(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum CanonicalOperation {
    Transfer(CanonicalTransfer),
    ContractCall(CanonicalContractCall),
    ContractDeploy(CanonicalContractDeploy),
    Stake(CanonicalStake),
    Generic(CanonicalGenericOperation),
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalTransfer {
    pub from: CanonicalAddress,
    pub to: CanonicalAddress,
    pub amount: CanonicalAmount,
    pub asset: CanonicalAssetId,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalContractCall {
    pub contract: CanonicalAddress,
    pub method: Vec<u8>,
    pub data: Vec<u8>,
    pub value: Option<CanonicalAmount>,
    pub resource_limits: CanonicalResourceLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalContractDeploy {
    pub bytecode: Vec<u8>,
    pub constructor_args: Vec<u8>,
    pub value: CanonicalAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalStake {
    pub validator: CanonicalAddress,
    pub amount: CanonicalAmount,
    pub operation_type: CanonicalStakeOperationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum CanonicalStakeOperationType {
    Delegate,
    Undelegate,
    Redelegate,
    Claim,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalGenericOperation {
    pub op_type: String,
    pub data: Vec<u8>,
    pub metadata: String, // JSON string
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalStateDeltas {
    pub inputs: Vec<CanonicalInputReference>,
    pub outputs: Vec<CanonicalOutputValue>,
    pub account_changes: Vec<CanonicalAccountChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalInputReference {
    pub prev_tx: Vec<u8>,
    pub output_index: u32,
    pub value: CanonicalAmount,
    pub script: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalOutputValue {
    pub index: u32,
    pub address: CanonicalAddress,
    pub value: CanonicalAmount,
    pub script: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalAccountChange {
    pub address: CanonicalAddress,
    pub nonce: Option<u64>,
    pub balance_change: i128,
    pub storage_changes: Vec<CanonicalStorageChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalStorageChange {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalAddress {
    pub bytes: Vec<u8>,
    pub human_readable: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalAmount {
    pub value: u128,
    pub decimals: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum CanonicalAssetId {
    Native,
    Token(Vec<u8>),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CanonicalResourceLimits {
    pub max_units: u64,
    pub unit_price: u64,
    pub resource_type: CanonicalResourceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum CanonicalResourceType {
    Gas,
    ComputeUnits,
    Weight,
    Custom(u32),
}

// Conversion implementations
impl<const V: u8> TxIR<'_, V> {
    /// Convert to canonical representation for serialization
    pub fn to_canonical(&self) -> CanonicalTxIR {
        CanonicalTxIR {
            version: V,
            chain: self.chain.clone(), // ChainRef is already serializable
            metadata: (&self.metadata).into(),
            authorization: (&self.authorization).into(),
            operations: self.operations.iter().map(Into::into).collect(),
            state_deltas: (&self.state_deltas).into(),
        }
    }
}

impl From<&TxMetadata> for CanonicalTxMetadata {
    fn from(meta: &TxMetadata) -> Self {
        Self {
            tx_hash: meta.tx_hash.clone(),
            block_height: meta.block_height,
            timestamp: meta.timestamp,
            size: meta.size,
            extra: meta.extra.clone(),
        }
    }
}

impl From<&AuthorizationPackage> for CanonicalAuthorizationPackage {
    fn from(auth: &AuthorizationPackage) -> Self {
        Self {
            signatures: auth.signatures.iter().map(Into::into).collect(),
            public_keys: auth.public_keys.iter().map(Into::into).collect(),
            signature_scheme: auth.signature_scheme.into(),
        }
    }
}

impl From<&Signature> for CanonicalSignature {
    fn from(sig: &Signature) -> Self {
        Self {
            data: sig.data.clone(),
            key_index: sig.key_index,
            metadata: sig.metadata.clone(),
        }
    }
}

impl From<&PublicKey> for CanonicalPublicKey {
    fn from(pk: &PublicKey) -> Self {
        Self {
            data: pk.data.clone(),
            key_type: pk.key_type.into(),
        }
    }
}

impl From<KeyType> for CanonicalKeyType {
    fn from(kt: KeyType) -> Self {
        match kt {
            KeyType::Secp256k1 => CanonicalKeyType::Secp256k1,
            KeyType::Ed25519 => CanonicalKeyType::Ed25519,
            KeyType::P256 => CanonicalKeyType::P256,
            KeyType::Custom(n) => CanonicalKeyType::Custom(n),
        }
    }
}

impl From<SignatureScheme> for CanonicalSignatureScheme {
    fn from(scheme: SignatureScheme) -> Self {
        match scheme {
            SignatureScheme::Ecdsa => CanonicalSignatureScheme::Ecdsa,
            SignatureScheme::EdDsa => CanonicalSignatureScheme::EdDsa,
            SignatureScheme::Schnorr => CanonicalSignatureScheme::Schnorr,
            SignatureScheme::Custom(n) => CanonicalSignatureScheme::Custom(n),
        }
    }
}

impl From<&Operation> for CanonicalOperation {
    fn from(op: &Operation) -> Self {
        match op {
            Operation::Transfer(t) => CanonicalOperation::Transfer(t.into()),
            Operation::ContractCall(c) => CanonicalOperation::ContractCall(c.into()),
            Operation::ContractDeploy(d) => CanonicalOperation::ContractDeploy(d.into()),
            Operation::Stake(s) => CanonicalOperation::Stake(s.into()),
            Operation::Generic(g) => CanonicalOperation::Generic(g.into()),
        }
    }
}

impl From<&Transfer> for CanonicalTransfer {
    fn from(t: &Transfer) -> Self {
        Self {
            from: (&t.from).into(),
            to: (&t.to).into(),
            amount: (&t.amount).into(),
            asset: (&t.asset).into(),
        }
    }
}

impl From<&Address> for CanonicalAddress {
    fn from(addr: &Address) -> Self {
        Self {
            bytes: addr.bytes.clone(),
            human_readable: addr.human_readable.clone(),
        }
    }
}

impl From<&Amount> for CanonicalAmount {
    fn from(amt: &Amount) -> Self {
        Self {
            value: amt.value,
            decimals: amt.decimals,
        }
    }
}

impl From<&AssetId> for CanonicalAssetId {
    fn from(asset: &AssetId) -> Self {
        match asset {
            AssetId::Native => CanonicalAssetId::Native,
            AssetId::Token(t) => CanonicalAssetId::Token(t.clone()),
            AssetId::Custom(c) => CanonicalAssetId::Custom(c.clone()),
        }
    }
}

impl From<&ContractCall> for CanonicalContractCall {
    fn from(c: &ContractCall) -> Self {
        Self {
            contract: (&c.contract).into(),
            method: c.method.clone(),
            data: c.data.clone(),
            value: c.value.as_ref().map(Into::into),
            resource_limits: (&c.resource_limits).into(),
        }
    }
}

impl From<&ResourceLimits> for CanonicalResourceLimits {
    fn from(r: &ResourceLimits) -> Self {
        Self {
            max_units: r.max_units,
            unit_price: r.unit_price,
            resource_type: r.resource_type.into(),
        }
    }
}

impl From<ResourceType> for CanonicalResourceType {
    fn from(rt: ResourceType) -> Self {
        match rt {
            ResourceType::Gas => CanonicalResourceType::Gas,
            ResourceType::ComputeUnits => CanonicalResourceType::ComputeUnits,
            ResourceType::Weight => CanonicalResourceType::Weight,
            ResourceType::Custom(n) => CanonicalResourceType::Custom(n),
        }
    }
}

impl From<&ContractDeploy> for CanonicalContractDeploy {
    fn from(d: &ContractDeploy) -> Self {
        Self {
            bytecode: d.bytecode.clone(),
            constructor_args: d.constructor_args.clone(),
            value: (&d.value).into(),
        }
    }
}

impl From<&Stake> for CanonicalStake {
    fn from(s: &Stake) -> Self {
        Self {
            validator: (&s.validator).into(),
            amount: (&s.amount).into(),
            operation_type: s.operation_type.into(),
        }
    }
}

impl From<StakeOperationType> for CanonicalStakeOperationType {
    fn from(sot: StakeOperationType) -> Self {
        match sot {
            StakeOperationType::Delegate => CanonicalStakeOperationType::Delegate,
            StakeOperationType::Undelegate => CanonicalStakeOperationType::Undelegate,
            StakeOperationType::Redelegate => CanonicalStakeOperationType::Redelegate,
            StakeOperationType::Claim => CanonicalStakeOperationType::Claim,
        }
    }
}

impl From<&GenericOperation> for CanonicalGenericOperation {
    fn from(g: &GenericOperation) -> Self {
        Self {
            op_type: g.op_type.clone(),
            data: g.data.clone(),
            metadata: g.metadata.clone(),
        }
    }
}

impl From<&StateDeltas> for CanonicalStateDeltas {
    fn from(sd: &StateDeltas) -> Self {
        Self {
            inputs: sd.inputs.iter().map(Into::into).collect(),
            outputs: sd.outputs.iter().map(Into::into).collect(),
            account_changes: sd.account_changes.iter().map(Into::into).collect(),
        }
    }
}

impl From<&InputReference> for CanonicalInputReference {
    fn from(i: &InputReference) -> Self {
        Self {
            prev_tx: i.prev_tx.clone(),
            output_index: i.output_index,
            value: (&i.value).into(),
            script: i.script.clone(),
        }
    }
}

impl From<&OutputValue> for CanonicalOutputValue {
    fn from(o: &OutputValue) -> Self {
        Self {
            index: o.index,
            address: (&o.address).into(),
            value: (&o.value).into(),
            script: o.script.clone(),
        }
    }
}

impl From<&AccountChange> for CanonicalAccountChange {
    fn from(ac: &AccountChange) -> Self {
        Self {
            address: (&ac.address).into(),
            nonce: ac.nonce,
            balance_change: ac.balance_change,
            storage_changes: ac.storage_changes.iter().map(Into::into).collect(),
        }
    }
}

impl From<&StorageChange> for CanonicalStorageChange {
    fn from(sc: &StorageChange) -> Self {
        Self {
            key: sc.key.clone(),
            value: sc.value.clone(),
        }
    }
}

impl<const V: u8> CanonicalSerialize for TxIR<'_, V> {
    fn to_canonical_bytes(&self) -> Result<Vec<u8>> {
        let canonical = self.to_canonical();
        borsh::to_vec(&canonical)
            .map_err(|e| DecoderError::serialization(format!("Borsh serialization failed: {}", e)))
    }

    fn from_canonical_bytes(_bytes: &[u8]) -> Result<Self> {
        Err(DecoderError::serialization(
            "Cannot deserialize TxIR with lifetime from bytes",
        ))
    }
}

impl CanonicalSerialize for CanonicalTxIR {
    fn to_canonical_bytes(&self) -> Result<Vec<u8>> {
        borsh::to_vec(self)
            .map_err(|e| DecoderError::serialization(format!("Borsh serialization failed: {}", e)))
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self> {
        borsh::from_slice(bytes).map_err(|e| {
            DecoderError::serialization(format!("Borsh deserialization failed: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_serialization_deterministic() {
        use crate::chain::ChainFamilyEncoded;

        let canonical_tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 0,
                name: "Bitcoin".to_string(),
                family: ChainFamilyEncoded::Utxo,
                network: Some("mainnet".to_string()),
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![1, 2, 3, 4],
                block_height: Some(100),
                timestamp: Some(1234567890),
                size: 250,
                extra: "{}".to_string(),
            },
            authorization: CanonicalAuthorizationPackage {
                signatures: vec![],
                public_keys: vec![],
                signature_scheme: CanonicalSignatureScheme::Ecdsa,
            },
            operations: vec![],
            state_deltas: CanonicalStateDeltas {
                inputs: vec![],
                outputs: vec![],
                account_changes: vec![],
            },
        };

        // Serialize twice - should produce identical bytes
        let bytes1 = canonical_tx.to_canonical_bytes().unwrap();
        let bytes2 = canonical_tx.to_canonical_bytes().unwrap();

        assert_eq!(
            bytes1, bytes2,
            "Canonical serialization must be deterministic"
        );
    }

    #[test]
    fn test_canonical_roundtrip() {
        use crate::chain::ChainFamilyEncoded;

        let canonical_tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 1,
                name: "Ethereum".to_string(),
                family: ChainFamilyEncoded::Account,
                network: Some("mainnet".to_string()),
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![0xff; 32],
                block_height: None,
                timestamp: None,
                size: 500,
                extra: "{}".to_string(),
            },
            authorization: CanonicalAuthorizationPackage {
                signatures: vec![],
                public_keys: vec![],
                signature_scheme: CanonicalSignatureScheme::Ecdsa,
            },
            operations: vec![],
            state_deltas: CanonicalStateDeltas {
                inputs: vec![],
                outputs: vec![],
                account_changes: vec![],
            },
        };

        // Serialize and deserialize
        let bytes = canonical_tx.to_canonical_bytes().unwrap();
        let deserialized = CanonicalTxIR::from_canonical_bytes(&bytes).unwrap();

        assert_eq!(canonical_tx, deserialized, "Roundtrip must preserve data");
    }

    #[test]
    fn test_canonical_hash_deterministic() {
        use crate::chain::ChainFamilyEncoded;

        let canonical_tx = CanonicalTxIR {
            version: 1,
            chain: ChainRef {
                id: 501,
                name: "Solana".to_string(),
                family: ChainFamilyEncoded::Instruction,
                network: Some("mainnet-beta".to_string()),
            },
            metadata: CanonicalTxMetadata {
                tx_hash: vec![],
                block_height: Some(999),
                timestamp: Some(9999999),
                size: 100,
                extra: "{}".to_string(),
            },
            authorization: CanonicalAuthorizationPackage {
                signatures: vec![],
                public_keys: vec![],
                signature_scheme: CanonicalSignatureScheme::EdDsa,
            },
            operations: vec![],
            state_deltas: CanonicalStateDeltas {
                inputs: vec![],
                outputs: vec![],
                account_changes: vec![],
            },
        };

        // Compute hash twice - should be identical
        let hash1 = canonical_tx.canonical_hash().unwrap();
        let hash2 = canonical_tx.canonical_hash().unwrap();

        assert_eq!(hash1, hash2, "Canonical hash must be deterministic");
        assert_eq!(hash1.len(), 32, "SHA-256 hash should be 32 bytes");
    }
}
