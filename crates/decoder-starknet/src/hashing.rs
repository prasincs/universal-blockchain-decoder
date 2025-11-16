//! Transaction hash computation
//!
//! Starknet uses two hash functions:
//! - **Pedersen hash** for v0 and v1 transactions (legacy)
//! - **Poseidon hash** for v3 transactions (current)

use crate::types::*;
use decoder_crypto_zk::{
    hash::pedersen::PedersenHasher, hash::poseidon::PoseidonHash, FieldElement,
};
use decoder_primitives::prelude::*;

// Transaction type prefixes for hash computation
const INVOKE_PREFIX: &str = "invoke";
const DECLARE_PREFIX: &str = "declare";
const DEPLOY_ACCOUNT_PREFIX: &str = "deploy_account";

/// Hash INVOKE transaction (v1) using Pedersen hash
pub fn hash_invoke_v1(tx: &InvokeTxV1) -> Result<Vec<u8>> {
    let mut hasher = PedersenHasher::new();

    // Hash format: hash(prefix, sender, calldata_hash, max_fee, nonce, chain_id)
    hasher.update(prefix_to_field(INVOKE_PREFIX)?);
    hasher.update(tx.sender_address);
    hasher.update(hash_calldata_pedersen(&tx.calldata)?);
    hasher.update(tx.max_fee);
    hasher.update(tx.nonce);

    let hash = hasher.finalize();

    Ok(hash.to_bytes_be().to_vec())
}

/// Hash INVOKE transaction (v3) using Poseidon hash
pub fn hash_invoke_v3(tx: &InvokeTxV3) -> Result<Vec<u8>> {
    // Build elements to hash
    let mut elements = Vec::new();
    elements.push(prefix_to_field(INVOKE_PREFIX)?);
    elements.push(tx.sender_address);
    elements.push(hash_calldata_poseidon(&tx.calldata)?);
    elements.push(hash_resource_bounds(&tx.resource_bounds)?);
    elements.push(FieldElement::from(tx.tip));
    elements.push(tx.nonce);
    elements.push(hash_da_modes(
        tx.nonce_data_availability_mode,
        tx.fee_data_availability_mode,
    )?);

    let hash = PoseidonHash::hash_many(&elements);

    Ok(hash.to_bytes_be().to_vec())
}

/// Hash DECLARE transaction (v0) using Pedersen hash
pub fn hash_declare_v0(tx: &DeclareTxV0) -> Result<Vec<u8>> {
    let mut hasher = PedersenHasher::new();

    hasher.update(prefix_to_field(DECLARE_PREFIX)?);
    hasher.update(tx.class_hash);
    hasher.update(tx.sender_address);
    hasher.update(tx.max_fee);

    let hash = hasher.finalize();

    Ok(hash.to_bytes_be().to_vec())
}

/// Hash DECLARE transaction (v3) using Poseidon hash
pub fn hash_declare_v3(tx: &DeclareTxV3) -> Result<Vec<u8>> {
    let mut elements = Vec::new();
    elements.push(prefix_to_field(DECLARE_PREFIX)?);
    elements.push(tx.class_hash);
    elements.push(tx.compiled_class_hash);
    elements.push(tx.sender_address);
    elements.push(hash_resource_bounds(&tx.resource_bounds)?);
    elements.push(FieldElement::from(tx.tip));
    elements.push(tx.nonce);
    elements.push(hash_da_modes(
        tx.nonce_data_availability_mode,
        tx.fee_data_availability_mode,
    )?);

    let hash = PoseidonHash::hash_many(&elements);

    Ok(hash.to_bytes_be().to_vec())
}

/// Hash DEPLOY_ACCOUNT transaction (v1) using Pedersen hash
pub fn hash_deploy_account_v1(tx: &DeployAccountTxV1) -> Result<Vec<u8>> {
    let mut hasher = PedersenHasher::new();

    hasher.update(prefix_to_field(DEPLOY_ACCOUNT_PREFIX)?);
    hasher.update(tx.class_hash);
    hasher.update(hash_calldata_pedersen(&tx.constructor_calldata)?);
    hasher.update(tx.contract_address_salt);
    hasher.update(tx.max_fee);
    hasher.update(tx.nonce);

    let hash = hasher.finalize();

    Ok(hash.to_bytes_be().to_vec())
}

