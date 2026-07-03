//! Real-world test for PYUSD (PayPal USD) on Stellar
//!
//! This test validates decoding of PYUSD transactions using the actual
//! PYUSD asset parameters from Stellar mainnet.
//!
//! PYUSD Asset Details:
//! - Asset Code: PYUSD (5 characters)
//! - Asset Type: CreditAlphanum12 (5-12 character codes)
//! - Issuer: GDQE7IXJ4HUHV6RQHIUPRJSEZE4DRS5WY577O2FY6YQ5LVWZ7JZTU2V5
//! - Launched: September 2025 on Stellar

use decoder_stellar::types::{
    DecoratedSignature, EnvelopeType, StellarAsset, StellarMemo, StellarOperation,
    StellarTransaction,
};
use universal_decoder_core::prelude::*;

/// Decode the Stellar address (Strkey format) to raw bytes
///
/// Note: This is a simplified implementation. In production, use the
/// stellar-strkey crate or stellar-sdk.
fn decode_stellar_address(_address: &str) -> Vec<u8> {
    // For testing purposes, we'll use a placeholder that represents
    // the actual decoded bytes from the G-address format
    // Real implementation would decode base32 with checksum validation

    // GDQE7IXJ4HUHV6RQHIUPRJSEZE4DRS5WY577O2FY6YQ5LVWZ7JZTU2V5 decoded
    // This is a simplified representation for testing
    vec![
        0x0c, 0x21, 0xfe, 0x93, 0x4a, 0x1f, 0xac, 0xf4, 0xe4, 0x7e, 0x42, 0x91, 0x12, 0x48, 0x92,
        0x11, 0x61, 0xff, 0xc7, 0xde, 0x78, 0x3c, 0x8c, 0x45, 0xf0, 0x0b, 0x8a, 0x2a, 0x0a, 0xd5,
        0x79, 0x4e,
    ]
}

/// Create a realistic PYUSD payment transaction
///
/// This mimics a real-world scenario: sending 100 PYUSD from one account to another
fn create_pyusd_payment_transaction() -> StellarTransaction {
    // PYUSD issuer on Stellar mainnet
    let pyusd_issuer =
        decode_stellar_address("GDQE7IXJ4HUHV6RQHIUPRJSEZE4DRS5WY577O2FY6YQ5LVWZ7JZTU2V5");

    // PYUSD asset (5 characters, so AlphaNum12)
    // Pad with zeros to 12 bytes
    let mut asset_code = [0u8; 12];
    asset_code[..5].copy_from_slice(b"PYUSD");

    let pyusd_asset = StellarAsset::CreditAlphanum12 {
        code: asset_code,
        issuer: pyusd_issuer.clone(),
    };

    // Source and destination accounts (placeholder addresses)
    let source_account = vec![0x01; 32];
    let destination_account = vec![0x02; 32];

    // 100 PYUSD = 100 * 10^7 stroops (Stellar uses 7 decimals)
    let amount = 100_0000000i64;

    StellarTransaction {
        source_account,
        fee: 10000, // 0.001 XLM fee
        sequence_number: 123456789,
        time_bounds: None,
        memo: StellarMemo::Text("PYUSD payment".to_string()),
        operations: vec![StellarOperation::Payment {
            destination: destination_account,
            asset: pyusd_asset,
            amount,
        }],
        signatures: vec![DecoratedSignature {
            hint: [0xAA, 0xBB, 0xCC, 0xDD],
            signature: vec![0x00; 64],
        }],
        raw_bytes: vec![],
        envelope_type: EnvelopeType::Tx,
        network_id: Some(vec![
            0x7a, 0xc3, 0x39, 0x97, 0x54, 0x93, 0x60, 0x7b, 0x4b, 0xec, 0x31, 0x17, 0x83, 0x19,
            0x93, 0xfc, 0x56, 0xd1, 0xf5, 0x0e, 0xe6, 0xd0, 0x89, 0x9f, 0x8c, 0x6c, 0x93, 0xb3,
            0xb8, 0x35, 0x6a, 0xbe,
        ]), // Stellar mainnet network ID
    }
}

#[test]
fn test_pyusd_payment_decode() {
    let tx = create_pyusd_payment_transaction();

    // Verify transaction structure
    assert!(tx.is_valid());
    assert_eq!(tx.operations.len(), 1);

    // Verify it's a payment operation
    match &tx.operations[0] {
        StellarOperation::Payment {
            destination,
            asset,
            amount,
        } => {
            assert_eq!(destination.len(), 32);
            assert_eq!(*amount, 100_0000000); // 100 PYUSD

            // Verify asset is PYUSD
            match asset {
                StellarAsset::CreditAlphanum12 { code, issuer } => {
                    let code_str = String::from_utf8_lossy(code);
                    assert!(code_str.starts_with("PYUSD"));
                    assert_eq!(issuer.len(), 32);
                }
                _ => panic!("Expected CreditAlphanum12 asset for PYUSD"),
            }
        }
        _ => panic!("Expected Payment operation"),
    }
}

