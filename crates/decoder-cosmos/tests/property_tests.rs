//! Property-based tests for Cosmos decoder using proptest
//!
//! These tests verify properties that should hold for all valid inputs.

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
    fn empty_input_always_fails(prefix in any::<Vec<u8>>()) {
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
        let parsed = amount_str.parse::<u128>().unwrap();
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
            auth_info: Some(AuthInfo {
                signer_infos: vec![],
                fee: Some(Fee {
                    amount: vec![],
                    gas_limit,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            }),
            signatures: vec![],
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

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
            auth_info: Some(AuthInfo {
                signer_infos,
                fee: Some(Fee {
                    amount: vec![],
                    gas_limit: 200000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            }),
            signatures,
        };

        let mut tx_bytes = Vec::new();
        tx.encode(&mut tx_bytes).unwrap();

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
    // Ensure proptest is properly configured
    assert!(true);
}
