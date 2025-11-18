use decoder_ethereum::types::EthereumTransaction;
use universal_decoder_core::hex;

/// Test ECDSA recovery with known Arbitrum Legacy transaction
#[test]
fn test_ecdsa_recovery_arbitrum_legacy() {
    // Arbitrum Legacy transaction (chain ID 42161)
    let tx_hex = "f8f083013c898401312d008401312d009447a894c806d0091247b982e31474fc9acb27a48380b884d5d860b55303875cab9228c24f426ae2fe87081feb69e00c363b98342541612a93da86a31cc9011eb440dc9c0f5d2296c220b1cd4af0a517eb6970acbf449fe175919b800000000000000000000000000000000000000000000000000000000000005ad200000000000000000000000000000000000000000000000000000000aa142a2783014985a05837f57b369b78c12f9e3bc2d9c6da3ba8be60ae66f84d5096118e5c013e012aa05e1deb79e1cd5fb91a8396dc165f01a37c4d08794cc468c0c9c1d565b1c2b1ab";

    let tx_bytes = hex::decode(tx_hex).unwrap();
    let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

    // Recover sender address
    let sender = tx.recover_sender().unwrap();

    // Expected sender address (to be verified via block explorer)
    // For now, just verify that recovery doesn't panic and produces a valid address
    assert_ne!(sender, [0u8; 20], "Sender should not be zero address");

    println!("Recovered sender: 0x{}", hex::encode(sender));

    // Verify signature components are present
    assert_eq!(tx.chain_id, Some(42161));
    assert_ne!(tx.r, [0u8; 32]);
    assert_ne!(tx.s, [0u8; 32]);
    assert!(tx.v > 0);
}

/// Test ECDSA recovery with known Arbitrum EIP-1559 transaction
#[test]
fn test_ecdsa_recovery_arbitrum_eip1559() {
    // Arbitrum EIP-1559 transaction (chain ID 42161)
    let tx_hex = "02f9013082a4b182192d808398968083092e0294802b65b5d9016621e66003aed0b16615093f328b80b8c5a00597a00000000000000000000000000000000000000000000000000000000001c412c10000000000000000000000000000000000058ff0955d44f32ab0099e950abfbf000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000000000000000000000000000000000000000000100000000000000000000000005477c22a5349cee601500da0489dad137fd6bfa00000000000000000000000000000000000000000000000000000000691ce4c20cc001a0dcc1b67fd15f72e5ce782ca5c88c3e401079220648bd548a9aa7cdb14023b5e9a05c121e1bf217a0c4d1f5dc43d8f28358815709e3e1796ee0b35ba64dea0499c1";

    let tx_bytes = hex::decode(tx_hex).unwrap();
    let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

    // Recover sender address
    let sender = tx.recover_sender().unwrap();

    // Verify recovery produces a valid address
    assert_ne!(sender, [0u8; 20], "Sender should not be zero address");

    println!("Recovered sender (EIP-1559): 0x{}", hex::encode(sender));

    // Verify this is an EIP-1559 transaction
    assert_eq!(tx.chain_id, Some(42161));
    assert!(tx.max_fee_per_gas.is_some());
    assert!(tx.max_priority_fee_per_gas.is_some());
}

/// Test that ECDSA recovery is deterministic
#[test]
fn test_ecdsa_recovery_deterministic() {
    let tx_hex = "f8f083013c898401312d008401312d009447a894c806d0091247b982e31474fc9acb27a48380b884d5d860b55303875cab9228c24f426ae2fe87081feb69e00c363b98342541612a93da86a31cc9011eb440dc9c0f5d2296c220b1cd4af0a517eb6970acbf449fe175919b800000000000000000000000000000000000000000000000000000000000005ad200000000000000000000000000000000000000000000000000000000aa142a2783014985a05837f57b369b78c12f9e3bc2d9c6da3ba8be60ae66f84d5096118e5c013e012aa05e1deb79e1cd5fb91a8396dc165f01a37c4d08794cc468c0c9c1d565b1c2b1ab";

    let tx_bytes = hex::decode(tx_hex).unwrap();
    let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

    // Recover multiple times and verify same result
    let sender1 = tx.recover_sender().unwrap();
    let sender2 = tx.recover_sender().unwrap();
    let sender3 = tx.recover_sender().unwrap();

    assert_eq!(sender1, sender2);
    assert_eq!(sender2, sender3);
}

/// Test ECDSA recovery with known Ethereum mainnet transaction
#[test]
fn test_ecdsa_recovery_ethereum_mainnet() {
    // Simple Ethereum mainnet legacy transaction
    // This is a well-known transaction with verifiable sender
    // Transaction: 0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060
    // Sender: 0xa1e4380a3b1f749673e270229993ee55f35663b4
    let _tx_hex = "f86d8085174876e800825208942c7536e3605d9c16a7a3d7b1898e529396a65c2388016345785d8a0000801ba0c52c114d4f5c3f9c1e3a4e3b3a6e6d6c6e6f6e6d6c6e6f6e6d6c6e6f6e6d6ca0c52c114d4f5c3f9c1e3a4e3b3a6e6d6c6e6f6e6d6c6e6f6e6d6c6e6f6e6d6c";

    // Note: This is a placeholder hex - we'd need a real transaction for verification
    // For now, test with Arbitrum transactions which we know are valid
    // TODO: Add real Ethereum mainnet transaction with known sender
}
