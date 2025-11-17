//! Integration tests for Starknet decoder
//!
//! These tests validate the full decoding pipeline with realistic transaction data.

use decoder_crypto_zk::FieldElement;
use decoder_primitives::prelude::*;
use decoder_starknet::*;

#[test]
fn test_decode_invoke_v1_minimal() {
    // Create minimal INVOKE v1 transaction
    let mut tx_bytes = Vec::new();

    // Version: 1
    tx_bytes.push(1);
    // Type: INVOKE (0)
    tx_bytes.push(0);

    // Sender address (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Calldata length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Max fee (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Signature length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Nonce (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Failed to decode: {:?}", result.err());

    let tx = result.unwrap();
    assert_eq!(tx.tx_type(), StarknetTxType::Invoke);
    assert_eq!(tx.version(), StarknetVersion::V1);
}

#[test]
fn test_decode_invoke_v3_minimal() {
    // Create minimal INVOKE v3 transaction
    let mut tx_bytes = Vec::new();

    // Version: 3
    tx_bytes.push(3);
    // Type: INVOKE (0)
    tx_bytes.push(0);

    // Sender address (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Calldata length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Signature length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Nonce (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Resource bounds (L1 gas)
    tx_bytes.extend_from_slice(&1000u64.to_be_bytes()); // max_amount
    tx_bytes.extend_from_slice(&100u128.to_be_bytes()); // max_price_per_unit

    // Resource bounds (L2 gas)
    tx_bytes.extend_from_slice(&2000u64.to_be_bytes()); // max_amount
    tx_bytes.extend_from_slice(&50u128.to_be_bytes()); // max_price_per_unit

    // Tip
    tx_bytes.extend_from_slice(&10u64.to_be_bytes());

    // Paymaster data length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Account deployment data length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Nonce DA mode: L1 (0)
    tx_bytes.push(0);

    // Fee DA mode: L1 (0)
    tx_bytes.push(0);

    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Failed to decode: {:?}", result.err());

    let tx = result.unwrap();
    assert_eq!(tx.tx_type(), StarknetTxType::Invoke);
    assert_eq!(tx.version(), StarknetVersion::V3);
}

#[test]
fn test_decode_declare_v0_minimal() {
    // Create minimal DECLARE v0 transaction
    let mut tx_bytes = Vec::new();

    // Version: 0
    tx_bytes.push(0);
    // Type: DECLARE (1)
    tx_bytes.push(1);

    // Class hash (32 bytes)
    tx_bytes.extend_from_slice(&[1u8; 32]);

    // Sender address (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Max fee (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Signature length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Failed to decode: {:?}", result.err());

    let tx = result.unwrap();
    assert_eq!(tx.tx_type(), StarknetTxType::Declare);
    assert_eq!(tx.version(), StarknetVersion::V0);
}

#[test]
fn test_decode_deploy_account_v1_minimal() {
    // Create minimal DEPLOY_ACCOUNT v1 transaction
    let mut tx_bytes = Vec::new();

    // Version: 1
    tx_bytes.push(1);
    // Type: DEPLOY_ACCOUNT (2)
    tx_bytes.push(2);

    // Class hash (32 bytes)
    tx_bytes.extend_from_slice(&[1u8; 32]);

    // Constructor calldata length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Contract address salt (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Max fee (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Signature length: 0
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Nonce (32 bytes)
    tx_bytes.extend_from_slice(&[0u8; 32]);

    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Failed to decode: {:?}", result.err());

    let tx = result.unwrap();
    assert_eq!(tx.tx_type(), StarknetTxType::DeployAccount);
    assert_eq!(tx.version(), StarknetVersion::V1);
}

