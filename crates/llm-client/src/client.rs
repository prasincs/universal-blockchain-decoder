//! Provider-agnostic completion client

use api_rate_limiter::{RateLimiter, UsageStats};
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::LlmError;
use crate::params::{GenerationParams, ThinkingMode};
use crate::pricing::PricingRegistry;
use crate::provider::Provider;

const ANTHROPIC_VERSION: &str = "2023-06-01";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Token usage reported by the provider.
#[derive(Debug, Clone, Copy, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl Usage {
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// A completed generation, normalized across providers.
#[derive(Debug, Clone)]
pub struct Completion {
    /// The model's answer text.
    pub text: String,
    /// Reasoning text, when the provider returns it (Anthropic summarized
    /// thinking blocks, or `reasoning_content` from open-model servers).
    pub thinking: Option<String>,
    /// Actual token usage from the provider (not an estimate).
    pub usage: Usage,
    /// Provider stop/finish reason, if reported.
    pub stop_reason: Option<String>,
}

/// A blocking LLM client bound to one provider, with optional rate limiting
/// and cost tracking.
pub struct LlmClient {
    provider: Provider,
    http: reqwest::blocking::Client,
    limiter: Option<RateLimiter>,
    pricing: PricingRegistry,
}

impl LlmClient {
    /// Create a client for the given provider with Claude default pricing
    /// and no rate limiter.
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            http: reqwest::blocking::Client::new(),
            limiter: None,
            pricing: PricingRegistry::with_claude_defaults(),
        }
    }

    /// Attach a rate limiter. Every call acquires permits before the request
    /// and reconciles the estimate with actual usage afterwards.
    pub fn with_limiter(mut self, limiter: RateLimiter) -> Self {
        self.limiter = Some(limiter);
        self
    }

    /// Replace the pricing registry (e.g. [`PricingRegistry::free`] for local
    /// open models, or one with custom entries registered).
    pub fn with_pricing(mut self, pricing: PricingRegistry) -> Self {
        self.pricing = pricing;
        self
    }

    /// Current usage statistics, if a rate limiter is attached.
    pub fn usage_stats(&self) -> Option<UsageStats> {
        self.limiter.as_ref().and_then(|l| l.stats().ok())
    }

    /// Run a single-turn completion.
    ///
    /// # Parameters
    /// - `model`: provider-specific model name (`claude-sonnet-4-6`,
    ///   `gpt-5`, `llama3.3:70b`, ...)
    /// - `system`: optional system prompt
    /// - `prompt`: the user message
    /// - `params`: normalized generation parameters
    pub fn complete(
        &self,
        model: &str,
        system: Option<&str>,
        prompt: &str,
        params: &GenerationParams,
    ) -> Result<Completion, LlmError> {
        let pricing = self.pricing.lookup(model);

        // Pre-flight estimate: ~4 chars per token for input, full max_tokens
        // for output (worst case; reconciled against actuals below).
        let estimated_input = (prompt.len() as u64 + system.map_or(0, |s| s.len() as u64)) / 4;
        let estimated_output = u64::from(params.max_tokens);
        let estimated_tokens = estimated_input + estimated_output;
        let estimated_cost = pricing.cost_microdollars(estimated_input, estimated_output);

        // Reserve the estimate.
        if let Some(limiter) = &self.limiter {
            limiter.acquire(1, estimated_tokens, estimated_cost)?;
        }

        match self.dispatch(model, system, prompt, params) {
            Ok(completion) => {
                // Commit: replace the estimate with what the provider
                // actually billed. Reconcile can only fail on a poisoned
                // lock, in which case the budget state is already lost —
                // the completed request is still returned.
                if let Some(limiter) = &self.limiter {
                    let actual_cost = pricing.cost_microdollars(
                        completion.usage.input_tokens,
                        completion.usage.output_tokens,
                    );
                    let _ = limiter.reconcile(
                        estimated_tokens,
                        completion.usage.total(),
                        estimated_cost,
                        actual_cost,
                    );
                }
                Ok(completion)
            }
            Err(e) => {
                // Cancel: transport, API, refusal, and parse failures must
                // not leave the estimated usage charged against the budget.
                // (The per-minute rate permits stay consumed — the attempt
                // happened. Mid-stream Anthropic refusals may bill partial
                // output that this cancel does not capture.)
                if let Some(limiter) = &self.limiter {
                    let _ = limiter.reconcile(estimated_tokens, 0, estimated_cost, 0);
                }
                Err(e)
            }
        }
    }

    /// Send the request to the configured provider and parse the response.
    fn dispatch(
        &self,
        model: &str,
        system: Option<&str>,
        prompt: &str,
        params: &GenerationParams,
    ) -> Result<Completion, LlmError> {
        match &self.provider {
            Provider::Anthropic { api_key, base_url } => {
                let body = build_anthropic_body(model, system, prompt, params);
                let response = self
                    .http
                    .post(format!("{base_url}/v1/messages"))
                    .timeout(REQUEST_TIMEOUT)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", ANTHROPIC_VERSION)
                    .header("content-type", "application/json")
                    .json(&body)
                    .send()?;
                let value = check_and_parse(response)?;
                parse_anthropic_response(&value)
            }
            Provider::OpenAiCompatible { api_key, base_url } => {
                let body = build_openai_body(model, system, prompt, params);
                let mut request = self
                    .http
                    .post(format!("{base_url}/chat/completions"))
                    .timeout(REQUEST_TIMEOUT)
                    .header("content-type", "application/json")
                    .json(&body);
                if let Some(key) = api_key {
                    request = request.header("authorization", format!("Bearer {key}"));
                }
                let value = check_and_parse(request.send()?)?;
                parse_openai_response(&value)
            }
        }
    }
}

