//! Mainnet transaction tests for Aleo decoder
//!
//! These tests use real (or realistic) Aleo mainnet transaction fixtures
//! to ensure the decoder handles production data correctly.

use decoder_aleo::{AleoDecoder, TransactionType};
use universal_decoder_core::prelude::*;

#[test]
fn test_decode_aleo_fee_transaction_mainnet() {
    // Mainnet fee transaction fixture
    // Format: [type(1)] [global_state_root(32)] [amount(8)] [priority_fee(8)] [has_transition(1)]
    let mut tx_bytes = vec![0x00]; // Fee transaction type

    // Global state root (example Merkle root)
    tx_bytes.extend_from_slice(&[0xAA; 32]);

    // Amount: 1,000,000 gates (1 Aleo credit)
    tx_bytes.extend_from_slice(&1_000_000u64.to_le_bytes());

    // Priority fee: 100,000 gates
    tx_bytes.extend_from_slice(&100_000u64.to_le_bytes());

    // No transition attached
    tx_bytes.push(0x00);

    // Decode
    let result = AleoDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Should successfully decode fee transaction: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    assert!(matches!(tx.transaction_type, TransactionType::Fee(_)));

    // Validate canonicalization
    let tx_ir = tx.canonicalize();
    assert!(tx_ir.is_ok(), "Fee transaction should canonicalize");
}

