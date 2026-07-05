//! In-process rate limiter and budget guard using the token bucket algorithm
//!
//! # Overview
//!
//! This crate provides thread-safe, **in-process** rate limiting for API
//! calls with:
//! - Token bucket algorithm for smooth rate limiting
//! - Multiple limit types (requests/minute, tokens/minute, tokens per UTC
//!   day, monetary budget per UTC calendar month)
//! - Atomic multi-dimensional acquisition (a failed acquire consumes nothing)
//! - Reconciliation of pre-flight estimates against actual usage
//! - Both sync and async support
//!
//! # Scope and limitations
//!
//! All state lives in process memory. **Restarting the process resets every
//! counter, and each process gets its own independent budget.** This makes
//! the crate suitable as a pacing and runaway-loop guard within one process
//! (e.g. one CI job), not as an authoritative spend control. Pair it with
//! server-side enforcement (provider workspace spend caps and per-key rate
//! limits), which survives restarts and cannot be bypassed by clients.
//!
//! Daily and monthly windows are **UTC calendar** days and months. Provider
//! billing periods may differ.
//!
//! # Units
//!
//! Monetary amounts passed to [`RateLimiter::acquire`] and
//! [`RateLimiter::reconcile`] are **microdollars** (1 dollar = 1,000,000
//! microdollars; 1 cent = 10,000 microdollars), so cheap per-call costs are
//! not rounded up to whole cents. The monthly budget in [`RateLimitConfig`]
//! stays in cents for human-friendly configuration.
//!
//! # Example (Sync)
//!
//! ```rust
//! use api_rate_limiter::{RateLimiter, RateLimitConfig, MICRODOLLARS_PER_CENT};
//!
//! let config = RateLimitConfig::builder()
//!     .requests_per_minute(10)
//!     .tokens_per_day(100_000)
//!     .monthly_budget_cents(1000) // $10.00
//!     .build();
//!
//! let limiter = RateLimiter::new(config)?;
//!
//! // Before making an API call: 1 request, 500 tokens, $0.15
//! limiter.acquire(1, 500, 15 * MICRODOLLARS_PER_CENT)?;
//!
//! // Make API call... then reconcile with the provider-reported usage:
//! limiter.reconcile(500, 431, 15 * MICRODOLLARS_PER_CENT, 129_300)?;
//! # Ok::<(), api_rate_limiter::RateLimitError>(())
//! ```

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod error;
pub use error::RateLimitError;

#[cfg(feature = "async")]
mod async_limiter;
#[cfg(feature = "async")]
pub use async_limiter::AsyncRateLimiter;

/// Microdollars per cent (1 cent = 10,000 microdollars).
pub const MICRODOLLARS_PER_CENT: u64 = 10_000;

/// Microdollars per dollar.
pub const MICRODOLLARS_PER_DOLLAR: u64 = 1_000_000;

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute (None = unlimited)
    pub requests_per_minute: Option<u32>,

    /// Maximum tokens per minute (None = unlimited)
    pub tokens_per_minute: Option<u64>,

    /// Maximum tokens per UTC calendar day (None = unlimited)
    pub tokens_per_day: Option<u64>,

    /// Monthly budget in cents, per UTC calendar month (None = unlimited)
    pub monthly_budget_cents: Option<u64>,
}

impl RateLimitConfig {
    /// Create a new builder for configuration
    pub fn builder() -> RateLimitConfigBuilder {
        RateLimitConfigBuilder::default()
    }

    /// Validate the configuration.
    ///
    /// Zero rates are rejected: a bucket that never refills makes every
    /// wait-time calculation divide by zero. Use `None` for "unlimited"
    /// instead.
    pub fn validate(&self) -> Result<(), RateLimitError> {
        if self.requests_per_minute == Some(0) {
            return Err(RateLimitError::InvalidConfiguration(
                "requests_per_minute must be > 0 (use None for unlimited)".to_string(),
            ));
        }
        if self.tokens_per_minute == Some(0) {
            return Err(RateLimitError::InvalidConfiguration(
                "tokens_per_minute must be > 0 (use None for unlimited)".to_string(),
            ));
        }
        Ok(())
    }