/// Convert an f32 temperature to a clean JSON number (avoids f32→f64
/// artifacts like 0.7 serializing as 0.699999988079071).
fn temperature_json(temperature: f32) -> Value {
    ((f64::from(temperature) * 1000.0).round() / 1000.0).into()
}

/// True for models that reject sampling parameters (`temperature`, `top_p`,
/// `top_k` return 400 on Claude Opus 4.7+ and Fable/Mythos).
fn rejects_sampling_params(model: &str) -> bool {
    model.contains("opus-4-7")
        || model.contains("opus-4-8")
        || model.contains("fable")
        || model.contains("mythos")
}

fn build_anthropic_body(
    model: &str,
    system: Option<&str>,
    prompt: &str,
    params: &GenerationParams,
) -> Value {
    let mut body = json!({
        "model": model,
        "max_tokens": params.max_tokens,
        "messages": [{"role": "user", "content": prompt}],
    });
    if let Some(system) = system {
        body["system"] = system.into();
    }
    if params.thinking == ThinkingMode::Adaptive {
        body["thinking"] = json!({"type": "adaptive"});
    }
    if let Some(effort) = params.effort {
        body["output_config"] = json!({"effort": effort.as_anthropic()});
    }
    if let Some(temperature) = params.temperature {
        // Thinking and temperature are mutually exclusive; newer models
        // reject sampling parameters entirely.
        if params.thinking == ThinkingMode::Off && !rejects_sampling_params(model) {
            body["temperature"] = temperature_json(temperature);
        }
    }
    body
}

fn build_openai_body(
    model: &str,
    system: Option<&str>,
    prompt: &str,
    params: &GenerationParams,
) -> Value {
    let mut messages = Vec::new();
    if let Some(system) = system {
        messages.push(json!({"role": "system", "content": system}));
    }
    messages.push(json!({"role": "user", "content": prompt}));

    let mut body = json!({
        "model": model,
        "max_tokens": params.max_tokens,
        "messages": messages,
    });
    if let Some(temperature) = params.temperature {
        body["temperature"] = temperature_json(temperature);
    }
    if let Some(effort) = params.effort {
        body["reasoning_effort"] = effort.as_openai().into();
    }
    body
}

fn check_and_parse(response: reqwest::blocking::Response) -> Result<Value, LlmError> {
    let status = response.status();
    if !status.is_success() {
        let message = response
            .text()
            .unwrap_or_else(|_| "unknown error".to_string());
        return Err(LlmError::Api {
            status: status.as_u16(),
            message,
        });
    }
    response
        .json::<Value>()
        .map_err(|e| LlmError::Parse(e.to_string()))
}

fn parse_anthropic_response(value: &Value) -> Result<Completion, LlmError> {
    let stop_reason = value["stop_reason"].as_str().map(String::from);

    // Check stop_reason before reading content — refused requests can have
    // an empty content array.
    if stop_reason.as_deref() == Some("refusal") {
        let category = value["stop_details"]["category"].as_str().map(String::from);
        return Err(LlmError::Refused { category });
    }

    let blocks = value["content"]
        .as_array()
        .ok_or_else(|| LlmError::Parse("missing content array".to_string()))?;

    let mut text = String::new();
    let mut thinking = String::new();
    for block in blocks {
        match block["type"].as_str() {
            Some("text") => {
                if let Some(t) = block["text"].as_str() {
                    text.push_str(t);
                }
            }
            Some("thinking") => {
                if let Some(t) = block["thinking"].as_str() {
                    thinking.push_str(t);
                }
            }
            _ => {}
        }
    }

    Ok(Completion {
        text,
        thinking: (!thinking.is_empty()).then_some(thinking),
        usage: Usage {
            input_tokens: value["usage"]["input_tokens"].as_u64().unwrap_or(0),
            output_tokens: value["usage"]["output_tokens"].as_u64().unwrap_or(0),
        },
        stop_reason,
    })
}

