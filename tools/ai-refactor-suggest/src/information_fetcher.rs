use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use octocrab::Octocrab;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::decoder_info::DecoderInfo;

/// Repository information organized by chain family
const FAMILY_REPOS: &[(&str, &[&str])] = &[
    // UTXO family: Bitcoin, Litecoin, Dogecoin, Cardano
    (
        "utxo",
        &[
            "bitcoin/bitcoin",              // Bitcoin Core
            "litecoin-project/litecoin",    // Litecoin
            "dogecoin/dogecoin",            // Dogecoin
            "input-output-hk/cardano-node", // Cardano
        ],
    ),
    // Account family: Ethereum, EVM chains
    (
        "account",
        &[
            "ethereum/EIPs",        // Ethereum Improvement Proposals
            "alloy-rs/alloy",       // Modern Ethereum Rust library
            "ethereum/go-ethereum", // Geth (reference for protocol updates)
        ],
    ),
    // Instruction family: Solana, Aptos, Sui
    (
        "instruction",
        &[
            "solana-labs/solana",    // Solana
            "aptos-labs/aptos-core", // Aptos
            "MystenLabs/sui",        // Sui
            "move-language/move",    // Move language
        ],
    ),
    // Other: XRP, Tron, Polkadot, NEAR, Cosmos, Stellar, Algorand
    (
        "other",
        &[
            "XRPLF/rippled",          // XRP Ledger
            "tronprotocol/java-tron", // Tron
            "paritytech/polkadot",    // Polkadot
            "near/nearcore",          // NEAR
            "cosmos/cosmos-sdk",      // Cosmos SDK
            "stellar/stellar-core",   // Stellar
            "algorand/go-algorand",   // Algorand
        ],
    ),
];

/// Important crates by chain family
const FAMILY_CRATES: &[(&str, &[&str])] = &[
    ("utxo", &["bitcoin", "bdk", "cardano-serialization-lib"]),
    (
        "account",
        &["alloy", "alloy-primitives", "alloy-rlp", "ethers"],
    ),
    (
        "instruction",
        &["solana-sdk", "aptos-sdk", "sui-sdk", "move-core-types"],
    ),
    (
        "other",
        &[
            "xrpl",
            "parity-scale-codec",
            "near-sdk",
            "cosmwasm-std",
            "stellar-base",
        ],
    ),
];

/// A GitHub release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub repo_name: String,
    pub tag_name: String,
    pub name: String,
    pub published_at: DateTime<Utc>,
    pub body: Option<String>,
    pub html_url: String,
}

/// An EIP status update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EipUpdate {
    pub number: u32,
    pub title: String,
    pub status: String,
    pub category: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// A dependency update from crates.io
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyUpdate {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub is_major_update: bool,
    pub changelog_url: Option<String>,
}

/// Protocol-specific update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolUpdate {
    pub source: String,
    pub title: String,
    pub summary: String,
    pub date: DateTime<Utc>,
    pub url: Option<String>,
}

/// Aggregated ecosystem updates
#[derive(Debug, Clone, Default)]
pub struct EcosystemUpdates {
    pub releases: Vec<Release>,
    pub eips: Vec<EipUpdate>,
    pub dependency_updates: Vec<DependencyUpdate>,
    pub protocol_updates: Vec<ProtocolUpdate>,
}

/// Cache entry with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    timestamp: DateTime<Utc>,
}

/// Information fetcher for real-time ecosystem updates
pub struct InformationFetcher {
    github_client: Octocrab,
    http_client: reqwest::Client,
    cache_dir: PathBuf,
    cache_duration: Duration,
}

