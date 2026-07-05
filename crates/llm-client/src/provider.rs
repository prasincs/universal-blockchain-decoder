//! LLM provider endpoints and authentication

/// An LLM provider endpoint.
///
/// Two wire formats cover essentially every model in practice:
/// - [`Provider::Anthropic`] — Claude models via the Messages API
/// - [`Provider::OpenAiCompatible`] — OpenAI itself plus every open-model
///   server that implements the chat-completions format (Ollama, vLLM,
///   LM Studio, Groq, Together, ...)
pub enum Provider {
    /// Anthropic Messages API (`POST {base_url}/v1/messages`).
    Anthropic { api_key: String, base_url: String },
    /// OpenAI-compatible chat completions (`POST {base_url}/chat/completions`).
    /// `base_url` should include the version segment, e.g.
    /// `https://api.openai.com/v1` or `http://localhost:11434/v1`.
    OpenAiCompatible {
        api_key: Option<String>,
        base_url: String,
    },
}

impl Provider {
    /// Anthropic direct API.
    pub fn anthropic(api_key: impl Into<String>) -> Self {
        Provider::Anthropic {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    /// OpenAI.
    pub fn openai(api_key: impl Into<String>) -> Self {
        Provider::OpenAiCompatible {
            api_key: Some(api_key.into()),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Local Ollama server (no authentication).
    pub fn ollama() -> Self {
        Provider::OpenAiCompatible {
            api_key: None,
            base_url: "http://localhost:11434/v1".to_string(),
        }
    }

    /// Any OpenAI-compatible endpoint (vLLM, LM Studio, Groq, Together, ...).
    pub fn openai_compatible(base_url: impl Into<String>, api_key: Option<String>) -> Self {
        Provider::OpenAiCompatible {
            api_key,
            base_url: base_url.into(),
        }
    }
}

// Manual Debug to keep API keys out of logs.
impl std::fmt::Debug for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Anthropic { base_url, .. } => f
                .debug_struct("Anthropic")
                .field("base_url", base_url)
                .field("api_key", &"<redacted>")
                .finish(),
            Provider::OpenAiCompatible { base_url, api_key } => f
                .debug_struct("OpenAiCompatible")
                .field("base_url", base_url)
                .field("api_key", &api_key.as_ref().map(|_| "<redacted>"))
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_redacts_api_key() {
        let provider = Provider::anthropic("sk-ant-secret-key");
        let debug = format!("{:?}", provider);
        assert!(!debug.contains("secret"));
        assert!(debug.contains("<redacted>"));
    }
}
