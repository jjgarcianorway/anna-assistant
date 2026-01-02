// v0.0.565: Settings Profiles (Phase 141)
// Named settings configurations that users can switch between
// Type definitions

use serde::{Deserialize, Serialize};

use crate::unified_settings::UnifiedSettings;

/// Profile identifier
pub type ProfileId = String;

/// Profile metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    /// Profile name
    pub name: String,
    /// Profile description
    pub description: String,
    /// Created timestamp
    pub created: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified: chrono::DateTime<chrono::Utc>,
    /// Is this the active profile?
    pub is_active: bool,
    /// Profile tags
    pub tags: Vec<String>,
}

impl ProfileMeta {
    /// Create new profile metadata
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            description: description.into(),
            created: now,
            modified: now,
            is_active: false,
            tags: Vec::new(),
        }
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// A named settings profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsProfile {
    /// Profile ID
    pub id: ProfileId,
    /// Profile metadata
    pub meta: ProfileMeta,
    /// Profile settings
    pub settings: UnifiedSettings,
}

impl SettingsProfile {
    /// Create new profile
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        settings: UnifiedSettings,
    ) -> Self {
        Self {
            id: id.into(),
            meta: ProfileMeta::new(name, description),
            settings,
        }
    }

    /// Create profile from current settings
    pub fn from_current(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        settings: &UnifiedSettings,
    ) -> Self {
        Self::new(id, name, description, settings.clone())
    }

    /// Update settings and modified time
    pub fn update_settings(&mut self, settings: UnifiedSettings) {
        self.settings = settings;
        self.meta.modified = chrono::Utc::now();
    }
}
