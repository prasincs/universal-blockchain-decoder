//! Async rate limiter with automatic waiting

use crate::{RateLimitConfig, RateLimitError, RateLimiter, UsageStats};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// Async rate limiter that can wait for permits
#[derive(Debug, Clone)]
pub struct AsyncRateLimiter {
    inner: RateLimiter,
    max_wait: Duration,
}

/// Randomized 0–99 ms jitter derived from the clock's sub-second nanos,
/// to desynchronize concurrent clients without a rand dependency.
fn jitter() -> Duration {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    Duration::from_millis(u64::from(nanos) % 100)
}

impl AsyncRateLimiter {
    /// Create a new async rate limiter with the default 60-second deadline.
    pub fn new(config: RateLimitConfig) -> Result<Self, RateLimitError> {
        Self::with_max_wait(config, Duration::from_secs(60))
    }

    /// Create a new async rate limiter with a custom deadline.
    ///
    /// `max_wait` bounds the **total elapsed time** of one `acquire_wait`
    /// call, across all retries.
    pub fn with_max_wait(
        config: RateLimitConfig,
        max_wait: Duration,
    ) -> Result<Self, RateLimitError> {
        Ok(Self {
            inner: RateLimiter::new(config)?,
            max_wait,
        })
    }

    /// Try to acquire permits (non-blocking, like sync version)
    pub fn try_acquire(
        &self,
        requests: u32,
        tokens: u64,
        cost_microdollars: u64,
    ) -> Result<(), RateLimitError> {
        self.inner.acquire(requests, tokens, cost_microdollars)
    }

    /// Acquire permits, waiting if necessary.
    ///
    /// The deadline is absolute: total time spent waiting (including
    /// jitter) never exceeds `max_wait`, regardless of how many retries
    /// occur. Non-retriable errors — window limits, `ExceedsCapacity`,
    /// invalid configuration — fail immediately without waiting.
    ///
    /// # Parameters
    /// - `requests`: Number of requests (usually 1)
    /// - `tokens`: Number of tokens consumed
    /// - `cost_microdollars`: Cost in microdollars (1 cent = 10,000)
    pub async fn acquire_wait(
        &self,
        requests: u32,
        tokens: u64,
        cost_microdollars: u64,
    ) -> Result<(), RateLimitError> {
        let deadline = Instant::now() + self.max_wait;

        loop {
            match self.inner.acquire(requests, tokens, cost_microdollars) {
                Ok(()) => return Ok(()),
                Err(e) if !e.is_retriable() => return Err(e),
                Err(e) => {
                    // Sleep for the reported refill time plus jitter, but
                    // never past the absolute deadline. The 10 ms floor
                    // prevents a zero-wait busy loop.
                    let wait = e
                        .wait_time()
                        .unwrap_or(Duration::from_millis(100))
                        .max(Duration::from_millis(10))
                        + jitter();

                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if wait > remaining {
                        return Err(e);
                    }
                    sleep(wait).await;
                }
            }
        }
    }

    /// Reconcile a pre-flight estimate with actual usage — see
    /// [`RateLimiter::reconcile`]. Reconcile with zero actuals to cancel a
    /// reservation after a failed request.
    pub fn reconcile(
        &self,
        estimated_tokens: u64,
        actual_tokens: u64,
        estimated_cost_microdollars: u64,
        actual_cost_microdollars: u64,
    ) -> Result<(), RateLimitError> {
        self.inner.reconcile(
            estimated_tokens,
            actual_tokens,
            estimated_cost_microdollars,
            actual_cost_microdollars,
        )
    }

    /// Get current usage statistics
    pub fn stats(&self) -> Result<UsageStats, RateLimitError> {
        self.inner.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_acquire_immediate() {
        let config = RateLimitConfig::builder().requests_per_minute(10).build();

        let limiter = AsyncRateLimiter::new(config).unwrap();

        // Should succeed immediately
        assert!(limiter.acquire_wait(1, 0, 0).await.is_ok());
    }

    #[tokio::test]
    async fn test_async_acquire_wait() {
        let config = RateLimitConfig::builder()
            .requests_per_minute(60) // 1 per second
            .build();

        let limiter = AsyncRateLimiter::new(config).unwrap();

        // Consume all permits
        for _ in 0..60 {
            limiter.try_acquire(1, 0, 0).unwrap();
        }

        // Next request should wait ~1 second
        let start = std::time::Instant::now();
        assert!(limiter.acquire_wait(1, 0, 0).await.is_ok());
        let elapsed = start.elapsed();

        // Should have waited at least 0.8 seconds (allow some slack)
        assert!(
            elapsed >= Duration::from_millis(800),
            "Expected to wait ~1s, but waited {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_async_total_wait_bounded_by_deadline() {
        let config = RateLimitConfig::builder()
            .requests_per_minute(60) // refills 1 permit per second
            .build();

        // Deadline far shorter than the ~1s refill wait
        let limiter = AsyncRateLimiter::with_max_wait(config, Duration::from_millis(200)).unwrap();

        for _ in 0..60 {
            limiter.try_acquire(1, 0, 0).unwrap();
        }

        // Must give up without sleeping past the 200 ms deadline
        let start = std::time::Instant::now();
        let result = limiter.acquire_wait(1, 0, 0).await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(
            elapsed < Duration::from_millis(300),
            "Deadline not enforced: waited {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_async_hard_limit_fails_immediately() {
        let config = RateLimitConfig::builder().monthly_budget_cents(100).build();

        let limiter = AsyncRateLimiter::new(config).unwrap();

        // Exceed the $1.00 budget (1,000,000 microdollars)
        limiter.try_acquire(1, 0, 1_000_000).unwrap();

        // Should fail immediately, not wait
        let start = std::time::Instant::now();
        let result = limiter.acquire_wait(1, 0, 1).await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(
            elapsed < Duration::from_millis(100),
            "Should fail immediately, but took {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_async_impossible_request_fails_immediately() {
        let config = RateLimitConfig::builder().tokens_per_minute(100).build();

        let limiter = AsyncRateLimiter::new(config).unwrap();

        // Larger than the bucket can ever hold: no retry loop
        let start = std::time::Instant::now();
        let result = limiter.acquire_wait(1, 1000, 0).await;
        let elapsed = start.elapsed();

        assert!(matches!(
            result,
            Err(RateLimitError::ExceedsCapacity { .. })
        ));
        assert!(elapsed < Duration::from_millis(100));
    }
}
