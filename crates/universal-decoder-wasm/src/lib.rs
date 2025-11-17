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
use decoder_starknet::StarknetDecoder;
use universal_decoder_core::prelude::{CanonicalSerialize, Canonicalizer, ChainDecoder};

/// Result of decoding a transaction (serializable to JavaScript).
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct DecodeResult {
    /// Hex-encoded canonical Borsh bytes
    pub canonical_hex: String,

    /// Hex-encoded canonical hash (for quick comparison)
    pub canonical_hash: String,

    /// Human-readable JSON representation
    #[wasm_bindgen(skip)]
    pub json: serde_json::Value,

    /// Chain name
    pub chain_name: String,

    /// Chain ID
    pub chain_id: u64,

    /// Canonical size in bytes
    pub canonical_size: usize,
}

#[wasm_bindgen]
impl DecodeResult {
    /// Get JSON as JsValue for JavaScript interop
    #[wasm_bindgen(getter)]
    pub fn json(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.json).unwrap_or(JsValue::NULL)
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
/// * `chain` - Chain name: "bitcoin", "ethereum", "solana", "cosmos", "starknet"
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
        "starknet" => decode_with::<StarknetDecoder>(&bytes),
        _ => Err(JsValue::from_str(&format!(
            "Unsupported chain: {}. Supported: bitcoin, ethereum, solana, cosmos, starknet",
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
        "starknet".to_string(),
    ]
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
    if StarknetDecoder::decode(&bytes).is_ok() {
        return Ok("starknet".to_string());
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

    Ok(DecodeResult {
        canonical_hex: universal_decoder_core::hex::encode(&canonical_bytes),
        canonical_hash: universal_decoder_core::hex::encode(&canonical_hash),
        json,
        chain_name: tx_ir.chain.name.clone(),
        chain_id: tx_ir.chain.id,
        canonical_size: canonical_bytes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_supported_chains() {
        let chains = supported_chains();
        assert_eq!(chains.len(), 5);
        assert!(chains.contains(&"bitcoin".to_string()));
        assert!(chains.contains(&"ethereum".to_string()));
        assert!(chains.contains(&"starknet".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_invalid_hex() {
        let result = decode_transaction("bitcoin", "not_hex");
        assert!(result.is_err());
    }
}
