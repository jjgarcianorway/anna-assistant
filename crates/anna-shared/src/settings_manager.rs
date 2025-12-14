// v0.0.621: Settings Manager (Phase 197)
// Unified manager for all settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Manager mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ManagerMode {
    /// Normal mode
    #[default]
    Normal,
    /// Maintenance mode
    Maintenance,
    /// ReadOnly mode
    ReadOnly,
    /// Debug mode
    Debug,
}

impl std::fmt::Display for ManagerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Maintenance => write!(f, "maintenance"),
            Self::ReadOnly => write!(f, "read_only"),
            Self::Debug => write!(f, "debug"),
        }
    }
}

/// Manager status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ManagerStatus {
    /// Initializing
    Initializing,
    /// Active
    #[default]
    Active,
    /// Degraded
    Degraded,
    /// Inactive
    Inactive,
    /// Error
    Error,
}

impl std::fmt::Display for ManagerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "initializing"),
            Self::Active => write!(f, "active"),
            Self::Degraded => write!(f, "degraded"),
            Self::Inactive => write!(f, "inactive"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Setting entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Category
    pub category: SettingsCategory,
    /// Modified timestamp
    pub modified_at: u64,
    /// Read-only
    pub read_only: bool,
}

impl SettingEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<String>, category: SettingsCategory) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            category,
            modified_at: 0,
            read_only: false,
        }
    }

    /// Set modified timestamp
    pub fn modified_at(mut self, ts: u64) -> Self {
        self.modified_at = ts;
        self
    }

    /// Set read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

/// Manager operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerOperation {
    /// Operation ID
    pub id: String,
    /// Operation type
    pub op_type: String,
    /// Target key
    pub key: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// Success
    pub success: bool,
}

impl ManagerOperation {
    /// Create new operation
    pub fn new(id: impl Into<String>, op_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            op_type: op_type.into(),
            key: None,
            timestamp: 0,
            success: false,
        }
    }

    /// Set key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Mark success
    pub fn mark_success(&mut self) {
        self.success = true;
    }
}

/// Settings manager
#[derive(Debug, Clone, Default)]
pub struct SettingsManager {
    /// Mode
    mode: ManagerMode,
    /// Status
    status: ManagerStatus,
    /// Settings
    settings: HashMap<String, SettingEntry>,
    /// Operation history
    history: Vec<ManagerOperation>,
    /// Max history
    max_history: usize,
}

impl SettingsManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Get mode
    pub fn mode(&self) -> ManagerMode {
        self.mode
    }

    /// Set mode
    pub fn set_mode(&mut self, mode: ManagerMode) {
        self.mode = mode;
    }

    /// Get status
    pub fn status(&self) -> ManagerStatus {
        self.status
    }

    /// Set status
    pub fn set_status(&mut self, status: ManagerStatus) {
        self.status = status;
    }

    /// Get setting
    pub fn get(&self, key: &str) -> Option<&SettingEntry> {
        self.settings.get(key)
    }

    /// Set setting
    pub fn set(&mut self, entry: SettingEntry) -> bool {
        if self.mode == ManagerMode::ReadOnly {
            return false;
        }
        if let Some(existing) = self.settings.get(&entry.key) {
            if existing.read_only {
                return false;
            }
        }
        self.settings.insert(entry.key.clone(), entry);
        true
    }

    /// Delete setting
    pub fn delete(&mut self, key: &str) -> bool {
        if self.mode == ManagerMode::ReadOnly {
            return false;
        }
        if let Some(entry) = self.settings.get(key) {
            if entry.read_only {
                return false;
            }
        }
        self.settings.remove(key).is_some()
    }

    /// List by category
    pub fn list_by_category(&self, category: SettingsCategory) -> Vec<&SettingEntry> {
        self.settings.values().filter(|e| e.category == category).collect()
    }

    /// Record operation
    pub fn record(&mut self, operation: ManagerOperation) {
        self.history.push(operation);
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Setting count
    pub fn count(&self) -> usize {
        self.settings.len()
    }

    /// History
    pub fn history(&self) -> &[ManagerOperation] {
        &self.history
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.status == ManagerStatus::Active
    }
}

/// Format manager
pub fn format_manager(manager: &SettingsManager) -> String {
    let mut output = String::new();
    output.push_str("Settings Manager:\n");
    output.push_str(&format!("  Mode: {}\n", manager.mode()));
    output.push_str(&format!("  Status: {}\n", manager.status()));
    output.push_str(&format!("  Settings: {}\n", manager.count()));
    output.push_str(&format!("  History: {}\n", manager.history().len()));
    output
}

/// Check if query is about manager
pub fn is_manager_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("manager")
        || lower.contains("settings manager")
        || lower.contains("manage settings")
}

/// Fun fact about manager
pub fn manager_fun_fact() -> &'static str {
    "Anna's settings manager is the central hub for all settings operations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", ManagerMode::Normal), "normal");
        assert_eq!(format!("{}", ManagerMode::ReadOnly), "read_only");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ManagerStatus::Active), "active");
        assert_eq!(format!("{}", ManagerStatus::Degraded), "degraded");
    }

    #[test]
    fn test_entry_new() {
        let e = SettingEntry::new("key", "value", SettingsCategory::Personality);
        assert!(!e.read_only);
    }

    #[test]
    fn test_entry_builder() {
        let e = SettingEntry::new("key", "value", SettingsCategory::Risk)
            .read_only(true)
            .modified_at(100);
        assert!(e.read_only);
    }

    #[test]
    fn test_operation_new() {
        let o = ManagerOperation::new("op1", "get");
        assert!(!o.success);
    }

    #[test]
    fn test_operation_mark_success() {
        let mut o = ManagerOperation::new("op1", "set");
        o.mark_success();
        assert!(o.success);
    }

    #[test]
    fn test_manager_new() {
        let m = SettingsManager::new();
        assert!(m.is_active());
    }

    #[test]
    fn test_manager_set_get() {
        let mut m = SettingsManager::new();
        m.set(SettingEntry::new("k1", "v1", SettingsCategory::Privacy));
        assert!(m.get("k1").is_some());
    }

    #[test]
    fn test_manager_read_only_mode() {
        let mut m = SettingsManager::new();
        m.set_mode(ManagerMode::ReadOnly);
        let result = m.set(SettingEntry::new("k1", "v1", SettingsCategory::Privacy));
        assert!(!result);
    }

    #[test]
    fn test_manager_delete() {
        let mut m = SettingsManager::new();
        m.set(SettingEntry::new("k1", "v1", SettingsCategory::Privacy));
        assert!(m.delete("k1"));
    }

    #[test]
    fn test_is_manager_query() {
        assert!(is_manager_query("settings manager"));
        assert!(!is_manager_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = manager_fun_fact();
        assert!(fact.contains("manager"));
    }
}