/// Hash DEPLOY_ACCOUNT transaction (v3) using Poseidon hash
pub fn hash_deploy_account_v3(tx: &DeployAccountTxV3) -> Result<Vec<u8>> {
    let mut elements = Vec::new();
    elements.push(prefix_to_field(DEPLOY_ACCOUNT_PREFIX)?);
    elements.push(tx.class_hash);
    elements.push(hash_calldata_poseidon(&tx.constructor_calldata)?);
    elements.push(tx.contract_address_salt);
    elements.push(hash_resource_bounds(&tx.resource_bounds)?);
    elements.push(FieldElement::from(tx.tip));
    elements.push(tx.nonce);
    elements.push(hash_da_modes(
        tx.nonce_data_availability_mode,
        tx.fee_data_availability_mode,
    )?);

    let hash = PoseidonHash::hash_many(&elements);

    Ok(hash.to_bytes_be().to_vec())
}

// Helper functions

fn prefix_to_field(prefix: &str) -> Result<FieldElement> {
    // Convert ASCII string to field element
    let bytes = prefix.as_bytes();
    let mut buf = [0u8; 32];
    buf[32 - bytes.len()..].copy_from_slice(bytes);

    FieldElement::from_bytes_be(&buf)
        .map_err(|e| DecoderError::invalid_structure(format!("Invalid prefix: {:?}", e)))
}

fn hash_calldata_pedersen(calldata: &[FieldElement]) -> Result<FieldElement> {
    if calldata.is_empty() {
        return Ok(FieldElement::ZERO);
    }

    let mut hasher = PedersenHasher::new();
    for element in calldata {
        hasher.update(*element);
    }

    Ok(hasher.finalize())
}

fn hash_calldata_poseidon(calldata: &[FieldElement]) -> Result<FieldElement> {
    if calldata.is_empty() {
        return Ok(FieldElement::ZERO);
    }

    Ok(PoseidonHash::hash_many(calldata))
}

fn hash_resource_bounds(bounds: &ResourceBounds) -> Result<FieldElement> {
    let elements = vec![
        FieldElement::from(bounds.l1_gas.max_amount),
        FieldElement::from(bounds.l1_gas.max_price_per_unit),
        FieldElement::from(bounds.l2_gas.max_amount),
        FieldElement::from(bounds.l2_gas.max_price_per_unit),
    ];

    Ok(PoseidonHash::hash_many(&elements))
}

fn hash_da_modes(
    nonce_mode: DataAvailabilityMode,
    fee_mode: DataAvailabilityMode,
) -> Result<FieldElement> {
    // Encode DA modes as single field element
    let nonce_val = match nonce_mode {
        DataAvailabilityMode::L1 => 0u64,
        DataAvailabilityMode::L2 => 1u64,
    };

    let fee_val = match fee_mode {
        DataAvailabilityMode::L1 => 0u64,
        DataAvailabilityMode::L2 => 1u64,
    };

    // Combine: (nonce_mode << 1) | fee_mode
    let combined = (nonce_val << 1) | fee_val;

    Ok(FieldElement::from(combined))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_to_field() {
        let field = prefix_to_field(INVOKE_PREFIX).unwrap();
        assert_ne!(field, FieldElement::ZERO);
    }

    #[test]
    fn test_hash_calldata_empty() {
        let hash = hash_calldata(&[]).unwrap();
        assert_eq!(hash, FieldElement::ZERO);
    }

    #[test]
    fn test_hash_calldata_single() {
        let calldata = vec![FieldElement::from(123u64)];
        let hash = hash_calldata(&calldata).unwrap();
        assert_ne!(hash, FieldElement::ZERO);
    }

    #[test]
    fn test_hash_da_modes() {
        let hash = hash_da_modes(DataAvailabilityMode::L1, DataAvailabilityMode::L2).unwrap();
        assert_ne!(hash, FieldElement::ZERO);
    }

    #[test]
    fn test_hash_invoke_v1() {
        let tx = InvokeTxV1 {
            sender_address: FieldElement::from(1u64),
            calldata: vec![FieldElement::from(2u64)],
            max_fee: FieldElement::from(1000u64),
            signature: vec![],
            nonce: FieldElement::from(0u64),
        };

        let hash = hash_invoke_v1(&tx).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_invoke_v3() {
        let tx = InvokeTxV3 {
            sender_address: FieldElement::from(1u64),
            calldata: vec![FieldElement::from(2u64)],
            signature: vec![],
            nonce: FieldElement::from(0u64),
            resource_bounds: ResourceBounds {
                l1_gas: ResourceBound {
                    max_amount: 1000,
                    max_price_per_unit: 100,
                },
                l2_gas: ResourceBound {
                    max_amount: 2000,
                    max_price_per_unit: 50,
                },
            },
            tip: 10,
            paymaster_data: vec![],
            account_deployment_data: vec![],
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
        };

        let hash = hash_invoke_v3(&tx).unwrap();
        assert_eq!(hash.len(), 32);
    }
}
