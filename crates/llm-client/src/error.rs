//! Error types for the LLM client

use api_rate_limiter::RateLimitError;
use thiserror::Error;

/// Errors from LLM API calls.
#[derive(Debug, Error)]
pub enum LlmError {
    /// Local rate limit or budget exceeded (checked before the request is sent).
    #[error("rate limit: {0}")]
    RateLimit(#[from] RateLimitError),

    /// Network or transport failure.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// The API returned a non-success status.
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    /// The model refused the request for safety reasons
    /// (Anthropic `stop_reason: "refusal"`).
    #[error("model refused the request{}", .category.as_deref().map(|c| format!(" (category: {c})")).unwrap_or_default())]
    Refused { category: Option<String> },

    /// The response body could not be parsed.
    #[error("failed to parse response: {0}")]
    Parse(String),
}
