//! Exposure Levels - Strict information boundary control.
//!
//! Levels are mutually exclusive and strictly ordered.
//! No implicit escalation. No partial overlap.

use serde::{Deserialize, Serialize};

/// Exposure levels control what information users can see.
///
/// These are strictly ordered: Silent < Summary < Dialogue < Debug
/// Each level is a superset of the previous (except Silent which shows nothing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExposureLevel {
    /// No internal information shown. Only final answers.
    /// This is the default for new users.
    #[default]
    Silent,
    /// Summary metadata only. No dialogue, no timing.
    /// Shows: iteration count, departments involved.
    Summary,
    /// Full dialogue with timing. No debug information.
    /// Shows: speaker exchanges, timestamps, processing stages.
    Dialogue,
    /// Everything including raw debug data.
    /// Shows: all of the above plus raw events, IDs, internal state.
    Debug,
}

impl ExposureLevel {
    /// Parse from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "silent" | "off" | "none" => Some(Self::Silent),
            "summary" | "minimal" => Some(Self::Summary),
            "dialogue" | "dialog" | "internal" => Some(Self::Dialogue),
            "debug" | "full" | "all" => Some(Self::Debug),
            _ => None,
        }
    }

    /// Human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Silent => "No internal information shown",
            Self::Summary => "Summary metadata only",
            Self::Dialogue => "Internal dialogue with timing",
            Self::Debug => "Full debug information",
        }
    }

    /// Short name for display.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::Summary => "summary",
            Self::Dialogue => "dialogue",
            Self::Debug => "debug",
        }
    }
}

/// Filter that determines what to show at a given exposure level.
#[derive(Debug, Clone, Copy)]
pub struct ExposureFilter {
    level: ExposureLevel,
}

impl ExposureFilter {
    /// Create a filter for the given level.
    pub fn new(level: ExposureLevel) -> Self {
        Self { level }
    }

    /// Check if dialogue lines should be shown.
    pub fn show_dialogue(&self) -> bool {
        self.level >= ExposureLevel::Dialogue
    }

    /// Check if timing information should be shown.
    pub fn show_timing(&self) -> bool {
        self.level >= ExposureLevel::Dialogue
    }

    /// Check if summary metadata should be shown.
    pub fn show_metadata(&self) -> bool {
        self.level >= ExposureLevel::Summary
    }

    /// Check if debug information should be shown.
    pub fn show_debug(&self) -> bool {
        self.level >= ExposureLevel::Debug
    }

    /// Check if internal IDs should be shown.
    pub fn show_internal_ids(&self) -> bool {
        self.level >= ExposureLevel::Debug
    }

    /// Check if raw events should be shown.
    pub fn show_raw_events(&self) -> bool {
        self.level >= ExposureLevel::Debug
    }

    /// Get the current level.
    pub fn level(&self) -> ExposureLevel {
        self.level
    }
}

/// Convenience function to check if something should be shown.
pub fn should_show(level: ExposureLevel, required: ExposureLevel) -> bool {
    level >= required
}

/// Information category for exposure gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoCategory {
    /// Final answer to user.
    FinalAnswer,
    /// Iteration/summary metadata.
    Metadata,
    /// Dialogue lines between components.
    Dialogue,
    /// Timing information.
    Timing,
    /// Debug/internal data.
    Debug,
    /// Raw event data.
    RawEvents,
}

/// Dialogue classification for structural tagging (Phase 13).
///
/// This is NOT sentiment analysis. It is structural tagging only.
/// Exposure rules act on classification plus level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogueClassification {
    /// General status updates and progress information.
    /// Example: "New request received", "Processing complete"
    #[default]
    Informational,
    /// Step-by-step processing actions.
    /// Example: "Running probe", "Executing recipe", "Checking docs"
    Procedural,
    /// Technical diagnostic output.
    /// Example: "Command output", "Error details", "Raw data"
    Diagnostic,
}

impl DialogueClassification {
    /// Minimum exposure level required to see this classification.
    pub fn required_level(&self) -> ExposureLevel {
        match self {
            // Informational shown at Summary and above
            Self::Informational => ExposureLevel::Summary,
            // Procedural shown at Dialogue and above
            Self::Procedural => ExposureLevel::Dialogue,
            // Diagnostic shown at Debug only
            Self::Diagnostic => ExposureLevel::Debug,
        }
    }

    /// Check if this classification is visible at the given level.
    pub fn visible_at(&self, level: ExposureLevel) -> bool {
        level >= self.required_level()
    }
}

impl InfoCategory {
    /// Minimum exposure level required to see this category.
    pub fn required_level(&self) -> ExposureLevel {
        match self {
            Self::FinalAnswer => ExposureLevel::Silent, // Always shown
            Self::Metadata => ExposureLevel::Summary,
            Self::Dialogue => ExposureLevel::Dialogue,
            Self::Timing => ExposureLevel::Dialogue,
            Self::Debug => ExposureLevel::Debug,
            Self::RawEvents => ExposureLevel::Debug,
        }
    }

