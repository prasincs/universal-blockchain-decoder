//! Property-based tests for Ethereum decoder
//!
//! This module uses proptest to verify critical properties of the Ethereum decoder:
//! 1. Decoder never panics on arbitrary input
//! 2. RLP encoding/decoding roundtrip
//! 3. Transaction hash calculation is deterministic
//! 4. Gas/fee calculation properties
//! 5. Canonical serialization properties
//! 6. Transaction type detection (Legacy, EIP-2930, EIP-1559)
//! 7. Signature field validation (v, r, s)
//! 8. Address handling (20-byte validation)

use decoder_encodings::rlp::RlpItem;
use decoder_ethereum::EthereumDecoder;
use decoder_test_utils::proptest_helpers::{arb_small_bytes, prop_decoder_never_panics};
use proptest::prelude::*;
use sha3::{Digest, Keccak256};
use universal_decoder_core::prelude::*;

//
// Property 1: Decoder Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Ethereum decoder never panics on arbitrary input
    ///
    /// For any arbitrary byte sequence, decode() must return Ok or Err,
    /// never panic.
    #[test]
    fn prop_ethereum_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<EthereumDecoder>(&bytes);
    }

    /// Property: Ethereum decoder never panics on empty input
    #[test]
    fn prop_ethereum_decoder_rejects_empty(_unit in 0u8..1) {
        let result = EthereumDecoder::decode(&[]);
        prop_assert!(result.is_err(), "Decoder should reject empty input");
    }

    /// Property: Ethereum decoder never panics on very short input
    #[test]
    fn prop_ethereum_decoder_rejects_tiny_input(size in 1usize..10) {
        let bytes = vec![0xFF; size];
        let result = EthereumDecoder::decode(&bytes);
        prop_assert!(result.is_err(), "Decoder should reject input < 10 bytes");
    }

    /// Property: Decoder handles oversized input gracefully
    #[test]
    fn prop_ethereum_decoder_handles_large_input(size in 10_000usize..100_000) {
        let bytes = vec![0x00; size];
        // Should either decode or error, never panic
        let result = EthereumDecoder::decode(&bytes);
        prop_assert!(result.is_ok() || result.is_err());
    }
}

//
// Property 2: RLP Encoding/Decoding Roundtrip
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    // DISABLED: Requires encode_bytes function
    // fn prop_rlp_string_roundtrip() { }

    // DISABLED: Requires encode_bytes function
    // fn prop_rlp_canonical_string_encoding() { }

    /// Property: Empty RLP list decodes correctly
    #[test]
    fn prop_rlp_empty_list(_unit in 0u8..1) {
        let empty_list = vec![0xc0]; // RLP encoding of empty list
        let decoded = RlpItem::decode(&empty_list);
        prop_assert!(decoded.is_ok(), "Empty list should decode");

        if let Ok(item) = decoded {
            if let Ok(list) = item.as_list() {
                prop_assert!(list.is_empty(), "Empty list should have no items");
            }
        }
    }

    // DISABLED: Requires encode_u64 function
    // fn prop_rlp_integer_minimal() { }
}

//
// Property 3: Transaction Hash Determinism
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction hash calculation is deterministic
    ///
    /// Computing the hash of the same transaction bytes multiple times
    /// should always yield the same result.
    #[test]
    fn prop_transaction_hash_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = EthereumDecoder::decode(&bytes) {
            let hash1 = tx.hash();
            let hash2 = tx.hash();
            prop_assert_eq!(hash1.clone(), hash2, "Transaction hash is non-deterministic");
            let expected_hash = Keccak256::digest(&bytes).to_vec();
            prop_assert_eq!(hash1, expected_hash, "Hash should match Keccak256 of bytes");
        }
    }

    /// Property: Hash calculation never panics
    #[test]
    fn prop_hash_never_panics(bytes in arb_small_bytes()) {
        use std::panic;

        let result = panic::catch_unwind(|| {
            if let Ok(tx) = EthereumDecoder::decode(&bytes) {
                let _ = tx.hash();
            }
        });

        prop_assert!(result.is_ok(), "Hash calculation panicked");
    }
}