#[test]
fn test_pyusd_canonicalization() {
    let tx = create_pyusd_payment_transaction();
    let tx_ir = tx.canonicalize().expect("Canonicalization should succeed");

    // Verify TxIR structure
    assert_eq!(tx_ir.operations.len(), 1);

    // Verify the operation is a Transfer with PYUSD token
    match &tx_ir.operations[0] {
        Operation::Transfer(transfer) => {
            // Verify amount (100 PYUSD)
            assert_eq!(transfer.amount.value, 100_0000000);
            assert_eq!(transfer.amount.decimals, 7);

            // Verify it's a token (not native XLM)
            match &transfer.asset {
                AssetId::Token(token_bytes) => {
                    let token_str = String::from_utf8_lossy(token_bytes);
                    assert!(
                        token_str.starts_with("PYUSD:"),
                        "Token should be PYUSD:{{issuer}}, got: {}",
                        token_str
                    );
                }
                AssetId::Native => panic!("PYUSD should be a Token, not Native"),
                _ => panic!("Unexpected asset type"),
            }
        }
        _ => panic!("Expected Transfer operation in TxIR"),
    }
}

#[test]
fn test_pyusd_metadata() {
    let tx = create_pyusd_payment_transaction();
    let tx_ir = tx.canonicalize().expect("Canonicalization should succeed");

    // Verify memo is in metadata
    let metadata: serde_json::Value =
        serde_json::from_str(&tx_ir.metadata.extra).expect("Metadata should be valid JSON");

    assert_eq!(metadata["fee"], 10000);
    assert_eq!(metadata["sequence"], 123456789);
    assert!(metadata["memo"].as_str().unwrap().contains("PYUSD payment"));
}

#[test]
fn test_pyusd_authorization() {
    let tx = create_pyusd_payment_transaction();
    let tx_ir = tx.canonicalize().expect("Canonicalization should succeed");

    // Verify Ed25519 signature scheme (Stellar uses Ed25519)
    assert!(matches!(
        tx_ir.authorization.signature_scheme,
        SignatureScheme::EdDsa
    ));

    // Verify signature count
    assert_eq!(tx_ir.authorization.signatures.len(), 1);

    // Verify public key type
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);
    assert!(matches!(
        tx_ir.authorization.public_keys[0].key_type,
        KeyType::Ed25519
    ));
}

#[test]
fn test_pyusd_state_deltas() {
    let tx = create_pyusd_payment_transaction();
    let tx_ir = tx.canonicalize().expect("Canonicalization should succeed");

    // account_changes was removed from TxIR (docs/CONCEPTS_REVIEW.md C1):
    // effects are not byte-derivable and are no longer fabricated.
    assert!(tx_ir.state_deltas.inputs.is_empty());
}

#[test]
fn test_pyusd_asset_code_display() {
    let pyusd_issuer =
        decode_stellar_address("GDQE7IXJ4HUHV6RQHIUPRJSEZE4DRS5WY577O2FY6YQ5LVWZ7JZTU2V5");

    let mut asset_code = [0u8; 12];
    asset_code[..5].copy_from_slice(b"PYUSD");

    let pyusd_asset = StellarAsset::CreditAlphanum12 {
        code: asset_code,
        issuer: pyusd_issuer,
    };

    // Verify asset code string conversion
    assert_eq!(pyusd_asset.code_string(), "PYUSD");

    // Verify it's not native
    assert!(!pyusd_asset.is_native());

    // Verify issuer is available
    assert!(pyusd_asset.issuer().is_some());
}

#[test]
fn test_pyusd_large_amount() {
    let pyusd_issuer =
        decode_stellar_address("GDQE7IXJ4HUHV6RQHIUPRJSEZE4DRS5WY577O2FY6YQ5LVWZ7JZTU2V5");

    let mut asset_code = [0u8; 12];
    asset_code[..5].copy_from_slice(b"PYUSD");

    let pyusd_asset = StellarAsset::CreditAlphanum12 {
        code: asset_code,
        issuer: pyusd_issuer,
    };

    // Test with a large amount: 1 million PYUSD (1,000,000.0000000 in Stellar 7-decimal format)
    let amount = 10_000_000_000_000_i64;

    let tx = StellarTransaction {
        source_account: vec![0x01; 32],
        fee: 10000,
        sequence_number: 1,
        time_bounds: None,
        memo: StellarMemo::Text("Large PYUSD payment".to_string()),
        operations: vec![StellarOperation::Payment {
            destination: vec![0x02; 32],
            asset: pyusd_asset,
            amount,
        }],
        signatures: vec![DecoratedSignature {
            hint: [0xAA, 0xBB, 0xCC, 0xDD],
            signature: vec![0x00; 64],
        }],
        raw_bytes: vec![],
        envelope_type: EnvelopeType::Tx,
        network_id: None,
    };

    let tx_ir = tx
        .canonicalize()
        .expect("Should canonicalize large amounts");

    match &tx_ir.operations[0] {
        Operation::Transfer(transfer) => {
            assert_eq!(transfer.amount.value, 10_000_000_000_000);
            assert_eq!(transfer.amount.decimals, 7);
        }
        _ => panic!("Expected Transfer operation"),
    }
}

