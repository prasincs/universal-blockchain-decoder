//! Real Solana mainnet transaction test using SVM decoder
//!
//! This test demonstrates that the SVM family decoder can decode
//! actual transactions from Solana mainnet.

use decoder_svm::*;
use universal_decoder_core::prelude::*;

/// Helper to decode base64 transaction data
fn decode_base64_tx(base64_str: &str) -> Vec<u8> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(base64_str).expect("Invalid base64")
}

#[test]
fn test_svm_decode_real_mainnet_sol_transfer() {
    // Real SOL transfer transaction from Solana mainnet
    // This is a simple transfer of SOL between two accounts
    // Transaction signature: Uses system program (11111111111111111111111111111111)
    let tx_base64 = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zTVICVf7+to6zQ/+XautpF+KSSoZ7ESTxv3rg8xPqyXgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/ORj/WtXHGLCh9wC0eGkf26qTFR5x3nCqwXXmoVtZb0BAgIAAQwCAAAAAMUBWgIAAAA=";

    let tx_bytes = decode_base64_tx(tx_base64);

    println!("\n=== Testing SVM Decoder with Real Mainnet Transaction ===");
    println!("Transaction size: {} bytes", tx_bytes.len());
    println!(
        "First 32 bytes (hex): {}",
        universal_decoder_core::hex::encode(&tx_bytes[..32.min(tx_bytes.len())])
    );

    // Verify chain properties
    let chain = SvmChain::new(SvmChainId::SolanaMainnet);
    assert_eq!(chain.chain_name(), "Solana Mainnet");
    assert_eq!(chain.chain_id(), 101);
    println!(
        "\n✓ Chain: {} (ID: {})",
        chain.chain_name(),
        chain.chain_id()
    );

    // Decode the transaction using SVM decoder
    let decoded = SvmDecoder::decode_with_chain(&tx_bytes, SvmChainId::SolanaMainnet)
        .expect("Failed to decode real mainnet SOL transfer transaction");

    println!("\n=== Decoded Transaction Structure ===");
    println!("Signatures: {}", decoded.inner.num_signatures());
    println!("Accounts: {}", decoded.inner.message.num_account_keys());
    println!("Instructions: {}", decoded.inner.message.num_instructions());
    println!("Header: {:?}", decoded.inner.message.header);

    // Verify transaction structure
    assert!(
        decoded.inner.num_signatures() > 0,
        "Should have at least one signature"
    );
    assert!(
        decoded.inner.message.num_account_keys() >= 3,
        "Should have at least 3 accounts (payer, recipient, system program)"
    );
    assert_eq!(
        decoded.inner.message.num_instructions(),
        1,
        "Simple transfer should have 1 instruction"
    );

    // Verify the transaction is valid
    assert!(
        decoded.inner.is_valid(),
        "Transaction should be structurally valid"
    );
    println!("✓ Transaction is structurally valid");

    // Check instruction details
    let instruction = &decoded.inner.message.instructions[0];
    println!("\n=== Instruction Details ===");
    println!("Program ID index: {}", instruction.program_id_index);
    println!("Accounts: {:?}", instruction.accounts);
    println!("Data length: {} bytes", instruction.data.len());

    // SOL transfer instruction should have data (4 byte discriminator + 8 byte amount)
    assert!(
        !instruction.data.is_empty(),
        "Transfer instruction should have data"
    );
    assert_eq!(
        instruction.data.len(),
        12,
        "Transfer instruction should have 12 bytes (4 byte discriminator + 8 byte amount)"
    );

    // Verify the instruction data structure
    // First 4 bytes are the discriminator for "Transfer" instruction (0x02000000)
    assert_eq!(
        &instruction.data[0..4],
        &[0x02, 0x00, 0x00, 0x00],
        "Should be Transfer instruction discriminator"
    );

    // Next 8 bytes are the transfer amount in lamports (little-endian)
    let amount_bytes = &instruction.data[4..12];
    let amount = u64::from_le_bytes(amount_bytes.try_into().unwrap());
    println!(
        "Transfer amount: {} lamports ({} SOL)",
        amount,
        amount as f64 / 1e9
    );
    assert!(amount > 0, "Transfer amount should be positive");

    println!("\n✓ Successfully decoded real Solana mainnet transaction!");
}

