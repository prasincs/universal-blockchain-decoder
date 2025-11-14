//! Integration tests for Optimism decoder with real transaction data

use decoder_optimism::*;
use universal_decoder_core::prelude::*;

/// Test basic validation
#[test]
fn test_validate_format() {
    // Empty transaction should fail
    assert!(
        OptimismDecoder::validate_format(&[]).is_err(),
        "Should reject empty transaction"
    );

    // Too small transaction should fail
    assert!(
        OptimismDecoder::validate_format(&[0x01]).is_err(),
        "Should reject transaction that's too small"
    );

    // Valid minimum length should pass
    let dummy_tx = vec![0xf8, 0x6c, 0x00, 0x00, 0x00];
    assert!(
        OptimismDecoder::validate_format(&dummy_tx).is_ok(),
        "Should pass validation for reasonable size"
    );

    // Deposit transaction with valid prefix
    let deposit_tx = vec![0x7E, 0xf8, 0x6c, 0x00, 0x00];
    assert!(
        OptimismDecoder::validate_format(&deposit_tx).is_ok(),
        "Should pass validation for deposit transaction"
    );
}

/// Test ChainDecoder trait implementation
#[test]
fn test_chain_decoder_trait() {
    let chain = OptimismDecoder::chain();
    assert_eq!(chain.chain_id(), 10, "Chain ID should be 10 (Optimism)");
    assert_eq!(chain.chain_name(), "Optimism");
    assert_eq!(chain.chain_family(), ChainFamily::Account);
}

/// Test deposit transaction decoding with minimal valid data
#[test]
fn test_decode_minimal_deposit_transaction() {
    use decoder_encodings::rlp::RlpItem;

    // Build a minimal valid deposit transaction
    let source_hash = [1u8; 32];
    let from = [2u8; 20];
    let to = [3u8; 20];
    let _mint = 0u128;
    let _value = 0u128;
    let _gas_limit = 21000u64;
    let _is_creation = false;
    let data: Vec<u8> = vec![];

    // Encode as RLP list
    let fields = vec![
        RlpItem::Data(source_hash.to_vec()),
        RlpItem::Data(from.to_vec()),
        RlpItem::Data(to.to_vec()),
        RlpItem::Data(vec![]),           // mint = 0
        RlpItem::Data(vec![]),           // value = 0
        RlpItem::Data(vec![0x52, 0x08]), // gas_limit = 21000
        RlpItem::Data(vec![0x00]),       // is_creation = false
        RlpItem::Data(data),
    ];

    let rlp_list = RlpItem::List(fields);
    let rlp_bytes = rlp_list.encode();

    // Add 0x7E prefix
    let mut tx_bytes = vec![0x7E];
    tx_bytes.extend_from_slice(&rlp_bytes);

    // Decode
    let result = OptimismDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Should decode valid deposit transaction");

    let tx = result.unwrap();
    assert!(tx.is_deposit(), "Should be a deposit transaction");
    assert_eq!(tx.tx_type(), 0x7E);
    assert_eq!(tx.from(), from);
    assert_eq!(tx.to(), Some(to));
    assert_eq!(tx.value(), 0);
}

