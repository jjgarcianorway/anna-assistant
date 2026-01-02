//! Package Tracker Utilities
//!
//! Helper functions for package tracker queries and fun facts.

use super::tracker::PackageTracker;

/// Check if query is about packages
pub fn is_package_tracker_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "installed packages",
        "package history",
        "what packages",
        "packages installed",
        "anna installed",
        "package tracker",
        "installed by anna",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about packages
pub fn package_fun_fact(tracker: &PackageTracker) -> String {
    if tracker.records.is_empty() {
        return "No packages tracked yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has installed {} packages for you.",
            tracker.anna_installed_count
        ),
        format!(
            "{} packages are currently installed.",
            tracker.installed_count()
        ),
        {
            if let Some((manager, count)) = tracker.by_manager.iter().max_by_key(|(_, v)| *v) {
                format!("{} is the most used package manager ({} packages).", manager, count)
            } else {
                "No package manager stats yet.".to_string()
            }
        },
        format!(
            "{} packages have been removed.",
            tracker.removed().len()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
