//! Sui transaction parsing using BCS encoding

use crate::types::*;
use decoder_encodings::bcs::{
    read_bool, read_bytes, read_fixed_bytes, read_option, read_string, read_u16, read_u64, read_u8,
    read_uleb128,
};
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Parse a Sui transaction from BCS-encoded bytes
pub fn parse_transaction(bytes: &[u8]) -> Result<SuiTransaction> {
    let mut cursor = Cursor::new(bytes);

    // Parse transaction data
    let data = parse_transaction_data(&mut cursor)?;

    // Parse signatures
    let signatures_len = read_uleb128(&mut cursor)?;
    let mut signatures = Vec::with_capacity(signatures_len as usize);
    for _ in 0..signatures_len {
        signatures.push(parse_signature(&mut cursor)?);
    }

    Ok(SuiTransaction {
        data,
        signatures,
        raw_bytes: bytes.to_vec(),
    })
}

/// Parse transaction data
fn parse_transaction_data<R: Read>(reader: &mut R) -> Result<TransactionData> {
    // Transaction kind (variant index)
    let kind = parse_transaction_kind(reader)?;

    // Sender address
    let sender = read_fixed_bytes::<_, 32>(reader)?;

    // Gas data
    let gas_data = parse_gas_data(reader)?;

    // Expiration
    let expiration = parse_transaction_expiration(reader)?;

    Ok(TransactionData {
        kind,
        sender,
        gas_data,
        expiration,
    })
}

/// Parse transaction kind
fn parse_transaction_kind<R: Read>(reader: &mut R) -> Result<TransactionKind> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => {
            // ProgrammableTransaction
            let pt = parse_programmable_transaction(reader)?;
            Ok(TransactionKind::ProgrammableTransaction(pt))
        }
        1 => {
            // ChangeEpoch
            let epoch = read_u64(reader)?;
            let storage_charge = read_u64(reader)?;
            let computation_charge = read_u64(reader)?;
            Ok(TransactionKind::ChangeEpoch {
                epoch,
                storage_charge,
                computation_charge,
            })
        }
        2 => {
            // Genesis
            let objects_len = read_uleb128(reader)?;
            let mut objects = Vec::with_capacity(objects_len as usize);
            for _ in 0..objects_len {
                objects.push(read_fixed_bytes::<_, 32>(reader)?);
            }
            Ok(TransactionKind::Genesis { objects })
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid transaction kind variant: {}",
            variant_index
        ))),
    }
}

/// Parse programmable transaction
fn parse_programmable_transaction<R: Read>(reader: &mut R) -> Result<ProgrammableTransaction> {
    // Inputs
    let inputs_len = read_uleb128(reader)?;
    let mut inputs = Vec::with_capacity(inputs_len as usize);
    for _ in 0..inputs_len {
        inputs.push(parse_call_arg(reader)?);
    }

    // Commands
    let commands_len = read_uleb128(reader)?;
    let mut commands = Vec::with_capacity(commands_len as usize);
    for _ in 0..commands_len {
        commands.push(parse_command(reader)?);
    }

    Ok(ProgrammableTransaction { inputs, commands })
}

/// Parse call argument
fn parse_call_arg<R: Read>(reader: &mut R) -> Result<CallArg> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => {
            // Pure
            let bytes = read_bytes(reader)?;
            Ok(CallArg::Pure(bytes))
        }
        1 => {
            // Object
            let object_arg = parse_object_arg(reader)?;
            Ok(CallArg::Object(object_arg))
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid call arg variant: {}",
            variant_index
        ))),
    }
}

/// Parse object argument
fn parse_object_arg<R: Read>(reader: &mut R) -> Result<ObjectArg> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => {
            // ImmOrOwnedObject
            let object_ref = parse_object_ref(reader)?;
            Ok(ObjectArg::ImmOrOwnedObject(object_ref))
        }
        1 => {
            // SharedObject
            let object_id = read_fixed_bytes::<_, 32>(reader)?;
            let initial_shared_version = read_u64(reader)?;
            let mutable = read_bool(reader)?;
            Ok(ObjectArg::SharedObject {
                object_id,
                initial_shared_version,
                mutable,
            })
        }
        2 => {
            // Receiving
            let object_ref = parse_object_ref(reader)?;
            Ok(ObjectArg::Receiving(object_ref))
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid object arg variant: {}",
            variant_index
        ))),
    }
}

