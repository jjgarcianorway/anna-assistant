// v0.0.532: Helper Record (Phase 108)
// Individual helper installation record

use serde::{Deserialize, Serialize};
use super::types::{HelperCategory, HelperInstaller, HelperStatus};

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
}
