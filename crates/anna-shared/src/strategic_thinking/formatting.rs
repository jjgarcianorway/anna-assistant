//! Strategic Thinking Formatting - Phase 91
//!
//! Display and formatting functions for strategic thinking.

use super::tracker::StrategicThinkingTracker;

/// Format strategic thinking tracker for display
pub fn format_strategic_tracker(tracker: &StrategicThinkingTracker) -> String {
    let mut lines = vec!["=== Strategic Thinking ===".to_string()];
    lines.push(String::new());

    if tracker.tasks.is_empty() {
        lines.push("No strategic thinking tasks.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total tasks: {}", tracker.total_count()));
    lines.push(format!("Completed: {}", tracker.completed_count()));
    lines.push(format!("Time spent: {} min", tracker.total_time_secs / 60));
    lines.push(format!("Recommendations: {}", tracker.total_recommendations));

    // Paused (resumable)
    let paused = tracker.paused();
    if !paused.is_empty() {
        lines.push(String::new());
        lines.push("Resumable tasks:".to_string());
        for t in paused.iter().take(3) {
            lines.push(format!("  [{}] {}", t.category.name(), t.description));
        }
    }

    // Pending
    let pending = tracker.pending();
    if !pending.is_empty() {
        lines.push(String::new());
        lines.push(format!("Pending: {}", pending.len()));
    }

    lines.join("\n")
}

/// Format strategic tracker compact
pub fn format_strategic_tracker_compact(tracker: &StrategicThinkingTracker) -> String {
    format!(
        "Strategic: {} tasks | {} completed | {} recommendations",
        tracker.total_count(),
        tracker.completed_count(),
        tracker.total_recommendations
    )
}

/// Format strategic tracker one-line
pub fn format_strategic_tracker_oneline(tracker: &StrategicThinkingTracker) -> String {
    format!(
        "{} thinking tasks ({} done)",
        tracker.total_count(),
        tracker.completed_count()
    )
}

/// Generate fun fact about strategic thinking
pub fn strategic_fun_fact(tracker: &StrategicThinkingTracker) -> String {
    if tracker.tasks.is_empty() {
        return "No strategic thinking done yet!".to_string();
    }

    let facts = [
        format!(
            "Seniors have completed {} strategic tasks.",
            tracker.completed_count()
        ),
        format!(
            "{} minutes spent on strategic thinking.",
            tracker.total_time_secs / 60
        ),
        format!(
            "{} improvement recommendations made.",
            tracker.total_recommendations
        ),
        format!(
            "{} tasks are paused and can be resumed.",
            tracker.paused().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
