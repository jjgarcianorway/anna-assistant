// v0.0.562: Settings Presets Module (Phase 138)
// Provides pre-configured settings profiles for different use cases

mod types;
mod presets;
mod manager;
mod utils;

// Re-export public API
pub use types::{PresetCategory, SettingsPreset};
pub use manager::PresetManager;
pub use utils::{apply_preset, find_preset_natural, format_preset_list, settings_presets_fun_fact};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_settings::UnifiedSettings;

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
