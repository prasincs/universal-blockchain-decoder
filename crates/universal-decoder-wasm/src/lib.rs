//! WebAssembly bindings for the Universal Blockchain Decoder.
//!
//! This crate provides a thin WASM wrapper around the existing decoder infrastructure,
//! exposing the decoder functionality to JavaScript/browser environments.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Import all decoders (reusing implementations!)
use decoder_algorand::AlgorandDecoder;
use decoder_aptos::AptosDecoder;
use decoder_arbitrum::ArbitrumDecoder;
use decoder_avalanche::CChainDecoder;
use decoder_bitcoin::BitcoinDecoder;
use decoder_bitcoin_cash::BitcoinCashDecoder;
use decoder_bnb::BnbDecoder;
use decoder_cardano::CardanoDecoder;
use decoder_cosmos::CosmosDecoder;
use decoder_dash::DashDecoder;
use decoder_dogecoin::DogecoinDecoder;
use decoder_ethereum::EthereumDecoder;
use decoder_litecoin::LitecoinDecoder;
use decoder_near::NearDecoder;
use decoder_optimism::OptimismDecoder;
use decoder_polkadot::PolkadotDecoder;
use decoder_polygon::PolygonDecoder;
use decoder_solana::SolanaDecoder;
use decoder_starknet::StarknetDecoder;
use decoder_stellar::StellarDecoder;
use decoder_sui::SuiDecoder;
use decoder_tron::TronDecoder;
use decoder_xrp::XrpDecoder;
use decoder_zcash::ZcashDecoder;
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

    /// Original transaction hash/ID from the blockchain
    pub tx_hash: String,

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
#[wasm_bindgen]
pub fn decode_transaction(chain: &str, hex: &str) -> Result<DecodeResult, JsValue> {
    let bytes = universal_decoder_core::hex::decode(hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid hex: {}", e)))?;

    match chain.to_lowercase().as_str() {
        // Bitcoin family
        "bitcoin" => decode_with::<BitcoinDecoder>(&bytes),
        "bitcoin-cash" | "bch" => decode_with::<BitcoinCashDecoder>(&bytes),
        "dogecoin" | "doge" => decode_with::<DogecoinDecoder>(&bytes),
        "litecoin" | "ltc" => decode_with::<LitecoinDecoder>(&bytes),
        "dash" => decode_with::<DashDecoder>(&bytes),
        "zcash" | "zec" => decode_with::<ZcashDecoder>(&bytes),

        // Ethereum and EVM chains
        "ethereum" | "eth" => decode_with::<EthereumDecoder>(&bytes),
        "polygon" | "matic" => decode_with::<PolygonDecoder>(&bytes),
        "arbitrum" | "arb" => decode_with::<ArbitrumDecoder>(&bytes),
        "optimism" | "op" => decode_with::<OptimismDecoder>(&bytes),
        "avalanche" | "avax" => decode_with::<CChainDecoder>(&bytes),
        "bnb" | "bsc" => decode_with::<BnbDecoder>(&bytes),

        // Other major chains
        "solana" | "sol" => decode_with::<SolanaDecoder>(&bytes),
        "cosmos" | "atom" => decode_with::<CosmosDecoder>(&bytes),
        "near" => decode_with::<NearDecoder>(&bytes),
        "aptos" | "apt" => decode_with::<AptosDecoder>(&bytes),
        "sui" => decode_with::<SuiDecoder>(&bytes),
        "algorand" | "algo" => decode_with::<AlgorandDecoder>(&bytes),
        "cardano" | "ada" => decode_with::<CardanoDecoder>(&bytes),
        "polkadot" | "dot" => decode_with::<PolkadotDecoder>(&bytes),
        "stellar" | "xlm" => decode_with::<StellarDecoder>(&bytes),
        "tron" | "trx" => decode_with::<TronDecoder>(&bytes),
        "starknet" => decode_with::<StarknetDecoder>(&bytes),
        "xrp" | "ripple" => decode_with::<XrpDecoder>(&bytes),

        _ => Err(JsValue::from_str(&format!("Unsupported chain: {}", chain))),
    }
}

/// List all supported blockchain names
#[wasm_bindgen]
pub fn supported_chains() -> Vec<String> {
    get_chains_metadata()
        .as_string()
        .and_then(|s| serde_json::from_str::<Vec<ChainMetadata>>(&s).ok())
        .map(|chains| chains.into_iter().map(|c| c.id).collect())
        .unwrap_or_default()
}

/// Get detailed metadata for all supported chains
#[wasm_bindgen]
pub fn get_chains_metadata() -> JsValue {
    let chains = vec![
        // Bitcoin family
        ChainMetadata {
            id: "bitcoin".to_string(),
            name: "Bitcoin".to_string(),
            family: "UTXO".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "bitcoin-cash".to_string(),
            name: "Bitcoin Cash".to_string(),
            family: "UTXO".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "dogecoin".to_string(),
            name: "Dogecoin".to_string(),
            family: "UTXO".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "litecoin".to_string(),
            name: "Litecoin".to_string(),
            family: "UTXO".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "dash".to_string(),
            name: "Dash".to_string(),
            family: "UTXO".to_string(),
            has_privacy: true,
        },
        ChainMetadata {
            id: "zcash".to_string(),
            name: "Zcash".to_string(),
            family: "Privacy".to_string(),
            has_privacy: true,
        },
        // Ethereum and EVM chains
        ChainMetadata {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "arbitrum".to_string(),
            name: "Arbitrum".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "optimism".to_string(),
            name: "Optimism".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "avalanche".to_string(),
            name: "Avalanche C-Chain".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "bnb".to_string(),
            name: "BNB Smart Chain".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        // Other major chains
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
        ChainMetadata {
            id: "near".to_string(),
            name: "NEAR".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "aptos".to_string(),
            name: "Aptos".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "sui".to_string(),
            name: "Sui".to_string(),
            family: "Object".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "algorand".to_string(),
            name: "Algorand".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "cardano".to_string(),
            name: "Cardano".to_string(),
            family: "UTXO".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "polkadot".to_string(),
            name: "Polkadot".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "stellar".to_string(),
            name: "Stellar".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "tron".to_string(),
            name: "Tron".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "starknet".to_string(),
            name: "StarkNet".to_string(),
            family: "Account".to_string(),
            has_privacy: false,
        },
        ChainMetadata {
            id: "xrp".to_string(),
            name: "XRP Ledger".to_string(),
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
    // Bitcoin family
    if BitcoinDecoder::decode(&bytes).is_ok() {
        return Ok("bitcoin".to_string());
    }
    // Ethereum and EVM (most common)
    if EthereumDecoder::decode(&bytes).is_ok() {
        return Ok("ethereum".to_string());
    }
    if PolygonDecoder::decode(&bytes).is_ok() {
        return Ok("polygon".to_string());
    }
    if ArbitrumDecoder::decode(&bytes).is_ok() {
        return Ok("arbitrum".to_string());
    }
    if OptimismDecoder::decode(&bytes).is_ok() {
        return Ok("optimism".to_string());
    }
    if CChainDecoder::decode(&bytes).is_ok() {
        return Ok("avalanche".to_string());
    }
    if BnbDecoder::decode(&bytes).is_ok() {
        return Ok("bnb".to_string());
    }
    // Other popular chains
    if SolanaDecoder::decode(&bytes).is_ok() {
        return Ok("solana".to_string());
    }
    if CosmosDecoder::decode(&bytes).is_ok() {
        return Ok("cosmos".to_string());
    }
    // Bitcoin variants
    if BitcoinCashDecoder::decode(&bytes).is_ok() {
        return Ok("bitcoin-cash".to_string());
    }
    if DogecoinDecoder::decode(&bytes).is_ok() {
        return Ok("dogecoin".to_string());
    }
    if LitecoinDecoder::decode(&bytes).is_ok() {
        return Ok("litecoin".to_string());
    }
    if DashDecoder::decode(&bytes).is_ok() {
        return Ok("dash".to_string());
    }
    if ZcashDecoder::decode(&bytes).is_ok() {
        return Ok("zcash".to_string());
    }
    // Other chains
    if NearDecoder::decode(&bytes).is_ok() {
        return Ok("near".to_string());
    }
    if AptosDecoder::decode(&bytes).is_ok() {
        return Ok("aptos".to_string());
    }
    if SuiDecoder::decode(&bytes).is_ok() {
        return Ok("sui".to_string());
    }
    if AlgorandDecoder::decode(&bytes).is_ok() {
        return Ok("algorand".to_string());
    }
    if CardanoDecoder::decode(&bytes).is_ok() {
        return Ok("cardano".to_string());
    }
    if PolkadotDecoder::decode(&bytes).is_ok() {
        return Ok("polkadot".to_string());
    }
    if StellarDecoder::decode(&bytes).is_ok() {
        return Ok("stellar".to_string());
    }
    if TronDecoder::decode(&bytes).is_ok() {
        return Ok("tron".to_string());
    }
    if StarknetDecoder::decode(&bytes).is_ok() {
        return Ok("starknet".to_string());
    }
    if XrpDecoder::decode(&bytes).is_ok() {
        return Ok("xrp".to_string());
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
        tx_hash: universal_decoder_core::hex::encode(&tx_ir.metadata.tx_hash),
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
            "inputs": tx_ir.state_deltas.inputs.iter().map(|input| json!({
                "prev_txid": universal_decoder_core::hex::encode(&input.prev_tx),
                "output_index": input.output_index,
                "value": input.value.value.to_string(),
                "decimals": input.value.decimals,
            })).collect::<Vec<_>>(),
            "outputs": tx_ir.state_deltas.outputs.iter().map(|output| json!({
                "index": output.index,
                "address": universal_decoder_core::hex::encode(&output.address.bytes),
                "address_readable": output.address.human_readable.as_ref().unwrap_or(&"".to_string()),
                "value": output.value.value.to_string(),
                "decimals": output.value.decimals,
            })).collect::<Vec<_>>(),
            "account_changes": tx_ir.state_deltas.account_changes.iter().map(|change| json!({
                "address": universal_decoder_core::hex::encode(&change.address.bytes),
                "address_readable": change.address.human_readable.as_ref().unwrap_or(&"".to_string()),
                "nonce": change.nonce,
                "balance_change": change.balance_change.to_string(),
                "storage_changes_count": change.storage_changes.len(),
            })).collect::<Vec<_>>(),
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

// ============================================================================
// Formal Verification Dashboard Data
// ============================================================================

/// Get Verus formal verification status for the dashboard
#[wasm_bindgen]
pub fn get_verification_status() -> JsValue {
    let verification_data = serde_json::json!({
        "overall": {
            "completed": 67,
            "total": 185,
            "percentage": 36
        },
        "phases": [
            {
                "id": "phase41",
                "name": "Phase 4.1: Core Library",
                "status": "complete",
                "vcs": 67,
                "total": 67,
                "modules": [
                    {
                        "id": "vt1",
                        "name": "VT-1: Amount Arithmetic Safety",
                        "vcs": 20,
                        "total": 20,
                        "status": "complete",
                        "critical": false,
                        "items": [
                            "checked_add overflow detection (5 VCs)",
                            "checked_sub underflow detection (4 VCs)",
                            "checked_mul overflow detection (6 VCs)",
                            "Decimal conversion correctness (5 VCs)"
                        ]
                    },
                    {
                        "id": "vt2",
                        "name": "VT-2: Canonicalization Determinism",
                        "vcs": 20,
                        "total": 20,
                        "status": "complete",
                        "critical": true,
                        "items": [
                            "to_canonical_bytes() determinism (8 VCs)",
                            "Borsh encoding panic-freedom (6 VCs)",
                            "Bounded output size (6 VCs)"
                        ]
                    },
                    {
                        "id": "vt3",
                        "name": "VT-3: Error Propagation Safety",
                        "vcs": 10,
                        "total": 10,
                        "status": "complete",
                        "critical": false,
                        "items": [
                            "Error conversion preserves info (4 VCs)",
                            "Error types exhaustive (3 VCs)",
                            "Error propagation panic-free (3 VCs)"
                        ]
                    },
                    {
                        "id": "vt4",
                        "name": "VT-4: Hook Execution Ordering",
                        "vcs": 12,
                        "total": 12,
                        "status": "complete",
                        "critical": false,
                        "items": [
                            "Priority-based ordering (5 VCs)",
                            "Failure propagation (4 VCs)",
                            "State consistency (3 VCs)"
                        ]
                    },
                    {
                        "id": "vt5",
                        "name": "VT-5: Version Isolation",
                        "vcs": 5,
                        "total": 5,
                        "status": "complete",
                        "critical": false,
                        "items": [
                            "Type-level distinction (3 VCs)",
                            "Version preservation (2 VCs)"
                        ]
                    }
                ]
            },
            {
                "id": "phase42",
                "name": "Phase 4.2: Bitcoin Decoder",
                "status": "planned",
                "vcs": 0,
                "total": 63,
                "modules": [
                    {
                        "id": "vt10",
                        "name": "VT-10: Script Parsing",
                        "vcs": 0,
                        "total": 15,
                        "status": "todo"
                    },
                    {
                        "id": "vt11",
                        "name": "VT-11: UTXO Validation",
                        "vcs": 0,
                        "total": 12,
                        "status": "todo"
                    },
                    {
                        "id": "vt12",
                        "name": "VT-12: SegWit Handling",
                        "vcs": 0,
                        "total": 10,
                        "status": "todo"
                    },
                    {
                        "id": "vt13",
                        "name": "VT-13: Signature Verification",
                        "vcs": 0,
                        "total": 18,
                        "status": "todo"
                    },
                    {
                        "id": "vt14",
                        "name": "VT-14: Address Validation",
                        "vcs": 0,
                        "total": 8,
                        "status": "todo"
                    }
                ]
            },
            {
                "id": "phase43",
                "name": "Phase 4.3: Ethereum Decoder",
                "status": "planned",
                "vcs": 0,
                "total": 55,
                "modules": [
                    {
                        "id": "vt20",
                        "name": "VT-20: RLP Encoding Safety",
                        "vcs": 0,
                        "total": 20,
                        "status": "todo"
                    },
                    {
                        "id": "vt21",
                        "name": "VT-21: EIP-155 Replay Protection",
                        "vcs": 0,
                        "total": 8,
                        "status": "todo"
                    },
                    {
                        "id": "vt22",
                        "name": "VT-22: Gas Calculation Bounds",
                        "vcs": 0,
                        "total": 15,
                        "status": "todo"
                    },
                    {
                        "id": "vt23",
                        "name": "VT-23: EIP-2930/1559 Transaction Types",
                        "vcs": 0,
                        "total": 12,
                        "status": "todo"
                    }
                ]
            }
        ],
        "properties": [
            {
                "name": "Panic-Freedom",
                "description": "Core library never panics on valid inputs",
                "impact": "No unexpected crashes in production",
                "vcs": "VT-1, VT-2, VT-3"
            },
            {
                "name": "Deterministic Serialization",
                "description": "Same transaction always produces same bytes",
                "impact": "Signature verification works reliably",
                "vcs": "VT-2.1, VT-2.2",
                "critical": true
            },
            {
                "name": "Injectivity (No Collisions)",
                "description": "Different transactions → different canonical bytes",
                "impact": "Combined with SHA-256 → collision resistance",
                "vcs": "VT-2.1"
            },
            {
                "name": "Overflow Safety",
                "description": "Arithmetic never overflows silently",
                "impact": "No integer overflow vulnerabilities",
                "vcs": "VT-1"
            },
            {
                "name": "Type Safety",
                "description": "TxIR<1> and TxIR<2> are distinct types",
                "impact": "No version confusion at compile time",
                "vcs": "VT-5"
            }
        ],
        "timeline": {
            "completed_weeks": 9,
            "total_weeks": 24,
            "remaining_weeks": 15
        }
    });

    serde_wasm_bindgen::to_value(&verification_data).unwrap_or(JsValue::NULL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_supported_chains() {
        let chains = supported_chains();
        assert!(chains.len() >= 24); // We have 24+ chains
        assert!(chains.contains(&"bitcoin".to_string()));
        assert!(chains.contains(&"ethereum".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_invalid_hex() {
        let result = decode_transaction("bitcoin", "not_hex");
        assert!(result.is_err());
    }
}
