//! Memory types and data structures

use serde::{Deserialize, Serialize};

/// Memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MemoryType {
    #[default]
    ShortTerm,
    LongTerm,
    Working,
    Episodic,
    Semantic,
}

impl MemoryType {
    pub fn name(&self) -> &'static str {
        match self {
            MemoryType::ShortTerm => "Short-term",
            MemoryType::LongTerm => "Long-term",
            MemoryType::Working => "Working",
            MemoryType::Episodic => "Episodic",
            MemoryType::Semantic => "Semantic",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            MemoryType::ShortTerm => "○",
            MemoryType::LongTerm => "●",
            MemoryType::Working => "◐",
            MemoryType::Episodic => "◑",
            MemoryType::Semantic => "◒",
        }
    }
}

/// Memory importance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub enum MemoryImportance {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl MemoryImportance {
    pub fn name(&self) -> &'static str {
        match self {
            MemoryImportance::Low => "Low",
            MemoryImportance::Normal => "Normal",
            MemoryImportance::High => "High",
            MemoryImportance::Critical => "Critical",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            MemoryImportance::Low => 1,
            MemoryImportance::Normal => 2,
            MemoryImportance::High => 3,
            MemoryImportance::Critical => 4,
        }
    }
}

/// A memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Memory key
    pub key: String,
    /// Memory content
    pub content: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Importance level
    pub importance: MemoryImportance,
    /// Access count
    pub access_count: u64,
    /// Created timestamp
    pub created_at: u64,
    /// Last accessed timestamp
    pub last_accessed: u64,
    /// Expires at (optional)
    pub expires_at: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type() {
        assert_eq!(MemoryType::LongTerm.name(), "Long-term");
        assert_eq!(MemoryType::ShortTerm.symbol(), "○");
    }

    #[test]
    fn test_memory_importance() {
        assert_eq!(MemoryImportance::High.name(), "High");
        assert_eq!(MemoryImportance::Critical.score(), 4);
        assert!(MemoryImportance::Critical > MemoryImportance::Normal);
    }
}
