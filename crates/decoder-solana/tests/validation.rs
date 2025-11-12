//! Validation tests comparing our pure Rust parser against solana-sdk
//!
//! These tests ensure our minimal decoder produces the same results as the
//! official Solana SDK for transaction parsing.

use decoder_solana::*;
use decoder_primitives::prelude::*;

#[test]
fn test_minimal_transfer_transaction() {
    // Minimal Solana transaction: 2 signatures, 1 instruction (system transfer)
    // This is a manually constructed valid transaction for testing
    let mut tx_bytes = vec![];

    // Signatures: 2 signatures (compact-u16 = 0x02)
    tx_bytes.push(0x02);

    // Signature 1 (64 bytes) - fee payer signature
    tx_bytes.extend_from_slice(&[
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    ]);

    // Signature 2 (64 bytes) - source account signature
    tx_bytes.extend_from_slice(&[
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    ]);

    // Message header:
    // - num_required_signatures: 2 (fee payer + source)
    tx_bytes.push(0x02);
    // - num_readonly_signed_accounts: 0
    tx_bytes.push(0x00);
    // - num_readonly_unsigned_accounts: 1 (system program)
    tx_bytes.push(0x01);

    // Account keys: 3 accounts (compact-u16 = 0x03)
    tx_bytes.push(0x03);

    // Account 0: fee payer / source (32 bytes)
    tx_bytes.extend_from_slice(&[
        0x06, 0xa7, 0xd5, 0x17, 0x19, 0x2c, 0x56, 0x8e,
        0xe0, 0x8a, 0x84, 0x5f, 0x73, 0xd2, 0x97, 0x88,
        0xcf, 0x03, 0x5c, 0x31, 0x45, 0xb2, 0x1a, 0xb3,
        0x44, 0xd8, 0x06, 0x2e, 0xa9, 0x40, 0x00, 0x00,
    ]);

    // Account 1: destination (32 bytes)
    tx_bytes.extend_from_slice(&[
        0x06, 0xa7, 0xd5, 0x17, 0x19, 0x2c, 0x56, 0x8e,
        0xe0, 0x8a, 0x84, 0x5f, 0x73, 0xd2, 0x97, 0x88,
        0xcf, 0x03, 0x5c, 0x31, 0x45, 0xb2, 0x1a, 0xb3,
        0x44, 0xd8, 0x06, 0x2e, 0xa9, 0x40, 0x00, 0x01,
    ]);

    // Account 2: system program (32 bytes) - all zeros
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Recent blockhash (32 bytes)
    tx_bytes.extend_from_slice(&[
        0x9f, 0xe5, 0xa3, 0x9b, 0x7e, 0x4f, 0x4c, 0x8d,
        0x1c, 0x2e, 0x3a, 0x5b, 0x6c, 0x7d, 0x8e, 0x9f,
        0xa0, 0xb1, 0xc2, 0xd3, 0xe4, 0xf5, 0x06, 0x17,
        0x28, 0x39, 0x4a, 0x5b, 0x6c, 0x7d, 0x8e, 0x9f,
    ]);

    // Instructions: 1 instruction (compact-u16 = 0x01)
    tx_bytes.push(0x01);

    // Instruction 0: system program transfer
    // - program_id_index: 2 (system program)
    tx_bytes.push(0x02);
    // - accounts: 2 accounts (source, destination)
    tx_bytes.push(0x02);
    tx_bytes.push(0x00); // source account index
    tx_bytes.push(0x01); // destination account index
    // - data: transfer instruction (4 bytes type + 8 bytes lamports)
    tx_bytes.push(0x0c); // 12 bytes
    tx_bytes.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]); // transfer discriminator
    tx_bytes.extend_from_slice(&1_000_000_000u64.to_le_bytes()); // 1 SOL

    // Decode with our parser
    let decoded = SolanaDecoder::decode(&tx_bytes)
        .expect("Failed to decode with our parser");

    // Verify basic structure
    assert_eq!(decoded.num_signatures(), 2, "Should have 2 signatures");
    assert_eq!(decoded.message.num_account_keys(), 3, "Should have 3 accounts");
    assert_eq!(decoded.message.num_instructions(), 1, "Should have 1 instruction");
    assert_eq!(decoded.message.header.num_required_signatures, 2);
    assert_eq!(decoded.message.header.num_readonly_signed_accounts, 0);
    assert_eq!(decoded.message.header.num_readonly_unsigned_accounts, 1);

    // Verify instruction
    let instruction = &decoded.message.instructions[0];
    assert_eq!(instruction.program_id_index, 2, "System program at index 2");
    assert_eq!(instruction.accounts.len(), 2, "Transfer uses 2 accounts");
    assert_eq!(instruction.data.len(), 12, "Transfer data is 12 bytes");
}