impl InformationFetcher {
    /// Create a new information fetcher
    pub fn new(cache_dir: PathBuf, github_token: Option<String>) -> Result<Self> {
        let mut builder = Octocrab::builder();

        if let Some(token) = github_token {
            builder = builder.personal_token(token);
        }

        let github_client = builder.build().context("Failed to create GitHub client")?;

        let http_client = reqwest::Client::builder()
            .user_agent("universal-blockchain-decoder-ai-refactor")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            github_client,
            http_client,
            cache_dir,
            cache_duration: Duration::hours(6), // Cache for 6 hours
        })
    }

    /// Fetch all updates for a decoder based on its chain family
    pub async fn fetch_all_updates(&self, decoder: &DecoderInfo) -> Result<EcosystemUpdates> {
        let mut updates = EcosystemUpdates::default();

        // Fetch GitHub releases for this chain family
        if let Ok(releases) = self.fetch_family_releases(&decoder.family).await {
            updates.releases = releases;
        }

        // Fetch EIP updates (for account/EVM family)
        if decoder.family == "account" {
            if let Ok(eips) = self.fetch_recent_eips().await {
                updates.eips = eips;
            }
        }

        // Check for dependency updates
        if let Ok(dep_updates) = self.check_dependency_updates(decoder).await {
            updates.dependency_updates = dep_updates;
        }

        // Fetch protocol-specific updates
        if let Ok(protocol_updates) = self.fetch_protocol_updates(decoder).await {
            updates.protocol_updates = protocol_updates;
        }

        Ok(updates)
    }

    /// Fetch latest releases for a chain family from GitHub
    async fn fetch_family_releases(&self, family: &str) -> Result<Vec<Release>> {
        // Check cache first
        let cache_key = format!("releases_family_{}", family);
        if let Some(cached) = self.get_cached::<Vec<Release>>(&cache_key).await? {
            return Ok(cached);
        }

        let mut all_releases = Vec::new();

        // Find tracked repositories for this family
        for (family_name, repos) in FAMILY_REPOS {
            if *family_name == family {
                for repo in *repos {
                    let parts: Vec<&str> = repo.split('/').collect();
                    if parts.len() != 2 {
                        continue;
                    }

                    match self.fetch_repo_releases(parts[0], parts[1], repo).await {
                        Ok(releases) => all_releases.extend(releases),
                        Err(e) => {
                            eprintln!("Warning: Failed to fetch releases from {}: {}", repo, e);
                        }
                    }
                }
                break;
            }
        }

        // Only keep releases from last 90 days
        let cutoff = Utc::now() - Duration::days(90);
        all_releases.retain(|r| r.published_at > cutoff);

        // Sort by date (newest first)
        all_releases.sort_by_key(|r| std::cmp::Reverse(r.published_at));

        // Keep only top 10 most recent across all repos
        all_releases.truncate(10);

        // Cache the results
        self.cache(&cache_key, &all_releases).await?;

        Ok(all_releases)
    }

    /// Fetch releases from a specific repository
    async fn fetch_repo_releases(
        &self,
        owner: &str,
        repo: &str,
        full_name: &str,
    ) -> Result<Vec<Release>> {
        let releases = self
            .github_client
            .repos(owner, repo)
            .releases()
            .list()
            .per_page(5)
            .send()
            .await
            .context("Failed to fetch releases from GitHub")?;

        let mut result = Vec::new();
        for release in releases.items {
            result.push(Release {
                repo_name: full_name.to_string(),
                tag_name: release.tag_name,
                name: release.name.unwrap_or_default(),
                published_at: release.published_at.unwrap_or_else(Utc::now),
                body: release.body,
                html_url: release.html_url.to_string(),
            });
        }

        Ok(result)
    }

    /// Fetch recent EIP updates (Ethereum Improvement Proposals)
    async fn fetch_recent_eips(&self) -> Result<Vec<EipUpdate>> {
        // Check cache
        let cache_key = "eips_recent";
        if let Some(cached) = self.get_cached::<Vec<EipUpdate>>(cache_key).await? {
            return Ok(cached);
        }

        let mut eips = Vec::new();

        // Query for recently updated EIP files
        match self.fetch_eip_updates_from_github().await {
            Ok(updates) => eips.extend(updates),
            Err(e) => {
                eprintln!("Warning: Failed to fetch EIP updates: {}", e);
            }
        }

        // Cache the results
        self.cache(cache_key, &eips).await?;

        Ok(eips)
    }

    /// Fetch EIP updates from GitHub
    async fn fetch_eip_updates_from_github(&self) -> Result<Vec<EipUpdate>> {
        let commits = self
            .github_client
            .repos("ethereum", "EIPs")
            .list_commits()
            .path("EIPS")
            .per_page(30)
            .send()
            .await
            .context("Failed to fetch EIP commits")?;

        let mut eips = Vec::new();
        let cutoff = Utc::now() - Duration::days(30);

        for commit in commits.items {
            if let Some(commit_date) = commit.commit.author.and_then(|a| a.date) {
                if commit_date < cutoff {
                    continue;
                }

                // Parse EIP number from commit message
                if let Some(eip_info) = Self::parse_eip_from_commit(&commit.commit.message) {
                    eips.push(EipUpdate {
                        number: eip_info.0,
                        title: eip_info.1,
                        status: eip_info.2,
                        category: eip_info.3,
                        updated_at: commit_date,
                    });
                }
            }
        }

        // Deduplicate by EIP number (keep most recent)
        let mut seen = std::collections::HashSet::new();
        eips.retain(|eip| seen.insert(eip.number));

        eips.sort_by_key(|eip| std::cmp::Reverse(eip.updated_at));
        eips.truncate(10);

        Ok(eips)
    }

    /// Parse EIP information from commit message
    fn parse_eip_from_commit(message: &str) -> Option<(u32, String, String, Option<String>)> {
        // Simple regex to extract EIP number
        let re = regex::Regex::new(r"EIP-(\d+)").ok()?;
        let caps = re.captures(message)?;
        let number: u32 = caps.get(1)?.as_str().parse().ok()?;

        // Extract status if present
        let status = if message.to_lowercase().contains("final") {
            "Final".to_string()
        } else if message.to_lowercase().contains("last call") {
            "Last Call".to_string()
        } else if message.to_lowercase().contains("review") {
            "Review".to_string()
        } else {
            "Draft".to_string()
        };

        // Extract title (first 100 chars of commit message)
        let title = message.lines().next().unwrap_or(message);
        let title = if title.len() > 100 {
            format!("{}...", &title[..100])
        } else {
            title.to_string()
        };

        Some((number, title, status, None))
    }

    /// Check for dependency updates on crates.io
    async fn check_dependency_updates(
        &self,
        decoder: &DecoderInfo,
    ) -> Result<Vec<DependencyUpdate>> {
        let cache_key = format!("deps_{}", decoder.name);
        if let Some(cached) = self.get_cached::<Vec<DependencyUpdate>>(&cache_key).await? {
            return Ok(cached);
        }

        let mut updates = Vec::new();

        // Check important crates for this family
        for (family, crates) in FAMILY_CRATES {
            if decoder.family == *family {
                for crate_name in *crates {
                    // Check if this crate is in dependencies
                    if let Some(current_version) = decoder.dependencies.get(*crate_name) {
                        if let Ok(Some(update)) = self
                            .check_single_dependency(crate_name, current_version)
                            .await
                        {
                            updates.push(update);
                        }
                    }
                }
            }
        }

        // Also check all existing dependencies
        for (dep_name, current_version) in &decoder.dependencies {
            // Skip if already checked above or is workspace/path dependency
            if current_version.contains("workspace")
                || current_version.contains("path")
                || current_version.contains("git")
            {
                continue;
            }

            if updates.iter().any(|u| u.name == *dep_name) {
                continue;
            }

            if let Ok(Some(update)) = self
                .check_single_dependency(dep_name, current_version)
                .await
            {
                updates.push(update);
            }
        }

        self.cache(&cache_key, &updates).await?;

        Ok(updates)
    }

    /// Check a single dependency for updates
    async fn check_single_dependency(
        &self,
        dep_name: &str,
        current_version: &str,
    ) -> Result<Option<DependencyUpdate>> {
        match self.fetch_crate_info(dep_name).await {
            Ok(latest_version) => {
                if let Ok(current) = Version::parse(current_version) {
                    if let Ok(latest) = Version::parse(&latest_version) {
                        if latest > current {
                            let is_major = latest.major > current.major;
                            return Ok(Some(DependencyUpdate {
                                name: dep_name.to_string(),
                                current_version: current_version.to_string(),
                                latest_version: latest_version.clone(),
                                is_major_update: is_major,
                                changelog_url: Some(format!(
                                    "https://crates.io/crates/{}/versions",
                                    dep_name
                                )),
                            }));
                        }
                    }
                }
                Ok(None)
            }
            Err(e) => {
                eprintln!("Warning: Failed to check {} on crates.io: {}", dep_name, e);
                Ok(None)
            }
        }
    }

    /// Fetch latest version of a crate from crates.io
    async fn fetch_crate_info(&self, crate_name: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct CrateResponse {
            #[serde(rename = "crate")]
            crate_info: CrateInfo,
        }

        #[derive(Deserialize)]
        struct CrateInfo {
            max_stable_version: String,
        }

        let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch from crates.io")?;

        if !response.status().is_success() {
            anyhow::bail!("crates.io returned status {}", response.status());
        }

        let crate_response: CrateResponse = response
            .json()
            .await
            .context("Failed to parse crates.io response")?;

        Ok(crate_response.crate_info.max_stable_version)
    }

    /// Fetch protocol-specific updates (simplified version)
    async fn fetch_protocol_updates(&self, _decoder: &DecoderInfo) -> Result<Vec<ProtocolUpdate>> {
        // Placeholder for future enhancement:
        // - Parse RSS feeds from chain foundations
        // - Monitor governance proposals
        // - Track Discord/forum announcements
        // - Check for hard fork schedules

        Ok(Vec::new())
    }

    /// Format all updates into a markdown string
    pub fn format_updates(&self, updates: &EcosystemUpdates) -> String {
        let mut output = String::new();

        // Recent releases
        if !updates.releases.is_empty() {
            output.push_str("### Recent GitHub Releases\n\n");
            for release in updates.releases.iter().take(5) {
                let days_ago = (Utc::now() - release.published_at).num_days();
                output.push_str(&format!(
                    "- **{}** ({}) - {} days ago\n",
                    release.name.trim(),
                    release.repo_name,
                    days_ago
                ));
                if let Some(body) = &release.body {
                    // Extract first line of body
                    if let Some(first_line) = body.lines().next() {
                        let truncated = if first_line.len() > 80 {
                            format!("{}...", &first_line[..80])
                        } else {
                            first_line.to_string()
                        };
                        output.push_str(&format!("  {}\n", truncated));
                    }
                }
                output.push_str(&format!("  [View Release]({})\n", release.html_url));
                output.push('\n');
            }
        }

        // EIP updates
        if !updates.eips.is_empty() {
            output.push_str("### EIP Status Changes (Last 30 Days)\n\n");
            for eip in updates.eips.iter().take(5) {
                output.push_str(&format!(
                    "- **EIP-{}**: {} (Status: {})\n",
                    eip.number,
                    eip.title.lines().next().unwrap_or(""),
                    eip.status
                ));
            }
            output.push('\n');
        }

        // Dependency updates
        if !updates.dependency_updates.is_empty() {
            output.push_str("### Dependency Updates Available\n\n");
            for update in &updates.dependency_updates {
                let update_type = if update.is_major_update {
                    "⚠️ MAJOR"
                } else {
                    "minor/patch"
                };
                output.push_str(&format!(
                    "- **{}**: {} → {} ({})\n",
                    update.name, update.current_version, update.latest_version, update_type
                ));
                if let Some(url) = &update.changelog_url {
                    output.push_str(&format!("  [Changelog]({})\n", url));
                }
            }
            output.push('\n');
        }

        // Protocol updates
        if !updates.protocol_updates.is_empty() {
            output.push_str("### Protocol News\n\n");
            for update in updates.protocol_updates.iter().take(3) {
                output.push_str(&format!("- **{}**: {}\n", update.source, update.title));
                if !update.summary.is_empty() {
                    output.push_str(&format!("  {}\n", update.summary));
                }
            }
            output.push('\n');
        }

        if output.is_empty() {
            "No recent updates found in the last 90 days. Chain family tracking active.".to_string()
        } else {
            output
        }
    }

    /// Get cached data if it exists and is not expired
    async fn get_cached<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let cache_path = self.cache_dir.join("fetcher_cache");

        match cacache::read(&cache_path, key).await {
            Ok(data) => {
                let entry: CacheEntry<T> = serde_json::from_slice(&data)?;

                // Check if cache is still valid
                if Utc::now() - entry.timestamp < self.cache_duration {
                    Ok(Some(entry.data))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Cache data with timestamp
    async fn cache<T: Serialize>(&self, key: &str, data: &T) -> Result<()> {
        let cache_path = self.cache_dir.join("fetcher_cache");

        let entry = CacheEntry {
            data,
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_vec(&entry)?;

        cacache::write(&cache_path, key, serialized)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write cache: {}", e))?;

        Ok(())
    }
}
