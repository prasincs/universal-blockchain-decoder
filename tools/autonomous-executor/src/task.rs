//! Task representation and types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub phase: String,
    pub title: String,
    pub status: String,
    pub time_estimate: Option<String>,
    pub priority: String,
    pub roi_score: f64,
    pub description: String,
    pub completed_items: Vec<String>,
    pub remaining_items: Vec<String>,
}

impl Task {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        phase: String,
        title: String,
        status: String,
        time_estimate: Option<String>,
        priority: String,
        description: String,
        completed_items: Vec<String>,
        remaining_items: Vec<String>,
    ) -> Self {
        Self {
            id,
            phase,
            title,
            status,
            time_estimate,
            priority,
            roi_score: 0.0,
            description,
            completed_items,
            remaining_items,
        }
    }

    /// Get completion percentage (0.0 to 1.0)
    pub fn completion_percentage(&self) -> f64 {
        let total = self.completed_items.len() + self.remaining_items.len();
        if total == 0 {
            0.0
        } else {
            self.completed_items.len() as f64 / total as f64
        }
    }
}
