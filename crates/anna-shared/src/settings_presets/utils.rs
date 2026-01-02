// v0.0.562: Preset Utilities (Phase 138)
// Helper functions for working with presets

use crate::unified_settings::UnifiedSettings;
use super::types::{PresetCategory, SettingsPreset};
use super::manager::PresetManager;

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
