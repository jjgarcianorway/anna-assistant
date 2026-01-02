// v0.0.659: Settings Restorer - Mode Types
// Restore modes and strategies

use serde::{Deserialize, Serialize};

/// Restore mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RestoreMode {
    /// Full restore (replace all)
    #[default]
    Full,
    /// Selective restore
    Selective,
    /// Merge restore (combine with existing)
    Merge,
    /// Override restore (only overwrite conflicts)
    Override,
}

impl std::fmt::Display for RestoreMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Selective => write!(f, "selective"),
            Self::Merge => write!(f, "merge"),
            Self::Override => write!(f, "override"),
        }
    }
}

/// Restore strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RestoreStrategy {
    /// Latest first
    #[default]
    LatestFirst,
    /// Oldest first
    OldestFirst,
    /// By priority
    ByPriority,
    /// Manual selection
    Manual,
}

impl std::fmt::Display for RestoreStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LatestFirst => write!(f, "latest_first"),
            Self::OldestFirst => write!(f, "oldest_first"),
            Self::ByPriority => write!(f, "by_priority"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_mode_display() {
        assert_eq!(format!("{}", RestoreMode::Full), "full");
        assert_eq!(format!("{}", RestoreMode::Selective), "selective");
    }

    #[test]
    fn test_restore_strategy_display() {
        assert_eq!(format!("{}", RestoreStrategy::LatestFirst), "latest_first");
        assert_eq!(format!("{}", RestoreStrategy::ByPriority), "by_priority");
    }
}
