//! WebAssembly bindings for the Universal Blockchain Decoder.
//!
//! This crate provides a thin WASM wrapper around the existing decoder infrastructure,
//! exposing the decoder functionality to JavaScript/browser environments.
//!
//! # Architecture
//!
//! This is a **thin wrapper** that reuses:
//! - Core library (`universal-decoder-core`) for traits and TxIR
//! - Existing decoders (`decoder-bitcoin`, `decoder-ethereum`, etc.)
//! - All canonicalization and validation logic already implemented
//!
//! # Usage (from JavaScript)
//!
//! ```javascript
//! import init, { decode_transaction, supported_chains } from './pkg/universal_decoder_wasm.js';
//!
//! await init();
//!
//! const chains = supported_chains();
//! console.log(chains); // ["bitcoin", "ethereum", "solana", "cosmos"]
//!
//! const result = decode_transaction("bitcoin", "0100000001...");
//! console.log(result.json);
//! console.log(result.canonical_hex);
//! ```

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Import existing decoders (reusing implementations!)
use decoder_bitcoin::{BitcoinDecoder, BitcoinTransaction};
use decoder_cosmos::{CosmosDecoder, CosmosTransaction};
use decoder_ethereum::{EthereumDecoder, EthereumTransaction};
use decoder_solana::{SolanaDecoder, SolanaTransaction};
use universal_decoder_core::{Canonicalizer, ChainDecoder};

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

    /// Privacy features detected (for highlighting)
    #[wasm_bindgen(skip)]
    pub privacy_features: Vec<String>,

    /// Privacy score (0 = fully observable, 100 = fully private)
    pub privacy_score: u8,

    /// Chain name
    pub chain_name: String,

    /// Chain ID
    pub chain_id: u64,

    /// Transaction type (e.g., "Transfer", "ContractCall")
    pub transaction_type: String,

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

    /// Get privacy features as JavaScript array
    #[wasm_bindgen(getter)]
    pub fn privacy_features(&self) -> Vec<String> {
        self.privacy_features.clone()
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
/// `DecodeResult` with canonical bytes, JSON, and privacy analysis
///
/// # Errors
///
/// Returns error if:
/// - Chain is not supported
/// - Hex is invalid
/// - Transaction parsing fails
#[wasm_bindgen]
pub fn decode_transaction(chain: &str, hex: &str) -> Result<DecodeResult, JsValue> {
    // Decode hex to bytes
    let bytes = universal_decoder_core::hex::decode(hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))?;

    // Decode based on chain (reusing existing decoders!)
    match chain.to_lowercase().as_str() {
        "bitcoin" | "btc" => decode_bitcoin_transaction(&bytes),
        "ethereum" | "eth" => decode_ethereum_transaction(&bytes),
        "solana" | "sol" => decode_solana_transaction(&bytes),
        "cosmos" | "atom" => decode_cosmos_transaction(&bytes),
        _ => Err(JsValue::from_str(&format!("Unsupported chain: {}", chain))),
    }
}

/// Get list of supported chains
#[wasm_bindgen]
pub fn supported_chains() -> Vec<String> {
    vec![
        "bitcoin".to_string(),
        "ethereum".to_string(),
        "solana".to_string(),
        "cosmos".to_string(),
    ]
}

/// Auto-detect chain from transaction bytes (best effort)
#[wasm_bindgen]
pub fn auto_detect_chain(hex: &str) -> Result<String, JsValue> {
    let bytes = universal_decoder_core::hex::decode(hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))?;

    // Try decoders in order (most common first)
    if BitcoinDecoder.decode(&bytes).is_ok() {
        return Ok("bitcoin".to_string());
    }
    if EthereumDecoder.decode(&bytes).is_ok() {
        return Ok("ethereum".to_string());
    }
    if SolanaDecoder.decode(&bytes).is_ok() {
        return Ok("solana".to_string());
    }
    if CosmosDecoder.decode(&bytes).is_ok() {
        return Ok("cosmos".to_string());
    }

    Err(JsValue::from_str("Could not auto-detect chain"))
}

// ============================================================================
// Chain-specific decoder wrappers (reusing existing implementations)
// ============================================================================

