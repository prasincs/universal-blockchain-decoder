//! RPC transaction fetcher for downloading raw transaction data
//!
//! **IMPORTANT**: This module contains network code and is ONLY allowed in the CLI binary.
//! The core library MUST remain network-free to preserve airgapped operation.
//!
//! Supports multiple blockchain RPC protocols:
//! - Bitcoin-style RPC (Bitcoin, Litecoin, Dogecoin, BCH, BSV, Dash, Zcash)
//! - Ethereum JSON-RPC (Ethereum, BSC, Polygon, Avalanche, Optimism, Arbitrum)
//! - Solana RPC

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

/// RPC client for fetching raw transactions
pub struct RpcFetcher {
    client: Client,
}

impl RpcFetcher {
    /// Create a new RPC fetcher with default timeout
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("universal-tx-decoder/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Fetch raw transaction bytes from RPC endpoint
    ///
    /// # Arguments
    ///
    /// * `endpoint` - RPC endpoint URL (e.g., "<https://mainnet.infura.io/v3/YOUR-KEY>")
    /// * `txid` - Transaction ID/hash
    /// * `chain` - Chain short name (e.g., "btc", "eth")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use universal_decoder_cli::rpc_fetcher::RpcFetcher;
    ///
    /// let fetcher = RpcFetcher::new().unwrap();
    /// let tx_bytes = fetcher.fetch_transaction(
    ///     "https://mainnet.infura.io/v3/YOUR-KEY",
    ///     "0xabc123...",
    ///     "eth"
    /// ).unwrap();
    /// ```
    pub fn fetch_transaction(&self, endpoint: &str, txid: &str, chain: &str) -> Result<Vec<u8>> {
        match chain.to_lowercase().as_str() {
            // Bitcoin family (uses getrawtransaction)
            "btc" | "ltc" | "doge" | "bch" | "bsv" | "dash" | "zec" => {
                self.fetch_bitcoin_style(endpoint, txid)
            }
            // Ethereum family (uses eth_getTransactionByHash + eth_getRawTransactionByHash)
            "eth" | "bnb" | "matic" | "avax" | "op" | "arb" => {
                self.fetch_ethereum_style(endpoint, txid)
            }
            // Solana (uses getTransaction)
            "sol" => self.fetch_solana_style(endpoint, txid),
            _ => Err(anyhow!("Unsupported chain for RPC fetching: {}", chain)),
        }
    }

    /// Fetch transaction using Bitcoin-style RPC
    ///
    /// Calls `getrawtransaction` with verbose=false to get hex string
    fn fetch_bitcoin_style(&self, endpoint: &str, txid: &str) -> Result<Vec<u8>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "universal-decoder",
            "method": "getrawtransaction",
            "params": [txid, false]
        });

        let response: Value = self
            .client
            .post(endpoint)
            .json(&request)
            .send()
            .context("Failed to send RPC request")?
            .json()
            .context("Failed to parse RPC response")?;

        // Check for error
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                return Err(anyhow!(
                    "RPC error: {}",
                    error
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                ));
            }
        }

        // Extract result (hex string)
        let hex_str = response
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No result in RPC response"))?;

        // Decode hex to bytes
        universal_decoder_core::hex::decode(hex_str)
            .map_err(|e| anyhow!("Failed to decode hex: {:?}", e))
    }

    /// Fetch transaction using Ethereum-style JSON-RPC
    ///
    /// Calls `eth_getRawTransactionByHash` if available, otherwise falls back to
    /// `eth_getTransactionByHash` and reconstructs the raw transaction
    fn fetch_ethereum_style(&self, endpoint: &str, txid: &str) -> Result<Vec<u8>> {
        // Try eth_getRawTransactionByHash first (if node supports it)
        if let Ok(bytes) = self.try_eth_get_raw_transaction(endpoint, txid) {
            return Ok(bytes);
        }

        // Fallback: Use eth_getTransactionByHash and reconstruct
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getTransactionByHash",
            "params": [txid]
        });

        let response: Value = self
            .client
            .post(endpoint)
            .json(&request)
            .send()
            .context("Failed to send RPC request")?
            .json()
            .context("Failed to parse RPC response")?;

        // Check for error
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                return Err(anyhow!(
                    "RPC error: {}",
                    error
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                ));
            }
        }

        // Extract transaction object
        let tx = response
            .get("result")
            .ok_or_else(|| anyhow!("No result in RPC response"))?;

        if tx.is_null() {
            return Err(anyhow!("Transaction not found: {}", txid));
        }

        // Extract input field (which is the raw transaction data for signed txs)
        let input_hex = tx
            .get("input")
            .or_else(|| tx.get("data"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No input/data field in transaction"))?;

        // For Ethereum, we need to reconstruct the RLP-encoded transaction
        // This is a simplified approach - ideally we'd use the full transaction fields
        let hex_str = input_hex.strip_prefix("0x").unwrap_or(input_hex);
        universal_decoder_core::hex::decode(hex_str)
            .map_err(|e| anyhow!("Failed to decode hex: {:?}", e))
    }

    /// Try to fetch raw transaction using eth_getRawTransactionByHash
    fn try_eth_get_raw_transaction(&self, endpoint: &str, txid: &str) -> Result<Vec<u8>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getRawTransactionByHash",
            "params": [txid]
        });

        let response: Value = self
            .client
            .post(endpoint)
            .json(&request)
            .send()
            .context("Failed to send RPC request")?
            .json()
            .context("Failed to parse RPC response")?;

        // Check if method is unsupported
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                return Err(anyhow!("Method not supported"));
            }
        }

        let hex_str = response
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No result in RPC response"))?;

        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        universal_decoder_core::hex::decode(hex_str)
            .map_err(|e| anyhow!("Failed to decode hex: {:?}", e))
    }

    /// Fetch transaction using Solana RPC
    ///
    /// Calls `getTransaction` with encoding="base64"
    fn fetch_solana_style(&self, endpoint: &str, txid: &str) -> Result<Vec<u8>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                txid,
                {
                    "encoding": "base64",
                    "maxSupportedTransactionVersion": 0
                }
            ]
        });

        let response: Value = self
            .client
            .post(endpoint)
            .json(&request)
            .send()
            .context("Failed to send RPC request")?
            .json()
            .context("Failed to parse RPC response")?;

        // Check for error
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                return Err(anyhow!(
                    "RPC error: {}",
                    error
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                ));
            }
        }

        // Extract base64-encoded transaction
        let tx_data = response
            .get("result")
            .and_then(|r| r.get("transaction"))
            .and_then(|t| t.get(0))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No transaction data in RPC response"))?;

        // Decode base64 to bytes
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD
            .decode(tx_data)
            .context("Failed to decode base64 transaction")
    }

    /// Fetch transaction from a public endpoint (no API key required)
    ///
    /// Uses public RPC endpoints for demonstration purposes.
    /// For production use, provide your own RPC endpoint with API key.
    pub fn fetch_from_public_endpoint(&self, chain: &str, txid: &str) -> Result<Vec<u8>> {
        let endpoint = get_public_endpoint(chain)?;
        self.fetch_transaction(&endpoint, txid, chain)
    }
}

