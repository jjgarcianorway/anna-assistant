//! Formatting and query functions for dependency tracking

use super::tracker::DependencyTracker;

/// Format dependency tracker for display
pub fn format_dependency_tracker(tracker: &DependencyTracker) -> String {
    let mut lines = vec!["=== Dependency Tracker ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No dependencies tracked yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total dependencies: {}", tracker.total_count()));
    lines.push(format!("Missing: {}", tracker.missing_count()));
    lines.push(format!("Broken packages: {}", tracker.broken_packages.len()));

    // By type
    if !tracker.by_type.is_empty() {
        lines.push(String::new());
        lines.push("By type:".to_string());
        for (t, count) in &tracker.by_type {
            lines.push(format!("  {}: {}", t, count));
        }
    }

    // Missing deps
    let missing = tracker.missing();
    if !missing.is_empty() {
        lines.push(String::new());
        lines.push("Missing dependencies:".to_string());
        for dep in missing.iter().take(10) {
            lines.push(format!("  {} → {} (missing)", dep.package, dep.dependency));
        }
    }

    lines.join("\n")
}

/// Format dependency tracker compact
pub fn format_dependency_tracker_compact(tracker: &DependencyTracker) -> String {
    format!(
        "Dependencies: {} tracked | {} missing | {} broken",
        tracker.total_count(),
        tracker.missing_count(),
        tracker.broken_packages.len()
    )
}

/// Format dependency tracker one-line
pub fn format_dependency_tracker_oneline(tracker: &DependencyTracker) -> String {
    format!("{} deps ({} missing)", tracker.total_count(), tracker.missing_count())
}

/// Check if query is about dependencies
pub fn is_dependency_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "dependency",
        "dependencies",
        "depends on",
        "what depends",
        "reverse deps",
        "orphan",
        "broken package",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about dependencies
pub fn dependency_fun_fact(tracker: &DependencyTracker) -> String {
    if tracker.records.is_empty() {
        return "No dependencies tracked yet!".to_string();
    }

    let facts = [
        format!("Anna tracks {} dependency relationships.", tracker.total_count()),
        format!("{} dependencies are missing.", tracker.missing_count()),
        format!("{} packages have broken dependencies.", tracker.broken_packages.len()),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
