//! Dependency Tracker - Phase 93
//!
//! Tracks software dependencies Anna manages.
//! VISION.md: Know what packages depend on what for safe removals.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dependency type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DependencyType {
    #[default]
    Runtime,
    Build,
    Optional,
    Recommended,
    Suggested,
    Conflict,
}

impl DependencyType {
    pub fn name(&self) -> &'static str {
        match self {
            DependencyType::Runtime => "Runtime",
            DependencyType::Build => "Build",
            DependencyType::Optional => "Optional",
            DependencyType::Recommended => "Recommended",
            DependencyType::Suggested => "Suggested",
            DependencyType::Conflict => "Conflict",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            DependencyType::Runtime => "→",
            DependencyType::Build => "⚙",
            DependencyType::Optional => "?",
            DependencyType::Recommended => "+",
            DependencyType::Suggested => "~",
            DependencyType::Conflict => "!",
        }
    }
}

/// Dependency status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DependencyStatus {
    #[default]
    Installed,
    Missing,
    Outdated,
    Orphaned,
    Unknown,
}

impl DependencyStatus {
    pub fn name(&self) -> &'static str {
        match self {
            DependencyStatus::Installed => "Installed",
            DependencyStatus::Missing => "Missing",
            DependencyStatus::Outdated => "Outdated",
            DependencyStatus::Orphaned => "Orphaned",
            DependencyStatus::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            DependencyStatus::Installed => "✓",
            DependencyStatus::Missing => "✗",
            DependencyStatus::Outdated => "↑",
            DependencyStatus::Orphaned => "○",
            DependencyStatus::Unknown => "?",
        }
    }
}

/// A dependency record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRecord {
    /// Package name
    pub package: String,
    /// Dependency name
    pub dependency: String,
    /// Type of dependency
    pub dep_type: DependencyType,
    /// Current status
    pub status: DependencyStatus,
    /// Required version (if any)
    pub version_req: Option<String>,
    /// Installed version (if any)
    pub installed_version: Option<String>,
    /// When last checked
    pub last_check: u64,
}

