//! Error types for rate limiting

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during rate limiting
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// Request rate limit exceeded (too many requests per minute).
    /// Retriable: capacity refills over time.
    #[error("Request rate limit exceeded. Wait {wait_time:?} before retrying")]
    RequestRateLimitExceeded { wait_time: Duration },

    /// Token rate limit exceeded (too many tokens per minute).
    /// Retriable: capacity refills over time.
    #[error("Token rate limit exceeded. Wait {wait_time:?} before retrying")]
    TokenRateLimitExceeded { wait_time: Duration },

    /// Daily token limit exceeded. Not retriable until the UTC day rolls over.
    #[error("Daily token limit exceeded: {current} + {requested} > {limit}")]
    DailyTokenLimitExceeded {
        limit: u64,
        current: u64,
        requested: u64,
    },

    /// Monthly budget exceeded. Not retriable until the UTC month rolls over.
    /// Values are in microdollars (1 dollar = 1,000,000 microdollars).
    #[error("Monthly budget exceeded: ${} + ${} > ${}", *.current_microdollars as f64 / 1_000_000.0, *.requested_microdollars as f64 / 1_000_000.0, *.limit_microdollars as f64 / 1_000_000.0)]
    MonthlyBudgetExceeded {
        limit_microdollars: u64,
        current_microdollars: u64,
        requested_microdollars: u64,
    },

    /// The requested amount exceeds the bucket's total capacity, so the
    /// acquisition can NEVER succeed regardless of how long the caller
    /// waits. Not retriable.
    #[error("{dimension} request of {requested} can never succeed: bucket capacity is {capacity}")]
    ExceedsCapacity {
        dimension: &'static str,
        requested: u64,
        capacity: u64,
    },

    /// The rate limit configuration is invalid (e.g. a zero refill rate).
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Internal error: lock poisoned
    #[error("Internal error: lock poisoned (this is a bug)")]
    LockPoisoned,
}

impl RateLimitError {
    /// Check if this is a temporary error that can be retried
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            RateLimitError::RequestRateLimitExceeded { .. }
                | RateLimitError::TokenRateLimitExceeded { .. }
        )
    }

    /// Get the wait time for retriable errors
    pub fn wait_time(&self) -> Option<Duration> {
        match self {
            RateLimitError::RequestRateLimitExceeded { wait_time }
            | RateLimitError::TokenRateLimitExceeded { wait_time } => Some(*wait_time),
            _ => None,
        }
    }
}
