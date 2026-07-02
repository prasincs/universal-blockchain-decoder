//! Integration tests for Algorand decoder
//!
//! Tests with real and realistic Algorand transactions

use decoder_algorand::{AlgorandDecoder, AlgorandTxType};
use decoder_primitives::prelude::*;
use serde::Serialize;

/// Helper to create a signed payment transaction
fn create_payment_transaction(
    sender: &[u8],
    receiver: &[u8],
    amount: u64,
    fee: u64,
    first_valid: u64,
    last_valid: u64,
) -> Vec<u8> {
    #[derive(Serialize)]
    struct SignedTx<'a> {
        #[serde(rename = "sig", skip_serializing_if = "Option::is_none")]
        sig: Option<Vec<u8>>,
        #[serde(rename = "txn")]
        txn: Transaction<'a>,
    }

    #[derive(Serialize)]
    struct Transaction<'a> {
        #[serde(rename = "amt")]
        amount: u64,
        #[serde(rename = "fee")]
        fee: u64,
        #[serde(rename = "fv")]
        first_valid: u64,
        #[serde(rename = "gen")]
        genesis_id: &'static str,
        #[serde(rename = "gh")]
        genesis_hash: &'a [u8],
        #[serde(rename = "lv")]
        last_valid: u64,
        #[serde(rename = "rcv")]
        receiver: &'a [u8],
        #[serde(rename = "snd")]
        sender: &'a [u8],
        #[serde(rename = "type")]
        tx_type: &'static str,
    }

    let tx = SignedTx {
        sig: Some(vec![0u8; 64]), // Dummy signature
        txn: Transaction {
            amount,
            fee,
            first_valid,
            genesis_id: "mainnet-v1.0",
            genesis_hash: &[0u8; 32],
            last_valid,
            receiver,
            sender,
            tx_type: "pay",
        },
    };

    // Use struct_map mode to serialize with named fields (maps) instead of positional (arrays)
    // This matches Algorand's actual MessagePack encoding
    let mut buf = Vec::new();
    tx.serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
        .expect("Failed to serialize transaction");
    buf
}

/// Helper to create an asset transfer transaction
fn create_asset_transfer(sender: &[u8], receiver: &[u8], asset_id: u64, amount: u64) -> Vec<u8> {
    #[derive(Serialize)]
    struct SignedTx<'a> {
        #[serde(rename = "txn")]
        txn: AssetTransferTx<'a>,
    }

    #[derive(Serialize)]
    struct AssetTransferTx<'a> {
        #[serde(rename = "type")]
        tx_type: &'static str,
        #[serde(rename = "snd")]
        sender: &'a [u8],
        #[serde(rename = "fee")]
        fee: u64,
        #[serde(rename = "fv")]
        first_valid: u64,
        #[serde(rename = "lv")]
        last_valid: u64,
        #[serde(rename = "gh")]
        genesis_hash: &'a [u8],
        #[serde(rename = "arcv")]
        asset_receiver: &'a [u8],
        #[serde(rename = "xaid")]
        xfer_asset: u64,
        #[serde(rename = "aamt")]
        asset_amount: u64,
    }

    let tx = SignedTx {
        txn: AssetTransferTx {
            tx_type: "axfer",
            sender,
            fee: 1000,
            first_valid: 1000,
            last_valid: 2000,
            genesis_hash: &[0u8; 32],
            asset_receiver: receiver,
            xfer_asset: asset_id,
            asset_amount: amount,
        },
    };

    // Use struct_map mode to serialize with named fields (maps) instead of positional (arrays)
    let mut buf = Vec::new();
    tx.serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
        .expect("Failed to serialize asset transfer");
    buf
}

