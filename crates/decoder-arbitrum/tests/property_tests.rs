//! Property-based tests for Arbitrum decoder
//!
//! These tests use proptest to verify properties hold for arbitrary inputs.

use decoder_arbitrum::*;
use proptest::prelude::*;
use universal_decoder_core::prelude::*;

// ============================================================================
// Test Strategies (Generators)
// ============================================================================

/// Generate arbitrary chain IDs in Arbitrum range
fn arb_arbitrum_chain_id() -> impl Strategy<Value = u64> {
    prop_oneof![
        Just(42161u64),     // Arbitrum One
        Just(42170u64),     // Arbitrum Nova
        Just(421614u64),    // Arbitrum Sepolia
        Just(421613u64),    // Arbitrum Goerli
        42000u64..43000u64, // Custom Orbit chains
    ]
}

/// Generate arbitrary addresses
fn arb_address() -> impl Strategy<Value = [u8; 20]> {
    prop::array::uniform20(any::<u8>())
}

/// Generate arbitrary hashes
fn arb_hash() -> impl Strategy<Value = [u8; 32]> {
    prop::array::uniform32(any::<u8>())
}

/// Generate arbitrary transaction bytes
fn arb_tx_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1000)
}

/// Generate valid Arbitrum transaction type bytes
#[allow(dead_code)]
fn arb_valid_tx_type() -> impl Strategy<Value = u8> {
    prop_oneof![
        Just(0x00u8),
        Just(0x01u8),
        Just(0x02u8),
        Just(0x64u8),
        Just(0x65u8),
        Just(0x66u8),
        Just(0x68u8),
        Just(0x69u8),
        Just(0x6Au8),
        0xC0u8..=0xFFu8, // Legacy RLP
    ]
}

/// Generate arbitrary deposit transaction
fn arb_deposit_tx() -> impl Strategy<Value = DepositTransaction> {
    (
        arb_arbitrum_chain_id(),
        any::<u64>().prop_filter("l1_block_number must be > 0", |&n| n > 0),
        arb_address(),
        prop::option::of(arb_address()),
        any::<u128>(),
        1u64..1_000_000_000u64, // gas_limit must be > 0
        prop::collection::vec(any::<u8>(), 0..100),
    )
        .prop_map(
            |(chain_id, l1_block_number, from, to, value, gas_limit, data)| DepositTransaction {
                chain_id,
                l1_block_number,
                from,
                to,
                value,
                gas_limit,
                data,
            },
        )
}

/// Generate arbitrary unsigned transaction
fn arb_unsigned_tx() -> impl Strategy<Value = UnsignedTransaction> {
    (
        arb_arbitrum_chain_id(),
        arb_address(),
        arb_address(),
        any::<u128>(),
        1u64..1_000_000_000u64, // gas_limit must be > 0
        any::<u128>(),
        any::<u64>(),
        prop::collection::vec(any::<u8>(), 0..100),
    )
        .prop_map(
            |(chain_id, from, to, value, gas_limit, gas_price, nonce, data)| UnsignedTransaction {
                chain_id,
                from,
                to,
                value,
                gas_limit,
                gas_price,
                nonce,
                data,
            },
        )
}

/// Generate arbitrary submit retryable transaction
fn arb_submit_retryable_tx() -> impl Strategy<Value = SubmitRetryableTransaction> {
    (
        arb_arbitrum_chain_id(),
        arb_hash(),
        any::<u128>(),
        any::<u128>(),
        any::<u128>(),
        1u128..u64::MAX as u128, // gas_fee_cap must be > 0
        1u64..1_000_000_000u64,  // gas_limit must be > 0
        any::<u128>(),
        arb_address(),
        arb_address(),
        arb_address(),
        prop::collection::vec(any::<u8>(), 0..100),
    )
        .prop_map(
            |(
                chain_id,
                request_id,
                l1_base_fee,
                deposit,
                callvalue_raw,
                gas_fee_cap,
                gas_limit,
                max_submission_fee,
                fee_refund_address,
                beneficiary,
                retry_to,
                retry_data,
            )| {
                // Ensure callvalue <= deposit
                let callvalue = callvalue_raw.min(deposit);
                SubmitRetryableTransaction {
                    chain_id,
                    request_id,
                    l1_base_fee,
                    deposit,
                    callvalue,
                    gas_fee_cap,
                    gas_limit,
                    max_submission_fee,
                    fee_refund_address,
                    beneficiary,
                    retry_to,
                    retry_data,
                }
            },
        )
}

