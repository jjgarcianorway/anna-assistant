//! Unified Settings Display.
//!
//! Provides a single view of all configurable settings:
//! - Preferences (learning mode, verbosity, personality)
//! - Notifications (email, desktop, wall, quiet hours)
//! - Debug (level, log file, redaction)
//! - Risk tolerance (auto-confirm, warnings, protection)
//!
//! Per VISION.md: "All settings changeable through annactl in natural language"

use crate::debug_config::format_debug_settings;
use crate::debug_mode::DebugConfig;
use crate::notification_config::format_notification_settings;
use crate::preference_config::format_preferences;
use crate::risk_config::{format_risk_settings, RiskTolerance};
use crate::user_profile::UserProfile;

/// All user-configurable settings
#[derive(Debug, Clone)]
pub struct AllSettings {
    /// User preferences
    pub preferences: UserPreferencesSnapshot,
    /// Notification settings
    pub notifications: NotificationSnapshot,
    /// Debug settings
    pub debug: DebugSnapshot,
    /// Risk tolerance
    pub risk: RiskTolerance,
}

/// Snapshot of user preferences
#[derive(Debug, Clone)]
pub struct UserPreferencesSnapshot {
    pub learning_mode: bool,
    pub verbosity: u8,
    pub auto_confirm_low_risk: bool,
    pub show_internal_comms: bool,
    pub formality: u8,
    pub humor: u8,
    pub technical_depth: u8,
}

/// Snapshot of notification settings
#[derive(Debug, Clone)]
pub struct NotificationSnapshot {
    pub email: Option<String>,
    pub desktop_enabled: bool,
    pub wall_enabled: bool,
    pub quiet_start: Option<String>,
    pub quiet_end: Option<String>,
}

/// Snapshot of debug settings
#[derive(Debug, Clone)]
pub struct DebugSnapshot {
    pub level: String,
    pub log_to_file: bool,
    pub redact_ips: bool,
    pub redact_emails: bool,
    pub redact_secrets: bool,
}

impl AllSettings {
    /// Create from user profile and configs
    pub fn from_profile(
        profile: &UserProfile,
        debug_config: &DebugConfig,
        risk: &RiskTolerance,
    ) -> Self {
        Self {
            preferences: UserPreferencesSnapshot {
                learning_mode: profile.preferences.learning_mode,
                verbosity: profile.preferences.verbosity,
                auto_confirm_low_risk: profile.preferences.auto_confirm_low_risk,
                show_internal_comms: profile.preferences.show_internal_comms,
                formality: profile.preferences.personality.formality,
                humor: profile.preferences.personality.humor,
                technical_depth: profile.preferences.personality.technical_depth,
            },
            notifications: NotificationSnapshot {
                email: profile.email.clone(),
                desktop_enabled: true, // Default
                wall_enabled: false,   // Default
                quiet_start: None,
                quiet_end: None,
            },
            debug: DebugSnapshot {
                level: debug_config.level.name().to_string(),
                log_to_file: debug_config.log_to_file,
                redact_ips: debug_config.redact.redact_private_ips,
                redact_emails: debug_config.redact.redact_emails,
                redact_secrets: debug_config.redact.redact_secrets,
            },
            risk: *risk,
        }
    }
}

/// Format all settings for display
pub fn format_all_settings(
    profile: &UserProfile,
    debug_config: &DebugConfig,
    risk: &RiskTolerance,
) -> String {
    let mut sections = vec![];

    // Header
    sections.push("All Settings".to_string());
    sections.push("============".to_string());
    sections.push(String::new());

    // Preferences section
    sections.push("[preferences]".to_string());
    sections.push(format_preferences(&profile.preferences));
    sections.push(String::new());

    // Notifications section
    sections.push("[notifications]".to_string());
    sections.push(format_notification_summary(profile));
    sections.push(String::new());

    // Debug section
    sections.push("[debug]".to_string());
    sections.push(format_debug_settings(debug_config));
    sections.push(String::new());

    // Risk section
    sections.push("[risk]".to_string());
    sections.push(format_risk_settings(risk));
    sections.push(String::new());

    // Help footer
    sections.push("Configure via natural language:".to_string());
    sections.push("  \"enable learning mode\"".to_string());
    sections.push("  \"be more formal\"".to_string());
    sections.push("  \"set my email to user@example.com\"".to_string());
    sections.push("  \"enable debug\"".to_string());
    sections.push("  \"be cautious\"".to_string());

    sections.join("\n")
}

