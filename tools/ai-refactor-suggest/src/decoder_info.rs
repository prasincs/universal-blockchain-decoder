use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Chain families as defined in universal-decoder-core/src/chain.rs
const CHAIN_FAMILIES: &[(&str, &[&str])] = &[
    ("utxo", &["bitcoin", "litecoin", "dogecoin", "cardano"]),
    (
        "account",
        &[
            "ethereum",
            "evm",
            "polygon",
            "bnb",
            "avalanche",
            "optimism",
            "arbitrum",
        ],
    ),
    ("instruction", &["solana", "aptos", "sui"]),
    (
        "other",
        &[
            "xrp", "tron", "polkadot", "near", "cosmos", "stellar", "algorand",
        ],
    ),
];

/// Information about a decoder crate
#[derive(Debug, Clone)]
pub struct DecoderInfo {
    pub name: String,
    pub path: PathBuf,
    pub family: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub loc: usize,
    pub has_tests: bool,
}

impl DecoderInfo {
    /// Read source files from the decoder
    pub fn read_source_files(&self, max_files: usize) -> Result<HashMap<String, String>> {
        let src_dir = self.path.join("src");
        let mut source_files = HashMap::new();

        if !src_dir.exists() {
            return Ok(source_files);
        }

        let walker = walkdir::WalkDir::new(&src_dir)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
            .take(max_files);

        for entry in walker {
            let path = entry.path();
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;

            let rel_path = path
                .strip_prefix(&self.path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            source_files.insert(rel_path, content);
        }

        Ok(source_files)
    }

    /// Get blockchain-specific dependencies (not in dev-dependencies)
    pub fn blockchain_dependencies(&self) -> Vec<String> {
        const BLOCKCHAIN_LIBS: &[&str] = &[
            "bitcoin",
            "ethers",
            "ethers-core",
            "alloy",
            "alloy-rs",
            "solana-sdk",
            "aptos-sdk",
            "sui-sdk",
            "near-sdk",
            "cosmos-sdk",
        ];

        self.dependencies
            .keys()
            .filter(|dep| BLOCKCHAIN_LIBS.iter().any(|lib| dep.contains(lib)))
            .cloned()
            .collect()
    }
}

/// Decoder discovery service
pub struct DecoderDiscovery {
    repo_root: PathBuf,
}

impl DecoderDiscovery {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Discover all decoder crates in the repository
    pub fn discover(&self) -> Result<Vec<DecoderInfo>> {
        let crates_dir = self.repo_root.join("crates");
        let mut decoders = Vec::new();

        if !crates_dir.exists() {
            anyhow::bail!("Crates directory not found: {}", crates_dir.display());
        }

        let entries = fs::read_dir(&crates_dir)
            .with_context(|| format!("Failed to read directory: {}", crates_dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().unwrap().to_string_lossy();

            // Skip non-decoder crates
            if !dir_name.starts_with("decoder-") {
                continue;
            }

            // Skip utility crates
            if dir_name.ends_with("-primitives")
                || dir_name.ends_with("-encodings")
                || dir_name.ends_with("-test-utils")
            {
                continue;
            }

            let cargo_toml = path.join("Cargo.toml");
            if !cargo_toml.exists() {
                continue;
            }

            // Extract decoder name
            let decoder_name = dir_name.strip_prefix("decoder-").unwrap().to_string();

            // Determine chain family
            let family = Self::get_chain_family(&decoder_name);

            // Parse Cargo.toml
            let (dependencies, dev_dependencies) = Self::parse_cargo_toml(&cargo_toml)?;

            // Count lines of code
            let loc = Self::count_loc(&path)?;

            // Check for tests
            let has_tests =
                path.join("tests").exists() || Self::has_test_modules(&path.join("src"))?;

            decoders.push(DecoderInfo {
                name: decoder_name,
                path,
                family: family.to_string(),
                dependencies,
                dev_dependencies,
                loc,
                has_tests,
            });
        }

        Ok(decoders)
    }

    fn get_chain_family(decoder_name: &str) -> &'static str {
        for (family, chains) in CHAIN_FAMILIES {
            if chains.contains(&decoder_name) {
                return family;
            }
        }
        "other"
    }

    fn parse_cargo_toml(path: &Path) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let cargo_toml: toml::Value = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML: {}", path.display()))?;

        let mut dependencies = HashMap::new();
        let mut dev_dependencies = HashMap::new();

        // Parse [dependencies]
        if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
            for (name, value) in deps {
                let version = Self::extract_version(value);
                dependencies.insert(name.clone(), version);
            }
        }

        // Parse [dev-dependencies]
        if let Some(deps) = cargo_toml
            .get("dev-dependencies")
            .and_then(|d| d.as_table())
        {
            for (name, value) in deps {
                let version = Self::extract_version(value);
                dev_dependencies.insert(name.clone(), version);
            }
        }

        Ok((dependencies, dev_dependencies))
    }

    fn extract_version(value: &toml::Value) -> String {
        match value {
            toml::Value::String(s) => s.clone(),
            toml::Value::Table(t) => {
                if let Some(toml::Value::String(v)) = t.get("version") {
                    v.clone()
                } else if t.contains_key("path") {
                    "[workspace]".to_string()
                } else if t.contains_key("git") {
                    "[git]".to_string()
                } else {
                    "[unknown]".to_string()
                }
            }
            _ => "[unknown]".to_string(),
        }
    }

    fn count_loc(decoder_dir: &Path) -> Result<usize> {
        let src_dir = decoder_dir.join("src");
        if !src_dir.exists() {
            return Ok(0);
        }

        let output = Command::new("find")
            .arg(&src_dir)
            .arg("-name")
            .arg("*.rs")
            .arg("-exec")
            .arg("wc")
            .arg("-l")
            .arg("{}")
            .arg("+")
            .output()
            .context("Failed to execute find command")?;

        if !output.status.success() {
            return Ok(0);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines = output_str.lines().collect::<Vec<_>>();

        if let Some(last_line) = lines.last() {
            let parts: Vec<&str> = last_line.split_whitespace().collect();
            if let Some(total) = parts.first() {
                return Ok(total.parse().unwrap_or(0));
            }
        }

        Ok(0)
    }

    fn has_test_modules(src_dir: &Path) -> Result<bool> {
        if !src_dir.exists() {
            return Ok(false);
        }

        for entry in walkdir::WalkDir::new(src_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                let content = fs::read_to_string(path)?;
                if content.contains("#[cfg(test)]") || content.contains("#[test]") {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
