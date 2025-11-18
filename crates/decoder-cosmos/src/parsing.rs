//! Cosmos transaction parsing logic
//!
//! This module handles decoding of Protobuf-encoded Cosmos SDK transactions
//! and converting them to our internal types.

use crate::types::*;
use decoder_primitives::prelude::*;
use prost::Message;
use prost_types::Any;

/// Parse a Cosmos transaction from raw bytes
///
/// Cosmos transactions are Protobuf-encoded following the cosmos.tx.v1beta1.Tx schema.
/// The transaction contains:
/// - body: Transaction messages and metadata
/// - auth_info: Fee, gas, and signer information
/// - signatures: Cryptographic signatures
pub fn parse_tx(data: &[u8]) -> Result<Tx> {
    // Decode Protobuf message
    let raw_tx = cosmos_sdk_proto::cosmos::tx::v1beta1::Tx::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to decode Cosmos tx: {}", e))
    })?;

    // Parse body
    let body = raw_tx
        .body
        .ok_or_else(|| DecoderError::invalid_structure("Missing transaction body"))?;

    let parsed_body = parse_tx_body(body)?;

    // Parse auth info
    let auth_info = raw_tx
        .auth_info
        .ok_or_else(|| DecoderError::invalid_structure("Missing auth info"))?;

    let parsed_auth_info = parse_auth_info(auth_info)?;

    Ok(Tx {
        body: parsed_body,
        auth_info: parsed_auth_info,
        signatures: raw_tx.signatures,
    })
}

/// Parse transaction body
fn parse_tx_body(body: cosmos_sdk_proto::cosmos::tx::v1beta1::TxBody) -> Result<TxBody> {
    // Convert Any messages to prost_types::Any
    let messages: Vec<Any> = body
        .messages
        .into_iter()
        .map(|msg| Any {
            type_url: msg.type_url,
            value: msg.value,
        })
        .collect();

    let extension_options: Vec<Any> = body
        .extension_options
        .into_iter()
        .map(|opt| Any {
            type_url: opt.type_url,
            value: opt.value,
        })
        .collect();

    let non_critical_extension_options: Vec<Any> = body
        .non_critical_extension_options
        .into_iter()
        .map(|opt| Any {
            type_url: opt.type_url,
            value: opt.value,
        })
        .collect();

    Ok(TxBody {
        messages,
        memo: body.memo,
        timeout_height: body.timeout_height,
        extension_options,
        non_critical_extension_options,
    })
}

/// Parse authentication information
fn parse_auth_info(auth_info: cosmos_sdk_proto::cosmos::tx::v1beta1::AuthInfo) -> Result<AuthInfo> {
    let signer_infos: Result<Vec<SignerInfo>> = auth_info
        .signer_infos
        .into_iter()
        .map(parse_signer_info)
        .collect();

    let fee = auth_info
        .fee
        .ok_or_else(|| DecoderError::invalid_structure("Missing fee"))?;

    let parsed_fee = parse_fee(fee)?;

    #[allow(deprecated)]
    let tip = auth_info.tip.map(|tip| Tip {
        amount: tip.amount.into_iter().map(parse_coin).collect(),
        tipper: tip.tipper,
    });

    Ok(AuthInfo {
        signer_infos: signer_infos?,
        fee: parsed_fee,
        tip,
    })
}

/// Parse signer information
fn parse_signer_info(
    info: cosmos_sdk_proto::cosmos::tx::v1beta1::SignerInfo,
) -> Result<SignerInfo> {
    let public_key = info.public_key.map(|pk| Any {
        type_url: pk.type_url,
        value: pk.value,
    });

    let mode_info = info.mode_info.map(parse_mode_info).transpose()?;

    Ok(SignerInfo {
        public_key,
        mode_info,
        sequence: info.sequence,
    })
}

