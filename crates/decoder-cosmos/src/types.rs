//! Cosmos transaction types (Protobuf structures)
//!
//! This module defines the Protobuf message structures for Cosmos SDK transactions.
//! These match the official Cosmos SDK protobuf definitions.

use prost::Message;
use prost_types::Any;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Cosmos transaction (Protobuf: cosmos.tx.v1beta1.Tx)
#[derive(Debug, Clone, PartialEq)]
pub struct Tx {
    /// Transaction body containing messages and metadata
    pub body: TxBody,
    /// Authentication information (fees, gas, signers)
    pub auth_info: AuthInfo,
    /// Signatures (one per signer)
    pub signatures: Vec<Vec<u8>>,
}

/// Transaction body (Protobuf: cosmos.tx.v1beta1.TxBody)
#[derive(Debug, Clone, PartialEq)]
pub struct TxBody {
    /// Messages to execute
    pub messages: Vec<Any>,
    /// Memo text
    pub memo: String,
    /// Optional timeout height
    pub timeout_height: u64,
    /// Extension options
    pub extension_options: Vec<Any>,
    /// Non-critical extension options
    pub non_critical_extension_options: Vec<Any>,
}

/// Authentication information (Protobuf: cosmos.tx.v1beta1.AuthInfo)
#[derive(Debug, Clone, PartialEq)]
pub struct AuthInfo {
    /// Signer information
    pub signer_infos: Vec<SignerInfo>,
    /// Transaction fee
    pub fee: Fee,
    /// Tip information (optional)
    pub tip: Option<Tip>,
}

/// Signer information (Protobuf: cosmos.tx.v1beta1.SignerInfo)
#[derive(Debug, Clone, PartialEq)]
pub struct SignerInfo {
    /// Public key
    pub public_key: Option<Any>,
    /// Mode info (single or multi-sig)
    pub mode_info: Option<ModeInfo>,
    /// Sequence number
    pub sequence: u64,
}

/// Mode info (Protobuf: cosmos.tx.v1beta1.ModeInfo)
#[derive(Debug, Clone, PartialEq)]
pub enum ModeInfo {
    /// Single signature mode
    Single(ModeInfoSingle),
    /// Multi-signature mode
    Multi(ModeInfoMulti),
}

/// Single signature mode info
#[derive(Debug, Clone, PartialEq)]
pub struct ModeInfoSingle {
    /// Signature mode (e.g., SIGN_MODE_DIRECT)
    pub mode: i32,
}

/// Multi-signature mode info
#[derive(Debug, Clone, PartialEq)]
pub struct ModeInfoMulti {
    /// Multisig bitarray
    pub bitarray: Option<CompactBitArray>,
    /// Mode infos for each signer
    pub mode_infos: Vec<ModeInfo>,
}

/// Compact bit array for multisig
#[derive(Debug, Clone, PartialEq)]
pub struct CompactBitArray {
    pub extra_bits_stored: u32,
    pub elems: Vec<u8>,
}

/// Transaction fee (Protobuf: cosmos.tx.v1beta1.Fee)
#[derive(Debug, Clone, PartialEq)]
pub struct Fee {
    /// Amount to pay for fees
    pub amount: Vec<Coin>,
    /// Gas limit
    pub gas_limit: u64,
    /// Payer address (if different from first signer)
    pub payer: String,
    /// Granter address (for fee grants)
    pub granter: String,
}

/// Tip information (Protobuf: cosmos.tx.v1beta1.Tip)
#[derive(Debug, Clone, PartialEq)]
pub struct Tip {
    /// Amount to tip
    pub amount: Vec<Coin>,
    /// Tipper address
    pub tipper: String,
}

/// Coin (Protobuf: cosmos.base.v1beta1.Coin)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coin {
    /// Denomination (e.g., "uatom")
    pub denom: String,
    /// Amount as string (decimal representation)
    pub amount: String,
}

/// Message types supported by the decoder
#[derive(Debug, Clone, PartialEq)]
pub enum CosmosMessage {
    /// Bank send message (cosmos.bank.v1beta1.MsgSend)
    Send(MsgSend),
    /// Bank multi-send message (cosmos.bank.v1beta1.MsgMultiSend)
    MultiSend(MsgMultiSend),
    /// Staking delegate message (cosmos.staking.v1beta1.MsgDelegate)
    Delegate(MsgDelegate),
    /// Staking undelegate message (cosmos.staking.v1beta1.MsgUndelegate)
    Undelegate(MsgUndelegate),
    /// Staking begin redelegate message (cosmos.staking.v1beta1.MsgBeginRedelegate)
    BeginRedelegate(MsgBeginRedelegate),
    /// IBC transfer message (ibc.applications.transfer.v1.MsgTransfer)
    IbcTransfer(MsgIbcTransfer),
    /// Governance vote message (cosmos.gov.v1beta1.MsgVote)
    Vote(MsgVote),
    /// Contract execute message (cosmwasm.wasm.v1.MsgExecuteContract)
    ExecuteContract(MsgExecuteContract),
    /// Unknown/unsupported message type
    Unknown { type_url: String, value: Vec<u8> },
}

