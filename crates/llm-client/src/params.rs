//! Normalized generation parameters mapped to each provider's wire format

/// Whether the model should use internal reasoning ("thinking") before answering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThinkingMode {
    /// No thinking configuration is sent. Models where thinking is always on
    /// (e.g. Claude Fable 5) still think; others answer directly.
    #[default]
    Off,
    /// The model decides when and how much to think.
    ///
    /// - Anthropic: `thinking: {"type": "adaptive"}`
    /// - OpenAI-compatible: no dedicated field; reasoning models think by
    ///   default, so this maps to sending nothing extra
    Adaptive,
}

/// Reasoning effort — how much compute the model spends thinking and acting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Low,
    Medium,
    High,
    /// Maximum effort. OpenAI-compatible endpoints have no `max` level, so
    /// this maps to `high` there.
    Max,
}

impl Effort {
    /// Parse from a CLI-style string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "low" => Some(Effort::Low),
            "medium" => Some(Effort::Medium),
            "high" => Some(Effort::High),
            "max" => Some(Effort::Max),
            _ => None,
        }
    }

    /// Value for Anthropic's `output_config.effort`.
    pub fn as_anthropic(&self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::Max => "max",
        }
    }

    /// Value for OpenAI's `reasoning_effort` (no `max` level — clamps to `high`).
    pub fn as_openai(&self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High | Effort::Max => "high",
        }
    }
}

/// Provider-agnostic generation parameters.
///
/// Each provider mapping only sends the fields that endpoint understands;
/// unsupported fields are silently omitted rather than causing API errors.
#[derive(Debug, Clone)]
pub struct GenerationParams {
    /// Maximum output tokens (enforced per-response ceiling).
    pub max_tokens: u32,
    /// Sampling temperature. Omitted for models that reject sampling
    /// parameters (Claude Opus 4.7+, Fable/Mythos).
    pub temperature: Option<f32>,
    /// Thinking configuration.
    pub thinking: ThinkingMode,
    /// Reasoning effort level.
    pub effort: Option<Effort>,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            temperature: None,
            thinking: ThinkingMode::Off,
            effort: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effort_parse() {
        assert_eq!(Effort::parse("high"), Some(Effort::High));
        assert_eq!(Effort::parse("MAX"), Some(Effort::Max));
        assert_eq!(Effort::parse("bogus"), None);
    }

    #[test]
    fn test_effort_openai_clamps_max() {
        assert_eq!(Effort::Max.as_openai(), "high");
        assert_eq!(Effort::Max.as_anthropic(), "max");
    }
}
