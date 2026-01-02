//! LLM Assignment Formatting
//!
//! Display and formatting functions for LLM assignments.

use super::tracker::LlmAssignmentTracker;

/// Format LLM tracker for display
pub fn format_llm_tracker(tracker: &LlmAssignmentTracker) -> String {
    let mut lines = vec!["=== LLM Assignments ===".to_string()];
    lines.push(String::new());

    // Available models
    if !tracker.available_models.is_empty() {
        lines.push(format!("Available models: {}", tracker.available_models.len()));
        for model in &tracker.available_models {
            lines.push(format!("  - {}", model));
        }
    }

    // Recommended tier
    if let Some(tier) = tracker.recommended_tier {
        lines.push(format!("Recommended tier: {}", tier.name()));
    }

    if tracker.assignments.is_empty() {
        lines.push(String::new());
        lines.push("No assignments yet.".to_string());
        return lines.join("\n");
    }

    // Active assignments
    let active = tracker.active_assignments();
    if !active.is_empty() {
        lines.push(String::new());
        lines.push("Active assignments:".to_string());
        for a in active {
            lines.push(format!(
                "  {} -> {} [{}]",
                a.specialist_id, a.model, a.tier.name()
            ));
        }
    }

    // Most used
    if let Some((model, count)) = tracker.most_used_model() {
        lines.push(String::new());
        lines.push(format!("Most used: {} ({} times)", model, count));
    }

    lines.join("\n")
}

/// Format LLM tracker compact
pub fn format_llm_tracker_compact(tracker: &LlmAssignmentTracker) -> String {
    let models = tracker.models_in_use();
    format!(
        "LLM: {} active | {} models | tier: {}",
        tracker.active_count(),
        models.len(),
        tracker.recommended_tier.map(|t| t.name()).unwrap_or("unknown")
    )
}

/// Format LLM tracker one-line
pub fn format_llm_tracker_oneline(tracker: &LlmAssignmentTracker) -> String {
    format!(
        "{} LLM assignments ({} active)",
        tracker.total_count(),
        tracker.active_count()
    )
}
