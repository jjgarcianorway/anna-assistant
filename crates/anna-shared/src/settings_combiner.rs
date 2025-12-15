// v0.0.690: Settings Combiner (Phase 266)
// Merge multiple settings collections

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

/// Settings combiner
#[derive(Debug, Clone, Default)]
pub struct SettingsCombiner {
    /// Config
    config: CombinerConfig,
    /// Stats
    stats: CombinerStats,
}

impl SettingsCombiner {
    /// Create new combiner
    pub fn new(config: CombinerConfig) -> Self {
        Self {
            config,
            stats: CombinerStats::default(),
        }
    }

    /// Merge two collections
    pub fn merge(&mut self, left: &HashMap<String, String>, right: &HashMap<String, String>) -> CombineResult {
        let mut merged = HashMap::new();
        let mut conflicts = Vec::new();
        let mut from_left = 0i32;
        let mut from_right = 0i32;

        // Add all left entries
        for (key, value) in left {
            if !self.config.preserve_empty && value.is_empty() {
                continue;
            }
            merged.insert(key.clone(), value.clone());
            from_left += 1;
        }

        // Process right entries
        for (key, value) in right {
            if !self.config.preserve_empty && value.is_empty() {
                continue;
            }

            if let Some(left_value) = left.get(key) {
                if left_value != value {
                    // Conflict
                    let conflict = CombineConflict::new(key.clone(), left_value.clone(), value.clone());

                    match self.config.strategy {
                        CombineStrategy::LeftWins => {
                            // Keep left (already in merged)
                        }
                        CombineStrategy::RightWins => {
                            merged.insert(key.clone(), value.clone());
                            from_right += 1;
                            from_left -= 1;
                        }
                        CombineStrategy::KeepBoth => {
                            let conflict_key = format!("{}{}", key, self.config.conflict_suffix);
                            merged.insert(conflict_key, value.clone());
                            from_right += 1;
                        }
                        CombineStrategy::ErrorOnConflict => {
                            conflicts.push(conflict);
                        }
                    }
                }
                // No conflict if values are equal
            } else {
                // New key from right
                merged.insert(key.clone(), value.clone());
                from_right += 1;
            }
        }

        let result = CombineResult::new(merged, conflicts, from_left.max(0) as usize, from_right.max(0) as usize);
        self.stats.record(&result, self.config.strategy);
        result
    }

    /// Merge multiple collections
    pub fn merge_all(&mut self, collections: &[HashMap<String, String>]) -> CombineResult {
        if collections.is_empty() {
            return CombineResult::default();
        }

        let mut result = collections[0].clone();
        for collection in collections.iter().skip(1) {
            let merge_result = self.merge(&result, collection);
            result = merge_result.merged;
        }

        CombineResult::new(result, Vec::new(), 0, 0)
    }

    /// Get stats
    pub fn stats(&self) -> &CombinerStats {
        &self.stats
    }
}

/// Combiner registry
#[derive(Debug, Clone, Default)]
pub struct CombinerRegistry {
    /// Combiners by ID
    combiners: HashMap<String, SettingsCombiner>,
}

impl CombinerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register combiner
    pub fn register(&mut self, id: impl Into<String>, combiner: SettingsCombiner) {
        self.combiners.insert(id.into(), combiner);
    }

    /// Unregister combiner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.combiners.remove(id).is_some()
    }

    /// Get combiner
    pub fn get(&self, id: &str) -> Option<&SettingsCombiner> {
        self.combiners.get(id)
    }

    /// Get combiner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCombiner> {
        self.combiners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.combiners.len()
    }
}

/// Format combiner registry
pub fn format_combiner_registry(registry: &CombinerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Combiner Registry:\n");
    output.push_str(&format!("  Combiners: {}\n", registry.count()));
    output
}

/// Check if query is about combiner
pub fn is_combiner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("combine settings") || lower.contains("settings combiner") || lower.contains("combine settings")
}

