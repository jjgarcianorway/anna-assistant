// v0.0.565: Settings Profiles (Phase 141)
// Named settings configurations that users can switch between

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::settings_persistence::{SettingsError, SettingsResult};
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

/// Profile manager for handling multiple profiles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileManager {
    /// All profiles
    profiles: HashMap<ProfileId, SettingsProfile>,
    /// Active profile ID
    active_id: Option<ProfileId>,
    /// Default profile ID
    default_id: Option<ProfileId>,
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

/// Format profiles list for display
pub fn format_profiles_list(manager: &ProfileManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Profiles ===\n\n");

    if manager.count() == 0 {
        output.push_str("No profiles configured.\n");
        return output;
    }

    for profile in manager.list() {
        let active = if profile.meta.is_active { " [ACTIVE]" } else { "" };
        let default = if Some(&profile.id) == manager.default_id.as_ref() {
            " (default)"
        } else {
            ""
        };

        output.push_str(&format!(
            "• {}{}{}\n  {}\n  Created: {}\n",
            profile.meta.name,
            active,
            default,
            profile.meta.description,
            profile.meta.created.format("%Y-%m-%d %H:%M")
        ));

        if !profile.meta.tags.is_empty() {
            output.push_str(&format!("  Tags: {}\n", profile.meta.tags.join(", ")));
        }
        output.push('\n');
    }

    output
}

/// Check if query is about profiles
pub fn is_profile_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("profile")
        || lower.contains("configuration")
        || lower.contains("switch settings")
        || lower.contains("create profile")
        || lower.contains("delete profile")
}

/// Fun fact about profiles
pub fn profiles_fun_fact() -> &'static str {
    "You can create multiple named settings profiles for different use cases!"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_settings() -> UnifiedSettings {
        UnifiedSettings::default()
    }

    #[test]
    fn test_profile_meta_new() {
        let meta = ProfileMeta::new("Test", "A test profile");
        assert_eq!(meta.name, "Test");
        assert_eq!(meta.description, "A test profile");
        assert!(!meta.is_active);
    }

    #[test]
    fn test_profile_meta_with_tag() {
        let meta = ProfileMeta::new("Test", "desc")
            .with_tag("work")
            .with_tag("dev");
        assert_eq!(meta.tags.len(), 2);
    }

    #[test]
    fn test_settings_profile_new() {
        let settings = sample_settings();
        let profile = SettingsProfile::new("test", "Test", "desc", settings);
        assert_eq!(profile.id, "test");
        assert_eq!(profile.meta.name, "Test");
    }

    #[test]
    fn test_profile_manager_new() {
        let manager = ProfileManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.active().is_none());
    }

    #[test]
    fn test_profile_manager_with_default() {
        let settings = sample_settings();
        let manager = ProfileManager::with_default(&settings);
        assert_eq!(manager.count(), 1);
        assert!(manager.active().is_some());
    }

    #[test]
    fn test_profile_manager_add() {
        let mut manager = ProfileManager::new();
        let profile = SettingsProfile::new("test", "Test", "desc", sample_settings());
        manager.add(profile);
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_profile_manager_get() {
        let mut manager = ProfileManager::new();
        let profile = SettingsProfile::new("test", "Test", "desc", sample_settings());
        manager.add(profile);
        assert!(manager.get("test").is_some());
        assert!(manager.get("none").is_none());
    }

    #[test]
    fn test_profile_manager_switch() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("p1", "Profile 1", "desc", sample_settings()));
        manager.add(SettingsProfile::new("p2", "Profile 2", "desc", sample_settings()));

        assert!(manager.switch_to("p2").is_ok());
        assert_eq!(manager.active_id(), Some("p2"));
    }

    #[test]
    fn test_profile_manager_remove_active() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("p1", "Profile 1", "desc", sample_settings()));

        // Can't remove active profile
        assert!(manager.remove("p1").is_none());
    }

    #[test]
    fn test_profile_manager_find_by_name() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("work", "Work Profile", "desc", sample_settings()));
        manager.add(SettingsProfile::new("home", "Home Setup", "desc", sample_settings()));

        let found = manager.find_by_name("work");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_profile_manager_duplicate() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("orig", "Original", "desc", sample_settings()));

        assert!(manager.duplicate("orig", "copy", "Copy").is_ok());
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_format_profiles_list() {
        let manager = ProfileManager::with_default(&sample_settings());
        let output = format_profiles_list(&manager);
        assert!(output.contains("Settings Profiles"));
        assert!(output.contains("Default"));
    }

    #[test]
    fn test_is_profile_query() {
        assert!(is_profile_query("show my profiles"));
        assert!(is_profile_query("create profile"));
        assert!(is_profile_query("switch settings"));
        assert!(!is_profile_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = profiles_fun_fact();
        assert!(fact.contains("profile"));
    }
}
