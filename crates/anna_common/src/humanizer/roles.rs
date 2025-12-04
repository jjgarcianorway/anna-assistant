//! Role and Tone Metadata for Humanizer v0.0.71
//!
//! Defines consistent roles, personas, and tones for the IT department transcript.
//! Each role has a fixed persona and can use different tones based on context.

use serde::{Deserialize, Serialize};

/// Staff role in the IT department
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaffRole {
    /// Service desk coordinator - routes and summarizes
    ServiceDesk,
    /// Department specialist (networking, storage, etc.)
    Department,
    /// Anna daemon - evidence gatherer
    Annad,
    /// Anna assistant - primary responder
    Anna,
    /// Translator - intent parser (internal)
    Translator,
    /// Junior reviewer (internal)
    Junior,
    /// Senior escalation (internal)
    Senior,
}

impl StaffRole {
    /// Whether this role is visible in human mode transcripts
    pub fn visible_in_human(&self) -> bool {
        matches!(self, Self::ServiceDesk | Self::Department | Self::Anna)
    }

    /// Get the fixed persona name for this role
    pub fn persona(&self) -> &'static str {
        match self {
            Self::ServiceDesk => "service desk",
            Self::Department => "specialist",
            Self::Annad => "annad",
            Self::Anna => "anna",
            Self::Translator => "translator",
            Self::Junior => "junior",
            Self::Senior => "senior",
        }
    }

    /// Get the display tag for human mode transcripts
    pub fn human_tag(&self) -> &'static str {
        match self {
            Self::ServiceDesk => "service desk",
            Self::Department => "specialist", // overridden by department name
            Self::Anna => "anna",
            _ => "", // not shown in human mode
        }
    }
}

/// Message tone - affects phrasing in human mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageTone {
    /// Normal, professional tone
    #[default]
    Neutral,
    /// Quick, efficient - high confidence
    Brisk,
    /// Questioning, uncertain - low confidence
    Skeptical,
    /// Supportive, explanatory
    Helpful,
    /// Careful, warning about risks
    Cautious,
    /// Time-sensitive, important
    Urgent,
}

impl MessageTone {
    /// Get tone from confidence level
    pub fn from_confidence(confidence: u8) -> Self {
        if confidence >= 85 {
            Self::Brisk
        } else if confidence >= 70 {
            Self::Neutral
        } else if confidence >= 50 {
            Self::Helpful
        } else {
            Self::Skeptical
        }
    }

    /// Get tone prefix for human mode
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Neutral => "",
            Self::Brisk => "",
            Self::Skeptical => "I'm not certain, but ",
            Self::Helpful => "",
            Self::Cautious => "Just to be safe, ",
            Self::Urgent => "Important: ",
        }
    }
}

/// Confidence hint derived from translator/junior confidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceHint {
    Low,
    #[default]
    Medium,
    High,
}

impl ConfidenceHint {
    pub fn from_score(score: u8) -> Self {
        if score >= 80 {
            Self::High
        } else if score >= 50 {
            Self::Medium
        } else {
            Self::Low
        }
    }

    pub fn to_score(&self) -> u8 {
        match self {
            Self::Low => 40,
            Self::Medium => 65,
            Self::High => 85,
        }
    }
}

/// Department identifier for transcript display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepartmentTag {
    Network,
    Storage,
    Performance,
    Audio,
    Graphics,
    Boot,
    Security,
    InfoDesk,
}

impl DepartmentTag {
    /// Human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Storage => "storage",
            Self::Performance => "performance",
            Self::Audio => "audio",
            Self::Graphics => "graphics",
            Self::Boot => "boot",
            Self::Security => "security",
            Self::InfoDesk => "info desk",
        }
    }

    /// Standard transcript tag
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Storage => "storage",
            Self::Performance => "performance",
            Self::Audio => "audio",
            Self::Graphics => "graphics",
            Self::Boot => "boot",
            Self::Security => "security",
            Self::InfoDesk => "info desk",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_visibility() {
        assert!(StaffRole::ServiceDesk.visible_in_human());
        assert!(StaffRole::Department.visible_in_human());
        assert!(StaffRole::Anna.visible_in_human());
        assert!(!StaffRole::Translator.visible_in_human());
        assert!(!StaffRole::Junior.visible_in_human());
        assert!(!StaffRole::Annad.visible_in_human());
    }

    #[test]
    fn test_tone_from_confidence() {
        assert_eq!(MessageTone::from_confidence(90), MessageTone::Brisk);
        assert_eq!(MessageTone::from_confidence(75), MessageTone::Neutral);
        assert_eq!(MessageTone::from_confidence(55), MessageTone::Helpful);
        assert_eq!(MessageTone::from_confidence(40), MessageTone::Skeptical);
    }

    #[test]
    fn test_confidence_hint() {
        assert_eq!(ConfidenceHint::from_score(85), ConfidenceHint::High);
        assert_eq!(ConfidenceHint::from_score(60), ConfidenceHint::Medium);
        assert_eq!(ConfidenceHint::from_score(30), ConfidenceHint::Low);
    }
}
