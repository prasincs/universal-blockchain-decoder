//! Integration tests for AO decoder
//!
//! Tests the full flow from raw ANS-104 bytes to TxIR canonicalization

use decoder_ao::AODecoder;
use decoder_primitives::prelude::*;

/// Test decoding a complete AO message with all fields
#[test]
fn test_full_ao_message_workflow() {
    // Construct a realistic ANS-104 message with all fields
    let mut bytes = Vec::new();

    // 1. Signature type (Solana = 4)
    bytes.extend_from_slice(&4u16.to_be_bytes());

    // 2. Signature (64 bytes for Solana Ed25519)
    let signature = [0xDE, 0xAD, 0xBE, 0xEF].repeat(16); // 64 bytes
    bytes.extend_from_slice(&signature);

    // 3. Owner/Public key (32 bytes for Solana)
    let owner = [0xCA, 0xFE, 0xBA, 0xBE].repeat(8); // 32 bytes
    bytes.extend_from_slice(&owner);

    // 4. Target present = 1
    bytes.push(1);
    let target = [0x12, 0x34].repeat(16); // 32 bytes
    bytes.extend_from_slice(&target);

    // 5. Anchor present = 1 (for replay protection)
    bytes.push(1);
    let anchor = [0xAB, 0xCD].repeat(16); // 32 bytes
    bytes.extend_from_slice(&anchor);

    // 6. Tags: 3 tags total
    bytes.extend_from_slice(&3u64.to_be_bytes());

    // Build tags
    let mut tag_bytes = Vec::new();

    // Tag 1: Action = "Eval"
    tag_bytes.push(6); // "Action" length
    tag_bytes.extend_from_slice(b"Action");
    tag_bytes.push(4); // "Eval" length
    tag_bytes.extend_from_slice(b"Eval");

    // Tag 2: Data-Protocol = "ao"
    tag_bytes.push(13); // "Data-Protocol" length
    tag_bytes.extend_from_slice(b"Data-Protocol");
    tag_bytes.push(2); // "ao" length
    tag_bytes.extend_from_slice(b"ao");

    // Tag 3: From = "user_123"
    tag_bytes.push(4); // "From" length
    tag_bytes.extend_from_slice(b"From");
    tag_bytes.push(8); // "user_123" length
    tag_bytes.extend_from_slice(b"user_123");

    bytes.extend_from_slice(&(tag_bytes.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&tag_bytes);

    // 7. Data payload (Lua code for AO process)
    let data = b"return { result = 'Hello from AO!' }";
    bytes.extend_from_slice(data);

    // Decode the message
    let tx = AODecoder::decode(&bytes).expect("Failed to decode AO message");

    // Verify parsed fields
    assert_eq!(
        tx.message.signature_type,
        decoder_ao::types::SignatureType::Solana
    );
    assert_eq!(tx.message.signature.len(), 64);
    assert_eq!(tx.message.owner.len(), 32);
    assert!(tx.message.target.is_some());
    assert_eq!(tx.message.target.as_ref().unwrap().len(), 32);
    assert!(tx.message.anchor.is_some());
    assert_eq!(tx.message.tags.len(), 3);
    assert_eq!(tx.message.data, data);

    // Verify tag access
    assert_eq!(tx.action(), Some("Eval"));
    assert_eq!(tx.message.get_tag("Data-Protocol"), Some("ao"));
    assert_eq!(tx.message.sender(), Some("user_123"));

    // Verify message ID is deterministic
    assert_eq!(tx.message_id.len(), 32); // SHA-256

    // Canonicalize to TxIR
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize");

    // Verify TxIR structure
    assert_eq!(tx_ir.chain.family(), ChainFamily::Actor);
    assert_eq!(tx_ir.chain.name, "AO");
    assert_eq!(tx_ir.metadata.tx_hash, tx.message_id);
    assert!(tx_ir.metadata.extra.contains("ao_message"));

    // Verify operations
    assert_eq!(tx_ir.operations.len(), 1);
    if let Operation::ContractCall(call) = &tx_ir.operations[0] {
        assert_eq!(call.method, b"Eval");
        assert_eq!(call.data, data);
    } else {
        panic!("Expected ContractCall operation");
    }

    // Verify authorization
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);
    assert_eq!(tx_ir.authorization.signature_scheme, SignatureScheme::EdDsa);

    // Verify state deltas
    assert_eq!(tx_ir.state_deltas.account_changes.len(), 1);
    assert_eq!(tx_ir.state_deltas.account_changes[0].address.bytes, target);
}

