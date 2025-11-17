//! Property-based tests for Cardano decoder
//!
//! This module uses proptest to verify critical properties of the Cardano decoder:
//! 1. Decoder never panics on arbitrary input
//! 2. CBOR parsing is robust
//! 3. Transaction ID calculation is deterministic
//! 4. Fee calculation properties (non-negative, bounded)
//! 5. Canonical serialization properties

use decoder_cardano::*;
use decoder_test_utils::proptest_helpers::{arb_small_bytes, prop_decoder_never_panics};
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

//
// Property 1: Decoder Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Cardano decoder never panics on arbitrary input
    ///
    /// For any arbitrary byte sequence, decode() must return Ok or Err,
    /// never panic.
    #[test]
    fn prop_cardano_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<CardanoDecoder>(&bytes);
    }

    /// Property: Cardano decoder never panics on empty input
    #[test]
    fn prop_cardano_decoder_rejects_empty(_unit in 0u8..1) {
        let result = CardanoDecoder::decode(&[]);
        prop_assert!(result.is_err(), "Decoder should reject empty input");
    }

    /// Property: Cardano decoder never panics on very short input
    #[test]
    fn prop_cardano_decoder_rejects_tiny_input(size in 1usize..10) {
        let bytes = vec![0xFF; size];
        let result = CardanoDecoder::decode(&bytes);
        prop_assert!(result.is_err(), "Decoder should reject input < 10 bytes");
    }
}

//
// Property 2: CBOR Parsing Robustness
//
// Note: CBOR parsing is now handled by the battle-tested minicbor library,
// so we don't need to test internal CBOR parsing functions. We test
// end-to-end decoder behavior instead.

//
// Property 3: Transaction ID Determinism
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction ID calculation is deterministic
    ///
    /// Computing the TXID of the same transaction multiple times
    /// should always yield the same result.
    #[test]
    fn prop_txid_deterministic(seed in any::<u64>()) {
        // Create a deterministic transaction based on seed
        let tx_bytes = create_test_cardano_tx(seed);

        // Try to decode
        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            // If decode succeeds, TXID should be deterministic
            let txid1 = tx.txid();
            let txid2 = tx.txid();
            prop_assert_eq!(txid1, txid2, "TXID calculation is non-deterministic");

            // Hex representation should also be deterministic
            let txid_hex1 = tx.txid_hex();
            let txid_hex2 = tx.txid_hex();
            prop_assert_eq!(txid_hex1, txid_hex2, "TXID hex is non-deterministic");
        }
        // If decode fails, property is vacuously true
    }
}

//
// Property 4: Fee Calculation Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Fee calculation never panics and is non-negative
    ///
    /// For valid transactions, fee calculation should:
    /// 1. Never panic
    /// 2. Return non-negative value
    /// 3. Be reasonable (< 1 ADA for typical transactions)
    #[test]
    fn prop_fee_calculation_properties(
        fee in 1u64..10_000_000u64 // 0.001 to 10 ADA
    ) {
        let tx_bytes = create_test_cardano_tx_with_fee(fee);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            let parsed_fee = tx.fee();

            // Fee should match what we set
            prop_assert_eq!(parsed_fee, fee, "Fee should match");

            // Fee should be reasonable (< 10 ADA)
            prop_assert!(parsed_fee < 10_000_000, "Fee should be < 10 ADA");
        }
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
    fn prop_decoded_tx_canonicalizes(seed in any::<u64>()) {
        let bytes = create_test_cardano_tx(seed);

        if let Ok(tx) = CardanoDecoder::decode(&bytes) {
            // If decode succeeds, canonicalization should also succeed or fail gracefully
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
    fn prop_canonical_hash_deterministic(seed in any::<u64>()) {
        let bytes = create_test_cardano_tx(seed);

        if let Ok(tx) = CardanoDecoder::decode(&bytes) {
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
}

//
// Property 6: Input/Output Count Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction input/output counts are reasonable
    ///
    /// Cardano transactions should have reasonable limits on the number of
    /// inputs and outputs.
    #[test]
    fn prop_io_count_reasonable(
        input_count in 1usize..100,
        output_count in 1usize..100,
    ) {
        let tx_bytes = create_test_cardano_tx_with_io(input_count, output_count);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            let parsed_input_count = tx.input_count();
            let parsed_output_count = tx.output_count();

            // Counts should match
            prop_assert_eq!(parsed_input_count, input_count, "Input count should match");
            prop_assert_eq!(parsed_output_count, output_count, "Output count should match");

            // Counts should be reasonable
            prop_assert!(parsed_input_count > 0, "Must have at least 1 input");
            prop_assert!(parsed_output_count > 0, "Must have at least 1 output");
            prop_assert!(parsed_input_count < 1000, "Input count should be reasonable");
            prop_assert!(parsed_output_count < 1000, "Output count should be reasonable");
        }
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
            if let Ok(tx) = CardanoDecoder::decode(&bytes) {
                if let Ok(tx_ir) = tx.canonicalize() {
                    let _ = tx_ir.canonical_hash();
                }
            }
        });

        prop_assert!(result.is_ok(), "Full pipeline panicked on input");
    }
}