    /// Create unlimited configuration (no limits)
    pub fn unlimited() -> Self {
        Self {
            requests_per_minute: None,
            tokens_per_minute: None,
            tokens_per_day: None,
            monthly_budget_cents: None,
        }
    }

    /// Create conservative configuration (10 RPM, $10/month)
    pub fn conservative() -> Self {
        Self {
            requests_per_minute: Some(10),
            tokens_per_minute: Some(10_000),
            tokens_per_day: Some(100_000),
            monthly_budget_cents: Some(1000), // $10.00
        }
    }

    /// Create moderate configuration (50 RPM, $100/month)
    pub fn moderate() -> Self {
        Self {
            requests_per_minute: Some(50),
            tokens_per_minute: Some(50_000),
            tokens_per_day: Some(1_000_000),
            monthly_budget_cents: Some(10_000), // $100.00
        }
    }
}

/// Builder for RateLimitConfig
#[derive(Default)]
pub struct RateLimitConfigBuilder {
    requests_per_minute: Option<u32>,
    tokens_per_minute: Option<u64>,
    tokens_per_day: Option<u64>,
    monthly_budget_cents: Option<u64>,
}

impl RateLimitConfigBuilder {
    pub fn requests_per_minute(mut self, limit: u32) -> Self {
        self.requests_per_minute = Some(limit);
        self
    }

    pub fn tokens_per_minute(mut self, limit: u64) -> Self {
        self.tokens_per_minute = Some(limit);
        self
    }

    pub fn tokens_per_day(mut self, limit: u64) -> Self {
        self.tokens_per_day = Some(limit);
        self
    }

    pub fn monthly_budget_cents(mut self, limit: u64) -> Self {
        self.monthly_budget_cents = Some(limit);
        self
    }

    pub fn build(self) -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: self.requests_per_minute,
            tokens_per_minute: self.tokens_per_minute,
            tokens_per_day: self.tokens_per_day,
            monthly_budget_cents: self.monthly_budget_cents,
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    /// Maximum capacity (tokens)
    capacity: f64,
    /// Current tokens available
    tokens: f64,
    /// Refill rate (tokens per second); validated to be > 0
    refill_rate: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = now;
    }

    /// Refill, then report whether `amount` is currently available.
    /// Does NOT consume — see `deduct`.
    fn has_capacity_for(&mut self, amount: f64) -> bool {
        self.refill();
        self.tokens >= amount
    }

    /// Deduct `amount`. Only call after `has_capacity_for` returned true
    /// under the same lock.
    fn deduct(&mut self, amount: f64) {
        self.tokens -= amount;
    }

    #[cfg(test)]
    fn try_consume(&mut self, amount: f64) -> bool {
        if self.has_capacity_for(amount) {
            self.deduct(amount);
            true
        } else {
            false
        }
    }

    fn time_until_available(&self, amount: f64) -> Duration {
        if self.tokens >= amount {
            return Duration::ZERO;
        }
        let needed = amount - self.tokens;
        let seconds = needed / self.refill_rate;
        Duration::from_secs_f64(seconds.max(0.0))
    }
}

/// Days since the Unix epoch → UTC (year, month, day).
///
/// Howard Hinnant's `civil_from_days` algorithm — exact for the entire
/// proleptic Gregorian calendar, no dependencies.
fn civil_from_days(days: i64) -> (i64, u64, u64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Current UTC day number and (year, month).
fn utc_day_and_month() -> (i64, (i64, u64)) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let day = secs.div_euclid(86_400);
    let (year, month, _) = civil_from_days(day);
    (day, (year, month))
}

/// Daily/monthly usage tracker, windowed on UTC calendar boundaries.
#[derive(Debug)]
struct UsageTracker {
    /// Token usage in the current UTC day
    daily_tokens: u64,
    /// Cost in the current UTC month, in microdollars
    monthly_cost_microdollars: u64,
    /// UTC day number the daily counter belongs to
    current_day: i64,
    /// UTC (year, month) the monthly counter belongs to
    current_month: (i64, u64),
}

impl UsageTracker {
    fn new() -> Self {
        let (day, month) = utc_day_and_month();
        Self {
            daily_tokens: 0,
            monthly_cost_microdollars: 0,
            current_day: day,
            current_month: month,
        }
    }

