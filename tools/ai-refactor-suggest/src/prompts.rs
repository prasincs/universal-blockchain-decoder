use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::decoder_info::DecoderInfo;

/// Prompt manager for loading and formatting chain family-specific prompts
pub struct PromptManager {
    prompts: HashMap<String, String>,
}

impl PromptManager {
    /// Load prompts from the prompts directory
    pub fn load_from_directory(prompts_dir: &Path) -> Result<Self> {
        let mut prompts = HashMap::new();

        if !prompts_dir.exists() {
            eprintln!(
                "Warning: Prompts directory not found: {}",
                prompts_dir.display()
            );
            return Ok(Self { prompts });
        }

        for entry in fs::read_dir(prompts_dir).with_context(|| {
            format!(
                "Failed to read prompts directory: {}",
                prompts_dir.display()
            )
        })? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "md").unwrap_or(false) {
                let filename = path.file_stem().unwrap().to_string_lossy();

                // Extract family name from filename (prompt-utxo.md -> utxo)
                let family = filename.strip_prefix("prompt-").unwrap_or(&filename);

                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read prompt file: {}", path.display()))?;

                prompts.insert(family.to_string(), content);
            }
        }

        Ok(Self { prompts })
    }

    /// Build a complete analysis prompt for a decoder
    pub fn build_prompt(
        &self,
        decoder: &DecoderInfo,
        source_files: &HashMap<String, String>,
        latest_updates: &str,
    ) -> String {
        // Load family-specific template
        let template = self
            .prompts
            .get(&decoder.family)
            .or_else(|| self.prompts.get("base"))
            .map(|s| s.as_str())
            .unwrap_or_else(|| Self::default_instructions(&decoder.family));

        // Build context
        let mut prompt = format!(
            r#"# Decoder Analysis Request

## Decoder Information
- **Name**: {}
- **Chain Family**: {}
- **Lines of Code**: {}
- **Has Tests**: {}
- **Production Dependencies**: {}
- **Dev Dependencies**: {}

### Blockchain-Specific Dependencies in Production
{}

## Latest Ecosystem Updates
{}

## Source Code

"#,
            decoder.name,
            decoder.family,
            decoder.loc,
            decoder.has_tests,
            decoder.dependencies.len(),
            decoder.dev_dependencies.len(),
            Self::format_blockchain_deps(decoder),
            latest_updates
        );

        // Add source file contents (truncated to avoid token limits)
        for (file_path, content) in source_files {
            let truncated = if content.len() > 4000 {
                format!(
                    "{}... [truncated, {} total chars]",
                    &content[..4000],
                    content.len()
                )
            } else {
                content.clone()
            };

            prompt.push_str(&format!(
                "\n### {}\n```rust\n{}\n```\n",
                file_path, truncated
            ));
        }

        // Add analysis instructions from template
        prompt.push_str("\n## Analysis Instructions\n\n");
        prompt.push_str(template);

        // Add output format requirements
        prompt.push_str(
            r#"

## Output Format

Provide suggestions in the following JSON format:

```json
[
  {
    "category": "dependency|security|performance|testing|architecture",
    "priority": "high|medium|low",
    "title": "Brief, actionable title",
    "description": "Detailed description of the issue and suggested improvement. Include specific references to files and functions where applicable.",
    "code_location": "Optional: path/to/file.rs:line_number",
    "suggested_change": "Optional: specific code snippet or refactoring approach"
  }
]
```

Focus on actionable, specific suggestions that align with the project's goals:
1. Minimal trusted computing base (< 3000 LOC core)
2. Formal verifiability (no unsafe, explicit contracts)
3. Canonical serialization (Borsh for TxIR, chain-native for transaction hashing)
4. Zero-cost abstractions
5. Pure Rust implementations (blockchain libs in dev-dependencies only)

**Important**: Return ONLY the JSON array. Do not include any explanatory text before or after the JSON.
"#,
        );

        prompt
    }

    fn format_blockchain_deps(decoder: &DecoderInfo) -> String {
        let blockchain_deps = decoder.blockchain_dependencies();

        if blockchain_deps.is_empty() {
            "✅ None (good! Pure Rust implementation)".to_string()
        } else {
            let mut result = "⚠️ Found blockchain-specific dependencies:\n".to_string();
            for dep in &blockchain_deps {
                let version = decoder.dependencies.get(dep).unwrap();
                result.push_str(&format!("   - {} = {}\n", dep, version));
            }
            result.push_str("   These should be moved to dev-dependencies (Phase 2 goal)");
            result
        }
    }

    fn default_instructions(family: &str) -> &'static str {
        match family {
            "utxo" => DEFAULT_UTXO_INSTRUCTIONS,
            "account" => DEFAULT_ACCOUNT_INSTRUCTIONS,
            "instruction" => DEFAULT_INSTRUCTION_INSTRUCTIONS,
            _ => DEFAULT_BASE_INSTRUCTIONS,
        }
    }
}

const DEFAULT_BASE_INSTRUCTIONS: &str = r#"Analyze this blockchain decoder for:

