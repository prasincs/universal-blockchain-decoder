//! Code generation from decoder specifications

use crate::spec::{DecoderSpec, FieldSpec};
use anyhow::Result;
use std::collections::HashMap;

pub struct GeneratedCode {
    pub files: HashMap<String, String>,
}

/// Generate all code for a decoder from its specification
pub fn generate(spec: &DecoderSpec) -> Result<GeneratedCode> {
    let mut files = HashMap::new();

    // Generate Cargo.toml
    files.insert("Cargo.toml".to_string(), generate_cargo_toml(spec));

    // Generate lib.rs
    files.insert("src/lib.rs".to_string(), generate_lib_rs(spec));

    // Generate parsing.rs
    files.insert("src/parsing.rs".to_string(), generate_parsing_rs(spec));

    // Generate types.rs
    files.insert("src/types.rs".to_string(), generate_types_rs(spec));

    // Generate tests
    files.insert(
        "tests/property_tests.rs".to_string(),
        generate_property_tests(spec),
    );

    Ok(GeneratedCode { files })
}

fn generate_cargo_toml(spec: &DecoderSpec) -> String {
    let chain_lower = spec.chain.name.to_lowercase();
    format!(
        r#"[package]
name = "decoder-{chain_lower}"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "{chain_name} transaction decoder for universal-decoder"

[dependencies]
universal-decoder-core = {{ path = "../universal-decoder-core" }}
decoder-primitives = {{ path = "../decoder-primitives" }}
decoder-encodings = {{ path = "../decoder-encodings" }}
serde = {{ workspace = true }}
thiserror = {{ workspace = true }}
sha2 = {{ workspace = true }}  # TODO: Make conditional based on hash algo

[dev-dependencies]
serde_json = {{ workspace = true }}
proptest = {{ workspace = true }}
decoder-test-utils = {{ path = "../decoder-test-utils" }}
"#,
        chain_lower = chain_lower,
        chain_name = spec.chain.name
    )
}

fn generate_lib_rs(spec: &DecoderSpec) -> String {
    let chain_name = &spec.chain.name;
    let chain_id = spec.chain.chain_id;
    let chain_family = &spec.chain.family;
    let chain_struct = format!("{}Chain", chain_name);
    let tx_struct = format!("{}Transaction", chain_name);
    let decoder_struct = format!("{}Decoder", chain_name);

    format!(
        r#"//! {chain_name} transaction decoder - Generated code
//!
//! This decoder was automatically generated from a specification.
//! To regenerate: cargo run -p decoder-generator -- generate specs/{chain_lower}.toml

use decoder_primitives::prelude::*;
use std::io::Cursor;

pub mod parsing;
pub mod types;

use parsing::*;
pub use types::{tx_struct};

/// {chain_name} chain identity
#[derive(Debug, Clone, Copy)]
pub struct {chain_struct};

impl ChainIdentity for {chain_struct} {{
    fn chain_id(&self) -> u64 {{
        {chain_id}
    }}

    fn chain_name(&self) -> &str {{
        "{chain_name}"
    }}

    fn chain_family(&self) -> ChainFamily {{
        ChainFamily::{family_variant}
    }}
}}

/// {chain_name} decoder
pub struct {decoder_struct};

impl ChainDecoder for {decoder_struct} {{
    type TxSpecific = {tx_struct};
    type Chain = {chain_struct};

    fn chain() -> Self::Chain {{
        {chain_struct}
    }}

    fn decode(raw_bytes: &[u8]) -> Result<Self::TxSpecific> {{
        Self::validate_format(raw_bytes)?;

        let mut cursor = Cursor::new(raw_bytes);

        // TODO: Generate field parsing based on spec

        unimplemented!("Field parsing generation coming soon")
    }}

    fn validate_format(raw_bytes: &[u8]) -> Result<()> {{
        if raw_bytes.is_empty() {{
            return Err(DecoderError::invalid_structure(
                "{chain_name} transaction cannot be empty"
            ));
        }}
        Ok(())
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_chain_identity() {{
        let chain = {decoder_struct}::chain();
        assert_eq!(chain.chain_id(), {chain_id});
        assert_eq!(chain.chain_name(), "{chain_name}");
    }}
}}
"#,
        chain_name = chain_name,
        chain_lower = chain_name.to_lowercase(),
        chain_id = chain_id,
        chain_struct = chain_struct,
        tx_struct = tx_struct,
        decoder_struct = decoder_struct,
        family_variant = match chain_family.as_str() {
            "utxo" => "Utxo",
            "account" => "Account",
            "instruction" => "Instruction",
            _ => "Custom",
        }
    )
}

fn generate_parsing_rs(spec: &DecoderSpec) -> String {
    format!(
        r#"//! Pure Rust {chain_name} transaction parsing utilities
//!
//! Generated from specification.

use decoder_primitives::prelude::*;
use std::io::Read;

// TODO: Generate parsing functions based on spec fields

pub const MAX_TRANSACTION_SIZE: usize = 100_000;
"#,
        chain_name = spec.chain.name
    )
}

fn generate_types_rs(spec: &DecoderSpec) -> String {
    let tx_struct = format!("{}Transaction", spec.chain.name);

    format!(
        r#"//! {chain_name}-specific transaction types
//!
//! Generated from specification.

use universal_decoder_core::prelude::*;

/// {chain_name} transaction representation
#[derive(Debug, Clone)]
pub struct {tx_struct} {{
    pub raw_bytes: Vec<u8>,
    // TODO: Generate fields based on spec
}}

impl {tx_struct} {{
    pub fn txid(&self) -> Vec<u8> {{
        // TODO: Generate based on hash spec
        use sha2::{{Digest, Sha256}};
        Sha256::digest(&self.raw_bytes).to_vec()
    }}
}}

impl<'a> Canonicalizer<'a> for {tx_struct} {{
    const VERSION: u8 = 1;

    fn canonicalize(&'a self) -> Result<TxIR<'a, 1>> {{
        // TODO: Generate based on spec
        unimplemented!("Canonicalization generation coming soon")
    }}

    fn validate(&self) -> Result<()> {{
        Ok(())
    }}
}}

impl TxHashable for {tx_struct} {{
    fn to_canonical_bytes(&self) -> Vec<u8> {{
        self.raw_bytes.clone()
    }}

    fn compute_hash(&self) -> Vec<u8> {{
        self.txid()
    }}
}}
"#,
        chain_name = spec.chain.name,
        tx_struct = tx_struct
    )
}

fn generate_property_tests(spec: &DecoderSpec) -> String {
    let decoder_struct = format!("{}Decoder", spec.chain.name);

    format!(
        r#"//! Property-based tests for {chain_name} decoder
//!
//! Generated from specification.

use decoder_{chain_lower}::*;
use proptest::prelude::*;
use decoder_test_utils::proptest_helpers::*;

proptest! {{
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_never_panics(bytes in arb_small_bytes()) {{
        prop_decoder_never_panics::<{decoder_struct}>(&bytes);
    }}
}}
"#,
        chain_name = spec.chain.name,
        chain_lower = spec.chain.name.to_lowercase(),
        decoder_struct = decoder_struct
    )
}