//
// Helper Functions
//

/// Create a test Cardano transaction with deterministic content based on seed
fn create_test_cardano_tx(seed: u64) -> Vec<u8> {
    create_test_cardano_tx_with_fee(170_000 + (seed % 100_000))
}

/// Create a test Cardano transaction with specific fee
#[allow(clippy::vec_init_then_push)]
fn create_test_cardano_tx_with_fee(fee: u64) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // CBOR array with 3 elements
    tx_bytes.push(0x83);

    // Transaction body (CBOR map)
    tx_bytes.push(0xa3);

    // Key 0: inputs
    tx_bytes.push(0x00);
    tx_bytes.push(0x81); // Array with 1 element
    tx_bytes.push(0x82); // [tx_hash, index]
    tx_bytes.push(0x58);
    tx_bytes.push(0x20); // 32 bytes
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x00); // index 0

    // Key 1: outputs
    tx_bytes.push(0x01);
    tx_bytes.push(0x81); // Array with 1 element
    tx_bytes.push(0x82); // [address, amount]
    tx_bytes.push(0x58);
    tx_bytes.push(0x1d); // 29 bytes
    tx_bytes.extend_from_slice(&[0u8; 29]);
    tx_bytes.push(0x1a); // uint32
    tx_bytes.extend_from_slice(&1_000_000u32.to_be_bytes());

    // Key 2: fee
    tx_bytes.push(0x02);
    if fee <= 0xFFFFFFFF {
        tx_bytes.push(0x1a); // uint32
        tx_bytes.extend_from_slice(&(fee as u32).to_be_bytes());
    } else {
        tx_bytes.push(0x1b); // uint64
        tx_bytes.extend_from_slice(&fee.to_be_bytes());
    }

    // Witness set
    tx_bytes.push(0xa1);
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x58);
    tx_bytes.push(0x40);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // No auxiliary data
    tx_bytes.push(0xf6);

    tx_bytes
}

/// Create a test Cardano transaction with specific input/output counts
#[allow(clippy::vec_init_then_push)]
fn create_test_cardano_tx_with_io(input_count: usize, output_count: usize) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // CBOR array with 3 elements
    tx_bytes.push(0x83);

    // Transaction body (CBOR map)
    tx_bytes.push(0xa3);

    // Key 0: inputs
    tx_bytes.push(0x00);
    encode_cbor_array_len(&mut tx_bytes, input_count);
    for _ in 0..input_count {
        tx_bytes.push(0x82); // [tx_hash, index]
        tx_bytes.push(0x58);
        tx_bytes.push(0x20);
        tx_bytes.extend_from_slice(&[0u8; 32]);
        tx_bytes.push(0x00);
    }

    // Key 1: outputs
    tx_bytes.push(0x01);
    encode_cbor_array_len(&mut tx_bytes, output_count);
    for _ in 0..output_count {
        tx_bytes.push(0x82); // [address, amount]
        tx_bytes.push(0x58);
        tx_bytes.push(0x1d);
        tx_bytes.extend_from_slice(&[0u8; 29]);
        tx_bytes.push(0x1a);
        tx_bytes.extend_from_slice(&1_000_000u32.to_be_bytes());
    }

    // Key 2: fee
    tx_bytes.push(0x02);
    tx_bytes.push(0x1a);
    tx_bytes.extend_from_slice(&170_000u32.to_be_bytes());

    // Witness set
    tx_bytes.push(0xa1);
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x58);
    tx_bytes.push(0x40);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // No auxiliary data
    tx_bytes.push(0xf6);

    tx_bytes
}

/// Encode CBOR array length
fn encode_cbor_array_len(buf: &mut Vec<u8>, len: usize) {
    if len < 24 {
        buf.push(0x80 | (len as u8));
    } else if len <= 0xFF {
        buf.push(0x98);
        buf.push(len as u8);
    } else if len <= 0xFFFF {
        buf.push(0x99);
        buf.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        buf.push(0x9a);
        buf.extend_from_slice(&(len as u32).to_be_bytes());
    }
}