/// Helper to create an application call transaction
fn create_app_call(sender: &[u8], app_id: u64, app_args: Vec<Vec<u8>>) -> Vec<u8> {
    #[derive(Serialize)]
    struct SignedTx<'a> {
        #[serde(rename = "txn")]
        txn: AppCallTx<'a>,
    }

    #[derive(Serialize)]
    struct AppCallTx<'a> {
        #[serde(rename = "type")]
        tx_type: &'static str,
        #[serde(rename = "snd")]
        sender: &'a [u8],
        #[serde(rename = "fee")]
        fee: u64,
        #[serde(rename = "fv")]
        first_valid: u64,
        #[serde(rename = "lv")]
        last_valid: u64,
        #[serde(rename = "gh")]
        genesis_hash: &'a [u8],
        #[serde(rename = "apid")]
        application_id: u64,
        #[serde(rename = "apaa")]
        app_arguments: Vec<Vec<u8>>,
    }

    let tx = SignedTx {
        txn: AppCallTx {
            tx_type: "appl",
            sender,
            fee: 1000,
            first_valid: 1000,
            last_valid: 2000,
            genesis_hash: &[0u8; 32],
            application_id: app_id,
            app_arguments: app_args,
        },
    };

    // Use struct_map mode to serialize with named fields (maps) instead of positional (arrays)
    let mut buf = Vec::new();
    tx.serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
        .expect("Failed to serialize app call");
    buf
}

#[test]
fn test_decode_payment_transaction() {
    let sender = vec![1u8; 32];
    let receiver = vec![2u8; 32];
    let tx_bytes = create_payment_transaction(&sender, &receiver, 1_000_000, 1000, 1000, 2000);

    let decoded = AlgorandDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Verify transaction fields
    assert_eq!(
        decoded.signed_tx.transaction.tx_type,
        AlgorandTxType::Payment
    );
    assert_eq!(decoded.signed_tx.transaction.sender, sender);
    assert_eq!(decoded.signed_tx.transaction.receiver, Some(receiver));
    assert_eq!(decoded.signed_tx.transaction.amount, Some(1_000_000));
    assert_eq!(decoded.signed_tx.transaction.fee, 1000);

    // Verify raw bytes preserved
    assert_eq!(decoded.raw_bytes, tx_bytes);
}

#[test]
fn test_canonicalize_payment_transaction() {
    let sender = vec![1u8; 32];
    let receiver = vec![2u8; 32];
    let tx_bytes = create_payment_transaction(&sender, &receiver, 1_000_000, 1000, 1000, 2000);

    let decoded = AlgorandDecoder::decode(&tx_bytes).unwrap();
    let tx_ir = decoded.canonicalize().expect("Failed to canonicalize");

    // Verify chain identity
    assert_eq!(tx_ir.chain.name, "Algorand");
    assert_eq!(tx_ir.chain.family, ChainFamily::Account.into());

    // Verify metadata
    assert_eq!(tx_ir.metadata.size, tx_bytes.len());
    assert_eq!(tx_ir.metadata.tx_hash.len(), 32); // SHA-512/256

    // Verify authorization (Ed25519)
    assert_eq!(tx_ir.authorization.signature_scheme, SignatureScheme::EdDsa);
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
    assert_eq!(tx_ir.authorization.signatures[0].data.len(), 64);
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);

    // Verify operations (transfer)
    assert_eq!(tx_ir.operations.len(), 1);
    match &tx_ir.operations[0] {
        Operation::Transfer(transfer) => {
            assert_eq!(transfer.from.bytes, sender);
            assert_eq!(transfer.to.bytes, receiver);
            assert_eq!(transfer.amount.value, 1_000_000);
            assert_eq!(transfer.amount.decimals, 6);
        }
        _ => panic!("Expected Transfer operation"),
    }

    // account_changes was removed from TxIR (docs/CONCEPTS_REVIEW.md C1):
    // effects are not byte-derivable and are no longer fabricated.
    assert!(tx_ir.state_deltas.inputs.is_empty());
    let _ = (&sender, &receiver);
}

