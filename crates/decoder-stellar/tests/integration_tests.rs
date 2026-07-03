//! Integration tests for Stellar decoder
//!
//! These tests use manually crafted XDR data to test the decoder
//! against known good transaction structures.

use decoder_stellar::types::{
    DecoratedSignature, EnvelopeType, StellarAsset, StellarMemo, StellarOperation,
    StellarTransaction,
};
use decoder_stellar::StellarDecoder;
use universal_decoder_core::prelude::*;

/// Helper to create a minimal valid Stellar transaction envelope (XDR)
///
/// This creates a transaction with:
/// - Envelope type: Tx (2)
/// - Source account: 32 zero bytes
/// - Fee: 100 stroops
/// - Sequence: 1
/// - No time bounds
/// - No memo
/// - 1 payment operation
/// - 1 signature
fn create_minimal_tx_envelope() -> Vec<u8> {
    let mut xdr = Vec::new();

    // Envelope type: ENVELOPE_TYPE_TX = 2
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x02]);

    // Transaction V1
    // Source account: PublicKeyTypeEd25519(0) + 32-byte public key
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // KEY_TYPE_ED25519
    xdr.extend_from_slice(&[0x01; 32]); // Public key (all ones)

    // Fee: 100
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x64]);

    // Sequence number: 1
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);

    // Time bounds: Optional(0) = None
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    // Memo: MEMO_NONE(0)
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    // Operations: Array length = 1
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);

    // Operation 1:
    // - Source account: Optional(0) = None
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    // - Operation type: PAYMENT(1)
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
    // - Destination: PublicKeyTypeEd25519(0) + 32-byte key
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // KEY_TYPE_ED25519
    xdr.extend_from_slice(&[0x02; 32]); // Destination key (all twos)
                                        // - Asset: ASSET_TYPE_NATIVE(0)
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    // - Amount: 10000000 (1 XLM in stroops)
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x98, 0x96, 0x80]);

    // Ext (reserved): 0
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    // Signatures: Array length = 1
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);

    // Decorated signature:
    // - Hint: 4 bytes
    xdr.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
    // - Signature: Variable bytes (length + data + padding)
    xdr.extend_from_slice(&[0x00, 0x00, 0x00, 0x40]); // Length = 64
    xdr.extend_from_slice(&[0x00; 64]); // 64-byte signature

    xdr
}

#[test]
fn test_decode_minimal_transaction() {
    let xdr_bytes = create_minimal_tx_envelope();

    let result = StellarDecoder::decode(&xdr_bytes);
    assert!(result.is_ok(), "Failed to decode: {:?}", result.err());

    let tx = result.unwrap();
    assert_eq!(tx.fee, 100);
    assert_eq!(tx.sequence_number, 1);
    assert_eq!(tx.operations.len(), 1);
    assert_eq!(tx.signatures.len(), 1);
}

#[test]
fn test_decode_and_canonicalize() {
    let xdr_bytes = create_minimal_tx_envelope();

    let tx = StellarDecoder::decode(&xdr_bytes).unwrap();
    let tx_ir = tx.canonicalize();

    assert!(tx_ir.is_ok(), "Failed to canonicalize: {:?}", tx_ir.err());

    let ir = tx_ir.unwrap();
    assert_eq!(ir.operations.len(), 1);

    // Check that the operation is a transfer
    match &ir.operations[0] {
        Operation::Transfer(transfer) => {
            assert_eq!(transfer.amount.decimals, 7); // Stellar uses 7 decimals
            assert_eq!(transfer.amount.value, 10000000); // 1 XLM in stroops
            assert!(matches!(transfer.asset, AssetId::Native));
        }
        _ => panic!("Expected Transfer operation"),
    }
}

#[test]
fn test_native_asset_conversion() {
    let xdr_bytes = create_minimal_tx_envelope();
    let tx = StellarDecoder::decode(&xdr_bytes).unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // First operation should be a native XLM transfer
    match &tx_ir.operations[0] {
        Operation::Transfer(transfer) => {
            assert!(matches!(transfer.asset, AssetId::Native));
        }
        _ => panic!("Expected Transfer operation"),
    }
}

