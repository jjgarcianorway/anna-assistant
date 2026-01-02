//! Context-aware greeting insights from system state (v0.0.335).
//!
//! Enriches Anna's greetings with observations about the system state,
//! making her feel more aware and proactive without being annoying.
//!
//! v0.0.245: Initial implementation.
//! v0.0.326: Added learning progress insights.
//! v0.0.335: Enhanced with health status and trends.

mod format;
mod learning;
mod system;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::GreetingInsight;

// Re-export public functions
pub use format::{format_insights_for_greeting, quick_status_line};

use crate::snapshot::{DeltaItem, SystemSnapshot};

use learning::add_learning_insights;
use system::{add_delta_insights, add_disk_insights, add_memory_insights, add_service_insights};

/// Generate greeting insights from system snapshot
pub fn generate_insights(snapshot: &SystemSnapshot, deltas: &[DeltaItem]) -> Vec<GreetingInsight> {
    let mut insights = Vec::new();

    // Check for critical issues first
    add_disk_insights(snapshot, &mut insights);
    add_memory_insights(snapshot, &mut insights);
    add_service_insights(snapshot, &mut insights);
    add_delta_insights(deltas, &mut insights);

    // v0.0.326: Add learning progress (low priority - only shows if no issues)
    add_learning_insights(&mut insights);

    // Sort by priority (highest first)
    insights.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Limit to top 2 insights for greeting (don't overwhelm)
    insights.truncate(2);

    insights
}
