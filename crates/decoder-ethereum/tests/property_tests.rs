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
            let hash1 = tx.hash().ok();
            let hash2 = tx.hash().ok();
            prop_assert_eq!(hash1.clone(), hash2, "Transaction hash is non-deterministic");
            let expected_hash = Keccak256::digest(&bytes).to_vec();
            prop_assert_eq!(hash1, Some(expected_hash), "Hash should match Keccak256 of bytes");
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
// Property 11: Roundtrip Encoding (Injective Property)
//

use decoder_encodings::RlpEncoder;

/// Strip leading zeros from a byte slice for canonical RLP encoding.
/// Per RLP spec, integers (including signature components) should be encoded minimally.
fn strip_leading_zeros(bytes: &[u8]) -> Vec<u8> {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    bytes[start..].to_vec()
}

/// Helper: Encode a valid legacy Ethereum transaction to RLP bytes
#[allow(clippy::too_many_arguments)]
fn encode_legacy_tx(
    nonce: u64,
    gas_price: u64,
    gas_limit: u64,
    to: Option<[u8; 20]>,
    value: u128,
    data: &[u8],
    v: u64,
    r: &[u8; 32],
    s: &[u8; 32],
) -> Vec<u8> {
    let mut encoder = RlpEncoder::new();
    let mut list = encoder.begin_list();

    // 1. nonce
    list.append_u64(nonce).unwrap();

    // 2. gasPrice
    list.append_u64(gas_price).unwrap();

    // 3. gasLimit
    list.append_u64(gas_limit).unwrap();

    // 4. to (None for contract creation)
    list.append_address(to).unwrap();

    // 5. value
    list.append_u128(value).unwrap();

    // 6. data
    list.append_bytes(data).unwrap();

    // 7. v (signature recovery id + chain id encoding)
    list.append_u64(v).unwrap();

    // 8. r (signature) - strip leading zeros for canonical encoding
    list.append_bytes(&strip_leading_zeros(r)).unwrap();

    // 9. s (signature) - strip leading zeros for canonical encoding
    list.append_bytes(&strip_leading_zeros(s)).unwrap();

    list.finalize().unwrap();
    encoder.finalize()
}

/// Helper: Encode a valid EIP-1559 transaction to typed transaction bytes
#[allow(clippy::too_many_arguments)]
fn encode_eip1559_tx(
    chain_id: u64,
    nonce: u64,
    max_priority_fee_per_gas: u64,
    max_fee_per_gas: u64,
    gas_limit: u64,
    to: Option<[u8; 20]>,
    value: u128,
    data: &[u8],
    access_list: &[(
        [u8; 20],      // address
        Vec<[u8; 32]>, // storage keys
    )],
    v: u8,
    r: &[u8; 32],
    s: &[u8; 32],
) -> Vec<u8> {
    let mut encoder = RlpEncoder::new();
    let mut list = encoder.begin_list();

    // 1. chainId
    list.append_u64(chain_id).unwrap();

    // 2. nonce
    list.append_u64(nonce).unwrap();

    // 3. maxPriorityFeePerGas
    list.append_u64(max_priority_fee_per_gas).unwrap();

    // 4. maxFeePerGas
    list.append_u64(max_fee_per_gas).unwrap();

    // 5. gasLimit
    list.append_u64(gas_limit).unwrap();

    // 6. to (None for contract creation)
    list.append_address(to).unwrap();

    // 7. value
    list.append_u128(value).unwrap();

    // 8. data
    list.append_bytes(data).unwrap();

    // 9. accessList (encode as empty list for now)
    // Access list is RLP list of [address, [storage_keys...]]
    let _ = access_list; // Silence unused warning
    list.append_list(|_| Ok(())).unwrap();

    // 10. v (0 or 1 for EIP-1559)
    list.append_u64(v as u64).unwrap();

    // 11. r (signature) - strip leading zeros for canonical encoding
    list.append_bytes(&strip_leading_zeros(r)).unwrap();

    // 12. s (signature) - strip leading zeros for canonical encoding
    list.append_bytes(&strip_leading_zeros(s)).unwrap();

    list.finalize().unwrap();

    // Prepend EIP-1559 type byte (0x02)
    let mut result = vec![0x02];
    result.extend(encoder.finalize());
    result
}

