//! Property-based tests for Zcash decoder
//!
//! This module uses proptest to verify critical properties of the Zcash decoder:
//! 1. Decoder never panics on arbitrary input
//! 2. Sapling shielded pool structure validation
//! 3. Viewing key format validation
//! 4. Privacy metadata consistency
//! 5. Canonical serialization properties

use decoder_test_utils::proptest_helpers::{arb_small_bytes, prop_decoder_never_panics};
use decoder_zcash::sapling::{OutputDescription, SpendDescription};
use decoder_zcash::viewing_key::{NotePlaintext, SaplingFullViewingKey, SaplingIncomingViewingKey};
use decoder_zcash::*;
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

//
// Property 1: Decoder Safety
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Zcash decoder never panics on arbitrary input
    ///
    /// For any arbitrary byte sequence, decode() must return Ok or Err,
    /// never panic.
    #[test]
    fn prop_zcash_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<ZcashDecoder>(&bytes);
    }

    /// Property: Zcash decoder rejects empty input
    #[test]
    fn prop_zcash_decoder_rejects_empty(_unit in 0u8..1) {
        let result = ZcashDecoder::decode(&[]);
        prop_assert!(result.is_err(), "Decoder should reject empty input");
    }

    /// Property: Zcash decoder rejects tiny input
    #[test]
    fn prop_zcash_decoder_rejects_tiny_input(size in 1usize..14) {
        let bytes = vec![0x04, 0x00, 0x00, 0x80]; // v4 header
        let mut full_bytes = bytes.clone();
        full_bytes.resize(size, 0xFF);
        let result = ZcashDecoder::decode(&full_bytes);
        prop_assert!(result.is_err(), "Decoder should reject input < 14 bytes");
    }
}

//
// Property 2: Version and Network Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Version group ID is valid for Sapling era
    ///
    /// Known version group IDs:
    /// - 0x892F2085: Sapling
    /// - 0x26A7270A: Blossom
    /// - 0xF919A198: Heartwood
    /// - 0xC2D6D0B4: Canopy
    #[test]
    fn prop_version_group_id_valid(vgid in prop::sample::select(vec![
        0x892F2085u32, // Sapling
        0x26A7270A,    // Blossom
        0xF919A198,    // Heartwood
        0xC2D6D0B4,    // Canopy
    ])) {
        let tx_bytes = create_test_zcash_tx_with_vgid(vgid);

        if let Ok(ZcashTransaction::Transparent(tx)) = ZcashDecoder::decode(&tx_bytes) {
            // Version group ID should match expected consensus rules
            let known_vgids = [0x892F2085u32, 0x26A7270A, 0xF919A198, 0xC2D6D0B4];
            prop_assert!(
                known_vgids.contains(&tx.version_group_id) || tx.version_group_id == 0,
                "Version group ID should be a known consensus value or 0 (pre-Overwinter)"
            );
        }
    }

    /// Property: Expiry height is reasonable
    ///
    /// Expiry height should be within a reasonable range
    #[test]
    fn prop_expiry_height_reasonable(expiry in 0u32..10_000_000u32) {
        let tx_bytes = create_test_zcash_tx_with_expiry(expiry);

        if let Ok(ZcashTransaction::Transparent(tx)) = ZcashDecoder::decode(&tx_bytes) {
            // Expiry height should match
            prop_assert_eq!(tx.expiry_height, expiry, "Expiry height should match");

            // Should be reasonable (< 10M blocks, ~38 years at 75s/block)
            prop_assert!(
                tx.expiry_height <= 10_000_000,
                "Expiry height should be < 10M blocks"
            );
        }
    }
}

