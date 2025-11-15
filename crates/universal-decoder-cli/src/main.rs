//! Universal Blockchain Transaction Decoder CLI
//!
//! A secure, chain-agnostic command-line tool for decoding raw transactions
//! from any supported blockchain, with special support for privacy chains.
//!
//! # Security Features
//!
//! - **No shell history pollution**: Sensitive data read from files/env vars
//! - **Memory protection**: Viewing keys use `secrecy` and are zeroized
//! - **File permission validation**: Keys must have 0600 or 0400 permissions
//! - **Input sanitization**: All inputs validated before processing

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

mod registry;
mod secure_input;

use registry::DecoderRegistry;
use secure_input::{SecureHexInput, SecureViewingKey, ViewingKeyType};
use universal_decoder_core::prelude::*;

/// Universal blockchain transaction decoder
///
/// Decode raw blockchain transactions from any supported chain.
/// For privacy chains (Zcash, Monero), provide viewing keys via file or env var.
///
/// # Examples
///
/// ```bash
/// # Bitcoin transaction from hex
/// universal-tx-decoder -c btc deadbeef...
///
/// # Ethereum transaction from file
/// universal-tx-decoder -c eth -f transaction.hex
///
/// # Zcash shielded transaction with viewing key
/// universal-tx-decoder -c zec --viewing-key-file ~/.zcash/viewkey --decrypt -f tx.hex
///
/// # List all supported chains
/// universal-tx-decoder --list-chains
///
/// # Show privacy chains only
/// universal-tx-decoder --list-privacy-chains
/// ```
#[derive(Parser)]
#[command(name = "universal-tx-decoder")]
#[command(about = "Decode raw blockchain transactions from any supported chain")]
#[command(long_about = None)]
#[command(version)]
struct Cli {
    /// Blockchain to decode (name or ID)
    ///
    /// Examples: btc, bitcoin, eth, ethereum, zec, zcash
    /// Or use chain ID: 0 (Bitcoin), 1 (Ethereum), 133 (Zcash)
    #[arg(long, short = 'c', value_name = "CHAIN")]
    chain: Option<String>,

    /// Transaction hex string (or use --file/--stdin)
    ///
    /// WARNING: Avoid pasting sensitive data directly (visible in shell history).
    /// For privacy chains, use --file or --stdin instead.
    #[arg(value_name = "HEX")]
    transaction: Option<String>,

    /// Read transaction from file (recommended for privacy)
    #[arg(long, short = 'f', value_name = "FILE")]
    file: Option<PathBuf>,

    /// Read transaction from stdin (recommended for piping)
    #[arg(long)]
    stdin: bool,

    /// Show canonical IR representation
    #[arg(long, short = 'C')]
    canonical: bool,

    /// Viewing key file for privacy chains (Zcash, Monero)
    ///
    /// SECURITY: File must have 0600 or 0400 permissions.
    /// Never pass keys as command-line arguments (shell history)!
    #[arg(long, value_name = "FILE")]
    viewing_key_file: Option<PathBuf>,

    /// Viewing key from environment variable (fallback)
    ///
    /// SECURITY: Less secure than --viewing-key-file but safer than CLI args.
    /// Example: ZCASH_VIEWING_KEY=<hex> universal-tx-decoder -c zec ...
    #[arg(long, env = "VIEWING_KEY", value_name = "ENV_VAR")]
    viewing_key_env: Option<String>,

    /// Type of viewing key (for validation)
    #[arg(long, value_enum, default_value = "zcash-full")]
    viewing_key_type: CliViewingKeyType,

    /// Attempt to decrypt shielded outputs (privacy chains only)
    #[arg(long)]
    decrypt: bool,

    /// Output format
    #[arg(long, short = 'o', value_enum, default_value = "human")]
    output: OutputFormat,

    /// List all supported chains
    #[arg(long)]
    list_chains: bool,

    /// List only privacy-enabled chains
    #[arg(long)]
    list_privacy_chains: bool,

