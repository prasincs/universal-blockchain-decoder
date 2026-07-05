# llm-client

Internal LLM adapter with rate limiting, cost tracking, and normalized
reasoning parameters, covering the two most common wire formats. Built for
this repository's tooling; not published to crates.io (`publish = false`).

## Supported Providers

| Provider | Wire format | Covers |
|---|---|---|
| `Provider::Anthropic` | Anthropic Messages API | All Claude models |
| `Provider::OpenAiCompatible` | OpenAI chat completions | OpenAI, plus open-model servers: **Ollama, vLLM, LM Studio, Groq, Together**, ... |

## Features

- **Normalized parameters**: `max_tokens`, `temperature`, `thinking`
  (adaptive), and reasoning `effort` — mapped to each provider's wire format,
  with unsupported fields omitted instead of causing API errors
- **Model-aware safety**: temperature is automatically dropped for models
  that reject sampling parameters (Claude Opus 4.7+, Fable/Mythos); thinking
  and temperature are never sent together
- **Refusal handling**: Anthropic `stop_reason: "refusal"` surfaces as a
  typed `LlmError::Refused` instead of an index panic on empty content
- **Configurable pricing**: prefix-matched `PricingRegistry` seeded with
  current Claude prices; register any model, or use `PricingRegistry::free()`
  for local open models
- **Rate limiting with reservation semantics**: integrates
  `api-rate-limiter` — reserves the pre-flight estimate, commits the
  provider-reported actual usage on success, and cancels the reservation on
  transport/API/parse failure

## Non-goals (currently unsupported)

Streaming, tool calls, structured output, multimodal content, multi-turn
conversations, embeddings, cached-token accounting, provider rate-limit
header synchronization, retry/backoff policy, async HTTP, and injectable
transports. Projects needing those should look at full-featured crates such
as `genai` or `rig-core`.

## Usage

### Closed model (Anthropic) with thinking + effort

```rust
use llm_client::{Effort, GenerationParams, LlmClient, Provider, RateLimitConfig, RateLimiter, ThinkingMode};

let limiter = RateLimiter::new(RateLimitConfig::conservative())?;
let client = LlmClient::new(Provider::anthropic(std::env::var("ANTHROPIC_API_KEY")?))
    .with_limiter(limiter);

let params = GenerationParams {
    max_tokens: 8000,
    temperature: None,
    thinking: ThinkingMode::Adaptive,
    effort: Some(Effort::High),
};

let completion = client.complete("claude-sonnet-4-6", None, "Summarize this design...", &params)?;
println!("{}", completion.text);
if let Some(thinking) = completion.thinking {
    println!("(reasoned: {thinking})");
}
```

### Open model (local Ollama)

```rust
use llm_client::{GenerationParams, LlmClient, PricingRegistry, Provider};

let client = LlmClient::new(Provider::ollama())        // http://localhost:11434/v1
    .with_pricing(PricingRegistry::free());            // local inference is free

let params = GenerationParams { max_tokens: 4000, temperature: Some(0.3), ..Default::default() };
let completion = client.complete("llama3.3:70b", Some("Be terse."), "Hello!", &params)?;
```

### Any OpenAI-compatible endpoint

```rust
use llm_client::{LlmClient, ModelPricing, PricingRegistry, Provider};

// OpenAI itself
let openai = LlmClient::new(Provider::openai(std::env::var("OPENAI_API_KEY")?));

// vLLM / Groq / Together / a corporate gateway — register real pricing so
// budgets are tracked correctly
let mut pricing = PricingRegistry::free();
pricing.register("llama-3.3-70b", ModelPricing {
    input_cents_per_mtok: 59,   // $0.59/MTok
    output_cents_per_mtok: 79,  // $0.79/MTok
});  // costs are computed in microdollars (1 cent = 10,000 microdollars)
let groq = LlmClient::new(Provider::openai_compatible(
    "https://api.groq.com/openai/v1",
    Some(std::env::var("GROQ_API_KEY")?),
)).with_pricing(pricing);
```

## Parameter Mapping

| `GenerationParams` field | Anthropic | OpenAI-compatible |
|---|---|---|
| `max_tokens` | `max_tokens` | `max_tokens` |
| `temperature` | `temperature` (omitted with thinking, or on Opus 4.7+/Fable which reject it) | `temperature` |
| `thinking: Adaptive` | `thinking: {"type": "adaptive"}` | *(nothing — reasoning models think by default)* |
| `effort` | `output_config.effort` (`low`/`medium`/`high`/`max`) | `reasoning_effort` (`max` clamps to `high`) |

## Cost Tracking

Pricing rates are cents per million tokens, looked up by longest model-name
prefix; per-call costs are computed in **microdollars** (1 cent = 10,000
microdollars, ceiling-rounded), so cheap or local-model calls are not
overstated by whole-cent rounding. `PricingRegistry::with_claude_defaults()`
(the default) ships current Claude prices and falls back to Opus-tier pricing
for unknown models — conservative, so budgets deplete faster than reality,
never slower.

When a `RateLimiter` is attached, each call follows a
reserve → commit / cancel lifecycle:

1. **Reserve**: estimate usage (`prompt bytes / 4` input + full `max_tokens`
   output) and acquire permits — failing fast if a rate, daily-token, or
   monthly-budget limit would be exceeded
2. Send the request
3. **Commit** the exact `usage` the provider returned, or **cancel** the
   reservation if the request failed (per-minute rate permits stay consumed,
   since the attempt happened)

Caveat: reconciliation is aggregate (no reservation IDs), and a mid-stream
Anthropic refusal that bills partial output is cancelled rather than
partially committed.

## Scope Note

This crate is workflow tooling, **not** part of the trusted decoding core —
it makes network calls by design and is excluded from the < 3000 LOC TCB.

## Testing

```bash
cargo test -p llm-client
```
