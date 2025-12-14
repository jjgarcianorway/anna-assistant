// v0.0.592: Settings Snapshot (Phase 168)
// Point-in-time settings snapshots

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Snapshot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotType {
    /// Manual snapshot
    Manual,
    /// Auto snapshot
    Auto,
    /// Pre-change snapshot
    PreChange,
    /// Scheduled snapshot
    Scheduled,
}

impl Default for SnapshotType {
    fn default() -> Self {
        Self::Manual
    }
}

impl std::fmt::Display for SnapshotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::Auto => write!(f, "auto"),
            Self::PreChange => write!(f, "pre_change"),
            Self::Scheduled => write!(f, "scheduled"),
        }
    }
}

/// Snapshot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SnapshotStatus {
    /// Active/valid snapshot
    #[default]
    Active,
    /// Archived
    Archived,
    /// Expired
    Expired,
    /// Corrupted
    Corrupted,
}

impl std::fmt::Display for SnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
            Self::Expired => write!(f, "expired"),
            Self::Corrupted => write!(f, "corrupted"),
        }
    }
}

/// Settings snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSnapshot {
    /// Snapshot ID
    pub id: String,
    /// Name
    pub name: String,
    /// Type
    pub snapshot_type: SnapshotType,
    /// Status
    pub status: SnapshotStatus,
    /// Created time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expires at (optional)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Description
    pub description: Option<String>,
    /// Settings data (serialized by category)
    pub data: HashMap<SettingsCategory, String>,
    /// Hash for integrity
    pub hash: String,
    /// Size in bytes
    pub size: usize,
}

impl SettingsSnapshot {
    /// Create new snapshot
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            snapshot_type: SnapshotType::Manual,
            status: SnapshotStatus::Active,
            created_at: chrono::Utc::now(),
            expires_at: None,
            description: None,
            data: HashMap::new(),
            hash: String::new(),
            size: 0,
        }
    }

    /// Set type
    pub fn snapshot_type(mut self, t: SnapshotType) -> Self {
        self.snapshot_type = t;
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set expiration
    pub fn expires_in(mut self, seconds: i64) -> Self {
        self.expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(seconds));
        self
    }

    /// Add category data
    pub fn add_data(&mut self, category: SettingsCategory, data: impl Into<String>) {
        let d = data.into();
        self.size += d.len();
        self.data.insert(category, d);
    }

    /// Get category data
    pub fn get_data(&self, category: SettingsCategory) -> Option<&str> {
        self.data.get(&category).map(|s| s.as_str())
    }

    /// Category count
    pub fn category_count(&self) -> usize {
        self.data.len()
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }

    /// Archive snapshot
    pub fn archive(&mut self) {
        self.status = SnapshotStatus::Archived;
    }

    /// Finalize (calculate hash)
    pub fn finalize(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.size.hash(&mut hasher);
        self.hash = format!("{:x}", hasher.finish());
    }
}

/// Snapshot manager
#[derive(Debug, Clone, Default)]
pub struct SnapshotManager {
    /// Snapshots
    snapshots: Vec<SettingsSnapshot>,
    /// Max snapshots to keep
    max_snapshots: usize,
    /// Auto-expire enabled
    auto_expire: bool,
}

impl SnapshotManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            max_snapshots: 50,
            auto_expire: true,
            ..Default::default()
        }
    }

    /// Create snapshot
    pub fn create(&mut self, name: impl Into<String>) -> &mut SettingsSnapshot {
        let snapshot = SettingsSnapshot::new(name);
        self.snapshots.push(snapshot);

        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }

        self.snapshots.last_mut().unwrap()
    }

    /// Get snapshot by ID
    pub fn get(&self, id: &str) -> Option<&SettingsSnapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    /// Get mutable snapshot
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSnapshot> {
        self.snapshots.iter_mut().find(|s| s.id == id)
    }

    /// Delete snapshot
    pub fn delete(&mut self, id: &str) -> bool {
        let len = self.snapshots.len();
        self.snapshots.retain(|s| s.id != id);
        self.snapshots.len() < len
    }

    /// Get latest snapshot
    pub fn latest(&self) -> Option<&SettingsSnapshot> {
        self.snapshots.last()
    }

    /// List all snapshots
    pub fn all(&self) -> &[SettingsSnapshot] {
        &self.snapshots
    }

    /// List by type
    pub fn by_type(&self, t: SnapshotType) -> Vec<&SettingsSnapshot> {
        self.snapshots.iter().filter(|s| s.snapshot_type == t).collect()
    }

    /// List active snapshots
    pub fn active(&self) -> Vec<&SettingsSnapshot> {
        self.snapshots.iter().filter(|s| s.status == SnapshotStatus::Active).collect()
    }

    /// Clean expired snapshots
    pub fn clean_expired(&mut self) -> usize {
        let len = self.snapshots.len();
        self.snapshots.retain(|s| !s.is_expired());
        len - self.snapshots.len()
    }

    /// Snapshot count
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }

    /// Total size
    pub fn total_size(&self) -> usize {
        self.snapshots.iter().map(|s| s.size).sum()
    }

    /// Clear all
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
}

