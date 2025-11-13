//! Decoder generator CLI tool
//!
//! **Purpose**: Bootstrap new decoder crates (ONE-TIME GENERATION)
//!
//! This tool generates initial decoder code from a config.
//! After generation, YOU OWN THE CODE. Never regenerate!
//!
//! Think of it like `cargo new` - it creates the scaffold, then you maintain it.

use anyhow::{Context, Result};
use decoder_generator::{generate_decoder, validate_spec};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "generate" => {
            if args.len() < 3 {
                eprintln!("Usage: decoder-gen generate <spec-file>");
                std::process::exit(1);
            }

            let spec_path = PathBuf::from(&args[2]);
            let output_dir = if args.len() > 3 {
                PathBuf::from(&args[3])
            } else {
                // Default: crates/decoder-<chain-name>/
                let stem = spec_path.file_stem().unwrap().to_str().unwrap();
                PathBuf::from(format!("crates/decoder-{}", stem))
            };

            // Check if already exists
            if output_dir.exists() {
                print_warning_already_exists(&output_dir);
                if !confirm_overwrite()? {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            generate_decoder(&spec_path, &output_dir)?;
            print_post_generation_instructions(&output_dir);
        }
        "validate" => {
            if args.len() < 3 {
                eprintln!("Usage: decoder-gen validate <spec-file>");
                std::process::exit(1);
            }

            let spec_path = PathBuf::from(&args[2]);
            validate_spec(&spec_path)?;
        }
        "interactive" => {
            interactive_mode()?;
        }
        "new" => {
            // Quick generation from CLI args
            quick_generate_from_args(&args[2..])?;
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!(
        r#"Decoder Generator - Bootstrap new blockchain decoders

⚠️  ONE-TIME GENERATION ONLY
    After generation, YOU OWN THE CODE. This tool never runs again.
    Think of it like `cargo new` - creates scaffold, you maintain it.

USAGE:
    decoder-gen <COMMAND> [OPTIONS]

COMMANDS:
    generate <spec-file> [output-dir]    Generate from TOML spec
    new <name> --chain-id <id> ...       Quick CLI-based generation
    interactive                          Interactive mode (recommended)
    validate <spec-file>                 Validate a spec file

EXAMPLES:
    # Generate from TOML (one-time only!)
    decoder-gen generate specs/dogecoin.toml

    # Quick generation from CLI
    decoder-gen new litecoin --chain-id 2 --family utxo --hash double-sha256

    # Interactive mode (asks questions)
    decoder-gen interactive

    # Validate a spec before generating
    decoder-gen validate specs/example.toml

AFTER GENERATION:
    1. Edit the generated code (it's yours now)
    2. Add your parsing logic
    3. Write tests
    4. NEVER regenerate (you'll lose changes)

RECOMMENDED WORKFLOW:
    1. Use this tool to create initial scaffold
    2. Implement actual parsing logic
    3. Test against real blockchain data
    4. Commit to git (code is source of truth)
    5. Delete the spec file (it's just documentation now)
"#
    );
}

fn print_warning_already_exists(path: &Path) {
    eprintln!("\n⚠️  WARNING: {} already exists", path.display());
    eprintln!("    Regenerating will OVERWRITE your changes!");
    eprintln!("    This tool is for ONE-TIME generation only.\n");
}

fn confirm_overwrite() -> Result<bool> {
    print!("Do you want to overwrite? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn print_post_generation_instructions(output_dir: &Path) {
    println!("\n✅ Generated decoder at {}", output_dir.display());
    println!("\n📋 NEXT STEPS:");
    println!("   1. cd {}", output_dir.display());
    println!("   2. Edit src/parsing.rs (add your parsing logic)");
    println!("   3. Edit src/types.rs (implement canonicalize)");
    println!("   4. Run: cargo test");
    println!("   5. Add real test fixtures to tests/fixtures/");
    println!("\n⚠️  IMPORTANT:");
    println!("   - This was ONE-TIME generation");
    println!("   - YOU OWN THIS CODE NOW");
    println!("   - NEVER regenerate (you'll lose changes)");
    println!("   - Code is the source of truth, not the spec");
    println!("\n💡 TIP:");
    println!("   If you need to add another similar chain, COPY this decoder");
    println!("   and modify it. Don't use the generator again.");
    println!();
}

fn interactive_mode() -> Result<()> {
    println!("🚀 Interactive Decoder Generator\n");

    // Ask questions
    let chain_name = prompt("Chain name (e.g., Litecoin): ")?;
    let chain_id = prompt("Chain ID (number): ")?
        .parse::<u64>()
        .context("Chain ID must be a number")?;
    let family = prompt("Family [utxo/account/instruction]: ")?;
    let hash = prompt("Hash algorithm [sha256/double-sha256/keccak256]: ")?;
    let endianness = prompt("Endianness [little/big]: ")?;

    println!("\n📝 Generating with:");
    println!("   Name: {}", chain_name);
    println!("   ID: {}", chain_id);
    println!("   Family: {}", family);
    println!("   Hash: {}", hash);
    println!("   Endianness: {}", endianness);
    println!();

    // TODO: Actually generate from these params
    println!("⚠️  Interactive mode not fully implemented yet.");
    println!("    For now, create a TOML spec and use `generate` command.");

    Ok(())
}

fn quick_generate_from_args(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!("Usage: decoder-gen new <name> --chain-id <id> --family <family> --hash <hash>");
        std::process::exit(1);
    }

    let chain_name = &args[0];
    println!("Quick generation for {}", chain_name);
    println!("⚠️  CLI mode not fully implemented yet.");
    println!("    For now, use interactive mode or TOML spec.");

    Ok(())
}

fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