/// Parse mode info (single or multi-sig)
fn parse_mode_info(info: cosmos_sdk_proto::cosmos::tx::v1beta1::ModeInfo) -> Result<ModeInfo> {
    use cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Sum;

    match info.sum {
        Some(Sum::Single(single)) => Ok(ModeInfo::Single(ModeInfoSingle { mode: single.mode })),
        Some(Sum::Multi(multi)) => {
            let bitarray = multi.bitarray.map(|ba| CompactBitArray {
                extra_bits_stored: ba.extra_bits_stored,
                elems: ba.elems,
            });

            let mode_infos: Result<Vec<ModeInfo>> =
                multi.mode_infos.into_iter().map(parse_mode_info).collect();

            Ok(ModeInfo::Multi(ModeInfoMulti {
                bitarray,
                mode_infos: mode_infos?,
            }))
        }
        None => Err(DecoderError::invalid_structure("Missing mode info")),
    }
}

/// Parse transaction fee
fn parse_fee(fee: cosmos_sdk_proto::cosmos::tx::v1beta1::Fee) -> Result<Fee> {
    let amount: Vec<Coin> = fee.amount.into_iter().map(parse_coin).collect();

    Ok(Fee {
        amount,
        gas_limit: fee.gas_limit,
        payer: fee.payer,
        granter: fee.granter,
    })
}

/// Parse coin from Protobuf
fn parse_coin(coin: cosmos_sdk_proto::cosmos::base::v1beta1::Coin) -> Coin {
    Coin {
        denom: coin.denom,
        amount: coin.amount,
    }
}

/// Parse a specific message type from Any
pub fn parse_message(msg: &Any) -> Result<CosmosMessage> {
    match msg.type_url.as_str() {
        // Bank messages
        type_urls::MSG_SEND => parse_msg_send(&msg.value),
        type_urls::MSG_MULTI_SEND => parse_msg_multi_send(&msg.value),

        // Staking messages
        type_urls::MSG_DELEGATE => parse_msg_delegate(&msg.value),
        type_urls::MSG_UNDELEGATE => parse_msg_undelegate(&msg.value),
        type_urls::MSG_BEGIN_REDELEGATE => parse_msg_begin_redelegate(&msg.value),

        // IBC messages
        type_urls::MSG_IBC_TRANSFER => parse_msg_ibc_transfer(&msg.value),
        type_urls::MSG_IBC_RECV_PACKET => parse_msg_ibc_recv_packet(&msg.value),
        type_urls::MSG_IBC_ACKNOWLEDGEMENT => parse_msg_ibc_acknowledgement(&msg.value),
        type_urls::MSG_IBC_TIMEOUT => parse_msg_ibc_timeout(&msg.value),
        type_urls::MSG_IBC_CREATE_CLIENT => parse_msg_ibc_create_client(&msg.value),
        type_urls::MSG_IBC_UPDATE_CLIENT => parse_msg_ibc_update_client(&msg.value),

        // Governance messages
        type_urls::MSG_VOTE => parse_msg_vote(&msg.value),

        // CosmWasm messages
        type_urls::MSG_STORE_CODE => parse_msg_store_code(&msg.value),
        type_urls::MSG_INSTANTIATE_CONTRACT => parse_msg_instantiate_contract(&msg.value),
        type_urls::MSG_EXECUTE_CONTRACT => parse_msg_execute_contract(&msg.value),
        type_urls::MSG_MIGRATE_CONTRACT => parse_msg_migrate_contract(&msg.value),

        _ => Ok(CosmosMessage::Unknown {
            type_url: msg.type_url.clone(),
            value: msg.value.clone(),
        }),
    }
}

/// Parse MsgSend
fn parse_msg_send(data: &[u8]) -> Result<CosmosMessage> {
    let msg = cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend::decode(data)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to parse MsgSend: {}", e)))?;

    Ok(CosmosMessage::Send(MsgSend {
        from_address: msg.from_address,
        to_address: msg.to_address,
        amount: msg.amount.into_iter().map(parse_coin).collect(),
    }))
}

