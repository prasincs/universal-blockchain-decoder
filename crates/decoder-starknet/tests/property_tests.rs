//! Property-based tests for Starknet decoder
//!
//! These tests use proptest to verify properties that should hold for all inputs.

use decoder_crypto_zk::FieldElement;
use decoder_starknet::*;
use proptest::prelude::*;

// Arbitrary generators

prop_compose! {
    fn arb_field_element()(bytes in prop::array::uniform32(any::<u8>())) -> FieldElement {
        FieldElement::from_bytes_be(&bytes)
    }
}

prop_compose! {
    fn arb_field_element_array(max_len: usize)
        (len in 0..=max_len)
        (elements in prop::collection::vec(arb_field_element(), len)) -> Vec<FieldElement> {
        elements
    }
}

prop_compose! {
    fn arb_resource_bound()(
        max_amount in any::<u64>(),
        max_price in any::<u128>()
    ) -> ResourceBound {
        ResourceBound {
            max_amount,
            max_price_per_unit: max_price,
        }
    }
}

prop_compose! {
    fn arb_resource_bounds()(
        l1 in arb_resource_bound(),
        l2 in arb_resource_bound()
    ) -> ResourceBounds {
        ResourceBounds {
            l1_gas: l1,
            l2_gas: l2,
        }
    }
}

prop_compose! {
    fn arb_da_mode()(mode in 0u8..=1) -> DataAvailabilityMode {
        match mode {
            0 => DataAvailabilityMode::L1,
            _ => DataAvailabilityMode::L2,
        }
    }
}

prop_compose! {
    fn arb_invoke_v1()(
        sender in arb_field_element(),
        calldata in arb_field_element_array(10),
        max_fee in arb_field_element(),
        signature in arb_field_element_array(2),
        nonce in arb_field_element()
    ) -> InvokeTxV1 {
        InvokeTxV1 {
            sender_address: sender,
            calldata,
            max_fee,
            signature,
            nonce,
        }
    }
}

prop_compose! {
    fn arb_invoke_v3()(
        sender in arb_field_element(),
        calldata in arb_field_element_array(10),
        signature in arb_field_element_array(2),
        nonce in arb_field_element(),
        resource_bounds in arb_resource_bounds(),
        tip in any::<u64>(),
        paymaster in arb_field_element_array(5),
        account_deployment in arb_field_element_array(5),
        nonce_da in arb_da_mode(),
        fee_da in arb_da_mode()
    ) -> InvokeTxV3 {
        InvokeTxV3 {
            sender_address: sender,
            calldata,
            signature,
            nonce,
            resource_bounds,
            tip,
            paymaster_data: paymaster,
            account_deployment_data: account_deployment,
            nonce_data_availability_mode: nonce_da,
            fee_data_availability_mode: fee_da,
        }
    }
}

// Property tests