/// MsgSend - Transfer tokens from one account to another
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgSend {
    pub from_address: String,
    pub to_address: String,
    pub amount: Vec<Coin>,
}

/// MsgMultiSend - Transfer tokens from multiple accounts to multiple accounts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgMultiSend {
    pub inputs: Vec<MultiSendInput>,
    pub outputs: Vec<MultiSendOutput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiSendInput {
    pub address: String,
    pub coins: Vec<Coin>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiSendOutput {
    pub address: String,
    pub coins: Vec<Coin>,
}

/// MsgDelegate - Delegate tokens to a validator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgDelegate {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: Coin,
}

/// MsgUndelegate - Undelegate tokens from a validator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgUndelegate {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: Coin,
}

/// MsgBeginRedelegate - Redelegate tokens from one validator to another
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgBeginRedelegate {
    pub delegator_address: String,
    pub validator_src_address: String,
    pub validator_dst_address: String,
    pub amount: Coin,
}

/// MsgIbcTransfer - IBC transfer message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgIbcTransfer {
    pub source_port: String,
    pub source_channel: String,
    pub token: Coin,
    pub sender: String,
    pub receiver: String,
    pub timeout_height: Option<IbcHeight>,
    pub timeout_timestamp: u64,
    pub memo: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IbcHeight {
    pub revision_number: u64,
    pub revision_height: u64,
}

/// MsgVote - Vote on a governance proposal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgVote {
    pub proposal_id: u64,
    pub voter: String,
    pub option: i32,
}

/// MsgExecuteContract - Execute a CosmWasm contract
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgExecuteContract {
    pub sender: String,
    pub contract: String,
    pub msg: Vec<u8>,
    pub funds: Vec<Coin>,
}

impl fmt::Display for CosmosMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CosmosMessage::Send(msg) => {
                write!(f, "MsgSend: {} -> {}", msg.from_address, msg.to_address)
            }
            CosmosMessage::MultiSend(_) => write!(f, "MsgMultiSend"),
            CosmosMessage::Delegate(msg) => {
                write!(
                    f,
                    "MsgDelegate: {} to {}",
                    msg.delegator_address, msg.validator_address
                )
            }
            CosmosMessage::Undelegate(msg) => {
                write!(
                    f,
                    "MsgUndelegate: {} from {}",
                    msg.delegator_address, msg.validator_address
                )
            }
            CosmosMessage::BeginRedelegate(msg) => {
                write!(
                    f,
                    "MsgBeginRedelegate: {} {} -> {}",
                    msg.delegator_address, msg.validator_src_address, msg.validator_dst_address
                )
            }
            CosmosMessage::IbcTransfer(msg) => {
                write!(
                    f,
                    "MsgIbcTransfer: {} -> {} (channel: {})",
                    msg.sender, msg.receiver, msg.source_channel
                )
            }
            CosmosMessage::Vote(msg) => {
                write!(f, "MsgVote: {} on proposal {}", msg.voter, msg.proposal_id)
            }
            CosmosMessage::ExecuteContract(msg) => {
                write!(f, "MsgExecuteContract: {} on {}", msg.sender, msg.contract)
            }
            CosmosMessage::Unknown { type_url, .. } => {
                write!(f, "Unknown: {}", type_url)
            }
        }
    }
}

/// Known message type URLs
pub mod type_urls {
    pub const MSG_SEND: &str = "/cosmos.bank.v1beta1.MsgSend";
    pub const MSG_MULTI_SEND: &str = "/cosmos.bank.v1beta1.MsgMultiSend";
    pub const MSG_DELEGATE: &str = "/cosmos.staking.v1beta1.MsgDelegate";
    pub const MSG_UNDELEGATE: &str = "/cosmos.staking.v1beta1.MsgUndelegate";
    pub const MSG_BEGIN_REDELEGATE: &str = "/cosmos.staking.v1beta1.MsgBeginRedelegate";
    pub const MSG_IBC_TRANSFER: &str = "/ibc.applications.transfer.v1.MsgTransfer";
    pub const MSG_VOTE: &str = "/cosmos.gov.v1beta1.MsgVote";
    pub const MSG_EXECUTE_CONTRACT: &str = "/cosmwasm.wasm.v1.MsgExecuteContract";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coin_creation() {
        let coin = Coin {
            denom: "uatom".to_string(),
            amount: "1000000".to_string(),
        };
        assert_eq!(coin.denom, "uatom");
        assert_eq!(coin.amount, "1000000");
    }

    #[test]
    fn test_message_display() {
        let msg = CosmosMessage::Send(MsgSend {
            from_address: "cosmos1xxx".to_string(),
            to_address: "cosmos1yyy".to_string(),
            amount: vec![],
        });
        assert!(format!("{}", msg).contains("MsgSend"));
    }
}