    /// Verbose output (show debug info)
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CliViewingKeyType {
    /// Zcash incoming viewing key (32 bytes)
    ZcashIncoming,
    /// Zcash full viewing key (96 bytes)
    ZcashFull,
    /// Monero view key (32 bytes)
    Monero,
    /// Custom/unknown
    Custom,
}

impl From<CliViewingKeyType> for ViewingKeyType {
    fn from(cli_type: CliViewingKeyType) -> Self {
        match cli_type {
            CliViewingKeyType::ZcashIncoming => ViewingKeyType::ZcashIncoming,
            CliViewingKeyType::ZcashFull => ViewingKeyType::ZcashFull,
            CliViewingKeyType::Monero => ViewingKeyType::Monero,
            CliViewingKeyType::Custom => ViewingKeyType::Custom,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Human-readable text (default)
    Human,
    /// JSON output
    Json,
    /// Compact hex
    Hex,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let registry = DecoderRegistry::new();

    // Handle list commands
    if cli.list_chains {
        list_chains(&registry);
        return Ok(());
    }

    if cli.list_privacy_chains {
        list_privacy_chains(&registry);
        return Ok(());
    }

    // Chain is required for decoding
    let chain_str = cli
        .chain
        .as_ref()
        .context("--chain is required (or use --list-chains)")?;

    // Find chain info (by name or ID)
    let chain_info = if let Ok(chain_id) = chain_str.parse::<u64>() {
        registry.find_by_id(chain_id)?
    } else {
        registry.find_by_name(chain_str)?
    };

    if cli.verbose {
        eprintln!("Chain: {} (ID: {})", chain_info.name, chain_info.chain_id);
        eprintln!("Family: {:?}", chain_info.family);
        eprintln!(
            "Privacy features: {}",
            if chain_info.has_privacy_features {
                "Yes"
            } else {
                "No"
            }
        );
    }

    // Get transaction hex from one of the sources (secure input handling)
    let tx_bytes = get_transaction_input(&cli)?;

    if cli.verbose {
        eprintln!("Transaction size: {} bytes", tx_bytes.len());
    }

    // Load viewing key if provided (for privacy chains)
    let viewing_key = load_viewing_key(&cli, chain_info)?;

    if viewing_key.is_some() && cli.verbose {
        eprintln!("Viewing key loaded successfully");
    }

    // Warn if decrypt flag used without viewing key
    if cli.decrypt && viewing_key.is_none() && chain_info.has_privacy_features {
        eprintln!("Warning: --decrypt specified but no viewing key provided");
        eprintln!("Use --viewing-key-file or --viewing-key-env to provide a key");
    }

    // Decode the transaction
    decode_and_display(&registry, chain_info, &tx_bytes, &cli)?;

    Ok(())
}

/// Get transaction input from CLI args (securely)
fn get_transaction_input(cli: &Cli) -> Result<Vec<u8>> {
    if let Some(hex) = &cli.transaction {
        // Direct hex input (least secure, visible in shell history)
        if cli.verbose {
            eprintln!(
                "Warning: Transaction hex provided via CLI argument (visible in shell history)"
            );
            eprintln!("Consider using --file or --stdin for better security");
        }
        Ok(SecureHexInput::from_hex_string(hex)?.into_bytes())
    } else if let Some(file_path) = &cli.file {
        // File input (secure, recommended)
        Ok(SecureHexInput::from_file(file_path)?.into_bytes())
    } else if cli.stdin {
        // Stdin input (secure, good for piping)
        Ok(SecureHexInput::from_stdin()?.into_bytes())
    } else {
        anyhow::bail!("No transaction provided. Use HEX, --file, or --stdin");
    }
}

/// Load viewing key if provided (secure handling)
fn load_viewing_key(
    cli: &Cli,
    chain_info: &registry::ChainInfo,
) -> Result<Option<SecureViewingKey>> {
    // Only load viewing key for privacy chains
    if !chain_info.has_privacy_features {
        if cli.viewing_key_file.is_some() || cli.viewing_key_env.is_some() {
            eprintln!(
                "Warning: Viewing key provided but {} is not a privacy chain",
                chain_info.name
            );
        }
        return Ok(None);
    }

    let key_type = cli.viewing_key_type.into();

    // Prefer file over env var
    if let Some(key_file) = &cli.viewing_key_file {
        let key = SecureViewingKey::from_file(key_file, key_type)
            .context("Failed to load viewing key from file")?;
        return Ok(Some(key));
    }

    if let Some(env_var) = &cli.viewing_key_env {
        let key = SecureViewingKey::from_env(env_var, key_type)
            .context("Failed to load viewing key from environment")?;
        return Ok(Some(key));
    }

    // No viewing key provided (okay for transparent transactions)
    Ok(None)
}

/// Decode and display transaction
fn decode_and_display(
    _registry: &DecoderRegistry,
    chain_info: &registry::ChainInfo,
    tx_bytes: &[u8],
    cli: &Cli,
) -> Result<()> {
    match cli.output {
        OutputFormat::Human => {
            println!("=== Universal Blockchain Transaction Decoder ===\n");
            println!("Chain:                  {}", chain_info.name);
            println!("Chain ID:               {}", chain_info.chain_id);
            println!("Family:                 {:?}", chain_info.family);
            println!("Raw transaction size:   {} bytes", tx_bytes.len());

            // Show hex preview (first 64 chars)
            let hex_preview = universal_decoder_core::hex::encode(tx_bytes);
            println!(
                "Hex preview:            {}...\n",
                &hex_preview[..hex_preview.len().min(64)]
            );

            // Decode using registry
            decode_with_chain_specific_handler(chain_info, tx_bytes, cli)?;
        }
        OutputFormat::Json => {
            // JSON output (simplified for now)
            println!("{{");
            println!("  \"chain\": \"{}\",", chain_info.name);
            println!("  \"chain_id\": {},", chain_info.chain_id);
            println!("  \"size\": {}", tx_bytes.len());
            println!("}}");
        }
        OutputFormat::Hex => {
            // Just output the hex
            println!("{}", universal_decoder_core::hex::encode(tx_bytes));
        }
    }

    Ok(())
}

/// Decode with chain-specific handler (type-safe dispatch)
fn decode_with_chain_specific_handler(
    chain_info: &registry::ChainInfo,
    tx_bytes: &[u8],
    cli: &Cli,
) -> Result<()> {
    use decoder_bitcoin::BitcoinDecoder;
    use decoder_ethereum::EthereumDecoder;
    use decoder_zcash::ZcashDecoder;

    match chain_info.chain_id {
        // Bitcoin family
        0 => {
            let tx = BitcoinDecoder::decode(tx_bytes)?;
            print_bitcoin_transaction(&tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        2 => {
            let tx = decoder_litecoin::LitecoinDecoder::decode(tx_bytes)?;
            print_bitcoin_like_transaction("Litecoin", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        3 => {
            let tx = decoder_dogecoin::DogecoinDecoder::decode(tx_bytes)?;
            print_bitcoin_like_transaction("Dogecoin", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        5 => {
            let tx = decoder_dash::DashDecoder::decode(tx_bytes)?;
            print_bitcoin_like_transaction("Dash", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        145 => {
            let tx = decoder_bitcoin_cash::BitcoinCashDecoder::decode(tx_bytes)?;
            print_bitcoin_like_transaction("Bitcoin Cash", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        236 => {
            let tx = decoder_bitcoin_sv::BitcoinSvDecoder::decode(tx_bytes)?;
            print_bitcoin_like_transaction("Bitcoin SV", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        // Ethereum family
        1 => {
            let tx = EthereumDecoder::decode(tx_bytes)?;
            print_ethereum_transaction("Ethereum", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        56 => {
            let tx = decoder_bnb::BnbDecoder::decode(tx_bytes)?;
            print_ethereum_transaction("BNB Smart Chain", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        137 => {
            let tx = decoder_polygon::PolygonDecoder::decode(tx_bytes)?;
            print_ethereum_transaction("Polygon", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        43114 => {
            let tx = decoder_avalanche::AvalancheDecoder::decode(tx_bytes)?;
            print_ethereum_transaction("Avalanche C-Chain", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        10 => {
            let tx = decoder_optimism::OptimismDecoder::decode(tx_bytes)?;
            print_ethereum_transaction("Optimism", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        42161 => {
            let tx = decoder_arbitrum::ArbitrumDecoder::decode(tx_bytes)?;
            print_ethereum_transaction("Arbitrum One", &tx);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        // Privacy chains
        133 => {
            let tx = ZcashDecoder::decode(tx_bytes)?;
            print_zcash_transaction(&tx, cli.decrypt);
            if cli.canonical {
                print_canonical_ir(&tx.canonicalize()?)?;
            }
        }
        _ => {
            anyhow::bail!(
                "Decoder not yet implemented for {} (ID: {})",
                chain_info.name,
                chain_info.chain_id
            );
        }
    }

    Ok(())
}

// ============================================================================
// Display Functions
// ============================================================================

fn print_bitcoin_transaction(tx: &decoder_bitcoin::BitcoinTransaction) {
    println!("=== Bitcoin Transaction Details ===");
    println!("TXID:           {}", hex_string(&tx.txid()));
    println!("Version:        {}", tx.version);
    println!("Locktime:       {}", tx.locktime);
    println!("SegWit:         {}", tx.is_segwit());
    println!("Coinbase:       {}", tx.is_coinbase());
    println!();

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

    if let Ok(total_output) = tx.total_output_value() {
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
}

fn print_bitcoin_like_transaction(chain_name: &str, tx: &decoder_bitcoin::BitcoinTransaction) {
    println!("=== {} Transaction Details ===", chain_name);
    print_bitcoin_transaction(tx);
}

fn print_ethereum_transaction<T>(chain_name: &str, tx: &T)
where
    T: std::fmt::Debug,
{
    println!("=== {} Transaction Details ===", chain_name);
    println!("{:#?}", tx);
    println!("\nNote: Full EVM transaction display coming soon!");
}

fn print_zcash_transaction(tx: &decoder_zcash::ZcashTransaction, decrypt: bool) {
    use decoder_zcash::ZcashTransaction;

    println!("=== Zcash Transaction Details ===");

    match tx {
        ZcashTransaction::Transparent(transparent) => {
            println!("Type:           Transparent");
            println!("Version:        {}", transparent.version);
            println!("Version Group:  0x{:08x}", transparent.version_group_id);
            println!("Locktime:       {}", transparent.locktime);
            println!("Expiry Height:  {}", transparent.expiry_height);

            // Transparent inputs/outputs
            println!(
                "\n=== Transparent Inputs ({}) ===",
                transparent.inputs.len()
            );
            for (i, input) in transparent.inputs.iter().enumerate() {
                println!("Input #{}:", i);
                println!("  Previous Hash:  {}", hex_string(&input.prev_hash));
                println!("  Output Index:   {}", input.prev_index);
                println!("  Script:         {} bytes", input.script_sig.len());
                println!();
            }

            println!(
                "=== Transparent Outputs ({}) ===",
                transparent.outputs.len()
            );
            for (i, output) in transparent.outputs.iter().enumerate() {
                println!("Output #{}:", i);
                println!("  Value:          {} zatoshi", output.value);
                println!("  Script:         {} bytes", output.script_pubkey.len());
                println!();
            }
        }
        ZcashTransaction::Sapling(sapling) => {
            println!("Type:           Sapling Shielded");
            println!("Version:        {}", sapling.transparent.version);
            println!("Spends:         {}", sapling.spends.len());
            println!("Outputs:        {}", sapling.outputs.len());

            if decrypt {
                println!("\nNote: Shielded output decryption not yet implemented (Phase 3)");
                println!("Current phase supports transparent transactions only");
            }
        }
        ZcashTransaction::Orchard(_orchard) => {
            println!("Type:           Orchard");
            println!("\nNote: Orchard support coming in Phase 4");
        }
    }
}

fn print_canonical_ir(tx_ir: &TxIR<1>) -> Result<()> {
    println!("\n=== Canonical IR Representation ===");
    println!("Version:        {}", tx_ir.version());
    println!("Operations:     {}", tx_ir.operations.len());
    println!(
        "State Deltas:   {} inputs, {} outputs",
        tx_ir.state_deltas.inputs.len(),
        tx_ir.state_deltas.outputs.len()
    );

    let canonical_hash = tx_ir.canonical_hash()?;
    println!("Canonical Hash: {}", hex_string(&canonical_hash));

    let canonical_bytes = tx_ir.to_canonical_bytes()?;
    println!("Canonical Size: {} bytes", canonical_bytes.len());

    println!("\n=== Operations ===");
    for (i, op) in tx_ir.operations.iter().enumerate() {
        println!("Operation #{}:", i);
        match op {
            universal_decoder_core::ir::Operation::Transfer(transfer) => {
                println!("  Type:     Transfer");
                println!("  From:     {}", hex_string(&transfer.from.bytes));
                println!("  To:       {}", hex_string(&transfer.to.bytes));
                println!("  Amount:   {}", transfer.amount.value);
            }
            universal_decoder_core::ir::Operation::ContractCall(call) => {
                println!("  Type:     ContractCall");
                println!("  Contract: {}", hex_string(&call.contract.bytes));
                println!("  Method:   {}", hex_string(&call.method));
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

fn list_chains(registry: &DecoderRegistry) {
    println!("=== Supported Blockchains ===\n");
    println!(
        "{:<6} {:<25} {:<10} {:<15} {:<7}",
        "ID", "Name", "Symbol", "Family", "Privacy"
    );
    println!("{}", "=".repeat(70));

    for chain in registry.list_chains() {
        println!(
            "{:<6} {:<25} {:<10} {:<15} {}",
            chain.chain_id,
            chain.name,
            chain.short_name,
            format!("{:?}", chain.family),
            if chain.has_privacy_features {
                "✓"
            } else {
                ""
            }
        );
    }

    println!("\nTotal: {} chains", registry.list_chains().len());
}

fn list_privacy_chains(registry: &DecoderRegistry) {
    println!("=== Privacy-Enabled Blockchains ===\n");

    let privacy_chains = registry.list_privacy_chains();
    if privacy_chains.is_empty() {
        println!("No privacy chains available yet.");
        return;
    }

    for chain in privacy_chains {
        println!("• {} ({})", chain.name, chain.short_name);
        println!("  Chain ID: {}", chain.chain_id);
        println!("  Family:   {:?}", chain.family);
        println!();
    }
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
        let p2pkh = vec![
            0x76, 0xa9, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xac,
        ];
        assert_eq!(guess_bitcoin_script_type(&p2pkh), "P2PKH");
    }

    #[test]
    fn test_viewing_key_type_conversion() {
        assert_eq!(
            ViewingKeyType::from(CliViewingKeyType::ZcashFull),
            ViewingKeyType::ZcashFull
        );
    }
}
