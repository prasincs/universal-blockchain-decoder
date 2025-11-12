//! Test fixture loading utilities
//!
//! This module provides standardized ways to load and manage test fixtures
//! (real blockchain transaction data) for decoder testing.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use universal_decoder_core::hex;

/// A test fixture containing transaction data
///
/// Test fixtures are JSON files with standardized format containing
/// real blockchain transaction data for testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFixture {
    /// Human-readable description of this test case
    pub description: String,

    /// Chain identifier (e.g., "bitcoin", "ethereum")
    pub chain: String,

    /// Raw transaction bytes (hex-encoded)
    pub raw_hex: String,

    /// Expected properties after decoding (optional)
    #[serde(default)]
    pub expected: ExpectedProperties,

    /// Additional metadata
    #[serde(default)]
    pub metadata: FixtureMetadata,
}

/// Expected properties after decoding
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpectedProperties {
    /// Expected transaction hash (hex)
    pub tx_hash: Option<String>,

    /// Expected version
    pub version: Option<u32>,

    /// Expected number of inputs (UTXO chains)
    pub num_inputs: Option<usize>,

    /// Expected number of outputs (UTXO chains)
    pub num_outputs: Option<usize>,

    /// Expected sender address (Account chains)
    pub from_address: Option<String>,

    /// Expected recipient address
    pub to_address: Option<String>,

    /// Expected value/amount (in smallest unit, as string)
    pub value: Option<String>,

    /// Expected fee (in smallest unit, as string)
    pub fee: Option<String>,

    /// Whether this should decode successfully
    #[serde(default = "default_true")]
    pub should_decode: bool,

    /// Whether this is a SegWit transaction (Bitcoin)
    pub is_segwit: Option<bool>,

    /// Whether this is a coinbase transaction (Bitcoin)
    pub is_coinbase: Option<bool>,

    /// Transaction type (Ethereum: "legacy", "eip1559", etc.)
    pub tx_type: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Additional fixture metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixtureMetadata {
    /// Source of this test data (e.g., "bitcoin-core", "etherscan")
    pub source: Option<String>,

    /// Block number where this transaction appeared
    pub block_number: Option<u64>,

    /// Block hash where this transaction appeared
    pub block_hash: Option<String>,

    /// Testnet or mainnet
    pub network: Option<String>,

    /// URL to blockchain explorer for this transaction
    pub explorer_url: Option<String>,

    /// Tags for categorizing tests (e.g., ["segwit", "multisig"])
    #[serde(default)]
    pub tags: Vec<String>,
}

impl TestFixture {
    /// Get the raw transaction bytes
    ///
    /// Decodes the hex string to bytes.
    ///
    /// # Panics
    ///
    /// Panics if the hex string is invalid.
    pub fn raw_bytes(&self) -> Vec<u8> {
        hex::decode(&self.raw_hex).expect("Invalid hex in fixture")
    }

    /// Check if this fixture has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.metadata.tags.iter().any(|t| t == tag)
    }

    /// Check if this fixture should decode successfully
    pub fn should_decode(&self) -> bool {
        self.expected.should_decode
    }

    /// Get the expected transaction hash (if specified)
    pub fn expected_tx_hash(&self) -> Option<Vec<u8>> {
        self.expected
            .tx_hash
            .as_ref()
            .and_then(|h| hex::decode(h.trim_start_matches("0x")).ok())
    }
}

/// Load a single test fixture from a JSON file
///
/// # Arguments
///
/// * `path` - Path to the JSON fixture file
///
/// # Returns
///
/// The loaded test fixture
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::fixtures::load_fixture;
///
/// let fixture = load_fixture("tests/fixtures/bitcoin/genesis_coinbase.json");
/// let tx_bytes = fixture.raw_bytes();
/// assert_eq!(fixture.chain, "bitcoin");
/// ```
///
/// # Panics
///
/// Panics if the file doesn't exist or contains invalid JSON.
pub fn load_fixture<P: AsRef<Path>>(path: P) -> TestFixture {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read fixture file {:?}: {}", path, e));

    serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse fixture file {:?}: {}", path, e))
}

