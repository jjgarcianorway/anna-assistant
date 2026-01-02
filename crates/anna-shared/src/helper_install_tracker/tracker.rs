// v0.0.532: Helper Install Tracker (Phase 108)
// Tracks helper tools installed by Anna vs user per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::record::HelperRecord;
use super::types::{HelperCategory, HelperInstaller};

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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::HelperCategory;

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
}
