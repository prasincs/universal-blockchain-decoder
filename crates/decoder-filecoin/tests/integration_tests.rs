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

// TODO: Add tests with real Filecoin transaction fixtures
// These would be CBOR-encoded signed messages from the Filecoin mainnet
//
// Example test structure:
// #[test]
// fn test_decode_real_filecoin_transfer() {
//     // Real Filecoin transaction bytes (CBOR-encoded)
//     let tx_bytes = include_bytes!("fixtures/filecoin_transfer.cbor");
//
//     let tx = FilecoinDecoder::decode(tx_bytes).unwrap();
//     assert!(tx.message().is_transfer());
//
//     // Verify canonicalization
//     let tx_ir = tx.canonicalize().unwrap();
//     assert_eq!(tx_ir.chain.chain_name, "Filecoin");
//     assert!(matches!(tx_ir.operations[0], Operation::Transfer(_)));
// }
