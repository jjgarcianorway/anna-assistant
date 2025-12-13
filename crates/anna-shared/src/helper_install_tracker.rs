// v0.0.532: Helper Install Tracker (Phase 108)
// Tracks helper tools installed by Anna vs user per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Who installed the helper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum HelperInstaller {
    #[default]
    User,
    Anna,
    System,
}

impl std::fmt::Display for HelperInstaller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Anna => write!(f, "Anna"),
            Self::System => write!(f, "System"),
        }
    }
}

/// Helper category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HelperCategory {
    SystemInfo,
    NetworkDiag,
    DiskUtils,
    HardwareProbe,
    AudioVideo,
    Security,
    DevTools,
    Monitoring,
}

impl Default for HelperCategory {
    fn default() -> Self {
        Self::SystemInfo
    }
}

impl std::fmt::Display for HelperCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SystemInfo => write!(f, "System Info"),
            Self::NetworkDiag => write!(f, "Network Diagnostics"),
            Self::DiskUtils => write!(f, "Disk Utilities"),
            Self::HardwareProbe => write!(f, "Hardware Probe"),
            Self::AudioVideo => write!(f, "Audio/Video"),
            Self::Security => write!(f, "Security"),
            Self::DevTools => write!(f, "Dev Tools"),
            Self::Monitoring => write!(f, "Monitoring"),
        }
    }
}

/// Helper installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HelperStatus {
    #[default]
    NotInstalled,
    Installing,
    Installed,
    Failed,
    Removed,
}

impl std::fmt::Display for HelperStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "Not Installed"),
            Self::Installing => write!(f, "Installing"),
            Self::Installed => write!(f, "Installed"),
            Self::Failed => write!(f, "Failed"),
            Self::Removed => write!(f, "Removed"),
        }
    }
}

/// Individual helper record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperRecord {
    pub name: String,
    pub package: String,
    pub category: HelperCategory,
    pub status: HelperStatus,
    pub installed_by: HelperInstaller,
    pub purpose: String,
    pub usage_count: u32,
    pub installed_at: Option<String>,
    pub last_used: Option<String>,
    pub hardware_required: Option<String>,
}

impl HelperRecord {
    /// Create a new helper record
    pub fn new(name: &str, package: &str, category: HelperCategory, purpose: &str) -> Self {
        Self {
            name: name.to_string(),
            package: package.to_string(),
            category,
            status: HelperStatus::NotInstalled,
            installed_by: HelperInstaller::User,
            purpose: purpose.to_string(),
            usage_count: 0,
            installed_at: None,
            last_used: None,
            hardware_required: None,
        }
    }

    /// Set hardware requirement
    pub fn requires_hardware(&mut self, hw: &str) {
        self.hardware_required = Some(hw.to_string());
    }

    /// Install helper
    pub fn install(&mut self, by: HelperInstaller, timestamp: &str) {
        self.status = HelperStatus::Installed;
        self.installed_by = by;
        self.installed_at = Some(timestamp.to_string());
    }

    /// Mark as removed
    pub fn remove(&mut self) {
        self.status = HelperStatus::Removed;
    }

    /// Record usage
    pub fn record_use(&mut self, timestamp: &str) {
        self.usage_count += 1;
        self.last_used = Some(timestamp.to_string());
    }

    /// Is installed?
    pub fn is_installed(&self) -> bool {
        self.status == HelperStatus::Installed
    }

    /// Should remove on uninstall?
    pub fn remove_on_uninstall(&self) -> bool {
        self.installed_by == HelperInstaller::Anna && self.is_installed()
    }
}

/// Helper install tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperInstallTracker {
    helpers: HashMap<String, HelperRecord>,
}

impl HelperInstallTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            helpers: HashMap::new(),
        }
    }

    /// Register a helper
    pub fn register(&mut self, helper: HelperRecord) {
        self.helpers.insert(helper.name.clone(), helper);
    }

    /// Get helper by name
    pub fn get(&self, name: &str) -> Option<&HelperRecord> {
        self.helpers.get(name)
    }

    /// Get mutable helper
    pub fn get_mut(&mut self, name: &str) -> Option<&mut HelperRecord> {
        self.helpers.get_mut(name)
    }

    /// Get installed helpers
    pub fn installed(&self) -> Vec<&HelperRecord> {
        self.helpers.values().filter(|h| h.is_installed()).collect()
    }

    /// Get helpers installed by Anna
    pub fn installed_by_anna(&self) -> Vec<&HelperRecord> {
        self.helpers
            .values()
            .filter(|h| h.installed_by == HelperInstaller::Anna && h.is_installed())
            .collect()
    }

    /// Get helpers to remove on uninstall
    pub fn to_remove_on_uninstall(&self) -> Vec<&HelperRecord> {
        self.helpers
            .values()
            .filter(|h| h.remove_on_uninstall())
            .collect()
    }

    /// Get helpers by category
    pub fn by_category(&self, cat: &HelperCategory) -> Vec<&HelperRecord> {
        self.helpers
            .values()
            .filter(|h| &h.category == cat && h.is_installed())
            .collect()
    }

    /// Check if helper would be useless (no required hardware)
    pub fn would_be_useless(&self, name: &str, available_hw: &[String]) -> bool {
        if let Some(helper) = self.helpers.get(name) {
            if let Some(required) = &helper.hardware_required {
                return !available_hw.iter().any(|hw| hw.contains(required));
            }
        }
        false
    }

    /// Get most used helpers
    pub fn most_used(&self, n: usize) -> Vec<&HelperRecord> {
        let mut list: Vec<_> = self.installed().into_iter().collect();
        list.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        list.into_iter().take(n).collect()
    }

    /// Category stats
    pub fn category_stats(&self) -> HashMap<HelperCategory, usize> {
        let mut stats = HashMap::new();
        for h in self.installed() {
            *stats.entry(h.category.clone()).or_insert(0) += 1;
        }
        stats
    }

    /// Total helpers
    pub fn total(&self) -> usize {
        self.helpers.len()
    }

    /// Installed count
    pub fn installed_count(&self) -> usize {
        self.installed().len()
    }

    /// All helpers
    pub fn all(&self) -> Vec<&HelperRecord> {
        self.helpers.values().collect()
    }
}

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

