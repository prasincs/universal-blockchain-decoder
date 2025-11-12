#!/usr/bin/env rust
//! Universal Blockchain Transaction Decoder CLI
//!
//! A unified command-line tool for decoding raw transactions from any blockchain.
//!
//! # Usage
//!
//! ```bash
//! # Decode Bitcoin transaction
//! cargo run --bin universal-tx-decoder -- --chain bitcoin <hex_string>
//!
//! # Decode Ethereum transaction
//! cargo run --bin universal-tx-decoder -- --chain ethereum <hex_string>
//!
//! # Decode from file
//! cargo run --bin universal-tx-decoder -- --chain bitcoin --file transaction.hex
//!
//! # Show canonical IR
//! cargo run --bin universal-tx-decoder -- --chain bitcoin --canonical <hex_string>
//! ```

use std::fs;
use std::io::{self, Read};
use universal_decoder_core::prelude::*;

// Import decoders
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let mut chain: Option<String> = None;
    let mut show_canonical = false;
    let mut tx_hex = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            "--chain" => {
                if i + 1 < args.len() {
                    i += 1;
                    chain = Some(args[i].to_lowercase());
                } else {
                    return Err("--chain requires a chain name".into());
                }
            }
            "--canonical" | "-c" => {
                show_canonical = true;
            }
            "--file" | "-f" => {
                if i + 1 < args.len() {
                    i += 1;
                    tx_hex = fs::read_to_string(&args[i])?;
                } else {
                    return Err("--file requires a filename".into());
                }
            }
            "--stdin" => {
                io::stdin().read_to_string(&mut tx_hex)?;
            }
            arg if !arg.starts_with("--") && !arg.starts_with("-") => {
                tx_hex = arg.to_string();
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]).into());
            }
        }
        i += 1;
    }

    let chain = chain.ok_or("Missing required --chain option")?;

    if tx_hex.is_empty() {
        return Err("No transaction hex provided".into());
    }

    // Decode hex string
    let tx_hex = tx_hex.trim();
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex)
        .map_err(|e| format!("Failed to decode hex: {}", e))?;

    println!("=== Universal Blockchain Transaction Decoder ===\n");
    println!("Chain:                  {}", chain.to_uppercase());
    println!("Raw transaction size:   {} bytes", tx_bytes.len());
    println!("Hex preview:            {}...\n", &tx_hex[..tx_hex.len().min(64)]);

    // Decode based on chain
    match chain.as_str() {
        "bitcoin" | "btc" => {
            let decoded = BitcoinDecoder::decode(&tx_bytes)?;
            print_bitcoin_transaction(&decoded);

            if show_canonical {
                let tx_ir = decoded.canonicalize()?;
                print_canonical_ir(&tx_ir)?;
            }
        }
        "ethereum" | "eth" => {
            // Ethereum support coming soon
            return Err("Ethereum decoder support coming soon!".into());
        }
        _ => {
            return Err(format!(
                "Unsupported chain: {}. Supported: bitcoin",
                chain
            )
            .into());
        }
    }

    Ok(())
}

fn print_usage() {
    println!(
        r#"
Universal Blockchain Transaction Decoder

USAGE:
    universal-tx-decoder --chain <CHAIN> [OPTIONS] <HEX_STRING>
    universal-tx-decoder --chain <CHAIN> [OPTIONS] --file <FILE>
    universal-tx-decoder --chain <CHAIN> [OPTIONS] --stdin

REQUIRED:
    --chain <CHAIN>     Blockchain to decode (bitcoin, ethereum)

OPTIONS:
    -h, --help          Show this help message
    -c, --canonical     Show canonical IR representation
    -f, --file <FILE>   Read transaction from file
    --stdin             Read transaction from stdin

SUPPORTED CHAINS:
    bitcoin, btc        Bitcoin blockchain (✓ implemented)
    ethereum, eth       Ethereum blockchain (coming soon)

EXAMPLES:
    # Decode Bitcoin transaction
    universal-tx-decoder --chain bitcoin 0100000001...

    # Decode from file with canonical IR
    universal-tx-decoder --chain bitcoin --file tx.hex --canonical

    # Pipe from Bitcoin Core
    bitcoin-cli getrawtransaction <txid> | universal-tx-decoder --chain bitcoin --stdin
"#
    );
}

