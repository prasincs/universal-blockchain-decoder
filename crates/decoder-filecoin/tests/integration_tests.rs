//! Integration tests for Filecoin decoder
//!
//! These tests verify that the Filecoin decoder correctly parses real
//! Filecoin transactions and produces valid TxIR representations.

use decoder_filecoin::types::*;
use decoder_filecoin::FilecoinDecoder;
use universal_decoder_core::prelude::*;

#[test]
fn test_filecoin_chain_identity() {
    let chain = FilecoinDecoder::chain();
    assert_eq!(chain.chain_id(), 461);
    assert_eq!(chain.chain_name(), "Filecoin");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
}

#[test]
fn test_validate_empty_transaction() {
    let result = FilecoinDecoder::validate_format(&[]);
    assert!(result.is_err());
}

#[test]
fn test_validate_invalid_cbor_marker() {
    // Not a CBOR array marker
    let result = FilecoinDecoder::validate_format(&[0x01, 0x02, 0x03]);
    assert!(result.is_err());
}

#[test]
fn test_validate_valid_cbor_marker() {
    // Valid CBOR array marker (0x82 = array of 2 elements)
    let result = FilecoinDecoder::validate_format(&[0x82, 0x00, 0x00]);
    assert!(result.is_ok());
}

#[test]
fn test_address_protocol_parsing() {
    assert_eq!(AddressProtocol::from_byte(0).unwrap(), AddressProtocol::Id);
    assert_eq!(
        AddressProtocol::from_byte(1).unwrap(),
        AddressProtocol::Secp256k1
    );
    assert_eq!(
        AddressProtocol::from_byte(2).unwrap(),
        AddressProtocol::Actor
    );
    assert_eq!(AddressProtocol::from_byte(3).unwrap(), AddressProtocol::Bls);
}

#[test]
fn test_signature_type_parsing() {
    assert_eq!(
        SignatureType::from_byte(1).unwrap(),
        SignatureType::Secp256k1
    );
    assert_eq!(SignatureType::from_byte(2).unwrap(), SignatureType::Bls);
}

#[test]
fn test_filecoin_address_creation() {
    let addr = FilecoinAddress::new(AddressProtocol::Id, vec![0x01, 0x02]);
    assert_eq!(addr.protocol, AddressProtocol::Id);
    assert_eq!(addr.payload, vec![0x01, 0x02]);

    let bytes = addr.to_bytes();
    assert_eq!(bytes[0], 0); // Protocol byte
    assert_eq!(&bytes[1..], &[0x01, 0x02]); // Payload
}

#[test]
fn test_filecoin_message_is_transfer() {
    let addr = FilecoinAddress::new(AddressProtocol::Id, vec![0x01]);

    let transfer_msg = FilecoinMessage {
        version: 0,
        from: addr.clone(),
        to: addr.clone(),
        sequence: 0,
        value: vec![],
        gas_limit: 1000000,
        gas_fee_cap: vec![],
        gas_premium: vec![],
        method_num: 0, // Transfer
        params: vec![],
    };

    assert!(transfer_msg.is_transfer());

    let method_call_msg = FilecoinMessage {
        version: 0,
        from: addr.clone(),
        to: addr,
        sequence: 0,
        value: vec![],
        gas_limit: 1000000,
        gas_fee_cap: vec![],
        gas_premium: vec![],
        method_num: 1, // Actor method
        params: vec![],
    };

    assert!(!method_call_msg.is_transfer());
}

// Fixture tests with real CBOR-encoded transactions

#[test]
fn test_decode_simple_transfer_fixture() {
    // Load raw transaction bytes from fixture
    let tx_hex = include_str!("fixtures/fil_simple_transfer.hex");
    let tx_bytes = hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    // Decode transaction
    let tx = FilecoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Verify message fields
    assert_eq!(tx.message().version, 0);
    assert_eq!(tx.message().method_num, 0); // Transfer
    assert_eq!(tx.message().sequence, 0);
    assert!(tx.message().is_transfer());

    // Verify addresses
    assert_eq!(tx.message().from.protocol, AddressProtocol::Id);
    assert_eq!(tx.message().to.protocol, AddressProtocol::Id);

    // Verify signature
    assert_eq!(tx.signature().sig_type, SignatureType::Secp256k1);

    // Verify canonicalization
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize");
    assert_eq!(tx_ir.metadata.size, tx_bytes.len());

    // Verify operations
    assert!(!tx_ir.operations.is_empty());
    assert!(matches!(tx_ir.operations[0], Operation::Transfer(_)));

    // Verify hash calculation (Blake2b-256)
    let hash = tx.hash();
    assert_eq!(hash.len(), 32); // Blake2b-256 produces 32 bytes
}

#[test]
fn test_decode_actor_call_fixture() {
    // Load raw transaction bytes from fixture
    let tx_hex = include_str!("fixtures/fil_actor_call.hex");
    let tx_bytes = hex::decode(tx_hex.trim()).expect("Failed to decode hex fixture");

    // Decode transaction
    let tx = FilecoinDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Verify message fields
    assert_eq!(tx.message().version, 0);
    assert_eq!(tx.message().method_num, 1); // Actor method call
    assert_eq!(tx.message().sequence, 1);
    assert!(!tx.message().is_transfer()); // Not a simple transfer

    // Verify params are present
    assert!(!tx.message().params.is_empty());

    // Verify signature type
    assert_eq!(tx.signature().sig_type, SignatureType::Bls);

    // Verify canonicalization
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize");

    // Verify operations (should be ContractCall for actor methods)
    assert!(!tx_ir.operations.is_empty());
    assert!(matches!(tx_ir.operations[0], Operation::ContractCall(_)));

    // Verify hash calculation
    let hash = tx.hash();
    assert_eq!(hash.len(), 32); // Blake2b-256 produces 32 bytes
}

#[test]
fn test_fixture_validation() {
    // Test both fixtures pass validation
    let transfer_hex = include_str!("fixtures/fil_simple_transfer.hex");
    let transfer_bytes = hex::decode(transfer_hex.trim()).unwrap();
    assert!(FilecoinDecoder::validate_format(&transfer_bytes).is_ok());

    let actor_hex = include_str!("fixtures/fil_actor_call.hex");
    let actor_bytes = hex::decode(actor_hex.trim()).unwrap();
    assert!(FilecoinDecoder::validate_format(&actor_bytes).is_ok());
}

#[test]
fn test_fixture_hash_determinism() {
    // Ensure hash calculation is deterministic
    let tx_hex = include_str!("fixtures/fil_simple_transfer.hex");
    let tx_bytes = hex::decode(tx_hex.trim()).unwrap();

    let tx1 = FilecoinDecoder::decode(&tx_bytes).unwrap();
    let tx2 = FilecoinDecoder::decode(&tx_bytes).unwrap();

    let hash1 = tx1.hash();
    let hash2 = tx2.hash();

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 32); // Blake2b-256
}