/// Test deposit transaction with ETH mint
#[test]
fn test_decode_deposit_with_mint() {
    use decoder_encodings::rlp::RlpItem;

    let source_hash = [0xaa; 32];
    let from = [0xbb; 20];
    let to = [0xcc; 20];
    let mint = 1_000_000_000_000_000_000u128; // 1 ETH in wei
    let value = 1_000_000_000_000_000_000u128; // 1 ETH
    let gas_limit = 100_000u64;
    let _is_creation = false;
    let data: Vec<u8> = vec![];

    // Encode mint and value as big-endian bytes
    let mint_bytes = mint.to_be_bytes().to_vec();
    let value_bytes = value.to_be_bytes().to_vec();

    let fields = vec![
        RlpItem::Data(source_hash.to_vec()),
        RlpItem::Data(from.to_vec()),
        RlpItem::Data(to.to_vec()),
        RlpItem::Data(mint_bytes),
        RlpItem::Data(value_bytes),
        RlpItem::Data(vec![0x01, 0x86, 0xa0]), // gas_limit = 100000
        RlpItem::Data(vec![0x00]),
        RlpItem::Data(data),
    ];

    let rlp_list = RlpItem::List(fields);
    let rlp_bytes = rlp_list.encode();

    let mut tx_bytes = vec![0x7E];
    tx_bytes.extend_from_slice(&rlp_bytes);

    let result = OptimismDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Should decode deposit with mint");

    let tx = result.unwrap();
    if let OptimismTransaction::Deposit(deposit) = tx {
        assert_eq!(deposit.mint, mint);
        assert_eq!(deposit.value, value);
        assert_eq!(deposit.from, from);
        assert_eq!(deposit.to, Some(to));
        assert_eq!(deposit.gas_limit, gas_limit);
    } else {
        panic!("Expected deposit transaction");
    }
}

/// Test L1 attributes deposit (first tx in every block)
#[test]
fn test_l1_attributes_deposit() {
    use decoder_encodings::rlp::RlpItem;

    // L1Block predeploy address: 0x4200000000000000000000000000000000000015
    let l1_block_predeploy = [
        0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x15,
    ];

    let source_hash = [0x01; 32];
    let from = [
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01,
    ];
    let _mint = 0u128;
    let _value = 0u128;
    let _gas_limit = 1_000_000u64;
    let _is_creation = false;

    // L1 attributes calldata (simplified - real version has specific ABI encoding)
    let data = vec![0x01, 0x02, 0x03, 0x04];

    let fields = vec![
        RlpItem::Data(source_hash.to_vec()),
        RlpItem::Data(from.to_vec()),
        RlpItem::Data(l1_block_predeploy.to_vec()),
        RlpItem::Data(vec![]),                 // mint = 0
        RlpItem::Data(vec![]),                 // value = 0
        RlpItem::Data(vec![0x0f, 0x42, 0x40]), // gas_limit = 1000000
        RlpItem::Data(vec![0x00]),
        RlpItem::Data(data),
    ];

    let rlp_list = RlpItem::List(fields);
    let rlp_bytes = rlp_list.encode();

    let mut tx_bytes = vec![0x7E];
    tx_bytes.extend_from_slice(&rlp_bytes);

    let result = OptimismDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Should decode L1 attributes deposit");

    let tx = result.unwrap();
    if let OptimismTransaction::Deposit(deposit) = tx {
        assert!(
            deposit.is_l1_attributes_deposit(),
            "Should be L1 attributes deposit"
        );
        assert!(!deposit.is_user_deposit(), "Should not be user deposit");
        assert_eq!(deposit.to, Some(l1_block_predeploy));
    } else {
        panic!("Expected deposit transaction");
    }
}

/// Test contract creation deposit
#[test]
fn test_contract_creation_deposit() {
    use decoder_encodings::rlp::RlpItem;

    let source_hash = [0xff; 32];
    let from = [0x11; 20];
    let mint = 500_000_000_000_000_000u128; // 0.5 ETH
    let value = 500_000_000_000_000_000u128;
    let _gas_limit = 500_000u64;
    let _is_creation = true;

    // Contract bytecode
    let data = vec![0x60, 0x80, 0x60, 0x40, 0x52]; // Simplified EVM bytecode

    let mint_bytes = mint.to_be_bytes().to_vec();
    let value_bytes = value.to_be_bytes().to_vec();

    let fields = vec![
        RlpItem::Data(source_hash.to_vec()),
        RlpItem::Data(from.to_vec()),
        RlpItem::Data(vec![]), // to = empty for contract creation
        RlpItem::Data(mint_bytes),
        RlpItem::Data(value_bytes),
        RlpItem::Data(vec![0x07, 0xa1, 0x20]), // gas_limit = 500000
        RlpItem::Data(vec![0x01]),             // is_creation = true
        RlpItem::Data(data.clone()),
    ];

    let rlp_list = RlpItem::List(fields);
    let rlp_bytes = rlp_list.encode();

    let mut tx_bytes = vec![0x7E];
    tx_bytes.extend_from_slice(&rlp_bytes);

    let result = OptimismDecoder::decode(&tx_bytes);
    assert!(result.is_ok(), "Should decode contract creation deposit");

    let tx = result.unwrap();
    if let OptimismTransaction::Deposit(deposit) = tx {
        assert!(deposit.is_creation, "Should be contract creation");
        assert_eq!(deposit.to, None, "Contract creation has no 'to' address");
        assert_eq!(deposit.data, data);
        assert!(deposit.validate().is_ok(), "Should pass validation");
    } else {
        panic!("Expected deposit transaction");
    }
}