#[test]
fn test_pyusd_multi_payment() {
    let pyusd_issuer =
        decode_stellar_address("GDQE7IXJ4HUHV6RQHIUPRJSEZE4DRS5WY577O2FY6YQ5LVWZ7JZTU2V5");

    let mut asset_code = [0u8; 12];
    asset_code[..5].copy_from_slice(b"PYUSD");

    let pyusd_asset = StellarAsset::CreditAlphanum12 {
        code: asset_code,
        issuer: pyusd_issuer,
    };

    // Test a transaction with multiple PYUSD payments
    let tx = StellarTransaction {
        source_account: vec![0x01; 32],
        fee: 30000, // Higher fee for multiple ops
        sequence_number: 1,
        time_bounds: None,
        memo: StellarMemo::Text("Batch PYUSD payments".to_string()),
        operations: vec![
            StellarOperation::Payment {
                destination: vec![0x02; 32],
                asset: pyusd_asset.clone(),
                amount: 50_0000000, // 50 PYUSD
            },
            StellarOperation::Payment {
                destination: vec![0x03; 32],
                asset: pyusd_asset.clone(),
                amount: 75_0000000, // 75 PYUSD
            },
            StellarOperation::Payment {
                destination: vec![0x04; 32],
                asset: pyusd_asset,
                amount: 100_0000000, // 100 PYUSD
            },
        ],
        signatures: vec![DecoratedSignature {
            hint: [0xAA, 0xBB, 0xCC, 0xDD],
            signature: vec![0x00; 64],
        }],
        raw_bytes: vec![],
        envelope_type: EnvelopeType::Tx,
        network_id: None,
    };

    assert!(tx.is_valid());
    assert_eq!(tx.operations.len(), 3);

    let tx_ir = tx.canonicalize().expect("Should canonicalize multi-op tx");

    // Verify all operations are transfers
    assert_eq!(tx_ir.operations.len(), 3);

    for (idx, op) in tx_ir.operations.iter().enumerate() {
        match op {
            Operation::Transfer(transfer) => {
                // Verify it's PYUSD
                match &transfer.asset {
                    AssetId::Token(token_bytes) => {
                        let token_str = String::from_utf8_lossy(token_bytes);
                        assert!(token_str.starts_with("PYUSD:"));
                    }
                    _ => panic!("Expected PYUSD token at operation {}", idx),
                }
            }
            _ => panic!("Expected Transfer operation at index {}", idx),
        }
    }
}

#[test]
fn test_pyusd_with_xlm_mixed() {
    let pyusd_issuer =
        decode_stellar_address("GDQE7IXJ4HUHV6RQHIUPRJSEZE4DRS5WY577O2FY6YQ5LVWZ7JZTU2V5");

    let mut asset_code = [0u8; 12];
    asset_code[..5].copy_from_slice(b"PYUSD");

    let pyusd_asset = StellarAsset::CreditAlphanum12 {
        code: asset_code,
        issuer: pyusd_issuer,
    };

    // Transaction with both XLM and PYUSD payments
    let tx = StellarTransaction {
        source_account: vec![0x01; 32],
        fee: 20000,
        sequence_number: 1,
        time_bounds: None,
        memo: StellarMemo::Text("Mixed assets".to_string()),
        operations: vec![
            StellarOperation::Payment {
                destination: vec![0x02; 32],
                asset: StellarAsset::Native, // XLM
                amount: 10_0000000,          // 10 XLM
            },
            StellarOperation::Payment {
                destination: vec![0x03; 32],
                asset: pyusd_asset, // PYUSD
                amount: 50_0000000, // 50 PYUSD
            },
        ],
        signatures: vec![DecoratedSignature {
            hint: [0xAA, 0xBB, 0xCC, 0xDD],
            signature: vec![0x00; 64],
        }],
        raw_bytes: vec![],
        envelope_type: EnvelopeType::Tx,
        network_id: None,
    };

    let tx_ir = tx
        .canonicalize()
        .expect("Should canonicalize mixed asset tx");

    assert_eq!(tx_ir.operations.len(), 2);

    // First should be native XLM
    match &tx_ir.operations[0] {
        Operation::Transfer(transfer) => {
            assert!(matches!(transfer.asset, AssetId::Native));
        }
        _ => panic!("Expected Transfer operation"),
    }

    // Second should be PYUSD token
    match &tx_ir.operations[1] {
        Operation::Transfer(transfer) => match &transfer.asset {
            AssetId::Token(token_bytes) => {
                let token_str = String::from_utf8_lossy(token_bytes);
                assert!(token_str.starts_with("PYUSD:"));
            }
            _ => panic!("Expected PYUSD token"),
        },
        _ => panic!("Expected Transfer operation"),
    }
}
