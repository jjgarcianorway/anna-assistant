//! Package Installation Tracker - Phase 80
//!
//! Tracks packages installed by Anna vs user-installed packages.
//! VISION.md mentions tracking what Anna installed vs user installed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Who installed the package
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstalledBy {
    Anna,
    User,
    System,
    Unknown,
}

impl InstalledBy {
    pub fn symbol(&self) -> &'static str {
        match self {
            InstalledBy::Anna => "A",
            InstalledBy::User => "U",
            InstalledBy::System => "S",
            InstalledBy::Unknown => "?",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            InstalledBy::Anna => "installed by Anna",
            InstalledBy::User => "installed by user",
            InstalledBy::System => "system package",
            InstalledBy::Unknown => "unknown source",
        }
    }
}

/// Package manager type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManager {
    Pacman,
    Apt,
    Dnf,
    Zypper,
    Flatpak,
    Snap,
    Pip,
    Npm,
    Cargo,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "pacman",
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Zypper => "zypper",
            PackageManager::Flatpak => "flatpak",
            PackageManager::Snap => "snap",
            PackageManager::Pip => "pip",
            PackageManager::Npm => "npm",
            PackageManager::Cargo => "cargo",
        }
    }
}

/// A single package installation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRecord {
    /// Package name
    pub name: String,
    /// Version if known
    pub version: Option<String>,
    /// Who installed it
    pub installed_by: InstalledBy,
    /// Package manager used
    pub manager: PackageManager,
    /// Timestamp when installed
    pub installed_at: u64,
    /// Why it was installed
    pub reason: Option<String>,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// Whether currently installed
    pub is_installed: bool,
    /// Timestamp when removed (if removed)
    pub removed_at: Option<u64>,
}

/// Package installation tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageTracker {
    /// All package records
    pub records: Vec<PackageRecord>,
    /// Count by installer
    pub by_installer: HashMap<String, u64>,
    /// Count by manager
    pub by_manager: HashMap<String, u64>,
    /// Total Anna-installed packages
    pub anna_installed_count: u64,
    /// Total user-installed packages
    pub user_installed_count: u64,
}

impl PackageTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a package installation
    pub fn record_install(&mut self, record: PackageRecord) {
        let installer_key = format!("{:?}", record.installed_by);
        let manager_key = record.manager.name().to_string();

        *self.by_installer.entry(installer_key).or_insert(0) += 1;
        *self.by_manager.entry(manager_key).or_insert(0) += 1;

        match record.installed_by {
            InstalledBy::Anna => self.anna_installed_count += 1,
            InstalledBy::User => self.user_installed_count += 1,
            _ => {}
        }

        self.records.push(record);
    }

    /// Mark a package as removed
    pub fn record_removal(&mut self, name: &str) -> bool {
        let found = self.records.iter().position(|r| r.name == name && r.is_installed);
        if let Some(idx) = found {
            self.records[idx].is_installed = false;
            self.records[idx].removed_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            );
            true
        } else {
            false
        }
    }

    /// Get package by name
    pub fn get(&self, name: &str) -> Option<&PackageRecord> {
        self.records.iter().find(|r| r.name == name)
    }

    /// Get currently installed packages
    pub fn installed(&self) -> Vec<&PackageRecord> {
        self.records.iter().filter(|r| r.is_installed).collect()
    }

    /// Get packages installed by Anna
    pub fn anna_installed(&self) -> Vec<&PackageRecord> {
        self.records
            .iter()
            .filter(|r| r.installed_by == InstalledBy::Anna && r.is_installed)
            .collect()
    }

    /// Get packages installed by user
    pub fn user_installed(&self) -> Vec<&PackageRecord> {
        self.records
            .iter()
            .filter(|r| r.installed_by == InstalledBy::User && r.is_installed)
            .collect()
    }

    /// Get recent installations
    pub fn recent(&self, limit: usize) -> Vec<&PackageRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get packages by manager
    pub fn by_package_manager(&self, manager: PackageManager) -> Vec<&PackageRecord> {
        self.records.iter().filter(|r| r.manager == manager).collect()
    }

    /// Total package count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Currently installed count
    pub fn installed_count(&self) -> usize {
        self.installed().len()
    }

    /// Get removed packages
    pub fn removed(&self) -> Vec<&PackageRecord> {
        self.records.iter().filter(|r| !r.is_installed).collect()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_package(name: &str, by: InstalledBy) -> PackageRecord {
        PackageRecord {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            installed_by: by,
            manager: PackageManager::Pacman,
            installed_at: 1234567890,
            reason: Some("test".to_string()),
            ticket_id: None,
            is_installed: true,
            removed_at: None,
        }
    }

    #[test]
    fn test_installed_by() {
        assert_eq!(InstalledBy::Anna.symbol(), "A");
        assert_eq!(InstalledBy::User.description(), "installed by user");
    }

    #[test]
    fn test_package_manager() {
        assert_eq!(PackageManager::Pacman.name(), "pacman");
        assert_eq!(PackageManager::Apt.name(), "apt");
    }

    #[test]
    fn test_package_tracker_record() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.anna_installed_count, 1);
    }

    #[test]
    fn test_package_tracker_removal() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        assert!(tracker.record_removal("vim"));
        assert_eq!(tracker.installed_count(), 0);
        assert_eq!(tracker.removed().len(), 1);
    }

    #[test]
    fn test_anna_installed() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));
        tracker.record_install(make_package("htop", InstalledBy::User));

        assert_eq!(tracker.anna_installed().len(), 1);
        assert_eq!(tracker.user_installed().len(), 1);
    }

    #[test]
    fn test_by_package_manager() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let mut pkg = make_package("code", InstalledBy::User);
        pkg.manager = PackageManager::Flatpak;
        tracker.record_install(pkg);

        assert_eq!(tracker.by_package_manager(PackageManager::Pacman).len(), 1);
        assert_eq!(tracker.by_package_manager(PackageManager::Flatpak).len(), 1);
    }

    #[test]
    fn test_format_package_tracker() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let output = format_package_tracker(&tracker);
        assert!(output.contains("Package Installation History"));
        assert!(output.contains("Total tracked: 1"));
    }

    #[test]
    fn test_is_package_tracker_query() {
        assert!(is_package_tracker_query("show installed packages"));
        assert!(is_package_tracker_query("what packages did anna install?"));
        assert!(is_package_tracker_query("package history"));
        assert!(!is_package_tracker_query("what is my disk space?"));
    }

    #[test]
    fn test_package_fun_fact() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let fact = package_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut tracker = PackageTracker::new();
        tracker.record_install(make_package("vim", InstalledBy::Anna));

        let compact = format_package_tracker_compact(&tracker);
        assert!(compact.contains("Packages: 1 installed"));

        let oneline = format_package_tracker_oneline(&tracker);
        assert!(oneline.contains("1 packages"));
    }
}