#[test]
fn test_decode_invoke_v1_with_calldata() {
    // INVOKE v1 with non-empty calldata
    let mut tx_bytes = Vec::new();

    tx_bytes.push(1); // version
    tx_bytes.push(0); // type: INVOKE

    // Sender address
    let sender = FieldElement::from(12345u64);
    tx_bytes.extend_from_slice(&sender.to_bytes_be());

    // Calldata: [1, 2, 3]
    tx_bytes.extend_from_slice(&3u64.to_be_bytes());
    tx_bytes.extend_from_slice(&FieldElement::from(1u64).to_bytes_be());
    tx_bytes.extend_from_slice(&FieldElement::from(2u64).to_bytes_be());
    tx_bytes.extend_from_slice(&FieldElement::from(3u64).to_bytes_be());

    // Max fee
    tx_bytes.extend_from_slice(&FieldElement::from(1000000u64).to_bytes_be());

    // Signature: empty
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());

    // Nonce
    tx_bytes.extend_from_slice(&FieldElement::from(5u64).to_bytes_be());

    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_ok());

    let tx = result.unwrap();
    match tx.variant {
        StarknetTxVariant::InvokeV1(invoke_tx) => {
            assert_eq!(invoke_tx.sender_address, sender);
            assert_eq!(invoke_tx.calldata.len(), 3);
            assert_eq!(invoke_tx.nonce, FieldElement::from(5u64));
        }
        _ => panic!("Expected InvokeV1"),
    }
}

#[test]
fn test_hash_verification() {
    // Create transaction and verify hash
    let mut tx_bytes = Vec::new();

    tx_bytes.push(1); // version
    tx_bytes.push(0); // type: INVOKE
    tx_bytes.extend_from_slice(&[0u8; 32]); // sender
    tx_bytes.extend_from_slice(&0u64.to_be_bytes()); // calldata length
    tx_bytes.extend_from_slice(&[0u8; 32]); // max_fee
    tx_bytes.extend_from_slice(&0u64.to_be_bytes()); // signature length
    tx_bytes.extend_from_slice(&[0u8; 32]); // nonce

    let tx = StarknetDecoder::decode(&tx_bytes).unwrap();

    // Hash should be 32 bytes
    assert_eq!(tx.tx_hash.len(), 32);

    // Verify hash matches
    assert!(tx.verify_hash().unwrap());
}

#[test]
fn test_canonicalize_invoke_v1() {
    // Create and canonicalize transaction
    let mut tx_bytes = Vec::new();

    tx_bytes.push(1);
    tx_bytes.push(0);
    tx_bytes.extend_from_slice(&[0u8; 32]);
    tx_bytes.extend_from_slice(&0u64.to_be_bytes());
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Add signature (2 field elements)
    tx_bytes.extend_from_slice(&2u64.to_be_bytes());
    tx_bytes.extend_from_slice(&FieldElement::from(111u64).to_bytes_be());
    tx_bytes.extend_from_slice(&FieldElement::from(222u64).to_bytes_be());

    tx_bytes.extend_from_slice(&[0u8; 32]);

    let tx = StarknetDecoder::decode(&tx_bytes).unwrap();

    // Should canonicalize without error
    let result = tx.canonicalize();
    assert!(result.is_ok());

    let tx_ir = result.unwrap();
    // Verify TxIR has operations
    assert!(!tx_ir.operations.is_empty());
    assert!(!tx_ir.authorization.signatures.is_empty());
}

#[test]
fn test_invalid_version() {
    let tx_bytes = vec![99, 0]; // invalid version
    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_err());
}

#[test]
fn test_invalid_tx_type() {
    let tx_bytes = vec![1, 99]; // invalid tx type
    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_err());
}

#[test]
fn test_empty_transaction() {
    let tx_bytes = vec![];
    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_err());
}

#[test]
fn test_truncated_transaction() {
    let tx_bytes = vec![1]; // only version, no type
    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_err());
}

#[test]
fn test_array_size_limit() {
    // Test DOS protection: arrays limited to 10000 elements
    let mut tx_bytes = Vec::new();

    tx_bytes.push(1); // version
    tx_bytes.push(0); // type: INVOKE
    tx_bytes.extend_from_slice(&[0u8; 32]); // sender

    // Calldata with 10001 elements (exceeds limit)
    tx_bytes.extend_from_slice(&10001u64.to_be_bytes());

    let result = StarknetDecoder::decode(&tx_bytes);
    assert!(result.is_err());

    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("too large"));
}

#[test]
fn test_chain_registry() {
    let registry = StarknetRegistry::new();

    // Test mainnet
    let mainnet = registry.get(23448594291968336);
    assert!(mainnet.is_some());
    assert_eq!(mainnet.unwrap().name, "Starknet Mainnet");

    // Test sepolia
    let sepolia = registry.get(23448594291968337);
    assert!(sepolia.is_some());
    assert_eq!(sepolia.unwrap().name, "Starknet Sepolia");
}