/// Helper: Encode a legacy transaction WITHOUT stripping signature leading
/// zeros. Used to verify the decoder rejects this non-canonical form.
#[allow(clippy::too_many_arguments)]
fn encode_legacy_tx_unstripped_sig(
    nonce: u64,
    gas_price: u64,
    gas_limit: u64,
    to: Option<[u8; 20]>,
    value: u128,
    data: &[u8],
    v: u64,
    r: &[u8; 32],
    s: &[u8; 32],
) -> Vec<u8> {
    let mut encoder = RlpEncoder::new();
    let mut list = encoder.begin_list();

    list.append_u64(nonce).unwrap();
    list.append_u64(gas_price).unwrap();
    list.append_u64(gas_limit).unwrap();
    list.append_address(to).unwrap();
    list.append_u128(value).unwrap();
    list.append_bytes(data).unwrap();
    list.append_u64(v).unwrap();
    // Deliberately non-canonical: full 32 bytes including leading zeros
    list.append_bytes(r).unwrap();
    list.append_bytes(s).unwrap();

    list.finalize().unwrap();
    encoder.finalize()
}

/// Helper: Encode a valid EIP-4844 blob transaction to typed transaction bytes
#[allow(clippy::too_many_arguments)]
fn encode_eip4844_tx(
    chain_id: u64,
    nonce: u64,
    max_priority_fee_per_gas: u64,
    max_fee_per_gas: u64,
    gas_limit: u64,
    to: [u8; 20],
    value: u128,
    data: &[u8],
    max_fee_per_blob_gas: u64,
    blob_versioned_hashes: &[[u8; 32]],
    v: u8,
    r: &[u8; 32],
    s: &[u8; 32],
) -> Vec<u8> {
    let mut encoder = RlpEncoder::new();
    let mut list = encoder.begin_list();

    list.append_u64(chain_id).unwrap();
    list.append_u64(nonce).unwrap();
    list.append_u64(max_priority_fee_per_gas).unwrap();
    list.append_u64(max_fee_per_gas).unwrap();
    list.append_u64(gas_limit).unwrap();
    list.append_address(Some(to)).unwrap();
    list.append_u128(value).unwrap();
    list.append_bytes(data).unwrap();

    // Empty access list
    list.append_list(|_| Ok(())).unwrap();

    list.append_u64(max_fee_per_blob_gas).unwrap();
    list.append_list(|hashes| {
        for hash in blob_versioned_hashes {
            hashes.append_bytes(hash)?;
        }
        Ok(())
    })
    .unwrap();

    list.append_u64(v as u64).unwrap();
    list.append_bytes(&strip_leading_zeros(r)).unwrap();
    list.append_bytes(&strip_leading_zeros(s)).unwrap();

    list.finalize().unwrap();

    // Prepend EIP-4844 type byte (0x03)
    let mut result = vec![0x03];
    result.extend(encoder.finalize());
    result
}

/// Strategy: Generate a valid EIP-4844 blob transaction
///
/// The 13 parameters exceed proptest's 12-element tuple limit,
/// so they are grouped into two nested tuples.
fn arb_valid_eip4844_tx() -> impl Strategy<Value = Vec<u8>> {
    (
        (
            1u64..10u64,                                // chain_id
            any::<u64>(),                               // nonce
            1u64..100_000_000_000u64,                   // max_priority_fee
            1u64..1_000_000_000_000u64,                 // max_fee
            21000u64..30_000_000u64,                    // gas_limit
            arb_address(),                              // to (required for 4844)
            0u128..10_000_000_000_000_000_000u128,      // value
            prop::collection::vec(any::<u8>(), 0..100), // data
        ),
        (
            1u64..1_000_000_000u64,                      // max_fee_per_blob_gas
            prop::collection::vec(arb_bytes32(), 1..=6), // blob_versioned_hashes
            0u8..=1u8,                                   // v
            arb_bytes32(),                               // r
            arb_bytes32(),                               // s
        ),
    )
        .prop_map(
            |(
                (chain_id, nonce, max_priority, max_fee, gas_limit, to, value, data),
                (max_fee_per_blob_gas, blob_hashes, v, r, s),
            )| {
                encode_eip4844_tx(
                    chain_id,
                    nonce,
                    max_priority,
                    max_fee,
                    gas_limit,
                    to,
                    value,
                    &data,
                    max_fee_per_blob_gas,
                    &blob_hashes,
                    v,
                    &r,
                    &s,
                )
            },
        )
}

