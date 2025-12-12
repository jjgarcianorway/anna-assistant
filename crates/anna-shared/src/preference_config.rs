//! Preference Configuration via Natural Language (v0.0.467).
//!
//! Allows users to configure Anna's personality and behavior through
//! natural language commands. Per VISION.md: "All settings changeable
//! through annactl in natural language."
//!
//! Example commands:
//! - "be more formal"
//! - "less technical"
//! - "enable learning mode"
//! - "disable auto-confirm"
//! - "make answers shorter"

use crate::user_profile::{PersonalityTraits, UserPreferences, UserProfile};

/// Configuration change detected from natural language
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChange {
    /// Learning mode (explanations)
    LearningMode(bool),
    /// Verbosity (0=minimal, 1=normal, 2=detailed)
    Verbosity(u8),
    /// Auto-confirm low-risk changes
    AutoConfirm(bool),
    /// Show internal comms
    ShowInternalComms(bool),
    /// Formality (0=casual, 1=balanced, 2=formal)
    Formality(u8),
    /// Humor (0=none, 1=subtle, 2=playful)
    Humor(u8),
    /// Technical depth (0=simple, 1=balanced, 2=expert)
    TechnicalDepth(u8),
}

impl ConfigChange {
    /// Human-readable description of the change
    pub fn description(&self) -> String {
        match self {
            ConfigChange::LearningMode(true) => "Enabled learning mode (I'll explain why commands work)".to_string(),
            ConfigChange::LearningMode(false) => "Disabled learning mode".to_string(),
            ConfigChange::Verbosity(0) => "Set verbosity to minimal (short answers)".to_string(),
            ConfigChange::Verbosity(1) => "Set verbosity to normal".to_string(),
            ConfigChange::Verbosity(2) => "Set verbosity to detailed (comprehensive answers)".to_string(),
            ConfigChange::Verbosity(v) => format!("Set verbosity to {}", v),
            ConfigChange::AutoConfirm(true) => "Enabled auto-confirm for low-risk changes".to_string(),
            ConfigChange::AutoConfirm(false) => "Disabled auto-confirm (I'll ask before changes)".to_string(),
            ConfigChange::ShowInternalComms(true) => "Enabled internal comms (fly-on-the-wall view)".to_string(),
            ConfigChange::ShowInternalComms(false) => "Hidden internal comms".to_string(),
            ConfigChange::Formality(0) => "Set personality to casual".to_string(),
            ConfigChange::Formality(1) => "Set personality to balanced".to_string(),
            ConfigChange::Formality(2) => "Set personality to formal".to_string(),
            ConfigChange::Formality(f) => format!("Set formality to {}", f),
            ConfigChange::Humor(0) => "Disabled humor (professional mode)".to_string(),
            ConfigChange::Humor(1) => "Set humor to subtle".to_string(),
            ConfigChange::Humor(2) => "Set humor to playful".to_string(),
            ConfigChange::Humor(h) => format!("Set humor to {}", h),
            ConfigChange::TechnicalDepth(0) => "Set answers to simple (beginner-friendly)".to_string(),
            ConfigChange::TechnicalDepth(1) => "Set answers to balanced".to_string(),
            ConfigChange::TechnicalDepth(2) => "Set answers to expert (technical details)".to_string(),
            ConfigChange::TechnicalDepth(t) => format!("Set technical depth to {}", t),
        }
    }
}

/// Detect configuration changes from natural language
pub fn detect_config_change(query: &str) -> Option<ConfigChange> {
    let lower = query.to_lowercase();

    // Learning mode
    if matches_any(&lower, &["enable learning", "turn on learning", "explain commands", "teach me"]) {
        return Some(ConfigChange::LearningMode(true));
    }
    if matches_any(&lower, &["disable learning", "turn off learning", "no explanations", "just answer"]) {
        return Some(ConfigChange::LearningMode(false));
    }

    // Verbosity
    if matches_any(&lower, &["shorter answers", "be brief", "minimal", "less verbose", "concise"]) {
        return Some(ConfigChange::Verbosity(0));
    }
    if matches_any(&lower, &["normal verbosity", "balanced answers", "default verbosity"]) {
        return Some(ConfigChange::Verbosity(1));
    }
    if matches_any(&lower, &["detailed answers", "more verbose", "comprehensive", "explain more", "longer answers"]) {
        return Some(ConfigChange::Verbosity(2));
    }

    // Auto-confirm
    if matches_any(&lower, &["enable auto-confirm", "auto confirm", "don't ask", "just do it"]) {
        return Some(ConfigChange::AutoConfirm(true));
    }
    if matches_any(&lower, &["disable auto-confirm", "ask before", "confirm changes", "ask first"]) {
        return Some(ConfigChange::AutoConfirm(false));
    }

    // Internal comms
    if matches_any(&lower, &["show internal", "show comms", "fly on wall", "show team"]) {
        return Some(ConfigChange::ShowInternalComms(true));
    }
    if matches_any(&lower, &["hide internal", "hide comms", "no internal", "hide team"]) {
        return Some(ConfigChange::ShowInternalComms(false));
    }

    // Formality
    if matches_any(&lower, &["be casual", "informal", "relaxed", "friendly"]) {
        return Some(ConfigChange::Formality(0));
    }
    if matches_any(&lower, &["be formal", "more formal", "professional", "business", "serious"]) {
        return Some(ConfigChange::Formality(2));
    }
    if matches_any(&lower, &["balanced formality", "normal formality"]) {
        return Some(ConfigChange::Formality(1));
    }

    // Humor
    if matches_any(&lower, &["no jokes", "no humor", "serious only", "disable humor"]) {
        return Some(ConfigChange::Humor(0));
    }
    if matches_any(&lower, &["more humor", "be funny", "more funny", "playful", "enable humor"]) {
        return Some(ConfigChange::Humor(2));
    }
    if matches_any(&lower, &["subtle humor", "some humor"]) {
        return Some(ConfigChange::Humor(1));
    }

    // Technical depth
    if matches_any(&lower, &["simple answers", "beginner", "less technical", "explain simply"]) {
        return Some(ConfigChange::TechnicalDepth(0));
    }
    if matches_any(&lower, &["expert mode", "more technical", "detailed technical", "advanced"]) {
        return Some(ConfigChange::TechnicalDepth(2));
    }
    if matches_any(&lower, &["balanced technical", "normal technical"]) {
        return Some(ConfigChange::TechnicalDepth(1));
    }

    None
}

