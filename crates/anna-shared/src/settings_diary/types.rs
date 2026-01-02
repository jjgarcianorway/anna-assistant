// v0.0.694: Settings Diary (Phase 270)
// Diary entry types and enums

use serde::{Deserialize, Serialize};

/// Diary entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DiaryEntryType {
    /// Note
    #[default]
    Note,
    /// Change
    Change,
    /// Alert
    Alert,
    /// Milestone
    Milestone,
}

impl std::fmt::Display for DiaryEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Note => write!(f, "note"),
            Self::Change => write!(f, "change"),
            Self::Alert => write!(f, "alert"),
            Self::Milestone => write!(f, "milestone"),
        }
    }
}

/// Diary importance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DiaryImportance {
    /// Low
    #[default]
    Low,
    /// Normal
    Normal,
    /// High
    High,
    /// Critical
    Critical,
}

impl std::fmt::Display for DiaryImportance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}
