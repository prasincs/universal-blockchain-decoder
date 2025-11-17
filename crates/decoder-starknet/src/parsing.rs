//! Transaction parsing logic
//!
//! Parses raw bytes into Starknet transaction types using decoder-crypto-zk primitives.

use crate::types::*;
use decoder_crypto_zk::FieldElement;
use decoder_primitives::prelude::*;
use std::io::{Cursor, Read};

/// Parse a Starknet transaction from raw bytes
///
/// Format (simplified):
/// - 1 byte: version
/// - 1 byte: transaction type
/// - Remaining bytes: transaction-specific data
pub fn parse_transaction(raw_bytes: &[u8]) -> Result<StarknetTxVariant> {
    let mut cursor = Cursor::new(raw_bytes);

    // Read version
    let version = read_u8(&mut cursor)?;
    let tx_version = match version {
        0 => StarknetVersion::V0,
        1 => StarknetVersion::V1,
        3 => StarknetVersion::V3,
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Unsupported Starknet version: {}",
                version
            )))
        }
    };

    // Read transaction type
    let tx_type_byte = read_u8(&mut cursor)?;
    let tx_type = match tx_type_byte {
        0 => StarknetTxType::Invoke,
        1 => StarknetTxType::Declare,
        2 => StarknetTxType::DeployAccount,
        3 => StarknetTxType::L1Handler,
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Unsupported transaction type: {}",
                tx_type_byte
            )))
        }
    };

    // Parse based on type and version
    match (tx_type, tx_version) {
        (StarknetTxType::Invoke, StarknetVersion::V1) => {
            Ok(StarknetTxVariant::InvokeV1(parse_invoke_v1(&mut cursor)?))
        }
        (StarknetTxType::Invoke, StarknetVersion::V3) => {
            Ok(StarknetTxVariant::InvokeV3(parse_invoke_v3(&mut cursor)?))
        }
        (StarknetTxType::Declare, StarknetVersion::V0) => {
            Ok(StarknetTxVariant::DeclareV0(parse_declare_v0(&mut cursor)?))
        }
        (StarknetTxType::Declare, StarknetVersion::V3) => {
            Ok(StarknetTxVariant::DeclareV3(parse_declare_v3(&mut cursor)?))
        }
        (StarknetTxType::DeployAccount, StarknetVersion::V1) => Ok(
            StarknetTxVariant::DeployAccountV1(parse_deploy_account_v1(&mut cursor)?),
        ),
        (StarknetTxType::DeployAccount, StarknetVersion::V3) => Ok(
            StarknetTxVariant::DeployAccountV3(parse_deploy_account_v3(&mut cursor)?),
        ),
        _ => Err(DecoderError::invalid_structure(format!(
            "Unsupported combination: {:?} {:?}",
            tx_type, tx_version
        ))),
    }
}

fn parse_invoke_v1(cursor: &mut Cursor<&[u8]>) -> Result<InvokeTxV1> {
    let sender_address = read_field_element(cursor)?;
    let calldata = read_field_element_array(cursor)?;
    let max_fee = read_field_element(cursor)?;
    let signature = read_field_element_array(cursor)?;
    let nonce = read_field_element(cursor)?;

    Ok(InvokeTxV1 {
        sender_address,
        calldata,
        max_fee,
        signature,
        nonce,
    })
}

fn parse_invoke_v3(cursor: &mut Cursor<&[u8]>) -> Result<InvokeTxV3> {
    let sender_address = read_field_element(cursor)?;
    let calldata = read_field_element_array(cursor)?;
    let signature = read_field_element_array(cursor)?;
    let nonce = read_field_element(cursor)?;
    let resource_bounds = read_resource_bounds(cursor)?;
    let tip = read_u64(cursor)?;
    let paymaster_data = read_field_element_array(cursor)?;
    let account_deployment_data = read_field_element_array(cursor)?;
    let nonce_data_availability_mode = read_da_mode(cursor)?;
    let fee_data_availability_mode = read_da_mode(cursor)?;

    Ok(InvokeTxV3 {
        sender_address,
        calldata,
        signature,
        nonce,
        resource_bounds,
        tip,
        paymaster_data,
        account_deployment_data,
        nonce_data_availability_mode,
        fee_data_availability_mode,
    })
}

fn parse_declare_v0(cursor: &mut Cursor<&[u8]>) -> Result<DeclareTxV0> {
    let class_hash = read_field_element(cursor)?;
    let sender_address = read_field_element(cursor)?;
    let max_fee = read_field_element(cursor)?;
    let signature = read_field_element_array(cursor)?;

    Ok(DeclareTxV0 {
        class_hash,
        sender_address,
        max_fee,
        signature,
    })
}

fn parse_declare_v3(cursor: &mut Cursor<&[u8]>) -> Result<DeclareTxV3> {
    let class_hash = read_field_element(cursor)?;
    let compiled_class_hash = read_field_element(cursor)?;
    let sender_address = read_field_element(cursor)?;
    let signature = read_field_element_array(cursor)?;
    let nonce = read_field_element(cursor)?;
    let resource_bounds = read_resource_bounds(cursor)?;
    let tip = read_u64(cursor)?;
    let paymaster_data = read_field_element_array(cursor)?;
    let account_deployment_data = read_field_element_array(cursor)?;
    let nonce_data_availability_mode = read_da_mode(cursor)?;
    let fee_data_availability_mode = read_da_mode(cursor)?;

    Ok(DeclareTxV3 {
        class_hash,
        compiled_class_hash,
        sender_address,
        signature,
        nonce,
        resource_bounds,
        tip,
        paymaster_data,
        account_deployment_data,
        nonce_data_availability_mode,
        fee_data_availability_mode,
    })
}

