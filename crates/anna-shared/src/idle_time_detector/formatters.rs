//! Formatting functions for idle time display

use super::tracker::IdleTimeTracker;

/// Format idle tracker for display
pub fn format_idle_tracker(tracker: &IdleTimeTracker) -> String {
    let mut lines = vec!["=== Idle Time Tracker ===".to_string()];
    lines.push(String::new());

    // Current state
    lines.push(format!("Current state: {} [{}]",
        tracker.current_state.name(),
        tracker.current_state.symbol()
    ));

    lines.push(format!("Background work: {}",
        if tracker.can_do_background_work() { "allowed" } else { "not allowed" }
    ));

    // Stats
    lines.push(String::new());
    lines.push(format!("Total idle time: {} min", tracker.total_idle_secs / 60));
    lines.push(format!("Idle periods: {}", tracker.period_count()));
    lines.push(format!("Avg idle duration: {:.1} min", tracker.avg_idle_duration() / 60.0));
    lines.push(format!("Tasks completed: {}", tracker.tasks_completed));

    // Longest
    if let Some(longest) = tracker.longest_idle() {
        lines.push(format!("Longest idle: {} min", longest / 60));
    }

    // Config
    lines.push(String::new());
    lines.push(format!("Idle threshold: {} sec", tracker.config.idle_threshold_secs));
    lines.push(format!("Deep idle: {} sec", tracker.config.deep_idle_threshold_secs));

    lines.join("\n")
}

/// Format idle tracker compact
pub fn format_idle_tracker_compact(tracker: &IdleTimeTracker) -> String {
    format!(
        "Idle: {} | {} min total | {} tasks",
        tracker.current_state.name(),
        tracker.total_idle_secs / 60,
        tracker.tasks_completed
    )
}

/// Format idle tracker one-line
pub fn format_idle_tracker_oneline(tracker: &IdleTimeTracker) -> String {
    format!(
        "{} ({} periods)",
        tracker.current_state.name(),
        tracker.period_count()
    )
}