fn print_bitcoin_transaction(tx: &BitcoinTransaction) {
    println!("=== Bitcoin Transaction Details ===");
    println!("TXID:           {}", hex_string(&tx.txid()));
    println!("Version:        {}", tx.version);
    println!("Locktime:       {}", tx.locktime);
    println!("SegWit:         {}", tx.is_segwit());
    println!("Coinbase:       {}", tx.is_coinbase());
    println!();

    // Inputs
    println!("=== Inputs ({}) ===", tx.inputs.len());
    for (i, input) in tx.inputs.iter().enumerate() {
        println!("Input #{}:", i);
        println!("  Previous TXID:  {}", hex_string(&input.prev_hash));
        println!("  Output Index:   {}", input.prev_index);
        println!("  Script Length:  {} bytes", input.script_sig.len());
        println!("  Sequence:       0x{:08x}", input.sequence);
        if tx.is_segwit() && i < tx.witnesses.len() && !tx.witnesses[i].items.is_empty() {
            println!("  Witness Items:  {}", tx.witnesses[i].items.len());
        }
        println!();
    }

    // Outputs
    println!("=== Outputs ({}) ===", tx.outputs.len());
    for (i, output) in tx.outputs.iter().enumerate() {
        let btc_value = output.value as f64 / 100_000_000.0;
        println!("Output #{}:", i);
        println!("  Value:          {} satoshis ({:.8} BTC)", output.value, btc_value);
        println!("  Script Length:  {} bytes", output.script_pubkey.len());
        println!("  Script Type:    {}", guess_bitcoin_script_type(&output.script_pubkey));
        println!();
    }

    // Summary
    match tx.total_output_value() {
        Ok(total_output) => {
            let btc_total = total_output as f64 / 100_000_000.0;
            println!("=== Summary ===");
            println!("Total Output:   {} satoshis ({:.8} BTC)", total_output, btc_total);

            if !tx.is_coinbase() {
                println!("\nNote: Fee calculation requires input values from the blockchain");
            }
        }
        Err(_) => {
            println!("Warning: Total output value overflow");
        }
    }
}

fn print_canonical_ir<'a>(tx_ir: &TxIR<'a, 1>) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Canonical IR Representation ===");

    println!("Version:        {}", tx_ir.version());
    println!("Operations:     {}", tx_ir.operations.len());
    println!("State Deltas:   {} inputs, {} outputs",
        tx_ir.state_deltas.inputs.len(),
        tx_ir.state_deltas.outputs.len());

    // Canonical hash
    let canonical_hash = tx_ir.canonical_hash()?;
    println!("Canonical Hash: {}", hex_string(&canonical_hash));

    // Canonical bytes
    let canonical_bytes = tx_ir.to_canonical_bytes()?;
    println!("Canonical Size: {} bytes", canonical_bytes.len());

    // Show operations
    println!("\n=== Operations ===");
    for (i, op) in tx_ir.operations.iter().enumerate() {
        println!("Operation #{}:", i);
        match op {
            universal_decoder_core::ir::Operation::Transfer(transfer) => {
                println!("  Type:     Transfer");
                println!("  From:     {}", hex_string(&transfer.from.bytes));
                println!("  To:       {}", hex_string(&transfer.to.bytes));
                let display_value = if transfer.amount.decimals == 8 {
                    transfer.amount.value as f64 / 100_000_000.0
                } else {
                    transfer.amount.value as f64
                };
                println!("  Amount:   {} (decimals: {})", display_value, transfer.amount.decimals);
            }
            universal_decoder_core::ir::Operation::ContractCall(call) => {
                println!("  Type:     ContractCall");
                println!("  Contract: {}", hex_string(&call.contract.bytes));
                println!("  Method:   {}", hex_string(&call.method));
                println!("  Data Len: {} bytes", call.data.len());
            }
            universal_decoder_core::ir::Operation::ContractDeploy(deploy) => {
                println!("  Type:     ContractDeploy");
                println!("  Bytecode: {} bytes", deploy.bytecode.len());
            }
            universal_decoder_core::ir::Operation::Stake(stake) => {
                println!("  Type:     Stake");
                println!("  Validator: {}", hex_string(&stake.validator.bytes));
            }
            universal_decoder_core::ir::Operation::Generic(generic) => {
                println!("  Type:     Generic");
                println!("  Data:     {} bytes", generic.data.len());
            }
        }
        println!();
    }

    Ok(())
}

fn hex_string(bytes: &[u8]) -> String {
    universal_decoder_core::hex::encode(bytes)
}

fn guess_bitcoin_script_type(script: &[u8]) -> &'static str {
    match script.len() {
        25 if script.get(0..3) == Some(&[0x76, 0xa9, 0x14]) => "P2PKH",
        23 if script.get(0..2) == Some(&[0xa9, 0x14]) => "P2SH",
        22 if script.get(0..2) == Some(&[0x00, 0x14]) => "P2WPKH",
        34 if script.get(0..2) == Some(&[0x00, 0x20]) => "P2WSH",
        34 if script.get(0..2) == Some(&[0x51, 0x20]) => "P2TR (Taproot)",
        67 if script.last() == Some(&0xac) => "P2PK",
        _ if script.is_empty() => "Empty (Provably Unspendable)",
        _ => "Unknown/Custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_genesis_coinbase() {
        let tx_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
        let tx_bytes = universal_decoder_core::hex::decode(tx_hex).unwrap();
        let decoded = BitcoinDecoder::decode(&tx_bytes).unwrap();

        assert_eq!(decoded.version, 1);
        assert!(decoded.is_coinbase());
        assert_eq!(decoded.outputs.len(), 1);
        assert_eq!(decoded.outputs[0].value, 5_000_000_000);
    }

    #[test]
    fn test_script_type_detection() {
        // P2PKH script
        let p2pkh = vec![
            0x76, 0xa9, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xac,
        ];
        assert_eq!(guess_bitcoin_script_type(&p2pkh), "P2PKH");

        // P2SH script
        let p2sh = vec![
            0xa9, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x87,
        ];
        assert_eq!(guess_bitcoin_script_type(&p2sh), "P2SH");

        // P2WPKH script
        let p2wpkh = vec![
            0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(guess_bitcoin_script_type(&p2wpkh), "P2WPKH");
    }
}
