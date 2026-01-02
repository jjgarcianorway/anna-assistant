// v0.0.588: Settings Version History (Phase 164)
// Version history management and comparison

use crate::unified_settings::SettingsCategory;

use super::types::{SettingsVersion, VersionChange};

/// Version history
#[derive(Debug, Clone, Default)]
pub struct VersionHistory {
    /// All versions
    versions: Vec<SettingsVersion>,
    /// Current version
    current: u64,
    /// Max versions to keep
    max_versions: usize,
}

impl VersionHistory {
    /// Create new history
    pub fn new() -> Self {
        Self {
            max_versions: 100,
            ..Default::default()
        }
    }

    /// Set max versions
    pub fn max_versions(mut self, max: usize) -> Self {
        self.max_versions = max;
        self
    }

    /// Create new version
    pub fn create(&mut self, message: impl Into<String>) -> &mut SettingsVersion {
        self.current += 1;
        let version = SettingsVersion::new(self.current, message);
        self.versions.push(version);

        while self.versions.len() > self.max_versions {
            self.versions.remove(0);
        }

        self.versions.last_mut().unwrap()
    }

    /// Get current version
    pub fn current(&self) -> Option<&SettingsVersion> {
        self.versions.last()
    }

    /// Get version by number
    pub fn get(&self, version: u64) -> Option<&SettingsVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Get all versions
    pub fn all(&self) -> &[SettingsVersion] {
        &self.versions
    }

    /// Get recent versions
    pub fn recent(&self, count: usize) -> Vec<&SettingsVersion> {
        self.versions.iter().rev().take(count).collect()
    }

    /// Version count
    pub fn count(&self) -> usize {
        self.versions.len()
    }

    /// Current version number
    pub fn current_version(&self) -> u64 {
        self.current
    }

    /// Compare two versions
    pub fn compare(&self, v1: u64, v2: u64) -> Option<Vec<&VersionChange>> {
        let ver1 = self.get(v1)?;
        let ver2 = self.get(v2)?;

        // Get all changes between v1 and v2
        let min = v1.min(v2);
        let max = v1.max(v2);

        let changes: Vec<_> = self.versions
            .iter()
            .filter(|v| v.version > min && v.version <= max)
            .flat_map(|v| &v.changes)
            .collect();

        Some(changes)
    }

    /// Find version by message
    pub fn find_by_message(&self, search: &str) -> Vec<&SettingsVersion> {
        let lower = search.to_lowercase();
        self.versions
            .iter()
            .filter(|v| v.message.to_lowercase().contains(&lower))
            .collect()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.versions.clear();
        self.current = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_history_new() {
        let history = VersionHistory::new();
        assert_eq!(history.count(), 0);
    }

    #[test]
    fn test_version_history_create() {
        let mut history = VersionHistory::new();
        history.create("First version");
        assert_eq!(history.count(), 1);
        assert_eq!(history.current_version(), 1);
    }

    #[test]
    fn test_version_history_get() {
        let mut history = VersionHistory::new();
        history.create("Version 1");
        history.create("Version 2");
        assert!(history.get(1).is_some());
        assert!(history.get(2).is_some());
        assert!(history.get(3).is_none());
    }

    #[test]
    fn test_version_history_recent() {
        let mut history = VersionHistory::new();
        history.create("V1");
        history.create("V2");
        history.create("V3");
        let recent = history.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].version, 3);
    }
}
