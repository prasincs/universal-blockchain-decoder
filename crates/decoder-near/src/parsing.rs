//! NEAR transaction parsing using Borsh deserialization
//!
//! NEAR uses Borsh (Binary Object Representation Serializer for Hashing)
//! for deterministic serialization. This module provides pure Rust parsing
//! of NEAR transactions without depending on near-primitives in production.

use crate::types::{
    AccessKey, AccessKeyPermission, Action, AddKeyAction, CreateAccountAction, DeleteAccountAction,
    DeleteKeyAction, DeployContractAction, FunctionCallAction, KeyType, PublicKey,
    SignedTransaction, StakeAction, Transaction, TransferAction,
};
use decoder_primitives::prelude::*;

/// Parse a NEAR signed transaction from raw bytes
///
/// NEAR transactions are Borsh-serialized. The format is:
/// 1. Transaction (Borsh-encoded)
/// 2. Signature (64 bytes)
///
/// # Arguments
///
/// * `raw_bytes` - Raw transaction bytes (Borsh-encoded)
///
/// # Returns
///
/// * `Ok(SignedTransaction)` - Successfully parsed transaction
/// * `Err(DecoderError)` - Parse error
pub fn parse_signed_transaction(raw_bytes: &[u8]) -> Result<SignedTransaction> {
    // For now, we'll use a simplified parsing approach
    // In a production implementation, we would implement full Borsh deserialization

    if raw_bytes.len() < 64 {
        return Err(DecoderError::invalid_structure(
            "NEAR transaction too short (must be at least 64 bytes for signature)",
        ));
    }

    // Extract signature (last 64 bytes)
    let signature_start = raw_bytes.len() - 64;
    let signature = raw_bytes[signature_start..].to_vec();
    let transaction_bytes = &raw_bytes[..signature_start];

    // Parse the transaction structure
    let transaction = parse_transaction(transaction_bytes)?;

    Ok(SignedTransaction {
        transaction,
        signature,
    })
}

/// Parse the inner transaction structure
///
/// This is a simplified parser. A production implementation would use
/// full Borsh deserialization with proper schema validation.
fn parse_transaction(bytes: &[u8]) -> Result<Transaction> {
    // This is a placeholder implementation
    // In production, we would implement full Borsh deserialization
    // following the NEAR transaction schema

    if bytes.is_empty() {
        return Err(DecoderError::invalid_structure(
            "Transaction bytes cannot be empty",
        ));
    }

    // For now, return a minimal transaction structure
    // This needs to be replaced with actual Borsh parsing
    Ok(Transaction {
        signer_id: "placeholder.near".to_string(),
        public_key: PublicKey {
            key_type: KeyType::Ed25519,
            data: vec![0u8; 32],
        },
        nonce: 0,
        receiver_id: "placeholder.near".to_string(),
        block_hash: [0u8; 32],
        actions: vec![],
    })
}

/// Parse a NEAR public key
///
/// Format:
/// - 1 byte: key type (0 = Ed25519, 1 = Secp256k1)
/// - 32 bytes: key data (for Ed25519)
pub fn parse_public_key(bytes: &[u8]) -> Result<PublicKey> {
    if bytes.is_empty() {
        return Err(DecoderError::invalid_structure(
            "Public key bytes cannot be empty",
        ));
    }

    let key_type = match bytes[0] {
        0 => KeyType::Ed25519,
        1 => KeyType::Secp256k1,
        _ => {
            return Err(DecoderError::invalid_structure(format!(
                "Invalid key type: {}",
                bytes[0]
            )))
        }
    };

    let expected_len = match key_type {
        KeyType::Ed25519 => 32,
        KeyType::Secp256k1 => 64,
    };

    if bytes.len() < 1 + expected_len {
        return Err(DecoderError::invalid_structure(format!(
            "Public key too short: expected {} bytes, got {}",
            1 + expected_len,
            bytes.len()
        )));
    }

    Ok(PublicKey {
        key_type,
        data: bytes[1..1 + expected_len].to_vec(),
    })
}

