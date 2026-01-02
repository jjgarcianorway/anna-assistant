// v0.0.592: Snapshot Manager (Phase 168)
// Manager for settings snapshots

use super::snapshot::SettingsSnapshot;
use super::types::{SnapshotStatus, SnapshotType};

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
