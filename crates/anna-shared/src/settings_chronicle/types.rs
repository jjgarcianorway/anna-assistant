// v0.0.692: Settings Chronicle Types (Phase 268)
// Track event and mode enums

use serde::{Deserialize, Serialize};

/// Track event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ChronicleEvent {
    /// Value changed
    #[default]
    Changed,
    /// Value added
    Added,
    /// Value removed
    Removed,
    /// Value accessed
    Accessed,
}

impl std::fmt::Display for ChronicleEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Changed => write!(f, "changed"),
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Accessed => write!(f, "accessed"),
        }
    }
}

/// Track mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ChronicleMode {
    /// Track all changes
    #[default]
    All,
    /// Track writes only
    WritesOnly,
    /// Track specific keys
    Specific,
    /// Track patterns
    Pattern,
}

impl std::fmt::Display for ChronicleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::WritesOnly => write!(f, "writes_only"),
            Self::Specific => write!(f, "specific"),
            Self::Pattern => write!(f, "pattern"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_event_display() {
        assert_eq!(format!("{}", ChronicleEvent::Changed), "changed");
        assert_eq!(format!("{}", ChronicleEvent::Added), "added");
    }

    #[test]
    fn test_track_mode_display() {
        assert_eq!(format!("{}", ChronicleMode::All), "all");
        assert_eq!(format!("{}", ChronicleMode::WritesOnly), "writes_only");
    }
}
