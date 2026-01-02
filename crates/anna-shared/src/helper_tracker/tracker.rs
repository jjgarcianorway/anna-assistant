//! Helper tracker implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{HelperPurpose, HelperRecord, InstallerSource};

/// Helper tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperTracker {
    /// All helper records
    pub helpers: Vec<HelperRecord>,
    /// Count by installer source
    pub by_source: HashMap<String, u64>,
    /// Count by purpose
    pub by_purpose: HashMap<String, u64>,
    /// Total usage count
    pub total_usage: u64,
}

impl HelperTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a helper
    pub fn register(&mut self, helper: HelperRecord) {
        *self.by_source.entry(helper.installed_by.name().to_string()).or_insert(0) += 1;
        *self.by_purpose.entry(helper.purpose.name().to_string()).or_insert(0) += 1;
        self.helpers.push(helper);
    }

    /// Record helper usage
    pub fn record_usage(&mut self, name: &str, timestamp: u64) -> bool {
        let found = self.helpers.iter().position(|h| h.name == name);
        if let Some(idx) = found {
            self.helpers[idx].usage_count += 1;
            self.helpers[idx].last_used = Some(timestamp);
            self.total_usage += 1;
            true
        } else {
            false
        }
    }

    /// Mark helper as unavailable
    pub fn mark_unavailable(&mut self, name: &str) -> bool {
        let found = self.helpers.iter().position(|h| h.name == name);
        if let Some(idx) = found {
            self.helpers[idx].available = false;
            true
        } else {
            false
        }
    }

    /// Get helpers installed by Anna
    pub fn anna_installed(&self) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.installed_by == InstallerSource::Anna).collect()
    }

    /// Get helpers installed by user
    pub fn user_installed(&self) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.installed_by == InstallerSource::User).collect()
    }

    /// Get available helpers
    pub fn available(&self) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.available).collect()
    }

    /// Get helpers by purpose
    pub fn by_helper_purpose(&self, purpose: HelperPurpose) -> Vec<&HelperRecord> {
        self.helpers.iter().filter(|h| h.purpose == purpose).collect()
    }

    /// Get helper by name
    pub fn get(&self, name: &str) -> Option<&HelperRecord> {
        self.helpers.iter().find(|h| h.name == name)
    }

    /// Check if helper exists
    pub fn has(&self, name: &str) -> bool {
        self.helpers.iter().any(|h| h.name == name)
    }

    /// Total helper count
    pub fn total_count(&self) -> usize {
        self.helpers.len()
    }

    /// Available helper count
    pub fn available_count(&self) -> usize {
        self.helpers.iter().filter(|h| h.available).count()
    }

    /// Most used helper
    pub fn most_used(&self) -> Option<(&str, u64)> {
        self.helpers
            .iter()
            .max_by_key(|h| h.usage_count)
            .map(|h| (h.name.as_str(), h.usage_count))
    }

    /// Most common purpose
    pub fn most_common_purpose(&self) -> Option<(&str, u64)> {
        self.by_purpose
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Helpers that can be removed on uninstall (Anna-installed only)
    pub fn removable_on_uninstall(&self) -> Vec<&HelperRecord> {
        self.helpers
            .iter()
            .filter(|h| h.installed_by == InstallerSource::Anna && h.available)
            .collect()
    }
}
