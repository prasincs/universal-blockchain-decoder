//! Registry Generator
//!
//! Unified tool to generate compact Borsh-serialized chain registries from
//! various upstream sources (ethereum-lists/chains, cosmos/chain-registry,
//! ethereum-optimism/superchain-registry).
//!
//! Usage:
//!   cargo run -p registry-generator -- evm
//!   cargo run -p registry-generator -- cosmos
//!   cargo run -p registry-generator -- superchain

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cosmos;
mod evm;
mod superchain;
mod utils;

#[derive(Parser)]
#[command(name = "registry-generator")]
#[command(about = "Generate compact Borsh-serialized chain registries")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate EVM chain registry (ethereum-lists/chains)
    Evm {
        /// Input directory containing chain JSON files
        #[arg(
            short,
            long,
            default_value = "crates/decoder-evm/vendored/chainlist/_data/chains"
        )]
        input: PathBuf,

        /// Output file for Borsh binary
        #[arg(short, long, default_value = "crates/decoder-evm/data/chains.borsh")]
        output: PathBuf,

        /// Output metadata file
        #[arg(
            short,
            long,
            default_value = "crates/decoder-evm/data/chains.metadata.txt"
        )]
        metadata: PathBuf,

        /// Upstream commit hash (optional, read from VENDORED.md if not provided)
        #[arg(long)]
        commit: Option<String>,
    },

    /// Generate Cosmos chain registry (cosmos/chain-registry)
    Cosmos {
        /// Input directory containing chain directories
        #[arg(
            short,
            long,
            default_value = "crates/decoder-cosmos/vendored/chain-registry"
        )]
        input: PathBuf,

        /// Output file for Borsh binary
        #[arg(
            short,
            long,
            default_value = "crates/decoder-cosmos/data/cosmos-chains.borsh"
        )]
        output: PathBuf,

        /// Output metadata file
        #[arg(
            short,
            long,
            default_value = "crates/decoder-cosmos/data/cosmos-chains.metadata.txt"
        )]
        metadata: PathBuf,

        /// Upstream commit hash (optional, read from VENDORED.md if not provided)
        #[arg(long)]
        commit: Option<String>,
    },

    /// Generate Superchain registry (ethereum-optimism/superchain-registry)
    Superchain {
        /// Input file (chainList.json)
        #[arg(
            short,
            long,
            default_value = "crates/decoder-optimism/vendored/superchain-registry/chainList.json"
        )]
        input: PathBuf,

        /// Output file for Borsh binary
        #[arg(
            short,
            long,
            default_value = "crates/decoder-optimism/data/op-chains.borsh"
        )]
        output: PathBuf,

        /// Output metadata file
        #[arg(
            short,
            long,
            default_value = "crates/decoder-optimism/data/op-chains.metadata.txt"
        )]
        metadata: PathBuf,

        /// Upstream commit hash (optional, read from VENDORED.md if not provided)
        #[arg(long)]
        commit: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Evm {
            input,
            output,
            metadata,
            commit,
        } => evm::generate_evm_registry(input, output, metadata, commit),

        Commands::Cosmos {
            input,
            output,
            metadata,
            commit,
        } => cosmos::generate_cosmos_registry(input, output, metadata, commit),

        Commands::Superchain {
            input,
            output,
            metadata,
            commit,
        } => superchain::generate_superchain_registry(input, output, metadata, commit),
    }
}