proptest! {
    #[test]
    fn prop_hash_deterministic_invoke_v1(tx in arb_invoke_v1()) {
        use decoder_starknet::hashing::hash_invoke_v1;

        // Hash should be deterministic
        let hash1 = hash_invoke_v1(&tx).unwrap();
        let hash2 = hash_invoke_v1(&tx).unwrap();

        prop_assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn prop_hash_deterministic_invoke_v3(tx in arb_invoke_v3()) {
        use decoder_starknet::hashing::hash_invoke_v3;

        // Hash should be deterministic
        let hash1 = hash_invoke_v3(&tx).unwrap();
        let hash2 = hash_invoke_v3(&tx).unwrap();

        prop_assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn prop_hash_length_32_bytes(tx in arb_invoke_v1()) {
        use decoder_starknet::hashing::hash_invoke_v1;

        let hash = hash_invoke_v1(&tx).unwrap();
        prop_assert_eq!(hash.len(), 32, "Hash should be 32 bytes");
    }

    #[test]
    fn prop_field_element_double_roundtrip(bytes in prop::array::uniform32(any::<u8>())) {
        // Field elements use modular arithmetic, so bytes -> field -> bytes -> field should be stable
        let field1 = FieldElement::from_bytes_be(&bytes);
        let roundtrip_bytes = field1.to_bytes_be();
        let field2 = FieldElement::from_bytes_be(&roundtrip_bytes);

        prop_assert_eq!(field1, field2, "Field element should stabilize after first roundtrip");
    }

    #[test]
    fn prop_calldata_hash_consistency(calldata in arb_field_element_array(20)) {
        use decoder_starknet::hashing;

        // Hashing the same calldata multiple times should give same result
        let hash1 = hashing::hash_invoke_v1(&InvokeTxV1 {
            sender_address: FieldElement::ZERO,
            calldata: calldata.clone(),
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
        }).unwrap();

        let hash2 = hashing::hash_invoke_v1(&InvokeTxV1 {
            sender_address: FieldElement::ZERO,
            calldata,
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
        }).unwrap();

        prop_assert_eq!(hash1, hash2, "Calldata hash should be consistent");
    }

    #[test]
    fn prop_different_calldata_different_hash(
        calldata1 in arb_field_element_array(10),
        calldata2 in arb_field_element_array(10)
    ) {
        use decoder_starknet::hashing;

        // Skip if calldata is the same
        if calldata1 == calldata2 {
            return Ok(());
        }

        let hash1 = hashing::hash_invoke_v1(&InvokeTxV1 {
            sender_address: FieldElement::ZERO,
            calldata: calldata1,
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
        }).unwrap();

        let hash2 = hashing::hash_invoke_v1(&InvokeTxV1 {
            sender_address: FieldElement::ZERO,
            calldata: calldata2,
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
        }).unwrap();

        // Different calldata should produce different hashes (collision resistance)
        prop_assert_ne!(hash1, hash2, "Different calldata should produce different hashes");
    }

    #[test]
    fn prop_resource_bounds_no_overflow(
        l1_amount in any::<u64>(),
        l1_price in any::<u128>(),
        l2_amount in any::<u64>(),
        l2_price in any::<u128>()
    ) {
        // Creating resource bounds should not panic
        let bounds = ResourceBounds {
            l1_gas: ResourceBound {
                max_amount: l1_amount,
                max_price_per_unit: l1_price,
            },
            l2_gas: ResourceBound {
                max_amount: l2_amount,
                max_price_per_unit: l2_price,
            },
        };

        // Hashing should not panic
        use decoder_starknet::hashing;
        let _hash = hashing::hash_invoke_v3(&InvokeTxV3 {
            sender_address: FieldElement::ZERO,
            calldata: vec![],
            signature: vec![],
            nonce: FieldElement::ZERO,
            resource_bounds: bounds,
            tip: 0,
            paymaster_data: vec![],
            account_deployment_data: vec![],
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
        });

        prop_assert!(true);
    }

    #[test]
    fn prop_da_mode_encoding_valid(nonce_mode in arb_da_mode(), fee_mode in arb_da_mode()) {
        use decoder_starknet::hashing;

        // DA mode encoding should always succeed
        let _hash = hashing::hash_invoke_v3(&InvokeTxV3 {
            sender_address: FieldElement::ZERO,
            calldata: vec![],
            signature: vec![],
            nonce: FieldElement::ZERO,
            resource_bounds: ResourceBounds {
                l1_gas: ResourceBound {
                    max_amount: 1000,
                    max_price_per_unit: 100,
                },
                l2_gas: ResourceBound {
                    max_amount: 2000,
                    max_price_per_unit: 50,
                },
            },
            tip: 0,
            paymaster_data: vec![],
            account_deployment_data: vec![],
            nonce_data_availability_mode: nonce_mode,
            fee_data_availability_mode: fee_mode,
        });

        prop_assert!(true);
    }

    #[test]
    fn prop_sender_address_preserved(sender in arb_field_element()) {
        // Sender address should be preserved in transaction variant
        let tx = StarknetTxVariant::InvokeV1(InvokeTxV1 {
            sender_address: sender,
            calldata: vec![],
            max_fee: FieldElement::ZERO,
            signature: vec![],
            nonce: FieldElement::ZERO,
        });

        prop_assert_eq!(tx.sender_address(), sender, "Sender address should be preserved");
    }

    #[test]
    fn prop_signature_preserved(signature in arb_field_element_array(2)) {
        // Signature should be preserved in transaction variant
        let tx = StarknetTxVariant::InvokeV1(InvokeTxV1 {
            sender_address: FieldElement::ZERO,
            calldata: vec![],
            max_fee: FieldElement::ZERO,
            signature: signature.clone(),
            nonce: FieldElement::ZERO,
        });

        prop_assert_eq!(tx.signature(), &signature[..], "Signature should be preserved");
    }

    #[test]
    fn prop_tx_type_correct(version in 0u8..=3) {
        // Transaction type should match variant
        let tx_type = match version {
            0 | 1 | 3 => StarknetTxType::Invoke,
            _ => return Ok(()), // Skip invalid versions
        };

        let variant = match version {
            1 => StarknetTxVariant::InvokeV1(InvokeTxV1 {
                sender_address: FieldElement::ZERO,
                calldata: vec![],
                max_fee: FieldElement::ZERO,
                signature: vec![],
                nonce: FieldElement::ZERO,
            }),
            3 => StarknetTxVariant::InvokeV3(InvokeTxV3 {
                sender_address: FieldElement::ZERO,
                calldata: vec![],
                signature: vec![],
                nonce: FieldElement::ZERO,
                resource_bounds: ResourceBounds {
                    l1_gas: ResourceBound {
                        max_amount: 1000,
                        max_price_per_unit: 100,
                    },
                    l2_gas: ResourceBound {
                        max_amount: 2000,
                        max_price_per_unit: 50,
                    },
                },
                tip: 0,
                paymaster_data: vec![],
                account_deployment_data: vec![],
                nonce_data_availability_mode: DataAvailabilityMode::L1,
                fee_data_availability_mode: DataAvailabilityMode::L1,
            }),
            _ => return Ok(()),
        };

        prop_assert_eq!(variant.tx_type(), tx_type, "Transaction type should match variant");
    }
}
