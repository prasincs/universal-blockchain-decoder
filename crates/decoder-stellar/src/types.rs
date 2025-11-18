//! Stellar-specific transaction types
//!
//! This module defines the core types for Stellar transactions, operations, and assets
//! following a pure Rust implementation approach.
//!
//! Stellar uses XDR (External Data Representation) for encoding, which is a binary
//! format with big-endian encoding.

use serde::{Deserialize, Serialize};
use std::fmt;
use universal_decoder_core::hex;

/// Stellar account ID (32-byte Ed25519 public key)
pub type AccountId = Vec<u8>;

/// Stellar signature (64-byte Ed25519 signature)
pub type StellarSignature = Vec<u8>;

/// Stellar transaction hash (32 bytes)
pub type TxHash = Vec<u8>;

/// Stellar asset representation
///
/// Stellar supports three types of assets:
/// - Native (XLM)
/// - CreditAlphanum4 (e.g., USDC, with 4-character code)
/// - CreditAlphanum12 (e.g., LONGERNAME, with up to 12-character code)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StellarAsset {
    /// Native asset (XLM - Stellar Lumens)
    Native,

    /// Asset with 4-character alphanumeric code
    CreditAlphanum4 {
        /// Asset code (e.g., "USDC")
        code: [u8; 4],
        /// Issuer account ID
        issuer: AccountId,
    },

    /// Asset with up to 12-character alphanumeric code
    CreditAlphanum12 {
        /// Asset code (e.g., "LONGERNAME")
        code: [u8; 12],
        /// Issuer account ID
        issuer: AccountId,
    },
}

impl StellarAsset {
    /// Get the asset code as a string (for display purposes)
    pub fn code_string(&self) -> String {
        match self {
            StellarAsset::Native => "XLM".to_string(),
            StellarAsset::CreditAlphanum4 { code, .. } => String::from_utf8_lossy(code)
                .trim_end_matches('\0')
                .to_string(),
            StellarAsset::CreditAlphanum12 { code, .. } => String::from_utf8_lossy(code)
                .trim_end_matches('\0')
                .to_string(),
        }
    }

    /// Check if this is the native asset (XLM)
    pub fn is_native(&self) -> bool {
        matches!(self, StellarAsset::Native)
    }

    /// Get the issuer account ID (None for native)
    pub fn issuer(&self) -> Option<&AccountId> {
        match self {
            StellarAsset::Native => None,
            StellarAsset::CreditAlphanum4 { issuer, .. } => Some(issuer),
            StellarAsset::CreditAlphanum12 { issuer, .. } => Some(issuer),
        }
    }
}

/// Stellar memo types
///
/// Memos are optional 28-byte fields that can contain arbitrary data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StellarMemo {
    /// No memo
    None,
    /// Text memo (UTF-8 string, up to 28 bytes)
    Text(String),
    /// ID memo (unsigned 64-bit integer)
    Id(u64),
    /// Hash memo (32-byte hash)
    Hash([u8; 32]),
    /// Return hash memo (32-byte hash for returns/refunds)
    Return([u8; 32]),
}

/// Time bounds for transaction validity
///
/// Transactions are only valid within a specific time window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeBounds {
    /// Minimum time (Unix timestamp)
    pub min_time: u64,
    /// Maximum time (Unix timestamp, 0 means no maximum)
    pub max_time: u64,
}

impl TimeBounds {
    /// Check if time bounds are valid (min <= max)
    pub fn is_valid(&self) -> bool {
        self.max_time == 0 || self.min_time <= self.max_time
    }
}