fn decode_bitcoin_transaction(bytes: &[u8]) -> Result<DecodeResult, JsValue> {
    // Decode using existing BitcoinDecoder
    let tx: BitcoinTransaction = BitcoinDecoder
        .decode(bytes)
        .map_err(|e| JsValue::from_str(&format!("Bitcoin decode error: {}", e)))?;

    // Get TxIR using existing canonicalizer
    let tx_ir = tx
        .to_intermediate_representation()
        .map_err(|e| JsValue::from_str(&format!("TxIR conversion error: {}", e)))?;

    // Get canonical bytes (already implemented!)
    let canonical_bytes = tx_ir
        .to_canonical_bytes()
        .map_err(|e| JsValue::from_str(&format!("Canonical encoding error: {}", e)))?;

    // Get canonical hash (already implemented!)
    let canonical_hash = tx_ir
        .canonical_hash()
        .map_err(|e| JsValue::from_str(&format!("Hash error: {}", e)))?;

    // Serialize to JSON for display
    let json = serde_json::to_value(&tx_ir)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {}", e)))?;

    // Extract privacy features (Bitcoin is transparent by default)
    let privacy_features = extract_privacy_features(&tx_ir);
    let privacy_score = calculate_privacy_score(&tx_ir);

    Ok(DecodeResult {
        canonical_hex: universal_decoder_core::hex::encode(&canonical_bytes),
        canonical_hash: universal_decoder_core::hex::encode(&canonical_hash),
        json,
        privacy_features,
        privacy_score,
        chain_name: tx_ir.chain.chain_name().to_string(),
        chain_id: tx_ir.chain.chain_id(),
        transaction_type: format!(
            "{:?}",
            tx_ir
                .operations
                .first()
                .map(|o| &o.operation_type)
                .unwrap_or(&universal_decoder_core::OperationType::Transfer)
        ),
        canonical_size: canonical_bytes.len(),
    })
}

fn decode_ethereum_transaction(bytes: &[u8]) -> Result<DecodeResult, JsValue> {
    // Decode using existing EthereumDecoder
    let tx: EthereumTransaction = EthereumDecoder
        .decode(bytes)
        .map_err(|e| JsValue::from_str(&format!("Ethereum decode error: {}", e)))?;

    // Get TxIR using existing canonicalizer
    let tx_ir = tx
        .to_intermediate_representation()
        .map_err(|e| JsValue::from_str(&format!("TxIR conversion error: {}", e)))?;

    // Get canonical bytes (already implemented!)
    let canonical_bytes = tx_ir
        .to_canonical_bytes()
        .map_err(|e| JsValue::from_str(&format!("Canonical encoding error: {}", e)))?;

    // Get canonical hash (already implemented!)
    let canonical_hash = tx_ir
        .canonical_hash()
        .map_err(|e| JsValue::from_str(&format!("Hash error: {}", e)))?;

    // Serialize to JSON for display
    let json = serde_json::to_value(&tx_ir)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {}", e)))?;

    // Extract privacy features
    let privacy_features = extract_privacy_features(&tx_ir);
    let privacy_score = calculate_privacy_score(&tx_ir);

    Ok(DecodeResult {
        canonical_hex: universal_decoder_core::hex::encode(&canonical_bytes),
        canonical_hash: universal_decoder_core::hex::encode(&canonical_hash),
        json,
        privacy_features,
        privacy_score,
        chain_name: tx_ir.chain.chain_name().to_string(),
        chain_id: tx_ir.chain.chain_id(),
        transaction_type: format!(
            "{:?}",
            tx_ir
                .operations
                .first()
                .map(|o| &o.operation_type)
                .unwrap_or(&universal_decoder_core::OperationType::Transfer)
        ),
        canonical_size: canonical_bytes.len(),
    })
}

