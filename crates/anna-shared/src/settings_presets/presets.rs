// v0.0.562: Built-in Settings Presets (Phase 138)
// Definitions of all built-in settings presets

use crate::unified_settings::UnifiedSettings;
use super::types::{PresetCategory, SettingsPreset};

/// Experience presets

pub fn beginner_preset() -> SettingsPreset {
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

pub fn intermediate_preset() -> SettingsPreset {
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

pub fn expert_preset() -> SettingsPreset {
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

/// Security presets

pub fn paranoid_preset() -> SettingsPreset {
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

pub fn balanced_security_preset() -> SettingsPreset {
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

/// Performance presets

pub fn speed_preset() -> SettingsPreset {
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

pub fn quality_preset() -> SettingsPreset {
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

/// Privacy presets

pub fn maximum_privacy_preset() -> SettingsPreset {
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

pub fn convenience_preset() -> SettingsPreset {
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

/// Use case presets

pub fn server_admin_preset() -> SettingsPreset {
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

pub fn developer_preset() -> SettingsPreset {
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

pub fn desktop_user_preset() -> SettingsPreset {
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
