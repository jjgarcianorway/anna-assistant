//! Configuration change types and methods.

use crate::user_profile::UserPreferences;

/// A parsed config change request
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChange {
    /// Toggle learning mode (show explanations)
    LearningMode(bool),
    /// Change verbosity (0=minimal, 1=normal, 2=detailed)
    Verbosity(u8),
    /// Toggle auto-confirm for low-risk changes
    AutoConfirmLowRisk(bool),
    /// Toggle internal comms display (fly on wall)
    ShowInternalComms(bool),
    /// Change formality (0=casual, 1=balanced, 2=formal)
    Formality(u8),
    /// Change humor level (0=none, 1=subtle, 2=playful)
    Humor(u8),
    /// Change technical depth (0=simple, 1=balanced, 2=expert)
    TechnicalDepth(u8),
    /// v0.0.239: Set email address for notifications
    Email(String),
    /// v0.0.239: Clear email (disable notifications)
    ClearEmail,
}

impl ConfigChange {
    /// Apply this change to preferences (for preference-based settings only)
    /// Email changes need to be handled separately via apply_email()
    pub fn apply(&self, prefs: &mut UserPreferences) {
        match self {
            ConfigChange::LearningMode(v) => prefs.learning_mode = *v,
            ConfigChange::Verbosity(v) => prefs.verbosity = *v,
            ConfigChange::AutoConfirmLowRisk(v) => prefs.auto_confirm_low_risk = *v,
            ConfigChange::ShowInternalComms(v) => prefs.show_internal_comms = *v,
            ConfigChange::Formality(v) => prefs.personality.formality = *v,
            ConfigChange::Humor(v) => prefs.personality.humor = *v,
            ConfigChange::TechnicalDepth(v) => prefs.personality.technical_depth = *v,
            // Email changes are handled separately
            ConfigChange::Email(_) | ConfigChange::ClearEmail => {}
        }
    }

    /// Check if this is an email-related change
    pub fn is_email_change(&self) -> bool {
        matches!(self, ConfigChange::Email(_) | ConfigChange::ClearEmail)
    }

    /// Get email address if this is an email change
    pub fn get_email(&self) -> Option<&str> {
        match self {
            ConfigChange::Email(e) => Some(e),
            _ => None,
        }
    }

    /// Describe the change in human-readable form
    pub fn description(&self) -> String {
        match self {
            ConfigChange::LearningMode(true) => {
                "Enabled learning mode - I'll explain why commands work.".to_string()
            }
            ConfigChange::LearningMode(false) => {
                "Disabled learning mode - answers will be more concise.".to_string()
            }
            ConfigChange::Verbosity(0) => {
                "Set verbosity to minimal - short answers only.".to_string()
            }
            ConfigChange::Verbosity(1) => "Set verbosity to normal.".to_string(),
            ConfigChange::Verbosity(2) => {
                "Set verbosity to detailed - full explanations.".to_string()
            }
            ConfigChange::Verbosity(_) => "Verbosity updated.".to_string(),
            ConfigChange::AutoConfirmLowRisk(true) => {
                "Enabled auto-confirm for low-risk changes.".to_string()
            }
            ConfigChange::AutoConfirmLowRisk(false) => {
                "Disabled auto-confirm - I'll ask before changes.".to_string()
            }
            ConfigChange::ShowInternalComms(true) => {
                "Showing internal IT communications.".to_string()
            }
            ConfigChange::ShowInternalComms(false) => {
                "Hidden internal communications - cleaner output.".to_string()
            }
            ConfigChange::Formality(0) => "Made my style more casual.".to_string(),
            ConfigChange::Formality(1) => "Set to balanced formality.".to_string(),
            ConfigChange::Formality(2) => "Made my style more formal.".to_string(),
            ConfigChange::Formality(_) => "Formality updated.".to_string(),
            ConfigChange::Humor(0) => "Disabled humor - professional mode.".to_string(),
            ConfigChange::Humor(1) => "Set to subtle humor.".to_string(),
            ConfigChange::Humor(2) => "Enabled playful humor mode.".to_string(),
            ConfigChange::Humor(_) => "Humor level updated.".to_string(),
            ConfigChange::TechnicalDepth(0) => "Simplified technical language.".to_string(),
            ConfigChange::TechnicalDepth(1) => "Balanced technical depth.".to_string(),
            ConfigChange::TechnicalDepth(2) => "Expert mode - full technical details.".to_string(),
            ConfigChange::TechnicalDepth(_) => "Technical depth updated.".to_string(),
            ConfigChange::Email(addr) => format!(
                "Got it! I'll notify you at {} for long-running tickets.",
                addr
            ),
            ConfigChange::ClearEmail => "Email notifications disabled.".to_string(),
        }
    }
}
