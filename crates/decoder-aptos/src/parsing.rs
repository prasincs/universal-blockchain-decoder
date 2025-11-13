//! Aptos transaction parsing using BCS encoding

use crate::types::*;
use decoder_encodings::bcs::{
    read_bytes, read_fixed_bytes, read_option, read_string, read_u64, read_u8, read_uleb128,
};
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Parse an Aptos signed transaction from BCS-encoded bytes
pub fn parse_signed_transaction(bytes: &[u8]) -> Result<SignedTransaction> {
    let mut cursor = Cursor::new(bytes);

    // Parse raw transaction
    let raw_txn = parse_raw_transaction(&mut cursor)?;

    // Parse authenticator
    let authenticator = parse_transaction_authenticator(&mut cursor)?;

    Ok(SignedTransaction {
        raw_txn,
        authenticator,
    })
}

/// Parse a raw transaction (unsigned)
fn parse_raw_transaction<R: Read>(reader: &mut R) -> Result<RawTransaction> {
    // Sender address (32 bytes)
    let sender = read_fixed_bytes::<_, 32>(reader)?;

    // Sequence number
    let sequence_number = read_u64(reader)?;

    // Payload
    let payload = parse_transaction_payload(reader)?;

    // Max gas amount
    let max_gas_amount = read_u64(reader)?;

    // Gas unit price
    let gas_unit_price = read_u64(reader)?;

    // Expiration timestamp (seconds since Unix epoch)
    let expiration_timestamp_secs = read_u64(reader)?;

    // Chain ID
    let chain_id = read_u8(reader)?;

    Ok(RawTransaction {
        sender,
        sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        chain_id,
    })
}

/// Parse transaction payload
fn parse_transaction_payload<R: Read>(reader: &mut R) -> Result<TransactionPayload> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => parse_script_payload(reader),
        1 => {
            // Deprecated module bundle
            Err(DecoderError::invalid_structure(
                "Module bundle payload is deprecated",
            ))
        }
        2 => parse_entry_function_payload(reader),
        3 => parse_multisig_payload(reader),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid transaction payload variant: {}",
            variant_index
        ))),
    }
}

/// Parse script payload
fn parse_script_payload<R: Read>(reader: &mut R) -> Result<TransactionPayload> {
    let code = read_bytes(reader)?;

    // Type arguments
    let type_args_len = read_uleb128(reader)?;
    let mut type_args = Vec::with_capacity(type_args_len as usize);
    for _ in 0..type_args_len {
        type_args.push(parse_type_tag(reader)?);
    }

    // Arguments
    let args_len = read_uleb128(reader)?;
    let mut args = Vec::with_capacity(args_len as usize);
    for _ in 0..args_len {
        args.push(read_bytes(reader)?);
    }

    Ok(TransactionPayload::Script {
        code,
        type_args,
        args,
    })
}

/// Parse entry function payload
fn parse_entry_function_payload<R: Read>(reader: &mut R) -> Result<TransactionPayload> {
    // Module ID
    let module = parse_module_id(reader)?;

    // Function name
    let function = read_string(reader)?;

    // Type arguments
    let type_args_len = read_uleb128(reader)?;
    let mut type_args = Vec::with_capacity(type_args_len as usize);
    for _ in 0..type_args_len {
        type_args.push(parse_type_tag(reader)?);
    }

    // Arguments
    let args_len = read_uleb128(reader)?;
    let mut args = Vec::with_capacity(args_len as usize);
    for _ in 0..args_len {
        args.push(read_bytes(reader)?);
    }

    Ok(TransactionPayload::EntryFunction {
        module,
        function,
        type_args,
        args,
    })
}

/// Parse multisig payload
fn parse_multisig_payload<R: Read>(reader: &mut R) -> Result<TransactionPayload> {
    let multisig_address = read_fixed_bytes::<_, 32>(reader)?;

    // Optional inner payload
    let transaction_payload = read_option(reader, |r| Ok(Box::new(parse_transaction_payload(r)?)))?;

    Ok(TransactionPayload::Multisig {
        multisig_address,
        transaction_payload,
    })
}

