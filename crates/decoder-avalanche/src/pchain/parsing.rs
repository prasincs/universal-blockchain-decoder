//! P-Chain transaction parsing

use crate::common::*;
use crate::pchain::types::*;
use crate::xchain::parsing::{parse_base_tx, parse_transferable_input, parse_transferable_output};
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

// Re-export helper functions from xchain (they're the same for P-Chain)
use crate::xchain::parsing::{
    read_bytes, read_bytes_20, read_bytes_32, read_u16_be, read_u32_be, read_u64_be, read_u8,
};

// P-Chain specific transaction type IDs
const ADD_DELEGATOR_TX: u32 = 0x0000000e;
const IMPORT_TX: u32 = 0x00000011;
const EXPORT_TX: u32 = 0x00000012;
const REMOVE_SUBNET_VALIDATOR_TX: u32 = 0x00000015;
const TRANSFORM_SUBNET_TX: u32 = 0x00000016;
const ADD_PERMISSIONLESS_VALIDATOR_TX: u32 = 0x00000019;
const ADD_PERMISSIONLESS_DELEGATOR_TX: u32 = 0x0000001a;

/// Parse P-Chain transaction from raw bytes
pub fn parse_pchain_transaction(raw_bytes: &[u8]) -> Result<PChainTransaction> {
    let mut cursor = Cursor::new(raw_bytes);

    // Parse codec ID (2 bytes, big-endian)
    let codec_id = read_u16_be(&mut cursor)?;
    if codec_id != CODEC_ID {
        return Err(DecoderError::invalid_structure(format!(
            "Invalid codec ID: 0x{:04x}",
            codec_id
        )));
    }

    // Parse type ID (4 bytes, big-endian)
    let type_id = read_u32_be(&mut cursor)?;

    // Parse transaction based on type
    let tx_type = match type_id {
        BASE_TX => {
            let base = parse_base_tx(&mut cursor)?;
            PChainTxType::Base(base)
        }
        ADD_VALIDATOR_TX => parse_add_validator_tx(&mut cursor)?,
        ADD_DELEGATOR_TX => parse_add_delegator_tx(&mut cursor)?,
        CREATE_SUBNET_TX => parse_create_subnet_tx(&mut cursor)?,
        ADD_SUBNET_VALIDATOR_TX => parse_add_subnet_validator_tx(&mut cursor)?,
        IMPORT_TX => parse_import_tx(&mut cursor)?,
        EXPORT_TX => parse_export_tx(&mut cursor)?,
        REMOVE_SUBNET_VALIDATOR_TX => parse_remove_subnet_validator_tx(&mut cursor)?,
        TRANSFORM_SUBNET_TX => parse_transform_subnet_tx(&mut cursor)?,
        ADD_PERMISSIONLESS_VALIDATOR_TX => parse_add_permissionless_validator_tx(&mut cursor)?,
        ADD_PERMISSIONLESS_DELEGATOR_TX => parse_add_permissionless_delegator_tx(&mut cursor)?,
        _ => {
            // Unknown transaction type
            let mut data = Vec::new();
            cursor.read_to_end(&mut data)?;
            PChainTxType::Unknown { type_id, data }
        }
    };

    Ok(PChainTransaction {
        codec_id,
        tx_type,
        raw_bytes: raw_bytes.to_vec(),
    })
}

/// Parse validator information
fn parse_validator(cursor: &mut Cursor<&[u8]>) -> Result<Validator> {
    let node_id = read_bytes_20(cursor)?;
    let start_time = read_u64_be(cursor)?;
    let end_time = read_u64_be(cursor)?;
    let weight = read_u64_be(cursor)?;

    Ok(Validator {
        node_id,
        start_time,
        end_time,
        weight,
    })
}

