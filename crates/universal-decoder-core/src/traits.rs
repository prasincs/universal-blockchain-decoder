//! Core trait definitions for the universal blockchain decoder.
//!
//! This module defines the trait hierarchy that enables modular, compile-time safe
//! decoding of blockchain transactions through static dispatch.

use crate::chain::ChainIdentity;
use crate::error::{DecoderError, Result};
use crate::ir::TxIR;

/// The main trait for chain-specific transaction decoding.
///
/// This trait is the entry point for decoding raw transaction bytes from a specific
/// blockchain into a chain-specific structured type. The associated type `TxSpecific`
/// must implement the `Canonicalizer` trait to enable transformation into the universal TxIR,
/// and must also implement `ChainEncoder` to support re-encoding back to original bytes.
///
/// # Type Parameters
///
/// - `TxSpecific`: The chain-specific transaction type that can be canonicalized and re-encoded
/// - `Chain`: The chain identity type implementing `ChainIdentity`
///
/// # Example
///
/// ```ignore
/// struct BitcoinChain;
/// impl ChainIdentity for BitcoinChain {
///     fn chain_id(&self) -> u64 { 0 }
///     fn chain_name(&self) -> &str { "Bitcoin" }
///     fn chain_family(&self) -> ChainFamily { ChainFamily::Utxo }
/// }
///
/// struct BitcoinDecoder;
/// impl ChainDecoder for BitcoinDecoder {
///     type TxSpecific = BitcoinTransaction;
///     type Chain = BitcoinChain;
///
///     fn chain() -> Self::Chain { BitcoinChain }
///
///     fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {
///         // Parse Bitcoin transaction format
///         todo!()
///     }
/// }
/// ```
pub trait ChainDecoder {
    /// The chain-specific transaction type that implements Canonicalizer and ChainEncoder
    type TxSpecific: for<'a> Canonicalizer<'a> + ChainEncoder;

    /// The chain identity type
    type Chain: ChainIdentity;

    /// Get the chain identity for this decoder
    fn chain() -> Self::Chain;

    /// Decode raw transaction bytes into the chain-specific structure
    ///
    /// # Arguments
    ///
    /// * `raw_bytes` - Raw transaction bytes in the chain's native format
    ///
    /// # Returns
    ///
    /// * `Result<Self::TxSpecific>` - The decoded chain-specific transaction or an error
    ///
    /// # Formal Properties
    ///
    /// This method must satisfy the injective property (roundtrip):
    /// ```text
    /// ∀ tx_bytes: Self::decode(tx_bytes)?.to_bytes() == tx_bytes
    /// ```
    ///
    /// This property is verified through property-based testing.
    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific>;

    /// Optional: Validate the raw bytes before decoding
    ///
    /// This can be used for quick rejection of obviously invalid data
    fn validate_format(raw_bytes: &[u8]) -> Result<()> {
        // Default implementation: accept all
        let _ = raw_bytes;
        Ok(())
    }
}

