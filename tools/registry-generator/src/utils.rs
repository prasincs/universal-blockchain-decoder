//! Shared utility functions for registry generation

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Detect upstream commit hash from VENDORED.md file
pub fn detect_upstream_commit(vendored_md_path: &str) -> Option<String> {
    if let Ok(contents) = fs::read_to_string(vendored_md_path) {
        for line in contents.lines() {
            if line.contains("Upstream Commit") {
                if let Some(commit) = line.split('`').nth(1) {
                    return Some(commit.to_string());
                }
            }
        }
    }
    None
}

/// Metadata information for registry generation
pub struct MetadataInfo<'a> {
    pub source_name: &'a str,
    pub source_repo: &'a str,
    pub chain_count: usize,
    pub byte_size: usize,
    pub commit: &'a str,
    pub verify_commands: Vec<String>,
}

/// Write metadata file with generation information
#[allow(clippy::too_many_arguments)]
pub fn write_metadata(path: &Path, info: &MetadataInfo) -> Result<()> {
    let generated_date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let verification_steps = info
        .verify_commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| format!("{}. {}", i + 1, cmd))
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        r#"Chain Registry Binary Data
=========================

Format: Borsh (Binary Object Representation Serializer for Hashing)
Source: {source_name}
Repository: {source_repo}
Upstream Commit: {commit}
Chain Count: {chain_count}
Generated: {generated_date}

File Size: {size_mb:.2} MB ({byte_size} bytes)

Verification:
{verification_steps}

This binary file is embedded at compile time via include_bytes!() and
deserialized on first use using borsh::BorshDeserialize.

Benefits:
- ✅ Compact: Borsh binary vs JSON (85-95% size reduction)
- ✅ Fast builds: No JSON parsing at build time
- ✅ Fast runtime: Deserialize once, cache forever
- ✅ Verifiable: Git commit hash provides full audit trail
- ✅ Airgapped: Compile-time embedding, zero runtime network I/O
"#,
        source_name = info.source_name,
        source_repo = info.source_repo,
        commit = info.commit,
        chain_count = info.chain_count,
        generated_date = generated_date,
        size_mb = info.byte_size as f64 / 1_048_576.0,
        byte_size = info.byte_size,
        verification_steps = verification_steps,
    );

    fs::write(path, content)
        .with_context(|| format!("Failed to write metadata file: {}", path.display()))?;

    Ok(())
}

/// Print generation summary
pub fn print_summary(
    registry_name: &str,
    chain_count: usize,
    byte_size: usize,
    commit: &str,
    output_path: &Path,
    test_command: &str,
) {
    println!();
    println!("✨ Generation complete!");
    println!();
    println!("Summary:");
    println!("  Registry: {}", registry_name);
    println!("  Chains: {}", chain_count);
    println!(
        "  Size: {} bytes ({:.2} MB)",
        byte_size,
        byte_size as f64 / 1_048_576.0
    );
    println!("  Commit: {}", commit);
    println!();
    println!("Next steps:");
    if let Some(parent) = output_path.parent() {
        println!("  1. Review: ls -lh {}", parent.display());
    }
    println!("  2. Test: {}", test_command);
    println!("  3. Commit: git add {}", output_path.display());
    println!();
}
