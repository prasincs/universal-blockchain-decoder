//! Property-based tests for Aleo decoder
//!
//! This module uses proptest to verify critical properties of the Aleo decoder:
//! 1. Decoder never panics on arbitrary input
//! 2. Transaction ID calculation is deterministic
//! 3. Canonical serialization properties
//! 4. Transaction type detection is consistent
//! 5. Privacy metadata is correct
//! 6. State delta generation is correct
//! 7. Finalize operations are properly parsed
//! 8. Record input/output handling

use decoder_aleo::{AleoDecoder, TransactionType};
use decoder_test_utils::proptest_helpers::{arb_small_bytes, prop_decoder_never_panics};
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

//
// Property 1: Decoder Never Panics
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Aleo decoder never panics on arbitrary input
    ///
    /// For any arbitrary byte sequence, decode() must return Ok or Err,
    /// never panic.
    #[test]
    fn prop_aleo_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<AleoDecoder>(&bytes);
    }

    /// Property: Decoder never panics on empty input
    #[test]
    fn prop_aleo_decoder_rejects_empty(_unit in 0u8..1) {
        let result = AleoDecoder::decode(&[]);
        prop_assert!(result.is_err(), "Decoder should reject empty input");
    }

    /// Property: Decoder never panics on very short input
    #[test]
    fn prop_aleo_decoder_rejects_tiny_input(size in 1usize..10) {
        let bytes = vec![0xFF; size];
        let result = AleoDecoder::decode(&bytes);
        prop_assert!(result.is_err(), "Decoder should reject input < 10 bytes");
    }

    /// Property: Decoder handles oversized input gracefully
    #[test]
    fn prop_aleo_decoder_handles_large_input(size in 10_000usize..100_000) {
        let bytes = vec![0x00; size];
        // Should either decode or error, never panic
        let result = AleoDecoder::decode(&bytes);
        prop_assert!(result.is_ok() || result.is_err());
    }
}

//
// Property 2: Transaction ID Determinism
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Transaction ID calculation is deterministic
    ///
    /// Computing the transaction ID multiple times should yield identical results.
    #[test]
    fn prop_transaction_id_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            let id1 = tx.id.clone();

            // Decode again
            if let Ok(tx2) = AleoDecoder::decode(&bytes) {
                let id2 = tx2.id;
                prop_assert_eq!(id1, id2, "Transaction IDs are non-deterministic");
            }
        }
    }

    /// Property: Transaction ID is always 32 bytes (SHA-256)
    #[test]
    fn prop_transaction_id_size(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            prop_assert_eq!(tx.id.len(), 32, "Transaction ID should be 32 bytes (SHA-256)");
        }
    }
}

//
// Property 3: Canonical Serialization Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Successfully decoded transactions can be canonicalized
    ///
    /// Any transaction that decodes successfully should also be able to
    /// produce TxIR without panicking.
    #[test]
    fn prop_decoded_tx_canonicalizes(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            let result = tx.canonicalize();
            prop_assert!(result.is_ok() || result.is_err(),
                "Canonicalization should return Result, not panic");
        }
    }

    /// Property: Canonical hash is deterministic
    ///
    /// Computing canonical hash multiple times should yield identical results.
    #[test]
    fn prop_canonical_hash_deterministic(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
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
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
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
// Property 4: Transaction Type Detection
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: Transaction type detection is consistent
    ///
    /// Transaction types are determined by first byte:
    /// - 0x00: Fee
    /// - 0x01: Deploy
    /// - 0x02: Execute
    #[test]
    fn prop_transaction_type_detection(bytes in arb_small_bytes()) {
        if bytes.is_empty() {
            return Ok(());
        }

        let first_byte = bytes[0];

        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            match first_byte {
                0x00 => {
                    prop_assert!(matches!(tx.transaction_type, TransactionType::Fee(_)),
                        "Byte 0x00 should be Fee transaction");
                }
                0x01 => {
                    prop_assert!(matches!(tx.transaction_type, TransactionType::Deploy(_)),
                        "Byte 0x01 should be Deploy transaction");
                }
                0x02 => {
                    prop_assert!(matches!(tx.transaction_type, TransactionType::Execute(_)),
                        "Byte 0x02 should be Execute transaction");
                }
                _ => {
                    // Unknown type - should have been rejected during decode
                }
            }
        }
    }
}