/// Trait for transforming chain-specific transactions into the canonical TxIR.
///
/// This trait handles the semantic mapping from chain-specific structures
/// (e.g., UTXO sets, account state transitions, instruction lists) into
/// the unified intermediate representation.
///
/// # Lifetime Parameters
///
/// - `'a`: The lifetime of the source data, ensuring proper lifetime tracking
pub trait Canonicalizer<'a> {
    /// The transaction version supported by this canonicalizer
    const VERSION: u8;

    /// Transform the chain-specific transaction into canonical TxIR
    ///
    /// This method performs the semantic transformation, mapping chain-specific
    /// concepts into the universal representation.
    ///
    /// # Returns
    ///
    /// * `Result<TxIR<'a, 1>>` - The canonical intermediate representation or an error
    ///
    /// Note: Due to Rust const generic limitations, we use version 1 as the base type.
    /// The actual version is available via the VERSION constant.
    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>>;

    /// Validate the transaction structure before canonicalization
    ///
    /// This can include checking for semantic invariants specific to the chain
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Trait for re-encoding chain-specific transactions back to their original byte format.
///
/// **CRITICAL REQUIREMENT**: This trait enables verification of the injective property.
/// Every chain decoder MUST implement this trait to support roundtrip verification.
///
/// # Purpose
///
/// This trait is used for:
/// - ✅ Verifying lossless decoding (roundtrip property)
/// - ✅ Forensic reconstruction of exact original bytes
/// - ✅ Formal verification of codec correctness
/// - ✅ Integrity checks and auditing
///
/// This is **NOT** for transaction construction (building new transactions from scratch).
///
/// # Formal Properties
///
/// Implementations MUST satisfy the injective property:
/// ```text
/// ∀ tx_bytes: ChainDecoder::decode(tx_bytes)?.to_bytes() == tx_bytes
/// ```
///
/// This property is verified through mandatory property-based testing.
///
/// # Example
///
/// ```ignore
/// use universal_decoder_core::prelude::*;
///
/// // Decode transaction
/// let decoded_tx = BitcoinDecoder::decode(original_bytes)?;
///
/// // Re-encode back to original format
/// let re_encoded = decoded_tx.to_bytes()?;
///
/// // Verify roundtrip (injective property)
/// assert_eq!(original_bytes, re_encoded);
/// ```
///
/// # Implementation Requirements
///
/// 1. **Determinism**: `to_bytes()` must always produce the same output for the same input
/// 2. **Exact Reconstruction**: Must produce byte-for-byte identical output to original input
/// 3. **No External Dependencies**: Must not require chain state, network calls, or external data
/// 4. **Panic-Freedom**: Must return `Result::Err` on failure, never panic
///
/// # Example Implementation
///
/// ```ignore
/// impl ChainEncoder for BitcoinTransaction {
///     fn to_bytes(&self) -> Result<Vec<u8>> {
///         // For structures that store original bytes, simply return them
///         Ok(self.raw_bytes.clone())
///
///         // For structures that don't store original bytes,
///         // reconstruct by serializing each field
///         // (this is more complex but still stateless and deterministic)
///     }
/// }
/// ```
pub trait ChainEncoder {
    /// Re-encode the transaction back to its original chain-specific byte format
    ///
    /// This method MUST produce the exact same bytes that were originally decoded.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>>` - The re-encoded transaction bytes, or an error if encoding fails
    ///
    /// # Errors
    ///
    /// Should return an error if:
    /// - The transaction structure is invalid (though this should be caught during decode)
    /// - Internal data is inconsistent
    ///
    /// # Formal Guarantees (Verus)
    ///
    /// ```text
    /// ensures(result.is_ok() ==> decode(result.unwrap())? == self)
    /// ensures(forall |tx_bytes| decode(tx_bytes)?.to_bytes()? == tx_bytes)
    /// ```
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

/// Trait for computing canonical byte representation of transactions.
///
/// This trait ensures non-malleability by providing a deterministic byte
/// representation that can be used for hashing and signature verification.
///
/// The implementation must guarantee: encode(decode(bytes)) == bytes
/// (injectivity of the canonicalization function)
pub trait TxHashable {
    /// Compute the canonical byte representation
    ///
    /// This must be deterministic and bijective with the transaction structure.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The canonical byte representation
    fn to_canonical_bytes(&self) -> Vec<u8>;

    /// Compute the transaction hash
    ///
    /// Default implementation uses SHA-256 of canonical bytes
    fn compute_hash(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let bytes = self.to_canonical_bytes();
        Sha256::digest(&bytes).to_vec()
    }

    /// Compute a hash using a specific algorithm
    fn compute_hash_with<H: HashAlgorithm>(&self) -> Vec<u8> {
        H::hash(&self.to_canonical_bytes())
    }
}

/// Trait for hash algorithms used in transaction hashing
pub trait HashAlgorithm {
    /// Hash the input bytes
    fn hash(data: &[u8]) -> Vec<u8>;

