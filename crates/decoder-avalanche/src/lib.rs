//! Avalanche transaction decoder supporting all three chains
//!
//! This module provides decoders for all Avalanche chains:
//! - **X-Chain**: Exchange chain (AVM) for asset transfers using UTXO model
//! - **P-Chain**: Platform chain for staking and subnet management
//! - **C-Chain**: Contract chain (EVM-compatible) for smart contracts
//!
//! ## Architecture
//!
//! Avalanche consists of three primary blockchains:
//!
//! ### X-Chain (Exchange Chain)
//! - Instance of the Avalanche Virtual Machine (AVM)
//! - UTXO-based model for creating and trading assets
//! - Transaction types: BaseTx, CreateAssetTx, OperationTx, ImportTx, ExportTx
//! - Uses codec 0x0000 for serialization
//!
//! ### P-Chain (Platform Chain)
//! - Manages validators, subnets, and staking
//! - Transaction types: AddValidatorTx, AddDelegatorTx, CreateSubnetTx, etc.
//! - Also UTXO-based but specialized for platform operations
//! - Only AVAX asset is valid on P-Chain
//!
//! ### C-Chain (Contract Chain)
//! - EVM-compatible chain for smart contracts
//! - Uses identical format to Ethereum (RLP-encoded)
//! - Chain ID: 43114
//! - Supports all Ethereum transaction types (Legacy, EIP-2930, EIP-1559)
//!
//! ## Examples
//!
//! ### Decoding X-Chain transaction
//!
//! ```rust,ignore
//! use decoder_avalanche::xchain::*;
//! use decoder_primitives::prelude::*;
//!
//! let tx_bytes = hex::decode("0000...")?; // X-Chain transaction
//! let tx = XChainDecoder::decode(&tx_bytes)?;
//! ```
//!
//! ### Decoding P-Chain transaction
//!
//! ```rust,ignore
//! use decoder_avalanche::pchain::*;
//! use decoder_primitives::prelude::*;
//!
//! let tx_bytes = hex::decode("0000...")?; // P-Chain transaction
//! let tx = PChainDecoder::decode(&tx_bytes)?;
//! ```
//!
//! ### Decoding C-Chain transaction
//!
//! ```rust,ignore
//! use decoder_avalanche::cchain::*;
//! use decoder_primitives::prelude::*;
//!
//! let tx_bytes = hex::decode("f86c...")?; // C-Chain transaction (RLP)
//! let tx = CChainDecoder::decode(&tx_bytes)?;
//! ```
//!
//! ## References
//!
//! - [X-Chain Transaction Format](https://docs.avax.network/reference/avalanchego/x-chain/txn-format)
//! - [P-Chain Transaction Format](https://docs.avax.network/reference/avalanchego/p-chain/txn-format)
//! - [C-Chain API](https://docs.avax.network/reference/avalanchego/c-chain/api)

pub mod cchain;
pub mod common;
pub mod pchain;
pub mod xchain;

// Re-export commonly used types
pub use cchain::{CChain, CChainDecoder};
pub use common::*;
pub use pchain::{PChain, PChainDecoder, PChainTransaction};
pub use xchain::{XChain, XChainDecoder, XChainTransaction};

// Legacy exports for backward compatibility
pub use cchain::{CChain as AvalancheChain, CChainDecoder as AvalancheDecoder};

#[cfg(test)]
mod tests {
    use super::*;
    use decoder_primitives::prelude::*;

    #[test]
    fn test_all_chains_have_unique_names() {
        let x_chain = xchain::XChainDecoder::chain();
        let p_chain = pchain::PChainDecoder::chain();
        let c_chain = cchain::CChainDecoder::chain();

        assert_eq!(x_chain.chain_name(), "Avalanche-X");
        assert_eq!(p_chain.chain_name(), "Avalanche-P");
        assert_eq!(c_chain.chain_name(), "Avalanche-C");

        // All names should be unique
        assert_ne!(x_chain.chain_name(), p_chain.chain_name());
        assert_ne!(x_chain.chain_name(), c_chain.chain_name());
        assert_ne!(p_chain.chain_name(), c_chain.chain_name());
    }

    #[test]
    fn test_chain_families() {
        let x_chain = xchain::XChainDecoder::chain();
        let p_chain = pchain::PChainDecoder::chain();
        let c_chain = cchain::CChainDecoder::chain();

        // X-Chain uses UTXO model
        assert_eq!(x_chain.chain_family(), ChainFamily::Utxo);

        // P-Chain uses Account model for platform operations
        assert_eq!(p_chain.chain_family(), ChainFamily::Account);

        // C-Chain is EVM-compatible (Account model)
        assert_eq!(c_chain.chain_family(), ChainFamily::Account);
    }

    #[test]
    fn test_codec_id_constant() {
        // Validate that codec ID is correct
        assert_eq!(CODEC_ID, 0x0000);
    }

    #[test]
    fn test_type_ids() {
        // Validate transaction type IDs
        assert_eq!(BASE_TX, 0x00000000);
        assert_eq!(CREATE_ASSET_TX, 0x00000001);
        assert_eq!(OPERATION_TX, 0x00000002);
        assert_eq!(IMPORT_TX, 0x00000003);
        assert_eq!(EXPORT_TX, 0x00000004);
        assert_eq!(ADD_VALIDATOR_TX, 0x0000000c);
        assert_eq!(CREATE_SUBNET_TX, 0x00000010);
        assert_eq!(ADD_SUBNET_VALIDATOR_TX, 0x0000000d);
    }

    #[test]
    fn test_secp256k1_type_ids() {
        // Validate SECP256K1 type IDs
        assert_eq!(SECP256K1_TRANSFER_INPUT, 0x00000005);
        assert_eq!(SECP256K1_TRANSFER_OUTPUT, 0x00000007);
        assert_eq!(SECP256K1_MINT_OUTPUT, 0x00000006);
    }
}