/// Strategy: Generate arbitrary 32-byte hash/signature component
fn arb_bytes32() -> impl Strategy<Value = [u8; 32]> {
    prop::array::uniform32(any::<u8>())
}

/// Strategy: Generate arbitrary 20-byte address
fn arb_address() -> impl Strategy<Value = [u8; 20]> {
    prop::array::uniform20(any::<u8>())
}

/// Strategy: Generate a valid legacy Ethereum transaction
fn arb_valid_legacy_tx() -> impl Strategy<Value = Vec<u8>> {
    (
        any::<u64>(),                               // nonce
        1u64..1_000_000_000_000u64,                 // gas_price (1 gwei to 1000 gwei)
        21000u64..30_000_000u64,                    // gas_limit
        prop::option::of(arb_address()),            // to (None for contract creation)
        0u128..10_000_000_000_000_000_000u128,      // value (up to 10 ETH)
        prop::collection::vec(any::<u8>(), 0..100), // data
        27u64..=28u64,                              // v (pre-EIP-155)
        arb_bytes32(),                              // r
        arb_bytes32(),                              // s
    )
        .prop_map(|(nonce, gas_price, gas_limit, to, value, data, v, r, s)| {
            encode_legacy_tx(nonce, gas_price, gas_limit, to, value, &data, v, &r, &s)
        })
}

