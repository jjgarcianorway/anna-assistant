// v0.0.562: Settings Presets (Phase 138)
// Provides pre-configured settings profiles for different use cases

use serde::{Deserialize, Serialize};

use crate::unified_settings::UnifiedSettings;

/// Preset category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresetCategory {
    /// User experience level
    Experience,
    /// Security posture
    Security,
    /// Performance optimization
    Performance,
    /// Privacy focus
    Privacy,
    /// Use case specific
    UseCase,
}

impl std::fmt::Display for PresetCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Experience => write!(f, "Experience"),
            Self::Security => write!(f, "Security"),
            Self::Performance => write!(f, "Performance"),
            Self::Privacy => write!(f, "Privacy"),
            Self::UseCase => write!(f, "Use Case"),
        }
    }
}

/// A settings preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPreset {
    /// Preset identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: PresetCategory,
    /// The preset settings
    pub settings: UnifiedSettings,
    /// Is this a built-in preset?
    pub builtin: bool,
}

impl SettingsPreset {
    /// Create a new preset
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: PresetCategory,
        settings: UnifiedSettings,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category,
            settings,
            builtin: false,
        }
    }

    /// Mark as built-in
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }
}

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
        self.presets.push(Self::beginner_preset());
        self.presets.push(Self::intermediate_preset());
        self.presets.push(Self::expert_preset());

        // Security presets
        self.presets.push(Self::paranoid_preset());
        self.presets.push(Self::balanced_security_preset());

        // Performance presets
        self.presets.push(Self::speed_preset());
        self.presets.push(Self::quality_preset());

        // Privacy presets
        self.presets.push(Self::maximum_privacy_preset());
        self.presets.push(Self::convenience_preset());

        // Use case presets
        self.presets.push(Self::server_admin_preset());
        self.presets.push(Self::developer_preset());
        self.presets.push(Self::desktop_user_preset());
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

    // Built-in preset definitions

    fn beginner_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.learning.enable();
        settings.verbosity.level = crate::verbosity_config::VerbosityLevel::Verbose;
        settings.confirmation.style = crate::confirmation_behavior_config::ConfirmationStyle::Dialog;

        SettingsPreset::new(
            "beginner",
            "Beginner",
            "Detailed explanations, dialog confirmations, learning mode enabled",
            PresetCategory::Experience,
            settings,
        )
        .builtin()
    }

    fn intermediate_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.verbosity.level = crate::verbosity_config::VerbosityLevel::Normal;
        settings.confirmation.style = crate::confirmation_behavior_config::ConfirmationStyle::Inline;

        SettingsPreset::new(
            "intermediate",
            "Intermediate",
            "Balanced verbosity and confirmations for experienced users",
            PresetCategory::Experience,
            settings,
        )
        .builtin()
    }

    fn expert_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.verbosity.level = crate::verbosity_config::VerbosityLevel::Minimal;
        settings.confirmation.style = crate::confirmation_behavior_config::ConfirmationStyle::Silent;
        settings.learning.disable();

        SettingsPreset::new(
            "expert",
            "Expert",
            "Minimal verbosity, silent confirmations for power users",
            PresetCategory::Experience,
            settings,
        )
        .builtin()
    }

    fn paranoid_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.confirmation.style = crate::confirmation_behavior_config::ConfirmationStyle::Dialog;
        settings.risk.require_root_confirmation = true;
        settings.risk.require_delete_confirmation = true;
        settings.privacy.data_collection = crate::privacy_config::DataCollectionLevel::Minimal;
        settings.backup.encrypt_backups = true;

        SettingsPreset::new(
            "paranoid",
            "Paranoid",
            "Maximum security - confirm everything, encrypt backups, minimal data collection",
            PresetCategory::Security,
            settings,
        )
        .builtin()
    }

    fn balanced_security_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.risk.require_root_confirmation = true;
        settings.privacy.data_collection = crate::privacy_config::DataCollectionLevel::Standard;

        SettingsPreset::new(
            "balanced_security",
            "Balanced Security",
            "Good security without excessive interruptions",
            PresetCategory::Security,
            settings,
        )
        .builtin()
    }

    fn speed_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.model = crate::model_config::ModelConfig::fast();
        settings.timeout = crate::timeout_config::TimeoutConfig::fast();
        settings.verbosity.level = crate::verbosity_config::VerbosityLevel::Minimal;

        SettingsPreset::new(
            "speed",
            "Speed",
            "Optimized for fast responses - smaller models, shorter timeouts",
            PresetCategory::Performance,
            settings,
        )
        .builtin()
    }

    fn quality_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.model = crate::model_config::ModelConfig::quality();
        settings.timeout = crate::timeout_config::TimeoutConfig::patient();

        SettingsPreset::new(
            "quality",
            "Quality",
            "Optimized for best answers - larger models, longer timeouts",
            PresetCategory::Performance,
            settings,
        )
        .builtin()
    }

    fn maximum_privacy_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.privacy = crate::privacy_config::PrivacyConfig::maximum();
        settings.backup.encrypt_backups = true;

        SettingsPreset::new(
            "maximum_privacy",
            "Maximum Privacy",
            "No telemetry, no history, encrypted backups",
            PresetCategory::Privacy,
            settings,
        )
        .builtin()
    }

    fn convenience_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.privacy = crate::privacy_config::PrivacyConfig::convenience();
        settings.confirmation.style = crate::confirmation_behavior_config::ConfirmationStyle::Silent;

        SettingsPreset::new(
            "convenience",
            "Convenience",
            "Easy to use - minimal confirmations, full features enabled",
            PresetCategory::Privacy,
            settings,
        )
        .builtin()
    }

    fn server_admin_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.confirmation.style = crate::confirmation_behavior_config::ConfirmationStyle::Dialog;
        settings.risk.require_root_confirmation = true;
        settings.verbosity.level = crate::verbosity_config::VerbosityLevel::Normal;
        settings.output_style.animation = crate::output_style_config::AnimationStyle::None;

        SettingsPreset::new(
            "server_admin",
            "Server Admin",
            "Conservative settings for production server administration",
            PresetCategory::UseCase,
            settings,
        )
        .builtin()
    }

    fn developer_preset() -> SettingsPreset {
        let mut settings = UnifiedSettings::default();
        settings.verbosity.level = crate::verbosity_config::VerbosityLevel::Verbose;
        settings.learning.enable();
        settings.confirmation.style = crate::confirmation_behavior_config::ConfirmationStyle::Prompt;

        SettingsPreset::new(
            "developer",
            "Developer",
            "Verbose output with learning mode for development work",
            PresetCategory::UseCase,
            settings,
        )
        .builtin()
    }

    fn desktop_user_preset() -> SettingsPreset {
        let settings = UnifiedSettings::default();

        SettingsPreset::new(
            "desktop_user",
            "Desktop User",
            "Balanced defaults for everyday desktop use",
            PresetCategory::UseCase,
            settings,
        )
        .builtin()
    }
}

