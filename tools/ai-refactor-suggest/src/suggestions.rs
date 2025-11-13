use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A refactoring suggestion from Claude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorSuggestion {
    pub decoder: String,
    pub category: String, // "dependency", "security", "performance", "testing", "architecture"
    pub priority: String, // "high", "medium", "low"
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_change: Option<String>,
}

impl RefactorSuggestion {
    /// Create a new suggestion
    #[allow(dead_code)]
    pub fn new(
        decoder: String,
        category: String,
        priority: String,
        title: String,
        description: String,
    ) -> Self {
        Self {
            decoder,
            category,
            priority,
            title,
            description,
            code_location: None,
            suggested_change: None,
        }
    }

    /// Set optional fields
    #[allow(dead_code)]
    pub fn with_location(mut self, location: String) -> Self {
        self.code_location = Some(location);
        self
    }

    #[allow(dead_code)]
    pub fn with_change(mut self, change: String) -> Self {
        self.suggested_change = Some(change);
        self
    }
}

/// Report generator for refactoring suggestions
pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a markdown report from suggestions
    pub fn generate_markdown_report(
        &self,
        suggestions: &[RefactorSuggestion],
        output_path: &Path,
    ) -> Result<()> {
        // Group by decoder
        let by_decoder = self.group_by_decoder(suggestions);

        // Count by priority
        let priority_counts = self.count_by_priority(suggestions);

        // Generate report content
        let mut report = format!(
            r#"# AI Refactoring Suggestions

**Generated**: {}
**Total Suggestions**: {}
**Decoders Analyzed**: {}

## Summary by Priority

"#,
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            suggestions.len(),
            by_decoder.len()
        );

        report.push_str(&format!(
            "- **High Priority**: {}\n",
            priority_counts.get("high").unwrap_or(&0)
        ));
        report.push_str(&format!(
            "- **Medium Priority**: {}\n",
            priority_counts.get("medium").unwrap_or(&0)
        ));
        report.push_str(&format!(
            "- **Low Priority**: {}\n\n",
            priority_counts.get("low").unwrap_or(&0)
        ));

        // Summary by category
        let category_counts = self.count_by_category(suggestions);
        report.push_str("## Summary by Category\n\n");
        for (category, count) in &category_counts {
            report.push_str(&format!(
                "- **{}**: {}\n",
                Self::capitalize(category),
                count
            ));
        }
        report.push('\n');

        // Detailed suggestions by decoder
        report.push_str("## Detailed Suggestions\n\n");

        for decoder_name in by_decoder.keys() {
            let decoder_suggestions = &by_decoder[decoder_name];
            report.push_str(&format!("### {}\n\n", decoder_name));

            // Group by priority within decoder
            for priority in &["high", "medium", "low"] {
                let priority_suggestions: Vec<_> = decoder_suggestions
                    .iter()
                    .filter(|s| &s.priority == priority)
                    .collect();

                if priority_suggestions.is_empty() {
                    continue;
                }

                report.push_str(&format!("#### {} Priority\n\n", Self::capitalize(priority)));

                for suggestion in priority_suggestions {
                    report.push_str(&format!(
                        "**{}** ({})\n\n",
                        suggestion.title, suggestion.category
                    ));
                    report.push_str(&format!("{}\n\n", suggestion.description));

                    if let Some(ref location) = suggestion.code_location {
                        report.push_str(&format!("*Location*: `{}`\n\n", location));
                    }

                    if let Some(ref change) = suggestion.suggested_change {
                        report.push_str("*Suggested change*:\n```rust\n");
                        report.push_str(change);
                        report.push_str("\n```\n\n");
                    }

                    report.push_str("---\n\n");
                }
            }
        }

        // Write report
        fs::write(output_path, report)
            .with_context(|| format!("Failed to write report to {}", output_path.display()))?;

        println!("Report written to: {}", output_path.display());

        Ok(())
    }

    /// Generate GitHub issue templates for high-priority suggestions
    pub fn generate_github_issues(
        &self,
        suggestions: &[RefactorSuggestion],
        output_dir: &Path,
    ) -> Result<()> {
        // Create output directory
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create directory: {}", output_dir.display()))?;

        // Filter high-priority suggestions
        let high_priority: Vec<_> = suggestions
            .iter()
            .filter(|s| s.priority == "high")
            .collect();

        if high_priority.is_empty() {
            println!("No high-priority suggestions to generate issues for");
            return Ok(());
        }

        for (i, suggestion) in high_priority.iter().enumerate() {
            let filename = format!("issue-{}-{}.md", suggestion.decoder, i + 1);
            let issue_path = output_dir.join(filename);

            let mut issue_content = format!(
                r#"---
title: "[{}] {}"
labels: refactoring, {}, ai-suggested
---

## Description

{}

## Decoder

`decoder-{}`

## Category

{}

## Priority

{}

"#,
                suggestion.decoder,
                suggestion.title,
                suggestion.category,
                suggestion.description,
                suggestion.decoder,
                suggestion.category,
                suggestion.priority
            );

            if let Some(ref location) = suggestion.code_location {
                issue_content.push_str(&format!("## Location\n\n`{}`\n\n", location));
            }

            if let Some(ref change) = suggestion.suggested_change {
                issue_content.push_str(&format!(
                    "## Suggested Change\n\n```rust\n{}\n```\n\n",
                    change
                ));
            }

            issue_content.push_str(
                r#"## Additional Context

This issue was automatically generated by the AI refactoring suggestion system.
Please review and validate before implementing.

## Checklist

- [ ] Review suggestion validity
- [ ] Implement changes
- [ ] Add tests
- [ ] Update documentation
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test --all`
"#,
            );

            fs::write(&issue_path, issue_content)
                .with_context(|| format!("Failed to write issue to {}", issue_path.display()))?;
        }

        println!(
            "Generated {} GitHub issue templates in {}",
            high_priority.len(),
            output_dir.display()
        );

        Ok(())
    }

    fn group_by_decoder(
        &self,
        suggestions: &[RefactorSuggestion],
    ) -> HashMap<String, Vec<RefactorSuggestion>> {
        let mut by_decoder: HashMap<String, Vec<RefactorSuggestion>> = HashMap::new();

        for suggestion in suggestions {
            by_decoder
                .entry(suggestion.decoder.clone())
                .or_default()
                .push(suggestion.clone());
        }

        // Sort decoders alphabetically
        for suggestions in by_decoder.values_mut() {
            suggestions.sort_by(|a, b| {
                // Sort by priority first (high > medium > low)
                let priority_order = |p: &str| match p {
                    "high" => 0,
                    "medium" => 1,
                    "low" => 2,
                    _ => 3,
                };
                priority_order(&a.priority)
                    .cmp(&priority_order(&b.priority))
                    .then_with(|| a.title.cmp(&b.title))
            });
        }

        by_decoder
    }

    fn count_by_priority(&self, suggestions: &[RefactorSuggestion]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for suggestion in suggestions {
            *counts.entry(suggestion.priority.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn count_by_category(&self, suggestions: &[RefactorSuggestion]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for suggestion in suggestions {
            *counts.entry(suggestion.category.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}
