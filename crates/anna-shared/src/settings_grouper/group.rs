// v0.0.676: Settings Grouper - Group Data Structures (Phase 252)
// Settings group and group result

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Settings group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsGroup {
    /// Group name
    pub name: String,
    /// Entries in group
    pub entries: Vec<(String, String)>,
    /// Count
    pub count: usize,
}

impl SettingsGroup {
    /// Create new group
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
            count: 0,
        }
    }

    /// Add entry
    pub fn add(&mut self, key: String, value: String) {
        self.entries.push((key, value));
        self.count += 1;
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Group result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResult {
    /// Groups
    pub groups: HashMap<String, SettingsGroup>,
    /// Total entries
    pub total_entries: usize,
    /// Total groups
    pub total_groups: usize,
    /// Ungrouped count
    pub ungrouped: usize,
}

impl GroupResult {
    /// Create new result
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            total_entries: 0,
            total_groups: 0,
            ungrouped: 0,
        }
    }

    /// Add to group
    pub fn add_to_group(&mut self, group_name: &str, key: String, value: String) {
        let group = self.groups.entry(group_name.to_string())
            .or_insert_with(|| SettingsGroup::new(group_name));
        group.add(key, value);
        self.total_entries += 1;
    }

    /// Finalize
    pub fn finalize(&mut self) {
        self.total_groups = self.groups.len();
    }

    /// Get group names
    pub fn group_names(&self) -> Vec<&str> {
        self.groups.keys().map(|s| s.as_str()).collect()
    }

    /// Get group
    pub fn get_group(&self, name: &str) -> Option<&SettingsGroup> {
        self.groups.get(name)
    }
}

impl Default for GroupResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_new() {
        let g = SettingsGroup::new("test");
        assert!(g.is_empty());
    }

    #[test]
    fn test_group_add() {
        let mut g = SettingsGroup::new("test");
        g.add("key".to_string(), "value".to_string());
        assert_eq!(g.count, 1);
        assert!(!g.is_empty());
    }

    #[test]
    fn test_result_new() {
        let r = GroupResult::new();
        assert_eq!(r.total_groups, 0);
    }

    #[test]
    fn test_result_add_to_group() {
        let mut r = GroupResult::new();
        r.add_to_group("g1", "k".to_string(), "v".to_string());
        r.finalize();
        assert_eq!(r.total_entries, 1);
        assert_eq!(r.total_groups, 1);
    }
}