#[test]
fn test_decode_asset_transfer() {
    let sender = vec![3u8; 32];
    let receiver = vec![4u8; 32];
    let asset_id = 12345;
    let amount = 500_000;

    let tx_bytes = create_asset_transfer(&sender, &receiver, asset_id, amount);
    let decoded = AlgorandDecoder::decode(&tx_bytes).expect("Failed to decode asset transfer");

    // Verify transaction type and fields
    assert_eq!(
        decoded.signed_tx.transaction.tx_type,
        AlgorandTxType::AssetTransfer
    );
    assert_eq!(decoded.signed_tx.transaction.sender, sender);
    assert_eq!(
        decoded.signed_tx.transaction.asset_receiver,
        Some(receiver.clone())
    );
    assert_eq!(decoded.signed_tx.transaction.xfer_asset, Some(asset_id));
    assert_eq!(decoded.signed_tx.transaction.asset_amount, Some(amount));

    // Canonicalize and verify
    let tx_ir = decoded.canonicalize().expect("Failed to canonicalize");

    // Should have a transfer operation with asset ID
    assert_eq!(tx_ir.operations.len(), 1);
    match &tx_ir.operations[0] {
        Operation::Transfer(transfer) => {
            assert_eq!(transfer.from.bytes, sender);
            assert_eq!(transfer.to.bytes, receiver);
            assert_eq!(transfer.amount.value, 500_000);
            // Asset ID is encoded as Token(bytes)
            match &transfer.asset {
                decoder_primitives::AssetId::Token(_) => {} // Correct
                _ => panic!("Expected Token asset ID"),
            }
        }
        _ => panic!("Expected Transfer operation"),
    }
}

#[test]
fn test_decode_application_call() {
    let sender = vec![5u8; 32];
    let app_id = 999;
    let app_args = vec![b"method_name".to_vec(), vec![0, 1, 2, 3]];

    let tx_bytes = create_app_call(&sender, app_id, app_args.clone());
    let decoded = AlgorandDecoder::decode(&tx_bytes).expect("Failed to decode app call");

    // Verify transaction type and fields
    assert_eq!(
        decoded.signed_tx.transaction.tx_type,
        AlgorandTxType::ApplicationCall
    );
    assert_eq!(decoded.signed_tx.transaction.sender, sender);
    assert_eq!(decoded.signed_tx.transaction.application_id, Some(app_id));
    assert_eq!(
        decoded.signed_tx.transaction.app_arguments,
        Some(app_args.clone())
    );

    // Canonicalize and verify
    let tx_ir = decoded.canonicalize().expect("Failed to canonicalize");

    // Should have a contract call operation
    assert_eq!(tx_ir.operations.len(), 1);
    match &tx_ir.operations[0] {
        Operation::ContractCall(call) => {
            assert_eq!(call.contract.human_readable.as_ref().unwrap(), "app-999");
            // Call data is flattened app_arguments
            assert!(!call.data.is_empty());
        }
        _ => panic!("Expected ContractCall operation"),
    }
}

#[test]
fn test_transaction_id_computation() {
    let sender = vec![1u8; 32];
    let receiver = vec![2u8; 32];
    let tx_bytes = create_payment_transaction(&sender, &receiver, 1_000_000, 1000, 1000, 2000);

    let decoded = AlgorandDecoder::decode(&tx_bytes).unwrap();
    let tx_id = decoded.tx_id();

    // Transaction ID should be 32 bytes (SHA-512/256)
    assert_eq!(tx_id.len(), 32);

    // Should be deterministic
    let tx_id2 = decoded.tx_id();
    assert_eq!(tx_id, tx_id2);
}

#[test]
fn test_address_encoding() {
    let pubkey = vec![0xAAu8; 32];
    let tx_bytes = create_payment_transaction(&pubkey, &[0xBBu8; 32], 1000, 1000, 1000, 2000);

    let decoded = AlgorandDecoder::decode(&tx_bytes).unwrap();
    let sender_addr = decoded.sender_address();

    // Algorand addresses are base32 encoded
    assert!(sender_addr
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));

    // Should be deterministic
    let sender_addr2 = decoded.sender_address();
    assert_eq!(sender_addr, sender_addr2);

    // Receiver address (optional for payment)
    let receiver_addr = decoded.receiver_address();
    assert!(receiver_addr.is_some());
}

#[test]
fn test_validation_valid_payment() {
    let sender = vec![1u8; 32];
    let receiver = vec![2u8; 32];
    let tx_bytes = create_payment_transaction(&sender, &receiver, 1_000_000, 1000, 1000, 2000);

    let decoded = AlgorandDecoder::decode(&tx_bytes).unwrap();
    let validation = decoded.validate();

    assert!(validation.is_ok(), "Valid payment should pass validation");
}

