/// Convert TRON contracts to universal Operations
use crate::hashing::address_to_hex;
use crate::types::*;
use decoder_primitives::prelude::*;
use prost::Message;

/// Parse TRON contracts into universal Operations
pub fn parse_operations(contracts: &[Contract]) -> Result<Vec<Operation>> {
    contracts
        .iter()
        .map(|contract| {
            parse_single_contract(contract).or_else(|e| {
                // For unknown contract types, create a generic operation
                let parameter_data = contract
                    .parameter
                    .as_ref()
                    .map(|p| p.value.clone())
                    .unwrap_or_default();

                Ok(Operation::Generic(GenericOperation {
                    op_type: format!("TronContract({})", contract.r#type),
                    data: parameter_data,
                    metadata: format!("error: {}", e),
                }))
            })
        })
        .collect()
}

/// Parse a single contract
fn parse_single_contract(contract: &Contract) -> Result<Operation> {
    let contract_type = ContractType::try_from(contract.r#type).map_err(|_| {
        DecoderError::invalid_structure(format!("Unknown contract type: {}", contract.r#type))
    })?;

    let parameter = contract
        .parameter
        .as_ref()
        .ok_or_else(|| DecoderError::invalid_structure("Contract missing parameter"))?;

    match contract_type {
        ContractType::TransferContract => parse_transfer_contract(&parameter.value),
        ContractType::TransferAssetContract => parse_transfer_asset_contract(&parameter.value),
        ContractType::TriggerSmartContract => parse_trigger_smart_contract(&parameter.value),
        ContractType::FreezeBalanceContract => parse_freeze_balance_contract(&parameter.value),
        ContractType::FreezeBalanceV2Contract => parse_freeze_balance_v2_contract(&parameter.value),
        ContractType::UnfreezeBalanceContract => parse_unfreeze_balance_contract(&parameter.value),
        _ => Err(DecoderError::invalid_structure(format!(
            "Contract type {:?} not yet implemented",
            contract_type
        ))),
    }
}

/// Parse TransferContract (TRX transfer)
fn parse_transfer_contract(data: &[u8]) -> Result<Operation> {
    let transfer = TransferContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to decode TransferContract: {}", e))
    })?;

    Ok(Operation::Transfer(Transfer {
        from: Address {
            bytes: transfer.owner_address.clone(),
            human_readable: Some(address_to_hex(&transfer.owner_address)),
        },
        to: Address {
            bytes: transfer.to_address.clone(),
            human_readable: Some(address_to_hex(&transfer.to_address)),
        },
        amount: Amount {
            value: transfer.amount as u128,
            decimals: 6, // TRX has 6 decimals (1 TRX = 1,000,000 sun)
        },
        asset: AssetId::Native,
    }))
}

/// Parse TransferAssetContract (TRC-10 token transfer)
fn parse_transfer_asset_contract(data: &[u8]) -> Result<Operation> {
    let transfer = TransferAssetContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to decode TransferAssetContract: {}", e))
    })?;

    let asset_name = String::from_utf8_lossy(&transfer.asset_name).to_string();

    Ok(Operation::Transfer(Transfer {
        from: Address {
            bytes: transfer.owner_address.clone(),
            human_readable: Some(address_to_hex(&transfer.owner_address)),
        },
        to: Address {
            bytes: transfer.to_address.clone(),
            human_readable: Some(address_to_hex(&transfer.to_address)),
        },
        amount: Amount {
            value: transfer.amount as u128,
            decimals: 0, // TRC-10 tokens typically have variable decimals
        },
        asset: AssetId::Custom(asset_name),
    }))
}

/// Parse TriggerSmartContract
fn parse_trigger_smart_contract(data: &[u8]) -> Result<Operation> {
    let trigger = TriggerSmartContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to decode TriggerSmartContract: {}", e))
    })?;

    // Extract function selector (first 4 bytes of data)
    let method = if trigger.data.len() >= 4 {
        trigger.data[0..4].to_vec()
    } else {
        vec![]
    };

    Ok(Operation::ContractCall(ContractCall {
        contract: Address {
            bytes: trigger.contract_address.clone(),
            human_readable: Some(address_to_hex(&trigger.contract_address)),
        },
        method,
        data: trigger.data.clone(),
        value: if trigger.call_value > 0 {
            Some(Amount {
                value: trigger.call_value as u128,
                decimals: 6,
            })
        } else {
            None
        },
        resource_limits: ResourceLimits {
            max_units: 0, // TRON uses energy/bandwidth, not set here
            unit_price: 0,
            resource_type: ResourceType::Custom(1), // TRON energy
        },
    }))
}

