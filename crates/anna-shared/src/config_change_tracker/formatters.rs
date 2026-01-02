//! Formatting functions for config change tracker

use super::types::ConfigChangeTracker;

/// Format config change tracker for display
pub fn format_config_tracker(tracker: &ConfigChangeTracker) -> String {
    let mut lines = vec!["=== Configuration Change History ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No config changes yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total changes: {}", tracker.total_count()));
    lines.push(format!("Unique files: {}", tracker.unique_files()));
    lines.push(format!("Rollbacks: {}", tracker.rollback_count));

    // By category
    if !tracker.by_category.is_empty() {
        lines.push(String::new());
        lines.push("By category:".to_string());
        for (cat, count) in &tracker.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    // Most changed
    if let Some((file, count)) = tracker.most_changed_file() {
        lines.push(String::new());
        lines.push(format!("Most changed: {} ({} changes)", file, count));
    }

    // Recent changes
    let recent = tracker.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent changes:".to_string());
        for change in recent {
            let symbol = change.change_type.symbol();
            let rolled_back = if change.rolled_back { " [rolled back]" } else { "" };
            lines.push(format!(
                "  [{}] {} - {}{}",
                symbol, change.file_path, change.target, rolled_back
            ));
        }
    }

    lines.join("\n")
}

/// Format config tracker compact
pub fn format_config_tracker_compact(tracker: &ConfigChangeTracker) -> String {
    format!(
        "Config: {} changes | {} files | {} rollbacks",
        tracker.total_count(),
        tracker.unique_files(),
        tracker.rollback_count
    )
}

/// Format config tracker one-line
pub fn format_config_tracker_oneline(tracker: &ConfigChangeTracker) -> String {
    format!(
        "{} config changes ({} files)",
        tracker.total_count(),
        tracker.unique_files()
    )
}