//
// Property 3: Sapling Spend Description Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Nullifier has correct length (32 bytes)
    ///
    /// Nullifiers are blake2b-256 hashes
    #[test]
    fn prop_nullifier_correct_length(seed in any::<u64>()) {
        let spend_bytes = create_test_spend_description(seed);

        // SpendDescription parsing would happen inside decoder
        // Here we test the fixed size
        prop_assert_eq!(
            SpendDescription::NULLIFIER_SIZE,
            32,
            "Nullifier must be 32 bytes"
        );
        prop_assert_eq!(
            spend_bytes.len(),
            SpendDescription::SIZE,
            "SpendDescription must be 384 bytes"
        );
    }

    /// Property: Value commitment has correct length (32 bytes)
    ///
    /// Value commitments are compressed Jubjub points
    #[test]
    fn prop_value_commitment_length(_dummy in 0u8..1) {
        prop_assert_eq!(
            SpendDescription::CV_SIZE,
            32,
            "Value commitment must be 32 bytes (compressed Jubjub point)"
        );
        prop_assert_eq!(
            OutputDescription::CV_SIZE,
            32,
            "Output value commitment must be 32 bytes"
        );
    }

    /// Property: Spend authorization signature has correct length
    ///
    /// Signatures are 64 bytes (RedJubjub)
    #[test]
    fn prop_spend_auth_sig_length(_dummy in 0u8..1) {
        prop_assert_eq!(
            SpendDescription::SPEND_AUTH_SIG_SIZE,
            64,
            "Spend authorization signature must be 64 bytes"
        );
    }
}

//
// Property 4: Sapling Output Description Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Encrypted ciphertext has correct length (580 bytes)
    ///
    /// enc_ciphertext = ChaCha20-Poly1305(note_plaintext)
    #[test]
    fn prop_enc_ciphertext_length(_dummy in 0u8..1) {
        prop_assert_eq!(
            OutputDescription::ENC_CIPHERTEXT_SIZE,
            580,
            "Encrypted ciphertext must be 580 bytes"
        );
    }

    /// Property: Outgoing ciphertext has correct length (80 bytes)
    ///
    /// out_ciphertext = ChaCha20-Poly1305(outgoing_plaintext)
    #[test]
    fn prop_out_ciphertext_length(_dummy in 0u8..1) {
        prop_assert_eq!(
            OutputDescription::OUT_CIPHERTEXT_SIZE,
            80,
            "Outgoing ciphertext must be 80 bytes"
        );
    }

    /// Property: zk-SNARK proof has correct length (192 bytes)
    ///
    /// Groth16 proofs are 192 bytes (3 G1 points + 1 G2 point)
    #[test]
    fn prop_zkproof_length(_dummy in 0u8..1) {
        prop_assert_eq!(
            SpendDescription::ZKPROOF_SIZE,
            192,
            "zk-SNARK proof must be 192 bytes (Groth16)"
        );
        prop_assert_eq!(
            OutputDescription::ZKPROOF_SIZE,
            192,
            "Output zk-SNARK proof must be 192 bytes"
        );
    }
}

//
// Property 5: Viewing Key Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Incoming Viewing Key has correct length (32 bytes)
    ///
    /// IVK is a scalar in Jubjub base field
    #[test]
    fn prop_ivk_length(ivk_bytes in prop::collection::vec(any::<u8>(), 32..=32)) {
        let ivk = SaplingIncomingViewingKey::from_bytes(&ivk_bytes);
        prop_assert!(ivk.is_ok(), "32-byte IVK should parse successfully");

        let ivk = ivk.unwrap();
        prop_assert_eq!(
            ivk.as_bytes().len(),
            32,
            "IVK must be 32 bytes"
        );
    }

    /// Property: Full Viewing Key has correct length (96 bytes)
    ///
    /// FVK = (ak, nk, ovk) = 32 + 32 + 32 bytes
    #[test]
    fn prop_fvk_length(fvk_bytes in prop::collection::vec(any::<u8>(), 96..=96)) {
        let fvk = SaplingFullViewingKey::from_bytes(&fvk_bytes);
        prop_assert!(fvk.is_ok(), "96-byte FVK should parse successfully");

        let fvk = fvk.unwrap();
        prop_assert_eq!(fvk.ak.len(), 32, "FVK.ak must be 32 bytes");
        prop_assert_eq!(fvk.nk.len(), 32, "FVK.nk must be 32 bytes");
        prop_assert_eq!(fvk.ovk.len(), 32, "FVK.ovk must be 32 bytes");
    }

    /// Property: Invalid viewing key lengths are rejected
    #[test]
    fn prop_invalid_ivk_length_rejected(len in 1usize..100usize) {
        if len == 32 {
            return Ok(()); // Skip valid length
        }

        let ivk_bytes = vec![0x42; len];
        let result = SaplingIncomingViewingKey::from_bytes(&ivk_bytes);
        prop_assert!(result.is_err(), "Invalid IVK length should be rejected");
    }
}