/// Parse object reference
fn parse_object_ref<R: Read>(reader: &mut R) -> Result<ObjectRef> {
    let object_id = read_fixed_bytes::<_, 32>(reader)?;
    let version = read_u64(reader)?;
    let digest = read_fixed_bytes::<_, 32>(reader)?;

    Ok(ObjectRef {
        object_id,
        version,
        digest,
    })
}

/// Parse command
fn parse_command<R: Read>(reader: &mut R) -> Result<Command> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => parse_move_call_command(reader),
        1 => parse_transfer_objects_command(reader),
        2 => parse_split_coins_command(reader),
        3 => parse_merge_coins_command(reader),
        4 => parse_publish_command(reader),
        5 => parse_make_move_vec_command(reader),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid command variant: {}",
            variant_index
        ))),
    }
}

/// Parse MoveCall command
fn parse_move_call_command<R: Read>(reader: &mut R) -> Result<Command> {
    let package = read_fixed_bytes::<_, 32>(reader)?;
    let module = read_string(reader)?;
    let function = read_string(reader)?;

    // Type arguments
    let type_args_len = read_uleb128(reader)?;
    let mut type_arguments = Vec::with_capacity(type_args_len as usize);
    for _ in 0..type_args_len {
        type_arguments.push(parse_type_tag(reader)?);
    }

    // Arguments
    let args_len = read_uleb128(reader)?;
    let mut arguments = Vec::with_capacity(args_len as usize);
    for _ in 0..args_len {
        arguments.push(parse_argument(reader)?);
    }

    Ok(Command::MoveCall {
        package,
        module,
        function,
        type_arguments,
        arguments,
    })
}

/// Parse TransferObjects command
fn parse_transfer_objects_command<R: Read>(reader: &mut R) -> Result<Command> {
    let objects_len = read_uleb128(reader)?;
    let mut objects = Vec::with_capacity(objects_len as usize);
    for _ in 0..objects_len {
        objects.push(parse_argument(reader)?);
    }

    let address = parse_argument(reader)?;

    Ok(Command::TransferObjects { objects, address })
}

/// Parse SplitCoins command
fn parse_split_coins_command<R: Read>(reader: &mut R) -> Result<Command> {
    let coin = parse_argument(reader)?;

    let amounts_len = read_uleb128(reader)?;
    let mut amounts = Vec::with_capacity(amounts_len as usize);
    for _ in 0..amounts_len {
        amounts.push(parse_argument(reader)?);
    }

    Ok(Command::SplitCoins { coin, amounts })
}

/// Parse MergeCoins command
fn parse_merge_coins_command<R: Read>(reader: &mut R) -> Result<Command> {
    let destination = parse_argument(reader)?;

    let sources_len = read_uleb128(reader)?;
    let mut sources = Vec::with_capacity(sources_len as usize);
    for _ in 0..sources_len {
        sources.push(parse_argument(reader)?);
    }

    Ok(Command::MergeCoins {
        destination,
        sources,
    })
}

/// Parse Publish command
fn parse_publish_command<R: Read>(reader: &mut R) -> Result<Command> {
    let modules_len = read_uleb128(reader)?;
    let mut modules = Vec::with_capacity(modules_len as usize);
    for _ in 0..modules_len {
        modules.push(read_bytes(reader)?);
    }

    let dependencies_len = read_uleb128(reader)?;
    let mut dependencies = Vec::with_capacity(dependencies_len as usize);
    for _ in 0..dependencies_len {
        dependencies.push(read_fixed_bytes::<_, 32>(reader)?);
    }

    Ok(Command::Publish {
        modules,
        dependencies,
    })
}

