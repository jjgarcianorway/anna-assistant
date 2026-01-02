// v0.0.532: Helper Formatting Functions (Phase 108)
// Display and formatting utilities for helpers

use super::record::HelperRecord;
use super::tracker::HelperInstallTracker;

/// Format helper for display
pub fn format_helper(helper: &HelperRecord) -> String {
    format!(
        "{} ({})\n  Package: {} | Status: {}\n  Installed by: {} | Category: {}\n  Usage: {} times | Purpose: {}",
        helper.name,
        helper.status,
        helper.package,
        helper.status,
        helper.installed_by,
        helper.category,
        helper.usage_count,
        helper.purpose
    )
}

/// Format helper compact
pub fn format_helper_compact(helper: &HelperRecord) -> String {
    format!(
        "{} [{}] - {} ({})",
        helper.name, helper.installed_by, helper.category, helper.usage_count
    )
}

/// Format helper oneline
pub fn format_helper_oneline(helper: &HelperRecord) -> String {
    format!("{} [{}]", helper.name, helper.installed_by)
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &HelperInstallTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Helper Tools ===\n\n");

    output.push_str(&format!(
        "Total: {} | Installed: {}\n",
        tracker.total(),
        tracker.installed_count()
    ));

    let anna_helpers = tracker.installed_by_anna();
    output.push_str(&format!("Installed by Anna: {}\n\n", anna_helpers.len()));

    output.push_str("--- By Category ---\n");
    for (cat, count) in tracker.category_stats() {
        output.push_str(&format!("  {}: {}\n", cat, count));
    }

    if !anna_helpers.is_empty() {
        output.push_str("\n--- Anna-Installed (removed on uninstall) ---\n");
        for h in anna_helpers {
            output.push_str(&format!("  {}\n", h.name));
        }
    }

    output
}
