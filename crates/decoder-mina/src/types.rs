//! Mina Protocol transaction types
//!
//! This module defines the core transaction types for Mina Protocol:
//! - zkApp transactions (smart contract transactions)
//! - Payment transactions (simple value transfers)
//! - Account updates (state changes)
//! - Delegation transactions (stake delegation)

use decoder_crypto_zk::field::pallas::PallasFieldElement;
use serde::{Deserialize, Serialize};

/// Mina transaction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinaTransaction {
    /// Payment transaction (simple value transfer)
    Payment(PaymentTransaction),

    /// zkApp transaction (smart contract transaction)
    ZkApp(ZkAppTransaction),

    /// Stake delegation transaction
    Delegation(DelegationTransaction),
}

/// Payment transaction
///
/// Standard Mina payment from one account to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentTransaction {
    /// Source public key (sender)
    pub source: PublicKey,

    /// Receiver public key
    pub receiver: PublicKey,

    /// Amount in nanomina (10^-9 MINA)
    pub amount: u64,

    /// Transaction fee in nanomina
    pub fee: u64,

    /// Account nonce (for replay protection)
    pub nonce: u32,

    /// Valid until (global slot number)
    pub valid_until: Option<u32>,

    /// Optional memo (32 bytes max)
    pub memo: Option<Vec<u8>>,

    /// Signature
    pub signature: Signature,
}

/// zkApp transaction
///
/// Smart contract transaction containing account updates and proofs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkAppTransaction {
    /// Fee payer (who pays for this transaction)
    pub fee_payer: PublicKey,

    /// Fee amount in nanomina
    pub fee: u64,

    /// Fee payer nonce
    pub nonce: u32,

    /// Valid until (global slot number)
    pub valid_until: Option<u32>,

    /// Account updates (state changes)
    pub account_updates: Vec<AccountUpdate>,

    /// Optional memo (32 bytes max)
    pub memo: Option<Vec<u8>>,
}

/// Account update in a zkApp transaction
///
/// Represents a state change to an account, including:
/// - Balance changes
/// - State updates (8 field elements)
/// - Verification key updates
/// - Permissions changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountUpdate {
    /// Public key of the account being updated
    pub public_key: PublicKey,

    /// Token ID (default is MINA token)
    pub token_id: TokenId,

    /// Update type
    pub update: Update,

    /// Balance change (positive = receive, negative = send)
    pub balance_change: i64,

    /// Account state updates (8 field elements)
    pub state_update: Option<[PallasFieldElement; 8]>,

    /// Call depth (for nested zkApp calls)
    pub call_depth: u8,

    /// Proof or signature authorization
    pub authorization: Authorization,
}

/// Update data for an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    /// App state updates (8 field elements)
    pub app_state: Option<Vec<PallasFieldElement>>,

    /// Delegate update
    pub delegate: Option<PublicKey>,

    /// Verification key update
    pub verification_key: Option<Vec<u8>>,

    /// Permissions update
    pub permissions: Option<Permissions>,

    /// Zkapp URI update
    pub zkapp_uri: Option<String>,

    /// Token symbol update
    pub token_symbol: Option<String>,

    /// Timing update (vesting schedule)
    pub timing: Option<Timing>,

    /// Voting for update
    pub voting_for: Option<PallasFieldElement>,
}

/// Authorization for an account update
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Authorization {
    /// No authorization required
    None,

    /// Signature authorization
    Signature(Signature),

    /// Proof authorization (recursive zkSNARK)
    Proof(Vec<u8>),
}

/// Mina public key (Pallas curve point)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    /// X coordinate (Pallas field element)
    pub x: PallasFieldElement,

    /// Y parity (compressed point encoding)
    pub is_odd: bool,
}

impl PublicKey {
    /// Create a new public key
    pub fn new(x: PallasFieldElement, is_odd: bool) -> Self {
        Self { x, is_odd }
    }

    /// Convert to base58check address
    pub fn to_address(&self) -> String {
        // TODO: Implement base58check encoding
        // Mina addresses start with "B62q..."
        format!("B62q{:?}", self.x)
    }
}

/// Signature (Schnorr-like signature on Pallas curve)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// R point (field element)
    pub r: PallasFieldElement,

    /// S scalar (field element)
    pub s: PallasFieldElement,
}

/// Token ID (for custom tokens in zkApps)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenId(pub PallasFieldElement);

impl TokenId {
    /// Default MINA token ID
    pub fn default_mina() -> Self {
        Self(PallasFieldElement::one())
    }
}

/// Permissions for an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    /// Edit state permission
    pub edit_state: Permission,

    /// Send permission
    pub send: Permission,

    /// Receive permission
    pub receive: Permission,

    /// Set delegate permission
    pub set_delegate: Permission,

    /// Set permissions permission
    pub set_permissions: Permission,

    /// Set verification key permission
    pub set_verification_key: Permission,
}

/// Permission level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// No permission required
    None,

    /// Requires signature
    Signature,

    /// Requires proof
    Proof,

    /// Impossible (permission disabled)
    Impossible,
}

/// Timing (vesting schedule)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timing {
    /// Initial minimum balance
    pub initial_minimum_balance: u64,

    /// Cliff time (when vesting starts)
    pub cliff_time: u32,

    /// Cliff amount
    pub cliff_amount: u64,

    /// Vesting period (in slots)
    pub vesting_period: u32,

    /// Vesting increment
    pub vesting_increment: u64,
}

/// Delegation transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationTransaction {
    /// Delegator public key
    pub delegator: PublicKey,

    /// New delegate public key
    pub new_delegate: PublicKey,

    /// Transaction fee
    pub fee: u64,

    /// Nonce
    pub nonce: u32,

    /// Valid until
    pub valid_until: Option<u32>,

    /// Memo
    pub memo: Option<Vec<u8>>,

    /// Signature
    pub signature: Signature,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_creation() {
        let x = PallasFieldElement::from(12345u64);
        let pk = PublicKey::new(x.clone(), true);

        assert_eq!(pk.x, x);
        assert!(pk.is_odd);
    }

    #[test]
    fn test_token_id_default() {
        let default_token = TokenId::default_mina();
        assert_eq!(default_token.0, PallasFieldElement::one());
    }

    #[test]
    fn test_signature_creation() {
        let r = PallasFieldElement::from(111u64);
        let s = PallasFieldElement::from(222u64);
        let sig = Signature {
            r: r.clone(),
            s: s.clone(),
        };

        assert_eq!(sig.r, r);
        assert_eq!(sig.s, s);
    }
}
