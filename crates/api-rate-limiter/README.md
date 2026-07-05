# api-rate-limiter

In-process rate limiter and budget guard using the token bucket algorithm.

## Scope — read this first

All state lives in process memory. **Restarting the process resets every
counter, and each process gets its own independent budget.** This crate is a
pacing and runaway-loop guard for a single process (e.g. one CI job) — it is
**not** an authoritative spend control. Pair it with server-side enforcement
(provider workspace spend caps, per-key rate limits), which survives restarts
and cannot be bypassed by clients.

This crate is internal workspace tooling (`publish = false`). For generic
rate limiting needs, prefer established crates such as `governor`; the
differentiated part of this crate is the combination of rate quotas, token
quotas, monetary budgets, and estimate/actual reconciliation for LLM calls.

## Features

- **Token bucket algorithm** with per-minute request and token rates
- **Window limits**: tokens per UTC calendar day, monetary budget per UTC
  calendar month (provider billing periods may differ)
- **Atomic acquisition**: a failed multi-dimensional acquire consumes
  nothing from any bucket or counter
- **Reconciliation**: replace pre-flight estimates with the actual usage the
  provider reported; cancel a reservation by reconciling with zero actuals
- **Typed failure modes**: retriable rate limits (with wait time) vs.
  permanent failures (`ExceedsCapacity`, window limits,
  `InvalidConfiguration`)
- **Sync and async** (`async` feature); the async `acquire_wait` enforces
  `max_wait` as an absolute deadline across all retries

## Units

Monetary amounts in `acquire`/`reconcile` are **microdollars**
(1 dollar = 1,000,000 µ$; 1 cent = 10,000 µ$) so cheap calls are not rounded
up to whole cents. The monthly budget in `RateLimitConfig` stays in **cents**
for human-friendly configuration. `MICRODOLLARS_PER_CENT` and
`MICRODOLLARS_PER_DOLLAR` constants are exported.

## Usage

```rust
use api_rate_limiter::{RateLimiter, RateLimitConfig, MICRODOLLARS_PER_CENT};

let config = RateLimitConfig::builder()
    .requests_per_minute(10)
    .tokens_per_day(100_000)
    .monthly_budget_cents(1000) // $10.00
    .build();

let limiter = RateLimiter::new(config)?; // validates the configuration

// Reserve the pre-flight estimate: 1 request, 500 tokens, $0.15
limiter.acquire(1, 500, 15 * MICRODOLLARS_PER_CENT)?;

// ... make the API call, read response usage ...

// Commit actuals (or cancel with zero actuals if the request failed)
limiter.reconcile(500, 431, 15 * MICRODOLLARS_PER_CENT, 129_300)?;

let stats = limiter.stats()?;
println!("month so far: ${:.4}", stats.monthly_cost_dollars());
# Ok::<(), api_rate_limiter::RateLimitError>(())
```

### Async with deadline

```rust,ignore
use api_rate_limiter::{AsyncRateLimiter, RateLimitConfig};
use std::time::Duration;

let limiter = AsyncRateLimiter::with_max_wait(
    RateLimitConfig::conservative(),
    Duration::from_secs(30), // total elapsed bound, across all retries
)?;

limiter.acquire_wait(1, 500, 1_500_000).await?;
```

`acquire_wait` retries only retriable errors (per-minute rate exhaustion).
Window limits, `ExceedsCapacity`, and configuration errors fail immediately.

## Error Handling

```rust,ignore
match limiter.acquire(1, tokens, cost) {
    Ok(()) => { /* proceed */ }
    Err(e) if e.is_retriable() => {
        // Per-minute capacity refills; e.wait_time() says how long
    }
    Err(RateLimitError::ExceedsCapacity { .. }) => {
        // Larger than the bucket can EVER hold — do not retry
    }
    Err(e) => {
        // Daily/monthly window exhausted, invalid config, or internal error
    }
}
```

## Known limitations

Deliberate simplifications, documented rather than hidden:

- **No persistence** — counters reset on restart (see Scope above)
- **Aggregate reconciliation** — no reservation IDs; callers must pair each
  `acquire` with exactly one `reconcile`. Duplicate or missing reconciles
  skew the counters.
- **UTC calendar windows** — daily/monthly boundaries are UTC, which may not
  match a provider's billing period
- **Wall-clock based** — no injectable clock yet, so window-boundary
  behavior is not unit-tested

## Testing

```bash
cargo test -p api-rate-limiter --all-features
```

## License

MIT OR Apache-2.0
