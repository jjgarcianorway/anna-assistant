//! Error recovery tracker formatting functions
//!
//! Provides display formatters for error recovery data.

use super::tracker::ErrorRecoveryTracker;

/// Format error recovery tracker for display
pub fn format_error_recovery_tracker(tracker: &ErrorRecoveryTracker) -> String {
    let mut lines = vec!["=== Error Recovery Tracker ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No error recovery records yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total errors: {}", tracker.total_errors));
    lines.push(format!("Total recovered: {}", tracker.total_recovered));
    lines.push(format!("Recovery rate: {}%", tracker.recovery_rate()));

    // By category
    if !tracker.by_category.is_empty() {
        lines.push(String::new());
        lines.push("By category:".to_string());
        for (cat, count) in &tracker.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    // Best strategies
    let best = tracker.best_strategies(5);
    if !best.is_empty() {
        lines.push(String::new());
        lines.push("Best strategies:".to_string());
        for (strategy, rate) in best {
            lines.push(format!("  {}: {}% success", strategy, rate));
        }
    }

    lines.join("\n")
}

/// Format error recovery tracker compact
pub fn format_error_recovery_tracker_compact(tracker: &ErrorRecoveryTracker) -> String {
    format!(
        "Errors: {} total | {} recovered | {}% rate",
        tracker.total_errors,
        tracker.total_recovered,
        tracker.recovery_rate()
    )
}

/// Format error recovery tracker one-line
pub fn format_error_recovery_tracker_oneline(tracker: &ErrorRecoveryTracker) -> String {
    format!(
        "{} errors ({}% recovered)",
        tracker.total_errors,
        tracker.recovery_rate()
    )
}