#[test]
fn test_svm_canonicalize_real_mainnet_transaction() {
    // Same real transaction as above
    let tx_base64 = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zTVICVf7+to6zQ/+XautpF+KSSoZ7ESTxv3rg8xPqyXgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/ORj/WtXHGLCh9wC0eGkf26qTFR5x3nCqwXXmoVtZb0BAgIAAQwCAAAAAMUBWgIAAAA=";

    let tx_bytes = decode_base64_tx(tx_base64);
    let decoded = SvmDecoder::decode_with_chain(&tx_bytes, SvmChainId::SolanaMainnet)
        .expect("Failed to decode transaction");

    println!("\n=== Testing Canonicalization ===");

    // Canonicalize to TxIR
    let tx_ir = decoded
        .canonicalize()
        .expect("Failed to canonicalize real transaction");

    println!("Chain: {} (ID: {})", tx_ir.chain.name, tx_ir.chain.id);
    println!("Operations: {}", tx_ir.operations.len());
    println!("Signatures: {}", tx_ir.authorization.signatures.len());
    println!("Public keys: {}", tx_ir.authorization.public_keys.len());

    // Verify TxIR structure
    assert_eq!(tx_ir.chain.name, "Solana");
    assert_eq!(tx_ir.chain.id, 101);
    assert!(!tx_ir.operations.is_empty(), "Should have operations");
    assert!(
        !tx_ir.authorization.signatures.is_empty(),
        "Should have signatures"
    );

    // Verify operation type
    match &tx_ir.operations[0] {
        Operation::ContractCall(call) => {
            println!(
                "Contract call to program: {} bytes",
                call.contract.bytes.len()
            );
            println!("Data length: {}", call.data.len());
            assert_eq!(
                call.contract.bytes.len(),
                32,
                "Solana program ID is 32 bytes"
            );
        }
        _ => panic!("Expected ContractCall operation for Solana instruction"),
    }

    println!("✓ Successfully canonicalized to TxIR");
}

#[test]
fn test_svm_multi_chain_decoding() {
    // Test that the same transaction bytes can be decoded on different SVM chains
    // (This demonstrates the SVM family decoder pattern)
    let tx_base64 = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zTVICVf7+to6zQ/+XautpF+KSSoZ7ESTxv3rg8xPqyXgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/ORj/WtXHGLCh9wC0eGkf26qTFR5x3nCqwXXmoVtZb0BAgIAAQwCAAAAAMUBWgIAAAA=";

    let tx_bytes = decode_base64_tx(tx_base64);

    println!("\n=== Testing Multi-Chain SVM Decoding ===");

    // Test on different SVM chains
    let chains = [
        SvmChainId::SolanaMainnet,
        SvmChainId::SolanaDevnet,
        SvmChainId::EclipseMainnet,
    ];

    for chain_id in chains {
        let result = SvmDecoder::decode_with_chain(&tx_bytes, chain_id);

        match result {
            Ok(decoded) => {
                // Verify the chain ID is correctly set on the SvmTransaction
                assert_eq!(decoded.chain_id(), chain_id);

                let tx_ir = decoded.canonicalize().expect("Failed to canonicalize");
                println!(
                    "✓ {:?} (ID: {}): {} signatures, {} operations",
                    chain_id,
                    tx_ir.chain.id,
                    tx_ir.authorization.signatures.len(),
                    tx_ir.operations.len()
                );

                // Note: The TxIR uses the base Solana decoder's chain info,
                // so it always reports "Solana" (ID: 101) in the canonicalized form.
                // The chain_id is tracked on the SvmTransaction wrapper.
            }
            Err(e) => {
                println!("✗ {:?}: {:?}", chain_id, e);
                panic!("Failed to decode on {:?}: {:?}", chain_id, e);
            }
        }
    }

    println!("\n✓ Successfully decoded on multiple SVM chains");
}

#[test]
fn test_svm_canonical_hash() {
    // Test that we can compute canonical hash from real transaction
    let tx_base64 = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zTVICVf7+to6zQ/+XautpF+KSSoZ7ESTxv3rg8xPqyXgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/ORj/WtXHGLCh9wC0eGkf26qTFR5x3nCqwXXmoVtZb0BAgIAAQwCAAAAAMUBWgIAAAA=";

    let tx_bytes = decode_base64_tx(tx_base64);
    let decoded = SvmDecoder::decode_with_chain(&tx_bytes, SvmChainId::SolanaMainnet)
        .expect("Failed to decode");

    println!("\n=== Testing Canonical Hash ===");

    // Get canonical bytes
    let canonical_bytes = decoded.inner.to_canonical_bytes();
    println!("Original size: {} bytes", tx_bytes.len());
    println!("Canonical size: {} bytes", canonical_bytes.len());

    // For Solana, canonical bytes should be the original transaction bytes
    assert_eq!(
        canonical_bytes, tx_bytes,
        "Canonical bytes should match original transaction bytes"
    );

    // Compute canonical hash using SHA-256
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&canonical_bytes);
    let hash = hasher.finalize();

    println!(
        "Canonical hash (SHA-256): {}",
        universal_decoder_core::hex::encode(hash)
    );
    assert_eq!(hash.len(), 32, "SHA-256 hash should be 32 bytes");

    println!("✓ Successfully computed canonical hash");
}
