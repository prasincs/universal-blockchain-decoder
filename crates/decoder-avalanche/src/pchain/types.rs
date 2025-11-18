//! P-Chain transaction types

use crate::common::*;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// P-Chain transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PChainTransaction {
    /// Codec ID (should be 0x0000)
    pub codec_id: u16,

    /// Transaction type
    pub tx_type: PChainTxType,

    /// Raw transaction bytes (for re-encoding)
    pub raw_bytes: Vec<u8>,
}

/// P-Chain transaction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum PChainTxType {
    /// Base transaction (Type 0x00000000)
    Base(BaseTx),

    /// Add validator transaction (Type 0x0000000c)
    AddValidator {
        /// Base transaction fields
        base: BaseTx,
        /// Validator information
        validator: Validator,
        /// Stake outputs
        stake: Vec<TransferableOutput>,
        /// Rewards owner
        rewards_owner: RewardsOwner,
        /// Shares (delegation fee percentage, in basis points)
        shares: u32,
    },

    /// Add delegator transaction (Type 0x0000000e)
    AddDelegator {
        /// Base transaction fields
        base: BaseTx,
        /// Validator information
        validator: Validator,
        /// Stake outputs
        stake: Vec<TransferableOutput>,
        /// Rewards owner
        rewards_owner: RewardsOwner,
    },

    /// Create subnet transaction (Type 0x00000010)
    CreateSubnet {
        /// Base transaction fields
        base: BaseTx,
        /// Subnet owner
        owner: SubnetOwner,
    },

    /// Add subnet validator transaction (Type 0x0000000d)
    AddSubnetValidator {
        /// Base transaction fields
        base: BaseTx,
        /// Validator information
        validator: Validator,
        /// Subnet ID
        subnet_id: [u8; 32],
    },

    /// Import transaction (Type 0x00000011)
    Import {
        /// Base transaction fields
        base: BaseTx,
        /// Source chain ID
        source_chain: [u8; 32],
        /// Imported inputs
        imported_inputs: Vec<TransferableInput>,
    },

    /// Export transaction (Type 0x00000012)
    Export {
        /// Base transaction fields
        base: BaseTx,
        /// Destination chain ID
        destination_chain: [u8; 32],
        /// Exported outputs
        exported_outputs: Vec<TransferableOutput>,
    },

    /// Remove subnet validator transaction (Type 0x00000015)
    RemoveSubnetValidator {
        /// Base transaction fields
        base: BaseTx,
        /// Node ID to remove
        node_id: [u8; 20],
        /// Subnet ID
        subnet_id: [u8; 32],
    },

    /// Transform subnet transaction (Type 0x00000016)
    TransformSubnet {
        /// Base transaction fields
        base: BaseTx,
        /// Subnet ID
        subnet_id: [u8; 32],
        /// Asset ID
        asset_id: [u8; 32],
        /// Initial supply
        initial_supply: u64,
        /// Maximum supply
        maximum_supply: u64,
        /// Min consumption rate (basis points)
        min_consumption_rate: u64,
        /// Max consumption rate (basis points)
        max_consumption_rate: u64,
        /// Min validator stake
        min_validator_stake: u64,
        /// Max validator stake
        max_validator_stake: u64,
        /// Min stake duration
        min_stake_duration: u32,
        /// Max stake duration
        max_stake_duration: u32,
        /// Min delegation fee (basis points)
        min_delegation_fee: u32,
        /// Min delegator stake
        min_delegator_stake: u64,
        /// Max validator weight factor
        max_validator_weight_factor: u8,
        /// Uptime requirement (percentage)
        uptime_requirement: u32,
    },

    /// Add permissionless validator transaction (Type 0x00000019)
    AddPermissionlessValidator {
        /// Base transaction fields
        base: BaseTx,
        /// Validator information
        validator: Validator,
        /// Subnet ID
        subnet_id: [u8; 32],
        /// Signer (BLS proof of possession)
        signer: Option<Vec<u8>>,
        /// Stake outputs
        stake: Vec<TransferableOutput>,
        /// Validator rewards owner
        validator_rewards_owner: RewardsOwner,
        /// Delegator rewards owner
        delegator_rewards_owner: RewardsOwner,
        /// Delegation shares (basis points)
        delegation_shares: u32,
    },

    /// Add permissionless delegator transaction (Type 0x0000001a)
    AddPermissionlessDelegator {
        /// Base transaction fields
        base: BaseTx,
        /// Validator information
        validator: Validator,
        /// Subnet ID
        subnet_id: [u8; 32],
        /// Stake outputs
        stake: Vec<TransferableOutput>,
        /// Rewards owner
        rewards_owner: RewardsOwner,
    },

    /// Unknown transaction type
    Unknown { type_id: u32, data: Vec<u8> },
}

/// Rewards owner
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RewardsOwner {
    /// Locktime
    pub locktime: u64,

    /// Threshold
    pub threshold: u32,

    /// Addresses
    pub addresses: Vec<[u8; 20]>,
}

/// Subnet owner
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubnetOwner {
    /// Locktime
    pub locktime: u64,

    /// Threshold
    pub threshold: u32,

    /// Addresses
    pub addresses: Vec<[u8; 20]>,
}
