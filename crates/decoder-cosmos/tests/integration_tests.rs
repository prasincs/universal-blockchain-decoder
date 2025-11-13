//! Integration tests for Cosmos decoder
//!
//! These tests validate the decoder against real Cosmos SDK transactions
//! using the official cosmos-sdk-proto library for reference.

#![allow(deprecated)]

use decoder_cosmos::{CosmosDecoder, CosmosMessage};
use decoder_primitives::prelude::*;

#[test]
fn test_decode_msg_send_transaction() {
    // This is a simplified test - in practice, you would load real transaction bytes
    // from mainnet or testnet

    // Create a minimal valid Protobuf transaction using cosmos-sdk-proto
    use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use cosmos_sdk_proto::Any;
    use prost::Message;

    // Create MsgSend
    let msg_send = MsgSend {
        from_address: "cosmos1sender".to_string(),
        to_address: "cosmos1receiver".to_string(),
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "1000000".to_string(),
        }],
    };

    // Encode as Any
    let mut msg_bytes = Vec::new();
    msg_send.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
        value: msg_bytes,
    };

    // Create transaction body
    let tx_body = TxBody {
        messages: vec![any_msg],
        memo: "test transaction".to_string(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    // Create fee
    let fee = Fee {
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "5000".to_string(),
        }],
        gas_limit: 200000,
        payer: String::new(),
        granter: String::new(),
    };

    // Create auth info
    let auth_info = AuthInfo {
        signer_infos: vec![],
        fee: Some(fee),
        tip: None,
    };

    // Create transaction
    let tx = Tx {
        body: Some(tx_body),
        auth_info: Some(auth_info),
        signatures: vec![vec![0u8; 64]], // Dummy signature
    };

    // Encode transaction
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    // Decode using our decoder
    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();

    // Validate
    assert_eq!(decoded.tx_hash.len(), 32);
    assert_eq!(decoded.memo(), "test transaction");
    assert_eq!(decoded.gas_limit(), 200000);
    assert_eq!(decoded.signatures().len(), 1);

    // Parse messages
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::Send(send) => {
            assert_eq!(send.from_address, "cosmos1sender");
            assert_eq!(send.to_address, "cosmos1receiver");
            assert_eq!(send.amount.len(), 1);
            assert_eq!(send.amount[0].denom, "uatom");
            assert_eq!(send.amount[0].amount, "1000000");
        }
        _ => panic!("Expected MsgSend"),
    }

    // Canonicalize to TxIR
    let tx_ir = decoded.canonicalize().unwrap();

    // Validate TxIR
    assert!(!tx_ir.metadata.tx_hash.is_empty());
    assert_eq!(tx_ir.metadata.size, tx_bytes.len());
    assert_eq!(tx_ir.operations.len(), 1);
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
}

#[test]
fn test_decode_msg_delegate_transaction() {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use prost::Message;
    use cosmos_sdk_proto::Any;

    // Create MsgDelegate
    let msg_delegate = MsgDelegate {
        delegator_address: "cosmos1delegator".to_string(),
        validator_address: "cosmosvaloper1validator".to_string(),
        amount: Some(Coin {
            denom: "uatom".to_string(),
            amount: "5000000".to_string(),
        }),
    };

    // Encode as Any
    let mut msg_bytes = Vec::new();
    msg_delegate.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        value: msg_bytes,
    };

    // Create transaction
    let tx = Tx {
        body: Some(TxBody {
            messages: vec![any_msg],
            memo: "delegate".to_string(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
        }),
        auth_info: Some(AuthInfo {
            signer_infos: vec![],
            fee: Some(Fee {
                amount: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: "2000".to_string(),
                }],
                gas_limit: 150000,
                payer: String::new(),
                granter: String::new(),
            }),
            tip: None,
        }),
        signatures: vec![vec![0u8; 64]],
    };

    // Encode transaction
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    // Decode
    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();

    // Parse messages
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::Delegate(delegate) => {
            assert_eq!(delegate.delegator_address, "cosmos1delegator");
            assert_eq!(delegate.validator_address, "cosmosvaloper1validator");
            assert_eq!(delegate.amount.denom, "uatom");
            assert_eq!(delegate.amount.amount, "5000000");
        }
        _ => panic!("Expected MsgDelegate"),
    }

    // Canonicalize
    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.operations.len(), 1);
}

