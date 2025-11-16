//! WebAssembly bindings for the Universal Blockchain Decoder.
//!
//! This crate provides a thin WASM wrapper around the existing decoder infrastructure,
//! exposing the decoder functionality to JavaScript/browser environments.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Import existing decoders (reusing implementations!)
use decoder_bitcoin::BitcoinDecoder;
use decoder_cosmos::CosmosDecoder;
use decoder_ethereum::EthereumDecoder;
use decoder_solana::SolanaDecoder;
use universal_decoder_core::prelude::{CanonicalSerialize, Canonicalizer, ChainDecoder, TxIR};

/// Chain metadata for frontend display
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct ChainMetadata {
    /// Chain identifier (lowercase, e.g., "bitcoin", "ethereum")
    pub id: String,

    /// Human-readable name (e.g., "Bitcoin", "Ethereum")
    pub name: String,

    /// Chain family type
    pub family: String,

    /// Whether privacy features are supported
    pub has_privacy: bool,
}

/// Result of decoding a transaction (serializable to JavaScript).
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct DecodeResult {
    /// Hex-encoded canonical Borsh bytes (raw payload)
    pub canonical_hex: String,

    /// Hex-encoded canonical hash (for quick comparison)
    pub canonical_hash: String,

    /// Human-readable JSON representation
    #[wasm_bindgen(skip)]
    pub json: serde_json::Value,

    /// Borsh-decoded fields (structured representation)
    #[wasm_bindgen(skip)]
    pub borsh_fields: serde_json::Value,

    /// Chain name
    pub chain_name: String,

    /// Chain ID
    pub chain_id: u64,

    /// Canonical size in bytes
    pub canonical_size: usize,

    /// Privacy score (0-100)
    pub privacy_score: u8,

    /// Privacy features detected
    #[wasm_bindgen(skip)]
    pub privacy_features: Vec<String>,

    /// Transaction type
    pub transaction_type: String,
}

#[wasm_bindgen]
impl DecodeResult {
    /// Get JSON as JsValue for JavaScript interop
    #[wasm_bindgen(getter)]
    pub fn json(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.json).unwrap_or(JsValue::NULL)
    }

    /// Get Borsh fields as JsValue for JavaScript interop
    #[wasm_bindgen(getter)]
    pub fn borsh_fields(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.borsh_fields).unwrap_or(JsValue::NULL)
    }

    /// Get privacy features as JsValue for JavaScript interop
    #[wasm_bindgen(getter)]
    pub fn privacy_features(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.privacy_features).unwrap_or(JsValue::NULL)
    }
}

/// Initialize the WASM module. Call this once before using any functions.
#[wasm_bindgen(start)]
pub fn init() {
    // Set up better panic messages in browser console
    console_error_panic_hook::set_once();
}

/// Main entry point: Decode a transaction from any supported chain.
///
/// # Arguments
///
/// * `chain` - Chain name: "bitcoin", "ethereum", "solana", "cosmos"
/// * `hex` - Hex-encoded transaction bytes
///
/// # Returns
///
/// `DecodeResult` containing:
/// - `canonical_hex`: Borsh-encoded canonical representation
/// - `canonical_hash`: SHA-256 hash of canonical bytes
/// - `json`: Human-readable JSON (via `.json()` getter)
/// - `chain_name`, `chain_id`: Chain identification
/// - `canonical_size`: Size of canonical representation in bytes
#[wasm_bindgen]
pub fn decode_transaction(chain: &str, hex: &str) -> Result<DecodeResult, JsValue> {
    let bytes = universal_decoder_core::hex::decode(hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))?;

    match chain.to_lowercase().as_str() {
        "bitcoin" => decode_with::<BitcoinDecoder>(&bytes),
        "ethereum" => decode_with::<EthereumDecoder>(&bytes),
        "solana" => decode_with::<SolanaDecoder>(&bytes),
        "cosmos" => decode_with::<CosmosDecoder>(&bytes),
        _ => Err(JsValue::from_str(&format!(
            "Unsupported chain: {}. Supported: bitcoin, ethereum, solana, cosmos",
            chain
        ))),
    }
}

