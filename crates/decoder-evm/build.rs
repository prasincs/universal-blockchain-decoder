use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChainlistChain {
    name: String,
    chain: String,
    #[serde(default)]
    icon: Option<String>,
    rpc: Vec<String>,
    #[serde(default)]
    features: Vec<Feature>,
    #[serde(default)]
    faucets: Vec<String>,
    native_currency: NativeCurrency,
    #[serde(rename = "infoURL")]
    info_url: String,
    short_name: String,
    chain_id: u64,
    network_id: u64,
    #[serde(default)]
    slip44: Option<u64>,
    #[serde(default)]
    ens: Option<Ens>,
    #[serde(default)]
    explorers: Vec<Explorer>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Feature {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NativeCurrency {
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Deserialize, Serialize)]
struct Ens {
    registry: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Explorer {
    name: String,
    url: String,
    #[serde(default)]
    standard: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

fn main() {
    println!("cargo:rerun-if-changed=vendored/chainlist/_data/chains");

    let chains_dir = Path::new("vendored/chainlist/_data/chains");

    if !chains_dir.exists() {
        eprintln!("Warning: Chainlist directory not found. Run: git subtree add --prefix crates/decoder-evm/vendored/chainlist https://github.com/ethereum-lists/chains.git master --squash");
        // Create empty registry for compilation to succeed
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("chain_registry.rs");
        fs::write(
            &dest_path,
            "pub fn get_embedded_chains() -> std::collections::HashMap<u64, crate::types::ChainInfo> {\n    std::collections::HashMap::new()\n}\n"
        ).unwrap();
        return;
    }

    let mut chains: HashMap<u64, ChainlistChain> = HashMap::new();
    let mut parse_errors = 0;
    let mut total_files = 0;

    // Parse all EIP-155 chain files
    for entry in fs::read_dir(chains_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with("eip155-") {
                    total_files += 1;

                    match fs::read_to_string(&path) {
                        Ok(contents) => {
                            match serde_json::from_str::<ChainlistChain>(&contents) {
                                Ok(chain) => {
                                    chains.insert(chain.chain_id, chain);
                                }
                                Err(e) => {
                                    if parse_errors < 5 {
                                        eprintln!("Warning: Failed to parse {}: {}", filename, e);
                                    }
                                    parse_errors += 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to read {}: {}", filename, e);
                            parse_errors += 1;
                        }
                    }
                }
            }
        }
    }

    println!("cargo:warning=Parsed {} EVM chains from chainlist ({} files, {} parse errors)",
        chains.len(), total_files, parse_errors);

    // Generate Rust code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("chain_registry.rs");

    let mut code = String::new();
    code.push_str("// Auto-generated from ethereum-lists/chains at build time\n");
    code.push_str("// DO NOT EDIT MANUALLY\n\n");
    code.push_str("pub fn get_embedded_chains() -> HashMap<u64, crate::types::ChainInfo> {\n");
    code.push_str("    let mut chains = HashMap::new();\n\n");

    // Sort chains by chain ID for deterministic output
    let mut sorted_chains: Vec<_> = chains.iter().collect();
    sorted_chains.sort_by_key(|(id, _)| *id);

    for (chain_id, chain) in sorted_chains.iter() {
        // Escape strings for Rust code
        let name = chain.name.replace('\\', "\\\\").replace('"', "\\\"");
        let short_name = chain.short_name.replace('\\', "\\\\").replace('"', "\\\"");
        let chain_symbol = chain.chain.replace('\\', "\\\\").replace('"', "\\\"");
        let info_url = chain.info_url.replace('\\', "\\\\").replace('"', "\\\"");

        let currency_name = chain.native_currency.name.replace('\\', "\\\\").replace('"', "\\\"");
        let currency_symbol = chain.native_currency.symbol.replace('\\', "\\\\").replace('"', "\\\"");

        // Detect if chain is testnet (heuristic)
        let is_testnet = name.to_lowercase().contains("test") ||
                         name.to_lowercase().contains("sepolia") ||
                         name.to_lowercase().contains("goerli") ||
                         name.to_lowercase().contains("holesky");

        // Detect special chain types
        let has_custom_tx_types = **chain_id == 10 || // Optimism
                                   **chain_id == 42161 || // Arbitrum
                                   **chain_id == 324; // zkSync Era

        code.push_str(&format!("    chains.insert({}, crate::types::ChainInfo {{\n", chain_id));
        code.push_str(&format!("        chain_id: {},\n", chain_id));
        code.push_str(&format!("        name: \"{}\".to_string(),\n", name));
        code.push_str(&format!("        short_name: \"{}\".to_string(),\n", short_name));
        code.push_str(&format!("        chain: \"{}\".to_string(),\n", chain_symbol));
        code.push_str(&format!("        network_id: {},\n", chain.network_id));
        code.push_str(&format!("        is_testnet: {},\n", is_testnet));
        code.push_str(&format!("        has_custom_tx_types: {},\n", has_custom_tx_types));
        code.push_str("        native_currency: crate::types::CurrencyInfo {\n");
        code.push_str(&format!("            name: \"{}\".to_string(),\n", currency_name));
        code.push_str(&format!("            symbol: \"{}\".to_string(),\n", currency_symbol));
        code.push_str(&format!("            decimals: {},\n", chain.native_currency.decimals));
        code.push_str("        },\n");
        code.push_str(&format!("        info_url: \"{}\".to_string(),\n", info_url));

        // Add RPC URLs (limit to first 3 to keep code size reasonable)
        code.push_str("        rpc: vec![\n");
        for rpc_url in chain.rpc.iter().take(3) {
            let escaped_url = rpc_url.replace('\\', "\\\\").replace('"', "\\\"");
            code.push_str(&format!("            \"{}\".to_string(),\n", escaped_url));
        }
        code.push_str("        ],\n");

        // Add explorers (limit to first 2)
        code.push_str("        explorers: vec![\n");
        for explorer in chain.explorers.iter().take(2) {
            let explorer_name = explorer.name.replace('\\', "\\\\").replace('"', "\\\"");
            let explorer_url = explorer.url.replace('\\', "\\\\").replace('"', "\\\"");
            let standard = explorer.standard.as_ref().map(|s| s.replace('\\', "\\\\").replace('"', "\\\"")).unwrap_or_default();

            code.push_str("            crate::types::ExplorerInfo {\n");
            code.push_str(&format!("                name: \"{}\".to_string(),\n", explorer_name));
            code.push_str(&format!("                url: \"{}\".to_string(),\n", explorer_url));
            code.push_str(&format!("                standard: \"{}\".to_string(),\n", standard));
            code.push_str("            },\n");
        }
        code.push_str("        ],\n");

        code.push_str("    });\n\n");
    }

    code.push_str("    chains\n");
    code.push_str("}\n");

    fs::write(&dest_path, code).unwrap();
}
