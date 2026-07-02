/// Integration tests for TRON decoder with real transactions
use decoder_primitives::prelude::*;
use decoder_tron::TronDecoder;

#[test]
fn test_decode_real_transfer_transaction() {
    // Real TRON transfer transaction from mainnet
    // Transaction ID: 9f62a65d0616c749643c4e2620b7877efd0f04dd5b2b4cd14004570d39858d7e
    // This is the properly formatted Transaction message (with raw_data wrapper)
    let tx_hex = "0a83010a020add22086c2763abadf9ed2940c8d5deea822e5a65080112610a2d747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e73666572436f6e747261637412300a15418840e6c55b9ada326d211d818c34a994aeced808121541d3136787e667d1e055d2cd5db4b5f6c880563049186470ac89dbea822e";

    let tx_bytes = hex::decode(tx_hex).expect("Failed to decode hex");

    // Decode transaction
    let tx = TronDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Validate structure
    assert!(tx.transaction.raw_data.is_some());
    let raw_data = tx.transaction.raw_data.as_ref().unwrap();
    assert_eq!(raw_data.contract.len(), 1);

    // Canonicalize
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize");

    // Validate metadata
    assert!(!tx_ir.metadata.tx_hash.is_empty());
    assert_eq!(tx_ir.metadata.size, tx_bytes.len());

    // Validate operations
    assert_eq!(tx_ir.operations.len(), 1);
    if let Operation::Transfer(transfer) = &tx_ir.operations[0] {
        // Verify it's a TRX transfer (native asset)
        assert_eq!(transfer.asset, AssetId::Native);
        assert_eq!(transfer.amount.decimals, 6);
    } else {
        panic!("Expected Transfer operation");
    }

    // account_changes was removed from TxIR (docs/CONCEPTS_REVIEW.md C1):
    // effects are not byte-derivable and are no longer fabricated.
    assert!(tx_ir.state_deltas.inputs.is_empty());
}

#[test]
fn test_decode_freeze_balance_transaction() {
    // Real TRON FreezeBalance transaction from mainnet
    // Transaction ID: e54bab34838a59e85d5684e46a2e8e512cd11dfb07b35a9728adeaf3d2666fa6
    // This is the properly formatted Transaction message (with raw_data wrapper)
    let tx_hex = "0a760a0271392208d291dee52544509340e8d39598f72f5a58080b12540a32747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e467265657a6542616c616e6365436f6e7472616374121e0a15411fafb1e96dfe4f609e2259bfaf8c77b60c535b9310a0968001180370a7939298f72f";

    let tx_bytes = hex::decode(tx_hex).expect("Failed to decode hex");

    // Decode transaction
    let tx = TronDecoder::decode(&tx_bytes).expect("Failed to decode transaction");

    // Validate structure
    assert!(tx.transaction.raw_data.is_some());
    let raw_data = tx.transaction.raw_data.as_ref().unwrap();
    assert_eq!(raw_data.contract.len(), 1);

    // Canonicalize
    let tx_ir = tx.canonicalize().expect("Failed to canonicalize");

    // Validate operations (should be Stake operation)
    assert_eq!(tx_ir.operations.len(), 1);
    if let Operation::Stake(stake) = &tx_ir.operations[0] {
        assert_eq!(stake.operation_type, StakeOperationType::Delegate);
        assert_eq!(stake.amount.decimals, 6);
    } else {
        panic!("Expected Stake operation, got {:?}", tx_ir.operations[0]);
    }
}

#[test]
fn test_chain_identity() {
    let chain = TronDecoder::chain();
    assert_eq!(chain.chain_id(), 195);
    assert_eq!(chain.chain_name(), "Tron");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
}

#[test]
fn test_validate_empty_transaction() {
    let result = TronDecoder::decode(&[]);
    assert!(result.is_err());
}

#[test]
fn test_validate_invalid_protobuf() {
    let invalid_data = vec![0xFF; 100];
    let result = TronDecoder::decode(&invalid_data);
    assert!(result.is_err());
}

#[test]
fn test_canonicalization_deterministic() {
    // Same transaction decoded twice should produce identical canonical bytes
    let tx_hex = "0a83010a020add22086c2763abadf9ed2940c8d5deea822e5a65080112610a2d747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e73666572436f6e747261637412300a15418840e6c55b9ada326d211d818c34a994aeced808121541d3136787e667d1e055d2cd5db4b5f6c880563049186470ac89dbea822e";
    let tx_bytes = hex::decode(tx_hex).unwrap();

    let tx1 = TronDecoder::decode(&tx_bytes).unwrap();
    let tx2 = TronDecoder::decode(&tx_bytes).unwrap();

    let tx_ir1 = tx1.canonicalize().unwrap();
    let tx_ir2 = tx2.canonicalize().unwrap();

    let canonical1 = tx_ir1.to_canonical_bytes().unwrap();
    let canonical2 = tx_ir2.to_canonical_bytes().unwrap();

    assert_eq!(canonical1, canonical2);
}

#[test]
fn test_transaction_hash_computation() {
    let tx_hex = "0a83010a020add22086c2763abadf9ed2940c8d5deea822e5a65080112610a2d747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e73666572436f6e747261637412300a15418840e6c55b9ada326d211d818c34a994aeced808121541d3136787e667d1e055d2cd5db4b5f6c880563049186470ac89dbea822e";
    let tx_bytes = hex::decode(tx_hex).unwrap();

    let tx = TronDecoder::decode(&tx_bytes).unwrap();
    let tx_ir = tx.canonicalize().unwrap();

    // Transaction hash should be 32 bytes (SHA-256)
    assert_eq!(tx_ir.metadata.tx_hash.len(), 32);

    // Verify hash is deterministic
    let tx2 = TronDecoder::decode(&tx_bytes).unwrap();
    let tx_ir2 = tx2.canonicalize().unwrap();
    assert_eq!(tx_ir.metadata.tx_hash, tx_ir2.metadata.tx_hash);
}