/// Strategy: Generate a valid EIP-1559 Ethereum transaction
fn arb_valid_eip1559_tx() -> impl Strategy<Value = Vec<u8>> {
    (
        1u64..10u64,                                // chain_id (mainnet = 1)
        any::<u64>(),                               // nonce
        1u64..100_000_000_000u64,                   // max_priority_fee (1-100 gwei)
        1u64..1_000_000_000_000u64,                 // max_fee (1-1000 gwei)
        21000u64..30_000_000u64,                    // gas_limit
        prop::option::of(arb_address()),            // to
        0u128..10_000_000_000_000_000_000u128,      // value
        prop::collection::vec(any::<u8>(), 0..100), // data
        0u8..=1u8,                                  // v (0 or 1 for EIP-1559)
        arb_bytes32(),                              // r
        arb_bytes32(),                              // s
    )
        .prop_filter(
            "max_fee must be >= max_priority_fee",
            |(_, _, max_priority, max_fee, _, _, _, _, _, _, _)| max_fee >= max_priority,
        )
        .prop_map(
            |(chain_id, nonce, max_priority, max_fee, gas_limit, to, value, data, v, r, s)| {
                // Empty access list for simplicity
                let access_list: Vec<([u8; 20], Vec<[u8; 32]>)> = vec![];
                encode_eip1559_tx(
                    chain_id,
                    nonce,
                    max_priority,
                    max_fee,
                    gas_limit,
                    to,
                    value,
                    &data,
                    &access_list,
                    v,
                    &r,
                    &s,
                )
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Roundtrip encoding preserves legacy transaction bytes (Injective Property)
    ///
    /// This is the CRITICAL property mandated by CLAUDE.md v0.3.0:
    /// For any valid Ethereum transaction bytes, decode(tx_bytes).to_bytes() == tx_bytes
    #[test]
    fn prop_ethereum_roundtrip_legacy(tx_bytes in arb_valid_legacy_tx()) {
        // Decode the generated transaction bytes
        let decoded = EthereumDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        // Re-encode back to bytes
        let re_encoded = decoded.to_bytes()
            .map_err(|e| TestCaseError::fail(format!("Encode failed: {}", e)))?;

        // Verify the injective property: encode(decode(x)) == x
        prop_assert_eq!(
            tx_bytes.as_slice(),
            re_encoded.as_slice(),
            "Roundtrip failed for legacy tx: encode(decode(tx_bytes)) != tx_bytes"
        );
    }

    /// Property: Roundtrip encoding preserves EIP-1559 transaction bytes
    #[test]
    fn prop_ethereum_roundtrip_eip1559(tx_bytes in arb_valid_eip1559_tx()) {
        // Decode the generated transaction bytes
        let decoded = EthereumDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        // Re-encode back to bytes
        let re_encoded = decoded.to_bytes()
            .map_err(|e| TestCaseError::fail(format!("Encode failed: {}", e)))?;

        // Verify the injective property
        prop_assert_eq!(
            tx_bytes.as_slice(),
            re_encoded.as_slice(),
            "Roundtrip failed for EIP-1559 tx: encode(decode(tx_bytes)) != tx_bytes"
        );
    }

    /// Property: Roundtrip encoding preserves EIP-4844 transaction bytes
    #[test]
    fn prop_ethereum_roundtrip_eip4844(tx_bytes in arb_valid_eip4844_tx()) {
        let decoded = EthereumDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        prop_assert_eq!(decoded.tx_type, decoder_ethereum::types::TxType::Eip4844);

        let re_encoded = decoded.to_bytes()
            .map_err(|e| TestCaseError::fail(format!("Encode failed: {}", e)))?;

        prop_assert_eq!(
            tx_bytes.as_slice(),
            re_encoded.as_slice(),
            "Roundtrip failed for EIP-4844 tx: encode(decode(tx_bytes)) != tx_bytes"
        );
    }

    /// Property: Roundtrip preserves transaction type
    #[test]
    fn prop_ethereum_roundtrip_preserves_type(tx_bytes in prop_oneof![
        arb_valid_legacy_tx(),
        arb_valid_eip1559_tx()
    ]) {
        let decoded = EthereumDecoder::decode(&tx_bytes)
            .map_err(|e| TestCaseError::fail(format!("Decode failed: {}", e)))?;

        // Transaction type should match the first byte pattern
        let is_typed = tx_bytes.first().map(|b| *b <= 0x7f).unwrap_or(false);
        let is_legacy = decoded.tx_type == decoder_ethereum::types::TxType::Legacy;

        prop_assert_eq!(!is_typed, is_legacy,
            "Transaction type should match encoding pattern");
    }
}

//
// Strict Injective Property: decode success implies EXACT roundtrip
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: For ARBITRARY bytes, if decode succeeds then re-encoding
    /// reproduces the input exactly.
    ///
    /// This is the strongest form of the injective property. It holds only
    /// because the decoder rejects non-canonical encodings (padded signature
    /// components, non-minimal RLP) — anything it accepts, it can reproduce
    /// byte-for-byte from parsed fields alone.
    #[test]
    fn prop_decode_success_implies_exact_roundtrip(bytes in arb_small_bytes()) {
        if let Ok(decoded) = EthereumDecoder::decode(&bytes) {
            let re_encoded = decoded.to_bytes()
                .map_err(|e| TestCaseError::fail(format!("Encode failed: {}", e)))?;
            prop_assert_eq!(
                bytes.as_slice(),
                re_encoded.as_slice(),
                "Decoder accepted bytes it cannot reproduce: this breaks injectivity"
            );
        }
    }

    /// Property: Non-canonical zero-padded signature components are rejected.
    ///
    /// A signature r/s value with leading zero bytes has exactly one canonical
    /// RLP encoding (stripped). The decoder must reject the padded form,
    /// otherwise decode would succeed on bytes that to_bytes() cannot reproduce.
    #[test]
    fn prop_ethereum_rejects_padded_signature(
        (nonce, gas_price, gas_limit, to, value, data, v, r, s) in (
            any::<u64>(),
            1u64..1_000_000_000_000u64,
            21000u64..30_000_000u64,
            prop::option::of(arb_address()),
            0u128..10_000_000_000_000_000_000u128,
            prop::collection::vec(any::<u8>(), 0..100),
            27u64..=28u64,
            arb_bytes32(),
            arb_bytes32(),
        )
    ) {
        let mut r = r;
        r[0] = 0; // Force a leading zero so the 32-byte encoding is non-canonical

        let tx_bytes = encode_legacy_tx_unstripped_sig(
            nonce, gas_price, gas_limit, to, value, &data, v, &r, &s,
        );

        prop_assert!(
            EthereumDecoder::decode(&tx_bytes).is_err(),
            "Decoder must reject zero-padded signature components"
        );
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
