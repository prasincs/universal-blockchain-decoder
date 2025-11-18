//! ROADMAP.md parser

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::task::Task;

lazy_static! {
    // Pattern: ### 3.X: Task Title STATUS
    static ref PHASE_PATTERN: Regex = Regex::new(
        r"###\s+(\d+\.\d+(?:[a-z])?):?\s+(.+?)\s+(✅|🚧|⚠️|📋)"
    )
    .unwrap();

    static ref PRIORITY_PATTERN: Regex =
        Regex::new(r"\*\*Priority\*\*:\s*(CRITICAL|HIGH|MEDIUM|LOW)").unwrap();

    static ref TIME_PATTERN: Regex = Regex::new(r"\*\*Time[^:]*\*\*:\s*([^\n]+)").unwrap();

    static ref COMPLETED_PATTERN: Regex = Regex::new(r"-\s+✅\s+(.+)").unwrap();

    static ref REMAINING_PATTERN: Regex = Regex::new(r"-\s+\[\s*\]\s+(.+)").unwrap();

    static ref PENDING_PATTERN: Regex = Regex::new(r"-\s+⏳\s+(.+)").unwrap();
}

pub struct RoadmapParser {
    content: String,
}

impl RoadmapParser {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read roadmap file")?;
        Ok(Self { content })
    }

    pub fn parse(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        // Split content into sections (between ### headers)
        let sections: Vec<_> = self.content.split("\n###").collect();

        for section in sections {
            let full_section = if section.starts_with("###") {
                section.to_string()
            } else {
                format!("###{}", section)
            };

            if let Some(task) = self.parse_section(&full_section) {
                tasks.push(task);
            }
        }

        Ok(tasks)
    }

    fn parse_section(&self, section: &str) -> Option<Task> {
        // Extract phase header
        let caps = PHASE_PATTERN.captures(section)?;
        let phase_id = caps.get(1)?.as_str();
        let title = caps.get(2)?.as_str().trim();
        let status_emoji = caps.get(3)?.as_str();

        // Map emoji to status text
        let status = match status_emoji {
            "✅" => "✅ COMPLETE",
            "🚧" => "🚧 IN PROGRESS",
            "⚠️" => "⚠️ NEEDS ATTENTION",
            "📋" => "📋 Planned",
            _ => "📋 Planned",
        };

        // Extract priority
        let priority = PRIORITY_PATTERN
            .captures(section)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "MEDIUM".to_string());

        // Extract time estimate
        let time_estimate = TIME_PATTERN
            .captures(section)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string());

        // Extract completed items
        let completed_items: Vec<String> = COMPLETED_PATTERN
            .captures_iter(section)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        // Extract remaining items (both [ ] and ⏳)
        let mut remaining_items: Vec<String> = REMAINING_PATTERN
            .captures_iter(section)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        let pending: Vec<String> = PENDING_PATTERN
            .captures_iter(section)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        remaining_items.extend(pending);

        // Extract description (first 500 chars after header)
        let description = section.chars().take(500).collect::<String>();

        Some(Task::new(
            format!("phase-{}", phase_id),
            format!("Phase {}", phase_id),
            title.to_string(),
            status.to_string(),
            time_estimate,
            priority,
            description,
            completed_items,
            remaining_items,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_section() {
        let section = r#"
### 3.2: Complete OP Stack 🚧 IN PROGRESS (90% Complete, ~4 hours remaining)

**Priority**: HIGH (35+ OP Stack chains)
**Time**: ~4 hours

**Completed**:
- ✅ Vendor superchain-registry (63 chains, 7KB Borsh)
- ✅ Deposit transaction (0x7E) parsing

**Remaining**:
- [ ] Fix EthereumTransaction trait implementations
- ⏳ Integration tests with real OP Stack deposit transactions
"#;

        let parser = RoadmapParser {
            content: String::new(),
        };
        let task = parser.parse_section(section).unwrap();

        assert_eq!(task.id, "phase-3.2");
        assert_eq!(task.title, "Complete OP Stack");
        assert_eq!(task.status, "🚧 IN PROGRESS");
        assert_eq!(task.priority, "HIGH");
        assert_eq!(task.time_estimate, Some("~4 hours".to_string()));
        assert_eq!(task.completed_items.len(), 2);
        assert_eq!(task.remaining_items.len(), 2);
    }
}