/// List all supported blockchain names
#[wasm_bindgen]
pub fn supported_chains() -> Vec<String> {
    vec![
        "bitcoin".to_string(),
        "ethereum".to_string(),
        "solana".to_string(),
        "cosmos".to_string(),
    ]
}

/// Get detailed metadata for all supported chains
#[wasm_bindgen]
pub fn get_chains_metadata() -> JsValue {
    let chains = vec![
        ChainMetadata {
            id: "bitcoin".to_string(),
            name: "Bitcoin".to_string(),
            family: "UTXO".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "solana".to_string(),
            name: "Solana".to_string(),
            family: "Instruction".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "cosmos".to_string(),
            name: "Cosmos".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
    ];

    serde_wasm_bindgen::to_value(&chains).unwrap_or(JsValue::NULL)
}

/// Attempt to automatically detect which blockchain a transaction belongs to
/// by trying decoders in order of popularity.
#[wasm_bindgen]
pub fn auto_detect_chain(hex: &str) -> Result<String, JsValue> {
    let bytes = universal_decoder_core::hex::decode(hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))?;

    // Try decoders in order (most common first)
    if BitcoinDecoder::decode(&bytes).is_ok() {
        return Ok("bitcoin".to_string());
    }
    if EthereumDecoder::decode(&bytes).is_ok() {
        return Ok("ethereum".to_string());
    }
    if SolanaDecoder::decode(&bytes).is_ok() {
        return Ok("solana".to_string());
    }
    if CosmosDecoder::decode(&bytes).is_ok() {
        return Ok("cosmos".to_string());
    }

    Err(JsValue::from_str("Could not auto-detect chain"))
}

// ============================================================================
// Generic decoder helper
// ============================================================================

fn decode_with<D: ChainDecoder>(bytes: &[u8]) -> Result<DecodeResult, JsValue> {
    // Decode using chain-specific decoder
    let tx = D::decode(bytes).map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

    // Canonicalize to TxIR
    let tx_ir = tx
        .canonicalize()
        .map_err(|e| JsValue::from_str(&format!("TxIR conversion error: {}", e)))?;

    // Get canonical bytes (Borsh encoding)
    let canonical_bytes = tx_ir
        .to_canonical_bytes()
        .map_err(|e| JsValue::from_str(&format!("Canonical encoding error: {}", e)))?;

    // Get canonical hash (SHA-256 of Borsh bytes)
    let canonical_hash = tx_ir
        .canonical_hash()
        .map_err(|e| JsValue::from_str(&format!("Hash error: {}", e)))?;

    // Serialize to JSON for display
    let json = serde_json::to_value(&tx_ir)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {}", e)))?;

    // Extract privacy information
    let (privacy_score, privacy_features) = extract_privacy_info(&tx_ir);

    // Create Borsh fields representation (structured view)
    let borsh_fields = create_borsh_fields(&tx_ir)?;

    // Determine transaction type
    let transaction_type = determine_tx_type(&tx_ir);

    Ok(DecodeResult {
        canonical_hex: universal_decoder_core::hex::encode(&canonical_bytes),
        canonical_hash: universal_decoder_core::hex::encode(&canonical_hash),
        json,
        borsh_fields,
        chain_name: tx_ir.chain.name.clone(),
        chain_id: tx_ir.chain.id,
        canonical_size: canonical_bytes.len(),
        privacy_score,
        privacy_features,
        transaction_type,
    })
}

/// Extract privacy information from TxIR
fn extract_privacy_info(tx_ir: &TxIR<'_, 1>) -> (u8, Vec<String>) {
    match &tx_ir.privacy {
        Some(privacy) => {
            let features: Vec<String> = privacy
                .features
                .iter()
                .map(|f| format!("{:?}", f))
                .collect();

            // Calculate privacy score based on features
            let score = if features.is_empty() {
                0
            } else {
                ((features.len() * 25).min(100)) as u8
            };

            (score, features)
        }
        None => (0, vec![]),
    }
}