fn decode_solana_transaction(bytes: &[u8]) -> Result<DecodeResult, JsValue> {
    // Decode using existing SolanaDecoder
    let tx: SolanaTransaction = SolanaDecoder
        .decode(bytes)
        .map_err(|e| JsValue::from_str(&format!("Solana decode error: {}", e)))?;

    // Get TxIR using existing canonicalizer
    let tx_ir = tx
        .to_intermediate_representation()
        .map_err(|e| JsValue::from_str(&format!("TxIR conversion error: {}", e)))?;

    // Get canonical bytes (already implemented!)
    let canonical_bytes = tx_ir
        .to_canonical_bytes()
        .map_err(|e| JsValue::from_str(&format!("Canonical encoding error: {}", e)))?;

    // Get canonical hash (already implemented!)
    let canonical_hash = tx_ir
        .canonical_hash()
        .map_err(|e| JsValue::from_str(&format!("Hash error: {}", e)))?;

    // Serialize to JSON for display
    let json = serde_json::to_value(&tx_ir)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {}", e)))?;

    // Extract privacy features
    let privacy_features = extract_privacy_features(&tx_ir);
    let privacy_score = calculate_privacy_score(&tx_ir);

    Ok(DecodeResult {
        canonical_hex: universal_decoder_core::hex::encode(&canonical_bytes),
        canonical_hash: universal_decoder_core::hex::encode(&canonical_hash),
        json,
        privacy_features,
        privacy_score,
        chain_name: tx_ir.chain.chain_name().to_string(),
        chain_id: tx_ir.chain.chain_id(),
        transaction_type: format!(
            "{:?}",
            tx_ir
                .operations
                .first()
                .map(|o| &o.operation_type)
                .unwrap_or(&universal_decoder_core::OperationType::Transfer)
        ),
        canonical_size: canonical_bytes.len(),
    })
}

fn decode_cosmos_transaction(bytes: &[u8]) -> Result<DecodeResult, JsValue> {
    // Decode using existing CosmosDecoder
    let tx: CosmosTransaction = CosmosDecoder
        .decode(bytes)
        .map_err(|e| JsValue::from_str(&format!("Cosmos decode error: {}", e)))?;

    // Get TxIR using existing canonicalizer
    let tx_ir = tx
        .to_intermediate_representation()
        .map_err(|e| JsValue::from_str(&format!("TxIR conversion error: {}", e)))?;

    // Get canonical bytes (already implemented!)
    let canonical_bytes = tx_ir
        .to_canonical_bytes()
        .map_err(|e| JsValue::from_str(&format!("Canonical encoding error: {}", e)))?;

    // Get canonical hash (already implemented!)
    let canonical_hash = tx_ir
        .canonical_hash()
        .map_err(|e| JsValue::from_str(&format!("Hash error: {}", e)))?;

    // Serialize to JSON for display
    let json = serde_json::to_value(&tx_ir)
        .map_err(|e| JsValue::from_str(&format!("JSON error: {}", e)))?;

    // Extract privacy features
    let privacy_features = extract_privacy_features(&tx_ir);
    let privacy_score = calculate_privacy_score(&tx_ir);

    Ok(DecodeResult {
        canonical_hex: universal_decoder_core::hex::encode(&canonical_bytes),
        canonical_hash: universal_decoder_core::hex::encode(&canonical_hash),
        json,
        privacy_features,
        privacy_score,
        chain_name: tx_ir.chain.chain_name().to_string(),
        chain_id: tx_ir.chain.chain_id(),
        transaction_type: format!(
            "{:?}",
            tx_ir
                .operations
                .first()
                .map(|o| &o.operation_type)
                .unwrap_or(&universal_decoder_core::OperationType::Transfer)
        ),
        canonical_size: canonical_bytes.len(),
    })
}

// ============================================================================
// Privacy analysis helpers (simple heuristics for now)
// ============================================================================

fn extract_privacy_features(tx_ir: &universal_decoder_core::TxIR<1>) -> Vec<String> {
    let mut features = Vec::new();

    // Check for privacy-related metadata (from TxIR)
    if let Some(privacy) = &tx_ir.privacy {
        for feature in &privacy.features {
            features.push(format!("{:?}", feature));
        }
    }

    // If no explicit privacy features, mark as transparent
    if features.is_empty() {
        features.push("Fully Transparent".to_string());
    }

    features
}

fn calculate_privacy_score(tx_ir: &universal_decoder_core::TxIR<1>) -> u8 {
    match &tx_ir.privacy {
        None => 0, // Fully observable (Bitcoin, Ethereum)
        Some(p) => {
            // Simple heuristic: more privacy features = higher score
            let feature_count = p.features.len() as u8;
            (feature_count * 25).min(100)
        }
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
        assert!(chains.contains(&"solana".to_string()));
        assert!(chains.contains(&"cosmos".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_invalid_hex() {
        let result = decode_transaction("bitcoin", "not-hex");
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_unsupported_chain() {
        let result = decode_transaction("unknown-chain", "0123456789abcdef");
        assert!(result.is_err());
    }
}