/// Parse MakeMoveVec command
fn parse_make_move_vec_command<R: Read>(reader: &mut R) -> Result<Command> {
    let type_tag = read_option(reader, parse_type_tag)?;

    let elements_len = read_uleb128(reader)?;
    let mut elements = Vec::with_capacity(elements_len as usize);
    for _ in 0..elements_len {
        elements.push(parse_argument(reader)?);
    }

    Ok(Command::MakeMoveVec { type_tag, elements })
}

/// Parse argument
fn parse_argument<R: Read>(reader: &mut R) -> Result<Argument> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => Ok(Argument::GasCoin),
        1 => {
            let index = read_u16(reader)?;
            Ok(Argument::Input(index))
        }
        2 => {
            let index = read_u16(reader)?;
            Ok(Argument::Result(index))
        }
        3 => {
            let outer_index = read_u16(reader)?;
            let inner_index = read_u16(reader)?;
            Ok(Argument::NestedResult(outer_index, inner_index))
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid argument variant: {}",
            variant_index
        ))),
    }
}

/// Parse type tag
fn parse_type_tag<R: Read>(reader: &mut R) -> Result<TypeTag> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => Ok(TypeTag::Bool),
        1 => Ok(TypeTag::U8),
        2 => Ok(TypeTag::U64),
        3 => Ok(TypeTag::U128),
        4 => Ok(TypeTag::Address),
        5 => Ok(TypeTag::Signer),
        6 => {
            let inner = parse_type_tag(reader)?;
            Ok(TypeTag::Vector(Box::new(inner)))
        }
        7 => {
            let struct_tag = parse_struct_tag(reader)?;
            Ok(TypeTag::Struct(struct_tag))
        }
        8 => Ok(TypeTag::U16),
        9 => Ok(TypeTag::U32),
        10 => Ok(TypeTag::U256),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid type tag variant: {}",
            variant_index
        ))),
    }
}

/// Parse struct tag
fn parse_struct_tag<R: Read>(reader: &mut R) -> Result<StructTag> {
    let address = read_fixed_bytes::<_, 32>(reader)?;
    let module = read_string(reader)?;
    let name = read_string(reader)?;

    let type_params_len = read_uleb128(reader)?;
    let mut type_params = Vec::with_capacity(type_params_len as usize);
    for _ in 0..type_params_len {
        type_params.push(parse_type_tag(reader)?);
    }

    Ok(StructTag {
        address,
        module,
        name,
        type_params,
    })
}

/// Parse gas data
fn parse_gas_data<R: Read>(reader: &mut R) -> Result<GasData> {
    // Payment objects
    let payment_len = read_uleb128(reader)?;
    let mut payment = Vec::with_capacity(payment_len as usize);
    for _ in 0..payment_len {
        payment.push(parse_object_ref(reader)?);
    }

    let owner = read_fixed_bytes::<_, 32>(reader)?;
    let price = read_u64(reader)?;
    let budget = read_u64(reader)?;

    Ok(GasData {
        payment,
        owner,
        price,
        budget,
    })
}

/// Parse transaction expiration
fn parse_transaction_expiration<R: Read>(reader: &mut R) -> Result<TransactionExpiration> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => Ok(TransactionExpiration::None),
        1 => {
            let epoch = read_u64(reader)?;
            Ok(TransactionExpiration::Epoch(epoch))
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid transaction expiration variant: {}",
            variant_index
        ))),
    }
}

/// Parse signature
fn parse_signature<R: Read>(reader: &mut R) -> Result<SuiSignature> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => {
            // Ed25519
            let signature = read_fixed_bytes::<_, 64>(reader)?;
            let public_key = read_fixed_bytes::<_, 32>(reader)?;
            Ok(SuiSignature::Ed25519 {
                signature,
                public_key,
            })
        }
        1 => {
            // Secp256k1
            let signature = read_bytes(reader)?;
            let public_key = read_bytes(reader)?;
            Ok(SuiSignature::Secp256k1 {
                signature,
                public_key,
            })
        }
        2 => {
            // Secp256r1
            let signature = read_bytes(reader)?;
            let public_key = read_bytes(reader)?;
            Ok(SuiSignature::Secp256r1 {
                signature,
                public_key,
            })
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid signature variant: {}",
            variant_index
        ))),
    }
}
