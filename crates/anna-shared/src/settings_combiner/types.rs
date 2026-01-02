// v0.0.690: Settings Combiner Types (Phase 266)
// Type definitions for settings combining

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Combine strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CombineStrategy {
    /// Left wins conflicts
    #[default]
    LeftWins,
    /// Right wins conflicts
    RightWins,
    /// Keep both (suffix)
    KeepBoth,
    /// Error on conflict
    ErrorOnConflict,
}

impl std::fmt::Display for CombineStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftWins => write!(f, "left_wins"),
            Self::RightWins => write!(f, "right_wins"),
            Self::KeepBoth => write!(f, "keep_both"),
            Self::ErrorOnConflict => write!(f, "error_on_conflict"),
        }
    }
}

/// Merge depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CombineDepth {
    /// Shallow merge
    #[default]
    Shallow,
    /// Deep merge
    Deep,
    /// Recursive merge
    Recursive,
    /// Flat merge
    Flat,
}

impl std::fmt::Display for CombineDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shallow => write!(f, "shallow"),
            Self::Deep => write!(f, "deep"),
            Self::Recursive => write!(f, "recursive"),
            Self::Flat => write!(f, "flat"),
        }
    }
}

/// Combiner config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinerConfig {
    /// Combine strategy
    pub strategy: CombineStrategy,
    /// Merge depth
    pub depth: CombineDepth,
    /// Preserve empty values
    pub preserve_empty: bool,
    /// Conflict suffix
    pub conflict_suffix: String,
}

impl CombinerConfig {
    /// Create new config
    pub fn new(strategy: CombineStrategy) -> Self {
        Self {
            strategy,
            depth: CombineDepth::Shallow,
            preserve_empty: false,
            conflict_suffix: "_conflict".to_string(),
        }
    }

    /// Set depth
    pub fn depth(mut self, depth: CombineDepth) -> Self {
        self.depth = depth;
        self
    }

    /// Set preserve empty
    pub fn preserve_empty(mut self, preserve: bool) -> Self {
        self.preserve_empty = preserve;
        self
    }

    /// Set conflict suffix
    pub fn conflict_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.conflict_suffix = suffix.into();
        self
    }
}

impl Default for CombinerConfig {
    fn default() -> Self {
        Self::new(CombineStrategy::RightWins)
    }
}

/// Merge conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombineConflict {
    /// Key
    pub key: String,
    /// Left value
    pub left_value: String,
    /// Right value
    pub right_value: String,
    /// Resolved value
    pub resolved_value: Option<String>,
}

impl CombineConflict {
    /// Create new conflict
    pub fn new(key: impl Into<String>, left: impl Into<String>, right: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            left_value: left.into(),
            right_value: right.into(),
            resolved_value: None,
        }
    }

    /// Resolve with value
    pub fn resolve(mut self, value: impl Into<String>) -> Self {
        self.resolved_value = Some(value.into());
        self
    }

    /// Is resolved
    pub fn is_resolved(&self) -> bool {
        self.resolved_value.is_some()
    }
}

/// Merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombineResult {
    /// Merged settings
    pub merged: HashMap<String, String>,
    /// Conflicts
    pub conflicts: Vec<CombineConflict>,
    /// Keys from left
    pub from_left: usize,
    /// Keys from right
    pub from_right: usize,
    /// Conflicts count
    pub conflict_count: usize,
}

impl CombineResult {
    /// Create new result
    pub fn new(merged: HashMap<String, String>, conflicts: Vec<CombineConflict>, left: usize, right: usize) -> Self {
        let conflict_count = conflicts.len();
        Self {
            merged,
            conflicts,
            from_left: left,
            from_right: right,
            conflict_count,
        }
    }

    /// Has conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Total keys
    pub fn total_keys(&self) -> usize {
        self.merged.len()
    }

    /// Get merged value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.merged.get(key)
    }
}

impl Default for CombineResult {
    fn default() -> Self {
        Self::new(HashMap::new(), Vec::new(), 0, 0)
    }
}

/// Combiner stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CombinerStats {
    /// Total merges
    pub total_merges: usize,
    /// Total keys merged
    pub total_keys: usize,
    /// Total conflicts
    pub total_conflicts: usize,
    /// By strategy
    pub by_strategy: HashMap<String, usize>,
}

impl CombinerStats {
    /// Record merge
    pub fn record(&mut self, result: &CombineResult, strategy: CombineStrategy) {
        self.total_merges += 1;
        self.total_keys += result.merged.len();
        self.total_conflicts += result.conflict_count;
        *self.by_strategy.entry(strategy.to_string()).or_insert(0) += 1;
    }

    /// Conflict rate
    pub fn conflict_rate(&self) -> f64 {
        if self.total_keys == 0 {
            0.0
        } else {
            self.total_conflicts as f64 / self.total_keys as f64
        }
    }
}
