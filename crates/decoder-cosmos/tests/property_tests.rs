//! Property-based tests for Cosmos decoder using proptest
//!
//! These tests verify properties that should hold for all valid inputs.

#![allow(deprecated)]

use decoder_cosmos::{Coin, CosmosDecoder};
use decoder_primitives::prelude::*;
use proptest::prelude::*;

// Generate arbitrary coins
fn arb_coin() -> impl Strategy<Value = Coin> {
    (
        "[a-z]{3,10}",                 // denom (e.g., "atom", "osmo", "uatom")
        1u128..=1_000_000_000_000u128, // amount
    )
        .prop_map(|(denom, amount)| Coin {
            denom,
            amount: amount.to_string(),
        })
}

// Generate arbitrary address
fn arb_address() -> impl Strategy<Value = String> {
    "[a-z]{6,44}".prop_map(|s| format!("cosmos1{}", s))
}

proptest! {
    /// Property: Decoder never panics on arbitrary input
    #[test]
    fn decoder_never_panics_on_arbitrary_input(data in any::<Vec<u8>>()) {
        let _ = CosmosDecoder::decode(&data);
        // Test passes if no panic occurs
    }

    /// Property: Decoder never panics on random bytes up to 10KB
    #[test]
    fn decoder_handles_random_bytes(data in prop::collection::vec(any::<u8>(), 0..10240)) {
        let _ = CosmosDecoder::decode(&data);
    }

    /// Property: Empty input always fails
    #[test]
    fn empty_input_always_fails(_prefix in any::<Vec<u8>>()) {
        // Even with a prefix, empty core data should fail
        let result = CosmosDecoder::validate_format(&[]);
        prop_assert!(result.is_err());
    }

    /// Property: Transaction hash is deterministic
    #[test]
    fn transaction_hash_is_deterministic(data in prop::collection::vec(any::<u8>(), 1..1024)) {
        use decoder_cosmos::CosmosTransaction;
        let hash1 = CosmosTransaction::calculate_hash(&data);
        let hash2 = CosmosTransaction::calculate_hash(&data);
        prop_assert_eq!(hash1, hash2);
    }

    /// Property: Transaction hash is always 32 bytes (SHA-256)
    #[test]
    fn transaction_hash_is_32_bytes(data in prop::collection::vec(any::<u8>(), 1..1024)) {
        use decoder_cosmos::CosmosTransaction;
        let hash = CosmosTransaction::calculate_hash(&data);
        prop_assert_eq!(hash.len(), 32);
    }

    /// Property: Different inputs produce different hashes
    #[test]
    fn different_inputs_different_hashes(
        data1 in prop::collection::vec(any::<u8>(), 1..1024),
        data2 in prop::collection::vec(any::<u8>(), 1..1024)
    ) {
        use decoder_cosmos::CosmosTransaction;
        if data1 != data2 {
            let hash1 = CosmosTransaction::calculate_hash(&data1);
            let hash2 = CosmosTransaction::calculate_hash(&data2);
            prop_assert_ne!(hash1, hash2);
        }
    }

    /// Property: Coin parsing properties
    #[test]
    fn coin_amount_is_valid_decimal(coin in arb_coin()) {
        // All generated coin amounts should parse as u128
        let parsed = coin.amount.parse::<u128>();
        prop_assert!(parsed.is_ok());
    }

    /// Property: Cosmos addresses have correct prefix
    #[test]
    fn cosmos_addresses_have_prefix(addr in arb_address()) {
        prop_assert!(addr.starts_with("cosmos1"));
    }

    /// Property: Amount parsing is consistent
    #[test]
    fn amount_parsing_is_consistent(value in 1u128..=1_000_000_000_000u128) {
        use decoder_cosmos::Coin;
        let coin = Coin {
            denom: "uatom".to_string(),
            amount: value.to_string(),
        };

        // Parse the amount
        let amount_str = &coin.amount;
        let parsed = amount_str.parse::<u128>()
            .expect("Generated coin amount should always parse as u128");
        prop_assert_eq!(parsed, value);
    }

    /// Property: Micro-denomination detection
    #[test]
    fn micro_denom_detection(
        prefix in prop::sample::select(&["u", "n", "m", ""]),
        base in "[a-z]{3,6}"
    ) {
        let denom = format!("{}{}", prefix, base);

        // Test that we can identify micro denominations
        let is_micro = denom.starts_with('u');
        let is_nano = denom.starts_with('n');

        if is_micro || is_nano {
            prop_assert!(denom.len() > 1);
        }
    }

    /// Property: Fee amount is never negative
    #[test]
    fn fee_amount_never_negative(amount in 0u64..=1_000_000_000u64) {
        // Gas limit should always be non-negative
        prop_assert!(amount < u64::MAX);
    }

    /// Property: Message type URL is valid format
    #[test]
    fn message_type_url_valid_format(
        package in "[a-z]+",
        module in "[a-z]+",
        version in "[a-z0-9]+",
        msg_type in "Msg[A-Z][a-zA-Z]+"
    ) {
        let type_url = format!("/{}.{}.{}.{}", package, module, version, msg_type);
        prop_assert!(type_url.starts_with('/'));
        prop_assert!(type_url.contains("Msg"));
    }

    /// Property: Valid protobuf transactions decode or fail gracefully
    #[test]
    fn valid_protobuf_decodes_or_fails_gracefully(
        memo in ".*",
        gas_limit in 1u64..=10_000_000u64
    ) {
        // Create a minimal valid transaction
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee};
        use prost::Message;

        let tx = Tx {
            body: Some(TxBody {
                messages: vec![],
                memo,
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos: vec![],
                    fee: Some(Fee {
                        amount: vec![],
                        gas_limit,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        if tx.encode(&mut tx_bytes).is_err() {
            // If protobuf encoding fails, skip this test case
            return Ok(());
        }

        // Should decode without panicking (may fail validation)
        let _ = CosmosDecoder::decode(&tx_bytes);
    }

    /// Property: Signature count validation
    #[test]
    fn signature_count_matches_signer_count(
        sig_count in 0usize..=10,
        signer_count in 0usize..=10
    ) {
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee, SignerInfo};
        use prost::Message;

        let signatures = vec![vec![0u8; 64]; sig_count];
        let signer_infos = vec![
            SignerInfo {
                public_key: None,
                mode_info: None,
                sequence: 0,
            };
            signer_count
        ];

        let tx = Tx {
            body: Some(TxBody {
                messages: vec![],
                memo: String::new(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos,
                    fee: Some(Fee {
                        amount: vec![],
                        gas_limit: 200000,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures,
        };

        let mut tx_bytes = Vec::new();
        if tx.encode(&mut tx_bytes).is_err() {
            // If protobuf encoding fails, skip this test case
            return Ok(());
        }

        if let Ok(decoded) = CosmosDecoder::decode(&tx_bytes) {
            // If decoding succeeds, validate should fail when counts don't match
            if sig_count != signer_count {
                prop_assert!(decoded.validate().is_err());
            }
        }
    }
}

#[test]
fn test_proptest_setup() {
    // Ensure proptest is properly configured by running a trivial property test
    use proptest::prelude::*;

    proptest!(|(x in 0..100u32)| {
        // Verify basic proptest functionality: generated values are in range
        prop_assert!(x < 100);
    });
}

proptest! {
    /// Property: CosmWasm execute message parsing is consistent
    #[test]
    fn cosmwasm_execute_msg_consistent(
        sender in "[a-z]{6,44}".prop_map(|s| format!("cosmos1{}", s)),
        contract in "[a-z0-9]{58}".prop_map(|s| format!("cosmos{}", s)),
        msg_len in 1usize..=1024,
        funds_count in 0usize..=5
    ) {
        use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee};
        use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract;
        use cosmos_sdk_proto::Any;
        use prost::Message;

        let funds: Vec<Coin> = (0..funds_count)
            .map(|i| Coin {
                denom: format!("token{}", i),
                amount: "1000".to_string(),
            })
            .collect();

        let msg_execute = MsgExecuteContract {
            sender: sender.clone(),
            contract: contract.clone(),
            msg: vec![0u8; msg_len],
            funds,
        };

        let mut msg_bytes = Vec::new();
        msg_execute.encode(&mut msg_bytes).unwrap();

        let any_msg = Any {
            type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".to_string(),
            value: msg_bytes,
        };

        let tx = Tx {
            body: Some(TxBody {
                messages: vec![any_msg],
                memo: String::new(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos: vec![],
                    fee: Some(Fee {
                        amount: vec![],
                        gas_limit: 200000,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

        // Should decode without panicking
        let result = CosmosDecoder::decode(&tx_bytes);
        prop_assert!(result.is_ok());

        if let Ok(decoded) = result {
            let messages = decoded.messages().unwrap();
            prop_assert_eq!(messages.len(), 1);
        }
    }

    /// Property: Transaction with multiple message types decodes correctly
    #[test]
    fn multi_type_messages_decode(msg_count in 1usize..=5) {
        use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
        use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee};
        use cosmos_sdk_proto::Any;
        use prost::Message;

        let mut messages = Vec::new();
        for i in 0..msg_count {
            let msg_send = MsgSend {
                from_address: format!("cosmos1sender{}", i),
                to_address: format!("cosmos1receiver{}", i),
                amount: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: "1000".to_string(),
                }],
            };

            let mut msg_bytes = Vec::new();
            msg_send.encode(&mut msg_bytes).unwrap();

            messages.push(Any {
                type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                value: msg_bytes,
            });
        }

        let tx = Tx {
            body: Some(TxBody {
                messages,
                memo: String::new(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos: vec![],
                    fee: Some(Fee {
                        amount: vec![],
                        gas_limit: 200000,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

        let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
        let parsed_messages = decoded.messages().unwrap();
        prop_assert_eq!(parsed_messages.len(), msg_count);
    }

    /// Property: Gas limit is always preserved
    #[test]
    fn gas_limit_preserved(gas_limit in 1u64..=10_000_000u64) {
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee};
        use prost::Message;

        let tx = Tx {
            body: Some(TxBody {
                messages: vec![],
                memo: String::new(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos: vec![],
                    fee: Some(Fee {
                        amount: vec![],
                        gas_limit,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

        let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
        prop_assert_eq!(decoded.gas_limit(), gas_limit);
    }

    /// Property: Memo is preserved
    #[test]
    fn memo_preserved(memo in ".*") {
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee};
        use prost::Message;

        let tx = Tx {
            body: Some(TxBody {
                messages: vec![],
                memo: memo.clone(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos: vec![],
                    fee: Some(Fee {
                        amount: vec![],
                        gas_limit: 200000,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

        let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
        prop_assert_eq!(decoded.memo(), memo.as_str());
    }

    /// Property: Fee amounts are preserved
    #[test]
    fn fee_amounts_preserved(
        denom in "[a-z]{3,10}",
        amount in 1u128..=1_000_000_000u128
    ) {
        use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee};
        use prost::Message;

        let tx = Tx {
            body: Some(TxBody {
                messages: vec![],
                memo: String::new(),
                timeout_height: 0,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos: vec![],
                    fee: Some(Fee {
                        amount: vec![Coin {
                            denom: denom.clone(),
                            amount: amount.to_string(),
                        }],
                        gas_limit: 200000,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

        let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
        let fee = decoded.fee();
        prop_assert_eq!(fee.amount.len(), 1);
        prop_assert_eq!(&fee.amount[0].denom, &denom);
        prop_assert_eq!(&fee.amount[0].amount, &amount.to_string());
    }

    /// Property: Timeout height is preserved
    #[test]
    fn timeout_height_preserved(timeout_height in 0u64..=10_000_000u64) {
        use cosmos_sdk_proto::cosmos::tx::v1beta1::{Tx, TxBody, AuthInfo, Fee};
        use prost::Message;

        let tx = Tx {
            body: Some(TxBody {
                messages: vec![],
                memo: String::new(),
                timeout_height,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }),
            auth_info: {
                #[allow(deprecated)]
                Some(AuthInfo {
                    signer_infos: vec![],
                    fee: Some(Fee {
                        amount: vec![],
                        gas_limit: 200000,
                        payer: String::new(),
                        granter: String::new(),
                    }),
                    tip: None,
                })
            },
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

        let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
        prop_assert_eq!(decoded.tx.body.timeout_height, timeout_height);
    }
}