/// Dependency tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyTracker {
    /// All dependency records
    pub records: Vec<DependencyRecord>,
    /// Count by type
    pub by_type: HashMap<String, u64>,
    /// Count by status
    pub by_status: HashMap<String, u64>,
    /// Packages with missing dependencies
    pub broken_packages: Vec<String>,
    /// Last full scan
    pub last_scan: Option<u64>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency record
    pub fn add(&mut self, record: DependencyRecord) {
        *self.by_type.entry(record.dep_type.name().to_string()).or_insert(0) += 1;
        *self.by_status.entry(record.status.name().to_string()).or_insert(0) += 1;

        if record.status == DependencyStatus::Missing {
            if !self.broken_packages.contains(&record.package) {
                self.broken_packages.push(record.package.clone());
            }
        }
        self.records.push(record);
    }

    /// Get dependencies for a package
    pub fn deps_for(&self, package: &str) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.package == package).collect()
    }

    /// Get reverse dependencies (what depends on this)
    pub fn reverse_deps(&self, dependency: &str) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.dependency == dependency).collect()
    }

    /// Check if package has missing deps
    pub fn has_missing(&self, package: &str) -> bool {
        self.records
            .iter()
            .any(|r| r.package == package && r.status == DependencyStatus::Missing)
    }

    /// Check if package is safe to remove (nothing depends on it)
    pub fn safe_to_remove(&self, package: &str) -> bool {
        !self.records.iter().any(|r| {
            r.dependency == package
                && r.status == DependencyStatus::Installed
                && r.dep_type == DependencyType::Runtime
        })
    }

    /// Get orphaned packages (installed but nothing depends on them)
    pub fn orphaned(&self) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.status == DependencyStatus::Orphaned).collect()
    }

    /// Get missing dependencies
    pub fn missing(&self) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.status == DependencyStatus::Missing).collect()
    }

    /// Get outdated dependencies
    pub fn outdated(&self) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.status == DependencyStatus::Outdated).collect()
    }

    /// Get by type
    pub fn by_dep_type(&self, dep_type: DependencyType) -> Vec<&DependencyRecord> {
        self.records.iter().filter(|r| r.dep_type == dep_type).collect()
    }

    /// Update dependency status
    pub fn update_status(&mut self, package: &str, dep: &str, status: DependencyStatus) -> bool {
        let found = self.records.iter().position(|r| r.package == package && r.dependency == dep);
        if let Some(idx) = found {
            let old_status = self.records[idx].status;
            if let Some(count) = self.by_status.get_mut(old_status.name()) {
                *count = count.saturating_sub(1);
            }
            *self.by_status.entry(status.name().to_string()).or_insert(0) += 1;
            self.records[idx].status = status;
            true
        } else {
            false
        }
    }

    /// Total record count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Missing count
    pub fn missing_count(&self) -> usize {
        self.records.iter().filter(|r| r.status == DependencyStatus::Missing).count()
    }

    /// Record a full scan
    pub fn record_scan(&mut self, timestamp: u64) {
        self.last_scan = Some(timestamp);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dep(pkg: &str, dep: &str, dep_type: DependencyType, status: DependencyStatus) -> DependencyRecord {
        DependencyRecord {
            package: pkg.to_string(),
            dependency: dep.to_string(),
            dep_type,
            status,
            version_req: Some(">=1.0".to_string()),
            installed_version: Some("1.2.3".to_string()),
            last_check: 1234567890,
        }
    }

    #[test]
    fn test_dependency_type() {
        assert_eq!(DependencyType::Runtime.name(), "Runtime");
        assert_eq!(DependencyType::Build.symbol(), "⚙");
    }

    #[test]
    fn test_dependency_status() {
        assert_eq!(DependencyStatus::Missing.name(), "Missing");
        assert_eq!(DependencyStatus::Missing.symbol(), "✗");
    }

    #[test]
    fn test_add_dependency() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.deps_for("app").len(), 1);
    }

    #[test]
    fn test_reverse_deps() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app1", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));
        tracker.add(make_dep("app2", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        assert_eq!(tracker.reverse_deps("libfoo").len(), 2);
    }

    #[test]
    fn test_broken_packages() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Missing));

        assert!(tracker.has_missing("app"));
        assert!(tracker.broken_packages.contains(&"app".to_string()));
    }

    #[test]
    fn test_safe_to_remove() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        // libfoo is needed by app, not safe to remove
        assert!(!tracker.safe_to_remove("libfoo"));
        // app has nothing depending on it
        assert!(tracker.safe_to_remove("app"));
    }

    #[test]
    fn test_update_status() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Missing));

        assert!(tracker.update_status("app", "libfoo", DependencyStatus::Installed));
        assert_eq!(tracker.deps_for("app")[0].status, DependencyStatus::Installed);
    }

    #[test]
    fn test_by_type() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));
        tracker.add(make_dep("app", "cmake", DependencyType::Build, DependencyStatus::Installed));

        assert_eq!(tracker.by_dep_type(DependencyType::Runtime).len(), 1);
        assert_eq!(tracker.by_dep_type(DependencyType::Build).len(), 1);
    }

    #[test]
    fn test_format_tracker() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        let output = format_dependency_tracker(&tracker);
        assert!(output.contains("Dependency Tracker"));
        assert!(output.contains("Total dependencies: 1"));
    }

    #[test]
    fn test_is_dependency_query() {
        assert!(is_dependency_query("what are the dependencies?"));
        assert!(is_dependency_query("what depends on libfoo"));
        assert!(is_dependency_query("show orphan packages"));
        assert!(!is_dependency_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = DependencyTracker::new();
        tracker.add(make_dep("app", "libfoo", DependencyType::Runtime, DependencyStatus::Installed));

        let fact = dependency_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