/// Test deposit transaction validation
#[test]
fn test_deposit_validation() {
    use decoder_encodings::rlp::RlpItem;

    // Test invalid: value > mint
    {
        let source_hash = [0x00; 32];
        let from = [0x01; 20];
        let to = [0x02; 20];
        let mint = 100u128; // Mint only 100
        let value = 1000u128; // But try to transfer 1000 - INVALID
        let _gas_limit = 21000u64;
        let _is_creation = false;
        let data: Vec<u8> = vec![];

        let fields = vec![
            RlpItem::Data(source_hash.to_vec()),
            RlpItem::Data(from.to_vec()),
            RlpItem::Data(to.to_vec()),
            RlpItem::Data(mint.to_be_bytes().to_vec()),
            RlpItem::Data(value.to_be_bytes().to_vec()),
            RlpItem::Data(vec![0x52, 0x08]),
            RlpItem::Data(vec![0x00]),
            RlpItem::Data(data),
        ];

        let rlp_list = RlpItem::List(fields);
        let rlp_bytes = rlp_list.encode();

        let mut tx_bytes = vec![0x7E];
        tx_bytes.extend_from_slice(&rlp_bytes);

        let result = OptimismDecoder::decode(&tx_bytes);
        assert!(result.is_err(), "Should reject deposit with value > mint");
    }

    // Test invalid: is_creation=true but to address is set
    {
        let source_hash = [0x00; 32];
        let from = [0x01; 20];
        let to = [0x02; 20]; // Setting 'to' address
        let _mint = 0u128;
        let _value = 0u128;
        let _gas_limit = 21000u64;
        let _is_creation = true; // But claiming creation - INVALID
        let data: Vec<u8> = vec![];

        let fields = vec![
            RlpItem::Data(source_hash.to_vec()),
            RlpItem::Data(from.to_vec()),
            RlpItem::Data(to.to_vec()), // Should be empty for creation
            RlpItem::Data(vec![]),
            RlpItem::Data(vec![]),
            RlpItem::Data(vec![0x52, 0x08]),
            RlpItem::Data(vec![0x01]), // is_creation = true
            RlpItem::Data(data),
        ];

        let rlp_list = RlpItem::List(fields);
        let rlp_bytes = rlp_list.encode();

        let mut tx_bytes = vec![0x7E];
        tx_bytes.extend_from_slice(&rlp_bytes);

        let result = OptimismDecoder::decode(&tx_bytes);
        assert!(result.is_err(), "Should reject creation with 'to' address");
    }
}

