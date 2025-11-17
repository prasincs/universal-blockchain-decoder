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

// === Tests for New IBC Message Types ===

#[test]
fn test_decode_ibc_recv_packet() {
    use cosmos_sdk_proto::Any;
    use ibc_proto::ibc::core::channel::v1::{MsgRecvPacket, Packet};
    use ibc_proto::ibc::core::client::v1::Height;
    use prost::Message;

    let packet = Packet {
        sequence: 1,
        source_port: "transfer".to_string(),
        source_channel: "channel-0".to_string(),
        destination_port: "transfer".to_string(),
        destination_channel: "channel-1".to_string(),
        data: vec![1, 2, 3, 4],
        timeout_height: Some(Height {
            revision_number: 1,
            revision_height: 1000000,
        }),
        timeout_timestamp: 0,
    };

    let msg = MsgRecvPacket {
        packet: Some(packet),
        proof_commitment: vec![5, 6, 7, 8],
        proof_height: Some(Height {
            revision_number: 1,
            revision_height: 999999,
        }),
        signer: "cosmos1relayer".to_string(),
    };

    let mut msg_bytes = Vec::new();
    msg.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/ibc.core.channel.v1.MsgRecvPacket".to_string(),
        value: msg_bytes,
    };

    let tx = create_test_tx(vec![any_msg]);
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::IbcRecvPacket(recv) => {
            assert_eq!(recv.packet.sequence, 1);
            assert_eq!(recv.packet.source_channel, "channel-0");
            assert_eq!(recv.signer, "cosmos1relayer");
        }
        _ => panic!("Expected IbcRecvPacket"),
    }
}

#[test]
fn test_decode_ibc_acknowledgement() {
    use cosmos_sdk_proto::Any;
    use ibc_proto::ibc::core::channel::v1::{MsgAcknowledgement, Packet};
    use ibc_proto::ibc::core::client::v1::Height;
    use prost::Message;

    let packet = Packet {
        sequence: 2,
        source_port: "transfer".to_string(),
        source_channel: "channel-0".to_string(),
        destination_port: "transfer".to_string(),
        destination_channel: "channel-1".to_string(),
        data: vec![1, 2, 3, 4],
        timeout_height: None,
        timeout_timestamp: 1234567890,
    };

    let msg = MsgAcknowledgement {
        packet: Some(packet),
        acknowledgement: vec![0x01],
        proof_acked: vec![9, 10, 11],
        proof_height: Some(Height {
            revision_number: 1,
            revision_height: 1000001,
        }),
        signer: "cosmos1relayer2".to_string(),
    };

    let mut msg_bytes = Vec::new();
    msg.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/ibc.core.channel.v1.MsgAcknowledgement".to_string(),
        value: msg_bytes,
    };

    let tx = create_test_tx(vec![any_msg]);
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::IbcAcknowledgement(ack) => {
            assert_eq!(ack.packet.sequence, 2);
            assert_eq!(ack.acknowledgement, vec![0x01]);
            assert_eq!(ack.signer, "cosmos1relayer2");
        }
        _ => panic!("Expected IbcAcknowledgement"),
    }
}

#[test]
fn test_decode_ibc_update_client() {
    use cosmos_sdk_proto::Any;
    use ibc_proto::ibc::core::client::v1::MsgUpdateClient;
    use prost::Message;

    let msg = MsgUpdateClient {
        client_id: "07-tendermint-0".to_string(),
        client_message: Some(ibc_proto::google::protobuf::Any {
            type_url: "/ibc.lightclients.tendermint.v1.Header".to_string(),
            value: vec![1, 2, 3, 4, 5],
        }),
        signer: "cosmos1relayer3".to_string(),
    };

    let mut msg_bytes = Vec::new();
    msg.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/ibc.core.client.v1.MsgUpdateClient".to_string(),
        value: msg_bytes,
    };

    let tx = create_test_tx(vec![any_msg]);
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::IbcUpdateClient(update) => {
            assert_eq!(update.client_id, "07-tendermint-0");
            assert_eq!(update.signer, "cosmos1relayer3");
            assert_eq!(update.client_message.len(), 5);
        }
        _ => panic!("Expected IbcUpdateClient"),
    }
}

// === Tests for New CosmWasm Message Types ===

#[test]
fn test_decode_store_code() {
    use cosmos_sdk_proto::cosmwasm::wasm::v1::{AccessConfig, AccessType, MsgStoreCode};
    use cosmos_sdk_proto::Any;
    use prost::Message;

    let wasm_bytecode = vec![0x00, 0x61, 0x73, 0x6D]; // WASM magic number

    let msg = MsgStoreCode {
        sender: "cosmos1deployer".to_string(),
        wasm_byte_code: wasm_bytecode.clone(),
        instantiate_permission: Some(AccessConfig {
            permission: AccessType::Everybody as i32,
            addresses: vec![],
        }),
    };

    let mut msg_bytes = Vec::new();
    msg.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmwasm.wasm.v1.MsgStoreCode".to_string(),
        value: msg_bytes,
    };

    let tx = create_test_tx(vec![any_msg]);
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::StoreCode(store) => {
            assert_eq!(store.sender, "cosmos1deployer");
            assert_eq!(store.wasm_byte_code, wasm_bytecode);
            assert!(store.instantiate_permission.is_some());
        }
        _ => panic!("Expected StoreCode"),
    }

    // Test canonicalization
    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.operations.len(), 1);
}