/// Create structured Borsh fields representation
fn create_borsh_fields(tx_ir: &TxIR<'_, 1>) -> Result<serde_json::Value, JsValue> {
    use serde_json::json;
    use universal_decoder_core::prelude::Operation;

    // Create a structured view of the Borsh-encoded data
    Ok(json!({
        "chain": {
            "id": tx_ir.chain.id,
            "name": &tx_ir.chain.name,
        },
        "metadata": {
            "timestamp": tx_ir.metadata.timestamp,
            "block_height": tx_ir.metadata.block_height,
            "tx_hash": universal_decoder_core::hex::encode(&tx_ir.metadata.tx_hash),
            "size": tx_ir.metadata.size,
        },
        "authorization": {
            "signature_scheme": format!("{:?}", tx_ir.authorization.signature_scheme),
            "public_keys_count": tx_ir.authorization.public_keys.len(),
            "signatures_count": tx_ir.authorization.signatures.len(),
        },
        "operations": tx_ir.operations.iter().map(|op| {
            match op {
                Operation::Transfer(t) => json!({
                    "type": "Transfer",
                    "from": universal_decoder_core::hex::encode(&t.from.bytes),
                    "to": universal_decoder_core::hex::encode(&t.to.bytes),
                    "amount": t.amount.value.to_string(),
                }),
                Operation::ContractCall(c) => json!({
                    "type": "ContractCall",
                    "contract": universal_decoder_core::hex::encode(&c.contract.bytes),
                    "value": c.value.as_ref().map(|v| v.value.to_string()),
                }),
                Operation::ContractDeploy(d) => json!({
                    "type": "ContractDeploy",
                    "bytecode_size": d.bytecode.len(),
                    "value": d.value.value.to_string(),
                }),
                Operation::Stake(s) => json!({
                    "type": "Stake",
                    "validator": universal_decoder_core::hex::encode(&s.validator.bytes),
                    "amount": s.amount.value.to_string(),
                    "operation": format!("{:?}", s.operation_type),
                }),
                Operation::Generic(g) => json!({
                    "type": "Generic",
                    "op_type": &g.op_type,
                }),
            }
        }).collect::<Vec<_>>(),
        "state_deltas": {
            "inputs": tx_ir.state_deltas.inputs.len(),
            "outputs": tx_ir.state_deltas.outputs.len(),
            "account_changes": tx_ir.state_deltas.account_changes.len(),
        },
        "privacy": tx_ir.privacy.as_ref().map(|p| json!({
            "features": p.features.iter().map(|f| format!("{:?}", f)).collect::<Vec<_>>(),
            "observability": format!("{:?}", p.observability),
        })),
    }))
}

/// Determine transaction type from TxIR
fn determine_tx_type(tx_ir: &TxIR<'_, 1>) -> String {
    use universal_decoder_core::prelude::Operation;

    if tx_ir.operations.is_empty() {
        return "Unknown".to_string();
    }

    let first_op = &tx_ir.operations[0];
    match first_op {
        Operation::Transfer(_) => "Transfer".to_string(),
        Operation::ContractCall(_) => "ContractCall".to_string(),
        Operation::ContractDeploy(_) => "ContractDeploy".to_string(),
        Operation::Stake(_) => "Stake".to_string(),
        Operation::Generic(g) => format!("Generic({})", g.op_type),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_supported_chains() {
        let chains = supported_chains();
        assert_eq!(chains.len(), 4);
        assert!(chains.contains(&"bitcoin".to_string()));
        assert!(chains.contains(&"ethereum".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_invalid_hex() {
        let result = decode_transaction("bitcoin", "not_hex");
        assert!(result.is_err());
    }
}
