//! ROI (Return on Investment) calculator

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use crate::task::Task;

lazy_static! {
    static ref NUMBER_PATTERN: Regex = Regex::new(r"\d+").unwrap();
}

pub struct RoiCalculator {
    priority_weights: HashMap<String, f64>,
    status_multipliers: HashMap<String, f64>,
}

impl RoiCalculator {
    pub fn new() -> Self {
        let mut priority_weights = HashMap::new();
        priority_weights.insert("CRITICAL".to_string(), 100.0);
        priority_weights.insert("HIGH".to_string(), 50.0);
        priority_weights.insert("MEDIUM".to_string(), 25.0);
        priority_weights.insert("LOW".to_string(), 10.0);

        let mut status_multipliers = HashMap::new();
        status_multipliers.insert("🚧 IN PROGRESS".to_string(), 1.5);
        status_multipliers.insert("⚠️ NEEDS ATTENTION".to_string(), 1.3);
        status_multipliers.insert("📋 Planned".to_string(), 1.0);
        status_multipliers.insert("✅ COMPLETE".to_string(), 0.0);

        Self {
            priority_weights,
            status_multipliers,
        }
    }

    /// Calculate ROI score for a task
    ///
    /// Formula: (Priority × Status × Completion Boost) / Time
    pub fn calculate(&self, task: &Task) -> f64 {
        let priority_weight = self
            .priority_weights
            .get(&task.priority)
            .copied()
            .unwrap_or(10.0);

        let status_multiplier = self
            .status_multipliers
            .get(&task.status)
            .copied()
            .unwrap_or(1.0);

        let time_hours = self.parse_time_estimate(task.time_estimate.as_deref());

        // Completion percentage boost
        let completion_pct = task.completion_percentage();
        let completion_boost = if completion_pct >= 0.8 {
            1.5 // Boost almost-complete tasks
        } else {
            1.0
        };

        (priority_weight * status_multiplier * completion_boost) / time_hours
    }

    /// Parse time estimate string to hours
    ///
    /// Examples:
    /// - "4-6 hours" → 5.0
    /// - "1-2 weeks" → 120.0
    /// - "3 days" → 24.0
    fn parse_time_estimate(&self, time_str: Option<&str>) -> f64 {
        let time_str = match time_str {
            Some(s) => s.to_lowercase(),
            None => return 80.0, // Default: 1 week
        };

        // Extract numbers
        let numbers: Vec<f64> = NUMBER_PATTERN
            .find_iter(&time_str)
            .filter_map(|m| m.as_str().parse::<f64>().ok())
            .collect();

        if numbers.is_empty() {
            return 80.0;
        }

        // Calculate average if range
        let avg = numbers.iter().sum::<f64>() / numbers.len() as f64;

        // Convert to hours based on unit
        if time_str.contains("week") {
            avg * 80.0 // 80 hours/week (2 work weeks per estimate week)
        } else if time_str.contains("day") {
            avg * 8.0 // 8 hours/day
        } else if time_str.contains("hour") || time_str.contains("hr") {
            avg
        } else if time_str.contains("month") {
            avg * 160.0 // 160 hours/month
        } else {
            avg // Assume hours
        }
    }
}

impl Default for RoiCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_estimate() {
        let calc = RoiCalculator::new();

        assert_eq!(calc.parse_time_estimate(Some("4-6 hours")), 5.0);
        assert_eq!(calc.parse_time_estimate(Some("1-2 weeks")), 120.0);
        assert_eq!(calc.parse_time_estimate(Some("3 days")), 24.0);
        assert_eq!(calc.parse_time_estimate(Some("2 months")), 320.0);
        assert_eq!(calc.parse_time_estimate(None), 80.0);
    }

    #[test]
    fn test_calculate_roi() {
        let calc = RoiCalculator::new();

        // High priority, in progress, 90% complete, 4 hours
        let mut task = Task::new(
            "test-1".to_string(),
            "Phase 1".to_string(),
            "Test Task".to_string(),
            "🚧 IN PROGRESS".to_string(),
            Some("4 hours".to_string()),
            "HIGH".to_string(),
            "Description".to_string(),
            vec![
                "item1".to_string(),
                "item2".to_string(),
                "item3".to_string(),
            ],
            vec!["item4".to_string()],
        );

        let roi = calc.calculate(&task);
        // (50 * 1.5 * 1.0) / 4 = 18.75
        // Note: Completion is 75% (3/4), not 80%+, so no boost
        assert_eq!(roi, 18.75);

        // Now make it 80%+ complete
        task.completed_items.push("item5".to_string());
        let roi = calc.calculate(&task);
        // (50 * 1.5 * 1.5) / 4 = 28.125
        assert_eq!(roi, 28.125);
    }
}