fn parse_openai_response(value: &Value) -> Result<Completion, LlmError> {
    let choice = value["choices"]
        .get(0)
        .ok_or_else(|| LlmError::Parse("missing choices array".to_string()))?;
    let message = &choice["message"];

    Ok(Completion {
        text: message["content"].as_str().unwrap_or_default().to_string(),
        // DeepSeek-style reasoning models return reasoning_content
        thinking: message["reasoning_content"].as_str().map(String::from),
        usage: Usage {
            input_tokens: value["usage"]["prompt_tokens"].as_u64().unwrap_or(0),
            output_tokens: value["usage"]["completion_tokens"].as_u64().unwrap_or(0),
        },
        stop_reason: choice["finish_reason"].as_str().map(String::from),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Effort;

    fn params_with(
        thinking: ThinkingMode,
        effort: Option<Effort>,
        temp: Option<f32>,
    ) -> GenerationParams {
        GenerationParams {
            max_tokens: 1000,
            temperature: temp,
            thinking,
            effort,
        }
    }

    #[test]
    fn test_anthropic_body_thinking_and_effort() {
        let params = params_with(ThinkingMode::Adaptive, Some(Effort::High), None);
        let body = build_anthropic_body("claude-sonnet-4-6", Some("Be terse."), "Hi", &params);

        assert_eq!(body["thinking"]["type"], "adaptive");
        assert_eq!(body["output_config"]["effort"], "high");
        assert_eq!(body["system"], "Be terse.");
        assert_eq!(body["messages"][0]["role"], "user");
        assert!(body.get("temperature").is_none());
    }

    #[test]
    fn test_anthropic_body_omits_temperature_on_newer_models() {
        let params = params_with(ThinkingMode::Off, None, Some(0.3));

        // Sonnet 4.6 accepts temperature
        let sonnet = build_anthropic_body("claude-sonnet-4-6", None, "Hi", &params);
        assert_eq!(sonnet["temperature"], 0.3);

        // Opus 4.8 and Fable reject sampling params — must be omitted
        for model in ["claude-opus-4-8", "claude-fable-5"] {
            let body = build_anthropic_body(model, None, "Hi", &params);
            assert!(
                body.get("temperature").is_none(),
                "temperature sent to {model}"
            );
        }
    }

    #[test]
    fn test_anthropic_body_omits_temperature_with_thinking() {
        let params = params_with(ThinkingMode::Adaptive, None, Some(0.3));
        let body = build_anthropic_body("claude-sonnet-4-6", None, "Hi", &params);
        assert!(body.get("temperature").is_none());
    }

    #[test]
    fn test_openai_body_reasoning_effort() {
        let params = params_with(ThinkingMode::Adaptive, Some(Effort::Max), Some(0.7));
        let body = build_openai_body("gpt-5", Some("Be terse."), "Hi", &params);

        // max clamps to high on OpenAI-compatible endpoints
        assert_eq!(body["reasoning_effort"], "high");
        assert_eq!(body["temperature"], 0.7);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["role"], "user");
        // No Anthropic-specific fields leak through
        assert!(body.get("thinking").is_none());
        assert!(body.get("output_config").is_none());
    }

    #[test]
    fn test_parse_anthropic_response() {
        let value = json!({
            "content": [
                {"type": "thinking", "thinking": "reasoning here"},
                {"type": "text", "text": "answer"}
            ],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 100, "output_tokens": 50}
        });
        let completion = parse_anthropic_response(&value).unwrap();
        assert_eq!(completion.text, "answer");
        assert_eq!(completion.thinking.as_deref(), Some("reasoning here"));
        assert_eq!(completion.usage.input_tokens, 100);
        assert_eq!(completion.usage.output_tokens, 50);
    }

    #[test]
    fn test_parse_anthropic_refusal() {
        let value = json!({
            "content": [],
            "stop_reason": "refusal",
            "stop_details": {"category": "cyber"},
            "usage": {"input_tokens": 0, "output_tokens": 0}
        });
        match parse_anthropic_response(&value) {
            Err(LlmError::Refused { category }) => {
                assert_eq!(category.as_deref(), Some("cyber"));
            }
            other => panic!("expected Refused, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_openai_response() {
        let value = json!({
            "choices": [{
                "message": {"content": "answer", "reasoning_content": "thoughts"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 80, "completion_tokens": 40}
        });
        let completion = parse_openai_response(&value).unwrap();
        assert_eq!(completion.text, "answer");
        assert_eq!(completion.thinking.as_deref(), Some("thoughts"));
        assert_eq!(completion.usage.total(), 120);
        assert_eq!(completion.stop_reason.as_deref(), Some("stop"));
    }
}
