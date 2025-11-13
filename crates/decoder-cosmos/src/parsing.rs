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
        type_urls::MSG_SEND => parse_msg_send(&msg.value),
        type_urls::MSG_MULTI_SEND => parse_msg_multi_send(&msg.value),
        type_urls::MSG_DELEGATE => parse_msg_delegate(&msg.value),
        type_urls::MSG_UNDELEGATE => parse_msg_undelegate(&msg.value),
        type_urls::MSG_BEGIN_REDELEGATE => parse_msg_begin_redelegate(&msg.value),
        // TODO: Requires ibc feature flags
        // type_urls::MSG_IBC_TRANSFER => parse_msg_ibc_transfer(&msg.value),
        type_urls::MSG_VOTE => parse_msg_vote(&msg.value),
        // TODO: Requires cosmwasm feature flags
        // type_urls::MSG_EXECUTE_CONTRACT => parse_msg_execute_contract(&msg.value),
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

/// Parse IBC transfer message
/// TODO: Requires IBC feature flags in cosmos-sdk-proto
#[allow(dead_code)]
fn parse_msg_ibc_transfer(_data: &[u8]) -> Result<CosmosMessage> {
    Err(DecoderError::invalid_structure(
        "IBC transfer parsing requires additional dependencies",
    ))
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

/// Parse MsgExecuteContract (CosmWasm)
/// TODO: Requires CosmWasm feature flags in cosmos-sdk-proto
#[allow(dead_code)]
fn parse_msg_execute_contract(_data: &[u8]) -> Result<CosmosMessage> {
    Err(DecoderError::invalid_structure(
        "CosmWasm parsing requires additional dependencies",
    ))
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
