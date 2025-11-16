//! Common types and utilities shared across all Avalanche chains
//!
//! This module defines structures and functions used by X-Chain, P-Chain, and C-Chain decoders.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Codec ID used for transaction serialization
/// Currently only 0x0000 is valid
pub const CODEC_ID: u16 = 0x0000;

/// TypeID for secp256k1 transfer input
pub const SECP256K1_TRANSFER_INPUT: u32 = 0x00000005;

/// TypeID for secp256k1 transfer output
pub const SECP256K1_TRANSFER_OUTPUT: u32 = 0x00000007;

/// TypeID for secp256k1 mint output
pub const SECP256K1_MINT_OUTPUT: u32 = 0x00000006;

/// TypeID for BaseTx
pub const BASE_TX: u32 = 0x00000000;

/// TypeID for CreateAssetTx
pub const CREATE_ASSET_TX: u32 = 0x00000001;

/// TypeID for OperationTx
pub const OPERATION_TX: u32 = 0x00000002;

/// TypeID for ImportTx
pub const IMPORT_TX: u32 = 0x00000003;

/// TypeID for ExportTx
pub const EXPORT_TX: u32 = 0x00000004;

/// TypeID for AddValidatorTx (P-Chain)
pub const ADD_VALIDATOR_TX: u32 = 0x0000000c;

/// TypeID for CreateSubnetTx (P-Chain)
pub const CREATE_SUBNET_TX: u32 = 0x00000010;

/// TypeID for AddSubnetValidatorTx (P-Chain)
pub const ADD_SUBNET_VALIDATOR_TX: u32 = 0x0000000d;

/// A transferable input references a UTXO and includes authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TransferableInput {
    /// Transaction ID that created the UTXO
    pub tx_id: [u8; 32],

    /// Index of the UTXO in the creating transaction
    pub utxo_index: u32,

    /// Asset ID
    pub asset_id: [u8; 32],

    /// Input authorization
    pub input: Input,
}

/// Input authorization for spending a UTXO
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Input {
    /// SECP256K1 transfer input
    Secp256k1Transfer {
        /// Amount to transfer
        amount: u64,
        /// Signature indices
        address_indices: Vec<u32>,
    },

    /// Other input types (placeholder for future support)
    Unknown { type_id: u32, data: Vec<u8> },
}

/// A transferable output creates a new UTXO
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TransferableOutput {
    /// Asset ID
    pub asset_id: [u8; 32],

    /// Output specification
    pub output: Output,
}

/// Output specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Output {
    /// SECP256K1 transfer output
    Secp256k1Transfer {
        /// Amount to transfer
        amount: u64,
        /// Locktime (Unix timestamp)
        locktime: u64,
        /// Threshold for signatures
        threshold: u32,
        /// Addresses that can spend this output
        addresses: Vec<[u8; 20]>,
    },

    /// SECP256K1 mint output
    Secp256k1Mint {
        /// Locktime (Unix timestamp)
        locktime: u64,
        /// Threshold for signatures
        threshold: u32,
        /// Addresses that can mint
        addresses: Vec<[u8; 20]>,
    },

    /// Other output types (placeholder for future support)
    Unknown { type_id: u32, data: Vec<u8> },
}

/// Base transaction fields common to all transaction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct BaseTx {
    /// Network ID (1 for mainnet, 5 for fuji testnet)
    pub network_id: u32,

    /// Blockchain ID
    pub blockchain_id: [u8; 32],

    /// Outputs created by this transaction
    pub outputs: Vec<TransferableOutput>,

    /// Inputs consumed by this transaction
    pub inputs: Vec<TransferableInput>,

    /// Optional memo field
    pub memo: Vec<u8>,
}

/// Validator information for P-Chain staking transactions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Validator {
    /// Node ID
    pub node_id: [u8; 20],

    /// Start time (Unix timestamp)
    pub start_time: u64,

    /// End time (Unix timestamp)
    pub end_time: u64,

    /// Weight (stake amount)
    pub weight: u64,
}

/// Stake information for P-Chain transactions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct StakeOutput {
    /// Locktime
    pub locktime: u64,

    /// Threshold
    pub threshold: u32,

    /// Addresses
    pub addresses: Vec<[u8; 20]>,
}