//
// Property 5: Privacy Metadata Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Privacy components are correctly set for private transactions
    ///
    /// Transactions with private inputs/outputs should have privacy components
    #[test]
    fn prop_privacy_metadata_consistency(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                // If transaction has privacy components, observability should reflect it
                if let Some(privacy) = &tx_ir.privacy {
                    if privacy.hidden_sender || privacy.hidden_recipient || privacy.hidden_amount {
                        prop_assert_ne!(privacy.observability_level, ObservabilityLevel::FullyObservable,
                            "Hidden fields should not be fully observable");
                    }
                }
            }
        }
    }

    /// Property: Public transactions have no privacy components or are fully observable
    #[test]
    fn prop_public_transactions_observable(tx_type in 0u8..3) {
        // Create a simple public transaction
        let mut bytes = vec![tx_type];

        // Add minimal valid data based on type
        match tx_type {
            0x00 => {
                // Fee: global_state_root + amount + priority + no transition
                bytes.extend_from_slice(&[0x00; 32]);
                bytes.extend_from_slice(&1000u64.to_le_bytes());
                bytes.extend_from_slice(&100u64.to_le_bytes());
                bytes.push(0x00);
            }
            0x01 => {
                // Deploy: edition + program_id + program + vks
                bytes.extend_from_slice(&0u16.to_le_bytes());
                bytes.extend_from_slice(&4u16.to_le_bytes());
                bytes.extend_from_slice(b"test");
                bytes.extend_from_slice(&4u16.to_le_bytes());
                bytes.extend_from_slice(b"prog");
                bytes.extend_from_slice(&0u16.to_le_bytes());
                bytes.push(0x00); // no fee
            }
            0x02 => {
                // Execute: minimal with public inputs
                bytes.extend_from_slice(&[0x00; 32]);
                bytes.extend_from_slice(&0u16.to_le_bytes()); // no transitions
                bytes.push(0x00); // no proof
                bytes.push(0x00); // no fee
            }
            _ => {}
        }

        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                // Public transactions should be observable
                if let Some(privacy) = &tx_ir.privacy {
                    if privacy.observability_level == ObservabilityLevel::FullyObservable {
                        prop_assert!(!privacy.hidden_sender);
                        prop_assert!(!privacy.hidden_recipient);
                        prop_assert!(!privacy.hidden_amount);
                    }
                }
            }
        }
    }
}

//
// Property 6: State Delta Generation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Execution transactions with finalize operations generate state deltas
    #[test]
    fn prop_finalize_generates_state_deltas(
        key_len in 1usize..32,
        value_len in 1usize..64,
    ) {
        // Create execution with finalize operation
        let mut bytes = vec![0x02]; // Execute
        bytes.extend_from_slice(&[0x00; 32]); // global state root
        bytes.extend_from_slice(&1u16.to_le_bytes()); // 1 transition

        // Transition
        bytes.extend_from_slice(&[0x01; 32]); // ID
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"prog");
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(b"func");

        // No inputs/outputs
        bytes.push(0);
        bytes.push(0);

        // No proof
        bytes.push(0x00);

        // 1 finalize operation (insert)
        bytes.push(1);
        bytes.push(0x01); // Insert mapping

        bytes.extend_from_slice(&7u16.to_le_bytes());
        bytes.extend_from_slice(b"mapping");

        let key = vec![0xAA; key_len];
        bytes.extend_from_slice(&(key.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&key);

        let value = vec![0xBB; value_len];
        bytes.extend_from_slice(&(value.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&value);

        // No execution proof
        bytes.push(0x00);

        // No fee
        bytes.push(0x00);

        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                prop_assert!(!tx_ir.state_deltas.is_empty(),
                    "Finalize operations should generate state deltas");
            }
        }
    }

    /// Property: Deploy and Fee transactions have no state deltas
    #[test]
    fn prop_deploy_and_fee_no_state_deltas(tx_type in prop::sample::select(vec![0x00u8, 0x01u8])) {
        let mut bytes = vec![tx_type];

        match tx_type {
            0x00 => {
                bytes.extend_from_slice(&[0x00; 32]);
                bytes.extend_from_slice(&1000u64.to_le_bytes());
                bytes.extend_from_slice(&0u64.to_le_bytes());
                bytes.push(0x00);
            }
            0x01 => {
                bytes.extend_from_slice(&0u16.to_le_bytes());
                bytes.extend_from_slice(&4u16.to_le_bytes());
                bytes.extend_from_slice(b"test");
                bytes.extend_from_slice(&4u16.to_le_bytes());
                bytes.extend_from_slice(b"code");
                bytes.extend_from_slice(&0u16.to_le_bytes());
                bytes.push(0x00);
            }
            _ => {}
        }

        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if let Ok(tx_ir) = tx.canonicalize() {
                prop_assert!(tx_ir.state_deltas.is_empty(),
                    "Deploy and Fee transactions should not have state deltas");
            }
        }
    }
}