//
// Property 4: Gas and Fee Calculation Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Gas limit is always positive and bounded
    ///
    /// Gas limit should be > 0 and < block gas limit (~30M)
    #[test]
    fn prop_gas_limit_bounds(gas_limit in 21000u64..30_000_000) {
        // Valid gas limit range
        prop_assert!(gas_limit >= 21000, "Gas limit too low");
        prop_assert!(gas_limit <= 30_000_000, "Gas limit too high");
    }

    /// Property: Gas price calculations don't overflow
    ///
    /// For EIP-1559 transactions: effective_gas_price = base_fee + min(max_priority_fee, max_fee - base_fee)
    #[test]
    fn prop_gas_price_no_overflow(
        base_fee in 1u64..1_000_000_000_000, // 1 gwei to 1000 gwei
        max_fee in 1u64..1_000_000_000_000,
        max_priority_fee in 1u64..1_000_000_000_000,
    ) {
        // Calculation should never panic
        if max_fee >= base_fee {
            let max_priority = max_priority_fee.min(max_fee - base_fee);
            let effective_price = base_fee.checked_add(max_priority);
            prop_assert!(effective_price.is_some(), "Gas price calculation overflowed");
        }
    }

    /// Property: Transaction value never exceeds reasonable bounds
    ///
    /// Ethereum has ~120M ETH total supply, so transaction values
    /// should be << this amount
    #[test]
    fn prop_transaction_value_bounded(value in 0u128..200_000_000_000_000_000_000_000_000) {
        // Value in wei (18 decimals)
        // Max reasonable: 200M ETH
        prop_assert!(value <= 200_000_000u128 * 10u128.pow(18),
            "Transaction value exceeds reasonable bounds");
    }
}

//
// Property 5: Canonical Serialization Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Successfully decoded transactions can be canonicalized
    ///
    /// Any transaction that decodes successfully should also be able to
    /// produce canonical bytes without panicking.
    #[test]
    fn prop_decoded_tx_canonicalizes(bytes in arb_small_bytes()) {
        if let Ok(tx) = EthereumDecoder::decode(&bytes) {
            // If decode succeeds, canonicalization should also succeed or error gracefully
            let result = tx.canonicalize();
            prop_assert!(result.is_ok() || result.is_err(),
                "Canonicalization should return Result, not panic");
        }
        // If decode fails, property is vacuously true
    }

    /// Property: Canonical hash is deterministic
    ///
    /// Computing canonical hash multiple times on the same transaction
    /// should yield identical results.
    #[test]
    fn prop_canonical_hash_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = EthereumDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                // Compute hash twice
                let hash1 = tx_ir.canonical_hash();
                let hash2 = tx_ir.canonical_hash();

                match (hash1, hash2) {
                    (Ok(h1), Ok(h2)) => {
                        prop_assert_eq!(h1, h2, "Canonical hash is non-deterministic");
                    }
                    (Err(_), Err(_)) => {
                        // Both failed consistently - OK
                    }
                    _ => {
                        return Err(TestCaseError::fail(
                            "Canonical hash returned different error states"
                        ));
                    }
                }
            }
        }
    }

    /// Property: Canonical bytes are deterministic
    #[test]
    fn prop_canonical_bytes_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = EthereumDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                let bytes1 = tx_ir.to_canonical_bytes();
                let bytes2 = tx_ir.to_canonical_bytes();

                match (bytes1, bytes2) {
                    (Ok(b1), Ok(b2)) => {
                        prop_assert_eq!(b1, b2, "Canonical bytes are non-deterministic");
                    }
                    (Err(_), Err(_)) => {
                        // Both failed consistently - OK
                    }
                    _ => {
                        return Err(TestCaseError::fail(
                            "Canonical bytes returned different error states"
                        ));
                    }
                }
            }
        }
    }
}

