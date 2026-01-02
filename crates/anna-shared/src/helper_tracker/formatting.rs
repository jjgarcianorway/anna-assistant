//! Formatting functions for helper tracker display

use super::tracker::HelperTracker;

/// Format helper tracker for display
pub fn format_helper_tracker(tracker: &HelperTracker) -> String {
    let mut lines = vec!["=== Helper Tools ===".to_string()];
    lines.push(String::new());

    if tracker.helpers.is_empty() {
        lines.push("No helpers registered yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total helpers: {}", tracker.total_count()));
    lines.push(format!("Available: {}", tracker.available_count()));
    lines.push(format!("Total usage: {}", tracker.total_usage));

    // By source
    if !tracker.by_source.is_empty() {
        lines.push(String::new());
        lines.push("By installer:".to_string());
        for (source, count) in &tracker.by_source {
            lines.push(format!("  {}: {}", source, count));
        }
    }

    // Most used
    if let Some((name, count)) = tracker.most_used() {
        lines.push(String::new());
        lines.push(format!("Most used: {} ({} times)", name, count));
    }

    // Anna-installed (removable on uninstall)
    let removable = tracker.removable_on_uninstall();
    if !removable.is_empty() {
        lines.push(String::new());
        lines.push(format!("Anna-installed (removable): {}", removable.len()));
        for h in removable.iter().take(5) {
            lines.push(format!("  - {}", h.name));
        }
    }

    lines.join("\n")
}

/// Format helper tracker compact
pub fn format_helper_tracker_compact(tracker: &HelperTracker) -> String {
    let anna_count = tracker.anna_installed().len();
    let user_count = tracker.user_installed().len();
    format!(
        "Helpers: {} total | {} Anna-installed | {} user-installed",
        tracker.total_count(),
        anna_count,
        user_count
    )
}

/// Format helper tracker one-line
pub fn format_helper_tracker_oneline(tracker: &HelperTracker) -> String {
    format!(
        "{} helpers ({} available)",
        tracker.total_count(),
        tracker.available_count()
    )
}

/// Generate fun fact about helpers
pub fn helper_fun_fact(tracker: &HelperTracker) -> String {
    if tracker.helpers.is_empty() {
        return "No helper tools registered yet!".to_string();
    }

    let facts = [
        format!("Anna knows about {} helper tools.", tracker.total_count()),
        format!("{} helpers are currently available.", tracker.available_count()),
        {
            if let Some((name, count)) = tracker.most_used() {
                format!("{} is the most used helper ({} times).", name, count)
            } else {
                "No helper usage recorded yet.".to_string()
            }
        },
        format!(
            "{} helpers were installed by Anna.",
            tracker.anna_installed().len()
        ),
        format!(
            "{} helpers can be removed on uninstall.",
            tracker.removable_on_uninstall().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
