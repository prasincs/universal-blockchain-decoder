//! # Universal Blockchain Transaction Decoder
//!
//! A compile-time safe, universal transaction decoder architecture for heterogeneous
//! blockchains, leveraging canonical intermediate representations in Rust.
//!
//! ## Overview
//!
//! This library provides a modular, type-safe framework for decoding transactions
//! from different blockchain protocols (Bitcoin, Ethereum, Solana, etc.) into a
//! unified canonical intermediate representation (TxIR).
//!
//! ## Architecture
//!
//! The decoder follows a three-layer pipeline:
//!
//! 1. **Input Aggregation**: Raw byte slice handling
//! 2. **Chain-Specific Decoding**: Protocol-specific parsing (via `ChainDecoder` trait)
//! 3. **Canonicalization**: Transformation into universal TxIR (via `Canonicalizer` trait)
//!
//! ## Key Features
//!
//! - **Compile-Time Safety**: Uses const generics and associated types for type-level guarantees
//! - **Zero-Cost Abstractions**: Static dispatch via monomorphization
//! - **Extensible**: Hook system for custom processing at various pipeline stages
//! - **Non-Malleable**: Canonical representation ensures deterministic hashing
//! - **Formally Verifiable**: Designed for integration with tools like Prusti and Verus
//!
//! ## Example
//!
//! ```ignore
//! use universal_decoder_core::prelude::*;
//!
//! // Define a chain-specific decoder
//! struct MyChainDecoder;
//!
//! impl ChainDecoder for MyChainDecoder {
//!     type TxSpecific = MyChainTransaction;
//!
//!     fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
//!         // Parse chain-specific format
//!         todo!()
//!     }
//!
//!     fn chain_id() -> ChainId {
//!         ChainId::Custom(1)
//!     }
//! }
//!
//! // Use the decoder
//! let raw_tx = &[0x01, 0x02, 0x03];
//! let tx = MyChainDecoder::decode(raw_tx)?;
//! let canonical = tx.canonicalize()?;
//! ```

pub mod canonical;
pub mod chain;
pub mod error;
pub mod hooks;
pub mod ir;
pub mod traits;

// Re-export commonly used types
pub mod prelude {
    pub use crate::canonical::{CanonicalSerialize, CanonicalTxIR};
    pub use crate::chain::{ChainFamily, ChainIdentity, ChainRef};
    pub use crate::error::{DecoderError, Result};
    pub use crate::hooks::{
        Hook, HookContext, HookRegistry, HookRegistryBuilder, HookResult, HookStage,
    };
    pub use crate::ir::{
        AccountChange, Address, Amount, AssetId, AuthorizationPackage, ContractCall,
        ContractDeploy, GenericOperation, InputReference, KeyType, Operation, OutputValue,
        PublicKey, ResourceLimits, ResourceType, Signature, SignatureScheme, Stake,
        StakeOperationType, StateDeltas, StorageChange, Transfer, TxIR, TxMetadata,
    };
    pub use crate::traits::{
        BatchDecoder, Canonicalizer, ChainDecoder, DecoderPlugin, DoubleSha256, FormallyVerifiable,
        HashAlgorithm, Keccak256Hash, Sha256Hash, TxHashable, TxVerifier, TxVersion,
    };
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if the library was built with formal verification support
pub const FORMAL_VERIFICATION_ENABLED: bool = cfg!(feature = "formal-verification");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