/// Stellar operation types
///
/// Stellar supports 24 different operation types as of Protocol 20.
/// Each operation type has different parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StellarOperation {
    /// Create a new account with a starting balance
    CreateAccount {
        destination: AccountId,
        starting_balance: i64,
    },

    /// Send payment from source to destination
    Payment {
        destination: AccountId,
        asset: StellarAsset,
        amount: i64,
    },

    /// Send payment along a specific path
    PathPaymentStrictReceive {
        send_asset: StellarAsset,
        send_max: i64,
        destination: AccountId,
        dest_asset: StellarAsset,
        dest_amount: i64,
        path: Vec<StellarAsset>,
    },

    /// Send payment along a path with exact send amount
    PathPaymentStrictSend {
        send_asset: StellarAsset,
        send_amount: i64,
        destination: AccountId,
        dest_asset: StellarAsset,
        dest_min: i64,
        path: Vec<StellarAsset>,
    },

    /// Create or update a sell offer
    ManageSellOffer {
        selling: StellarAsset,
        buying: StellarAsset,
        amount: i64,
        price_n: i32,
        price_d: i32,
        offer_id: i64,
    },

    /// Create or update a buy offer
    ManageBuyOffer {
        selling: StellarAsset,
        buying: StellarAsset,
        buy_amount: i64,
        price_n: i32,
        price_d: i32,
        offer_id: i64,
    },

    /// Create a passive sell offer
    CreatePassiveSellOffer {
        selling: StellarAsset,
        buying: StellarAsset,
        amount: i64,
        price_n: i32,
        price_d: i32,
    },

    /// Set account options (thresholds, signers, etc.)
    SetOptions {
        inflation_dest: Option<AccountId>,
        clear_flags: Option<u32>,
        set_flags: Option<u32>,
        master_weight: Option<u32>,
        low_threshold: Option<u32>,
        med_threshold: Option<u32>,
        high_threshold: Option<u32>,
        home_domain: Option<String>,
        signer_key: Option<Vec<u8>>,
        signer_weight: Option<u32>,
    },

    /// Change account trustline
    ChangeTrust { line: StellarAsset, limit: i64 },

    /// Allow another account to hold your assets
    AllowTrust {
        trustor: AccountId,
        asset_code: String,
        authorize: bool,
    },

    /// Merge account into another account
    AccountMerge { destination: AccountId },

    /// Set or clear account data entries
    ManageData {
        data_name: String,
        data_value: Option<Vec<u8>>,
    },

    /// Bump sequence number
    BumpSequence { bump_to: i64 },

    /// Create claimable balance
    CreateClaimableBalance {
        asset: StellarAsset,
        amount: i64,
        claimants: Vec<Vec<u8>>,
    },

    /// Claim a claimable balance
    ClaimClaimableBalance { balance_id: Vec<u8> },

    /// Begin sponsoring future reserves
    BeginSponsoringFutureReserves { sponsored_id: AccountId },

    /// End sponsoring future reserves
    EndSponsoringFutureReserves,

    /// Revoke sponsorship
    RevokeSponsorship {
        account_id: Option<AccountId>,
        data_name: Option<String>,
    },

    /// Clawback assets from an account
    Clawback {
        asset: StellarAsset,
        from: AccountId,
        amount: i64,
    },

    /// Clawback a claimable balance
    ClawbackClaimableBalance { balance_id: Vec<u8> },

    /// Set trust line flags
    SetTrustLineFlags {
        trustor: AccountId,
        asset: StellarAsset,
        clear_flags: u32,
        set_flags: u32,
    },

    /// Liquidity pool deposit
    LiquidityPoolDeposit {
        pool_id: Vec<u8>,
        max_amount_a: i64,
        max_amount_b: i64,
        min_price_n: i32,
        min_price_d: i32,
        max_price_n: i32,
        max_price_d: i32,
    },

    /// Liquidity pool withdraw
    LiquidityPoolWithdraw {
        pool_id: Vec<u8>,
        amount: i64,
        min_amount_a: i64,
        min_amount_b: i64,
    },

    /// Invoke Soroban smart contract
    InvokeHostFunction {
        function_type: u32,
        parameters: Vec<u8>,
    },

    /// Extend TTL for Soroban contract
    ExtendFootprintTtl { extend_to: u32 },

    /// Restore Soroban contract footprint
    RestoreFootprint,
}