    /// Roll counters forward if the UTC day or month has changed.
    fn reset_if_needed(&mut self) {
        let (day, month) = utc_day_and_month();
        if day != self.current_day {
            self.daily_tokens = 0;
            self.current_day = day;
        }
        if month != self.current_month {
            self.monthly_cost_microdollars = 0;
            self.current_month = month;
        }
    }

    fn add_usage(&mut self, tokens: u64, cost_microdollars: u64) {
        self.daily_tokens = self.daily_tokens.saturating_add(tokens);
        self.monthly_cost_microdollars = self
            .monthly_cost_microdollars
            .saturating_add(cost_microdollars);
    }

    fn check_limits(
        &self,
        tokens: u64,
        cost_microdollars: u64,
        config: &RateLimitConfig,
    ) -> Result<(), RateLimitError> {
        // Check daily token limit
        if let Some(limit) = config.tokens_per_day {
            if self.daily_tokens.saturating_add(tokens) > limit {
                return Err(RateLimitError::DailyTokenLimitExceeded {
                    limit,
                    current: self.daily_tokens,
                    requested: tokens,
                });
            }
        }

        // Check monthly budget (configured in cents, tracked in microdollars)
        if let Some(limit_cents) = config.monthly_budget_cents {
            let limit_microdollars = limit_cents.saturating_mul(MICRODOLLARS_PER_CENT);
            if self
                .monthly_cost_microdollars
                .saturating_add(cost_microdollars)
                > limit_microdollars
            {
                return Err(RateLimitError::MonthlyBudgetExceeded {
                    limit_microdollars,
                    current_microdollars: self.monthly_cost_microdollars,
                    requested_microdollars: cost_microdollars,
                });
            }
        }

        Ok(())
    }
}

/// Thread-safe state for rate limiter
#[derive(Debug)]
struct RateLimiterState {
    config: RateLimitConfig,
    request_bucket: Option<TokenBucket>,
    token_bucket: Option<TokenBucket>,
    usage: UsageTracker,
}

impl RateLimiterState {
    fn new(config: RateLimitConfig) -> Self {
        let request_bucket = config.requests_per_minute.map(|rpm| {
            // Convert requests per minute to tokens per second
            let capacity = f64::from(rpm);
            let refill_rate = f64::from(rpm) / 60.0;
            TokenBucket::new(capacity, refill_rate)
        });

        let token_bucket = config.tokens_per_minute.map(|tpm| {
            let capacity = tpm as f64;
            let refill_rate = tpm as f64 / 60.0;
            TokenBucket::new(capacity, refill_rate)
        });

        Self {
            config,
            request_bucket,
            token_bucket,
            usage: UsageTracker::new(),
        }
    }
}

