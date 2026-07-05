//! Configurable per-model pricing for cost tracking

/// Pricing for one model, in cents per million tokens.
///
/// Example: Claude Sonnet 4.6 is $3/MTok input and $15/MTok output, so
/// `ModelPricing { input_cents_per_mtok: 300, output_cents_per_mtok: 1500 }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPricing {
    pub input_cents_per_mtok: u64,
    pub output_cents_per_mtok: u64,
}

impl ModelPricing {
    /// Zero-cost pricing for locally hosted open models.
    pub const FREE: Self = Self {
        input_cents_per_mtok: 0,
        output_cents_per_mtok: 0,
    };

    /// Cost in microdollars (1 dollar = 1,000,000 microdollars) for the
    /// given token counts, rounded up so budgets are never undercounted.
    ///
    /// Microdollar precision keeps cheap calls honest: a 10-token Haiku
    /// request costs microdollars, not a whole rounded-up cent. Intermediate
    /// math is u128, so token counts cannot overflow.
    pub fn cost_microdollars(&self, input_tokens: u64, output_tokens: u64) -> u64 {
        // Each u64×u64 product fits in u128, but their SUM can exceed
        // u128::MAX at extreme values — saturate rather than overflow.
        let raw_cents_tokens = (u128::from(input_tokens) * u128::from(self.input_cents_per_mtok))
            .saturating_add(u128::from(output_tokens) * u128::from(self.output_cents_per_mtok));
        // (cents per MTok · tokens) → microdollars:
        // divide by 1e6 (per-MTok) and multiply by 10,000 (µ$ per cent) = ÷100
        u64::try_from(raw_cents_tokens.div_ceil(100)).unwrap_or(u64::MAX)
    }
}

/// Maps model names to pricing via longest-prefix match, with a fallback for
/// unknown models.
#[derive(Debug, Clone)]
pub struct PricingRegistry {
    /// (model name prefix, pricing) pairs
    entries: Vec<(String, ModelPricing)>,
    fallback: ModelPricing,
}

impl PricingRegistry {
    /// Empty registry with the given fallback for every model.
    pub fn empty(fallback: ModelPricing) -> Self {
        Self {
            entries: Vec::new(),
            fallback,
        }
    }

    /// Registry where every model costs nothing — for local open-model
    /// servers (Ollama, vLLM, LM Studio).
    pub fn free() -> Self {
        Self::empty(ModelPricing::FREE)
    }

    /// Registry seeded with current Claude pricing (as of June 2026).
    ///
    /// | Prefix | Input $/MTok | Output $/MTok |
    /// |---|---|---|
    /// | `claude-fable-5`, `claude-mythos-5` | $10 | $50 |
    /// | `claude-opus-4` (4.5–4.8) | $5 | $25 |
    /// | `claude-sonnet-4` (4.5, 4.6) | $3 | $15 |
    /// | `claude-haiku-4` | $1 | $5 |
    ///
    /// Unknown models fall back to Opus-tier pricing (conservative — budgets
    /// are consumed faster, never slower, than reality). Register non-Claude
    /// models explicitly with [`PricingRegistry::register`].
    pub fn with_claude_defaults() -> Self {
        let mut registry = Self::empty(ModelPricing {
            input_cents_per_mtok: 500,
            output_cents_per_mtok: 2500,
        });
        registry.register(
            "claude-fable-5",
            ModelPricing {
                input_cents_per_mtok: 1000,
                output_cents_per_mtok: 5000,
            },
        );
        registry.register(
            "claude-mythos-5",
            ModelPricing {
                input_cents_per_mtok: 1000,
                output_cents_per_mtok: 5000,
            },
        );
        registry.register(
            "claude-opus-4",
            ModelPricing {
                input_cents_per_mtok: 500,
                output_cents_per_mtok: 2500,
            },
        );
        registry.register(
            "claude-sonnet-4",
            ModelPricing {
                input_cents_per_mtok: 300,
                output_cents_per_mtok: 1500,
            },
        );
        registry.register(
            "claude-haiku-4",
            ModelPricing {
                input_cents_per_mtok: 100,
                output_cents_per_mtok: 500,
            },
        );
        registry
    }

    /// Register pricing for a model name prefix. Longer prefixes win over
    /// shorter ones, so specific overrides can coexist with family defaults.
    pub fn register(&mut self, model_prefix: impl Into<String>, pricing: ModelPricing) {
        self.entries.push((model_prefix.into(), pricing));
    }

    /// Look up pricing for a model by longest matching prefix.
    pub fn lookup(&self, model: &str) -> ModelPricing {
        self.entries
            .iter()
            .filter(|(prefix, _)| model.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len())
            .map(|(_, pricing)| *pricing)
            .unwrap_or(self.fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_microdollar_precision() {
        let pricing = ModelPricing {
            input_cents_per_mtok: 300,
            output_cents_per_mtok: 1500,
        };
        // 1000 in + 500 out = (300_000 + 750_000) / 100 = 10_500 µ$ = $0.0105
        // (exact — not rounded up to a whole 2 cents)
        assert_eq!(pricing.cost_microdollars(1000, 500), 10_500);
        // Tiny request: 180 µ$ = $0.00018, not a whole cent
        assert_eq!(pricing.cost_microdollars(10, 10), 180);
        assert_eq!(pricing.cost_microdollars(0, 0), 0);
        // Sub-unit costs round UP, never to zero
        assert_eq!(pricing.cost_microdollars(0, 1), 15);
    }

    #[test]
    fn test_cost_does_not_overflow() {
        let pricing = ModelPricing {
            input_cents_per_mtok: u64::MAX,
            output_cents_per_mtok: u64::MAX,
        };
        // u64::MAX tokens at u64::MAX pricing saturates instead of wrapping
        assert_eq!(pricing.cost_microdollars(u64::MAX, u64::MAX), u64::MAX);
    }

    #[test]
    fn test_lookup_longest_prefix_wins() {
        let mut registry = PricingRegistry::with_claude_defaults();
        registry.register(
            "claude-sonnet-4-5",
            ModelPricing {
                input_cents_per_mtok: 999,
                output_cents_per_mtok: 999,
            },
        );

        // Specific override wins over the family prefix
        assert_eq!(
            registry
                .lookup("claude-sonnet-4-5-20250929")
                .input_cents_per_mtok,
            999
        );
        // Family prefix still applies to other members
        assert_eq!(
            registry.lookup("claude-sonnet-4-6").input_cents_per_mtok,
            300
        );
        assert_eq!(
            registry.lookup("claude-haiku-4-5").output_cents_per_mtok,
            500
        );
    }

    #[test]
    fn test_lookup_unknown_uses_fallback() {
        let registry = PricingRegistry::with_claude_defaults();
        let unknown = registry.lookup("gpt-5");
        assert_eq!(unknown.input_cents_per_mtok, 500);

        let free = PricingRegistry::free();
        assert_eq!(free.lookup("llama3.3:70b"), ModelPricing::FREE);
    }
}