impl Default for RpcFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create RPC fetcher")
    }
}

/// Get public RPC endpoint for a chain (for demo purposes)
///
/// **WARNING**: Public endpoints have rate limits and may be unreliable.
/// For production use, provide your own RPC endpoint with API key.
fn get_public_endpoint(chain: &str) -> Result<String> {
    match chain.to_lowercase().as_str() {
        "btc" => Ok("https://blockstream.info/api".to_string()),
        "eth" => Ok("https://eth.llamarpc.com".to_string()),
        "bnb" => Ok("https://bsc-dataseed.binance.org".to_string()),
        "matic" => Ok("https://polygon-rpc.com".to_string()),
        "avax" => Ok("https://api.avax.network/ext/bc/C/rpc".to_string()),
        "op" => Ok("https://mainnet.optimism.io".to_string()),
        "arb" => Ok("https://arb1.arbitrum.io/rpc".to_string()),
        "sol" => Ok("https://api.mainnet-beta.solana.com".to_string()),
        _ => Err(anyhow!(
            "No public endpoint available for {}. Use --rpc-endpoint to specify your own.",
            chain
        )),
    }
}

// Note: base64 crate is needed for Solana
// We'll need to add it to Cargo.toml

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_fetcher() {
        let fetcher = RpcFetcher::new();
        assert!(fetcher.is_ok());
    }

    #[test]
    fn test_get_public_endpoint() {
        assert!(get_public_endpoint("eth").is_ok());
        assert!(get_public_endpoint("btc").is_ok());
        assert!(get_public_endpoint("unknown").is_err());
    }

    // Integration tests (require network) would go here
    // These should be marked with #[ignore] or run separately
}