/// Check if query is helper-related
pub fn is_helper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("helper")
        || lower.contains("tool")
        || lower.contains("install")
        || lower.contains("package")
        || lower.contains("utility")
}

/// Fun fact about helpers
pub fn helper_fun_fact() -> &'static str {
    "Anna only installs helpers that are actually useful - no ethtool if you don't have ethernet! This keeps your system clean and efficient."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_creation() {
        let helper = HelperRecord::new("htop", "htop", HelperCategory::Monitoring, "Process viewer");
        assert_eq!(helper.name, "htop");
        assert_eq!(helper.status, HelperStatus::NotInstalled);
    }

    #[test]
    fn test_helper_install() {
        let mut helper = HelperRecord::new("lsof", "lsof", HelperCategory::SystemInfo, "List open files");
        helper.install(HelperInstaller::Anna, "2024-01-01");
        assert!(helper.is_installed());
        assert_eq!(helper.installed_by, HelperInstaller::Anna);
    }

    #[test]
    fn test_remove_on_uninstall() {
        let mut anna_helper = HelperRecord::new("a", "a", HelperCategory::SystemInfo, "p");
        anna_helper.install(HelperInstaller::Anna, "ts");
        let mut user_helper = HelperRecord::new("b", "b", HelperCategory::SystemInfo, "p");
        user_helper.install(HelperInstaller::User, "ts");
        assert!(anna_helper.remove_on_uninstall());
        assert!(!user_helper.remove_on_uninstall());
    }

    #[test]
    fn test_record_use() {
        let mut helper = HelperRecord::new("test", "test", HelperCategory::DevTools, "test");
        helper.install(HelperInstaller::User, "ts");
        helper.record_use("2024-01-02");
        assert_eq!(helper.usage_count, 1);
    }

    #[test]
    fn test_tracker_register() {
        let mut tracker = HelperInstallTracker::new();
        let helper = HelperRecord::new("htop", "htop", HelperCategory::Monitoring, "p");
        tracker.register(helper);
        assert_eq!(tracker.total(), 1);
    }

    #[test]
    fn test_installed_by_anna() {
        let mut tracker = HelperInstallTracker::new();
        let mut h1 = HelperRecord::new("a", "a", HelperCategory::SystemInfo, "p");
        h1.install(HelperInstaller::Anna, "ts");
        let mut h2 = HelperRecord::new("b", "b", HelperCategory::SystemInfo, "p");
        h2.install(HelperInstaller::User, "ts");
        tracker.register(h1);
        tracker.register(h2);
        assert_eq!(tracker.installed_by_anna().len(), 1);
    }

    #[test]
    fn test_by_category() {
        let mut tracker = HelperInstallTracker::new();
        let mut h1 = HelperRecord::new("a", "a", HelperCategory::NetworkDiag, "p");
        h1.install(HelperInstaller::User, "ts");
        let mut h2 = HelperRecord::new("b", "b", HelperCategory::NetworkDiag, "p");
        h2.install(HelperInstaller::User, "ts");
        tracker.register(h1);
        tracker.register(h2);
        assert_eq!(tracker.by_category(&HelperCategory::NetworkDiag).len(), 2);
    }

    #[test]
    fn test_would_be_useless() {
        let mut tracker = HelperInstallTracker::new();
        let mut helper = HelperRecord::new("ethtool", "ethtool", HelperCategory::NetworkDiag, "p");
        helper.requires_hardware("ethernet");
        tracker.register(helper);
        assert!(tracker.would_be_useless("ethtool", &["wifi".to_string()]));
        assert!(!tracker.would_be_useless("ethtool", &["ethernet".to_string()]));
    }

    #[test]
    fn test_is_helper_query() {
        assert!(is_helper_query("What helpers are installed?"));
        assert!(is_helper_query("Install a tool"));
        assert!(is_helper_query("Which packages did Anna install?"));
        assert!(!is_helper_query("What is my IP?"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = helper_fun_fact();
        assert!(fact.contains("ethtool") || fact.contains("useful"));
    }
}