#[test]
fn test_decode_instantiate_contract() {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgInstantiateContract;
    use cosmos_sdk_proto::Any;
    use prost::Message;

    let init_msg = br#"{"count": 0}"#.to_vec();

    let msg = MsgInstantiateContract {
        sender: "cosmos1creator".to_string(),
        admin: "cosmos1admin".to_string(),
        code_id: 42,
        label: "my-counter".to_string(),
        msg: init_msg.clone(),
        funds: vec![Coin {
            denom: "uatom".to_string(),
            amount: "1000".to_string(),
        }],
    };

    let mut msg_bytes = Vec::new();
    msg.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmwasm.wasm.v1.MsgInstantiateContract".to_string(),
        value: msg_bytes,
    };

    let tx = create_test_tx(vec![any_msg]);
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::InstantiateContract(instantiate) => {
            assert_eq!(instantiate.sender, "cosmos1creator");
            assert_eq!(instantiate.admin, "cosmos1admin");
            assert_eq!(instantiate.code_id, 42);
            assert_eq!(instantiate.label, "my-counter");
            assert_eq!(instantiate.msg, init_msg);
            assert_eq!(instantiate.funds.len(), 1);
        }
        _ => panic!("Expected InstantiateContract"),
    }
}

#[test]
fn test_decode_execute_contract() {
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract;
    use cosmos_sdk_proto::Any;
    use prost::Message;

    let exec_msg = br#"{"increment": {}}"#.to_vec();

    let msg = MsgExecuteContract {
        sender: "cosmos1user".to_string(),
        contract: "cosmos1contractaddr".to_string(),
        msg: exec_msg.clone(),
        funds: vec![],
    };

    let mut msg_bytes = Vec::new();
    msg.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".to_string(),
        value: msg_bytes,
    };

    let tx = create_test_tx(vec![any_msg]);
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::ExecuteContract(execute) => {
            assert_eq!(execute.sender, "cosmos1user");
            assert_eq!(execute.contract, "cosmos1contractaddr");
            assert_eq!(execute.msg, exec_msg);
            assert_eq!(execute.funds.len(), 0);
        }
        _ => panic!("Expected ExecuteContract"),
    }
}

#[test]
fn test_decode_migrate_contract() {
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgMigrateContract;
    use cosmos_sdk_proto::Any;
    use prost::Message;

    let migrate_msg = br#"{"migrate": "v2"}"#.to_vec();

    let msg = MsgMigrateContract {
        sender: "cosmos1admin".to_string(),
        contract: "cosmos1contractaddr".to_string(),
        code_id: 43,
        msg: migrate_msg.clone(),
    };

    let mut msg_bytes = Vec::new();
    msg.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmwasm.wasm.v1.MsgMigrateContract".to_string(),
        value: msg_bytes,
    };

    let tx = create_test_tx(vec![any_msg]);
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::MigrateContract(migrate) => {
            assert_eq!(migrate.sender, "cosmos1admin");
            assert_eq!(migrate.contract, "cosmos1contractaddr");
            assert_eq!(migrate.code_id, 43);
            assert_eq!(migrate.msg, migrate_msg);
        }
        _ => panic!("Expected MigrateContract"),
    }
}

// === Helper Functions ===

/// Helper to create a minimal test transaction
fn create_test_tx(
    messages: Vec<cosmos_sdk_proto::Any>,
) -> cosmos_sdk_proto::cosmos::tx::v1beta1::Tx {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};

    Tx {
        body: Some(TxBody {
            messages,
            memo: "test".to_string(),
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
                    gas_limit: 200000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            })
        },
        signatures: vec![vec![0u8; 64]],
    }
}