/// Fun fact about combiner
pub fn combiner_fun_fact() -> &'static str {
    "Anna's settings combiner combines configurations with smart conflict resolution!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(format!("{}", CombineStrategy::LeftWins), "left_wins");
        assert_eq!(format!("{}", CombineStrategy::RightWins), "right_wins");
    }

    #[test]
    fn test_merge_depth_display() {
        assert_eq!(format!("{}", CombineDepth::Shallow), "shallow");
        assert_eq!(format!("{}", CombineDepth::Deep), "deep");
    }

    #[test]
    fn test_config_new() {
        let c = CombinerConfig::new(CombineStrategy::LeftWins);
        assert_eq!(c.strategy, CombineStrategy::LeftWins);
    }

    #[test]
    fn test_config_builder() {
        let c = CombinerConfig::new(CombineStrategy::RightWins)
            .depth(CombineDepth::Deep)
            .preserve_empty(true);
        assert_eq!(c.depth, CombineDepth::Deep);
        assert!(c.preserve_empty);
    }

    #[test]
    fn test_conflict_new() {
        let c = CombineConflict::new("key", "left", "right");
        assert!(!c.is_resolved());
    }

    #[test]
    fn test_conflict_resolve() {
        let c = CombineConflict::new("key", "left", "right").resolve("final");
        assert!(c.is_resolved());
    }

    #[test]
    fn test_result_new() {
        let mut merged = HashMap::new();
        merged.insert("a".to_string(), "1".to_string());
        let r = CombineResult::new(merged, Vec::new(), 1, 0);
        assert_eq!(r.total_keys(), 1);
    }

    #[test]
    fn test_result_has_conflicts() {
        let conflicts = vec![CombineConflict::new("k", "l", "r")];
        let r = CombineResult::new(HashMap::new(), conflicts, 0, 0);
        assert!(r.has_conflicts());
    }

    #[test]
    fn test_stats_record() {
        let mut s = CombinerStats::default();
        let r = CombineResult::new(HashMap::new(), Vec::new(), 0, 0);
        s.record(&r, CombineStrategy::LeftWins);
        assert_eq!(s.total_merges, 1);
    }

    #[test]
    fn test_combiner_new() {
        let m = SettingsCombiner::new(CombinerConfig::default());
        assert_eq!(m.stats().total_merges, 0);
    }

    #[test]
    fn test_combiner_merge_no_conflict() {
        let mut m = SettingsCombiner::new(CombinerConfig::default());
        let mut left = HashMap::new();
        left.insert("a".to_string(), "1".to_string());
        let mut right = HashMap::new();
        right.insert("b".to_string(), "2".to_string());

        let result = m.merge(&left, &right);
        assert_eq!(result.total_keys(), 2);
        assert!(!result.has_conflicts());
    }

    #[test]
    fn test_combiner_merge_right_wins() {
        let mut m = SettingsCombiner::new(CombinerConfig::new(CombineStrategy::RightWins));
        let mut left = HashMap::new();
        left.insert("key".to_string(), "left".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "right".to_string());

        let result = m.merge(&left, &right);
        assert_eq!(result.get("key"), Some(&"right".to_string()));
    }

    #[test]
    fn test_combiner_merge_left_wins() {
        let mut m = SettingsCombiner::new(CombinerConfig::new(CombineStrategy::LeftWins));
        let mut left = HashMap::new();
        left.insert("key".to_string(), "left".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "right".to_string());

        let result = m.merge(&left, &right);
        assert_eq!(result.get("key"), Some(&"left".to_string()));
    }

    #[test]
    fn test_combiner_merge_keep_both() {
        let mut m = SettingsCombiner::new(CombinerConfig::new(CombineStrategy::KeepBoth));
        let mut left = HashMap::new();
        left.insert("key".to_string(), "left".to_string());
        let mut right = HashMap::new();
        right.insert("key".to_string(), "right".to_string());

        let result = m.merge(&left, &right);
        assert!(result.get("key").is_some());
        assert!(result.get("key_conflict").is_some());
    }

    #[test]
    fn test_combiner_merge_all() {
        let mut m = SettingsCombiner::new(CombinerConfig::default());
        let mut c1 = HashMap::new();
        c1.insert("a".to_string(), "1".to_string());
        let mut c2 = HashMap::new();
        c2.insert("b".to_string(), "2".to_string());

        let result = m.merge_all(&[c1, c2]);
        assert_eq!(result.total_keys(), 2);
    }

    #[test]
    fn test_registry_new() {
        let r = CombinerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CombinerRegistry::new();
        r.register("m1", SettingsCombiner::new(CombinerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_combiner_query() {
        assert!(is_combiner_query("combine settings"));
        assert!(!is_combiner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = combiner_fun_fact();
        assert!(fact.contains("combiner"));
    }
}