/// Generate arbitrary internal transaction
fn arb_internal_tx() -> impl Strategy<Value = InternalTransaction> {
    (
        arb_arbitrum_chain_id(),
        prop_oneof![
            Just(InternalTxType::UpdateL1BlockNumber),
            any::<u8>().prop_map(InternalTxType::Unknown)
        ],
        any::<u64>().prop_filter("l1_block_number must be > 0", |&n| n > 0),
        any::<u128>(),
        any::<u64>(),
    )
        .prop_map(
            |(chain_id, internal_type, l1_block_number, l1_base_fee, l1_timestamp)| {
                InternalTransaction {
                    chain_id,
                    internal_type,
                    l1_block_number,
                    l1_base_fee,
                    l1_timestamp,
                }
            },
        )
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Decoder never panics on arbitrary input
    #[test]
    fn prop_decoder_never_panics(bytes in arb_tx_bytes()) {
        let _ = ArbitrumDecoder::decode(&bytes);
    }

    /// Property: Valid format check never panics
    #[test]
    fn prop_validate_format_never_panics(bytes in arb_tx_bytes()) {
        let _ = ArbitrumDecoder::validate_format(&bytes);
    }

    /// Property: Chain ID detection is deterministic
    #[test]
    fn prop_chain_id_detection_deterministic(chain_id in any::<u64>()) {
        let result1 = ArbitrumChain::from_chain_id(chain_id);
        let result2 = ArbitrumChain::from_chain_id(chain_id);

        match (result1, result2) {
            (Some(c1), Some(c2)) => prop_assert_eq!(c1.chain_id, c2.chain_id),
            (None, None) => {},
            _ => prop_assert!(false, "Chain detection not deterministic"),
        }
    }

    /// Property: Valid Arbitrum chain IDs are always detected
    #[test]
    fn prop_valid_chain_ids_detected(chain_id in arb_arbitrum_chain_id()) {
        let chain = ArbitrumChain::from_chain_id(chain_id);
        prop_assert!(chain.is_some(), "Valid Arbitrum chain ID not detected: {}", chain_id);
    }

    /// Property: Chain ID range validation is correct
    #[test]
    fn prop_chain_id_range_validation(chain_id in any::<u64>()) {
        let detected = ArbitrumChain::from_chain_id(chain_id).is_some();

        // Known chains or in Arbitrum range (42xxx)
        let expected = matches!(chain_id, 42161 | 42170 | 421613 | 421614)
            || (42000..43000).contains(&chain_id);

        prop_assert_eq!(detected, expected);
    }

    /// Property: Deposit validation is consistent
    #[test]
    fn prop_deposit_validation_consistent(deposit in arb_deposit_tx()) {
        let result1 = deposit.validate();
        let result2 = deposit.validate();

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
    }

    /// Property: Deposit with zero gas limit always fails
    #[test]
    fn prop_deposit_zero_gas_fails(
        chain_id in arb_arbitrum_chain_id(),
        l1_block in 1u64..1_000_000u64,
        from in arb_address(),
        value in any::<u128>(),
    ) {
        let deposit = DepositTransaction {
            chain_id,
            l1_block_number: l1_block,
            from,
            to: None,
            value,
            gas_limit: 0,
            data: vec![],
        };

        prop_assert!(deposit.validate().is_err());
    }

    /// Property: Unsigned transaction validation is consistent
    #[test]
    fn prop_unsigned_validation_consistent(unsigned in arb_unsigned_tx()) {
        let result1 = unsigned.validate();
        let result2 = unsigned.validate();

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
    }

    /// Property: Submit retryable validation is consistent
    #[test]
    fn prop_retryable_validation_consistent(retryable in arb_submit_retryable_tx()) {
        let result1 = retryable.validate();
        let result2 = retryable.validate();

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
    }

    /// Property: Retryable with callvalue > deposit always fails
    #[test]
    fn prop_retryable_callvalue_exceeds_deposit_fails(
        chain_id in arb_arbitrum_chain_id(),
        deposit in 0u128..1_000_000u128,
    ) {
        let retryable = SubmitRetryableTransaction {
            chain_id,
            request_id: [0u8; 32],
            l1_base_fee: 1_000_000,
            deposit,
            callvalue: deposit + 1, // Exceeds deposit
            gas_fee_cap: 10_000_000,
            gas_limit: 100_000,
            max_submission_fee: 10_000,
            fee_refund_address: [0u8; 20],
            beneficiary: [0u8; 20],
            retry_to: [0u8; 20],
            retry_data: vec![],
        };

        prop_assert!(retryable.validate().is_err());
    }

    /// Property: Retryable with zero gas fee cap always fails
    #[test]
    fn prop_retryable_zero_gas_fee_cap_fails(
        chain_id in arb_arbitrum_chain_id(),
        deposit in any::<u128>(),
    ) {
        let retryable = SubmitRetryableTransaction {
            chain_id,
            request_id: [0u8; 32],
            l1_base_fee: 1_000_000,
            deposit,
            callvalue: deposit.min(1_000),
            gas_fee_cap: 0, // Zero gas fee cap
            gas_limit: 100_000,
            max_submission_fee: 10_000,
            fee_refund_address: [0u8; 20],
            beneficiary: [0u8; 20],
            retry_to: [0u8; 20],
            retry_data: vec![],
        };

        prop_assert!(retryable.validate().is_err());
    }

    /// Property: Internal transaction validation is consistent
    #[test]
    fn prop_internal_validation_consistent(internal in arb_internal_tx()) {
        let result1 = internal.validate();
        let result2 = internal.validate();

        prop_assert_eq!(result1.is_ok(), result2.is_ok());
    }

    /// Property: Internal with zero L1 block number always fails
    #[test]
    fn prop_internal_zero_block_fails(chain_id in arb_arbitrum_chain_id()) {
        let internal = InternalTransaction {
            chain_id,
            internal_type: InternalTxType::UpdateL1BlockNumber,
            l1_block_number: 0,
            l1_base_fee: 1_000_000,
            l1_timestamp: 1234567890,
        };

        prop_assert!(internal.validate().is_err());
    }

    /// Property: Canonicalization succeeds for valid transactions
    /// Note: Determinism testing (same input → same output) is better done via
    /// fixture-based regression tests rather than calling twice immediately,
    /// which won't catch time-dependent bugs effectively.
    #[test]
    fn prop_deposit_canonicalization_succeeds(deposit in arb_deposit_tx()) {
        prop_assume!(deposit.validate().is_ok());
        let tx = ArbitrumTransaction::Deposit(deposit);

        let ir = tx.canonicalize();
        prop_assert!(ir.is_ok(), "Valid deposit should canonicalize successfully");

        if let Ok(ir) = ir {
            let bytes = ir.to_canonical_bytes();
            prop_assert!(bytes.is_ok(), "Canonical TxIR should serialize successfully");
        }
    }

    /// Property: Canonicalization succeeds for valid unsigned transactions
    #[test]
    fn prop_unsigned_canonicalization_succeeds(unsigned in arb_unsigned_tx()) {
        prop_assume!(unsigned.validate().is_ok());
        let tx = ArbitrumTransaction::Unsigned(unsigned);

        let ir = tx.canonicalize();
        prop_assert!(ir.is_ok(), "Valid unsigned tx should canonicalize successfully");
    }

    /// Property: Canonicalization succeeds for valid retryable transactions
    #[test]
    fn prop_retryable_canonicalization_succeeds(retryable in arb_submit_retryable_tx()) {
        prop_assume!(retryable.validate().is_ok());
        let tx = ArbitrumTransaction::SubmitRetryable(retryable);

        let ir = tx.canonicalize();
        prop_assert!(ir.is_ok(), "Valid retryable should canonicalize successfully");
    }

    /// Property: Canonicalization succeeds for valid internal transactions
    #[test]
    fn prop_internal_canonicalization_succeeds(internal in arb_internal_tx()) {
        prop_assume!(internal.validate().is_ok());
        let tx = ArbitrumTransaction::Internal(internal);

        let ir = tx.canonicalize();
        prop_assert!(ir.is_ok(), "Valid internal tx should canonicalize successfully");
    }

    /// Property: Gas price conversion is deterministic
    #[test]
    fn prop_gas_price_conversion_deterministic(gas_price in any::<u128>()) {
        let converted1 = gas_price.min(u64::MAX as u128) as u64;
        let converted2 = gas_price.min(u64::MAX as u128) as u64;

        prop_assert_eq!(converted1, converted2);
    }

    /// Property: Gas price conversion never overflows
    #[test]
    fn prop_gas_price_conversion_no_overflow(gas_price in any::<u128>()) {
        let converted = gas_price.min(u64::MAX as u128) as u64;

        if gas_price <= u64::MAX as u128 {
            prop_assert_eq!(converted as u128, gas_price);
        } else {
            prop_assert_eq!(converted, u64::MAX);
        }
    }

    /// Property: Transaction type IDs are unique
    #[test]
    fn prop_tx_type_ids_unique(_unit in Just(())) {
        let types = [
            DepositTransaction::TYPE_ID,
            UnsignedTransaction::TYPE_ID,
            ContractTransaction::TYPE_ID,
            RetryTransaction::TYPE_ID,
            SubmitRetryableTransaction::TYPE_ID,
            InternalTransaction::TYPE_ID,
        ];

        for (i, &t1) in types.iter().enumerate() {
            for (j, &t2) in types.iter().enumerate() {
                if i != j {
                    prop_assert_ne!(t1, t2, "Type IDs not unique: 0x{:02X}", t1);
                }
            }
        }
    }

    /// Property: Mainnet/testnet detection is mutually exclusive
    #[test]
    fn prop_mainnet_testnet_mutually_exclusive(chain_id in arb_arbitrum_chain_id()) {
        if let Some(chain) = ArbitrumChain::from_chain_id(chain_id) {
            let is_mainnet = chain.is_mainnet();
            let is_testnet = chain.is_testnet();

            // Known chains should be exactly one
            if matches!(chain_id, 42161 | 42170 | 421613 | 421614) {
                prop_assert!(is_mainnet ^ is_testnet, "Chain must be either mainnet or testnet");
            } else {
                // Custom Orbit chains default to neither
                prop_assert!(!is_mainnet && !is_testnet);
            }
        }
    }
}