/// Format a brief summary of notification settings
fn format_notification_summary(profile: &UserProfile) -> String {
    let mut lines = vec![];

    let email_str = profile.email.as_deref().unwrap_or("not set");
    lines.push(format!("  email           {}", email_str));
    lines.push("  desktop         enabled".to_string());
    lines.push("  wall            disabled".to_string());
    lines.push("  quiet_hours     not set".to_string());

    lines.join("\n")
}

/// Format a compact one-line summary of key settings
pub fn format_settings_summary(
    profile: &UserProfile,
    debug_config: &DebugConfig,
    risk: &RiskTolerance,
) -> String {
    let learning = if profile.preferences.learning_mode {
        "learning"
    } else {
        "normal"
    };
    let verbosity = match profile.preferences.verbosity {
        0 => "minimal",
        2 => "detailed",
        _ => "normal",
    };
    let debug = debug_config.level.name();
    let risk_str = match risk.max_auto_confirm {
        crate::recipe_v3::RecipeRiskLevel::None => "cautious",
        crate::recipe_v3::RecipeRiskLevel::Low => "balanced",
        crate::recipe_v3::RecipeRiskLevel::Medium => "confident",
        crate::recipe_v3::RecipeRiskLevel::High => "expert",
    };

    format!(
        "mode={} verbosity={} debug={} risk={}",
        learning, verbosity, debug, risk_str
    )
}

/// Check if query is asking about all settings
pub fn is_all_settings_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    matches_any(&lower, &[
        "all settings", "show all settings", "all config",
        "show all config", "full settings", "every setting",
        "all my settings", "list settings", "list all settings"
    ])
}

/// Get which specific settings section was requested
pub fn get_settings_section(query: &str) -> SettingsSection {
    let lower = query.to_lowercase();

    if matches_any(&lower, &["preference", "personality", "learning mode", "verbosity"]) {
        SettingsSection::Preferences
    } else if matches_any(&lower, &["notification", "email", "desktop notify", "wall"]) {
        SettingsSection::Notifications
    } else if matches_any(&lower, &["debug", "log", "redact"]) {
        SettingsSection::Debug
    } else if matches_any(&lower, &["risk", "auto-confirm", "cautious", "expert"]) {
        SettingsSection::Risk
    } else {
        SettingsSection::All
    }
}

/// Settings section type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    All,
    Preferences,
    Notifications,
    Debug,
    Risk,
}

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_all_settings_query() {
        assert!(is_all_settings_query("show all settings"));
        assert!(is_all_settings_query("list all my settings"));
        assert!(!is_all_settings_query("how much disk space"));
    }

    #[test]
    fn test_get_settings_section() {
        assert_eq!(
            get_settings_section("show my preferences"),
            SettingsSection::Preferences
        );
        assert_eq!(
            get_settings_section("notification settings"),
            SettingsSection::Notifications
        );
        assert_eq!(
            get_settings_section("debug settings"),
            SettingsSection::Debug
        );
        assert_eq!(
            get_settings_section("risk level"),
            SettingsSection::Risk
        );
        assert_eq!(
            get_settings_section("all settings"),
            SettingsSection::All
        );
    }

    #[test]
    fn test_format_settings_summary() {
        let profile = UserProfile::default();
        let debug_config = DebugConfig::default();
        let risk = RiskTolerance::default();

        let summary = format_settings_summary(&profile, &debug_config, &risk);
        assert!(summary.contains("mode="));
        assert!(summary.contains("verbosity="));
        assert!(summary.contains("debug="));
        assert!(summary.contains("risk="));
    }

    #[test]
    fn test_format_all_settings() {
        let profile = UserProfile::default();
        let debug_config = DebugConfig::default();
        let risk = RiskTolerance::default();

        let output = format_all_settings(&profile, &debug_config, &risk);
        assert!(output.contains("All Settings"));
        assert!(output.contains("[preferences]"));
        assert!(output.contains("[notifications]"));
        assert!(output.contains("[debug]"));
        assert!(output.contains("[risk]"));
    }
}
