// v0.0.592: Settings Snapshot (Phase 168)
// Point-in-time settings snapshot structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;
use super::types::{SnapshotStatus, SnapshotType};

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