/// Parse FreezeBalanceContract
fn parse_freeze_balance_contract(data: &[u8]) -> Result<Operation> {
    let freeze = FreezeBalanceContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to decode FreezeBalanceContract: {}", e))
    })?;

    Ok(Operation::Stake(Stake {
        validator: if freeze.receiver_address.is_empty() {
            Address {
                bytes: freeze.owner_address.clone(),
                human_readable: Some(address_to_hex(&freeze.owner_address)),
            }
        } else {
            Address {
                bytes: freeze.receiver_address.clone(),
                human_readable: Some(address_to_hex(&freeze.receiver_address)),
            }
        },
        amount: Amount {
            value: freeze.frozen_balance as u128,
            decimals: 6,
        },
        operation_type: StakeOperationType::Delegate,
    }))
}

/// Parse FreezeBalanceV2Contract
fn parse_freeze_balance_v2_contract(data: &[u8]) -> Result<Operation> {
    let freeze = FreezeBalanceV2Contract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to decode FreezeBalanceV2Contract: {}", e))
    })?;

    Ok(Operation::Stake(Stake {
        validator: Address {
            bytes: freeze.owner_address.clone(),
            human_readable: Some(address_to_hex(&freeze.owner_address)),
        },
        amount: Amount {
            value: freeze.frozen_balance as u128,
            decimals: 6,
        },
        operation_type: StakeOperationType::Delegate,
    }))
}

/// Parse UnfreezeBalanceContract
fn parse_unfreeze_balance_contract(data: &[u8]) -> Result<Operation> {
    let unfreeze = UnfreezeBalanceContract::decode(data).map_err(|e| {
        DecoderError::invalid_structure(format!("Failed to decode UnfreezeBalanceContract: {}", e))
    })?;

    Ok(Operation::Stake(Stake {
        validator: if unfreeze.receiver_address.is_empty() {
            Address {
                bytes: unfreeze.owner_address.clone(),
                human_readable: Some(address_to_hex(&unfreeze.owner_address)),
            }
        } else {
            Address {
                bytes: unfreeze.receiver_address.clone(),
                human_readable: Some(address_to_hex(&unfreeze.receiver_address)),
            }
        },
        amount: Amount {
            value: 0, // Unfreeze amount not specified in contract
            decimals: 6,
        },
        operation_type: StakeOperationType::Undelegate,
    }))
}

/// Parse state deltas from contracts
pub fn parse_state_deltas(contracts: &[Contract]) -> Result<StateDeltas> {
    let mut account_changes = Vec::new();

    for contract in contracts {
        let contract_type = ContractType::try_from(contract.r#type).ok();
        let parameter = contract.parameter.as_ref();

        if let (Some(ContractType::TransferContract), Some(param)) = (contract_type, parameter) {
            if let Ok(transfer) = TransferContract::decode(param.value.as_slice()) {
                // From account: balance decrease
                account_changes.push(AccountChange {
                    address: Address {
                        bytes: transfer.owner_address.clone(),
                        human_readable: Some(address_to_hex(&transfer.owner_address)),
                    },
                    nonce: None,
                    balance_change: -(transfer.amount as i128),
                    storage_changes: vec![],
                });

                // To account: balance increase
                account_changes.push(AccountChange {
                    address: Address {
                        bytes: transfer.to_address.clone(),
                        human_readable: Some(address_to_hex(&transfer.to_address)),
                    },
                    nonce: None,
                    balance_change: transfer.amount as i128,
                    storage_changes: vec![],
                });
            }
        }
        // Other contract types can be added here
    }

    Ok(StateDeltas {
        inputs: vec![],
        outputs: vec![],
        account_changes,
    })
}
