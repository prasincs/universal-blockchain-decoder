//! Universal Blockchain Transaction Decoder CLI
//!
//! A unified command-line tool for decoding raw transactions from any blockchain.

use clap::{Parser, ValueEnum};
use std::fs;
use std::io::{self, Read};
use universal_decoder_core::prelude::*;

// Import decoders
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};

/// Universal blockchain transaction decoder
#[derive(Parser)]
#[command(name = "universal-tx-decoder")]
#[command(about = "Decode raw blockchain transactions from any supported chain", long_about = None)]
#[command(version)]
struct Cli {
    /// Blockchain to decode
    #[arg(long, short = 'c', value_enum)]
    chain: Chain,

    /// Transaction hex string (or use --file/--stdin)
    #[arg(value_name = "HEX")]
    transaction: Option<String>,

    /// Read transaction from file
    #[arg(long, short = 'f', value_name = "FILE")]
    file: Option<String>,

    /// Read transaction from stdin
    #[arg(long)]
    stdin: bool,

    /// Show canonical IR representation
    #[arg(long, short = 'C')]
    canonical: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Chain {
    /// Bitcoin blockchain
    Bitcoin,
    /// Ethereum blockchain
    Ethereum,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Get transaction hex from one of the sources
    let tx_hex = if let Some(hex) = cli.transaction {
        hex
    } else if let Some(file_path) = cli.file {
        fs::read_to_string(&file_path)?
    } else if cli.stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        anyhow::bail!("No transaction provided. Use HEX, --file, or --stdin");
    };

    // Decode hex string
    let tx_hex = tx_hex.trim();
    let tx_bytes = universal_decoder_core::hex::decode(tx_hex)
        .map_err(|e| anyhow::anyhow!("Failed to decode hex: {}", e))?;

    println!("=== Universal Blockchain Transaction Decoder ===\n");
    println!("Chain:                  {}", chain_name(cli.chain));
    println!("Raw transaction size:   {} bytes", tx_bytes.len());
    println!(
        "Hex preview:            {}...\n",
        &tx_hex[..tx_hex.len().min(64)]
    );

    // Decode based on chain
    match cli.chain {
        Chain::Bitcoin => {
            let decoded = BitcoinDecoder::decode(&tx_bytes)?;
            print_bitcoin_transaction(&decoded);

            if cli.canonical {
                let tx_ir = decoded.canonicalize()?;
                print_canonical_ir(&tx_ir)?;
            }
        }
        Chain::Ethereum => {
            anyhow::bail!("Ethereum decoder support coming soon!");
        }
    }

    Ok(())
}

fn chain_name(chain: Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "BITCOIN",
        Chain::Ethereum => "ETHEREUM",
    }
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
        println!(
            "  Value:          {} satoshis ({:.8} BTC)",
            output.value, btc_value
        );
        println!("  Script Length:  {} bytes", output.script_pubkey.len());
        println!(
            "  Script Type:    {}",
            guess_bitcoin_script_type(&output.script_pubkey)
        );
        println!();
    }

    // Summary
    match tx.total_output_value() {
        Ok(total_output) => {
            let btc_total = total_output as f64 / 100_000_000.0;
            println!("=== Summary ===");
            println!(
                "Total Output:   {} satoshis ({:.8} BTC)",
                total_output, btc_total
            );

            if !tx.is_coinbase() {
                println!("\nNote: Fee calculation requires input values from the blockchain");
            }
        }
        Err(_) => {
            println!("Warning: Total output value overflow");
        }
    }
}

fn print_canonical_ir<'a>(tx_ir: &TxIR<'a, 1>) -> anyhow::Result<()> {
    println!("\n=== Canonical IR Representation ===");

    println!("Version:        {}", tx_ir.version());
    println!("Operations:     {}", tx_ir.operations.len());
    println!(
        "State Deltas:   {} inputs, {} outputs",
        tx_ir.state_deltas.inputs.len(),
        tx_ir.state_deltas.outputs.len()
    );

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
                println!(
                    "  Amount:   {} (decimals: {})",
                    display_value, transfer.amount.decimals
                );
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

    #[test]
    fn test_chain_name() {
        assert_eq!(chain_name(Chain::Bitcoin), "BITCOIN");
        assert_eq!(chain_name(Chain::Ethereum), "ETHEREUM");
    }
}
