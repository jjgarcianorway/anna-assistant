// v0.0.554: Unified Settings Manager (Phase 130)
// Aggregates all configuration modules per VISION.md

use serde::{Deserialize, Serialize};

use crate::backup_config::BackupConfig;
use crate::confirmation_behavior_config::ConfirmationBehaviorConfig;
use crate::escalation_policy_config::EscalationPolicyConfig;
use crate::learning_mode_config::LearningModeConfig;
use crate::model_config::ModelConfig;
use crate::output_style_config::OutputStyleConfig;
use crate::personality_config::PersonalityConfig;
use crate::privacy_config::PrivacyConfig;
use crate::risk_level_config::RiskLevelConfig;
use crate::timeout_config::TimeoutConfig;
use crate::update_config::UpdateConfig;
use crate::verbosity_config::VerbosityConfig;

/// Settings category for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsCategory {
    Personality,
    Risk,
    Learning,
    Escalation,
    Verbosity,
    Confirmation,
    Timeout,
    OutputStyle,
    Privacy,
    Backup,
    Update,
    Model,
    Unknown,
}

impl std::fmt::Display for SettingsCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Personality => write!(f, "Personality"),
            Self::Risk => write!(f, "Risk"),
            Self::Learning => write!(f, "Learning"),
            Self::Escalation => write!(f, "Escalation"),
            Self::Verbosity => write!(f, "Verbosity"),
            Self::Confirmation => write!(f, "Confirmation"),
            Self::Timeout => write!(f, "Timeout"),
            Self::OutputStyle => write!(f, "Output Style"),
            Self::Privacy => write!(f, "Privacy"),
            Self::Backup => write!(f, "Backup"),
            Self::Update => write!(f, "Update"),
            Self::Model => write!(f, "Model"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Unified settings containing all configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSettings {
    pub personality: PersonalityConfig,
    pub risk: RiskLevelConfig,
    pub learning: LearningModeConfig,
    pub escalation: EscalationPolicyConfig,
    pub verbosity: VerbosityConfig,
    pub confirmation: ConfirmationBehaviorConfig,
    pub timeout: TimeoutConfig,
    pub output_style: OutputStyleConfig,
    pub privacy: PrivacyConfig,
    pub backup: BackupConfig,
    pub update: UpdateConfig,
    pub model: ModelConfig,
}

impl Default for UnifiedSettings {
    fn default() -> Self {
        Self {
            personality: PersonalityConfig::default(),
            risk: RiskLevelConfig::default(),
            learning: LearningModeConfig::default(),
            escalation: EscalationPolicyConfig::default(),
            verbosity: VerbosityConfig::default(),
            confirmation: ConfirmationBehaviorConfig::default(),
            timeout: TimeoutConfig::default(),
            output_style: OutputStyleConfig::default(),
            privacy: PrivacyConfig::default(),
            backup: BackupConfig::default(),
            update: UpdateConfig::default(),
            model: ModelConfig::default(),
        }
    }
}

impl UnifiedSettings {
    /// Create new settings with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Detect which category a request belongs to
    pub fn categorize_request(request: &str) -> SettingsCategory {
        let lower = request.to_lowercase();

        if lower.contains("personality") || lower.contains("formal") || lower.contains("friendly") {
            SettingsCategory::Personality
        } else if lower.contains("risk") || lower.contains("danger") || lower.contains("safe") {
            SettingsCategory::Risk
        } else if lower.contains("learn") || lower.contains("teach") || lower.contains("explain") {
            SettingsCategory::Learning
        } else if lower.contains("escalat") || lower.contains("senior") {
            SettingsCategory::Escalation
        } else if lower.contains("verbose") || lower.contains("detail") || lower.contains("brief") {
            SettingsCategory::Verbosity
        } else if lower.contains("confirm") || lower.contains("ask before") {
            SettingsCategory::Confirmation
        } else if lower.contains("timeout") || lower.contains("time limit") {
            SettingsCategory::Timeout
        } else if lower.contains("style") || lower.contains("theme") || lower.contains("color") {
            SettingsCategory::OutputStyle
        } else if lower.contains("privacy") || lower.contains("telemetry") || lower.contains("data") {
            SettingsCategory::Privacy
        } else if lower.contains("backup") || lower.contains("restore") {
            SettingsCategory::Backup
        } else if lower.contains("update") || lower.contains("version") {
            SettingsCategory::Update
        } else if lower.contains("model") || lower.contains("llm") || lower.contains("gpu") {
            SettingsCategory::Model
        } else {
            SettingsCategory::Unknown
        }
    }

    /// Apply a natural language change, routing to appropriate config
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let category = Self::categorize_request(request);

        match category {
            SettingsCategory::Personality => self.personality.apply_change(request),
            SettingsCategory::Risk => self.risk.apply_change(request),
            SettingsCategory::Learning => self.learning.apply_change(request),
            SettingsCategory::Escalation => self.escalation.apply_change(request),
            SettingsCategory::Verbosity => self.verbosity.apply_change(request),
            SettingsCategory::Confirmation => self.confirmation.apply_change(request),
            SettingsCategory::Timeout => self.timeout.apply_change(request),
            SettingsCategory::OutputStyle => self.output_style.apply_change(request),
            SettingsCategory::Privacy => self.privacy.apply_change(request),
            SettingsCategory::Backup => self.backup.apply_change(request),
            SettingsCategory::Update => self.update.apply_change(request),
            SettingsCategory::Model => self.model.apply_change(request),
            SettingsCategory::Unknown => None,
        }
    }

    /// Get all categories
    pub fn categories() -> Vec<SettingsCategory> {
        vec![
            SettingsCategory::Personality,
            SettingsCategory::Risk,
            SettingsCategory::Learning,
            SettingsCategory::Escalation,
            SettingsCategory::Verbosity,
            SettingsCategory::Confirmation,
            SettingsCategory::Timeout,
            SettingsCategory::OutputStyle,
            SettingsCategory::Privacy,
            SettingsCategory::Backup,
            SettingsCategory::Update,
            SettingsCategory::Model,
        ]
    }

    /// Reset all settings to defaults
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    /// Reset specific category to defaults
    pub fn reset_category(&mut self, category: SettingsCategory) {
        match category {
            SettingsCategory::Personality => self.personality = PersonalityConfig::default(),
            SettingsCategory::Risk => self.risk = RiskLevelConfig::default(),
            SettingsCategory::Learning => self.learning = LearningModeConfig::default(),
            SettingsCategory::Escalation => self.escalation = EscalationPolicyConfig::default(),
            SettingsCategory::Verbosity => self.verbosity = VerbosityConfig::default(),
            SettingsCategory::Confirmation => self.confirmation = ConfirmationBehaviorConfig::default(),
            SettingsCategory::Timeout => self.timeout = TimeoutConfig::default(),
            SettingsCategory::OutputStyle => self.output_style = OutputStyleConfig::default(),
            SettingsCategory::Privacy => self.privacy = PrivacyConfig::default(),
            SettingsCategory::Backup => self.backup = BackupConfig::default(),
            SettingsCategory::Update => self.update = UpdateConfig::default(),
            SettingsCategory::Model => self.model = ModelConfig::default(),
            SettingsCategory::Unknown => {}
        }
    }
}

