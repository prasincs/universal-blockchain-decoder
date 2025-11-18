use decoder_ethereum::types::EthereumTransaction;
use universal_decoder_core::prelude::*;

#[test]
fn test_arbitrum_tx_txir_completeness() {
    // Arbitrum transaction with contract call
    let tx_hex = "f8f083013c898401312d008401312d009447a894c806d0091247b982e31474fc9acb27a48380b884d5d860b55303875cab9228c24f426ae2fe87081feb69e00c363b98342541612a93da86a31cc9011eb440dc9c0f5d2296c220b1cd4af0a517eb6970acbf449fe175919b800000000000000000000000000000000000000000000000000000000000005ad200000000000000000000000000000000000000000000000000000000aa142a2783014985a05837f57b369b78c12f9e3bc2d9c6da3ba8be60ae66f84d5096118e5c013e012aa05e1deb79e1cd5fb91a8396dc165f01a37c4d08794cc468c0c9c1d565b1c2b1ab";

    let tx_bytes = universal_decoder_core::hex::decode(tx_hex).unwrap();
    let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

    // Print what we decoded
    println!("\n=== Raw Transaction ===");
    println!("Type: {:?}", tx.tx_type);
    println!("Chain ID: {:?}", tx.chain_id);
    println!("Nonce: {}", tx.nonce);
    println!("Gas Price: {:?}", tx.gas_price);
    println!("Gas Limit: {}", tx.gas_limit);
    println!("To: {:?}", tx.to.map(universal_decoder_core::hex::encode));
    println!("Value: {}", tx.value);
    println!("Data length: {} bytes", tx.data.len());
    println!("Data: {}", universal_decoder_core::hex::encode(&tx.data));
    println!("V: {}", tx.v);
    println!("R: {}", universal_decoder_core::hex::encode(tx.r));
    println!("S: {}", universal_decoder_core::hex::encode(tx.s));

    // Canonicalize to TxIR
    let tx_ir = tx.canonicalize().unwrap();

    println!("\n=== TxIR ===");
    println!("Chain: {} (ID: {})", tx_ir.chain.name, tx_ir.chain.id);
    println!("Operations count: {}", tx_ir.operations.len());

    for (i, op) in tx_ir.operations.iter().enumerate() {
        match op {
            Operation::ContractCall(call) => {
                println!("\nOperation {}: ContractCall", i);
                println!("  Contract address: {:?}", call.contract.human_readable);
                println!(
                    "  Method selector (4 bytes): {}",
                    universal_decoder_core::hex::encode(&call.method)
                );
                println!("  Full data length: {} bytes", call.data.len());
                println!(
                    "  Full data: {}",
                    universal_decoder_core::hex::encode(&call.data)
                );
                println!("  Value: {:?}", call.value);
                println!("  Gas limit: {}", call.resource_limits.max_units);
                println!("  Gas price: {}", call.resource_limits.unit_price);
            }
            Operation::Transfer(transfer) => {
                println!("\nOperation {}: Transfer", i);
                println!("  From: {:?}", transfer.from.human_readable);
                println!("  To: {:?}", transfer.to.human_readable);
                println!("  Amount: {}", transfer.amount.value);
            }
            _ => println!("\nOperation {}: {:?}", i, op),
        }
    }

    println!("\n=== Metadata ===");
    println!("{}", tx_ir.metadata.extra);

    println!("\n=== Authorization ===");
    println!("Signatures: {}", tx_ir.authorization.signatures.len());
    for (i, sig) in tx_ir.authorization.signatures.iter().enumerate() {
        println!("  Signature {}: {} bytes", i, sig.data.len());
        println!(
            "    Data: {}",
            universal_decoder_core::hex::encode(&sig.data)
        );
        println!("    Metadata: {:?}", sig.metadata);
    }

    // Check if all data is preserved
    assert_eq!(tx.data.len(), 132); // Expected data length for this tx

    // Verify the operation contains the full data
    if let Operation::ContractCall(call) = &tx_ir.operations[0] {
        assert_eq!(
            call.data.len(),
            tx.data.len(),
            "Data length mismatch in TxIR"
        );
        assert_eq!(call.data, tx.data, "Data content mismatch in TxIR");
    } else {
        panic!("Expected ContractCall operation");
    }
}

#[test]
fn test_eip1559_tx_txir_completeness() {
    // EIP-1559 transaction
    let tx_hex = "02f9013082a4b182192d808398968083092e0294802b65b5d9016621e66003aed0b16615093f328b80b8c5a00597a00000000000000000000000000000000000000000000000000000000001c412c10000000000000000000000000000000000058ff0955d44f32ab0099e950abfbf000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000000000000000000000000000000000000000000100000000000000000000000005477c22a5349cee601500da0489dad137fd6bfa00000000000000000000000000000000000000000000000000000000691ce4c20cc001a0dcc1b67fd15f72e5ce782ca5c88c3e401079220648bd548a9aa7cdb14023b5e9a05c121e1bf217a0c4d1f5dc43d8f28358815709e3e1796ee0b35ba64dea0499c1";

    let tx_bytes = universal_decoder_core::hex::decode(tx_hex).unwrap();
    let tx = EthereumTransaction::from_raw_bytes(&tx_bytes).unwrap();

    println!("\n=== Raw Transaction ===");
    println!("Type: {:?}", tx.tx_type);
    println!("Chain ID: {:?}", tx.chain_id);
    println!("Nonce: {}", tx.nonce);
    println!("Max Fee Per Gas: {:?}", tx.max_fee_per_gas);
    println!(
        "Max Priority Fee Per Gas: {:?}",
        tx.max_priority_fee_per_gas
    );
    println!("Gas Limit: {}", tx.gas_limit);
    println!("To: {:?}", tx.to.map(universal_decoder_core::hex::encode));
    println!("Value: {}", tx.value);
    println!("Data length: {} bytes", tx.data.len());
    println!("Data: {}", universal_decoder_core::hex::encode(&tx.data));
    println!("Access list: {} items", tx.access_list.len());

    let tx_ir = tx.canonicalize().unwrap();

    println!("\n=== TxIR ===");
    println!("Chain: {} (ID: {})", tx_ir.chain.name, tx_ir.chain.id);

    for (i, op) in tx_ir.operations.iter().enumerate() {
        if let Operation::ContractCall(call) = op {
            println!("\nOperation {}: ContractCall", i);
            println!("  Contract: {:?}", call.contract.human_readable);
            println!(
                "  Method: {}",
                universal_decoder_core::hex::encode(&call.method)
            );
            println!("  Data length: {} bytes", call.data.len());
            println!(
                "  Data: {}",
                universal_decoder_core::hex::encode(&call.data)
            );
        }
    }

    println!("\n=== Metadata ===");
    println!("{}", tx_ir.metadata.extra);

    // Verify data preservation
    if let Operation::ContractCall(call) = &tx_ir.operations[0] {
        assert_eq!(call.data.len(), tx.data.len(), "Data length mismatch");
        assert_eq!(call.data, tx.data, "Data content mismatch");
    }
}
