// v0.0.565: Settings Profiles (Phase 141)
// Profile manager for handling multiple profiles

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::settings_persistence::{SettingsError, SettingsResult};
use crate::unified_settings::UnifiedSettings;

use super::types::{ProfileId, SettingsProfile};

/// Profile manager for handling multiple profiles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileManager {
    /// All profiles
    profiles: HashMap<ProfileId, SettingsProfile>,
    /// Active profile ID
    active_id: Option<ProfileId>,
    /// Default profile ID
    pub(super) default_id: Option<ProfileId>,
}

impl ProfileManager {
    /// Create new profile manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default profile
    pub fn with_default(settings: &UnifiedSettings) -> Self {
        let mut manager = Self::new();
        let profile = SettingsProfile::new(
            "default",
            "Default",
            "Default settings profile",
            settings.clone(),
        );
        manager.add(profile);
        manager.default_id = Some("default".to_string());
        manager.active_id = Some("default".to_string());
        manager
    }

    /// Add a profile
    pub fn add(&mut self, mut profile: SettingsProfile) {
        if self.profiles.is_empty() {
            profile.meta.is_active = true;
            self.active_id = Some(profile.id.clone());
        }
        self.profiles.insert(profile.id.clone(), profile);
    }

    /// Remove a profile
    pub fn remove(&mut self, id: &str) -> Option<SettingsProfile> {
        if Some(id.to_string()) == self.active_id {
            return None; // Can't remove active profile
        }
        self.profiles.remove(id)
    }

    /// Get a profile
    pub fn get(&self, id: &str) -> Option<&SettingsProfile> {
        self.profiles.get(id)
    }

    /// Get a profile mutably
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsProfile> {
        self.profiles.get_mut(id)
    }

    /// List all profiles
    pub fn list(&self) -> Vec<&SettingsProfile> {
        self.profiles.values().collect()
    }

    /// Get profile count
    pub fn count(&self) -> usize {
        self.profiles.len()
    }

    /// Get active profile
    pub fn active(&self) -> Option<&SettingsProfile> {
        self.active_id.as_ref().and_then(|id| self.profiles.get(id))
    }

    /// Get active profile ID
    pub fn active_id(&self) -> Option<&str> {
        self.active_id.as_deref()
    }

    /// Switch to a profile
    pub fn switch_to(&mut self, id: &str) -> SettingsResult<&SettingsProfile> {
        if !self.profiles.contains_key(id) {
            return Err(SettingsError::PathUnavailable);
        }

        // Deactivate current
        if let Some(current_id) = &self.active_id {
            if let Some(current) = self.profiles.get_mut(current_id) {
                current.meta.is_active = false;
            }
        }

        // Activate new
        if let Some(profile) = self.profiles.get_mut(id) {
            profile.meta.is_active = true;
        }
        self.active_id = Some(id.to_string());

        Ok(self.profiles.get(id).unwrap())
    }

    /// Get default profile
    pub fn default_profile(&self) -> Option<&SettingsProfile> {
        self.default_id.as_ref().and_then(|id| self.profiles.get(id))
    }

    /// Set default profile
    pub fn set_default(&mut self, id: &str) -> SettingsResult<()> {
        if !self.profiles.contains_key(id) {
            return Err(SettingsError::PathUnavailable);
        }
        self.default_id = Some(id.to_string());
        Ok(())
    }

    /// Find profiles by name (partial match)
    pub fn find_by_name(&self, name: &str) -> Vec<&SettingsProfile> {
        let name_lower = name.to_lowercase();
        self.profiles
            .values()
            .filter(|p| p.meta.name.to_lowercase().contains(&name_lower))
            .collect()
    }

    /// Find profiles by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<&SettingsProfile> {
        let tag_lower = tag.to_lowercase();
        self.profiles
            .values()
            .filter(|p| {
                p.meta.tags.iter().any(|t| t.to_lowercase() == tag_lower)
            })
            .collect()
    }

    /// Duplicate a profile
    pub fn duplicate(&mut self, id: &str, new_id: &str, new_name: &str) -> SettingsResult<()> {
        let source = self.profiles.get(id)
            .ok_or(SettingsError::PathUnavailable)?
            .clone();

        let mut new_profile = SettingsProfile::new(
            new_id,
            new_name,
            format!("Copy of {}", source.meta.name),
            source.settings,
        );
        new_profile.meta.tags = source.meta.tags;

        self.add(new_profile);
        Ok(())
    }

    /// Rename a profile
    pub fn rename(&mut self, id: &str, new_name: &str) -> SettingsResult<()> {
        let profile = self.profiles.get_mut(id)
            .ok_or(SettingsError::PathUnavailable)?;
        profile.meta.name = new_name.to_string();
        profile.meta.modified = chrono::Utc::now();
        Ok(())
    }

    /// Update profile description
    pub fn set_description(&mut self, id: &str, description: &str) -> SettingsResult<()> {
        let profile = self.profiles.get_mut(id)
            .ok_or(SettingsError::PathUnavailable)?;
        profile.meta.description = description.to_string();
        profile.meta.modified = chrono::Utc::now();
        Ok(())
    }
}