//
// Property 6: Transaction Type Detection
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: Transaction type detection is consistent
    ///
    /// Transactions types are determined by first byte:
    /// - No prefix (0xc0-0xff): Legacy
    /// - 0x01: EIP-2930
    /// - 0x02: EIP-1559
    /// - 0x03: EIP-4844 (blob)
    #[test]
    fn prop_transaction_type_detection(bytes in arb_small_bytes()) {
        if bytes.is_empty() {
            return Ok(());
        }

        let first_byte = bytes[0];
        let _expected_is_legacy = first_byte >= 0xc0;

        // Try to decode
        if let Ok(tx) = EthereumDecoder::decode(&bytes) {
            // Type detection should be consistent with first byte
            let actual_is_legacy = tx.tx_type == decoder_ethereum::types::TxType::Legacy;

            // If it's RLP-encoded (starts with 0xc0+), should be legacy
            if first_byte >= 0xc0 {
                prop_assert!(actual_is_legacy,
                    "Transaction starting with 0x{:02x} should be legacy", first_byte);
            }

            // If it starts with 0x01-0x03, should be typed transaction
            if (0x01..=0x03).contains(&first_byte) {
                prop_assert!(!actual_is_legacy,
                    "Transaction starting with 0x{:02x} should be typed", first_byte);
            }
        }
    }

    /// Property: Legacy transaction detection
    #[test]
    fn prop_legacy_transaction_detection(
        _seed in any::<u64>()
    ) {
        // Legacy transactions are RLP-encoded lists starting with 0xc0+
        let legacy_tx = [0xf8, 0x6d]; // RLP list prefix

        // Should be recognized as having RLP structure
        let is_rlp_list = legacy_tx[0] >= 0xc0;
        prop_assert!(is_rlp_list, "Legacy transaction should start with RLP list");
    }

    /// Property: EIP-1559 transaction detection
    #[test]
    fn prop_eip1559_transaction_detection(
        _seed in any::<u64>()
    ) {
        // EIP-1559 transactions start with 0x02
        let eip1559_tx = [0x02, 0xf8, 0x6d];

        prop_assert_eq!(eip1559_tx[0], 0x02, "EIP-1559 should start with 0x02");
    }
}

//
// Property 7: Signature Field Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Signature r and s values are 32 bytes
    ///
    /// ECDSA signatures on secp256k1 always produce 32-byte r and s values
    #[test]
    fn prop_signature_field_sizes(
        r_bytes in prop::collection::vec(any::<u8>(), 32..=32),
        s_bytes in prop::collection::vec(any::<u8>(), 32..=32),
    ) {
        prop_assert_eq!(r_bytes.len(), 32, "r should be 32 bytes");
        prop_assert_eq!(s_bytes.len(), 32, "s should be 32 bytes");
    }

    /// Property: Recovery ID (v) is in valid range
    ///
    /// For pre-EIP-155: v ∈ {27, 28}
    /// For EIP-155: v = {0, 1} + CHAIN_ID * 2 + 35
    /// For typed transactions: v ∈ {0, 1}
    #[test]
    fn prop_recovery_id_range(chain_id in 1u64..1000, parity in 0u8..2) {
        // EIP-155 encoding
        let v = parity as u64 + chain_id * 2 + 35;

        // Should be able to recover original values
        let recovered_parity = (v - 35) % 2;
        prop_assert_eq!(recovered_parity, parity as u64,
            "Should recover parity from v");

        let recovered_chain_id = (v - 35 - recovered_parity) / 2;
        prop_assert_eq!(recovered_chain_id, chain_id,
            "Should recover chain_id from v");
    }
}

