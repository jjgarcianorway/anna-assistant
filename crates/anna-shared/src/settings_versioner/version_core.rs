// v0.0.660: Settings Versioner - Core Types
// Core version types (SettingsVersion, VersionResult, VersionerStats)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::version_types::BumpType;

/// Settings version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsVersion {
    /// Version string
    pub version: String,
    /// Major component
    pub major: u32,
    /// Minor component
    pub minor: u32,
    /// Patch component
    pub patch: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Description
    pub description: Option<String>,
}

impl SettingsVersion {
    /// Create new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            version: format!("{}.{}.{}", major, minor, patch),
            major,
            minor,
            patch,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            description: None,
        }
    }

    /// From string
    pub fn from_string(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 3 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            let patch = parts[2].parse().ok()?;
            Some(Self::new(major, minor, patch))
        } else {
            None
        }
    }

    /// With description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Bump version
    pub fn bump(&self, bump_type: BumpType) -> Self {
        let (major, minor, patch) = match bump_type {
            BumpType::Major => (self.major + 1, 0, 0),
            BumpType::Minor => (self.major, self.minor + 1, 0),
            BumpType::Patch => (self.major, self.minor, self.patch + 1),
            BumpType::Auto => (self.major, self.minor, self.patch + 1),
        };
        Self::new(major, minor, patch)
    }
}

impl Default for SettingsVersion {
    fn default() -> Self {
        Self::new(0, 0, 1)
    }
}

/// Version result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResult {
    /// Previous version
    pub previous: Option<SettingsVersion>,
    /// New version
    pub current: SettingsVersion,
    /// Bump type applied
    pub bump_type: BumpType,
    /// Changes count
    pub changes_count: usize,
}

impl VersionResult {
    /// Create new result
    pub fn new(current: SettingsVersion, bump_type: BumpType) -> Self {
        Self {
            previous: None,
            current,
            bump_type,
            changes_count: 0,
        }
    }

    /// With previous
    pub fn with_previous(mut self, prev: SettingsVersion) -> Self {
        self.previous = Some(prev);
        self
    }

    /// With changes count
    pub fn with_changes(mut self, count: usize) -> Self {
        self.changes_count = count;
        self
    }

    /// Was bumped
    pub fn was_bumped(&self) -> bool {
        self.previous.is_some()
    }
}

/// Versioner stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionerStats {
    /// Total versions created
    pub total_versions: usize,
    /// Total bumps
    pub total_bumps: usize,
    /// By bump type
    pub by_bump_type: HashMap<String, usize>,
    /// Current version
    pub current_version: Option<String>,
}

impl VersionerStats {
    /// Record version
    pub fn record(&mut self, bump_type: BumpType, version: &str) {
        self.total_versions += 1;
        self.total_bumps += 1;
        self.current_version = Some(version.to_string());
        *self.by_bump_type.entry(bump_type.to_string()).or_insert(0) += 1;
    }
}