/// Synchronous rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    ///
    /// Returns [`RateLimitError::InvalidConfiguration`] for configurations
    /// that can never work (e.g. a zero refill rate).
    pub fn new(config: RateLimitConfig) -> Result<Self, RateLimitError> {
        config.validate()?;
        Ok(Self {
            state: Arc::new(Mutex::new(RateLimiterState::new(config))),
        })
    }

    /// Try to acquire permits for an API call.
    ///
    /// Acquisition is **atomic across all dimensions**: if any check fails,
    /// nothing is consumed from any bucket or counter.
    ///
    /// # Parameters
    /// - `requests`: Number of requests (usually 1)
    /// - `tokens`: Number of tokens consumed (estimate; reconcile later)
    /// - `cost_microdollars`: Cost in microdollars (1 cent = 10,000)
    ///
    /// # Errors
    /// - Retriable ([`RateLimitError::is_retriable`] is true): per-minute
    ///   rate capacity is exhausted but will refill
    /// - Non-retriable: daily/monthly window limits, or
    ///   [`RateLimitError::ExceedsCapacity`] when the request is larger than
    ///   a bucket's total capacity and could never succeed
    pub fn acquire(
        &self,
        requests: u32,
        tokens: u64,
        cost_microdollars: u64,
    ) -> Result<(), RateLimitError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| RateLimitError::LockPoisoned)?;

        // Roll usage windows BEFORE evaluating limits, so the first request
        // of a new UTC day/month is judged against fresh counters.
        state.usage.reset_if_needed();

        // Requests larger than a bucket's total capacity can never succeed —
        // report that as permanent, not as "retry later".
        if let Some(bucket) = &state.request_bucket {
            if f64::from(requests) > bucket.capacity {
                return Err(RateLimitError::ExceedsCapacity {
                    dimension: "requests",
                    requested: u64::from(requests),
                    capacity: bucket.capacity as u64,
                });
            }
        }
        if let Some(bucket) = &state.token_bucket {
            if tokens as f64 > bucket.capacity {
                return Err(RateLimitError::ExceedsCapacity {
                    dimension: "tokens",
                    requested: tokens,
                    capacity: bucket.capacity as u64,
                });
            }
        }

        // Check long-term window limits (fail fast, consumes nothing)
        state
            .usage
            .check_limits(tokens, cost_microdollars, &state.config)?;

        // Two-phase bucket acquisition under the single lock: verify every
        // bucket first, deduct only after all checks pass. A failed
        // acquisition must consume nothing.
        if let Some(bucket) = &mut state.request_bucket {
            if !bucket.has_capacity_for(f64::from(requests)) {
                let wait_time = bucket.time_until_available(f64::from(requests));
                return Err(RateLimitError::RequestRateLimitExceeded { wait_time });
            }
        }
        if let Some(bucket) = &mut state.token_bucket {
            if !bucket.has_capacity_for(tokens as f64) {
                let wait_time = bucket.time_until_available(tokens as f64);
                return Err(RateLimitError::TokenRateLimitExceeded { wait_time });
            }
        }

        // All checks passed — commit. Buckets only gain tokens over time, so
        // the capacity verified above is still present.
        if let Some(bucket) = &mut state.request_bucket {
            bucket.deduct(f64::from(requests));
        }
        if let Some(bucket) = &mut state.token_bucket {
            bucket.deduct(tokens as f64);
        }
        state.usage.add_usage(tokens, cost_microdollars);

        Ok(())
    }

    /// Reconcile a pre-flight estimate with the actual usage reported by the
    /// API response.
    ///
    /// `acquire` is called with estimates before the request; the provider's
    /// response carries exact token counts. Calling this replaces the
    /// estimated tokens/cost in the daily and monthly trackers with the
    /// actual values, so budgets reflect reality instead of worst-case
    /// guesses. To cancel a reservation after a failed request, reconcile
    /// with zero actuals.
    ///
    /// Token-bucket (rate) state is intentionally not adjusted — pacing is
    /// about request timing, and the request attempt already happened.
    ///
    /// Note: this is aggregate accounting without reservation identifiers —
    /// callers must pair each `acquire` with exactly one `reconcile`.
    pub fn reconcile(
        &self,
        estimated_tokens: u64,
        actual_tokens: u64,
        estimated_cost_microdollars: u64,
        actual_cost_microdollars: u64,
    ) -> Result<(), RateLimitError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| RateLimitError::LockPoisoned)?;

        state.usage.reset_if_needed();
        let usage = &mut state.usage;
        usage.daily_tokens = usage
            .daily_tokens
            .saturating_sub(estimated_tokens)
            .saturating_add(actual_tokens);
        usage.monthly_cost_microdollars = usage
            .monthly_cost_microdollars
            .saturating_sub(estimated_cost_microdollars)
            .saturating_add(actual_cost_microdollars);

        Ok(())
    }

    /// Get current usage statistics
    pub fn stats(&self) -> Result<UsageStats, RateLimitError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| RateLimitError::LockPoisoned)?;

        state.usage.reset_if_needed();

        Ok(UsageStats {
            daily_tokens: state.usage.daily_tokens,
            monthly_cost_microdollars: state.usage.monthly_cost_microdollars,
            requests_available: state
                .request_bucket
                .as_ref()
                .map(|b| b.tokens as u32)
                .unwrap_or(u32::MAX),
            tokens_available: state
                .token_bucket
                .as_ref()
                .map(|b| b.tokens as u64)
                .unwrap_or(u64::MAX),
        })
    }

    /// Reset all usage counters (useful for testing)
    #[cfg(test)]
    pub fn reset(&self) -> Result<(), RateLimitError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| RateLimitError::LockPoisoned)?;

        state.usage = UsageTracker::new();

        if let Some(bucket) = &mut state.request_bucket {
            bucket.tokens = bucket.capacity;
        }

        if let Some(bucket) = &mut state.token_bucket {
            bucket.tokens = bucket.capacity;
        }

        Ok(())
    }
}