#[test]
#[ignore = "IBC modules not available in cosmos-sdk-proto 0.25.0"]
fn test_decode_ibc_transfer_transaction() {
    // This test is disabled because IBC modules are not available in cosmos-sdk-proto 0.25.0
    // To enable this test:
    // 1. Upgrade cosmos-sdk-proto to a version that includes IBC modules (e.g., 0.27.0+)
    // 2. Uncomment the test implementation below
    // 3. Enable IBC features in Cargo.toml if needed
    //
    // The test would validate IBC transfer message decoding and canonicalization
}

#[test]
fn test_decode_multi_message_transaction() {
    use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use prost::Message;
    use cosmos_sdk_proto::Any;

    // Create MsgSend
    let msg_send = MsgSend {
        from_address: "cosmos1sender".to_string(),
        to_address: "cosmos1receiver".to_string(),
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "1000000".to_string(),
        }],
    };

    let mut send_bytes = Vec::new();
    msg_send.encode(&mut send_bytes).unwrap();

    let any_send = Any {
        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
        value: send_bytes,
    };

    // Create MsgDelegate
    let msg_delegate = MsgDelegate {
        delegator_address: "cosmos1delegator".to_string(),
        validator_address: "cosmosvaloper1validator".to_string(),
        amount: Some(Coin {
            denom: "uatom".to_string(),
            amount: "2000000".to_string(),
        }),
    };

    let mut delegate_bytes = Vec::new();
    msg_delegate.encode(&mut delegate_bytes).unwrap();

    let any_delegate = Any {
        type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        value: delegate_bytes,
    };

    // Create transaction with multiple messages
    let tx = Tx {
        body: Some(TxBody {
            messages: vec![any_send, any_delegate],
            memo: "multi-message".to_string(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
        }),
        auth_info: Some(AuthInfo {
            signer_infos: vec![],
            fee: Some(Fee {
                amount: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: "5000".to_string(),
                }],
                gas_limit: 300000,
                payer: String::new(),
                granter: String::new(),
            }),
            tip: None,
        }),
        signatures: vec![vec![0u8; 64]],
    };

    // Encode transaction
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    // Decode
    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();

    // Parse messages
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 2);

    // Canonicalize
    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.operations.len(), 2);
}

#[test]
fn test_validate_signature_count() {
    use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, SignerInfo, Tx, TxBody};
    use prost::Message;
    use cosmos_sdk_proto::Any;

    let msg_send = MsgSend {
        from_address: "cosmos1sender".to_string(),
        to_address: "cosmos1receiver".to_string(),
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "1000000".to_string(),
        }],
    };

    let mut msg_bytes = Vec::new();
    msg_send.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
        value: msg_bytes,
    };

    // Create transaction with mismatched signature count
    let tx = Tx {
        body: Some(TxBody {
            messages: vec![any_msg],
            memo: String::new(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
        }),
        auth_info: Some(AuthInfo {
            signer_infos: vec![SignerInfo {
                public_key: None,
                mode_info: None,
                sequence: 0,
            }],
            fee: Some(Fee {
                amount: vec![],
                gas_limit: 200000,
                payer: String::new(),
                granter: String::new(),
            }),
            tip: None,
        }),
        signatures: vec![], // No signatures, but 1 signer!
    };

    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();

    // Validation should fail
    let result = decoded.validate();
    assert!(result.is_err());
}

#[test]
fn test_decode_empty_transaction() {
    let result = CosmosDecoder::decode(&[]);
    assert!(result.is_err());
}

#[test]
fn test_decode_invalid_protobuf() {
    let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
    let result = CosmosDecoder::decode(&invalid_data);
    assert!(result.is_err());
}