/// Parse MsgMultiSend
fn parse_msg_multi_send(data: &[u8]) -> Result<CosmosMessage> {
    let msg = cosmos_sdk_proto::cosmos::bank::v1beta1::MsgMultiSend::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgMultiSend: {}", e))
    })?;

    let inputs = msg
        .inputs
        .into_iter()
        .map(|input| MultiSendInput {
            address: input.address,
            coins: input.coins.into_iter().map(parse_coin).collect(),
        })
        .collect();

    let outputs = msg
        .outputs
        .into_iter()
        .map(|output| MultiSendOutput {
            address: output.address,
            coins: output.coins.into_iter().map(parse_coin).collect(),
        })
        .collect();

    Ok(CosmosMessage::MultiSend(MsgMultiSend { inputs, outputs }))
}

/// Parse MsgDelegate
fn parse_msg_delegate(data: &[u8]) -> Result<CosmosMessage> {
    let msg =
        cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate::decode(data).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse MsgDelegate: {}", e))
        })?;

    let amount = msg
        .amount
        .ok_or_else(|| DecoderError::invalid_structure("Missing delegation amount"))?;

    Ok(CosmosMessage::Delegate(MsgDelegate {
        delegator_address: msg.delegator_address,
        validator_address: msg.validator_address,
        amount: parse_coin(amount),
    }))
}

/// Parse MsgUndelegate
fn parse_msg_undelegate(data: &[u8]) -> Result<CosmosMessage> {
    let msg =
        cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate::decode(data).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse MsgUndelegate: {}", e))
        })?;

    let amount = msg
        .amount
        .ok_or_else(|| DecoderError::invalid_structure("Missing undelegation amount"))?;

    Ok(CosmosMessage::Undelegate(MsgUndelegate {
        delegator_address: msg.delegator_address,
        validator_address: msg.validator_address,
        amount: parse_coin(amount),
    }))
}

/// Parse MsgBeginRedelegate
fn parse_msg_begin_redelegate(data: &[u8]) -> Result<CosmosMessage> {
    let msg = cosmos_sdk_proto::cosmos::staking::v1beta1::MsgBeginRedelegate::decode(data)
        .map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse MsgBeginRedelegate: {}", e))
        })?;

    let amount = msg
        .amount
        .ok_or_else(|| DecoderError::invalid_structure("Missing redelegation amount"))?;

    Ok(CosmosMessage::BeginRedelegate(MsgBeginRedelegate {
        delegator_address: msg.delegator_address,
        validator_src_address: msg.validator_src_address,
        validator_dst_address: msg.validator_dst_address,
        amount: parse_coin(amount),
    }))
}

/// Parse IBC transfer message using ibc-proto
fn parse_msg_ibc_transfer(data: &[u8]) -> Result<CosmosMessage> {
    let msg =
        ibc_proto::ibc::applications::transfer::v1::MsgTransfer::decode(data).map_err(|e| {
            DecoderError::invalid_structure(format!("Failed to parse MsgTransfer: {}", e))
        })?;

    let token = msg
        .token
        .ok_or_else(|| DecoderError::invalid_structure("Missing token in IBC transfer"))?;

    let timeout_height = msg.timeout_height.map(|h| IbcHeight {
        revision_number: h.revision_number,
        revision_height: h.revision_height,
    });

    Ok(CosmosMessage::IbcTransfer(MsgIbcTransfer {
        source_port: msg.source_port,
        source_channel: msg.source_channel,
        token: Coin {
            denom: token.denom,
            amount: token.amount,
        },
        sender: msg.sender,
        receiver: msg.receiver,
        timeout_height,
        timeout_timestamp: msg.timeout_timestamp,
        memo: msg.memo,
    }))
}

/// Parse MsgVote
fn parse_msg_vote(data: &[u8]) -> Result<CosmosMessage> {
    let msg = cosmos_sdk_proto::cosmos::gov::v1beta1::MsgVote::decode(data)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to parse MsgVote: {}", e)))?;

    Ok(CosmosMessage::Vote(MsgVote {
        proposal_id: msg.proposal_id,
        voter: msg.voter,
        option: msg.option,
    }))
}

// === IBC Parsing Functions ===