/// Usage statistics
#[derive(Debug, Clone, Copy)]
pub struct UsageStats {
    /// Tokens used in the current UTC day
    pub daily_tokens: u64,
    /// Cost in the current UTC month, in microdollars
    pub monthly_cost_microdollars: u64,
    pub requests_available: u32,
    pub tokens_available: u64,
}

impl UsageStats {
    /// Monthly cost in dollars, for display.
    pub fn monthly_cost_dollars(&self) -> f64 {
        self.monthly_cost_microdollars as f64 / MICRODOLLARS_PER_DOLLAR as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10.0, 1.0); // 10 capacity, 1 token/sec

        // Can consume initial tokens
        assert!(bucket.try_consume(5.0));
        assert_eq!(bucket.tokens, 5.0);

        // Can consume remaining
        assert!(bucket.try_consume(5.0));
        assert!(bucket.tokens < 0.001); // Approximately 0 (floating point precision)

        // Cannot consume more
        assert!(!bucket.try_consume(1.0));
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10.0, 10.0); // 10 capacity, 10 tokens/sec

        // Consume all
        assert!(bucket.try_consume(10.0));

        // Wait 1 second
        std::thread::sleep(Duration::from_secs(1));

        // Should have refilled ~10 tokens
        bucket.refill();
        assert!(bucket.tokens >= 9.0 && bucket.tokens <= 10.0);
    }

    #[test]
    fn test_civil_from_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(18_262), (2020, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29)); // leap day
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn test_rate_limiter_request_limit() {
        let config = RateLimitConfig::builder().requests_per_minute(10).build();

        let limiter = RateLimiter::new(config).unwrap();

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(limiter.acquire(1, 0, 0).is_ok());
        }

        // 11th request should fail
        assert!(matches!(
            limiter.acquire(1, 0, 0),
            Err(RateLimitError::RequestRateLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_failed_acquisition_consumes_nothing() {
        // Request bucket has room; token bucket will be the failing dimension
        let config = RateLimitConfig::builder()
            .requests_per_minute(3)
            .tokens_per_minute(100)
            .build();

        let limiter = RateLimiter::new(config).unwrap();

        // Consume most of the token bucket (1 request, 80 tokens)
        limiter.acquire(1, 80, 0).unwrap();

        // This fails on the token dimension (needs 80, only ~20 left)...
        assert!(matches!(
            limiter.acquire(1, 80, 0),
            Err(RateLimitError::TokenRateLimitExceeded { .. })
        ));

        // ...and must NOT have consumed a request permit: two more small
        // requests still fit in the 3-request bucket. If the failed acquire
        // had leaked a request permit, the second of these would fail.
        assert!(limiter.acquire(1, 5, 0).is_ok());
        assert!(limiter.acquire(1, 5, 0).is_ok());
    }

    #[test]
    fn test_exceeds_capacity_is_permanent() {
        let config = RateLimitConfig::builder()
            .requests_per_minute(10)
            .tokens_per_minute(100)
            .build();

        let limiter = RateLimiter::new(config).unwrap();

        // More tokens than the bucket can ever hold: permanent, not retriable
        let err = limiter.acquire(1, 200, 0).unwrap_err();
        assert!(matches!(err, RateLimitError::ExceedsCapacity { .. }));
        assert!(!err.is_retriable());

        // Same for requests
        let err = limiter.acquire(11, 0, 0).unwrap_err();
        assert!(matches!(err, RateLimitError::ExceedsCapacity { .. }));

        // And nothing was consumed by either failure
        assert!(limiter.acquire(1, 100, 0).is_ok());
    }

    #[test]
    fn test_invalid_configuration_rejected() {
        let config = RateLimitConfig::builder().requests_per_minute(0).build();
        assert!(matches!(
            RateLimiter::new(config),
            Err(RateLimitError::InvalidConfiguration(_))
        ));

        let config = RateLimitConfig::builder().tokens_per_minute(0).build();
        assert!(matches!(
            RateLimiter::new(config),
            Err(RateLimitError::InvalidConfiguration(_))
        ));
    }

    #[test]
    fn test_rate_limiter_daily_token_limit() {
        let config = RateLimitConfig::builder().tokens_per_day(1000).build();

        let limiter = RateLimiter::new(config).unwrap();

        // Can consume up to 1000 tokens
        assert!(limiter.acquire(1, 500, 0).is_ok());
        assert!(limiter.acquire(1, 500, 0).is_ok());

        // Next request should fail
        assert!(matches!(
            limiter.acquire(1, 1, 0),
            Err(RateLimitError::DailyTokenLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_rate_limiter_monthly_budget() {
        let config = RateLimitConfig::builder()
            .monthly_budget_cents(100) // $1.00 = 1,000,000 microdollars
            .build();

        let limiter = RateLimiter::new(config).unwrap();

        // Can spend up to $1.00
        assert!(limiter.acquire(1, 0, 500_000).is_ok()); // $0.50
        assert!(limiter.acquire(1, 0, 500_000).is_ok()); // $0.50

        // Even one more microdollar should fail
        assert!(matches!(
            limiter.acquire(1, 0, 1),
            Err(RateLimitError::MonthlyBudgetExceeded { .. })
        ));
    }

    #[test]
    fn test_rate_limiter_stats() {
        let config = RateLimitConfig::builder()
            .requests_per_minute(10)
            .tokens_per_day(1000)
            .monthly_budget_cents(100)
            .build();

        let limiter = RateLimiter::new(config).unwrap();

        limiter.acquire(1, 100, 12_345).unwrap();

        let stats = limiter.stats().unwrap();
        assert_eq!(stats.daily_tokens, 100);
        assert_eq!(stats.monthly_cost_microdollars, 12_345);
        assert!((stats.monthly_cost_dollars() - 0.012_345).abs() < 1e-9);
    }

    #[test]
    fn test_reconcile_replaces_estimate_with_actual() {
        let config = RateLimitConfig::builder()
            .tokens_per_day(10_000)
            .monthly_budget_cents(1000)
            .build();

        let limiter = RateLimiter::new(config).unwrap();

        // Acquire with a worst-case estimate
        limiter.acquire(1, 5000, 1_000_000).unwrap();

        // Actual usage came in much lower
        limiter.reconcile(5000, 1200, 1_000_000, 250_000).unwrap();

        let stats = limiter.stats().unwrap();
        assert_eq!(stats.daily_tokens, 1200);
        assert_eq!(stats.monthly_cost_microdollars, 250_000);
    }

    #[test]
    fn test_reconcile_cancels_with_zero_actuals() {
        let limiter = RateLimiter::new(RateLimitConfig::unlimited()).unwrap();

        limiter.acquire(1, 5000, 1_000_000).unwrap();
        // Request failed in transport: cancel the reservation entirely
        limiter.reconcile(5000, 0, 1_000_000, 0).unwrap();

        let stats = limiter.stats().unwrap();
        assert_eq!(stats.daily_tokens, 0);
        assert_eq!(stats.monthly_cost_microdollars, 0);
    }

    #[test]
    fn test_reconcile_saturates_instead_of_underflowing() {
        let limiter = RateLimiter::new(RateLimitConfig::unlimited()).unwrap();

        // Reconciling more than was recorded must not panic or wrap
        limiter.reconcile(9999, 10, 9999, 1).unwrap();

        let stats = limiter.stats().unwrap();
        assert_eq!(stats.daily_tokens, 10);
        assert_eq!(stats.monthly_cost_microdollars, 1);
    }

    #[test]
    fn test_config_presets() {
        let conservative = RateLimitConfig::conservative();
        assert_eq!(conservative.requests_per_minute, Some(10));
        assert_eq!(conservative.monthly_budget_cents, Some(1000));

        let moderate = RateLimitConfig::moderate();
        assert_eq!(moderate.requests_per_minute, Some(50));
        assert_eq!(moderate.monthly_budget_cents, Some(10_000));

        let unlimited = RateLimitConfig::unlimited();
        assert!(unlimited.requests_per_minute.is_none());
        assert!(unlimited.monthly_budget_cents.is_none());
    }
}
