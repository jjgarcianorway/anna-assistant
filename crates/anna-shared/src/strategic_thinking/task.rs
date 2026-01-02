//! Strategic Thinking Task - Phase 91
//!
//! Individual strategic thinking task definition.

use serde::{Deserialize, Serialize};
use super::types::{ThinkingCategory, ThinkingPriority, ThinkingStatus};

/// A strategic thinking task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingTask {
    /// Unique ID
    pub id: String,
    /// Description of what to think about
    pub description: String,
    /// Category
    pub category: ThinkingCategory,
    /// Priority
    pub priority: ThinkingPriority,
    /// Status
    pub status: ThinkingStatus,
    /// Senior assigned
    pub assigned_to: Option<String>,
    /// Created timestamp
    pub created_at: u64,
    /// Started timestamp
    pub started_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
    /// Time spent thinking (seconds)
    pub time_spent_secs: u64,
    /// Findings/conclusions
    pub findings: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Was interrupted
    pub interrupted: bool,
    /// Resume point (for paused tasks)
    pub resume_point: Option<String>,
}
