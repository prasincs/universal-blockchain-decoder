//! X-Chain transaction types

use crate::common::*;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// X-Chain transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct XChainTransaction {
    /// Codec ID (should be 0x0000)
    pub codec_id: u16,

    /// Transaction type
    pub tx_type: XChainTxType,

    /// Raw transaction bytes (for re-encoding)
    pub raw_bytes: Vec<u8>,
}

/// X-Chain transaction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum XChainTxType {
    /// Base transaction (Type 0x00000000)
    Base(BaseTx),

    /// Create asset transaction (Type 0x00000001)
    CreateAsset {
        /// Base transaction fields
        base: BaseTx,
        /// Asset name
        name: String,
        /// Asset symbol
        symbol: String,
        /// Denomination
        denomination: u8,
        /// Initial states
        initial_states: Vec<InitialState>,
    },

    /// Operation transaction (Type 0x00000002)
    Operation {
        /// Base transaction fields
        base: BaseTx,
        /// Operations
        operations: Vec<XChainOperation>,
    },

    /// Import transaction (Type 0x00000003)
    Import {
        /// Base transaction fields
        base: BaseTx,
        /// Source chain ID
        source_chain: [u8; 32],
        /// Imported inputs
        imported_inputs: Vec<TransferableInput>,
    },

    /// Export transaction (Type 0x00000004)
    Export {
        /// Base transaction fields
        base: BaseTx,
        /// Destination chain ID
        destination_chain: [u8; 32],
        /// Exported outputs
        exported_outputs: Vec<TransferableOutput>,
    },

    /// Unknown transaction type
    Unknown { type_id: u32, data: Vec<u8> },
}

/// Initial state for asset creation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct InitialState {
    /// Feature extension ID
    pub fx_id: u32,

    /// Initial outputs
    pub outputs: Vec<Output>,
}

/// Operation for OperationTx
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct XChainOperation {
    /// Asset ID
    pub asset_id: [u8; 32],

    /// UTXOs to operate on
    pub utxo_ids: Vec<UtxoId>,

    /// Operation data
    pub operation_data: Vec<u8>,
}

/// UTXO identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UtxoId {
    /// Transaction ID
    pub tx_id: [u8; 32],

    /// Output index
    pub output_index: u32,
}
