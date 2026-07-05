//! Internal LLM adapter with rate limiting and cost tracking
//!
//! # Overview
//!
//! This crate is an **internal adapter** for this repository's tooling — a
//! single blocking, single-turn completion API over the two most common LLM
//! wire formats:
//! - **Anthropic Messages API** (Claude models, direct API)
//! - **OpenAI-compatible chat completions** — OpenAI itself, plus open-model
//!   servers such as Ollama, vLLM, LM Studio, Groq, and Together
//!
//! It normalizes generation parameters (max tokens, temperature, thinking,
//! reasoning effort) and maps them to each provider's wire format, tracks
//! cost through a configurable pricing registry, and integrates with
//! [`api_rate_limiter`] — reserving a pre-flight estimate, committing the
//! provider-reported actual usage on success, and cancelling the
//! reservation on failure.
//!
//! # Non-goals (currently unsupported)
//!
//! This is not a general-purpose provider-neutral client. Out of scope for
//! now: streaming, tool calls, structured output, multimodal content,
//! multi-turn conversations, embeddings, cached-token accounting, provider
//! rate-limit header synchronization, retry/backoff policy, async HTTP, and
//! injectable transports. Projects needing those should look at
//! full-featured crates such as `genai` or `rig-core`.
//!
//! # Example
//!
//! ```rust,no_run
//! use llm_client::{Effort, GenerationParams, LlmClient, Provider, ThinkingMode};
//!
//! // Closed model: Anthropic with adaptive thinking + effort
//! let client = LlmClient::new(Provider::anthropic("sk-ant-..."));
//! let params = GenerationParams {
//!     max_tokens: 4096,
//!     temperature: None,
//!     thinking: ThinkingMode::Adaptive,
//!     effort: Some(Effort::High),
//! };
//! let completion = client.complete("claude-sonnet-4-6", None, "Hello!", &params)?;
//! println!("{}", completion.text);
//!
//! // Open model: local Ollama (free, so use a zero-cost pricing registry)
//! use llm_client::PricingRegistry;
//! let local = LlmClient::new(Provider::ollama()).with_pricing(PricingRegistry::free());
//! let completion = local.complete("llama3.3:70b", None, "Hello!", &params)?;
//! # Ok::<(), llm_client::LlmError>(())
//! ```

mod client;
mod error;
mod params;
mod pricing;
mod provider;

pub use api_rate_limiter::{RateLimitConfig, RateLimiter, UsageStats};
pub use client::{Completion, LlmClient, Usage};
pub use error::LlmError;
pub use params::{Effort, GenerationParams, ThinkingMode};
pub use pricing::{ModelPricing, PricingRegistry};
pub use provider::Provider;