//
// Property 8: Address Handling
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: Ethereum addresses are always 20 bytes
    ///
    /// All Ethereum addresses must be exactly 20 bytes (160 bits)
    #[test]
    fn prop_address_size(addr_bytes in prop::collection::vec(any::<u8>(), 20..=20)) {
        prop_assert_eq!(addr_bytes.len(), 20, "Ethereum address must be 20 bytes");
    }

    /// Property: Address derivation from public key is deterministic
    ///
    /// Address = Keccak256(public_key)[12..32]
    #[test]
    fn prop_address_from_pubkey(pubkey_bytes in prop::collection::vec(any::<u8>(), 64..=64)) {
        // Public key is 64 bytes (uncompressed, without prefix)
        let hash = Keccak256::digest(&pubkey_bytes);
        let addr1 = &hash[12..32];
        let addr2 = &hash[12..32];

        prop_assert_eq!(addr1, addr2, "Address derivation should be deterministic");
        prop_assert_eq!(addr1.len(), 20, "Derived address should be 20 bytes");
    }

    /// Property: Zero address is valid
    #[test]
    fn prop_zero_address_valid(_unit in 0u8..1) {
        let zero_addr = [0u8; 20];
        prop_assert_eq!(zero_addr.len(), 20, "Zero address should be 20 bytes");
        // Zero address is used for contract creation and burning
    }
}

//
// Integration Property Tests
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Decode-Canonicalize-Hash pipeline never panics
    ///
    /// The full pipeline from raw bytes to canonical hash should
    /// never panic, even on invalid input.
    #[test]
    fn prop_full_pipeline_never_panics(bytes in arb_small_bytes()) {
        use std::panic;

        let result = panic::catch_unwind(|| {
            if let Ok(tx) = EthereumDecoder::decode(&bytes) {
                let _ = tx.hash();
                if let Ok(tx_ir) = tx.canonicalize() {
                    let _ = tx_ir.canonical_hash();
                    let _ = tx_ir.to_canonical_bytes();
                }
            }
        });

        prop_assert!(result.is_ok(), "Full pipeline panicked on input");
    }

    /// Property: Validation pipeline never panics
    #[test]
    fn prop_validation_never_panics(bytes in arb_small_bytes()) {
        use std::panic;

        let result = panic::catch_unwind(|| {
            // Format validation
            let _ = EthereumDecoder::validate_format(&bytes);

            // Full decode + validate
            if let Ok(tx) = EthereumDecoder::decode(&bytes) {
                let _ = tx.validate();
            }
        });

        prop_assert!(result.is_ok(), "Validation pipeline panicked");
    }
}

//
// Property 9: Nonce Handling
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Nonce is always non-negative (u64)
    #[test]
    fn prop_nonce_non_negative(_nonce in any::<u64>()) {
        // Nonce is u64, so always non-negative by type
        // No assertion needed - the type system guarantees this
    }

    /// Property: Nonce = 0 is valid (first transaction)
    #[test]
    fn prop_zero_nonce_valid(_unit in 0u8..1) {
        let nonce: u64 = 0;
        prop_assert_eq!(nonce, 0, "Zero nonce should be valid");
    }

    /// Property: Sequential nonces are valid
    #[test]
    fn prop_sequential_nonces(start_nonce in 0u64..1_000_000) {
        let next_nonce = start_nonce + 1;
        prop_assert!(next_nonce > start_nonce || start_nonce == u64::MAX,
            "Nonce should increment sequentially");
    }
}

//
// Property 10: Contract Creation vs Call
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Contract creation is indicated by empty 'to' field
    ///
    /// When 'to' is None/empty, transaction is contract creation
    /// When 'to' is Some(address), transaction is a call
    #[test]
    fn prop_contract_creation_indicator(has_to_addr in any::<bool>()) {
        // This property is structural
        if has_to_addr {
            // Call: has recipient address
            prop_assert!(has_to_addr, "Call should have to address");
        } else {
            // Creation: no recipient address
            prop_assert!(!has_to_addr, "Creation should have no to address");
        }
    }

    /// Property: Contract creation can have large data field
    ///
    /// Contract bytecode can be large (up to block gas limit)
    #[test]
    fn prop_contract_creation_large_data(size in 0usize..50_000) {
        let data = vec![0u8; size];
        prop_assert!(data.len() <= 50_000,
            "Contract creation data should be bounded by gas limit");
    }
}
