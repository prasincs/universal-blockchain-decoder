/// Core TRON transaction types
use prost::Message;

/// TRON Transaction (top-level message)
#[derive(Clone, PartialEq, Message)]
pub struct Transaction {
    #[prost(message, optional, tag = "1")]
    pub raw_data: Option<RawTransaction>,

    #[prost(bytes = "vec", repeated, tag = "2")]
    pub signature: Vec<Vec<u8>>,

    #[prost(message, repeated, tag = "5")]
    pub ret: Vec<TransactionResult>,
}

/// Raw transaction data (unsigned)
#[derive(Clone, PartialEq, Message)]
pub struct RawTransaction {
    #[prost(bytes = "vec", tag = "1")]
    pub ref_block_bytes: Vec<u8>,

    #[prost(int64, tag = "3")]
    pub ref_block_num: i64,

    #[prost(bytes = "vec", tag = "4")]
    pub ref_block_hash: Vec<u8>,

    #[prost(int64, tag = "8")]
    pub expiration: i64,

    #[prost(bytes = "vec", tag = "10")]
    pub data: Vec<u8>,

    #[prost(message, repeated, tag = "11")]
    pub contract: Vec<Contract>,

    #[prost(int64, tag = "14")]
    pub timestamp: i64,

    #[prost(int64, tag = "18")]
    pub fee_limit: i64,
}

/// Contract wrapper
#[derive(Clone, PartialEq, Message)]
pub struct Contract {
    #[prost(enumeration = "ContractType", tag = "1")]
    pub r#type: i32,

    #[prost(message, optional, tag = "2")]
    pub parameter: Option<prost_types::Any>,

    #[prost(bytes = "vec", tag = "3")]
    pub provider: Vec<u8>,

    #[prost(int32, tag = "5")]
    pub permission_id: i32,
}

/// Contract types
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
#[derive(Debug)]
pub enum ContractType {
    AccountCreateContract = 0,
    TransferContract = 1,
    TransferAssetContract = 2,
    VoteAssetContract = 3,
    VoteWitnessContract = 4,
    WitnessCreateContract = 5,
    AssetIssueContract = 6,
    WitnessUpdateContract = 8,
    ParticipateAssetIssueContract = 9,
    AccountUpdateContract = 10,
    FreezeBalanceContract = 11,
    UnfreezeBalanceContract = 12,
    WithdrawBalanceContract = 13,
    UnfreezeAssetContract = 14,
    UpdateAssetContract = 15,
    ProposalCreateContract = 16,
    ProposalApproveContract = 17,
    ProposalDeleteContract = 18,
    SetAccountIdContract = 19,
    CustomContract = 20,
    CreateSmartContract = 30,
    TriggerSmartContract = 31,
    GetContract = 32,
    UpdateSettingContract = 33,
    ExchangeCreateContract = 41,
    ExchangeInjectContract = 42,
    ExchangeWithdrawContract = 43,
    ExchangeTransactionContract = 44,
    UpdateEnergyLimitContract = 45,
    AccountPermissionUpdateContract = 46,
    ClearAbiContract = 48,
    UpdateBrokerageContract = 49,
    ShieldedTransferContract = 51,
    MarketSellAssetContract = 52,
    MarketCancelOrderContract = 53,
    FreezeBalanceV2Contract = 54,
    UnfreezeBalanceV2Contract = 55,
    WithdrawExpireUnfreezeContract = 56,
    DelegateResourceContract = 57,
    UnDelegateResourceContract = 58,
    CancelAllUnfreezeV2Contract = 59,
}

/// Transaction result
#[derive(Clone, PartialEq, Message)]
pub struct TransactionResult {
    #[prost(int64, tag = "1")]
    pub fee: i64,

    #[prost(enumeration = "ResultCode", tag = "2")]
    pub ret: i32,

    #[prost(enumeration = "ContractResultCode", tag = "3")]
    pub contract_ret: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
#[derive(Debug)]
pub enum ResultCode {
    Success = 0,
    Failed = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
#[derive(Debug)]
pub enum ContractResultCode {
    Default = 0,
    Success = 1,
    Revert = 2,
    BadJumpDestination = 3,
    OutOfMemory = 4,
    PrecompiledContract = 5,
    StackTooSmall = 6,
    StackTooLarge = 7,
    IllegalOperation = 8,
    StackOverflow = 9,
    OutOfEnergy = 10,
    OutOfTime = 11,
    JvmStackOverFlow = 12,
    Unknown = 13,
    TransferFailed = 14,
}

// Specific contract parameter types

/// Transfer TRX
#[derive(Clone, PartialEq, Message)]
pub struct TransferContract {
    #[prost(bytes = "vec", tag = "1")]
    pub owner_address: Vec<u8>,

    #[prost(bytes = "vec", tag = "2")]
    pub to_address: Vec<u8>,

    #[prost(int64, tag = "3")]
    pub amount: i64,
}

/// Transfer TRC-10 token
#[derive(Clone, PartialEq, Message)]
pub struct TransferAssetContract {
    #[prost(bytes = "vec", tag = "1")]
    pub asset_name: Vec<u8>,

    #[prost(bytes = "vec", tag = "2")]
    pub owner_address: Vec<u8>,

    #[prost(bytes = "vec", tag = "3")]
    pub to_address: Vec<u8>,

    #[prost(int64, tag = "4")]
    pub amount: i64,
}

/// Trigger smart contract (TRC-20, etc.)
#[derive(Clone, PartialEq, Message)]
pub struct TriggerSmartContract {
    #[prost(bytes = "vec", tag = "1")]
    pub owner_address: Vec<u8>,

    #[prost(bytes = "vec", tag = "2")]
    pub contract_address: Vec<u8>,

    #[prost(int64, tag = "3")]
    pub call_value: i64,

    #[prost(bytes = "vec", tag = "4")]
    pub data: Vec<u8>,

    #[prost(int64, tag = "5")]
    pub call_token_value: i64,

    #[prost(int64, tag = "6")]
    pub token_id: i64,
}

/// Freeze balance for resources
#[derive(Clone, PartialEq, Message)]
pub struct FreezeBalanceContract {
    #[prost(bytes = "vec", tag = "1")]
    pub owner_address: Vec<u8>,

    #[prost(int64, tag = "2")]
    pub frozen_balance: i64,

    #[prost(int64, tag = "3")]
    pub frozen_duration: i64,

    #[prost(enumeration = "ResourceCode", tag = "10")]
    pub resource: i32,

    #[prost(bytes = "vec", tag = "15")]
    pub receiver_address: Vec<u8>,
}

/// Freeze balance V2
#[derive(Clone, PartialEq, Message)]
pub struct FreezeBalanceV2Contract {
    #[prost(bytes = "vec", tag = "1")]
    pub owner_address: Vec<u8>,

    #[prost(int64, tag = "2")]
    pub frozen_balance: i64,

    #[prost(enumeration = "ResourceCode", tag = "3")]
    pub resource: i32,
}

/// Unfreeze balance
#[derive(Clone, PartialEq, Message)]
pub struct UnfreezeBalanceContract {
    #[prost(bytes = "vec", tag = "1")]
    pub owner_address: Vec<u8>,

    #[prost(enumeration = "ResourceCode", tag = "10")]
    pub resource: i32,

    #[prost(bytes = "vec", tag = "15")]
    pub receiver_address: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
#[derive(Debug)]
pub enum ResourceCode {
    Bandwidth = 0,
    Energy = 1,
}