/// Test decoding minimal message (no optional fields)
#[test]
fn test_minimal_ao_message() {
    let mut bytes = Vec::new();

    // Ethereum signature type
    bytes.extend_from_slice(&3u16.to_be_bytes());

    // Signature (65 bytes for Ethereum ECDSA)
    bytes.extend_from_slice(&[0xFF; 65]);

    // Owner (65 bytes for Ethereum)
    bytes.extend_from_slice(&[0xEE; 65]);

    // No target
    bytes.push(0);

    // No anchor
    bytes.push(0);

    // No tags
    bytes.extend_from_slice(&0u64.to_be_bytes());
    bytes.extend_from_slice(&0u64.to_be_bytes());

    // Minimal data
    bytes.extend_from_slice(b"ping");

    let tx = AODecoder::decode(&bytes).expect("Failed to decode minimal message");

    assert_eq!(
        tx.message.signature_type,
        decoder_ao::types::SignatureType::Ethereum
    );
    assert!(tx.message.target.is_none());
    assert!(tx.message.anchor.is_none());
    assert_eq!(tx.message.tags.len(), 0);
    assert_eq!(tx.message.data, b"ping");

    // Should still canonicalize successfully
    let tx_ir = tx
        .canonicalize()
        .expect("Failed to canonicalize minimal message");
    assert_eq!(tx_ir.chain.family(), ChainFamily::Actor);

    // No operations without Action tag
    assert_eq!(tx_ir.operations.len(), 0);

    // No account changes without target
    assert_eq!(tx_ir.state_deltas.account_changes.len(), 0);
}

/// Test validation errors
#[test]
fn test_validation_errors() {
    // Empty message
    let result = AODecoder::decode(&[]);
    assert!(result.is_err());

    // Message too short
    let short_msg = vec![0u8; 50];
    let result = AODecoder::decode(&short_msg);
    assert!(result.is_err());

    // Invalid signature type
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&999u16.to_be_bytes()); // Unknown signature type
    bytes.extend_from_slice(&[0u8; 100]); // Padding
    let result = AODecoder::decode(&bytes);
    // Should fail during parsing due to unknown signature length
    assert!(result.is_err());
}

/// Test message ID determinism
#[test]
fn test_message_id_determinism() {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&4u16.to_be_bytes());
    bytes.extend_from_slice(&[0x42; 64]);
    bytes.extend_from_slice(&[0x43; 32]);
    bytes.push(0);
    bytes.push(0);
    bytes.extend_from_slice(&0u64.to_be_bytes());
    bytes.extend_from_slice(&0u64.to_be_bytes());
    bytes.extend_from_slice(b"deterministic test");

    let tx1 = AODecoder::decode(&bytes).unwrap();
    let tx2 = AODecoder::decode(&bytes).unwrap();

    assert_eq!(tx1.message_id, tx2.message_id);

    let tx_ir1 = tx1.canonicalize().unwrap();
    let tx_ir2 = tx2.canonicalize().unwrap();

    assert_eq!(tx_ir1.metadata.tx_hash, tx_ir2.metadata.tx_hash);
}

/// Test chain identity
#[test]
fn test_chain_identity_details() {
    let chain = AODecoder::chain();

    assert_eq!(chain.chain_id(), 1000000);
    assert_eq!(chain.chain_name(), "AO");
    assert_eq!(chain.chain_family(), ChainFamily::Actor);
    assert_eq!(chain.network(), Some("mainnet"));
}