impl StellarOperation {
    /// Get the operation type as a string
    pub fn operation_type(&self) -> &'static str {
        match self {
            StellarOperation::CreateAccount { .. } => "CreateAccount",
            StellarOperation::Payment { .. } => "Payment",
            StellarOperation::PathPaymentStrictReceive { .. } => "PathPaymentStrictReceive",
            StellarOperation::PathPaymentStrictSend { .. } => "PathPaymentStrictSend",
            StellarOperation::ManageSellOffer { .. } => "ManageSellOffer",
            StellarOperation::ManageBuyOffer { .. } => "ManageBuyOffer",
            StellarOperation::CreatePassiveSellOffer { .. } => "CreatePassiveSellOffer",
            StellarOperation::SetOptions { .. } => "SetOptions",
            StellarOperation::ChangeTrust { .. } => "ChangeTrust",
            StellarOperation::AllowTrust { .. } => "AllowTrust",
            StellarOperation::AccountMerge { .. } => "AccountMerge",
            StellarOperation::ManageData { .. } => "ManageData",
            StellarOperation::BumpSequence { .. } => "BumpSequence",
            StellarOperation::CreateClaimableBalance { .. } => "CreateClaimableBalance",
            StellarOperation::ClaimClaimableBalance { .. } => "ClaimClaimableBalance",
            StellarOperation::BeginSponsoringFutureReserves { .. } => {
                "BeginSponsoringFutureReserves"
            }
            StellarOperation::EndSponsoringFutureReserves => "EndSponsoringFutureReserves",
            StellarOperation::RevokeSponsorship { .. } => "RevokeSponsorship",
            StellarOperation::Clawback { .. } => "Clawback",
            StellarOperation::ClawbackClaimableBalance { .. } => "ClawbackClaimableBalance",
            StellarOperation::SetTrustLineFlags { .. } => "SetTrustLineFlags",
            StellarOperation::LiquidityPoolDeposit { .. } => "LiquidityPoolDeposit",
            StellarOperation::LiquidityPoolWithdraw { .. } => "LiquidityPoolWithdraw",
            StellarOperation::InvokeHostFunction { .. } => "InvokeHostFunction",
            StellarOperation::ExtendFootprintTtl { .. } => "ExtendFootprintTtl",
            StellarOperation::RestoreFootprint => "RestoreFootprint",
        }
    }

    /// Check if this operation involves a token transfer
    pub fn is_transfer(&self) -> bool {
        matches!(
            self,
            StellarOperation::Payment { .. }
                | StellarOperation::PathPaymentStrictReceive { .. }
                | StellarOperation::PathPaymentStrictSend { .. }
        )
    }

    /// Check if this operation involves Soroban (smart contracts)
    pub fn is_soroban(&self) -> bool {
        matches!(
            self,
            StellarOperation::InvokeHostFunction { .. }
                | StellarOperation::ExtendFootprintTtl { .. }
                | StellarOperation::RestoreFootprint
        )
    }
}

/// Decorated signature (signature + hint)
///
/// Stellar signatures include a 4-byte hint to help identify the signing key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoratedSignature {
    /// 4-byte hint (last 4 bytes of the public key)
    pub hint: [u8; 4],
    /// 64-byte Ed25519 signature
    pub signature: StellarSignature,
}

/// Stellar transaction
///
/// This represents a complete Stellar transaction with all fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StellarTransaction {
    /// Source account (transaction fee payer)
    pub source_account: AccountId,

    /// Transaction fee (in stroops, 1 XLM = 10^7 stroops)
    pub fee: u32,

    /// Sequence number (must be account sequence + 1)
    pub sequence_number: i64,

    /// Optional time bounds
    pub time_bounds: Option<TimeBounds>,

    /// Optional memo
    pub memo: StellarMemo,

    /// Operations to execute (up to 100 operations per transaction)
    pub operations: Vec<StellarOperation>,

    /// Signatures (decorated signatures with hints)
    pub signatures: Vec<DecoratedSignature>,

    /// Raw transaction bytes (for hash computation)
    pub raw_bytes: Vec<u8>,

    /// Envelope type (determines hash prefix)
    pub envelope_type: EnvelopeType,

    /// Network passphrase hash (for signature verification)
    pub network_id: Option<Vec<u8>>,
}

/// Stellar envelope type
///
/// Different envelope types use different hash prefixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvelopeType {
    /// Legacy transaction envelope
    TxV0 = 0,
    /// Transaction envelope (Protocol >= 13)
    Tx = 2,
    /// Fee bump transaction envelope
    TxFeeBump = 5,
}

