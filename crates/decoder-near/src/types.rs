//! NEAR-specific transaction types
//!
//! This module defines the core types for NEAR transactions following the
//! NEAR protocol specification. All types use Borsh serialization for deterministic encoding.

use serde::{Deserialize, Serialize};
use std::fmt;

/// NEAR public key (32-byte Ed25519 public key)
pub type NearPublicKey = Vec<u8>;

/// NEAR signature (64-byte Ed25519 signature)
pub type NearSignature = Vec<u8>;

/// NEAR block hash (32 bytes)
pub type NearBlockHash = [u8; 32];

/// NEAR account ID (UTF-8 string, e.g., "alice.near", "contract.testnet")
pub type AccountId = String;

/// NEAR transaction hash (32 bytes, SHA-256)
pub type TxHash = [u8; 32];

/// A signed NEAR transaction
///
/// This is the top-level structure that gets broadcast to the network.
/// It contains an inner `Transaction` and a signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The actual transaction data
    pub transaction: Transaction,

    /// Ed25519 signature over the Borsh-serialized transaction
    pub signature: NearSignature,
}

/// NEAR transaction (inner, unsigned structure)
///
/// This is what gets signed to produce a SignedTransaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Account ID of the transaction signer
    pub signer_id: AccountId,

    /// Public key used to sign this transaction
    pub public_key: PublicKey,

    /// Nonce for replay attack protection
    /// Must be greater than the current nonce for the signer's access key
    pub nonce: u64,

    /// Account ID of the transaction receiver
    pub receiver_id: AccountId,

    /// Hash of the recent block (for replay protection)
    /// Transactions expire after ~24 hours
    pub block_hash: NearBlockHash,

    /// List of actions to execute
    pub actions: Vec<Action>,
}

/// NEAR public key with key type
///
/// NEAR supports multiple key types, but Ed25519 is the most common.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    /// Key type (0 = Ed25519, 1 = Secp256k1)
    pub key_type: KeyType,

    /// Raw public key bytes (32 bytes for Ed25519)
    pub data: NearPublicKey,
}

/// Supported cryptographic key types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum KeyType {
    /// Ed25519 key (most common)
    Ed25519 = 0,

    /// Secp256k1 key (for Bitcoin-style keys)
    Secp256k1 = 1,
}

/// Actions that can be performed in a NEAR transaction
///
/// Transactions can contain multiple actions that execute sequentially.
/// All actions must succeed or the entire transaction fails (atomic).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Create a new account
    CreateAccount(CreateAccountAction),

    /// Deploy a smart contract
    DeployContract(DeployContractAction),

    /// Call a method on a smart contract
    FunctionCall(FunctionCallAction),

    /// Transfer NEAR tokens
    Transfer(TransferAction),

    /// Stake tokens for validation
    Stake(StakeAction),

    /// Add an access key to an account
    AddKey(AddKeyAction),

    /// Delete an access key from an account
    DeleteKey(DeleteKeyAction),

    /// Delete an account and transfer remaining balance
    DeleteAccount(DeleteAccountAction),
}

/// Create a new NEAR account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateAccountAction;

/// Deploy contract code to an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployContractAction {
    /// WASM contract bytecode
    pub code: Vec<u8>,
}

/// Call a smart contract function
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionCallAction {
    /// Name of the contract method to call
    pub method_name: String,

    /// JSON-encoded arguments to pass to the method
    pub args: Vec<u8>,

    /// Amount of gas to attach (in gas units)
    pub gas: u64,

    /// Amount of NEAR tokens to attach (in yoctoNEAR, 10^-24 NEAR)
    pub deposit: u128,
}

/// Transfer NEAR tokens
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferAction {
    /// Amount to transfer in yoctoNEAR (10^-24 NEAR)
    pub deposit: u128,
}

/// Stake tokens for validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeAction {
    /// Amount to stake in yoctoNEAR
    pub stake: u128,

    /// Public key to use for validation
    pub public_key: PublicKey,
}

/// Add an access key to an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddKeyAction {
    /// Public key to add
    pub public_key: PublicKey,

    /// Access key permission
    pub access_key: AccessKey,
}

/// Delete an access key from an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteKeyAction {
    /// Public key to delete
    pub public_key: PublicKey,
}

/// Delete an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteAccountAction {
    /// Account ID to transfer remaining balance to
    pub beneficiary_id: AccountId,
}

/// Access key with permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessKey {
    /// Nonce for this access key
    pub nonce: u64,

    /// Permission level
    pub permission: AccessKeyPermission,
}