fn test_decode_cosmwasm_execute_no_funds() {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract;
    use cosmos_sdk_proto::Any;
    use prost::Message;

    // Create execute message without funds
    let execute_msg = r#"{"query_balance":{"address":"cosmos1xyz"}}"#;

    let msg_execute = MsgExecuteContract {
        sender: "cosmos1querier".to_string(),
        contract: "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr".to_string(),
        msg: execute_msg.as_bytes().to_vec(),
        funds: vec![], // No funds sent
    };

    // Encode as Any
    let mut msg_bytes = Vec::new();
    msg_execute.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".to_string(),
        value: msg_bytes,
    };

    // Create transaction
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
                    amount: vec![Coin {
                        denom: "uatom".to_string(),
                        amount: "500".to_string(),
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

    // Encode and decode
    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();
    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();

    // Validate
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::ExecuteContract(exec) => {
            assert_eq!(exec.sender, "cosmos1querier");
            assert_eq!(exec.funds.len(), 0); // No funds
        }
        _ => panic!("Expected MsgExecuteContract"),
    }
}
fn test_decode_withdraw_delegator_reward() {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::distribution::v1beta1::MsgWithdrawDelegatorReward;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use cosmos_sdk_proto::Any;
    use prost::Message;

    let msg_withdraw = MsgWithdrawDelegatorReward {
        delegator_address: "cosmos1delegator".to_string(),
        validator_address: "cosmosvaloper1validator".to_string(),
    };

    let mut msg_bytes = Vec::new();
    msg_withdraw.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
        value: msg_bytes,
    };

    let tx = Tx {
        body: Some(TxBody {
            messages: vec![any_msg],
            memo: "withdraw rewards".to_string(),
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
                        amount: "500".to_string(),
                    }],
                    gas_limit: 100000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            })
        },
        signatures: vec![vec![0u8; 64]],
    };

    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::WithdrawDelegatorReward(withdraw) => {
            assert_eq!(withdraw.delegator_address, "cosmos1delegator");
            assert_eq!(withdraw.validator_address, "cosmosvaloper1validator");
        }
        _ => panic!("Expected MsgWithdrawDelegatorReward"),
    }

    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.operations.len(), 1);
}
fn test_decode_submit_proposal() {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::gov::v1beta1::MsgSubmitProposal;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use cosmos_sdk_proto::Any;
    use prost::Message;

    // Create a text proposal content
    let proposal_content = Any {
        type_url: "/cosmos.gov.v1beta1.TextProposal".to_string(),
        value: vec![10, 11, 84, 101, 115, 116, 32, 84, 105, 116, 108, 101], // "Test Title" in proto
    };

    let msg_submit = MsgSubmitProposal {
        content: Some(proposal_content.clone()),
        initial_deposit: vec![Coin {
            denom: "uatom".to_string(),
            amount: "10000000".to_string(),
        }],
        proposer: "cosmos1proposer".to_string(),
    };

    let mut msg_bytes = Vec::new();
    msg_submit.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmos.gov.v1beta1.MsgSubmitProposal".to_string(),
        value: msg_bytes,
    };

    let tx = Tx {
        body: Some(TxBody {
            messages: vec![any_msg],
            memo: "submit proposal".to_string(),
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
                    gas_limit: 300000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            })
        },
        signatures: vec![vec![0u8; 64]],
    };

    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::SubmitProposal(submit) => {
            assert_eq!(submit.proposer, "cosmos1proposer");
            assert_eq!(submit.initial_deposit.len(), 1);
            assert_eq!(submit.initial_deposit[0].denom, "uatom");
            assert_eq!(submit.initial_deposit[0].amount, "10000000");
            assert_eq!(submit.content_type_url, proposal_content.type_url);
        }
        _ => panic!("Expected MsgSubmitProposal"),
    }

    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.operations.len(), 1);
}
fn test_decode_deposit() {
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use cosmos_sdk_proto::cosmos::gov::v1beta1::MsgDeposit;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, Fee, Tx, TxBody};
    use cosmos_sdk_proto::Any;
    use prost::Message;

    let msg_deposit = MsgDeposit {
        proposal_id: 42,
        depositor: "cosmos1depositor".to_string(),
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "5000000".to_string(),
        }],
    };

    let mut msg_bytes = Vec::new();
    msg_deposit.encode(&mut msg_bytes).unwrap();

    let any_msg = Any {
        type_url: "/cosmos.gov.v1beta1.MsgDeposit".to_string(),
        value: msg_bytes,
    };

    let tx = Tx {
        body: Some(TxBody {
            messages: vec![any_msg],
            memo: "deposit to proposal".to_string(),
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
                        amount: "1000".to_string(),
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

    let mut tx_bytes = Vec::new();
    tx.encode(&mut tx_bytes).unwrap();

    let decoded = CosmosDecoder::decode(&tx_bytes).unwrap();
    let messages = decoded.messages().unwrap();
    assert_eq!(messages.len(), 1);

    match &messages[0] {
        CosmosMessage::Deposit(deposit) => {
            assert_eq!(deposit.proposal_id, 42);
            assert_eq!(deposit.depositor, "cosmos1depositor");
            assert_eq!(deposit.amount.len(), 1);
            assert_eq!(deposit.amount[0].denom, "uatom");
            assert_eq!(deposit.amount[0].amount, "5000000");
        }
        _ => panic!("Expected MsgDeposit"),
    }

    let tx_ir = decoded.canonicalize().unwrap();
    assert_eq!(tx_ir.operations.len(), 1);
}