#[test]
fn test_transaction_metadata() {
    let xdr_bytes = create_minimal_tx_envelope();
    let tx = StellarDecoder::decode(&xdr_bytes).unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // Check metadata
    assert_eq!(tx_ir.metadata.size, xdr_bytes.len());
    assert!(!tx_ir.metadata.tx_hash.is_empty());

    // Parse extra metadata
    let extra: serde_json::Value = serde_json::from_str(&tx_ir.metadata.extra).unwrap();
    assert_eq!(extra["fee"], 100);
    assert_eq!(extra["sequence"], 1);
}

#[test]
fn test_authorization_package() {
    let xdr_bytes = create_minimal_tx_envelope();
    let tx = StellarDecoder::decode(&xdr_bytes).unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // Check authorization
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);
    assert!(matches!(
        tx_ir.authorization.signature_scheme,
        SignatureScheme::EdDsa
    ));

    // Check signature metadata (contains hint)
    let sig = &tx_ir.authorization.signatures[0];
    assert!(sig.metadata.is_some());
    assert!(sig.metadata.as_ref().unwrap().contains("aabbccdd")); // hex of hint
}

#[test]
fn test_state_deltas() {
    let xdr_bytes = create_minimal_tx_envelope();
    let tx = StellarDecoder::decode(&xdr_bytes).unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // Check state deltas
    assert_eq!(tx_ir.state_deltas.inputs.len(), 0); // Account-based, no inputs
    assert_eq!(tx_ir.state_deltas.outputs.len(), 0); // Account-based, no outputs
                                                     // account_changes was removed from TxIR (docs/CONCEPTS_REVIEW.md C1):
                                                     // effects are not byte-derivable and are no longer fabricated.
}

#[test]
fn test_validate_format() {
    // Empty bytes should fail
    assert!(StellarDecoder::validate_format(&[]).is_err());

    // Too short (< 4 bytes)
    assert!(StellarDecoder::validate_format(&[0x01, 0x02]).is_err());

    // Valid length should pass basic validation
    let valid = vec![0u8; 100];
    assert!(StellarDecoder::validate_format(&valid).is_ok());
}

#[test]
fn test_invalid_envelope_type() {
    let mut xdr = vec![0x00, 0x00, 0x00, 0xFF]; // Invalid envelope type
    xdr.extend_from_slice(&[0; 100]); // Padding

    let result = StellarDecoder::decode(&xdr);
    assert!(result.is_err());
}

#[test]
fn test_payment_operation() {
    let xdr_bytes = create_minimal_tx_envelope();
    let tx = StellarDecoder::decode(&xdr_bytes).unwrap();

    assert_eq!(tx.operations.len(), 1);

    match &tx.operations[0] {
        StellarOperation::Payment {
            destination,
            asset,
            amount,
        } => {
            assert_eq!(destination.len(), 32);
            assert!(matches!(asset, StellarAsset::Native));
            assert_eq!(*amount, 10000000); // 1 XLM in stroops
        }
        _ => panic!("Expected Payment operation"),
    }
}

#[test]
fn test_chain_identity() {
    let chain = StellarDecoder::chain();
    assert_eq!(chain.chain_id(), 144);
    assert_eq!(chain.chain_name(), "Stellar");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
}

#[test]
fn test_empty_operations_invalid() {
    let tx = StellarTransaction {
        source_account: vec![0; 32],
        fee: 100,
        sequence_number: 1,
        time_bounds: None,
        memo: StellarMemo::None,
        operations: vec![], // Empty!
        signatures: vec![DecoratedSignature {
            hint: [0; 4],
            signature: vec![0; 64],
        }],
        raw_bytes: vec![],
        envelope_type: EnvelopeType::Tx,
        network_id: None,
    };

    assert!(!tx.is_valid());
}

#[test]
fn test_too_many_operations_invalid() {
    let ops = vec![
        StellarOperation::Payment {
            destination: vec![0; 32],
            asset: StellarAsset::Native,
            amount: 100,
        };
        101
    ]; // 101 operations (max is 100)

    let tx = StellarTransaction {
        source_account: vec![0; 32],
        fee: 100,
        sequence_number: 1,
        time_bounds: None,
        memo: StellarMemo::None,
        operations: ops,
        signatures: vec![DecoratedSignature {
            hint: [0; 4],
            signature: vec![0; 64],
        }],
        raw_bytes: vec![],
        envelope_type: EnvelopeType::Tx,
        network_id: None,
    };

    assert!(!tx.is_valid());
}