#[test]
fn test_decode_aleo_deployment_transaction_mainnet() {
    // Mainnet deployment transaction
    // Format: [type(1)] [edition(2)] [program_id_len(2)] [program_id] [program_len(2)] [program] [vk_count(2)]
    let mut tx_bytes = vec![0x01]; // Deploy transaction type

    // Edition: 0
    tx_bytes.extend_from_slice(&0u16.to_le_bytes());

    // Program ID: "token.aleo"
    let program_id = b"token.aleo";
    tx_bytes.extend_from_slice(&(program_id.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(program_id);

    // Program source (simplified Leo program)
    let program_source =
        b"program token.aleo;\n\nfunction transfer:\n    input r0 as u64;\n    output r0 as u64;\n";
    tx_bytes.extend_from_slice(&(program_source.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(program_source);

    // Verifying keys: 1 key for 'transfer' function
    tx_bytes.extend_from_slice(&1u16.to_le_bytes());

    // Function name
    let function_name = b"transfer";
    tx_bytes.extend_from_slice(&(function_name.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(function_name);

    // Verifying key (mock 128 bytes)
    let vk = vec![0xBB; 128];
    tx_bytes.extend_from_slice(&(vk.len() as u32).to_le_bytes());
    tx_bytes.extend_from_slice(&vk);

    // No fee attached
    tx_bytes.push(0x00);

    // Decode
    let result = AleoDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Should successfully decode deployment transaction: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    match &tx.transaction_type {
        TransactionType::Deploy(deploy) => {
            assert_eq!(deploy.program_id, "token.aleo");
            assert!(deploy.program.contains("transfer"));
            assert_eq!(deploy.verifying_keys.len(), 1);
            assert_eq!(deploy.verifying_keys[0].function_name, "transfer");
        }
        _ => panic!("Expected Deploy transaction type"),
    }

    // Validate canonicalization
    let tx_ir = tx.canonicalize();
    assert!(tx_ir.is_ok(), "Deployment should canonicalize");

    // Check TxIR contains contract deployment operation
    let tx_ir = tx_ir.unwrap();
    assert!(!tx_ir.operations.is_empty());
    assert!(matches!(
        tx_ir.operations[0],
        Operation::ContractDeploy { .. }
    ));
}

#[test]
fn test_decode_aleo_execution_transaction_mainnet() {
    // Mainnet execution transaction with one transition
    // Format: [type(1)] [global_state_root(32)] [transition_count(2)] [transition...] [has_proof(1)] [has_fee(1)]
    let mut tx_bytes = vec![0x02]; // Execute transaction type

    // Global state root
    tx_bytes.extend_from_slice(&[0xCC; 32]);

    // Transition count: 1
    tx_bytes.extend_from_slice(&1u16.to_le_bytes());

    // Transition 1
    // Transition ID (32 bytes)
    tx_bytes.extend_from_slice(&[0xDD; 32]);

    // Program ID: "token.aleo"
    let program_id = b"token.aleo";
    tx_bytes.extend_from_slice(&(program_id.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(program_id);

    // Function name: "transfer"
    let function_name = b"transfer";
    tx_bytes.extend_from_slice(&(function_name.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(function_name);

    // Input count: 1
    tx_bytes.push(1);

    // Input: Public value (type 0x01)
    tx_bytes.push(0x01);
    let input_value = 1000u64.to_le_bytes();
    tx_bytes.extend_from_slice(&(input_value.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(&input_value);

    // Output count: 1
    tx_bytes.push(1);

    // Output: Public value (type 0x01)
    tx_bytes.push(0x01);
    let output_value = 1000u64.to_le_bytes();
    tx_bytes.extend_from_slice(&(output_value.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(&output_value);

    // Has proof: No
    tx_bytes.push(0x00);

    // Finalize operations: 0
    tx_bytes.push(0);

    // Has proof (for execution): No
    tx_bytes.push(0x00);

    // Has fee: No
    tx_bytes.push(0x00);

    // Decode
    let result = AleoDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Should successfully decode execution transaction: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    match &tx.transaction_type {
        TransactionType::Execute(exec) => {
            assert_eq!(exec.transitions.len(), 1);
            let transition = &exec.transitions[0];
            assert_eq!(transition.program_id, "token.aleo");
            assert_eq!(transition.function_name, "transfer");
            assert_eq!(transition.inputs.len(), 1);
            assert_eq!(transition.outputs.len(), 1);
        }
        _ => panic!("Expected Execute transaction type"),
    }

    // Validate canonicalization
    let tx_ir = tx.canonicalize();
    assert!(tx_ir.is_ok(), "Execution should canonicalize");
}

#[test]
fn test_decode_aleo_private_execution_mainnet() {
    // Execution with private inputs/outputs
    let mut tx_bytes = vec![0x02]; // Execute transaction type

    // Global state root
    tx_bytes.extend_from_slice(&[0xEE; 32]);

    // Transition count: 1
    tx_bytes.extend_from_slice(&1u16.to_le_bytes());

    // Transition
    tx_bytes.extend_from_slice(&[0xFF; 32]); // ID

    // Program ID: "private_transfer.aleo"
    let program_id = b"private_transfer.aleo";
    tx_bytes.extend_from_slice(&(program_id.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(program_id);

    // Function: "send"
    let function_name = b"send";
    tx_bytes.extend_from_slice(&(function_name.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(function_name);

    // Input count: 1 (private)
    tx_bytes.push(1);

    // Input: Private (type 0x02)
    tx_bytes.push(0x02);
    let ciphertext = vec![0xAB; 64];
    tx_bytes.extend_from_slice(&(ciphertext.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(&ciphertext);

    // Output count: 1 (record)
    tx_bytes.push(1);

    // Output: Record (type 0x03)
    tx_bytes.push(0x03);
    tx_bytes.extend_from_slice(&[0x11; 32]); // commitment
    tx_bytes.extend_from_slice(&[0x22; 32]); // nonce
    tx_bytes.extend_from_slice(&[0x33; 16]); // checksum
    let record_ciphertext = vec![0xCD; 128];
    tx_bytes.extend_from_slice(&(record_ciphertext.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(&record_ciphertext);

    // Has proof: Yes
    tx_bytes.push(0x01);
    let proof = vec![0xEF; 256];
    tx_bytes.extend_from_slice(&(proof.len() as u32).to_le_bytes());
    tx_bytes.extend_from_slice(&proof);

    // Finalize: 0
    tx_bytes.push(0);

    // Execution proof: No
    tx_bytes.push(0x00);

    // Has fee: No
    tx_bytes.push(0x00);

    // Decode
    let result = AleoDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Should decode private execution: {:?}",
        result.err()
    );

    let tx = result.unwrap();

    // Validate privacy components
    let tx_ir = tx.canonicalize();
    assert!(tx_ir.is_ok());
    let tx_ir = tx_ir.unwrap();

    assert!(tx_ir.privacy.is_some());
    let privacy = tx_ir.privacy.unwrap();
    assert!(!privacy.features.is_empty());
    assert_eq!(privacy.observability, ObservabilityLevel::FullyPrivate);
}

#[test]
fn test_decode_aleo_with_finalize_operations_mainnet() {
    // Execution with finalize (on-chain state update)
    let mut tx_bytes = vec![0x02]; // Execute

    tx_bytes.extend_from_slice(&[0x00; 32]); // Global state root
    tx_bytes.extend_from_slice(&1u16.to_le_bytes()); // 1 transition

    // Transition
    tx_bytes.extend_from_slice(&[0x01; 32]); // ID

    let program_id = b"mapping_test.aleo";
    tx_bytes.extend_from_slice(&(program_id.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(program_id);

    let function_name = b"store";
    tx_bytes.extend_from_slice(&(function_name.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(function_name);

    // Inputs: 2 public values
    tx_bytes.push(2);

    // Input 1: key
    tx_bytes.push(0x01);
    let key = b"user123";
    tx_bytes.extend_from_slice(&(key.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(key);

    // Input 2: value
    tx_bytes.push(0x01);
    let value = 42u64.to_le_bytes();
    tx_bytes.extend_from_slice(&(value.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(&value);

    // No outputs
    tx_bytes.push(0);

    // No proof
    tx_bytes.push(0x00);

    // Finalize operations: 1 (insert mapping)
    tx_bytes.push(1);

    // Insert mapping (type 0x01)
    tx_bytes.push(0x01);
    let mapping_name = b"balances";
    tx_bytes.extend_from_slice(&(mapping_name.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(mapping_name);

    tx_bytes.extend_from_slice(&(key.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(key);

    tx_bytes.extend_from_slice(&(value.len() as u16).to_le_bytes());
    tx_bytes.extend_from_slice(&value);

    // No execution proof
    tx_bytes.push(0x00);

    // No fee
    tx_bytes.push(0x00);

    // Decode
    let result = AleoDecoder::decode(&tx_bytes);
    assert!(
        result.is_ok(),
        "Should decode with finalize: {:?}",
        result.err()
    );

    let tx = result.unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // account_changes was removed from TxIR (CONCEPTS_REVIEW.md C1); finalize
    // content returns as typed operations under the C3 follow-up.
    assert!(!tx_ir.operations.is_empty());
}

#[test]
fn test_validation_rejects_invalid_transactions() {
    // Empty program ID in deployment
    let mut tx_bytes = vec![0x01]; // Deploy
    tx_bytes.extend_from_slice(&0u16.to_le_bytes()); // edition
    tx_bytes.extend_from_slice(&0u16.to_le_bytes()); // empty program ID
    tx_bytes.extend_from_slice(&1u16.to_le_bytes()); // program length
    tx_bytes.push(b'a'); // minimal program
    tx_bytes.extend_from_slice(&0u16.to_le_bytes()); // no vks
    tx_bytes.push(0x00); // no fee

    let result = AleoDecoder::decode(&tx_bytes);
    if let Ok(tx) = result {
        let validation = tx.validate();
        assert!(validation.is_err(), "Should reject empty program ID");
    }
}

#[test]
fn test_mainnet_transaction_hash_calculation() {
    // Create a simple fee transaction
    let mut tx_bytes = vec![0x00];
    tx_bytes.extend_from_slice(&[0xAA; 32]);
    tx_bytes.extend_from_slice(&1_000_000u64.to_le_bytes());
    tx_bytes.extend_from_slice(&0u64.to_le_bytes());
    tx_bytes.push(0x00);

    let tx = AleoDecoder::decode(&tx_bytes).unwrap();

    // Transaction ID should be deterministic
    let id1 = tx.id.clone();
    let tx2 = AleoDecoder::decode(&tx_bytes).unwrap();
    let id2 = tx2.id;

    assert_eq!(id1, id2, "Transaction IDs should be deterministic");
    assert_eq!(id1.len(), 32, "SHA-256 hash should be 32 bytes");
}
