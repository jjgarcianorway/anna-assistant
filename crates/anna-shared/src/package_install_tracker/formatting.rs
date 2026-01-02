//! Package Tracker Formatting
//!
//! Functions for formatting package tracker output.

use super::tracker::PackageTracker;

/// Format package tracker for display
pub fn format_package_tracker(tracker: &PackageTracker) -> String {
    let mut lines = vec!["=== Package Installation History ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No packages tracked yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total tracked: {}", tracker.total_count()));
    lines.push(format!("Currently installed: {}", tracker.installed_count()));
    lines.push(format!("Removed: {}", tracker.removed().len()));

    lines.push(String::new());
    lines.push("By installer:".to_string());
    lines.push(format!("  Anna: {}", tracker.anna_installed_count));
    lines.push(format!("  User: {}", tracker.user_installed_count));

    // By manager
    if !tracker.by_manager.is_empty() {
        lines.push(String::new());
        lines.push("By package manager:".to_string());
        for (manager, count) in &tracker.by_manager {
            lines.push(format!("  {}: {}", manager, count));
        }
    }

    // Recent installations
    let recent = tracker.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent packages:".to_string());
        for pkg in recent {
            let status = if pkg.is_installed { "+" } else { "-" };
            let version = pkg.version.as_deref().unwrap_or("?");
            lines.push(format!(
                "  [{}][{}] {} v{} ({})",
                status,
                pkg.installed_by.symbol(),
                pkg.name,
                version,
                pkg.manager.name()
            ));
        }
    }

    lines.join("\n")
}

/// Format package tracker compact
pub fn format_package_tracker_compact(tracker: &PackageTracker) -> String {
    format!(
        "Packages: {} installed | Anna: {} | User: {}",
        tracker.installed_count(),
        tracker.anna_installed().len(),
        tracker.user_installed().len()
    )
}

/// Format package tracker one-line
pub fn format_package_tracker_oneline(tracker: &PackageTracker) -> String {
    format!(
        "{} packages ({} by Anna)",
        tracker.installed_count(),
        tracker.anna_installed().len()
    )
}