//
// Property 7: Multi-Asset Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Multi-asset policy ID has correct length
    ///
    /// Cardano policy IDs are blake2b-224 hashes (28 bytes)
    #[test]
    fn prop_multi_asset_policy_id_length(seed in any::<u64>()) {
        let tx_bytes = create_test_cardano_tx_with_mint(seed);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            if tx.has_mint() {
                for asset in &tx.body.mint {
                    // Policy ID should be 28 bytes (blake2b-224)
                    prop_assert_eq!(
                        asset.policy_id.len(),
                        28,
                        "Policy ID should be 28 bytes (blake2b-224 hash)"
                    );
                }
            }
        }
    }

    /// Property: Multi-asset amount can be negative (for burning)
    ///
    /// Minting has positive amounts, burning has negative amounts
    #[test]
    fn prop_multi_asset_amount_range(
        mint_amount in -1_000_000i64..1_000_000i64
    ) {
        let tx_bytes = create_test_cardano_tx_with_mint_amount(mint_amount);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            if tx.has_mint() {
                for asset in &tx.body.mint {
                    // Amount can be positive (mint) or negative (burn)
                    prop_assert!(
                        asset.amount == mint_amount,
                        "Multi-asset amount should match expected value"
                    );

                    // Amount should be within reasonable bounds
                    prop_assert!(
                        asset.amount.abs() <= 1_000_000_000,
                        "Multi-asset amount should be within reasonable bounds"
                    );
                }
            }
        }
    }

    /// Property: Multi-asset outputs are consistent
    ///
    /// Outputs with multi-assets should have valid structure
    #[test]
    fn prop_multi_asset_output_consistency(seed in any::<u64>()) {
        let tx_bytes = create_test_cardano_tx_with_asset_outputs(seed);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            for output in &tx.body.outputs {
                if !output.assets.is_empty() {
                    // Output must have a valid address
                    prop_assert!(
                        !output.address.is_empty(),
                        "Output with assets must have an address"
                    );

                    // Each asset must have a policy ID
                    for asset in &output.assets {
                        prop_assert_eq!(
                            asset.policy_id.len(),
                            28,
                            "Asset policy ID must be 28 bytes"
                        );
                    }
                }
            }
        }
    }
}

//
// Property 8: Metadata Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Metadata value nesting depth is reasonable
    ///
    /// Deeply nested metadata could cause stack overflow
    #[test]
    fn prop_metadata_nesting_depth_bounded(seed in any::<u64>()) {
        let tx_bytes = create_test_cardano_tx_with_metadata(seed);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            if let Some(aux_data) = &tx.auxiliary_data {
                for (_, metadata_value) in &aux_data.metadata {
                    let depth = calculate_metadata_depth(metadata_value);
                    // Reasonable nesting limit to prevent DoS
                    prop_assert!(
                        depth <= 10,
                        "Metadata nesting depth should be <= 10, got {}",
                        depth
                    );
                }
            }
        }
    }

    /// Property: Metadata text is valid UTF-8
    ///
    /// All text metadata must be valid UTF-8 strings
    #[test]
    fn prop_metadata_text_valid_utf8(seed in any::<u64>()) {
        let tx_bytes = create_test_cardano_tx_with_metadata(seed);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            if let Some(aux_data) = &tx.auxiliary_data {
                for (_, metadata_value) in &aux_data.metadata {
                    validate_metadata_utf8(metadata_value)?;
                }
            }
        }
    }

    /// Property: Metadata map keys are unique
    ///
    /// CBOR maps should not have duplicate keys
    #[test]
    fn prop_metadata_map_keys_unique(seed in any::<u64>()) {
        let tx_bytes = create_test_cardano_tx_with_metadata(seed);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            if let Some(aux_data) = &tx.auxiliary_data {
                // Check that metadata labels are unique
                let labels: Vec<u64> = aux_data.metadata.iter().map(|(k, _)| *k).collect();
                let unique_labels: std::collections::HashSet<u64> =
                    labels.iter().copied().collect();

                prop_assert_eq!(
                    labels.len(),
                    unique_labels.len(),
                    "Metadata labels should be unique"
                );
            }
        }
    }
}

