//! Explorer URL parser for extracting chain and transaction ID
//!
//! Supports popular block explorers across multiple chains:
//! - Bitcoin: blockchain.com, blockchair.com, mempool.space
//! - Ethereum: etherscan.io, polygonscan.com, bscscan.com, etc.
//! - Generic: Any URL with txid as last path segment

use anyhow::{anyhow, Context, Result};
use url::Url;

/// Parsed explorer URL with chain and transaction ID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerUrl {
    /// Chain short name (e.g., "btc", "eth", "matic")
    pub chain: String,
    /// Transaction ID/hash
    pub txid: String,
    /// Original URL
    pub original_url: String,
}

impl ExplorerUrl {
    /// Parse an explorer URL to extract chain and txid
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_decoder_cli::explorer_parser::ExplorerUrl;
    ///
    /// let parsed = ExplorerUrl::parse("https://etherscan.io/tx/0xabc123").unwrap();
    /// assert_eq!(parsed.chain, "eth");
    /// assert_eq!(parsed.txid, "0xabc123");
    /// ```
    pub fn parse(url_str: &str) -> Result<Self> {
        let url = Url::parse(url_str).context("Invalid URL")?;
        let host = url.host_str().ok_or_else(|| anyhow!("No host in URL"))?;
        let path = url.path();

        // Detect chain from domain
        let chain = detect_chain_from_host(host)?;

        // Extract txid from path
        let txid = extract_txid_from_path(path)?;

        Ok(ExplorerUrl {
            chain,
            txid,
            original_url: url_str.to_string(),
        })
    }
}

/// Detect blockchain from explorer domain
fn detect_chain_from_host(host: &str) -> Result<String> {
    let host_lower = host.to_lowercase();

    // Bitcoin explorers
    if host_lower.contains("blockchain.com")
        || host_lower.contains("blockchair.com/bitcoin")
        || host_lower.contains("mempool.space")
        || host_lower.contains("blockstream.info")
    {
        return Ok("btc".to_string());
    }

    // Ethereum explorers
    if host_lower.contains("etherscan.io") {
        return Ok("eth".to_string());
    }

    // BSC (BNB Smart Chain)
    if host_lower.contains("bscscan.com") {
        return Ok("bnb".to_string());
    }

    // Polygon
    if host_lower.contains("polygonscan.com") {
        return Ok("matic".to_string());
    }

    // Avalanche
    if host_lower.contains("snowtrace.io") || host_lower.contains("avascan.info") {
        return Ok("avax".to_string());
    }

    // Optimism
    if host_lower.contains("optimistic.etherscan.io") || host_lower.contains("optimism.io") {
        return Ok("op".to_string());
    }

    // Arbitrum
    if host_lower.contains("arbiscan.io") || host_lower.contains("arbitrum.io") {
        return Ok("arb".to_string());
    }

    // Litecoin
    if host_lower.contains("blockchair.com/litecoin")
        || host_lower.contains("litecoinspace.org")
        || host_lower.contains("ltc.com")
    {
        return Ok("ltc".to_string());
    }

    // Dogecoin
    if host_lower.contains("blockchair.com/dogecoin")
        || host_lower.contains("dogechain.info")
        || host_lower.contains("dogecoin.com")
    {
        return Ok("doge".to_string());
    }

    // Bitcoin Cash
    if host_lower.contains("blockchair.com/bitcoin-cash") || host_lower.contains("bch.com") {
        return Ok("bch".to_string());
    }

    // Dash
    if host_lower.contains("blockchair.com/dash") || host_lower.contains("dash.org") {
        return Ok("dash".to_string());
    }

    // Zcash
    if host_lower.contains("blockchair.com/zcash")
        || host_lower.contains("zcash.blockexplorer.com")
        || host_lower.contains("zcha.in")
    {
        return Ok("zec".to_string());
    }

    // Solana
    if host_lower.contains("solscan.io")
        || host_lower.contains("explorer.solana.com")
        || host_lower.contains("solana.fm")
    {
        return Ok("sol".to_string());
    }

    Err(anyhow!(
        "Unsupported explorer domain: {}. Use --chain to specify manually.",
        host
    ))
}

/// Extract transaction ID from URL path
fn extract_txid_from_path(path: &str) -> Result<String> {
    // Common patterns:
    // - /tx/{txid}
    // - /transaction/{txid}
    // - /btc/tx/{txid}
    // - /{txid} (generic)

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.is_empty() {
        return Err(anyhow!("No path segments in URL"));
    }

    // Look for txid after "tx" or "transaction" keyword
    for i in 0..segments.len() {
        if (segments[i] == "tx" || segments[i] == "transaction") && i + 1 < segments.len() {
            return Ok(segments[i + 1].to_string());
        }
    }

    // Fallback: use last path segment as txid
    let last_segment = segments.last().unwrap();

    // Validate txid format (hex string with optional 0x prefix)
    if is_valid_txid(last_segment) {
        Ok(last_segment.to_string())
    } else {
        Err(anyhow!(
            "Could not extract valid transaction ID from path: {}",
            path
        ))
    }
}

/// Check if a string looks like a valid transaction ID
fn is_valid_txid(s: &str) -> bool {
    // Remove 0x prefix if present
    let hex_str = s.strip_prefix("0x").unwrap_or(s);

    // Must be hex and reasonable length (32-128 chars typical)
    !hex_str.is_empty() && hex_str.len() >= 32 && hex_str.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_etherscan_url() {
        let url = "https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let parsed = ExplorerUrl::parse(url).unwrap();
        assert_eq!(parsed.chain, "eth");
        assert_eq!(
            parsed.txid,
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }

    #[test]
    fn test_parse_blockchain_com_url() {
        let url =
            "https://www.blockchain.com/btc/tx/abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let parsed = ExplorerUrl::parse(url).unwrap();
        assert_eq!(parsed.chain, "btc");
        assert_eq!(
            parsed.txid,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        );
    }

    #[test]
    fn test_parse_polygonscan_url() {
        let url = "https://polygonscan.com/tx/0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let parsed = ExplorerUrl::parse(url).unwrap();
        assert_eq!(parsed.chain, "matic");
        assert_eq!(
            parsed.txid,
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        );
    }

    #[test]
    fn test_parse_bscscan_url() {
        let url = "https://bscscan.com/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let parsed = ExplorerUrl::parse(url).unwrap();
        assert_eq!(parsed.chain, "bnb");
    }

    #[test]
    fn test_parse_mempool_space_url() {
        let url = "https://mempool.space/tx/1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let parsed = ExplorerUrl::parse(url).unwrap();
        assert_eq!(parsed.chain, "btc");
    }

    #[test]
    fn test_invalid_url() {
        let result = ExplorerUrl::parse("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_explorer() {
        let result = ExplorerUrl::parse("https://unknown-explorer.com/tx/abc123");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_txid() {
        // Valid txids
        assert!(is_valid_txid(
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        ));
        assert!(is_valid_txid(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        ));

        // Invalid txids
        assert!(!is_valid_txid("too_short"));
        assert!(!is_valid_txid("not_hex_chars!@#$"));
        assert!(!is_valid_txid(""));
    }
}