fn parse_deploy_account_v1(cursor: &mut Cursor<&[u8]>) -> Result<DeployAccountTxV1> {
    let class_hash = read_field_element(cursor)?;
    let constructor_calldata = read_field_element_array(cursor)?;
    let contract_address_salt = read_field_element(cursor)?;
    let max_fee = read_field_element(cursor)?;
    let signature = read_field_element_array(cursor)?;
    let nonce = read_field_element(cursor)?;

    Ok(DeployAccountTxV1 {
        class_hash,
        constructor_calldata,
        contract_address_salt,
        max_fee,
        signature,
        nonce,
    })
}

fn parse_deploy_account_v3(cursor: &mut Cursor<&[u8]>) -> Result<DeployAccountTxV3> {
    let class_hash = read_field_element(cursor)?;
    let constructor_calldata = read_field_element_array(cursor)?;
    let contract_address_salt = read_field_element(cursor)?;
    let signature = read_field_element_array(cursor)?;
    let nonce = read_field_element(cursor)?;
    let resource_bounds = read_resource_bounds(cursor)?;
    let tip = read_u64(cursor)?;
    let paymaster_data = read_field_element_array(cursor)?;
    let nonce_data_availability_mode = read_da_mode(cursor)?;
    let fee_data_availability_mode = read_da_mode(cursor)?;

    Ok(DeployAccountTxV3 {
        class_hash,
        constructor_calldata,
        contract_address_salt,
        signature,
        nonce,
        resource_bounds,
        tip,
        paymaster_data,
        nonce_data_availability_mode,
        fee_data_availability_mode,
    })
}

// Helper functions for reading primitive types

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let mut buf = [0u8; 8];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_be_bytes(buf))
}

fn read_u128(cursor: &mut Cursor<&[u8]>) -> Result<u128> {
    let mut buf = [0u8; 16];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Failed to read u128: {}", e)))?;
    Ok(u128::from_be_bytes(buf))
}

fn read_field_element(cursor: &mut Cursor<&[u8]>) -> Result<FieldElement> {
    let mut buf = [0u8; 32];
    cursor.read_exact(&mut buf).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to read field element: {}", e))
    })?;

    // from_bytes_be always succeeds for 32-byte arrays
    Ok(FieldElement::from_bytes_be(&buf))
}

fn read_field_element_array(cursor: &mut Cursor<&[u8]>) -> Result<Vec<FieldElement>> {
    let length = read_u64(cursor)? as usize;

    // Sanity check: prevent DOS attacks with huge arrays
    if length > 10000 {
        return Err(DecoderError::invalid_structure(format!(
            "Field element array too large: {}",
            length
        )));
    }

    let mut elements = Vec::with_capacity(length);
    for _ in 0..length {
        elements.push(read_field_element(cursor)?);
    }

    Ok(elements)
}

fn read_resource_bounds(cursor: &mut Cursor<&[u8]>) -> Result<ResourceBounds> {
    let l1_gas = read_resource_bound(cursor)?;
    let l2_gas = read_resource_bound(cursor)?;

    Ok(ResourceBounds { l1_gas, l2_gas })
}

fn read_resource_bound(cursor: &mut Cursor<&[u8]>) -> Result<ResourceBound> {
    let max_amount = read_u64(cursor)?;
    let max_price_per_unit = read_u128(cursor)?;

    Ok(ResourceBound {
        max_amount,
        max_price_per_unit,
    })
}

fn read_da_mode(cursor: &mut Cursor<&[u8]>) -> Result<DataAvailabilityMode> {
    let mode_byte = read_u8(cursor)?;

    match mode_byte {
        0 => Ok(DataAvailabilityMode::L1),
        1 => Ok(DataAvailabilityMode::L2),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid DA mode: {}",
            mode_byte
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_field_element() {
        let data = [0u8; 32];
        let mut cursor = Cursor::new(&data[..]);
        let field = read_field_element(&mut cursor).unwrap();
        assert_eq!(field, FieldElement::ZERO);
    }

    #[test]
    fn test_read_field_element_array() {
        // Length (8 bytes) + 2 field elements (32 bytes each)
        let mut data = [0u8; 8 + 64];
        // Set length to 2
        data[7] = 2;

        let mut cursor = Cursor::new(&data[..]);
        let array = read_field_element_array(&mut cursor).unwrap();
        assert_eq!(array.len(), 2);
    }

    #[test]
    fn test_read_da_mode() {
        let data = [0u8];
        let mut cursor = Cursor::new(&data[..]);
        let mode = read_da_mode(&mut cursor).unwrap();
        assert_eq!(mode, DataAvailabilityMode::L1);

        let data = [1u8];
        let mut cursor = Cursor::new(&data[..]);
        let mode = read_da_mode(&mut cursor).unwrap();
        assert_eq!(mode, DataAvailabilityMode::L2);
    }
}
