#!/usr/bin/env rust
//! AI-Based Refactoring Suggestion Tool
//!
//! This tool uses Claude API to analyze blockchain decoder implementations
//! and suggest refactorings based on:
//! 1. Latest blockchain protocol releases
//! 2. Rust ecosystem best practices
//! 3. Chain family-specific patterns
//! 4. Security and performance improvements
//!
//! Run weekly in CI to maintain code quality.

mod analyzer;
mod decoder_info;
mod information_fetcher;
mod prompts;
mod suggestions;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

use analyzer::RefactorAnalyzer;
use decoder_info::DecoderDiscovery;
use suggestions::ReportGenerator;

/// AI-based refactoring suggestion tool using Claude
#[derive(Parser, Debug)]
#[command(name = "ai-refactor-suggest")]
#[command(about = "Analyze decoders and suggest refactorings using Claude API", long_about = None)]
struct Args {
    /// Analyze specific decoder only (e.g., 'bitcoin', 'ethereum')
    #[arg(long)]
    decoder: Option<String>,

    /// Analyze specific chain family only (utxo, account, instruction, other)
    #[arg(long)]
    family: Option<String>,

    /// Output report path
    #[arg(long, default_value = "refactor-suggestions.md")]
    output: PathBuf,

    /// Directory to generate GitHub issue templates
    #[arg(long, default_value = "github-issues")]
    issues_dir: PathBuf,

    /// Don't generate GitHub issue templates
    #[arg(long)]
    no_issues: bool,

    /// Configuration file path
    #[arg(long, default_value = "scripts/refactor-config.json")]
    config: PathBuf,

    /// Repository root directory
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,

    /// Anthropic API key (can also be set via ANTHROPIC_API_KEY env var)
    #[arg(long)]
    api_key: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Validate API key (from argument or environment variable)
    let api_key = args
        .api_key
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .context(
            "ANTHROPIC_API_KEY must be set (via --api-key argument or environment variable)",
        )?;

    if args.verbose {
        println!("🔍 Universal Blockchain Decoder - AI Refactoring Suggestions");
        println!("Repository: {}", args.repo_root.display());
    }

    // Discover decoders
    let discovery = DecoderDiscovery::new(args.repo_root.clone());
    let mut decoders = discovery
        .discover()
        .context("Failed to discover decoders")?;

    println!("📦 Discovered {} decoders", decoders.len());

    // Filter decoders
    if let Some(decoder_name) = &args.decoder {
        decoders.retain(|d| d.name == *decoder_name);
        if decoders.is_empty() {
            anyhow::bail!("Decoder '{}' not found", decoder_name);
        }
    } else if let Some(family) = &args.family {
        decoders.retain(|d| d.family == *family);
        if decoders.is_empty() {
            anyhow::bail!("No decoders found for family '{}'", family);
        }
    }

    println!("🎯 Analyzing {} decoders...", decoders.len());

    // Initialize analyzer
    let analyzer = RefactorAnalyzer::new(api_key, args.config, args.repo_root.clone())
        .context("Failed to initialize analyzer")?;

    // Analyze each decoder
    let mut all_suggestions = Vec::new();
    for decoder in &decoders {
        if args.verbose {
            println!(
                "  Analyzing {} ({} family)...",
                decoder.name, decoder.family
            );
        } else {
            print!("  {} ... ", decoder.name);
        }

        match analyzer.analyze_decoder(decoder).await {
            Ok(suggestions) => {
                println!("✓ {} suggestions", suggestions.len());
                all_suggestions.extend(suggestions);
            }
            Err(e) => {
                eprintln!("✗ Error: {}", e);
            }
        }
    }

    // Generate report
    println!("\n📝 Generating report...");
    let report_gen = ReportGenerator::new();
    report_gen
        .generate_markdown_report(&all_suggestions, &args.output)
        .context("Failed to generate report")?;

    // Generate GitHub issues
    if !args.no_issues {
        println!("📋 Generating GitHub issue templates...");
        report_gen
            .generate_github_issues(&all_suggestions, &args.issues_dir)
            .context("Failed to generate GitHub issues")?;
    }

    // Summary
    let high_priority = all_suggestions
        .iter()
        .filter(|s| s.priority == "high")
        .count();
    let medium_priority = all_suggestions
        .iter()
        .filter(|s| s.priority == "medium")
        .count();
    let low_priority = all_suggestions
        .iter()
        .filter(|s| s.priority == "low")
        .count();

    println!("\n✅ Analysis complete!");
    println!("   Total suggestions: {}", all_suggestions.len());
    println!("   High priority: {}", high_priority);
    println!("   Medium priority: {}", medium_priority);
    println!("   Low priority: {}", low_priority);
    println!("   Report: {}", args.output.display());

    Ok(())
}