1. **Dependency Management**: Are blockchain-specific libraries in dev-dependencies only? Can any dependencies be removed or vendored?

2. **Security**: Any unsafe code blocks? Proper input validation? Overflow/underflow protection? Canonical serialization used correctly (Borsh for TxIR)?

3. **Performance**: Unnecessary allocations? Can use zero-cost abstractions better? Opportunities for const generics or compile-time optimization?

4. **Testing**: Adequate test coverage? Property-based tests needed? Integration tests with real blockchain data?

5. **Architecture**: Follows trait-based extensibility? Minimal coupling to core? Clear separation of parsing vs validation?
"#;

const DEFAULT_UTXO_INSTRUCTIONS: &str = r#"Analyze this UTXO-based blockchain decoder focusing on:

1. **Transaction Structure**: Proper parsing of inputs (previous tx hash + index, scripts) and outputs (value, locking scripts)
2. **Script Parsing**: Correct handling of Bitcoin Script opcodes, SegWit witness data, Taproot (if applicable)
3. **Dependency Strategy**: Pure Rust implementation without bitcoin crate in production dependencies
4. **Security**: Script validation without executing untrusted code, overflow protection for satoshi values
5. **Testing**: Coverage of all script types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR if applicable)
"#;

const DEFAULT_ACCOUNT_INSTRUCTIONS: &str = r#"Analyze this EVM/Account-based blockchain decoder focusing on:

1. **Transaction Types**: Support for all EIP-2718 transaction types (Legacy, EIP-2930, EIP-1559, EIP-4844)
2. **RLP Encoding**: Pure Rust implementation without alloy/ethers in production dependencies
3. **Chain ID**: Proper EIP-155 chain ID handling
4. **Security**: RLP parsing bounds checking, signature verification, overflow protection
5. **Testing**: Coverage of all transaction types and real mainnet transactions
"#;

const DEFAULT_INSTRUCTION_INSTRUCTIONS: &str = r#"Analyze this instruction-based blockchain decoder focusing on:

1. **Encoding Format**: Pure Rust implementation (Solana: compact-array, Aptos/Sui: BCS) without SDK in production
2. **Transaction Structure**: Proper parsing of instructions/messages, account/object references
3. **Signature Verification**: Correct Ed25519 signature handling
4. **Security**: Deserialization bounds checking, max depth limits for recursive structures
5. **Testing**: Coverage of different instruction types, versioned transactions (Solana v0), sponsored transactions (Sui)
"#;

/// Generate information about latest ecosystem updates
pub fn get_latest_updates(decoder: &DecoderInfo) -> String {
    let mut updates: Vec<String> = Vec::new();

    match decoder.name.as_str() {
        "bitcoin" => {
            updates.push(
                "- Bitcoin Core 27.0: Improved transaction relay, mempool policy updates"
                    .to_string(),
            );
            updates
                .push("- Taproot adoption increasing: Ensure full BIP 340-342 support".to_string());
            updates.push(
                "- Ordinals/Inscriptions: Consider parsing inscription data in witness".to_string(),
            );
        }
        "ethereum" => {
            updates
                .push("- Cancun upgrade (2024): EIP-4844 blob transactions now active".to_string());
            updates
                .push("- alloy-rs 0.7+: Modern Ethereum library (successor to ethers)".to_string());
            updates.push("- EIP-1153: Transient storage opcodes".to_string());
        }
        "solana" => {
            updates.push(
                "- Solana 1.18: Versioned transactions (v0) with address lookup tables".to_string(),
            );
            updates.push("- Fee markets: Priority fees via compute budget program".to_string());
            updates.push("- State compression: Concurrent merkle trees for NFTs".to_string());
        }
        "aptos" => {
            updates.push("- Aptos Framework 1.x: Ongoing module updates".to_string());
            updates.push("- Keyless accounts: OAuth-based authentication".to_string());
            updates.push("- Parallel execution: BlockSTM consensus".to_string());
        }
        "sui" => {
            updates.push("- Sui Move: Diverging from Aptos Move standard library".to_string());
            updates.push("- Mysticeti consensus: High throughput improvements".to_string());
            updates.push("- Sponsored transactions: Gas payer separation".to_string());
        }
        name if name.contains("evm") || decoder.family == "account" => {
            updates.push("- EIP-4844: Blob transactions for L2 data availability".to_string());
            updates.push("- EIP-1559: Dynamic fee markets (most EVM chains)".to_string());
            updates.push("- Account abstraction (EIP-4337): Emerging standard".to_string());
        }
        _ => {
            updates.push("- Check for protocol updates specific to this chain".to_string());
        }
    }

    // Check for outdated dependencies
    for (dep, version) in &decoder.dependencies {
        if version.starts_with("0.") {
            updates.push(format!(
                "- Review dependency: {} at version {}",
                dep, version
            ));
        }
    }

    if updates.is_empty() {
        "No recent critical updates identified".to_string()
    } else {
        updates.join("\n")
    }
}
