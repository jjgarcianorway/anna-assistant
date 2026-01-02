// v0.0.682: Settings Collector Types (Phase 258)
// Core types and enums for settings collection

use serde::{Deserialize, Serialize};

/// Collect mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CollectMode {
    /// Merge sources (later overwrites)
    #[default]
    Merge,
    /// Union sources (no overwrite)
    Union,
    /// Intersect sources (only common keys)
    Intersect,
    /// Append all (keep duplicates with suffix)
    Append,
}

impl std::fmt::Display for CollectMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Merge => write!(f, "merge"),
            Self::Union => write!(f, "union"),
            Self::Intersect => write!(f, "intersect"),
            Self::Append => write!(f, "append"),
        }
    }
}

/// Source priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum SourcePriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    #[default]
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

impl std::fmt::Display for SourcePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_mode_display() {
        assert_eq!(format!("{}", CollectMode::Merge), "merge");
        assert_eq!(format!("{}", CollectMode::Union), "union");
    }

    #[test]
    fn test_source_priority_display() {
        assert_eq!(format!("{}", SourcePriority::Normal), "normal");
        assert_eq!(format!("{}", SourcePriority::High), "high");
    }
}