/// Parse rewards owner
fn parse_rewards_owner(cursor: &mut Cursor<&[u8]>) -> Result<RewardsOwner> {
    // Type ID for output owner
    let type_id = read_u32_be(cursor)?;
    if type_id != 0x0000000b {
        // SECP256K1OutputOwners type ID
        return Err(DecoderError::invalid_structure(format!(
            "Invalid output owner type ID: 0x{:08x}",
            type_id
        )));
    }

    let locktime = read_u64_be(cursor)?;
    let threshold = read_u32_be(cursor)?;

    let num_addresses = read_u32_be(cursor)?;
    let mut addresses = Vec::with_capacity(num_addresses.min(100) as usize);
    for _ in 0..num_addresses {
        addresses.push(read_bytes_20(cursor)?);
    }

    Ok(RewardsOwner {
        locktime,
        threshold,
        addresses,
    })
}

/// Parse subnet owner
fn parse_subnet_owner(cursor: &mut Cursor<&[u8]>) -> Result<SubnetOwner> {
    // Type ID for subnet auth
    let type_id = read_u32_be(cursor)?;
    if type_id != 0x0000000a {
        // SECP256K1SubnetAuth type ID
        return Err(DecoderError::invalid_structure(format!(
            "Invalid subnet auth type ID: 0x{:08x}",
            type_id
        )));
    }

    let locktime = read_u64_be(cursor)?;
    let threshold = read_u32_be(cursor)?;

    let num_addresses = read_u32_be(cursor)?;
    let mut addresses = Vec::with_capacity(num_addresses.min(100) as usize);
    for _ in 0..num_addresses {
        addresses.push(read_bytes_20(cursor)?);
    }

    Ok(SubnetOwner {
        locktime,
        threshold,
        addresses,
    })
}

/// Parse AddValidator transaction
fn parse_add_validator_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let validator = parse_validator(cursor)?;

    // Parse stake outputs
    let num_stake = read_u32_be(cursor)?;
    let mut stake = Vec::with_capacity(num_stake.min(100) as usize);
    for _ in 0..num_stake {
        stake.push(parse_transferable_output(cursor)?);
    }

    let rewards_owner = parse_rewards_owner(cursor)?;
    let shares = read_u32_be(cursor)?;

    Ok(PChainTxType::AddValidator {
        base,
        validator,
        stake,
        rewards_owner,
        shares,
    })
}

/// Parse AddDelegator transaction
fn parse_add_delegator_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let validator = parse_validator(cursor)?;

    // Parse stake outputs
    let num_stake = read_u32_be(cursor)?;
    let mut stake = Vec::with_capacity(num_stake.min(100) as usize);
    for _ in 0..num_stake {
        stake.push(parse_transferable_output(cursor)?);
    }

    let rewards_owner = parse_rewards_owner(cursor)?;

    Ok(PChainTxType::AddDelegator {
        base,
        validator,
        stake,
        rewards_owner,
    })
}

/// Parse CreateSubnet transaction
fn parse_create_subnet_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let owner = parse_subnet_owner(cursor)?;

    Ok(PChainTxType::CreateSubnet { base, owner })
}

/// Parse AddSubnetValidator transaction
fn parse_add_subnet_validator_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let validator = parse_validator(cursor)?;
    let subnet_id = read_bytes_32(cursor)?;

    Ok(PChainTxType::AddSubnetValidator {
        base,
        validator,
        subnet_id,
    })
}

/// Parse Import transaction
fn parse_import_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let source_chain = read_bytes_32(cursor)?;

    let num_inputs = read_u32_be(cursor)?;
    let mut imported_inputs = Vec::with_capacity(num_inputs.min(1000) as usize);
    for _ in 0..num_inputs {
        imported_inputs.push(parse_transferable_input(cursor)?);
    }

    Ok(PChainTxType::Import {
        base,
        source_chain,
        imported_inputs,
    })
}