/// Format snapshot
pub fn format_snapshot(snapshot: &SettingsSnapshot) -> String {
    let mut output = String::new();

    output.push_str(&format!("Snapshot: {}\n", snapshot.name));
    output.push_str(&format!("ID: {}\n", &snapshot.id[..8]));
    output.push_str(&format!("Type: {} | Status: {}\n", snapshot.snapshot_type, snapshot.status));
    output.push_str(&format!("Created: {}\n", snapshot.created_at.format("%Y-%m-%d %H:%M")));
    output.push_str(&format!("Categories: {} | Size: {} bytes\n", snapshot.category_count(), snapshot.size));

    output
}

/// Format snapshot manager
pub fn format_snapshots(manager: &SnapshotManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Snapshots ===\n\n");
    output.push_str(&format!(
        "Total: {} | Size: {} bytes\n\n",
        manager.count(),
        manager.total_size()
    ));

    for snapshot in manager.all().iter().rev().take(10) {
        output.push_str(&format!(
            "{} [{}] - {} categories\n",
            snapshot.name,
            snapshot.snapshot_type,
            snapshot.category_count()
        ));
    }

    output
}

/// Check if query is about snapshots
pub fn is_snapshot_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("snapshot")
        || lower.contains("point in time")
        || lower.contains("checkpoint")
}

/// Fun fact about snapshots
pub fn settings_snapshot_fun_fact() -> &'static str {
    "Anna can create point-in-time snapshots of your settings for easy recovery!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_type_display() {
        assert_eq!(format!("{}", SnapshotType::Manual), "manual");
        assert_eq!(format!("{}", SnapshotType::Auto), "auto");
    }

    #[test]
    fn test_snapshot_status_display() {
        assert_eq!(format!("{}", SnapshotStatus::Active), "active");
        assert_eq!(format!("{}", SnapshotStatus::Archived), "archived");
    }

    #[test]
    fn test_snapshot_new() {
        let snapshot = SettingsSnapshot::new("test");
        assert_eq!(snapshot.name, "test");
        assert_eq!(snapshot.status, SnapshotStatus::Active);
    }

    #[test]
    fn test_snapshot_add_data() {
        let mut snapshot = SettingsSnapshot::new("test");
        snapshot.add_data(SettingsCategory::Personality, "data");
        assert_eq!(snapshot.category_count(), 1);
        assert!(snapshot.size > 0);
    }

    #[test]
    fn test_snapshot_archive() {
        let mut snapshot = SettingsSnapshot::new("test");
        snapshot.archive();
        assert_eq!(snapshot.status, SnapshotStatus::Archived);
    }

    #[test]
    fn test_manager_new() {
        let manager = SnapshotManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_manager_create() {
        let mut manager = SnapshotManager::new();
        manager.create("test");
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_manager_get() {
        let mut manager = SnapshotManager::new();
        let snapshot = manager.create("test");
        let id = snapshot.id.clone();
        assert!(manager.get(&id).is_some());
    }

    #[test]
    fn test_manager_delete() {
        let mut manager = SnapshotManager::new();
        let snapshot = manager.create("test");
        let id = snapshot.id.clone();
        assert!(manager.delete(&id));
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_format_snapshot() {
        let snapshot = SettingsSnapshot::new("test");
        let output = format_snapshot(&snapshot);
        assert!(output.contains("Snapshot"));
    }

    #[test]
    fn test_format_snapshots() {
        let manager = SnapshotManager::new();
        let output = format_snapshots(&manager);
        assert!(output.contains("Snapshots"));
    }

    #[test]
    fn test_is_snapshot_query() {
        assert!(is_snapshot_query("create snapshot"));
        assert!(is_snapshot_query("checkpoint settings"));
        assert!(!is_snapshot_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_snapshot_fun_fact();
        assert!(fact.contains("snapshot"));
    }
}
