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
    #[allow(deprecated)]
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
    use cosmos_sdk_proto::Any;
    use prost::Message;

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
        auth_info: {
            #[allow(deprecated)]
            Some(AuthInfo {
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
            })
        },
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
fn test_decode_ibc_transfer_transaction() {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as CosmosCoin;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use cosmos_sdk_proto::Any;
    use ibc_proto::cosmos::base::v1beta1::Coin as IbcCoin;
    use ibc_proto::ibc::applications::transfer::v1::MsgTransfer;
    use ibc_proto::ibc::core::client::v1::Height;
    use prost::Message;

    // Create MsgTransfer (IBC)
    let msg_transfer = MsgTransfer {
        source_port: "transfer".to_string(),
        source_channel: "channel-0".to_string(),
        token: Some(IbcCoin {
            denom: "uatom".to_string(),
            amount: "10000000".to_string(),
        }),
        sender: "cosmos1sender".to_string(),
        receiver: "osmo1receiver".to_string(),
        timeout_height: Some(Height {
            revision_number: 1,
            revision_height: 1000000,
        }),
        timeout_timestamp: 0,
        memo: "IBC transfer".to_string(),
    };

    // Encode as Any
    let mut msg_bytes = Vec::new();
    msg_transfer.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: msg_bytes,
    };

    // Create transaction
    let tx = Tx {
        body: Some(TxBody {
            messages: vec![any_msg],
            memo: "IBC test".to_string(),
            timeout_height: 0,
            extension_options: vec![],
            non_critical_extension_options: vec![],
        }),
        auth_info: {
            #[allow(deprecated)]
            Some(AuthInfo {
                signer_infos: vec![],
                fee: Some(Fee {
                    amount: vec![CosmosCoin {
                        denom: "uatom".to_string(),
                        amount: "3000".to_string(),
                    }],
                    gas_limit: 250000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            })
        },
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
        CosmosMessage::IbcTransfer(ibc) => {
            assert_eq!(ibc.source_port, "transfer");
            assert_eq!(ibc.source_channel, "channel-0");
            assert_eq!(ibc.sender, "cosmos1sender");
            assert_eq!(ibc.receiver, "osmo1receiver");
            assert_eq!(ibc.token.denom, "uatom");
            assert_eq!(ibc.token.amount, "10000000");
        }
        _ => panic!("Expected MsgIbcTransfer"),
    }

    // Canonicalize
    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.operations.len(), 1);

    // Verify IBC transfer creates state delta with outputs (tokens leaving chain)
    assert!(!tx_ir.state_deltas.outputs.is_empty());
    assert_eq!(tx_ir.state_deltas.account_changes.len(), 1); // Sender account
}

#[test]
fn test_decode_multi_message_transaction() {
    use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate;
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
        auth_info: {
            #[allow(deprecated)]
            Some(AuthInfo {
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
            })
        },
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
    use cosmos_sdk_proto::Any;
    use prost::Message;

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
        auth_info: {
            #[allow(deprecated)]
            Some(AuthInfo {
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
            })
        },
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