/// Check if query is asking to show current preferences
pub fn is_show_preferences(query: &str) -> bool {
    let lower = query.to_lowercase();
    matches_any(&lower, &[
        "show preferences", "show settings", "my preferences", "my settings",
        "current settings", "current preferences", "what are my settings",
        "show config", "show configuration", "my config"
    ])
}

/// Apply a config change to a user profile
pub fn apply_config_change(profile: &mut UserProfile, change: &ConfigChange) {
    match change {
        ConfigChange::LearningMode(v) => profile.preferences.learning_mode = *v,
        ConfigChange::Verbosity(v) => profile.preferences.verbosity = *v,
        ConfigChange::AutoConfirm(v) => profile.preferences.auto_confirm_low_risk = *v,
        ConfigChange::ShowInternalComms(v) => profile.preferences.show_internal_comms = *v,
        ConfigChange::Formality(v) => profile.preferences.personality.formality = *v,
        ConfigChange::Humor(v) => profile.preferences.personality.humor = *v,
        ConfigChange::TechnicalDepth(v) => profile.preferences.personality.technical_depth = *v,
    }
}

/// Format current preferences for display
pub fn format_preferences(prefs: &UserPreferences) -> String {
    let mut lines = vec![
        format!("learning_mode     {}", if prefs.learning_mode { "enabled" } else { "disabled" }),
        format!("verbosity         {}", verbosity_name(prefs.verbosity)),
        format!("auto_confirm      {}", if prefs.auto_confirm_low_risk { "enabled" } else { "disabled" }),
        format!("internal_comms    {}", if prefs.show_internal_comms { "shown" } else { "hidden" }),
        format!("formality         {}", formality_name(prefs.personality.formality)),
        format!("humor             {}", humor_name(prefs.personality.humor)),
        format!("technical_depth   {}", technical_name(prefs.personality.technical_depth)),
    ];

    lines.push(String::new());
    lines.push("Change via natural language:".to_string());
    lines.push("  \"be more formal\"".to_string());
    lines.push("  \"enable learning mode\"".to_string());
    lines.push("  \"shorter answers\"".to_string());

    lines.join("\n")
}

fn verbosity_name(v: u8) -> &'static str {
    match v {
        0 => "minimal",
        1 => "normal",
        2 => "detailed",
        _ => "custom",
    }
}

fn formality_name(f: u8) -> &'static str {
    match f {
        0 => "casual",
        1 => "balanced",
        2 => "formal",
        _ => "custom",
    }
}

fn humor_name(h: u8) -> &'static str {
    match h {
        0 => "none",
        1 => "subtle",
        2 => "playful",
        _ => "custom",
    }
}

fn technical_name(t: u8) -> &'static str {
    match t {
        0 => "simple",
        1 => "balanced",
        2 => "expert",
        _ => "custom",
    }
}

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_learning_mode() {
        assert_eq!(
            detect_config_change("enable learning mode"),
            Some(ConfigChange::LearningMode(true))
        );
        assert_eq!(
            detect_config_change("disable learning"),
            Some(ConfigChange::LearningMode(false))
        );
    }

    #[test]
    fn test_detect_verbosity() {
        assert_eq!(
            detect_config_change("shorter answers please"),
            Some(ConfigChange::Verbosity(0))
        );
        assert_eq!(
            detect_config_change("more verbose"),
            Some(ConfigChange::Verbosity(2))
        );
    }

    #[test]
    fn test_detect_formality() {
        assert_eq!(
            detect_config_change("be more formal"),
            Some(ConfigChange::Formality(2))
        );
        assert_eq!(
            detect_config_change("be casual"),
            Some(ConfigChange::Formality(0))
        );
    }

    #[test]
    fn test_detect_humor() {
        assert_eq!(
            detect_config_change("no jokes please"),
            Some(ConfigChange::Humor(0))
        );
        assert_eq!(
            detect_config_change("be more funny"),
            Some(ConfigChange::Humor(2))
        );
    }

    #[test]
    fn test_detect_technical() {
        assert_eq!(
            detect_config_change("expert mode"),
            Some(ConfigChange::TechnicalDepth(2))
        );
        assert_eq!(
            detect_config_change("explain simply"),
            Some(ConfigChange::TechnicalDepth(0))
        );
    }

    #[test]
    fn test_is_show_preferences() {
        assert!(is_show_preferences("show my preferences"));
        assert!(is_show_preferences("what are my settings"));
        assert!(!is_show_preferences("how much disk space"));
    }

    #[test]
    fn test_apply_change() {
        let mut profile = UserProfile::default();
        assert!(profile.preferences.learning_mode);

        apply_config_change(&mut profile, &ConfigChange::LearningMode(false));
        assert!(!profile.preferences.learning_mode);
    }

    #[test]
    fn test_description() {
        let change = ConfigChange::Formality(2);
        assert!(change.description().contains("formal"));
    }
}