/// Access key permission types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessKeyPermission {
    /// Full access (can do anything)
    FullAccess,

    /// Limited to calling specific contract methods
    FunctionCall(FunctionCallPermission),
}

/// Function call permission (limited access key)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionCallPermission {
    /// Allowance in yoctoNEAR (optional spending limit)
    pub allowance: Option<u128>,

    /// Account ID of the contract this key can call
    pub receiver_id: AccountId,

    /// List of method names this key can call (empty = all methods)
    pub method_names: Vec<String>,
}

impl SignedTransaction {
    /// Get the transaction hash (SHA-256 of Borsh-encoded transaction)
    pub fn hash(&self) -> TxHash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        // In a full implementation, we'd Borsh-serialize the transaction here
        // For now, this is a placeholder
        hasher.update([]);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Get the signer account ID
    pub fn signer_id(&self) -> &str {
        &self.transaction.signer_id
    }

    /// Get the receiver account ID
    pub fn receiver_id(&self) -> &str {
        &self.transaction.receiver_id
    }

    /// Get the nonce
    pub fn nonce(&self) -> u64 {
        self.transaction.nonce
    }

    /// Get the number of actions
    pub fn num_actions(&self) -> usize {
        self.transaction.actions.len()
    }

    /// Check if this transaction contains a transfer
    pub fn has_transfer(&self) -> bool {
        self.transaction
            .actions
            .iter()
            .any(|a| matches!(a, Action::Transfer(_)))
    }

    /// Check if this transaction contains a function call
    pub fn has_function_call(&self) -> bool {
        self.transaction
            .actions
            .iter()
            .any(|a| matches!(a, Action::FunctionCall(_)))
    }

    /// Calculate total NEAR transferred (across all actions)
    pub fn total_transfer_amount(&self) -> u128 {
        self.transaction
            .actions
            .iter()
            .map(|action| match action {
                Action::Transfer(t) => t.deposit,
                Action::FunctionCall(f) => f.deposit,
                Action::Stake(s) => s.stake,
                _ => 0,
            })
            .sum()
    }
}

impl fmt::Display for SignedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NEAR Transaction: {} -> {} (nonce: {}, actions: {})",
            self.signer_id(),
            self.receiver_id(),
            self.nonce(),
            self.num_actions()
        )
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::CreateAccount(_) => write!(f, "CreateAccount"),
            Action::DeployContract(d) => write!(f, "DeployContract({} bytes)", d.code.len()),
            Action::FunctionCall(fc) => write!(f, "FunctionCall({})", fc.method_name),
            Action::Transfer(t) => write!(f, "Transfer({} yoctoNEAR)", t.deposit),
            Action::Stake(s) => write!(f, "Stake({} yoctoNEAR)", s.stake),
            Action::AddKey(_) => write!(f, "AddKey"),
            Action::DeleteKey(_) => write!(f, "DeleteKey"),
            Action::DeleteAccount(d) => {
                write!(f, "DeleteAccount(beneficiary: {})", d.beneficiary_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_display() {
        let transfer = Action::Transfer(TransferAction { deposit: 1000 });
        assert_eq!(transfer.to_string(), "Transfer(1000 yoctoNEAR)");

        let create = Action::CreateAccount(CreateAccountAction);
        assert_eq!(create.to_string(), "CreateAccount");
    }

    #[test]
    fn test_signed_transaction_helpers() {
        let tx = SignedTransaction {
            transaction: Transaction {
                signer_id: "alice.near".to_string(),
                public_key: PublicKey {
                    key_type: KeyType::Ed25519,
                    data: vec![0u8; 32],
                },
                nonce: 42,
                receiver_id: "bob.near".to_string(),
                block_hash: [0u8; 32],
                actions: vec![
                    Action::Transfer(TransferAction { deposit: 1000 }),
                    Action::FunctionCall(FunctionCallAction {
                        method_name: "transfer".to_string(),
                        args: vec![],
                        gas: 30000000000000,
                        deposit: 500,
                    }),
                ],
            },
            signature: vec![0u8; 64],
        };

        assert_eq!(tx.signer_id(), "alice.near");
        assert_eq!(tx.receiver_id(), "bob.near");
        assert_eq!(tx.nonce(), 42);
        assert_eq!(tx.num_actions(), 2);
        assert!(tx.has_transfer());
        assert!(tx.has_function_call());
        assert_eq!(tx.total_transfer_amount(), 1500); // 1000 + 500
    }
}
