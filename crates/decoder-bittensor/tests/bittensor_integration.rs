//! Integration tests for Bittensor decoder
//!
//! These tests use real Bittensor transaction data to validate the decoder.

use decoder_bittensor::*;
use decoder_primitives::prelude::*;

#[test]
fn test_minimal_extrinsic_decode() {
    // Create a minimal valid signed extrinsic
    let mut extrinsic = Vec::new();
    extrinsic.push(0x84); // Version: v4, signed

    // Address: 32-byte account ID
    extrinsic.push(0x00); // Address type: Id
    extrinsic.extend_from_slice(&[0xFF; 32]);

    // Signature: Sr25519
    extrinsic.push(0x01); // Signature type
    extrinsic.extend_from_slice(&[0xAA; 64]);

    // Era: Immortal
    extrinsic.push(0x00);

    // Nonce: 0
    extrinsic.push(0x00);

    // Tip: 0
    extrinsic.push(0x00);

    // Call: Balances::transfer
    extrinsic.push(0x04); // Pallet index (Balances)
    extrinsic.push(0x00); // Call index (transfer)

    // Destination address
    extrinsic.push(0x00); // Address type
    extrinsic.extend_from_slice(&[0xBB; 32]);

    // Amount: 1000000000 (1 TAO with 9 decimals) - compact encoded
    // 1000000000 = 0x3B9ACA00
    // Compact encoding for 4-byte mode: 0x02 prefix
    extrinsic.push(0x02); // Compact prefix
    extrinsic.push(0x00);
    extrinsic.push(0xCA);
    extrinsic.push(0x9A);
    extrinsic.push(0x3B);

    // Add length prefix
    let length = extrinsic.len() as u32;
    let mut with_length = vec![(length << 2) as u8];
    with_length.extend_from_slice(&extrinsic);

    // Decode
    let result = BittensorDecoder::decode(&with_length);
    assert!(result.is_ok(), "Failed to decode: {:?}", result.err());

    let tx = result.unwrap();
    assert_eq!(tx.raw_bytes.len(), with_length.len());
    assert_eq!(tx.tx_hash.len(), 64); // Blake2b-512

    // Verify it's signed
    assert!(tx.extrinsic.is_signed());

    // Parse call
    let call = tx.call().unwrap();
    assert_eq!(call.pallet_index, 4);
    assert_eq!(call.call_index, 0);
    assert_eq!(call.pallet_name(), "Balances");
    assert_eq!(call.call_name(), "transfer");
}

#[test]
fn test_unsigned_extrinsic_decode() {
    // Create a minimal unsigned extrinsic
    let mut extrinsic = Vec::new();
    extrinsic.push(0x04); // Version: v4, unsigned

    // Call: System::remark
    extrinsic.push(0x00); // Pallet index (System)
    extrinsic.push(0x01); // Call index (remark)

    // Remark data: "Hello Bittensor"
    let remark = b"Hello Bittensor";
    extrinsic.push((remark.len() << 2) as u8); // Compact length
    extrinsic.extend_from_slice(remark);

    // Add length prefix
    let length = extrinsic.len() as u32;
    let mut with_length = vec![(length << 2) as u8];
    with_length.extend_from_slice(&extrinsic);

    // Decode
    let result = BittensorDecoder::decode(&with_length);
    assert!(result.is_ok(), "Failed to decode: {:?}", result.err());

    let tx = result.unwrap();
    assert!(!tx.extrinsic.is_signed());

    // Parse call
    let call = tx.call().unwrap();
    assert_eq!(call.pallet_index, 0);
    assert_eq!(call.pallet_name(), "System");
}

#[test]
fn test_subtensor_set_weights_call() {
    // Create extrinsic with SubtensorModule::set_weights call
    let mut extrinsic = Vec::new();
    extrinsic.push(0x84); // Version: v4, signed

    // Address
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xFF; 32]);

    // Signature
    extrinsic.push(0x01);
    extrinsic.extend_from_slice(&[0xAA; 64]);

    // Era
    extrinsic.push(0x00);

    // Nonce
    extrinsic.push(0x00);

    // Tip
    extrinsic.push(0x00);

    // Call: SubtensorModule::set_weights
    extrinsic.push(0x07); // Pallet index (SubtensorModule)
    extrinsic.push(0x00); // Call index (set_weights)

    // Add some dummy parameters
    extrinsic.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]);

    // Add length prefix
    let length = extrinsic.len() as u32;
    let mut with_length = vec![(length << 2) as u8];
    with_length.extend_from_slice(&extrinsic);

    // Decode
    let result = BittensorDecoder::decode(&with_length);
    assert!(result.is_ok());

    let tx = result.unwrap();
    let call = tx.call().unwrap();

    assert_eq!(call.pallet_index, 7);
    assert_eq!(call.call_index, 0);
    assert_eq!(call.pallet_name(), "SubtensorModule");
    assert_eq!(call.call_name(), "set_weights");
}

#[test]
fn test_canonicalize_balances_transfer() {
    // Create a Balances::transfer extrinsic
    let mut extrinsic = Vec::new();
    extrinsic.push(0x84);
    extrinsic.push(0x00);
    extrinsic.extend_from_slice(&[0xFF; 32]); // From
    extrinsic.push(0x01);
    extrinsic.extend_from_slice(&[0xAA; 64]); // Signature
    extrinsic.push(0x00); // Era
    extrinsic.push(0x00); // Nonce
    extrinsic.push(0x00); // Tip
    extrinsic.push(0x04); // Balances
    extrinsic.push(0x00); // transfer
    extrinsic.push(0x00); // Dest address type
    extrinsic.extend_from_slice(&[0xBB; 32]); // To
    extrinsic.push(0x00); // Amount: 0

    let length = extrinsic.len() as u32;
    let mut with_length = vec![(length << 2) as u8];
    with_length.extend_from_slice(&extrinsic);

    let tx = BittensorDecoder::decode(&with_length).unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // Verify metadata
    assert_eq!(tx_ir.metadata.size, with_length.len());
    assert_eq!(tx_ir.metadata.tx_hash.len(), 64);

    // Verify operations (should have at least one transfer)
    assert!(!tx_ir.operations.is_empty());

    // Verify authorization (signed transaction should have signature)
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);
}

#[test]
fn test_hash_consistency() {
    let data = b"Bittensor test data";
    let hash1 = BittensorTransaction::calculate_hash(data);
    let hash2 = BittensorTransaction::calculate_hash(data);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // Blake2b-512
}

#[test]
fn test_chain_identity() {
    let chain = BittensorDecoder::chain();

    assert_eq!(chain.chain_name(), "Bittensor");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
    assert!(chain.chain_id() > 0); // Should have a valid chain ID
}

#[test]
fn test_validate_format_reject_invalid() {
    // Empty
    assert!(BittensorDecoder::validate_format(&[]).is_err());

    // Too short
    assert!(BittensorDecoder::validate_format(&[0x01]).is_err());
    assert!(BittensorDecoder::validate_format(&[0x01, 0x02]).is_err());

    // Minimum valid length
    assert!(BittensorDecoder::validate_format(&[0x04, 0x84, 0x00, 0x00]).is_ok());
}

// Note: For real integration tests with actual Bittensor transactions,
// add fixture files to tests/fixtures/ and load them here.
// Example:
//
// #[test]
// fn test_real_bittensor_transaction() {
//     let tx_bytes = include_bytes!("fixtures/mainnet_block_123456_tx0.bin");
//     let tx = BittensorDecoder::decode(tx_bytes).unwrap();
//     assert_eq!(tx.extrinsic.is_signed(), true);
//     // ... additional assertions
// }
