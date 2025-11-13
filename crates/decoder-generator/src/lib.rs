//! Decoder Generator - Code generation for blockchain decoders
//!
//! This crate provides tooling to generate decoder implementations from
//! declarative specifications, eliminating boilerplate and ensuring consistency.
//!
//! ## Goals
//!
//! 1. **Eliminate copy-paste**: Generate decoder code from specs
//! 2. **Ensure correctness**: Generated code is tested in CI
//! 3. **Stay maintainable**: Update generator, regenerate all chains
//! 4. **Enable rapid addition**: Add new chain by writing minimal spec
//!
//! ## Architecture
//!
//! ```text
//! chain-spec.toml  ──┐
//!                    ├──> decoder-gen ──> Generated Rust code
//! templates/       ──┘                    ├── lib.rs
//!                                         ├── parsing.rs
//!                                         ├── types.rs
//!                                         └── tests/
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Generate a new decoder from spec
//! cargo run -p decoder-generator -- generate specs/dogecoin.toml
//!
//! # Regenerate all decoders
//! cargo run -p decoder-generator -- regenerate-all
//!
//! # Validate a spec
//! cargo run -p decoder-generator -- validate specs/dogecoin.toml
//! ```

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

mod codegen;
mod spec;
mod templates;

pub use spec::{ChainSpec, DecoderSpec};

/// Generate a decoder crate from a specification
pub fn generate_decoder(spec_path: &Path, output_dir: &Path) -> Result<()> {
    // Parse spec
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec: {}", spec_path.display()))?;

    let spec: DecoderSpec = toml::from_str(&spec_content)
        .with_context(|| format!("Failed to parse spec: {}", spec_path.display()))?;

    // Validate spec
    spec.validate()?;

    // Generate code
    let generated = codegen::generate(&spec)?;

    // Write files
    fs::create_dir_all(output_dir)?;

    for (file_path, content) in generated.files {
        let full_path = output_dir.join(&file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, content)
            .with_context(|| format!("Failed to write {}", full_path.display()))?;
    }

    println!("✓ Generated decoder at {}", output_dir.display());
    Ok(())
}

/// Validate a decoder specification
pub fn validate_spec(spec_path: &Path) -> Result<()> {
    let spec_content = fs::read_to_string(spec_path)?;
    let spec: DecoderSpec = toml::from_str(&spec_content)?;
    spec.validate()?;
    println!("✓ Spec is valid: {}", spec_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_decoder() {
        // This will test the full generation pipeline
    }
}
