//! Decoder generator CLI tool

use anyhow::Result;
use decoder_generator::{generate_decoder, validate_spec};
use std::path::PathBuf;

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

            generate_decoder(&spec_path, &output_dir)?;
        }
        "validate" => {
            if args.len() < 3 {
                eprintln!("Usage: decoder-gen validate <spec-file>");
                std::process::exit(1);
            }

            let spec_path = PathBuf::from(&args[2]);
            validate_spec(&spec_path)?;
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
        r#"Decoder Generator - Generate blockchain decoders from specs

USAGE:
    decoder-gen <COMMAND> [OPTIONS]

COMMANDS:
    generate <spec-file> [output-dir]    Generate decoder from specification
    validate <spec-file>                 Validate a decoder specification

EXAMPLES:
    # Generate Dogecoin decoder
    decoder-gen generate specs/dogecoin.toml

    # Generate to custom directory
    decoder-gen generate specs/dogecoin.toml /tmp/decoder-dogecoin

    # Validate a spec
    decoder-gen validate specs/dogecoin.toml
"#
    );
}