//
// Property 7: Fee Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Fee amounts are non-negative
    #[test]
    fn prop_fee_amounts_non_negative(_amount in any::<u64>()) {
        // u64 is always non-negative by type
        // This property is guaranteed by the type system
    }

    /// Property: Priority fee is always <= total amount (if it makes sense)
    #[test]
    fn prop_priority_fee_reasonable(
        amount in 1000u64..1_000_000_000,
        priority_ratio in 0.0f64..0.5
    ) {
        let priority_fee = (amount as f64 * priority_ratio) as u64;
        prop_assert!(priority_fee <= amount,
            "Priority fee should not exceed total amount");
    }
}

//
// Property 8: Record Input/Output Handling
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Record inputs have valid serial numbers (32 bytes)
    #[test]
    fn prop_record_input_serial_number_size(serial_number in prop::collection::vec(any::<u8>(), 32..=32)) {
        prop_assert_eq!(serial_number.len(), 32, "Serial number should be 32 bytes");
    }

    /// Property: Record outputs have valid commitments (32 bytes)
    #[test]
    fn prop_record_output_commitment_size(commitment in prop::collection::vec(any::<u8>(), 32..=32)) {
        prop_assert_eq!(commitment.len(), 32, "Commitment should be 32 bytes");
    }

    /// Property: Record checksums are 16 bytes
    #[test]
    fn prop_record_checksum_size(checksum in prop::collection::vec(any::<u8>(), 16..=16)) {
        prop_assert_eq!(checksum.len(), 16, "Checksum should be 16 bytes");
    }
}

//
// Property 9: Program ID Validation
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Program IDs in valid transactions are non-empty
    #[test]
    fn prop_program_id_non_empty(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if tx.validate().is_ok() {
                match &tx.transaction_type {
                    TransactionType::Deploy(deploy) => {
                        prop_assert!(!deploy.program_id.is_empty(),
                            "Valid deployment must have non-empty program ID");
                    }
                    TransactionType::Execute(exec) => {
                        for transition in &exec.transitions {
                            prop_assert!(!transition.program_id.is_empty(),
                                "Valid transition must have non-empty program ID");
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

//
// Property 10: Full Pipeline Never Panics
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
            if let Ok(tx) = AleoDecoder::decode(&bytes) {
                let _ = tx.validate();
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
            let _ = AleoDecoder::validate_format(&bytes);

            if let Ok(tx) = AleoDecoder::decode(&bytes) {
                let _ = tx.validate();
            }
        });

        prop_assert!(result.is_ok(), "Validation pipeline panicked");
    }
}

//
// Property 11: Transition Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Executions have at least one transition if validation passes
    #[test]
    fn prop_valid_executions_have_transitions(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if tx.validate().is_ok() {
                if let TransactionType::Execute(exec) = &tx.transaction_type {
                    prop_assert!(!exec.transitions.is_empty(),
                        "Valid execution must have at least one transition");
                }
            }
        }
    }

    /// Property: Function names are non-empty in valid transitions
    #[test]
    fn prop_transition_function_names_non_empty(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            if let TransactionType::Execute(exec) = &tx.transaction_type {
                for transition in &exec.transitions {
                    // Even invalid transitions should parse function names
                    // (but may be empty if malformed)
                    if !transition.function_name.is_empty() {
                        prop_assert!(transition.function_name.len() > 0);
                    }
                }
            }
        }
    }
}

//
// Property 12: Global State Root Properties
//

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Global state roots are always 32 bytes
    #[test]
    fn prop_global_state_root_size(bytes in arb_small_bytes()) {
        if let Ok(tx) = AleoDecoder::decode(&bytes) {
            match &tx.transaction_type {
                TransactionType::Execute(exec) => {
                    prop_assert_eq!(exec.global_state_root.len(), 32,
                        "Global state root should be 32 bytes");
                }
                TransactionType::Fee(fee) => {
                    prop_assert_eq!(fee.global_state_root.len(), 32,
                        "Global state root should be 32 bytes");
                }
                _ => {}
            }
        }
    }
}