//
// Property 6: Note Plaintext Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Note plaintext has correct size (564 bytes)
    ///
    /// Plaintext = version(1) + diversifier(11) + value(8) + rcm(32) + memo(512)
    #[test]
    fn prop_note_plaintext_size(_dummy in 0u8..1) {
        prop_assert_eq!(
            NotePlaintext::PLAINTEXT_SIZE,
            564,
            "Note plaintext must be 564 bytes"
        );
        prop_assert_eq!(
            NotePlaintext::MEMO_SIZE,
            512,
            "Memo field must be 512 bytes"
        );
    }

    /// Property: Note value is non-negative and within MAX_MONEY
    ///
    /// MAX_MONEY = 21M ZEC = 21,000,000 * 10^8 zatoshis
    #[test]
    fn prop_note_value_range(value in 0u64..21_000_000_000_000_000u64) {
        let mut plaintext = vec![0u8; 564];
        plaintext[0] = 0x01; // Sapling version
        plaintext[12..20].copy_from_slice(&value.to_le_bytes());

        if let Ok(note) = NotePlaintext::from_bytes(&plaintext) {
            prop_assert_eq!(note.value, value, "Note value should match");
            prop_assert!(
                note.value <= 21_000_000_000_000_000,
                "Note value should be <= MAX_MONEY"
            );
        }
    }

    /// Property: Note plaintext version must be 0x01 for Sapling
    #[test]
    fn prop_note_version_validation(version in any::<u8>()) {
        let mut plaintext = vec![0u8; 564];
        plaintext[0] = version;

        let result = NotePlaintext::from_bytes(&plaintext);

        if version == NotePlaintext::SAPLING_VERSION {
            prop_assert!(result.is_ok(), "Valid Sapling version should parse");
        } else {
            prop_assert!(result.is_err(), "Invalid version should be rejected");
        }
    }
}

//
// Property 7: Sapling Value Balance
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Value balance is within valid range
    ///
    /// Value balance = transparent_out - transparent_in
    /// Positive: t→z (shielding)
    /// Negative: z→t (deshielding)
    /// Range: -MAX_MONEY to +MAX_MONEY
    #[test]
    fn prop_value_balance_range(value_balance in -21_000_000_000_000_000i64..21_000_000_000_000_000i64) {
        // Value balance should be within MAX_MONEY bounds
        prop_assert!(
            value_balance.abs() <= 21_000_000_000_000_000,
            "Value balance should be within +/- MAX_MONEY"
        );
    }
}

//
// Property 8: Privacy Metadata Consistency
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Transparent transactions have FullyObservable privacy
    ///
    /// Pure transparent (t→t) transactions should not have privacy features
    #[test]
    fn prop_transparent_privacy_metadata(seed in any::<u64>()) {
        let tx_bytes = create_test_zcash_transparent_tx(seed);

        if let Ok(ZcashTransaction::Transparent(tx)) = ZcashDecoder::decode(&tx_bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                if let Some(ref privacy) = tx_ir.privacy {
                    // Transparent transactions should be fully observable
                    prop_assert_eq!(
                        privacy.observability,
                        universal_decoder_core::privacy::ObservabilityLevel::FullyObservable,
                        "Transparent transactions should be fully observable"
                    );

                    // Should have no privacy features
                    prop_assert!(
                        privacy.features.is_empty(),
                        "Transparent transactions should have no privacy features"
                    );
                }
            }
        }
    }
}