/// Load all test fixtures from a directory
///
/// Recursively scans the directory for `.json` files and loads them.
///
/// # Arguments
///
/// * `dir` - Path to directory containing fixture files
///
/// # Returns
///
/// Vector of all loaded fixtures
///
/// # Examples
///
/// ```rust,no_run
/// use decoder_test_utils::fixtures::load_fixtures_dir;
///
/// let fixtures = load_fixtures_dir("tests/fixtures/bitcoin");
/// for fixture in fixtures {
///     println!("Testing: {}", fixture.description);
/// }
/// ```
pub fn load_fixtures_dir<P: AsRef<Path>>(dir: P) -> Vec<TestFixture> {
    let dir = dir.as_ref();
    let mut fixtures = Vec::new();

    if !dir.exists() {
        return fixtures;
    }

    for entry in fs::read_dir(dir).expect("Failed to read fixtures directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            fixtures.push(load_fixture(&path));
        } else if path.is_dir() {
            // Recurse into subdirectories
            fixtures.extend(load_fixtures_dir(&path));
        }
    }

    fixtures
}

/// Filter fixtures by chain
pub fn filter_by_chain<'a>(fixtures: &'a [TestFixture], chain: &str) -> Vec<&'a TestFixture> {
    fixtures.iter().filter(|f| f.chain == chain).collect()
}

/// Filter fixtures by tag
pub fn filter_by_tag<'a>(fixtures: &'a [TestFixture], tag: &str) -> Vec<&'a TestFixture> {
    fixtures.iter().filter(|f| f.has_tag(tag)).collect()
}

/// Create a simple test fixture programmatically
///
/// Useful for creating fixtures in tests without external files.
///
/// # Examples
///
/// ```rust
/// use decoder_test_utils::fixtures::create_fixture;
///
/// let fixture = create_fixture(
///     "bitcoin",
///     "Test transaction",
///     "0100000001...",
/// );
/// ```
pub fn create_fixture(chain: &str, description: &str, raw_hex: &str) -> TestFixture {
    TestFixture {
        chain: chain.to_string(),
        description: description.to_string(),
        raw_hex: raw_hex.to_string(),
        expected: Default::default(),
        metadata: Default::default(),
    }
}

/// Helper to create a fixture with expected properties
pub fn create_fixture_with_expected(
    chain: &str,
    description: &str,
    raw_hex: &str,
    expected: ExpectedProperties,
) -> TestFixture {
    TestFixture {
        chain: chain.to_string(),
        description: description.to_string(),
        raw_hex: raw_hex.to_string(),
        expected,
        metadata: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_fixture() {
        let fixture = create_fixture("bitcoin", "Test", "deadbeef");
        assert_eq!(fixture.chain, "bitcoin");
        assert_eq!(fixture.description, "Test");
        assert_eq!(fixture.raw_hex, "deadbeef");
        assert_eq!(fixture.raw_bytes(), vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_fixture_tags() {
        let fixture = TestFixture {
            chain: "bitcoin".to_string(),
            description: "Test".to_string(),
            raw_hex: "00".to_string(),
            expected: Default::default(),
            metadata: FixtureMetadata {
                tags: vec!["segwit".to_string(), "multisig".to_string()],
                ..Default::default()
            },
        };

        assert!(fixture.has_tag("segwit"));
        assert!(fixture.has_tag("multisig"));
        assert!(!fixture.has_tag("taproot"));
    }

    #[test]
    fn test_filter_by_chain() {
        let fixtures = vec![
            create_fixture("bitcoin", "BTC test", "00"),
            create_fixture("ethereum", "ETH test", "00"),
            create_fixture("bitcoin", "BTC test 2", "00"),
        ];

        let btc_fixtures = filter_by_chain(&fixtures, "bitcoin");
        assert_eq!(btc_fixtures.len(), 2);

        let eth_fixtures = filter_by_chain(&fixtures, "ethereum");
        assert_eq!(eth_fixtures.len(), 1);
    }

    #[test]
    fn test_expected_properties_defaults() {
        let expected = ExpectedProperties::default();
        assert!(expected.should_decode);
        assert!(expected.tx_hash.is_none());
    }

    #[test]
    fn test_fixture_serialization() {
        let fixture = create_fixture("bitcoin", "Test", "deadbeef");
        let json = serde_json::to_string(&fixture).unwrap();
        let deserialized: TestFixture = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.chain, fixture.chain);
    }
}
