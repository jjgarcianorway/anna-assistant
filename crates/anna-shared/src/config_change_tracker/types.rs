//! Core types for config change tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of configuration change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    Add,
    Modify,
    Delete,
    Replace,
    Append,
    Comment,
    Uncomment,
}

impl ChangeType {
    pub fn symbol(&self) -> &'static str {
        match self {
            ChangeType::Add => "+",
            ChangeType::Modify => "~",
            ChangeType::Delete => "-",
            ChangeType::Replace => "=",
            ChangeType::Append => ">",
            ChangeType::Comment => "#",
            ChangeType::Uncomment => "!",
        }
    }

    pub fn verb(&self) -> &'static str {
        match self {
            ChangeType::Add => "added",
            ChangeType::Modify => "modified",
            ChangeType::Delete => "deleted",
            ChangeType::Replace => "replaced",
            ChangeType::Append => "appended",
            ChangeType::Comment => "commented",
            ChangeType::Uncomment => "uncommented",
        }
    }
}

/// Category of config file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigCategory {
    Shell,
    Editor,
    Git,
    System,
    Service,
    Application,
    Network,
    Security,
    Other,
}

impl ConfigCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ConfigCategory::Shell => "Shell",
            ConfigCategory::Editor => "Editor",
            ConfigCategory::Git => "Git",
            ConfigCategory::System => "System",
            ConfigCategory::Service => "Service",
            ConfigCategory::Application => "Application",
            ConfigCategory::Network => "Network",
            ConfigCategory::Security => "Security",
            ConfigCategory::Other => "Other",
        }
    }
}

/// A single config change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeRecord {
    /// Unique change ID
    pub id: String,
    /// File path that was changed
    pub file_path: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Category of config
    pub category: ConfigCategory,
    /// What was changed (line/setting name)
    pub target: String,
    /// Old value (if applicable)
    pub old_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// Reason for change
    pub reason: Option<String>,
    /// Whether user confirmed
    pub user_confirmed: bool,
    /// Backup ID if backup was created
    pub backup_id: Option<String>,
    /// Whether change was rolled back
    pub rolled_back: bool,
}

/// Config change tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigChangeTracker {
    /// All change records
    pub records: Vec<ConfigChangeRecord>,
    /// Count by file
    pub by_file: HashMap<String, u64>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Count by change type
    pub by_type: HashMap<String, u64>,
    /// Total rollbacks
    pub rollback_count: u64,
}

impl ConfigChangeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a config change
    pub fn record(&mut self, change: ConfigChangeRecord) {
        *self.by_file.entry(change.file_path.clone()).or_insert(0) += 1;
        *self.by_category.entry(change.category.name().to_string()).or_insert(0) += 1;
        *self.by_type.entry(format!("{:?}", change.change_type)).or_insert(0) += 1;

        self.records.push(change);
    }

    /// Mark a change as rolled back
    pub fn mark_rolled_back(&mut self, id: &str) -> bool {
        let found = self.records.iter().position(|r| r.id == id);
        if let Some(idx) = found {
            self.records[idx].rolled_back = true;
            self.rollback_count += 1;
            true
        } else {
            false
        }
    }

    /// Get recent changes
    pub fn recent(&self, limit: usize) -> Vec<&ConfigChangeRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get changes for a file
    pub fn for_file(&self, path: &str) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.file_path == path).collect()
    }

    /// Get changes by category
    pub fn by_config_category(&self, category: ConfigCategory) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.category == category).collect()
    }

    /// Get changes by type
    pub fn by_change_type(&self, change_type: ChangeType) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.change_type == change_type).collect()
    }

    /// Get rolled back changes
    pub fn rolled_back(&self) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.rolled_back).collect()
    }

    /// Get active (not rolled back) changes
    pub fn active(&self) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| !r.rolled_back).collect()
    }

    /// Total change count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Unique files changed
    pub fn unique_files(&self) -> usize {
        self.by_file.len()
    }

    /// Most changed file
    pub fn most_changed_file(&self) -> Option<(&str, u64)> {
        self.by_file
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Most common category
    pub fn most_common_category(&self) -> Option<(&str, u64)> {
        self.by_category
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}