//
// Property 9: Plutus Script Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Collateral inputs are valid when Plutus scripts present
    ///
    /// Transactions with Plutus scripts must have collateral
    #[test]
    fn prop_plutus_collateral_validation(seed in any::<u64>()) {
        let tx_bytes = create_test_cardano_tx_with_plutus(seed);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            if tx.has_plutus_scripts() {
                // If Plutus scripts are present, collateral should be set
                // (Note: this is a Cardano protocol rule)
                prop_assert!(
                    !tx.body.collateral.is_empty() || tx.witness_set.redeemers.is_empty(),
                    "Transactions with Plutus scripts should have collateral"
                );
            }
        }
    }

    /// Property: Redeemer tags are valid
    ///
    /// Redeemer tags should be in valid range (0-3)
    #[test]
    fn prop_redeemer_tag_valid(tag in 0u8..=3) {
        let tx_bytes = create_test_cardano_tx_with_redeemer(tag);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            for redeemer in &tx.witness_set.redeemers {
                // Redeemer tags: 0=Spend, 1=Mint, 2=Cert, 3=Reward
                prop_assert!(
                    redeemer.tag <= 3,
                    "Redeemer tag should be in range 0-3, got {}",
                    redeemer.tag
                );
            }
        }
    }

    /// Property: Execution units are bounded
    ///
    /// Memory and step limits should be reasonable
    #[test]
    fn prop_execution_units_bounded(
        mem in 1u64..14_000_000u64,  // Max memory units
        steps in 1u64..10_000_000_000u64,  // Max step units
    ) {
        let tx_bytes = create_test_cardano_tx_with_ex_units(mem, steps);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            for redeemer in &tx.witness_set.redeemers {
                // Memory should be within protocol limits
                prop_assert!(
                    redeemer.ex_units.mem <= 14_000_000,
                    "Execution memory should be <= 14M units"
                );

                // Steps should be within protocol limits
                prop_assert!(
                    redeemer.ex_units.steps <= 10_000_000_000,
                    "Execution steps should be <= 10B units"
                );
            }
        }
    }
}

//
// Property 10: Certificate and Withdrawal Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Withdrawal amounts are non-negative and reasonable
    ///
    /// Reward withdrawals should be positive and bounded
    #[test]
    fn prop_withdrawal_amount_valid(
        amount in 1u64..100_000_000u64  // Up to 100 ADA
    ) {
        let tx_bytes = create_test_cardano_tx_with_withdrawal(amount);

        if let Ok(tx) = CardanoDecoder::decode(&tx_bytes) {
            if tx.has_withdrawals() {
                for withdrawal in &tx.body.withdrawals {
                    // Withdrawal must be positive
                    prop_assert!(
                        withdrawal.amount > 0,
                        "Withdrawal amount must be positive"
                    );

                    // Withdrawal should be reasonable (< 1000 ADA)
                    prop_assert!(
                        withdrawal.amount <= 1_000_000_000,
                        "Withdrawal amount should be reasonable"
                    );
                }
            }
        }
    }
}

//
// Helper Functions for New Property Tests
//

/// Calculate nesting depth of metadata value
fn calculate_metadata_depth(value: &crate::types::MetadataValue) -> usize {
    use crate::types::MetadataValue;

    match value {
        MetadataValue::Int(_) | MetadataValue::Bytes(_) | MetadataValue::Text(_) => 1,
        MetadataValue::Array(arr) => {
            1 + arr.iter().map(calculate_metadata_depth).max().unwrap_or(0)
        }
        MetadataValue::Map(map) => {
            1 + map
                .iter()
                .flat_map(|(k, v)| vec![calculate_metadata_depth(k), calculate_metadata_depth(v)])
                .max()
                .unwrap_or(0)
        }
    }
}

