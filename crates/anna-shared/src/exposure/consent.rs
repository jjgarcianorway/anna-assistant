//! Consent Tracking - First-time acknowledgement for internal dialogue.
//!
//! Internal dialogue must never appear by surprise.
//! First-time enablement requires explicit acknowledgement.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::anna_data_dir;
use super::levels::ExposureLevel;

/// The acknowledgement text shown on first-time enablement.
pub const CONSENT_ACKNOWLEDGEMENT: &str = "\
Internal dialogue mode shows processing stages in human-readable format.
This is a debugging aid that displays routing decisions and system activity.
It does not represent actual communication between conscious entities.
Anna is software that processes requests according to programmed rules.";

/// First-time notice for dialogue mode.
pub const DIALOGUE_FIRST_TIME_NOTICE: &str = "\
You are enabling internal dialogue view.
This shows processing stages formatted as conversation for readability.
Anna is a software tool. The dialogue format is for clarity, not consciousness.";

/// State of user consent for exposure features.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsentState {
    /// Whether user has acknowledged internal dialogue explanation.
    pub dialogue_acknowledged: bool,
    /// Whether user has acknowledged debug mode explanation.
    pub debug_acknowledged: bool,
    /// Highest exposure level ever enabled (for replay bounds).
    pub max_exposure_used: Option<ExposureLevel>,
    /// Timestamp of first acknowledgement.
    pub first_acknowledged_at: Option<String>,
    /// Current exposure level preference.
    pub current_level: ExposureLevel,
}

impl ConsentState {
    /// Path to consent state file.
    fn path() -> PathBuf {
        anna_data_dir().join("consent_state.json")
    }

    /// Load consent state from disk.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save consent state to disk.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(Self::path(), json)
    }

    /// Check if the given level has been acknowledged.
    pub fn is_acknowledged(&self, level: ExposureLevel) -> bool {
        match level {
            ExposureLevel::Silent => true, // No acknowledgement needed
            ExposureLevel::Summary => true, // No acknowledgement needed
            ExposureLevel::Dialogue => self.dialogue_acknowledged,
            ExposureLevel::Debug => self.debug_acknowledged,
        }
    }

    /// Record acknowledgement for a level.
    pub fn acknowledge(&mut self, level: ExposureLevel) {
        let now = chrono::Utc::now().to_rfc3339();

        match level {
            ExposureLevel::Dialogue => {
                self.dialogue_acknowledged = true;
            }
            ExposureLevel::Debug => {
                self.debug_acknowledged = true;
                // Debug implies dialogue
                self.dialogue_acknowledged = true;
            }
            _ => {}
        }

        // Track max exposure used (for replay bounds)
        if let Some(ref max) = self.max_exposure_used {
            if level > *max {
                self.max_exposure_used = Some(level);
            }
        } else {
            self.max_exposure_used = Some(level);
        }

        // Record first acknowledgement time
        if self.first_acknowledged_at.is_none() {
            self.first_acknowledged_at = Some(now);
        }

        self.current_level = level;
    }

    /// Get the acknowledgement text for a level.
    pub fn acknowledgement_text(level: ExposureLevel) -> Option<&'static str> {
        match level {
            ExposureLevel::Dialogue => Some(DIALOGUE_FIRST_TIME_NOTICE),
            ExposureLevel::Debug => Some(CONSENT_ACKNOWLEDGEMENT),
            _ => None,
        }
    }

    /// Check if first-time notice should be shown.
    pub fn needs_first_time_notice(&self, level: ExposureLevel) -> bool {
        !self.is_acknowledged(level) && Self::acknowledgement_text(level).is_some()
    }
}

/// Check if consent has been given for the specified level.
pub fn check_consent(level: ExposureLevel) -> bool {
    ConsentState::load().is_acknowledged(level)
}

/// Record consent for a level and save to disk.
pub fn record_consent(level: ExposureLevel) -> Result<(), std::io::Error> {
    let mut state = ConsentState::load();
    state.acknowledge(level);
    state.save()
}

/// Result of a consent check with potential notice.
#[derive(Debug)]
pub struct ConsentCheck {
    /// Whether consent is granted.
    pub granted: bool,
    /// Notice to show if first time.
    pub first_time_notice: Option<&'static str>,
}

/// Check consent and get any required notice.
pub fn check_consent_with_notice(level: ExposureLevel) -> ConsentCheck {
    let state = ConsentState::load();
    let granted = state.is_acknowledged(level);
    let first_time_notice = if state.needs_first_time_notice(level) {
        ConsentState::acknowledgement_text(level)
    } else {
        None
    };

    ConsentCheck {
        granted,
        first_time_notice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = ConsentState::default();
        assert!(!state.dialogue_acknowledged);
        assert!(!state.debug_acknowledged);
        assert!(state.max_exposure_used.is_none());
    }

    #[test]
    fn test_silent_always_acknowledged() {
        let state = ConsentState::default();
        assert!(state.is_acknowledged(ExposureLevel::Silent));
        assert!(state.is_acknowledged(ExposureLevel::Summary));
    }

    #[test]
    fn test_dialogue_needs_acknowledgement() {
        let state = ConsentState::default();
        assert!(!state.is_acknowledged(ExposureLevel::Dialogue));
        assert!(state.needs_first_time_notice(ExposureLevel::Dialogue));
    }

    #[test]
    fn test_acknowledge_dialogue() {
        let mut state = ConsentState::default();
        state.acknowledge(ExposureLevel::Dialogue);
        assert!(state.dialogue_acknowledged);
        assert!(state.is_acknowledged(ExposureLevel::Dialogue));
        assert!(!state.needs_first_time_notice(ExposureLevel::Dialogue));
    }

    #[test]
    fn test_debug_implies_dialogue() {
        let mut state = ConsentState::default();
        state.acknowledge(ExposureLevel::Debug);
        assert!(state.dialogue_acknowledged);
        assert!(state.debug_acknowledged);
    }

    #[test]
    fn test_max_exposure_tracking() {
        let mut state = ConsentState::default();
        state.acknowledge(ExposureLevel::Summary);
        assert_eq!(state.max_exposure_used, Some(ExposureLevel::Summary));

        state.acknowledge(ExposureLevel::Debug);
        assert_eq!(state.max_exposure_used, Some(ExposureLevel::Debug));

        // Lower level shouldn't reduce max
        state.acknowledge(ExposureLevel::Dialogue);
        assert_eq!(state.max_exposure_used, Some(ExposureLevel::Debug));
    }

    #[test]
    fn test_acknowledgement_text_exists() {
        assert!(ConsentState::acknowledgement_text(ExposureLevel::Dialogue).is_some());
        assert!(ConsentState::acknowledgement_text(ExposureLevel::Debug).is_some());
        assert!(ConsentState::acknowledgement_text(ExposureLevel::Silent).is_none());
    }

    #[test]
    fn test_consent_acknowledgement_is_professional() {
        // Verify the acknowledgement text follows our wording guidelines
        use super::super::sanitize::sanitize_dialogue;

        let result = sanitize_dialogue(CONSENT_ACKNOWLEDGEMENT);
        assert!(result.is_clean, "CONSENT_ACKNOWLEDGEMENT has forbidden patterns: {:?}", result.violations);

        let result = sanitize_dialogue(DIALOGUE_FIRST_TIME_NOTICE);
        assert!(result.is_clean, "DIALOGUE_FIRST_TIME_NOTICE has forbidden patterns: {:?}", result.violations);
    }
}
