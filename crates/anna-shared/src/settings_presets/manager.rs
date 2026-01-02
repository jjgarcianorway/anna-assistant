// v0.0.562: Preset Manager (Phase 138)
// Manages collection of settings presets

use super::types::{PresetCategory, SettingsPreset};
use super::presets::*;

/// Preset manager
#[derive(Debug, Clone, Default)]
pub struct PresetManager {
    /// Available presets
    presets: Vec<SettingsPreset>,
}

impl PresetManager {
    /// Create new preset manager with built-in presets
    pub fn new() -> Self {
        let mut manager = Self::default();
        manager.load_builtins();
        manager
    }

    /// Load built-in presets
    fn load_builtins(&mut self) {
        // Experience presets
        self.presets.push(beginner_preset());
        self.presets.push(intermediate_preset());
        self.presets.push(expert_preset());

        // Security presets
        self.presets.push(paranoid_preset());
        self.presets.push(balanced_security_preset());

        // Performance presets
        self.presets.push(speed_preset());
        self.presets.push(quality_preset());

        // Privacy presets
        self.presets.push(maximum_privacy_preset());
        self.presets.push(convenience_preset());

        // Use case presets
        self.presets.push(server_admin_preset());
        self.presets.push(developer_preset());
        self.presets.push(desktop_user_preset());
    }

    /// Get all presets
    pub fn all(&self) -> &[SettingsPreset] {
        &self.presets
    }

    /// Get presets by category
    pub fn by_category(&self, category: PresetCategory) -> Vec<&SettingsPreset> {
        self.presets
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Find preset by ID
    pub fn find(&self, id: &str) -> Option<&SettingsPreset> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// Find preset by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&SettingsPreset> {
        let lower = name.to_lowercase();
        self.presets
            .iter()
            .find(|p| p.name.to_lowercase().contains(&lower))
    }

    /// Add a custom preset
    pub fn add(&mut self, preset: SettingsPreset) {
        self.presets.push(preset);
    }

    /// Remove a custom preset (can't remove builtins)
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.presets.iter().position(|p| p.id == id && !p.builtin) {
            self.presets.remove(pos);
            true
        } else {
            false
        }
    }

    /// Preset count
    pub fn count(&self) -> usize {
        self.presets.len()
    }
}