    /// Check if this category is visible at the given level.
    pub fn visible_at(&self, level: ExposureLevel) -> bool {
        level >= self.required_level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(ExposureLevel::Silent < ExposureLevel::Summary);
        assert!(ExposureLevel::Summary < ExposureLevel::Dialogue);
        assert!(ExposureLevel::Dialogue < ExposureLevel::Debug);
    }

    #[test]
    fn test_level_parse() {
        assert_eq!(ExposureLevel::from_str("silent"), Some(ExposureLevel::Silent));
        assert_eq!(ExposureLevel::from_str("DIALOGUE"), Some(ExposureLevel::Dialogue));
        assert_eq!(ExposureLevel::from_str("off"), Some(ExposureLevel::Silent));
        assert_eq!(ExposureLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_filter_silent() {
        let filter = ExposureFilter::new(ExposureLevel::Silent);
        assert!(!filter.show_dialogue());
        assert!(!filter.show_timing());
        assert!(!filter.show_metadata());
        assert!(!filter.show_debug());
    }

    #[test]
    fn test_filter_summary() {
        let filter = ExposureFilter::new(ExposureLevel::Summary);
        assert!(!filter.show_dialogue());
        assert!(!filter.show_timing());
        assert!(filter.show_metadata());
        assert!(!filter.show_debug());
    }

    #[test]
    fn test_filter_dialogue() {
        let filter = ExposureFilter::new(ExposureLevel::Dialogue);
        assert!(filter.show_dialogue());
        assert!(filter.show_timing());
        assert!(filter.show_metadata());
        assert!(!filter.show_debug());
    }

    #[test]
    fn test_filter_debug() {
        let filter = ExposureFilter::new(ExposureLevel::Debug);
        assert!(filter.show_dialogue());
        assert!(filter.show_timing());
        assert!(filter.show_metadata());
        assert!(filter.show_debug());
    }

    #[test]
    fn test_info_category_visibility() {
        // Final answer always visible
        assert!(InfoCategory::FinalAnswer.visible_at(ExposureLevel::Silent));

        // Metadata needs Summary+
        assert!(!InfoCategory::Metadata.visible_at(ExposureLevel::Silent));
        assert!(InfoCategory::Metadata.visible_at(ExposureLevel::Summary));

        // Dialogue needs Dialogue+
        assert!(!InfoCategory::Dialogue.visible_at(ExposureLevel::Summary));
        assert!(InfoCategory::Dialogue.visible_at(ExposureLevel::Dialogue));

        // Debug needs Debug
        assert!(!InfoCategory::Debug.visible_at(ExposureLevel::Dialogue));
        assert!(InfoCategory::Debug.visible_at(ExposureLevel::Debug));
    }

    #[test]
    fn test_no_implicit_escalation() {
        // Verify that lower levels cannot see higher level information
        for level in [ExposureLevel::Silent, ExposureLevel::Summary, ExposureLevel::Dialogue] {
            let filter = ExposureFilter::new(level);
            // Debug info should never be visible below Debug level
            if level < ExposureLevel::Debug {
                assert!(!filter.show_debug(), "Level {:?} should not show debug", level);
                assert!(!filter.show_raw_events(), "Level {:?} should not show raw events", level);
            }
        }
    }

    #[test]
    fn test_dialogue_classification_visibility() {
        // Informational: Summary and above
        assert!(!DialogueClassification::Informational.visible_at(ExposureLevel::Silent));
        assert!(DialogueClassification::Informational.visible_at(ExposureLevel::Summary));
        assert!(DialogueClassification::Informational.visible_at(ExposureLevel::Dialogue));
        assert!(DialogueClassification::Informational.visible_at(ExposureLevel::Debug));

        // Procedural: Dialogue and above
        assert!(!DialogueClassification::Procedural.visible_at(ExposureLevel::Silent));
        assert!(!DialogueClassification::Procedural.visible_at(ExposureLevel::Summary));
        assert!(DialogueClassification::Procedural.visible_at(ExposureLevel::Dialogue));
        assert!(DialogueClassification::Procedural.visible_at(ExposureLevel::Debug));

        // Diagnostic: Debug only
        assert!(!DialogueClassification::Diagnostic.visible_at(ExposureLevel::Silent));
        assert!(!DialogueClassification::Diagnostic.visible_at(ExposureLevel::Summary));
        assert!(!DialogueClassification::Diagnostic.visible_at(ExposureLevel::Dialogue));
        assert!(DialogueClassification::Diagnostic.visible_at(ExposureLevel::Debug));
    }

    #[test]
    fn test_dialogue_classification_required_levels() {
        assert_eq!(DialogueClassification::Informational.required_level(), ExposureLevel::Summary);
        assert_eq!(DialogueClassification::Procedural.required_level(), ExposureLevel::Dialogue);
        assert_eq!(DialogueClassification::Diagnostic.required_level(), ExposureLevel::Debug);
    }

    #[test]
    fn test_dialogue_classification_default() {
        assert_eq!(DialogueClassification::default(), DialogueClassification::Informational);
    }
}