/// Apply a preset to settings
pub fn apply_preset(preset: &SettingsPreset, _current: &mut UnifiedSettings) -> UnifiedSettings {
    preset.settings.clone()
}

/// Find preset matching natural language
pub fn find_preset_natural(query: &str) -> Option<String> {
    let lower = query.to_lowercase();

    if lower.contains("beginner") || lower.contains("new user") || lower.contains("learning") {
        Some("beginner".to_string())
    } else if lower.contains("expert") || lower.contains("power user") || lower.contains("advanced") {
        Some("expert".to_string())
    } else if lower.contains("paranoid") || lower.contains("maximum security") {
        Some("paranoid".to_string())
    } else if lower.contains("speed") || lower.contains("fast") || lower.contains("quick") {
        Some("speed".to_string())
    } else if lower.contains("quality") || lower.contains("best") || lower.contains("accurate") {
        Some("quality".to_string())
    } else if lower.contains("privacy") || lower.contains("private") {
        Some("maximum_privacy".to_string())
    } else if lower.contains("server") || lower.contains("production") {
        Some("server_admin".to_string())
    } else if lower.contains("developer") || lower.contains("dev") || lower.contains("coding") {
        Some("developer".to_string())
    } else {
        None
    }
}

/// Format preset list for display
pub fn format_preset_list(manager: &PresetManager) -> String {
    let mut output = String::new();
    output.push_str("=== Available Presets ===\n\n");

    for category in [
        PresetCategory::Experience,
        PresetCategory::Security,
        PresetCategory::Performance,
        PresetCategory::Privacy,
        PresetCategory::UseCase,
    ] {
        let presets = manager.by_category(category);
        if !presets.is_empty() {
            output.push_str(&format!("{}:\n", category));
            for preset in presets {
                output.push_str(&format!("  {} - {}\n", preset.name, preset.description));
            }
            output.push('\n');
        }
    }

    output
}

