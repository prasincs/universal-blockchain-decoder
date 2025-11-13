//! Example: Trait-Based Decoder Extension (Better than TOML)
//!
//! This shows how to add new chains WITHOUT specs - just configuration.
//! The code IS the spec, so it can't drift.

use std::marker::PhantomData;

// ============================================================================
// Core Generic UTXO Decoder (based on Bitcoin)
// ============================================================================

/// Configuration for UTXO-based chains
pub trait UtxoChainConfig: Send + Sync + 'static {
    const CHAIN_ID: u64;
    const CHAIN_NAME: &'static str;
    const HAS_SEGWIT: bool;

    /// Hash algorithm: DoubleSha256, Scrypt, etc.
    type HashAlgorithm: HashAlgorithm;

    /// Override parsing if needed (default works for most chains)
    fn parse_custom_input<R: std::io::Read>(_reader: &mut R) -> Option<CustomInput> {
        None // Most chains use default
    }
}

/// Generic UTXO decoder - works for ANY UtxoChainConfig
pub struct UtxoDecoder<C: UtxoChainConfig> {
    _phantom: PhantomData<C>,
}

impl<C: UtxoChainConfig> UtxoDecoder<C> {
    pub fn chain_id() -> u64 {
        C::CHAIN_ID
    }

    pub fn chain_name() -> &'static str {
        C::CHAIN_NAME
    }

    pub fn decode(raw_bytes: &[u8]) -> Result<Transaction, DecoderError> {
        // Generic parsing logic using C's configuration
        let tx = parse_utxo_transaction::<C>(raw_bytes)?;

        // Use configured hash algorithm
        let txid = C::HashAlgorithm::hash(&tx.raw_bytes);

        Ok(tx.with_txid(txid))
    }
}

// ============================================================================
// Bitcoin Configuration (Reference Implementation)
// ============================================================================

pub struct Bitcoin;

impl UtxoChainConfig for Bitcoin {
    const CHAIN_ID: u64 = 0;
    const CHAIN_NAME: &'static str = "Bitcoin";
    const HAS_SEGWIT: bool = true;

    type HashAlgorithm = DoubleSha256;

    // Uses all defaults - no custom parsing needed
}

pub type BitcoinDecoder = UtxoDecoder<Bitcoin>;

// ============================================================================
// Litecoin: Almost Identical to Bitcoin
// ============================================================================

pub struct Litecoin;

impl UtxoChainConfig for Litecoin {
    const CHAIN_ID: u64 = 2;
    const CHAIN_NAME: &'static str = "Litecoin";
    const HAS_SEGWIT: bool = true; // Same as Bitcoin

    type HashAlgorithm = DoubleSha256; // Same as Bitcoin
}

pub type LitecoinDecoder = UtxoDecoder<Litecoin>;

// That's it! 7 lines to add Litecoin. No code duplication.

// ============================================================================
// Dogecoin: No SegWit
// ============================================================================

pub struct Dogecoin;

impl UtxoChainConfig for Dogecoin {
    const CHAIN_ID: u64 = 3;
    const CHAIN_NAME: &'static str = "Dogecoin";
    const HAS_SEGWIT: bool = false; // Only difference!

    type HashAlgorithm = DoubleSha256;
}

pub type DogecoinDecoder = UtxoDecoder<Dogecoin>;

// ============================================================================
// Zcash: Custom Hash Algorithm
// ============================================================================

pub struct Zcash;

impl UtxoChainConfig for Zcash {
    const CHAIN_ID: u64 = 133;
    const CHAIN_NAME: &'static str = "Zcash";
    const HAS_SEGWIT: bool = false;

    type HashAlgorithm = Blake2b; // Different hash!

    // Could also override parse_custom_input for shielded transactions
}

pub type ZcashDecoder = UtxoDecoder<Zcash>;

// ============================================================================
// Why This Is Better Than TOML
// ============================================================================

/*
✅ Type-safe: Won't compile if wrong
✅ Can't drift: Config IS the code
✅ IDE support: Autocomplete, jump-to-definition
✅ Tested: Compiling IS validation
✅ Refactorable: Rename "CHAIN_ID" → updates all chains
✅ Versioned: Git tracks changes
✅ No parser: No TOML parsing errors
✅ Fast: Zero-cost abstraction (monomorphized at compile time)

❌ TOML approach:
   - Can drift from code
   - No type safety
   - No IDE support
   - Requires parser
   - Another file to maintain
*/

// ============================================================================
// Testing: Ensured by Type System
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_config() {
        assert_eq!(Bitcoin::CHAIN_ID, 0);
        assert_eq!(Bitcoin::CHAIN_NAME, "Bitcoin");
        assert!(Bitcoin::HAS_SEGWIT);
    }

    #[test]
    fn test_litecoin_decoder() {
        assert_eq!(LitecoinDecoder::chain_id(), 2);
        assert_eq!(LitecoinDecoder::chain_name(), "Litecoin");
    }

    #[test]
    fn test_dogecoin_no_segwit() {
        assert!(!Dogecoin::HAS_SEGWIT);
        // Compiler ensures HAS_SEGWIT is used correctly in parsing
    }

    // No spec sync tests needed - impossible to be out of sync!
}

// ============================================================================
// Dummy types for example
// ============================================================================

pub struct Transaction {
    raw_bytes: Vec<u8>,
}

impl Transaction {
    fn with_txid(self, _txid: Vec<u8>) -> Self {
        self
    }
}

pub struct CustomInput;

pub struct DecoderError;

pub trait HashAlgorithm {
    fn hash(data: &[u8]) -> Vec<u8>;
}

pub struct DoubleSha256;
impl HashAlgorithm for DoubleSha256 {
    fn hash(_data: &[u8]) -> Vec<u8> {
        // In real implementation, this would use sha2 crate:
        // let hash1 = Sha256::digest(data);
        // Sha256::digest(hash1).to_vec()
        vec![] // Placeholder for example
    }
}

pub struct Blake2b;
impl HashAlgorithm for Blake2b {
    fn hash(_data: &[u8]) -> Vec<u8> {
        // Blake2b implementation would go here
        vec![]
    }
}

#[allow(clippy::extra_unused_type_parameters)]
fn parse_utxo_transaction<C: UtxoChainConfig>(
    _raw_bytes: &[u8],
) -> Result<Transaction, DecoderError> {
    Ok(Transaction { raw_bytes: vec![] })
}

fn main() {
    // This is an example demonstrating trait-based decoder extension
    // See ARCHITECTURE.md for full documentation
    println!("Trait-based decoder approach example");
    println!("This pattern allows adding new chains without modifying core code");
}