/// Helper function to parse IBC packet
fn parse_ibc_packet(packet: ibc_proto::ibc::core::channel::v1::Packet) -> IbcPacket {
    IbcPacket {
        sequence: packet.sequence,
        source_port: packet.source_port,
        source_channel: packet.source_channel,
        destination_port: packet.destination_port,
        destination_channel: packet.destination_channel,
        data: packet.data,
        timeout_height: packet.timeout_height.map(|h| IbcHeight {
            revision_number: h.revision_number,
            revision_height: h.revision_height,
        }),
        timeout_timestamp: packet.timeout_timestamp,
    }
}

/// Parse MsgRecvPacket
fn parse_msg_ibc_recv_packet(data: &[u8]) -> Result<CosmosMessage> {
    let msg = ibc_proto::ibc::core::channel::v1::MsgRecvPacket::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgRecvPacket: {}", e))
    })?;

    let packet = msg
        .packet
        .ok_or_else(|| DecoderError::invalid_structure("Missing packet in MsgRecvPacket"))?;

    let proof_height = msg
        .proof_height
        .ok_or_else(|| DecoderError::invalid_structure("Missing proof_height in MsgRecvPacket"))?;

    Ok(CosmosMessage::IbcRecvPacket(MsgIbcRecvPacket {
        packet: parse_ibc_packet(packet),
        proof_commitment: msg.proof_commitment,
        proof_height: IbcHeight {
            revision_number: proof_height.revision_number,
            revision_height: proof_height.revision_height,
        },
        signer: msg.signer,
    }))
}

/// Parse MsgAcknowledgement
fn parse_msg_ibc_acknowledgement(data: &[u8]) -> Result<CosmosMessage> {
    let msg = ibc_proto::ibc::core::channel::v1::MsgAcknowledgement::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgAcknowledgement: {}", e))
    })?;

    let packet = msg
        .packet
        .ok_or_else(|| DecoderError::invalid_structure("Missing packet in MsgAcknowledgement"))?;

    let proof_height = msg.proof_height.ok_or_else(|| {
        DecoderError::invalid_structure("Missing proof_height in MsgAcknowledgement")
    })?;

    Ok(CosmosMessage::IbcAcknowledgement(MsgIbcAcknowledgement {
        packet: parse_ibc_packet(packet),
        acknowledgement: msg.acknowledgement,
        proof_acked: msg.proof_acked,
        proof_height: IbcHeight {
            revision_number: proof_height.revision_number,
            revision_height: proof_height.revision_height,
        },
        signer: msg.signer,
    }))
}

/// Parse MsgTimeout
fn parse_msg_ibc_timeout(data: &[u8]) -> Result<CosmosMessage> {
    let msg = ibc_proto::ibc::core::channel::v1::MsgTimeout::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgTimeout: {}", e))
    })?;

    let packet = msg
        .packet
        .ok_or_else(|| DecoderError::invalid_structure("Missing packet in MsgTimeout"))?;

    let proof_height = msg
        .proof_height
        .ok_or_else(|| DecoderError::invalid_structure("Missing proof_height in MsgTimeout"))?;

    Ok(CosmosMessage::IbcTimeout(MsgIbcTimeout {
        packet: parse_ibc_packet(packet),
        proof_unreceived: msg.proof_unreceived,
        proof_height: IbcHeight {
            revision_number: proof_height.revision_number,
            revision_height: proof_height.revision_height,
        },
        next_sequence_recv: msg.next_sequence_recv,
        signer: msg.signer,
    }))
}

/// Parse MsgCreateClient
fn parse_msg_ibc_create_client(data: &[u8]) -> Result<CosmosMessage> {
    let msg = ibc_proto::ibc::core::client::v1::MsgCreateClient::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgCreateClient: {}", e))
    })?;

    let client_state = msg.client_state.ok_or_else(|| {
        DecoderError::invalid_structure("Missing client_state in MsgCreateClient")
    })?;

    let consensus_state = msg.consensus_state.ok_or_else(|| {
        DecoderError::invalid_structure("Missing consensus_state in MsgCreateClient")
    })?;

    Ok(CosmosMessage::IbcCreateClient(MsgIbcCreateClient {
        client_state: client_state.value,
        consensus_state: consensus_state.value,
        signer: msg.signer,
    }))
}