/// Fun fact about settings presets
pub fn settings_presets_fun_fact() -> &'static str {
    "Anna has 12 built-in presets - from 'Beginner' to 'Paranoid' to 'Server Admin'!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_category_display() {
        assert_eq!(format!("{}", PresetCategory::Experience), "Experience");
        assert_eq!(format!("{}", PresetCategory::Security), "Security");
    }

    #[test]
    fn test_settings_preset_new() {
        let preset = SettingsPreset::new(
            "test",
            "Test Preset",
            "A test preset",
            PresetCategory::UseCase,
            UnifiedSettings::default(),
        );
        assert_eq!(preset.id, "test");
        assert!(!preset.builtin);
    }

    #[test]
    fn test_settings_preset_builtin() {
        let preset = SettingsPreset::new(
            "test",
            "Test",
            "Test",
            PresetCategory::UseCase,
            UnifiedSettings::default(),
        )
        .builtin();
        assert!(preset.builtin);
    }

    #[test]
    fn test_preset_manager_new() {
        let manager = PresetManager::new();
        assert!(manager.count() >= 12); // At least 12 built-in presets
    }

    #[test]
    fn test_preset_manager_find() {
        let manager = PresetManager::new();
        let preset = manager.find("beginner");
        assert!(preset.is_some());
        assert_eq!(preset.unwrap().name, "Beginner");
    }

    #[test]
    fn test_preset_manager_find_by_name() {
        let manager = PresetManager::new();
        let preset = manager.find_by_name("server");
        assert!(preset.is_some());
    }

    #[test]
    fn test_preset_manager_by_category() {
        let manager = PresetManager::new();
        let experience = manager.by_category(PresetCategory::Experience);
        assert_eq!(experience.len(), 3); // beginner, intermediate, expert
    }

    #[test]
    fn test_preset_manager_add_custom() {
        let mut manager = PresetManager::new();
        let initial = manager.count();

        manager.add(SettingsPreset::new(
            "custom",
            "Custom",
            "Custom preset",
            PresetCategory::UseCase,
            UnifiedSettings::default(),
        ));

        assert_eq!(manager.count(), initial + 1);
    }

    #[test]
    fn test_preset_manager_remove_custom() {
        let mut manager = PresetManager::new();
        manager.add(SettingsPreset::new(
            "custom",
            "Custom",
            "Custom",
            PresetCategory::UseCase,
            UnifiedSettings::default(),
        ));

        assert!(manager.remove("custom"));
        assert!(manager.find("custom").is_none());
    }

    #[test]
    fn test_preset_manager_cannot_remove_builtin() {
        let mut manager = PresetManager::new();
        assert!(!manager.remove("beginner")); // Built-in, can't remove
    }

    #[test]
    fn test_find_preset_natural() {
        assert_eq!(find_preset_natural("I'm a beginner"), Some("beginner".to_string()));
        assert_eq!(find_preset_natural("maximum security"), Some("paranoid".to_string()));
        assert_eq!(find_preset_natural("fast responses"), Some("speed".to_string()));
    }

    #[test]
    fn test_format_preset_list() {
        let manager = PresetManager::new();
        let output = format_preset_list(&manager);
        assert!(output.contains("Beginner"));
        assert!(output.contains("Experience"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_presets_fun_fact();
        assert!(fact.contains("12"));
    }
}