/// Parse Export transaction
fn parse_export_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let destination_chain = read_bytes_32(cursor)?;

    let num_outputs = read_u32_be(cursor)?;
    let mut exported_outputs = Vec::with_capacity(num_outputs.min(1000) as usize);
    for _ in 0..num_outputs {
        exported_outputs.push(parse_transferable_output(cursor)?);
    }

    Ok(PChainTxType::Export {
        base,
        destination_chain,
        exported_outputs,
    })
}

/// Parse RemoveSubnetValidator transaction
fn parse_remove_subnet_validator_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let node_id = read_bytes_20(cursor)?;
    let subnet_id = read_bytes_32(cursor)?;

    Ok(PChainTxType::RemoveSubnetValidator {
        base,
        node_id,
        subnet_id,
    })
}

/// Parse TransformSubnet transaction
fn parse_transform_subnet_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let subnet_id = read_bytes_32(cursor)?;
    let asset_id = read_bytes_32(cursor)?;
    let initial_supply = read_u64_be(cursor)?;
    let maximum_supply = read_u64_be(cursor)?;
    let min_consumption_rate = read_u64_be(cursor)?;
    let max_consumption_rate = read_u64_be(cursor)?;
    let min_validator_stake = read_u64_be(cursor)?;
    let max_validator_stake = read_u64_be(cursor)?;
    let min_stake_duration = read_u32_be(cursor)?;
    let max_stake_duration = read_u32_be(cursor)?;
    let min_delegation_fee = read_u32_be(cursor)?;
    let min_delegator_stake = read_u64_be(cursor)?;
    let max_validator_weight_factor = read_u8(cursor)?;
    let uptime_requirement = read_u32_be(cursor)?;

    Ok(PChainTxType::TransformSubnet {
        base,
        subnet_id,
        asset_id,
        initial_supply,
        maximum_supply,
        min_consumption_rate,
        max_consumption_rate,
        min_validator_stake,
        max_validator_stake,
        min_stake_duration,
        max_stake_duration,
        min_delegation_fee,
        min_delegator_stake,
        max_validator_weight_factor,
        uptime_requirement,
    })
}

/// Parse AddPermissionlessValidator transaction
fn parse_add_permissionless_validator_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let validator = parse_validator(cursor)?;
    let subnet_id = read_bytes_32(cursor)?;

    // Parse optional signer (BLS proof of possession)
    let signer_len = read_u32_be(cursor)?;
    let signer = if signer_len > 0 {
        Some(read_bytes(cursor, signer_len as usize)?)
    } else {
        None
    };

    // Parse stake outputs
    let num_stake = read_u32_be(cursor)?;
    let mut stake = Vec::with_capacity(num_stake.min(100) as usize);
    for _ in 0..num_stake {
        stake.push(parse_transferable_output(cursor)?);
    }

    let validator_rewards_owner = parse_rewards_owner(cursor)?;
    let delegator_rewards_owner = parse_rewards_owner(cursor)?;
    let delegation_shares = read_u32_be(cursor)?;

    Ok(PChainTxType::AddPermissionlessValidator {
        base,
        validator,
        subnet_id,
        signer,
        stake,
        validator_rewards_owner,
        delegator_rewards_owner,
        delegation_shares,
    })
}

/// Parse AddPermissionlessDelegator transaction
fn parse_add_permissionless_delegator_tx(cursor: &mut Cursor<&[u8]>) -> Result<PChainTxType> {
    let base = parse_base_tx(cursor)?;
    let validator = parse_validator(cursor)?;
    let subnet_id = read_bytes_32(cursor)?;

    // Parse stake outputs
    let num_stake = read_u32_be(cursor)?;
    let mut stake = Vec::with_capacity(num_stake.min(100) as usize);
    for _ in 0..num_stake {
        stake.push(parse_transferable_output(cursor)?);
    }

    let rewards_owner = parse_rewards_owner(cursor)?;

    Ok(PChainTxType::AddPermissionlessDelegator {
        base,
        validator,
        subnet_id,
        stake,
        rewards_owner,
    })
}