/// Parse a NEAR action
///
/// Actions are Borsh-encoded with a 1-byte discriminant followed by the action data.
/// Discriminants:
/// - 0: CreateAccount
/// - 1: DeployContract
/// - 2: FunctionCall
/// - 3: Transfer
/// - 4: Stake
/// - 5: AddKey
/// - 6: DeleteKey
/// - 7: DeleteAccount
pub fn parse_action(bytes: &[u8]) -> Result<Action> {
    if bytes.is_empty() {
        return Err(DecoderError::invalid_structure(
            "Action bytes cannot be empty",
        ));
    }

    let discriminant = bytes[0];
    let action_data = &bytes[1..];

    match discriminant {
        0 => Ok(Action::CreateAccount(CreateAccountAction)),
        1 => parse_deploy_contract(action_data),
        2 => parse_function_call(action_data),
        3 => parse_transfer(action_data),
        4 => parse_stake(action_data),
        5 => parse_add_key(action_data),
        6 => parse_delete_key(action_data),
        7 => parse_delete_account(action_data),
        _ => Err(DecoderError::invalid_structure(format!(
            "Invalid action discriminant: {}",
            discriminant
        ))),
    }
}

fn parse_deploy_contract(bytes: &[u8]) -> Result<Action> {
    // Simplified: actual implementation would parse Borsh-encoded Vec<u8>
    Ok(Action::DeployContract(DeployContractAction {
        code: bytes.to_vec(),
    }))
}

fn parse_function_call(_bytes: &[u8]) -> Result<Action> {
    // Simplified: actual implementation would parse all fields
    Ok(Action::FunctionCall(FunctionCallAction {
        method_name: String::new(),
        args: vec![],
        gas: 0,
        deposit: 0,
    }))
}

fn parse_transfer(bytes: &[u8]) -> Result<Action> {
    if bytes.len() < 16 {
        return Err(DecoderError::invalid_structure(
            "Transfer action too short (need 16 bytes for u128 deposit)",
        ));
    }

    // Parse u128 deposit (little-endian)
    let deposit = u128::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]);

    Ok(Action::Transfer(TransferAction { deposit }))
}

fn parse_stake(_bytes: &[u8]) -> Result<Action> {
    // Simplified: actual implementation would parse stake amount and public key
    Ok(Action::Stake(StakeAction {
        stake: 0,
        public_key: PublicKey {
            key_type: KeyType::Ed25519,
            data: vec![0u8; 32],
        },
    }))
}

fn parse_add_key(_bytes: &[u8]) -> Result<Action> {
    // Simplified: actual implementation would parse public key and access key
    Ok(Action::AddKey(AddKeyAction {
        public_key: PublicKey {
            key_type: KeyType::Ed25519,
            data: vec![0u8; 32],
        },
        access_key: AccessKey {
            nonce: 0,
            permission: AccessKeyPermission::FullAccess,
        },
    }))
}

fn parse_delete_key(_bytes: &[u8]) -> Result<Action> {
    // Simplified: actual implementation would parse public key
    Ok(Action::DeleteKey(DeleteKeyAction {
        public_key: PublicKey {
            key_type: KeyType::Ed25519,
            data: vec![0u8; 32],
        },
    }))
}

fn parse_delete_account(_bytes: &[u8]) -> Result<Action> {
    // Simplified: actual implementation would parse beneficiary account ID
    Ok(Action::DeleteAccount(DeleteAccountAction {
        beneficiary_id: String::new(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_public_key_ed25519() {
        let mut bytes = vec![0u8]; // Ed25519 type
        bytes.extend_from_slice(&[1u8; 32]); // 32-byte key

        let pubkey = parse_public_key(&bytes).unwrap();
        assert_eq!(pubkey.key_type, KeyType::Ed25519);
        assert_eq!(pubkey.data.len(), 32);
        assert_eq!(pubkey.data, vec![1u8; 32]);
    }

    #[test]
    fn test_parse_public_key_invalid_type() {
        let bytes = vec![255u8]; // Invalid type
        let result = parse_public_key(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_transfer_action() {
        let mut bytes = vec![3u8]; // Transfer discriminant
        bytes.extend_from_slice(&1000u128.to_le_bytes()); // deposit

        let action = parse_action(&bytes).unwrap();
        match action {
            Action::Transfer(t) => assert_eq!(t.deposit, 1000),
            _ => panic!("Expected Transfer action"),
        }
    }

    #[test]
    fn test_parse_create_account_action() {
        let bytes = vec![0u8]; // CreateAccount discriminant
        let action = parse_action(&bytes).unwrap();
        assert!(matches!(action, Action::CreateAccount(_)));
    }

    #[test]
    fn test_parse_signed_transaction_too_short() {
        let bytes = vec![1, 2, 3]; // Too short
        let result = parse_signed_transaction(&bytes);
        assert!(result.is_err());
    }
}