/// Format unified settings summary
pub fn format_settings_summary(settings: &UnifiedSettings) -> String {
    let mut output = String::new();
    output.push_str("=== Anna Settings Overview ===\n\n");

    output.push_str(&format!("Personality: {}\n", settings.personality.formality));
    output.push_str(&format!("Risk Level: {}\n", settings.risk.confirmation_mode));
    output.push_str(&format!("Learning Mode: {}\n", settings.learning.level));
    output.push_str(&format!("Escalation: {}\n", settings.escalation.mode));
    output.push_str(&format!("Verbosity: {}\n", settings.verbosity.level));
    output.push_str(&format!("Confirmation: {}\n", settings.confirmation.style));
    output.push_str(&format!("Timeout: {}\n", settings.timeout.profile));
    output.push_str(&format!("Output Style: {}\n", settings.output_style.theme));
    output.push_str(&format!("Privacy: {}\n", settings.privacy.data_collection));
    output.push_str(&format!("Backup: {}\n", settings.backup.frequency));
    output.push_str(&format!("Update: {}\n", settings.update.check_frequency));
    output.push_str(&format!("Model: {}\n", settings.model.size_preference));

    output
}

/// Check if query is settings-related
pub fn is_settings_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("setting")
        || lower.contains("config")
        || lower.contains("preference")
        || lower.contains("option")
}

/// Fun fact about settings
pub fn settings_fun_fact() -> &'static str {
    "Anna has 12 configurable categories with hundreds of options - all controllable through natural language!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", SettingsCategory::Personality), "Personality");
        assert_eq!(format!("{}", SettingsCategory::Model), "Model");
    }

    #[test]
    fn test_default_settings() {
        let settings = UnifiedSettings::default();
        assert_eq!(settings.personality.formality, crate::personality_config::FormalityLevel::default());
    }

    #[test]
    fn test_categorize_personality() {
        let cat = UnifiedSettings::categorize_request("be more formal");
        assert_eq!(cat, SettingsCategory::Personality);
    }

    #[test]
    fn test_categorize_model() {
        let cat = UnifiedSettings::categorize_request("use faster GPU models");
        assert_eq!(cat, SettingsCategory::Model);
    }

    #[test]
    fn test_categorize_privacy() {
        let cat = UnifiedSettings::categorize_request("disable telemetry");
        assert_eq!(cat, SettingsCategory::Privacy);
    }

    #[test]
    fn test_categorize_unknown() {
        let cat = UnifiedSettings::categorize_request("install vim");
        assert_eq!(cat, SettingsCategory::Unknown);
    }

    #[test]
    fn test_apply_change_routing() {
        let mut settings = UnifiedSettings::new();
        let result = settings.apply_change("enable learning mode");
        assert!(result.is_some());
        assert!(settings.learning.is_enabled());
    }

    #[test]
    fn test_categories_count() {
        let cats = UnifiedSettings::categories();
        assert_eq!(cats.len(), 12);
    }

    #[test]
    fn test_reset_category() {
        let mut settings = UnifiedSettings::new();
        settings.learning.enable();
        assert!(settings.learning.is_enabled());
        settings.reset_category(SettingsCategory::Learning);
        assert!(!settings.learning.is_enabled());
    }

    #[test]
    fn test_is_settings_query() {
        assert!(is_settings_query("Show settings"));
        assert!(is_settings_query("Configure options"));
        assert!(!is_settings_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_fun_fact();
        assert!(fact.contains("12"));
    }
}