/// Parse module ID
fn parse_module_id<R: Read>(reader: &mut R) -> Result<ModuleId> {
    let address = read_fixed_bytes::<_, 32>(reader)?;
    let name = read_string(reader)?;

    Ok(ModuleId { address, name })
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
            // Vector
            let inner = parse_type_tag(reader)?;
            Ok(TypeTag::Vector(Box::new(inner)))
        }
        7 => {
            // Struct
            let struct_tag = parse_struct_tag(reader)?;
            Ok(TypeTag::Struct(struct_tag))
        }
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

    // Type parameters
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

/// Parse transaction authenticator
fn parse_transaction_authenticator<R: Read>(reader: &mut R) -> Result<TransactionAuthenticator> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => {
            // Ed25519
            let public_key = read_fixed_bytes::<_, 32>(reader)?;
            let signature = read_fixed_bytes::<_, 64>(reader)?;

            Ok(TransactionAuthenticator::Ed25519 {
                public_key,
                signature,
            })
        }
        1 => {
            // Multi-Ed25519
            let public_keys_len = read_uleb128(reader)?;
            let mut public_keys = Vec::with_capacity(public_keys_len as usize);
            for _ in 0..public_keys_len {
                public_keys.push(read_fixed_bytes::<_, 32>(reader)?);
            }

            let signatures_len = read_uleb128(reader)?;
            let mut signatures = Vec::with_capacity(signatures_len as usize);
            for _ in 0..signatures_len {
                signatures.push(read_fixed_bytes::<_, 64>(reader)?);
            }

            let bitmap = read_bytes(reader)?;

            Ok(TransactionAuthenticator::MultiEd25519 {
                public_keys,
                signatures,
                bitmap,
            })
        }
        2 => {
            // Multi-agent
            let sender = Box::new(parse_account_authenticator(reader)?);

            let secondary_signer_addresses_len = read_uleb128(reader)?;
            let mut secondary_signer_addresses =
                Vec::with_capacity(secondary_signer_addresses_len as usize);
            for _ in 0..secondary_signer_addresses_len {
                secondary_signer_addresses.push(read_fixed_bytes::<_, 32>(reader)?);
            }

            let secondary_signers_len = read_uleb128(reader)?;
            let mut secondary_signers = Vec::with_capacity(secondary_signers_len as usize);
            for _ in 0..secondary_signers_len {
                secondary_signers.push(parse_account_authenticator(reader)?);
            }

            Ok(TransactionAuthenticator::MultiAgent {
                sender,
                secondary_signer_addresses,
                secondary_signers,
            })
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid transaction authenticator variant: {}",
            variant_index
        ))),
    }
}

/// Parse account authenticator (simplified for now, same as transaction authenticator)
fn parse_account_authenticator<R: Read>(reader: &mut R) -> Result<TransactionAuthenticator> {
    let variant_index = read_u8(reader)?;

    match variant_index {
        0 => {
            // Ed25519
            let public_key = read_fixed_bytes::<_, 32>(reader)?;
            let signature = read_fixed_bytes::<_, 64>(reader)?;

            Ok(TransactionAuthenticator::Ed25519 {
                public_key,
                signature,
            })
        }
        1 => {
            // Multi-Ed25519
            let public_keys_len = read_uleb128(reader)?;
            let mut public_keys = Vec::with_capacity(public_keys_len as usize);
            for _ in 0..public_keys_len {
                public_keys.push(read_fixed_bytes::<_, 32>(reader)?);
            }

            let signatures_len = read_uleb128(reader)?;
            let mut signatures = Vec::with_capacity(signatures_len as usize);
            for _ in 0..signatures_len {
                signatures.push(read_fixed_bytes::<_, 64>(reader)?);
            }

            let bitmap = read_bytes(reader)?;

            Ok(TransactionAuthenticator::MultiEd25519 {
                public_keys,
                signatures,
                bitmap,
            })
        }
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid account authenticator variant: {}",
            variant_index
        ))),
    }
}