#[test]
fn test_validation_invalid_sender_length() {
    // Create transaction with invalid sender length
    #[derive(Serialize)]
    struct BadSignedTx {
        #[serde(rename = "txn")]
        txn: BadTx,
    }

    #[derive(Serialize)]
    struct BadTx {
        #[serde(rename = "type")]
        tx_type: &'static str,
        #[serde(rename = "snd")]
        sender: Vec<u8>, // Wrong length
        #[serde(rename = "fee")]
        fee: u64,
        #[serde(rename = "fv")]
        first_valid: u64,
        #[serde(rename = "lv")]
        last_valid: u64,
        #[serde(rename = "gh")]
        genesis_hash: Vec<u8>,
        #[serde(rename = "rcv")]
        receiver: Vec<u8>,
        #[serde(rename = "amt")]
        amount: u64,
    }

    let bad_tx = BadSignedTx {
        txn: BadTx {
            tx_type: "pay",
            sender: vec![1u8; 16], // Invalid: should be 32 bytes
            fee: 1000,
            first_valid: 1000,
            last_valid: 2000,
            genesis_hash: vec![0u8; 32],
            receiver: vec![2u8; 32],
            amount: 1_000_000,
        },
    };

    // Use struct_map mode for consistent serialization
    let mut buf = Vec::new();
    bad_tx
        .serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
        .unwrap();
    let tx_bytes = buf;
    let decoded = AlgorandDecoder::decode(&tx_bytes).unwrap();

    // Should fail validation
    assert!(
        decoded.validate().is_err(),
        "Invalid sender length should fail validation"
    );
}

#[test]
fn test_validation_invalid_round_range() {
    #[derive(Serialize)]
    struct BadSignedTx {
        #[serde(rename = "txn")]
        txn: BadTx,
    }

    #[derive(Serialize)]
    struct BadTx {
        #[serde(rename = "type")]
        tx_type: &'static str,
        #[serde(rename = "snd")]
        sender: Vec<u8>,
        #[serde(rename = "fee")]
        fee: u64,
        #[serde(rename = "fv")]
        first_valid: u64,
        #[serde(rename = "lv")]
        last_valid: u64,
        #[serde(rename = "gh")]
        genesis_hash: Vec<u8>,
        #[serde(rename = "rcv")]
        receiver: Vec<u8>,
        #[serde(rename = "amt")]
        amount: u64,
    }

    let bad_tx = BadSignedTx {
        txn: BadTx {
            tx_type: "pay",
            sender: vec![1u8; 32],
            fee: 1000,
            first_valid: 2000, // Invalid: first > last
            last_valid: 1000,
            genesis_hash: vec![0u8; 32],
            receiver: vec![2u8; 32],
            amount: 1_000_000,
        },
    };

    // Use struct_map mode for consistent serialization
    let mut buf = Vec::new();
    bad_tx
        .serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
        .unwrap();
    let tx_bytes = buf;
    let decoded = AlgorandDecoder::decode(&tx_bytes).unwrap();

    // Should fail validation
    assert!(
        decoded.validate().is_err(),
        "Invalid round range should fail validation"
    );
}

#[test]
fn test_multiple_transactions_independent() {
    // Ensure decoding multiple transactions doesn't affect each other
    let tx1_bytes = create_payment_transaction(&[1u8; 32], &[2u8; 32], 1000, 1000, 1000, 2000);
    let tx2_bytes = create_payment_transaction(&[3u8; 32], &[4u8; 32], 2000, 1000, 1000, 2000);

    let tx1 = AlgorandDecoder::decode(&tx1_bytes).unwrap();
    let tx2 = AlgorandDecoder::decode(&tx2_bytes).unwrap();

    // Transactions should be independent
    assert_ne!(tx1.tx_id(), tx2.tx_id());
    assert_ne!(
        tx1.signed_tx.transaction.sender,
        tx2.signed_tx.transaction.sender
    );
}

#[test]
fn test_decode_format_validation() {
    // Valid MessagePack map start
    assert!(AlgorandDecoder::validate_format(&[0x81]).is_ok()); // fixmap with 1 element

    // Invalid starts
    assert!(AlgorandDecoder::validate_format(&[]).is_err()); // empty
    assert!(AlgorandDecoder::validate_format(&[0x00]).is_err()); // not a map
    assert!(AlgorandDecoder::validate_format(&[0xFF]).is_err()); // invalid
}