impl StellarTransaction {
    /// Get the number of operations in this transaction
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Get the number of signatures
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }

    /// Check if this transaction has valid structure
    pub fn is_valid(&self) -> bool {
        // Stellar allows up to 100 operations per transaction
        if self.operations.is_empty() || self.operations.len() > 100 {
            return false;
        }

        // Must have at least one signature
        if self.signatures.is_empty() {
            return false;
        }

        // Time bounds must be valid if present
        if let Some(tb) = &self.time_bounds {
            if !tb.is_valid() {
                return false;
            }
        }

        true
    }

    /// Calculate Stellar transaction hash
    ///
    /// Stellar uses SHA-256 with a network-specific prefix:
    /// hash = SHA-256(network_id || envelope_type || tx_bytes)
    pub fn compute_hash(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Add network ID if available
        if let Some(ref network_id) = self.network_id {
            hasher.update(network_id);
        }

        // Add envelope type tag
        let envelope_tag: &[u8] = match self.envelope_type {
            EnvelopeType::TxV0 => b"ENVELOPE_TYPE_TX_V0",
            EnvelopeType::Tx => b"ENVELOPE_TYPE_TX",
            EnvelopeType::TxFeeBump => b"ENVELOPE_TYPE_TX_FEE_BUMP",
        };
        hasher.update(envelope_tag);

        // Add transaction bytes
        hasher.update(&self.raw_bytes);

        hasher.finalize().to_vec()
    }

    /// Get the fee per operation
    pub fn fee_per_operation(&self) -> u32 {
        if self.operations.is_empty() {
            0
        } else {
            self.fee / self.operations.len() as u32
        }
    }
}

impl fmt::Display for StellarAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StellarAsset::Native => write!(f, "XLM"),
            StellarAsset::CreditAlphanum4 { code, issuer } => {
                let code_str = String::from_utf8_lossy(code);
                let trimmed = code_str.trim_end_matches('\0');
                write!(f, "{}:{}", trimmed, hex::encode(&issuer[..4]))
            }
            StellarAsset::CreditAlphanum12 { code, issuer } => {
                let code_str = String::from_utf8_lossy(code);
                let trimmed = code_str.trim_end_matches('\0');
                write!(f, "{}:{}", trimmed, hex::encode(&issuer[..4]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_native() {
        let asset = StellarAsset::Native;
        assert!(asset.is_native());
        assert_eq!(asset.code_string(), "XLM");
        assert!(asset.issuer().is_none());
    }

    #[test]
    fn test_asset_alphanum4() {
        let issuer = vec![1, 2, 3, 4];
        let asset = StellarAsset::CreditAlphanum4 {
            code: [b'U', b'S', b'D', b'C'],
            issuer: issuer.clone(),
        };
        assert!(!asset.is_native());
        assert_eq!(asset.code_string(), "USDC");
        assert_eq!(asset.issuer(), Some(&issuer));
    }

    #[test]
    fn test_time_bounds_valid() {
        let tb = TimeBounds {
            min_time: 100,
            max_time: 200,
        };
        assert!(tb.is_valid());

        let tb_no_max = TimeBounds {
            min_time: 100,
            max_time: 0,
        };
        assert!(tb_no_max.is_valid());
    }

    #[test]
    fn test_time_bounds_invalid() {
        let tb = TimeBounds {
            min_time: 200,
            max_time: 100,
        };
        assert!(!tb.is_valid());
    }

    #[test]
    fn test_operation_type_names() {
        let op = StellarOperation::Payment {
            destination: vec![],
            asset: StellarAsset::Native,
            amount: 1000000,
        };
        assert_eq!(op.operation_type(), "Payment");
        assert!(op.is_transfer());
        assert!(!op.is_soroban());
    }

    #[test]
    fn test_transaction_validation() {
        let tx = StellarTransaction {
            source_account: vec![0; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: vec![StellarOperation::Payment {
                destination: vec![1; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            }],
            signatures: vec![DecoratedSignature {
                hint: [0, 0, 0, 0],
                signature: vec![0; 64],
            }],
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };
        assert!(tx.is_valid());
    }

    #[test]
    fn test_transaction_too_many_operations() {
        let ops = vec![
            StellarOperation::Payment {
                destination: vec![1; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            };
            101
        ];

        let tx = StellarTransaction {
            source_account: vec![0; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: ops,
            signatures: vec![DecoratedSignature {
                hint: [0, 0, 0, 0],
                signature: vec![0; 64],
            }],
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };
        assert!(!tx.is_valid());
    }
}
