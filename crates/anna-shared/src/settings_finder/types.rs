// v0.0.685: Settings Finder Types (Phase 261)
// Core types for settings finding

use serde::{Deserialize, Serialize};

/// Find mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FindMode {
    /// Find by exact key
    #[default]
    ExactKey,
    /// Find by key pattern
    KeyPattern,
    /// Find by value
    ByValue,
    /// Find by value pattern
    ValuePattern,
}

impl std::fmt::Display for FindMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExactKey => write!(f, "exact_key"),
            Self::KeyPattern => write!(f, "key_pattern"),
            Self::ByValue => write!(f, "by_value"),
            Self::ValuePattern => write!(f, "value_pattern"),
        }
    }
}

/// Find limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FindLimit {
    /// Find first match
    First,
    /// Find all matches
    #[default]
    All,
    /// Find up to N matches
    Max(usize),
}

impl std::fmt::Display for FindLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::All => write!(f, "all"),
            Self::Max(n) => write!(f, "max({})", n),
        }
    }
}

/// Found item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Match score (0.0-1.0)
    pub score: f64,
    /// Match type
    pub match_type: FindMode,
}

impl FoundItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, score: f64, match_type: FindMode) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            score,
            match_type,
        }
    }

    /// Is exact match
    pub fn is_exact(&self) -> bool {
        (self.score - 1.0).abs() < 0.001
    }

    /// Is partial match
    pub fn is_partial(&self) -> bool {
        self.score > 0.0 && self.score < 1.0
    }
}

/// Find result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    /// Found items
    pub items: Vec<FoundItem>,
    /// Total searched
    pub total_searched: usize,
    /// Total found
    pub total_found: usize,
    /// Mode used
    pub mode: FindMode,
}

impl FindResult {
    /// Create new result
    pub fn new(items: Vec<FoundItem>, searched: usize, mode: FindMode) -> Self {
        let total_found = items.len();
        Self {
            items,
            total_searched: searched,
            total_found,
            mode,
        }
    }

    /// Has results
    pub fn has_results(&self) -> bool {
        !self.items.is_empty()
    }

    /// Get first
    pub fn first(&self) -> Option<&FoundItem> {
        self.items.first()
    }

    /// Best match
    pub fn best_match(&self) -> Option<&FoundItem> {
        self.items.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Filter by score
    pub fn filter_by_score(&self, min_score: f64) -> Vec<&FoundItem> {
        self.items.iter().filter(|i| i.score >= min_score).collect()
    }
}

impl Default for FindResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, FindMode::ExactKey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_mode_display() {
        assert_eq!(format!("{}", FindMode::ExactKey), "exact_key");
        assert_eq!(format!("{}", FindMode::KeyPattern), "key_pattern");
    }

    #[test]
    fn test_find_limit_display() {
        assert_eq!(format!("{}", FindLimit::First), "first");
        assert_eq!(format!("{}", FindLimit::Max(5)), "max(5)");
    }

    #[test]
    fn test_found_item_new() {
        let i = FoundItem::new("key", "value", 1.0, FindMode::ExactKey);
        assert!(i.is_exact());
        assert!(!i.is_partial());
    }

    #[test]
    fn test_found_item_partial() {
        let i = FoundItem::new("key", "value", 0.5, FindMode::KeyPattern);
        assert!(!i.is_exact());
        assert!(i.is_partial());
    }

    #[test]
    fn test_result_new() {
        let r = FindResult::new(vec![FoundItem::new("k", "v", 1.0, FindMode::ExactKey)], 10, FindMode::ExactKey);
        assert!(r.has_results());
        assert_eq!(r.total_found, 1);
    }

    #[test]
    fn test_result_best_match() {
        let items = vec![
            FoundItem::new("k1", "v1", 0.5, FindMode::KeyPattern),
            FoundItem::new("k2", "v2", 0.8, FindMode::KeyPattern),
        ];
        let r = FindResult::new(items, 10, FindMode::KeyPattern);
        assert_eq!(r.best_match().unwrap().key, "k2");
    }
}
