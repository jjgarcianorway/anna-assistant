// v0.0.707: Settings Journal (Phase 283)
// Journal enums

use serde::{Deserialize, Serialize};

/// Journal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum JournalType {
    /// Personal journal
    #[default]
    Personal,
    /// Technical journal
    Technical,
    /// Research journal
    Research,
    /// Log journal
    Log,
}

impl std::fmt::Display for JournalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Personal => write!(f, "personal"),
            Self::Technical => write!(f, "technical"),
            Self::Research => write!(f, "research"),
            Self::Log => write!(f, "log"),
        }
    }
}

/// Journal mood
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum JournalMood {
    /// Productive
    #[default]
    Productive,
    /// Challenging
    Challenging,
    /// Learning
    Learning,
    /// Resolved
    Resolved,
}

impl std::fmt::Display for JournalMood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Productive => write!(f, "productive"),
            Self::Challenging => write!(f, "challenging"),
            Self::Learning => write!(f, "learning"),
            Self::Resolved => write!(f, "resolved"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_type_display() {
        assert_eq!(format!("{}", JournalType::Personal), "personal");
        assert_eq!(format!("{}", JournalType::Technical), "technical");
    }

    #[test]
    fn test_mood_display() {
        assert_eq!(format!("{}", JournalMood::Productive), "productive");
        assert_eq!(format!("{}", JournalMood::Learning), "learning");
    }
}
