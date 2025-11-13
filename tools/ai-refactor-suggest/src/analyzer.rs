use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::decoder_info::DecoderInfo;
use crate::information_fetcher::InformationFetcher;
use crate::prompts::{get_latest_updates, PromptManager};
use crate::suggestions::RefactorSuggestion;

/// Configuration for the analyzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub enabled_categories: Vec<String>,
    pub min_priority: String,
    pub excluded_decoders: Vec<String>,
    /// Enable real-time information fetching from GitHub, crates.io, etc.
    #[serde(default = "default_true")]
    pub enable_live_updates: bool,
    /// GitHub personal access token for higher API rate limits (optional)
    pub github_token: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-5-20250929".to_string(),
            max_tokens: 4096,
            temperature: 0.3,
            enabled_categories: vec![
                "dependency".to_string(),
                "security".to_string(),
                "performance".to_string(),
                "testing".to_string(),
                "architecture".to_string(),
            ],
            min_priority: "low".to_string(),
            excluded_decoders: Vec::new(),
            enable_live_updates: true,
            github_token: None,
        }
    }
}

/// Anthropic API request structures
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    temperature: f32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response structures
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

/// Main analyzer using Claude API
pub struct RefactorAnalyzer {
    api_key: String,
    client: reqwest::Client,
    config: AnalyzerConfig,
    prompt_manager: PromptManager,
    information_fetcher: Option<InformationFetcher>,
}

impl RefactorAnalyzer {
    /// Create a new analyzer
    pub fn new(api_key: String, config_path: PathBuf, repo_root: PathBuf) -> Result<Self> {
        // Load configuration
        let config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config: {}", config_path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse config: {}", config_path.display()))?
        } else {
            AnalyzerConfig::default()
        };

        // Initialize HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        // Load prompts
        let prompts_dir = repo_root.join("scripts/refactor-prompts");
        let prompt_manager = PromptManager::load_from_directory(&prompts_dir)?;

        // Initialize information fetcher if enabled
        let information_fetcher = if config.enable_live_updates {
            let cache_dir = repo_root.join(".cache");
            let github_token = config
                .github_token
                .clone()
                .or_else(|| std::env::var("GITHUB_TOKEN").ok());

            match InformationFetcher::new(cache_dir, github_token) {
                Ok(fetcher) => Some(fetcher),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize information fetcher: {}", e);
                    eprintln!("Falling back to static updates");
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            api_key,
            client,
            config,
            prompt_manager,
            information_fetcher,
        })
    }

    /// Analyze a decoder and return suggestions
    pub async fn analyze_decoder(&self, decoder: &DecoderInfo) -> Result<Vec<RefactorSuggestion>> {
        // Check if excluded
        if self.config.excluded_decoders.contains(&decoder.name) {
            return Ok(Vec::new());
        }

        // Read source files (limit to 10 files to avoid token limits)
        let source_files = decoder
            .read_source_files(10)
            .context("Failed to read source files")?;

        // Get latest updates (use information fetcher if available)
        let latest_updates = if let Some(ref fetcher) = self.information_fetcher {
            match fetcher.fetch_all_updates(decoder).await {
                Ok(updates) => fetcher.format_updates(&updates),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to fetch live updates for {}: {}",
                        decoder.name, e
                    );
                    get_latest_updates(decoder)
                }
            }
        } else {
            get_latest_updates(decoder)
        };

        // Build prompt
        let prompt = self
            .prompt_manager
            .build_prompt(decoder, &source_files, &latest_updates);

        // Call Claude API
        let response = self
            .call_claude_api(&prompt)
            .await
            .context("Failed to call Claude API")?;

        // Parse response
        let suggestions = self
            .parse_response(decoder, &response)
            .context("Failed to parse Claude response")?;

        // Filter by enabled categories and priority
        let filtered: Vec<_> = suggestions
            .into_iter()
            .filter(|s| self.config.enabled_categories.contains(&s.category))
            .filter(|s| self.meets_priority_threshold(&s.priority))
            .collect();

        Ok(filtered)
    }

    async fn call_claude_api(&self, prompt: &str) -> Result<String> {
        const API_URL: &str = "https://api.anthropic.com/v1/messages";
        const API_VERSION: &str = "2023-06-01";

        let request = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        // Extract text from response
        for content in &api_response.content {
            if content.content_type == "text" {
                if let Some(ref text) = content.text {
                    return Ok(text.clone());
                }
            }
        }

        anyhow::bail!("No text content found in API response");
    }

    fn parse_response(
        &self,
        decoder: &DecoderInfo,
        response_text: &str,
    ) -> Result<Vec<RefactorSuggestion>> {
        // Try to extract JSON from response
        // Claude might wrap it in markdown code blocks or include explanation text

        // First try: extract JSON from markdown code block
        let json_text = if let Some(json) = Self::extract_json_from_markdown(response_text) {
            json
        } else if let Some(json) = Self::extract_json_array(response_text) {
            // Second try: find JSON array directly
            json
        } else {
            // Last resort: try the whole response
            response_text.trim().to_string()
        };

        // Parse JSON
        let suggestions_data: Vec<serde_json::Value> = serde_json::from_str(&json_text)
            .with_context(|| {
                format!(
                    "Failed to parse JSON response. Response text:\n{}",
                    &response_text[..response_text.len().min(500)]
                )
            })?;

        // Convert to RefactorSuggestion objects
        let mut suggestions = Vec::new();
        for item in suggestions_data {
            let suggestion = RefactorSuggestion {
                decoder: decoder.name.clone(),
                category: item
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("other")
                    .to_string(),
                priority: item
                    .get("priority")
                    .and_then(|v| v.as_str())
                    .unwrap_or("medium")
                    .to_string(),
                title: item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string(),
                description: item
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                code_location: item
                    .get("code_location")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                suggested_change: item
                    .get("suggested_change")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };
            suggestions.push(suggestion);
        }

        Ok(suggestions)
    }

    fn extract_json_from_markdown(text: &str) -> Option<String> {
        let re = Regex::new(r"```json\s*(\[.*?\])\s*```").ok()?;
        re.captures(text)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }

    fn extract_json_array(text: &str) -> Option<String> {
        let re = Regex::new(r"\[\s*\{.*?\}\s*\]").ok()?;
        re.captures(text)
            .and_then(|cap| cap.get(0).map(|m| m.as_str().to_string()))
    }

    fn meets_priority_threshold(&self, priority: &str) -> bool {
        let priority_order = |p: &str| match p {
            "high" => 2,
            "medium" => 1,
            "low" => 0,
            _ => -1,
        };

        priority_order(priority) >= priority_order(&self.config.min_priority)
    }
}