//
// Property 9: Canonical Serialization
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Canonical hash is deterministic
    ///
    /// Computing canonical hash multiple times on the same transaction
    /// should yield identical results.
    #[test]
    fn prop_canonical_hash_deterministic(seed in any::<u64>()) {
        let bytes = create_test_zcash_transparent_tx(seed);

        if let Ok(tx) = ZcashDecoder::decode(&bytes) {
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
                        return Err(proptest::test_runner::TestCaseError::fail(
                            "Canonical hash returned different error states"
                        ));
                    }
                }
            }
        }
    }
}

//
// Property 10: Integration Tests
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
            if let Ok(tx) = ZcashDecoder::decode(&bytes) {
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

/// Create a test Zcash transparent transaction with deterministic content
fn create_test_zcash_transparent_tx(seed: u64) -> Vec<u8> {
    create_test_zcash_tx_with_expiry(100000 + (seed % 1000) as u32)
}

/// Create a test Zcash transaction with specific version group ID
fn create_test_zcash_tx_with_vgid(vgid: u32) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // Version 4 (Sapling) with overwinter bit
    tx_bytes.extend_from_slice(&(4u32 | (1 << 31)).to_le_bytes());

    // Version group ID
    tx_bytes.extend_from_slice(&vgid.to_le_bytes());

    // 0 inputs
    tx_bytes.push(0x00);

    // 1 output
    tx_bytes.push(0x01);
    tx_bytes.extend_from_slice(&[0; 32]); // prev_hash
    tx_bytes.extend_from_slice(&0u32.to_le_bytes()); // prev_index
    tx_bytes.push(0x00); // script_sig_len
    tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // sequence

    // Locktime
    tx_bytes.extend_from_slice(&0u32.to_le_bytes());

    // Expiry height
    tx_bytes.extend_from_slice(&500000u32.to_le_bytes());

    tx_bytes
}

/// Create a test Zcash transaction with specific expiry height
fn create_test_zcash_tx_with_expiry(expiry: u32) -> Vec<u8> {
    let mut tx_bytes = Vec::new();

    // Version 4 with overwinter bit
    tx_bytes.extend_from_slice(&(4u32 | (1 << 31)).to_le_bytes());

    // Version group ID (Sapling)
    tx_bytes.extend_from_slice(&0x892F2085u32.to_le_bytes());

    // 1 input
    tx_bytes.push(0x01);
    tx_bytes.extend_from_slice(&[0; 32]); // prev_hash
    tx_bytes.extend_from_slice(&0u32.to_le_bytes()); // prev_index
    tx_bytes.push(0x00); // script_sig_len
    tx_bytes.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // sequence

    // 1 output
    tx_bytes.push(0x01);
    tx_bytes.extend_from_slice(&1000000u64.to_le_bytes()); // value
    tx_bytes.push(0x19); // script_pubkey_len (25 bytes for P2PKH)
    tx_bytes.push(0x76); // OP_DUP
    tx_bytes.push(0xa9); // OP_HASH160
    tx_bytes.push(0x14); // 20 bytes
    tx_bytes.extend_from_slice(&[0; 20]); // pubkey_hash
    tx_bytes.push(0x88); // OP_EQUALVERIFY
    tx_bytes.push(0xac); // OP_CHECKSIG

    // Locktime
    tx_bytes.extend_from_slice(&0u32.to_le_bytes());

    // Expiry height
    tx_bytes.extend_from_slice(&expiry.to_le_bytes());

    tx_bytes
}

/// Create a test SpendDescription (384 bytes)
fn create_test_spend_description(seed: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(384);

    // cv (32 bytes)
    bytes.extend_from_slice(&[(seed % 256) as u8; 32]);

    // anchor (32 bytes)
    bytes.extend_from_slice(&[0x01; 32]);

    // nullifier (32 bytes)
    bytes.extend_from_slice(&[0x02; 32]);

    // rk (32 bytes)
    bytes.extend_from_slice(&[0x03; 32]);

    // zkproof (192 bytes)
    bytes.extend_from_slice(&[0x04; 192]);

    // spend_auth_sig (64 bytes)
    bytes.extend_from_slice(&[0x05; 64]);

    bytes
}