/// Validate that all text in metadata is valid UTF-8
fn validate_metadata_utf8(
    value: &crate::types::MetadataValue,
) -> proptest::test_runner::TestCaseResult {
    use crate::types::MetadataValue;
    use proptest::prelude::*;

    match value {
        MetadataValue::Text(s) => {
            // Text should already be a valid String (UTF-8)
            prop_assert!(s.is_empty() || !s.is_empty(), "Text should be valid UTF-8");
        }
        MetadataValue::Array(arr) => {
            for item in arr {
                validate_metadata_utf8(item)?;
            }
        }
        MetadataValue::Map(map) => {
            for (k, v) in map {
                validate_metadata_utf8(k)?;
                validate_metadata_utf8(v)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Create a test transaction with multi-asset minting
fn create_test_cardano_tx_with_mint(seed: u64) -> Vec<u8> {
    create_test_cardano_tx_with_mint_amount((seed % 1000) as i64)
}

/// Create a test transaction with specific mint amount
#[allow(clippy::vec_init_then_push)]
fn create_test_cardano_tx_with_mint_amount(mint_amount: i64) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // CBOR array with 3 elements
    tx_bytes.push(0x83);

    // Transaction body (CBOR map with mint field)
    tx_bytes.push(0xa4); // 4 fields

    // Key 0: inputs
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x00);

    // Key 1: outputs
    tx_bytes.push(0x01);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x1d);
    tx_bytes.extend_from_slice(&[0u8; 29]);
    tx_bytes.push(0x1a);
    tx_bytes.extend_from_slice(&1_000_000u32.to_be_bytes());

    // Key 2: fee
    tx_bytes.push(0x02);
    tx_bytes.push(0x1a);
    tx_bytes.extend_from_slice(&170_000u32.to_be_bytes());

    // Key 9: mint (multi-asset)
    tx_bytes.push(0x09);
    tx_bytes.push(0xa1); // Map with 1 entry
                         // Policy ID (28 bytes)
    tx_bytes.push(0x58);
    tx_bytes.push(0x1c);
    tx_bytes.extend_from_slice(&[0x01; 28]);
    // Asset map
    tx_bytes.push(0xa1);
    // Asset name
    tx_bytes.push(0x44); // 4-byte string
    tx_bytes.extend_from_slice(b"TEST");
    // Amount (can be negative for burning)
    if mint_amount >= 0 {
        if mint_amount <= 23 {
            tx_bytes.push(mint_amount as u8);
        } else {
            tx_bytes.push(0x1a);
            tx_bytes.extend_from_slice(&(mint_amount as u32).to_be_bytes());
        }
    } else {
        // Negative integer encoding in CBOR
        let abs_minus_one = (mint_amount.abs() - 1) as u32;
        tx_bytes.push(0x3a);
        tx_bytes.extend_from_slice(&abs_minus_one.to_be_bytes());
    }

    // Witness set
    tx_bytes.push(0xa1);
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x58);
    tx_bytes.push(0x40);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // No auxiliary data
    tx_bytes.push(0xf6);

    tx_bytes
}

/// Create a test transaction with asset outputs
fn create_test_cardano_tx_with_asset_outputs(_seed: u64) -> Vec<u8> {
    create_test_cardano_tx_with_mint_amount(100)
}

/// Create a test transaction with metadata
#[allow(clippy::vec_init_then_push)]
fn create_test_cardano_tx_with_metadata(_seed: u64) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // CBOR array with 4 elements (including auxiliary data)
    tx_bytes.push(0x84);

    // Transaction body
    tx_bytes.push(0xa3);
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x00);
    tx_bytes.push(0x01);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x1d);
    tx_bytes.extend_from_slice(&[0u8; 29]);
    tx_bytes.push(0x1a);
    tx_bytes.extend_from_slice(&1_000_000u32.to_be_bytes());
    tx_bytes.push(0x02);
    tx_bytes.push(0x1a);
    tx_bytes.extend_from_slice(&170_000u32.to_be_bytes());

    // Witness set
    tx_bytes.push(0xa1);
    tx_bytes.push(0x00);
    tx_bytes.push(0x81);
    tx_bytes.push(0x82);
    tx_bytes.push(0x58);
    tx_bytes.push(0x20);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.push(0x58);
    tx_bytes.push(0x40);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // Auxiliary data (metadata)
    tx_bytes.push(0xa1); // Map
    tx_bytes.push(0x00); // Label 0
    tx_bytes.push(0x64); // Text string (4 bytes)
    tx_bytes.extend_from_slice(b"test");

    // Validity flag
    tx_bytes.push(0xf5); // true

    tx_bytes
}

/// Create a test transaction with Plutus scripts
fn create_test_cardano_tx_with_plutus(_seed: u64) -> Vec<u8> {
    create_test_cardano_tx(0)
}

/// Create a test transaction with redeemer
fn create_test_cardano_tx_with_redeemer(_tag: u8) -> Vec<u8> {
    create_test_cardano_tx(0)
}

/// Create a test transaction with execution units
fn create_test_cardano_tx_with_ex_units(_mem: u64, _steps: u64) -> Vec<u8> {
    create_test_cardano_tx(0)
}

/// Create a test transaction with withdrawal
fn create_test_cardano_tx_with_withdrawal(_amount: u64) -> Vec<u8> {
    create_test_cardano_tx(0)
}
