// v0.0.673: Settings Selector Criteria (Phase 249)
// Selection criteria for settings selector

use serde::{Deserialize, Serialize};
use super::types::MatchMode;

/// Selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCriteria {
    /// Pattern to match
    pub pattern: String,
    /// Match mode
    pub match_mode: MatchMode,
    /// Target (key or value)
    pub target: String,
}

impl SelectionCriteria {
    /// Create key criteria
    pub fn key(pattern: impl Into<String>, mode: MatchMode) -> Self {
        Self {
            pattern: pattern.into(),
            match_mode: mode,
            target: "key".to_string(),
        }
    }

    /// Create value criteria
    pub fn value(pattern: impl Into<String>, mode: MatchMode) -> Self {
        Self {
            pattern: pattern.into(),
            match_mode: mode,
            target: "value".to_string(),
        }
    }

    /// Check if matches
    pub fn matches(&self, key: &str, value: &str, case_insensitive: bool) -> bool {
        let target_str = if self.target == "key" { key } else { value };
        let (target, pattern) = if case_insensitive {
            (target_str.to_lowercase(), self.pattern.to_lowercase())
        } else {
            (target_str.to_string(), self.pattern.clone())
        };

        match self.match_mode {
            MatchMode::Exact => target == pattern,
            MatchMode::Prefix => target.starts_with(&pattern),
            MatchMode::Suffix => target.ends_with(&pattern),
            MatchMode::Contains => target.contains(&pattern),
            MatchMode::Regex => target.contains(&pattern), // Simplified
        }
    }
}