#[test]
fn test_resource_bounds_parsing() {
    // Test v3 transaction with specific resource bounds
    let mut tx_bytes = Vec::new();

    tx_bytes.push(3);
    tx_bytes.push(0);
    tx_bytes.extend_from_slice(&[0u8; 32]); // sender
    tx_bytes.extend_from_slice(&0u64.to_be_bytes()); // calldata
    tx_bytes.extend_from_slice(&0u64.to_be_bytes()); // signature
    tx_bytes.extend_from_slice(&[0u8; 32]); // nonce

    // L1 gas bounds
    tx_bytes.extend_from_slice(&5000u64.to_be_bytes());
    tx_bytes.extend_from_slice(&200u128.to_be_bytes());

    // L2 gas bounds
    tx_bytes.extend_from_slice(&10000u64.to_be_bytes());
    tx_bytes.extend_from_slice(&100u128.to_be_bytes());

    tx_bytes.extend_from_slice(&50u64.to_be_bytes()); // tip
    tx_bytes.extend_from_slice(&0u64.to_be_bytes()); // paymaster
    tx_bytes.extend_from_slice(&0u64.to_be_bytes()); // account deployment
    tx_bytes.push(0); // nonce DA mode
    tx_bytes.push(1); // fee DA mode (L2)

    let tx = StarknetDecoder::decode(&tx_bytes).unwrap();

    match tx.variant {
        StarknetTxVariant::InvokeV3(invoke_tx) => {
            assert_eq!(invoke_tx.resource_bounds.l1_gas.max_amount, 5000);
            assert_eq!(invoke_tx.resource_bounds.l1_gas.max_price_per_unit, 200);
            assert_eq!(invoke_tx.resource_bounds.l2_gas.max_amount, 10000);
            assert_eq!(invoke_tx.resource_bounds.l2_gas.max_price_per_unit, 100);
            assert_eq!(invoke_tx.tip, 50);
            assert_eq!(
                invoke_tx.nonce_data_availability_mode,
                DataAvailabilityMode::L1
            );
            assert_eq!(
                invoke_tx.fee_data_availability_mode,
                DataAvailabilityMode::L2
            );
        }
        _ => panic!("Expected InvokeV3"),
    }
}

#[test]
fn test_data_availability_modes() {
    // Test all 4 combinations of DA modes
    let combinations = [
        (DataAvailabilityMode::L1, DataAvailabilityMode::L1),
        (DataAvailabilityMode::L1, DataAvailabilityMode::L2),
        (DataAvailabilityMode::L2, DataAvailabilityMode::L1),
        (DataAvailabilityMode::L2, DataAvailabilityMode::L2),
    ];

    for (nonce_mode, fee_mode) in combinations.iter() {
        let mut tx_bytes = Vec::new();

        tx_bytes.push(3);
        tx_bytes.push(0);
        tx_bytes.extend_from_slice(&[0u8; 32]);
        tx_bytes.extend_from_slice(&0u64.to_be_bytes());
        tx_bytes.extend_from_slice(&0u64.to_be_bytes());
        tx_bytes.extend_from_slice(&[0u8; 32]);

        // Resource bounds
        tx_bytes.extend_from_slice(&1000u64.to_be_bytes());
        tx_bytes.extend_from_slice(&100u128.to_be_bytes());
        tx_bytes.extend_from_slice(&2000u64.to_be_bytes());
        tx_bytes.extend_from_slice(&50u128.to_be_bytes());

        tx_bytes.extend_from_slice(&10u64.to_be_bytes());
        tx_bytes.extend_from_slice(&0u64.to_be_bytes());
        tx_bytes.extend_from_slice(&0u64.to_be_bytes());

        tx_bytes.push(match nonce_mode {
            DataAvailabilityMode::L1 => 0,
            DataAvailabilityMode::L2 => 1,
        });
        tx_bytes.push(match fee_mode {
            DataAvailabilityMode::L1 => 0,
            DataAvailabilityMode::L2 => 1,
        });

        let tx = StarknetDecoder::decode(&tx_bytes).unwrap();

        match tx.variant {
            StarknetTxVariant::InvokeV3(invoke_tx) => {
                assert_eq!(&invoke_tx.nonce_data_availability_mode, nonce_mode);
                assert_eq!(&invoke_tx.fee_data_availability_mode, fee_mode);
            }
            _ => panic!("Expected InvokeV3"),
        }
    }
}
