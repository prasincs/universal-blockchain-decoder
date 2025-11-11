//! Simple decoder example demonstrating the universal blockchain decoder

use decoder_bitcoin::BitcoinDecoder;
use decoder_ethereum::EthereumDecoder;
use universal_decoder_core::prelude::*;

fn main() {
    println!("Universal Blockchain Decoder - Example\n");
    println!("========================================\n");

    // Set up hook registry
    let registry = HookRegistryBuilder::new()
        .with_size_limit(1_000_000) // 1MB max transaction size
        .with_logging("example".to_string(), vec![HookStage::PreDecode])
        .build();

    println!("Hook registry initialized with {} hooks\n", registry.len());

    // Example 1: Bitcoin transaction decoding
    println!("Example 1: Bitcoin Transaction");
    println!("-------------------------------");
    demo_bitcoin_decoder(&registry);
    println!();

    // Example 2: Ethereum transaction decoding
    println!("Example 2: Ethereum Transaction");
    println!("--------------------------------");
    demo_ethereum_decoder(&registry);
    println!();

    // Example 3: Custom hook demonstration
    println!("Example 3: Custom Hook");
    println!("----------------------");
    demo_custom_hook();
    println!();

    println!("========================================");
    println!("Examples completed successfully!");
}

fn demo_bitcoin_decoder(registry: &HookRegistry) {
    // Create a dummy Bitcoin transaction for demonstration
    // In production, this would be actual transaction bytes
    let dummy_tx_bytes = create_dummy_bitcoin_tx();

    println!("Decoding Bitcoin transaction ({} bytes)...", dummy_tx_bytes.len());

    match decoder_bitcoin::decode_with_hooks(&dummy_tx_bytes, registry) {
        Ok(tx) => {
            println!("✓ Successfully decoded Bitcoin transaction");
            println!("  - Version: {}", tx.version());
            println!("  - Inputs: {}", tx.input_count());
            println!("  - Outputs: {}", tx.output_count());
            println!("  - TXID: {}", hex::encode(&tx.txid()));

            // Canonicalize to TxIR
            match tx.canonicalize() {
                Ok(tx_ir) => {
                    println!("✓ Successfully canonicalized to TxIR");
                    println!("  - Chain: {:?}", tx_ir.chain_id);
                    println!("  - Operations: {}", tx_ir.operations.len());
                    println!("  - Inputs consumed: {}", tx_ir.state_deltas.inputs.len());
                    println!("  - Outputs created: {}", tx_ir.state_deltas.outputs.len());

                    // Serialize to JSON
                    if let Ok(json) = serde_json::to_string_pretty(&tx_ir) {
                        println!("\nCanonical IR (JSON):");
                        println!("{}", json);
                    }
                }
                Err(e) => println!("✗ Canonicalization failed: {}", e),
            }
        }
        Err(e) => println!("✗ Decoding failed: {}", e),
    }
}

fn demo_ethereum_decoder(registry: &HookRegistry) {
    // Create a dummy Ethereum transaction for demonstration
    let dummy_tx_bytes = create_dummy_ethereum_tx();

    println!("Decoding Ethereum transaction ({} bytes)...", dummy_tx_bytes.len());

    match decoder_ethereum::decode_with_hooks(&dummy_tx_bytes, registry) {
        Ok(tx) => {
            println!("✓ Successfully decoded Ethereum transaction");
            println!("  - Nonce: {}", tx.nonce);
            println!("  - Gas limit: {}", tx.gas_limit);
            println!("  - EIP-1559: {}", tx.is_eip1559());
            println!("  - Contract creation: {}", tx.is_contract_creation());
            println!("  - Hash: {}", hex::encode(&tx.hash()));

            // Canonicalize to TxIR
            match tx.canonicalize() {
                Ok(tx_ir) => {
                    println!("✓ Successfully canonicalized to TxIR");
                    println!("  - Chain: {:?}", tx_ir.chain_id);
                    println!("  - Operations: {}", tx_ir.operations.len());
                    println!("  - Account changes: {}", tx_ir.state_deltas.account_changes.len());
                }
                Err(e) => println!("✗ Canonicalization failed: {}", e),
            }
        }
        Err(e) => println!("✗ Decoding failed: {}", e),
    }
}

fn demo_custom_hook() {
    // Create a custom hook that validates transaction content
    struct ContentValidatorHook;

    impl Hook for ContentValidatorHook {
        fn name(&self) -> &str {
            "content_validator"
        }

        fn stages(&self) -> Vec<HookStage> {
            vec![HookStage::PreDecode]
        }

        fn execute(&self, context: &HookContext) -> Result<HookResult> {
            println!("  → Custom hook executing...");
            println!("    Stage: {:?}", context.stage);
            println!("    Raw bytes: {} bytes", context.raw_bytes.len());

            // Example validation: check for specific patterns
            if context.raw_bytes.len() > 500_000 {
                println!("    ✗ Transaction too large!");
                return Ok(HookResult::Abort("Transaction exceeds size limit".to_string()));
            }

            println!("    ✓ Validation passed");
            Ok(HookResult::Continue)
        }

        fn priority(&self) -> i32 {
            50
        }
    }

    let mut registry = HookRegistry::new();
    registry.register(ContentValidatorHook);

    println!("Custom hook registry created with 1 hook");

    let test_data = vec![0u8; 1000];
    let context = HookContext::new(HookStage::PreDecode, &test_data);

    match registry.execute_stage(&context) {
        Ok(result) => println!("✓ Hook execution result: {:?}", result),
        Err(e) => println!("✗ Hook execution failed: {}", e),
    }
}

// Helper functions to create dummy transactions for demonstration

fn create_dummy_bitcoin_tx() -> Vec<u8> {
    // This is a simplified Bitcoin transaction structure for demonstration
    // Version (4 bytes) + Input count (1 byte) + Input + Output count (1 byte) + Output + Locktime (4 bytes)
    vec![
        // Version
        0x01, 0x00, 0x00, 0x00,
        // Input count
        0x01,
        // Previous output hash (32 bytes)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // Previous output index (4 bytes)
        0xff, 0xff, 0xff, 0xff,
        // Script length
        0x00,
        // Sequence (4 bytes)
        0xff, 0xff, 0xff, 0xff,
        // Output count
        0x01,
        // Value (8 bytes)
        0x00, 0xe1, 0xf5, 0x05, 0x00, 0x00, 0x00, 0x00,
        // Script length
        0x00,
        // Locktime (4 bytes)
        0x00, 0x00, 0x00, 0x00,
    ]
}

fn create_dummy_ethereum_tx() -> Vec<u8> {
    // This is a simplified Ethereum transaction structure for demonstration
    // RLP-encoded legacy transaction
    vec![
        0xf8, 0x6c, // RLP list with length
        0x01, // Nonce
        0x85, 0x04, 0xa8, 0x17, 0xc8, 0x00, // Gas price
        0x82, 0x52, 0x08, // Gas limit
        0x94, // To address marker
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, // To address (20 bytes)
        0x85, 0x01, 0x00, 0x00, 0x00, 0x00, // Value
        0x80, // Data (empty)
        0x25, // V
        0xa0, // R marker
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // R (32 bytes)
        0xa0, // S marker
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // S (32 bytes)
    ]
}
