//! Property-based tests for Stellar decoder
//!
//! These tests use proptest to generate random inputs and verify
//! that the decoder handles them correctly without panicking.

use decoder_stellar::types::{
    DecoratedSignature, EnvelopeType, StellarAsset, StellarMemo, StellarOperation,
    StellarTransaction, TimeBounds,
};
use decoder_stellar::StellarDecoder;
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

// Import property test helpers
use decoder_test_utils::proptest_helpers::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Test that the decoder never panics on random bytes
    #[test]
    fn prop_stellar_decoder_never_panics(bytes in arb_small_bytes()) {
        prop_decoder_never_panics::<StellarDecoder>(&bytes);
    }

    /// Test that empty bytes are always rejected
    #[test]
    fn prop_stellar_rejects_empty(_unit in 0u8..1) {
        let result = StellarDecoder::decode(&[]);
        prop_assert!(result.is_err());
    }

    /// Test that very short inputs are rejected
    #[test]
    fn prop_stellar_rejects_short(bytes in prop::collection::vec(any::<u8>(), 0..4)) {
        let result = StellarDecoder::validate_format(&bytes);
        prop_assert!(result.is_err());
    }

    /// Test that valid transactions can be canonicalized
    #[test]
    fn prop_valid_tx_canonicalizes(
        fee in 100u32..1000000,
        seq in 1i64..1000000,
        amount in 1i64..1000000000,
    ) {
        let tx = StellarTransaction {
            source_account: vec![1; 32],
            fee,
            sequence_number: seq,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: vec![StellarOperation::Payment {
                destination: vec![2; 32],
                asset: StellarAsset::Native,
                amount,
            }],
            signatures: vec![DecoratedSignature {
                hint: [0; 4],
                signature: vec![0; 64],
            }],
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };

        let result = tx.canonicalize();
        prop_assert!(result.is_ok());

        let tx_ir = result.unwrap();
        prop_assert_eq!(tx_ir.operations.len(), 1);
        prop_assert!(!tx_ir.state_deltas.account_changes.is_empty());
    }

    /// Test that transaction validation works correctly
    #[test]
    fn prop_transaction_validation(
        op_count in 1usize..100,
        sig_count in 1usize..20,
    ) {
        let ops = vec![
            StellarOperation::Payment {
                destination: vec![2; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            };
            op_count
        ];

        let sigs = vec![
            DecoratedSignature {
                hint: [0; 4],
                signature: vec![0; 64],
            };
            sig_count
        ];

        let tx = StellarTransaction {
            source_account: vec![1; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: ops,
            signatures: sigs,
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };

        prop_assert!(tx.is_valid());
    }

    /// Test that asset types are handled correctly via canonicalization
    #[test]
    fn prop_asset_types_via_canonicalization(_code_byte in any::<u8>()) {
        // Test native asset
        let tx_native = StellarTransaction {
            source_account: vec![0; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: vec![StellarOperation::Payment {
                destination: vec![1; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            }],
            signatures: vec![DecoratedSignature {
                hint: [0; 4],
                signature: vec![0; 64],
            }],
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };

        let result = tx_native.canonicalize();
        prop_assert!(result.is_ok());
        let tx_ir = result.unwrap();
        if let Operation::Transfer(ref transfer) = tx_ir.operations[0] {
            prop_assert!(matches!(transfer.asset, AssetId::Native));
        }
    }

    /// Test that time bounds validation works correctly
    #[test]
    fn prop_time_bounds_validation(min in 0u64..1000000, max in 0u64..1000000) {
        let tb = TimeBounds {
            min_time: min,
            max_time: max,
        };

        if max == 0 || min <= max {
            prop_assert!(tb.is_valid());
        } else {
            prop_assert!(!tb.is_valid());
        }
    }

    /// Test that memo types are handled correctly
    #[test]
    fn prop_memo_handling(memo_id in any::<u64>()) {
        let tx = StellarTransaction {
            source_account: vec![1; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::Id(memo_id),
            operations: vec![StellarOperation::Payment {
                destination: vec![2; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            }],
            signatures: vec![DecoratedSignature {
                hint: [0; 4],
                signature: vec![0; 64],
            }],
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };

        let result = tx.canonicalize();
        prop_assert!(result.is_ok());

        // Memo should be in extra metadata
        let tx_ir = result.unwrap();
        let extra: serde_json::Value = serde_json::from_str(&tx_ir.metadata.extra).unwrap();
        prop_assert!(extra["memo"].as_str().unwrap().contains("Id"));
    }

    /// Test that operation count is preserved
    #[test]
    fn prop_operation_count_preserved(op_count in 1usize..100) {
        let ops = vec![
            StellarOperation::Payment {
                destination: vec![2; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            };
            op_count
        ];

        let tx = StellarTransaction {
            source_account: vec![1; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: ops,
            signatures: vec![DecoratedSignature {
                hint: [0; 4],
                signature: vec![0; 64],
            }],
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };

        let result = tx.canonicalize();
        prop_assert!(result.is_ok());

        let tx_ir = result.unwrap();
        // Each payment operation maps to 1 TxIR operation
        prop_assert_eq!(tx_ir.operations.len(), op_count);
    }

    /// Test that signature count is preserved
    #[test]
    fn prop_signature_count_preserved(sig_count in 1usize..20) {
        let sigs = vec![
            DecoratedSignature {
                hint: [0; 4],
                signature: vec![0; 64],
            };
            sig_count
        ];

        let tx = StellarTransaction {
            source_account: vec![1; 32],
            fee: 100,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: vec![StellarOperation::Payment {
                destination: vec![2; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            }],
            signatures: sigs,
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };

        let result = tx.canonicalize();
        prop_assert!(result.is_ok());

        let tx_ir = result.unwrap();
        prop_assert_eq!(tx_ir.authorization.signatures.len(), sig_count);
    }

    /// Test that fee is always reflected in state deltas
    #[test]
    fn prop_fee_in_state_deltas(fee in 100u32..1000000) {
        let tx = StellarTransaction {
            source_account: vec![1; 32],
            fee,
            sequence_number: 1,
            time_bounds: None,
            memo: StellarMemo::None,
            operations: vec![StellarOperation::Payment {
                destination: vec![2; 32],
                asset: StellarAsset::Native,
                amount: 1000000,
            }],
            signatures: vec![DecoratedSignature {
                hint: [0; 4],
                signature: vec![0; 64],
            }],
            raw_bytes: vec![],
            envelope_type: EnvelopeType::Tx,
            network_id: None,
        };

        let result = tx.canonicalize();
        prop_assert!(result.is_ok());

        let tx_ir = result.unwrap();
        prop_assert!(!tx_ir.state_deltas.account_changes.is_empty());

        // Source account should have negative balance change (fee)
        let source_change = &tx_ir.state_deltas.account_changes[0];
        prop_assert_eq!(source_change.balance_change, -(fee as i128));
    }
}

/// Test specific operation types
#[test]
fn test_operation_types() {
    let payment = StellarOperation::Payment {
        destination: vec![2; 32],
        asset: StellarAsset::Native,
        amount: 1000000,
    };
    assert!(payment.is_transfer());
    assert!(!payment.is_soroban());
    assert_eq!(payment.operation_type(), "Payment");

    let soroban = StellarOperation::InvokeHostFunction {
        function_type: 0,
        parameters: vec![],
    };
    assert!(!soroban.is_transfer());
    assert!(soroban.is_soroban());
    assert_eq!(soroban.operation_type(), "InvokeHostFunction");
}

/// Test asset code string conversion
#[test]
fn test_asset_code_string() {
    let native = StellarAsset::Native;
    assert_eq!(native.code_string(), "XLM");

    let usdc = StellarAsset::CreditAlphanum4 {
        code: [b'U', b'S', b'D', b'C'],
        issuer: vec![1; 32],
    };
    assert_eq!(usdc.code_string(), "USDC");

    // Test with null padding
    let padded = StellarAsset::CreditAlphanum4 {
        code: [b'U', b'S', b'D', 0],
        issuer: vec![1; 32],
    };
    assert_eq!(padded.code_string(), "USD");
}