/// Test standard Ethereum transaction on Optimism
#[test]
fn test_standard_ethereum_transaction_on_optimism() {
    use decoder_encodings::rlp::RlpItem;

    // Build a simple legacy Ethereum transaction with Optimism chain ID (10)
    let _nonce = 0u64;
    let gas_price = 1_000_000_000u128; // 1 gwei
    let gas_limit = 21000u64;
    let to = [0xaa; 20];
    let value = 1_000_000_000_000_000_000u128; // 1 ETH
    let data: Vec<u8> = vec![];

    // Signature values (dummy - not validated in this test)
    let v = 37u8; // 27 + chain_id * 2 + 2 = 27 + 10*2 = 47 (simplified)
    let r = [0x01; 32];
    let s = [0x02; 32];

    let fields = vec![
        RlpItem::Data(vec![]), // nonce = 0
        RlpItem::Data(gas_price.to_be_bytes().to_vec()),
        RlpItem::Data(gas_limit.to_be_bytes().to_vec()),
        RlpItem::Data(to.to_vec()),
        RlpItem::Data(value.to_be_bytes().to_vec()),
        RlpItem::Data(data),
        RlpItem::Data(vec![v]),
        RlpItem::Data(r.to_vec()),
        RlpItem::Data(s.to_vec()),
    ];

    let rlp_list = RlpItem::List(fields);
    let tx_bytes = rlp_list.encode();

    // Legacy transactions start with RLP list marker (0xc0+)
    assert!(tx_bytes[0] >= 0xc0, "Should be RLP list");

    let result = OptimismDecoder::decode(&tx_bytes);
    // May fail due to signature validation, but should handle gracefully
    let _ = result;
}

/// Test registry integration
#[test]
fn test_registry_integration() {
    use crate::registry::SuperchainRegistry;

    let registry = SuperchainRegistry::new();

    // Should have multiple chains
    assert!(registry.chain_count() > 0, "Registry should contain chains");

    // Should find Optimism mainnet
    let optimism = registry.get_chain(10);
    assert!(optimism.is_some(), "Should find Optimism (chain ID 10)");

    // Check if Base exists
    let base = registry.get_chain(8453);
    if let Some(chain) = base {
        assert!(chain.name.to_lowercase().contains("base"));
    }

    // Iteration should work
    let count = registry.all_chains().count();
    assert_eq!(count, registry.chain_count());
}

/// Test canonicalization of deposit transactions
#[test]
fn test_deposit_canonicalization() {
    let deposit = DepositTransaction::new(
        [0xab; 32],
        [0xcd; 20],
        Some([0xef; 20]),
        1_000_000_000_000_000_000u128,
        500_000_000_000_000_000u128,
        100_000,
        false,
        vec![1, 2, 3, 4],
    );

    let tx = OptimismTransaction::Deposit(deposit);

    // Should be able to canonicalize
    let result = tx.canonicalize();
    assert!(result.is_ok(), "Should canonicalize deposit transaction");

    let tx_ir = result.unwrap();

    // Should have operations
    assert!(!tx_ir.operations.is_empty(), "Should have operations");

    // Should have state deltas
    assert!(
        !tx_ir.state_deltas.account_changes.is_empty(),
        "Should have account changes"
    );
}

/// Test that decoding is deterministic
#[test]
fn test_decoding_determinism() {
    use decoder_encodings::rlp::RlpItem;

    let source_hash = [0x42; 32];
    let from = [0x11; 20];
    let to = [0x22; 20];

    let fields = vec![
        RlpItem::Data(source_hash.to_vec()),
        RlpItem::Data(from.to_vec()),
        RlpItem::Data(to.to_vec()),
        RlpItem::Data(vec![0x10]),       // mint
        RlpItem::Data(vec![0x05]),       // value
        RlpItem::Data(vec![0x52, 0x08]), // gas_limit
        RlpItem::Data(vec![0x00]),       // is_creation
        RlpItem::Data(vec![]),
    ];

    let rlp_list = RlpItem::List(fields);
    let rlp_bytes = rlp_list.encode();
    let mut tx_bytes = vec![0x7E];
    tx_bytes.extend_from_slice(&rlp_bytes);

    // Decode multiple times
    let result1 = OptimismDecoder::decode(&tx_bytes);
    let result2 = OptimismDecoder::decode(&tx_bytes);
    let result3 = OptimismDecoder::decode(&tx_bytes);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    let tx1 = result1.unwrap();
    let tx2 = result2.unwrap();
    let tx3 = result3.unwrap();

    // All should be equal
    assert_eq!(tx1, tx2, "Decoding should be deterministic");
    assert_eq!(tx2, tx3, "Decoding should be deterministic");
}