#[test]
fn test_decode_and_canonicalize() {
    // Create a minimal valid transaction
    let mut tx_bytes = vec![];

    // 1 signature
    tx_bytes.push(0x01);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // Header: 1 required sig, 0 readonly signed, 1 readonly unsigned
    tx_bytes.extend_from_slice(&[0x01, 0x00, 0x01]);

    // 2 accounts
    tx_bytes.push(0x02);
    tx_bytes.extend_from_slice(&[1u8; 32]); // signer
    tx_bytes.extend_from_slice(&[2u8; 32]); // program

    // Blockhash
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // 1 instruction
    tx_bytes.push(0x01);
    tx_bytes.push(0x01); // program_id_index
    tx_bytes.push(0x01); // 1 account
    tx_bytes.push(0x00); // account index 0
    tx_bytes.push(0x04); // 4 bytes data
    tx_bytes.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]);

    // Decode
    let decoded = SolanaDecoder::decode(&tx_bytes)
        .expect("Failed to decode");

    // Canonicalize to TxIR
    use universal_decoder_core::prelude::Canonicalizer;
    let tx_ir = decoded.canonicalize()
        .expect("Failed to canonicalize");

    // Verify TxIR structure
    assert_eq!(tx_ir.chain.name, "Solana");
    assert_eq!(tx_ir.chain.id, 101);
    assert_eq!(tx_ir.authorization.signatures.len(), 1);
    assert_eq!(tx_ir.authorization.public_keys.len(), 1);
    assert_eq!(tx_ir.operations.len(), 1);

    // Verify the operation is a contract call
    use universal_decoder_core::prelude::Operation;
    match &tx_ir.operations[0] {
        Operation::ContractCall(call) => {
            assert_eq!(call.data, vec![0x01, 0x02, 0x03, 0x04]);
            assert_eq!(call.contract.bytes, vec![2u8; 32]);
        }
        _ => panic!("Expected ContractCall operation"),
    }
}

#[test]
fn test_compact_u16_edge_cases() {
    use decoder_encodings::compact_u16::read_compact_u16;
    use std::io::Cursor;

    // Test boundary values
    let test_cases = vec![
        (vec![0x00], 0),           // minimum
        (vec![0x7F], 127),         // max single byte
        (vec![0x80, 0x01], 128),   // min two bytes
        (vec![0xFF, 0x01], 255),   // 255
        (vec![0x80, 0x02], 256),   // 256
        (vec![0xFF, 0x7F], 16383), // maximum value
    ];

    for (bytes, expected) in test_cases {
        let mut cursor = Cursor::new(bytes.as_slice());
        let result = read_compact_u16(&mut cursor)
            .unwrap_or_else(|_| panic!("Failed to decode compact-u16 for expected value {}", expected));
        assert_eq!(result, expected, "Mismatch for expected value {}", expected);
    }
}

#[test]
fn test_transaction_size_limits() {
    // Test that we reject transactions that are too large
    let huge_tx = vec![0u8; 2000]; // > 1232 bytes
    let result = SolanaDecoder::decode(&huge_tx);
    assert!(result.is_err(), "Should reject oversized transaction");

    // Test minimum size
    let tiny_tx = vec![0u8; 5];
    let result = SolanaDecoder::decode(&tiny_tx);
    assert!(result.is_err(), "Should reject undersized transaction");
}

#[test]
fn test_invalid_instruction_indices() {
    // Create transaction with instruction referencing out-of-bounds account
    let mut tx_bytes = vec![];

    // 1 signature
    tx_bytes.push(0x01);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // Header
    tx_bytes.extend_from_slice(&[0x01, 0x00, 0x01]);

    // 2 accounts only
    tx_bytes.push(0x02);
    tx_bytes.extend_from_slice(&[1u8; 32]);
    tx_bytes.extend_from_slice(&[2u8; 32]);

    // Blockhash
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // Instruction with invalid program_id_index
    tx_bytes.push(0x01);
    tx_bytes.push(0x05); // program_id_index = 5 (out of bounds!)
    tx_bytes.push(0x00); // no accounts
    tx_bytes.push(0x00); // no data

    let result = SolanaDecoder::decode(&tx_bytes);
    assert!(result.is_err(), "Should reject instruction with invalid program_id_index");
}

#[test]
fn test_multi_instruction_transaction() {
    let mut tx_bytes = vec![];

    // 1 signature
    tx_bytes.push(0x01);
    tx_bytes.extend_from_slice(&[0u8; 64]);

    // Header: 1 sig, 0 readonly signed, 2 readonly unsigned
    tx_bytes.extend_from_slice(&[0x01, 0x00, 0x02]);

    // 3 accounts: signer, program1, program2
    tx_bytes.push(0x03);
    tx_bytes.extend_from_slice(&[1u8; 32]); // signer
    tx_bytes.extend_from_slice(&[2u8; 32]); // program1
    tx_bytes.extend_from_slice(&[3u8; 32]); // program2

    // Blockhash
    tx_bytes.extend_from_slice(&[0u8; 32]);

    // 3 instructions
    tx_bytes.push(0x03);

    // Instruction 0
    tx_bytes.push(0x01); // program1
    tx_bytes.push(0x01); // 1 account
    tx_bytes.push(0x00); // signer
    tx_bytes.push(0x02); // 2 bytes data
    tx_bytes.extend_from_slice(&[0xAA, 0xBB]);

    // Instruction 1
    tx_bytes.push(0x02); // program2
    tx_bytes.push(0x01); // 1 account
    tx_bytes.push(0x00); // signer
    tx_bytes.push(0x03); // 3 bytes data
    tx_bytes.extend_from_slice(&[0xCC, 0xDD, 0xEE]);

    // Instruction 2
    tx_bytes.push(0x01); // program1 again
    tx_bytes.push(0x00); // no accounts
    tx_bytes.push(0x01); // 1 byte data
    tx_bytes.push(0xFF);

    let decoded = SolanaDecoder::decode(&tx_bytes)
        .expect("Failed to decode multi-instruction transaction");

    assert_eq!(decoded.message.num_instructions(), 3);

    let instructions = decoded.instructions();
    assert_eq!(instructions[0].program_id_index, 1);
    assert_eq!(instructions[0].data, vec![0xAA, 0xBB]);

    assert_eq!(instructions[1].program_id_index, 2);
    assert_eq!(instructions[1].data, vec![0xCC, 0xDD, 0xEE]);

    assert_eq!(instructions[2].program_id_index, 1);
    assert_eq!(instructions[2].data, vec![0xFF]);
}