/// Parse MsgUpdateClient
fn parse_msg_ibc_update_client(data: &[u8]) -> Result<CosmosMessage> {
    let msg = ibc_proto::ibc::core::client::v1::MsgUpdateClient::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgUpdateClient: {}", e))
    })?;

    let client_message = msg.client_message.ok_or_else(|| {
        DecoderError::invalid_structure("Missing client_message in MsgUpdateClient")
    })?;

    Ok(CosmosMessage::IbcUpdateClient(MsgIbcUpdateClient {
        client_id: msg.client_id,
        client_message: client_message.value,
        signer: msg.signer,
    }))
}

// === CosmWasm Parsing Functions ===

/// Parse MsgStoreCode
fn parse_msg_store_code(data: &[u8]) -> Result<CosmosMessage> {
    // Note: CosmWasm protos are in cosmos_sdk_proto under cosmwasm module
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgStoreCode as ProtoMsgStoreCode;

    let msg = ProtoMsgStoreCode::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgStoreCode: {}", e))
    })?;

    let instantiate_permission = msg.instantiate_permission.map(|perm| AccessConfig {
        permission: perm.permission,
        addresses: perm.addresses,
    });

    Ok(CosmosMessage::StoreCode(MsgStoreCode {
        sender: msg.sender,
        wasm_byte_code: msg.wasm_byte_code,
        instantiate_permission,
    }))
}

/// Parse MsgInstantiateContract
fn parse_msg_instantiate_contract(data: &[u8]) -> Result<CosmosMessage> {
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgInstantiateContract as ProtoMsgInstantiateContract;

    let msg = ProtoMsgInstantiateContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgInstantiateContract: {}", e))
    })?;

    Ok(CosmosMessage::InstantiateContract(MsgInstantiateContract {
        sender: msg.sender,
        admin: msg.admin,
        code_id: msg.code_id,
        label: msg.label,
        msg: msg.msg,
        funds: msg.funds.into_iter().map(parse_coin).collect(),
    }))
}

/// Parse MsgExecuteContract
fn parse_msg_execute_contract(data: &[u8]) -> Result<CosmosMessage> {
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract as ProtoMsgExecuteContract;

    let msg = ProtoMsgExecuteContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgExecuteContract: {}", e))
    })?;

    Ok(CosmosMessage::ExecuteContract(MsgExecuteContract {
        sender: msg.sender,
        contract: msg.contract,
        msg: msg.msg,
        funds: msg.funds.into_iter().map(parse_coin).collect(),
    }))
}

/// Parse MsgMigrateContract
fn parse_msg_migrate_contract(data: &[u8]) -> Result<CosmosMessage> {
    use cosmos_sdk_proto::cosmwasm::wasm::v1::MsgMigrateContract as ProtoMsgMigrateContract;

    let msg = ProtoMsgMigrateContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to parse MsgMigrateContract: {}", e))
    })?;

    Ok(CosmosMessage::MigrateContract(MsgMigrateContract {
        sender: msg.sender,
        contract: msg.contract,
        code_id: msg.code_id,
        msg: msg.msg,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coin() {
        let proto_coin = cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: "uatom".to_string(),
            amount: "1000000".to_string(),
        };
        let coin = parse_coin(proto_coin);
        assert_eq!(coin.denom, "uatom");
        assert_eq!(coin.amount, "1000000");
    }

    #[test]
    fn test_parse_message_unknown_type() {
        let msg = Any {
            type_url: "/unknown.type.v1.MsgUnknown".to_string(),
            value: vec![1, 2, 3],
        };
        let parsed = parse_message(&msg).unwrap();
        match parsed {
            CosmosMessage::Unknown { type_url, .. } => {
                assert_eq!(type_url, "/unknown.type.v1.MsgUnknown");
            }
            _ => panic!("Expected Unknown message"),
        }
    }

    #[test]
    fn test_parse_invalid_protobuf() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = parse_tx(&invalid_data);
        assert!(result.is_err());
    }
}