    /// Get the name of the hash algorithm
    fn name() -> &'static str;
}

/// SHA-256 hash algorithm
pub struct Sha256Hash;

impl HashAlgorithm for Sha256Hash {
    fn hash(data: &[u8]) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        Sha256::digest(data).to_vec()
    }

    fn name() -> &'static str {
        "SHA-256"
    }
}

/// SHA-256 double hash (used by Bitcoin)
pub struct DoubleSha256;

impl HashAlgorithm for DoubleSha256 {
    fn hash(data: &[u8]) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let first = Sha256::digest(data);
        Sha256::digest(first).to_vec()
    }

    fn name() -> &'static str {
        "Double SHA-256"
    }
}

/// Keccak-256 hash (used by Ethereum)
pub struct Keccak256Hash;

impl HashAlgorithm for Keccak256Hash {
    fn hash(data: &[u8]) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        Keccak256::digest(data).to_vec()
    }

    fn name() -> &'static str {
        "Keccak-256"
    }
}

/// Trait for transaction verification
pub trait TxVerifier {
    /// Verify the transaction's signatures
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction IR to verify
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - True if all signatures are valid, error otherwise
    fn verify_signatures<'a, const V: u8>(tx: &TxIR<'a, V>) -> Result<bool>;

    /// Verify structural invariants
    fn verify_structure<'a, const V: u8>(tx: &TxIR<'a, V>) -> Result<bool> {
        // Default implementation checks basic structural properties
        if tx.authorization.signatures.len() != tx.authorization.public_keys.len() {
            return Err(DecoderError::invalid_structure(
                "Signature count must match public key count",
            ));
        }
        Ok(true)
    }
}

/// Marker trait for transactions that support formal verification
///
/// Types implementing this trait have been designed with formal verification in mind
/// and provide annotations/specifications for verification tools like Prusti or Verus.
pub trait FormallyVerifiable {
    /// Check if the type has formal verification annotations
    fn has_formal_specs() -> bool {
        false
    }

    /// Get the verification tool used (if any)
    fn verification_tool() -> Option<&'static str> {
        None
    }
}

/// Type-level marker for transaction versions
///
/// This allows compile-time distinction between different transaction versions
pub struct TxVersion<const V: u8>;

impl<const V: u8> TxVersion<V> {
    pub const VERSION: u8 = V;
}

/// Trait for extensible decoding with plugin support
///
/// This enables the hook system where custom decoders can be registered
pub trait DecoderPlugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Check if this plugin can handle the given transaction bytes
    fn can_handle(&self, raw_bytes: &[u8]) -> bool;

    /// Decode using this plugin
    fn decode_with_plugin<'a>(&self, raw_bytes: &'a [u8]) -> Result<Box<dyn std::any::Any + 'a>>;
}

/// Trait for batch decoding optimization
pub trait BatchDecoder: ChainDecoder {
    /// Decode multiple transactions in a batch
    ///
    /// This can enable optimizations like parallel processing or shared context
    fn decode_batch(transactions: &[&[u8]]) -> Result<Vec<Self::TxSpecific>> {
        // Default implementation: decode sequentially
        transactions
            .iter()
            .map(|tx_bytes| Self::decode(tx_bytes))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_algorithms() {
        let data = b"hello world";

        let sha256 = Sha256Hash::hash(data);
        assert_eq!(sha256.len(), 32);

        let double_sha256 = DoubleSha256::hash(data);
        assert_eq!(double_sha256.len(), 32);

        let keccak = Keccak256Hash::hash(data);
        assert_eq!(keccak.len(), 32);

        // SHA-256 and double SHA-256 should produce different results
        assert_ne!(sha256, double_sha256);
    }

    #[test]
    fn test_tx_version_marker() {
        assert_eq!(TxVersion::<1>::VERSION, 1);
        assert_eq!(TxVersion::<2>::VERSION, 2);
    }
}
