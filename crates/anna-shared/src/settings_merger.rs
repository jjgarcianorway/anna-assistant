// v0.0.655: Settings Merger (Phase 231)
// Merger for combining multiple settings sources

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MergeStrategy {
    /// First wins (keep first value)
    FirstWins,
    /// Last wins (keep last value)
    #[default]
    LastWins,
    /// Higher priority wins
    PriorityWins,
    /// Combine values
    Combine,
    /// Union (include all unique)
    Union,
}

impl std::fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FirstWins => write!(f, "first_wins"),
            Self::LastWins => write!(f, "last_wins"),
            Self::PriorityWins => write!(f, "priority_wins"),
            Self::Combine => write!(f, "combine"),
            Self::Union => write!(f, "union"),
        }
    }
}

/// Conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConflictResolution {
    /// Use strategy
    #[default]
    UseStrategy,
    /// Skip conflicting
    Skip,
    /// Fail on conflict
    Fail,
    /// Prompt user
    Prompt,
}

impl std::fmt::Display for ConflictResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UseStrategy => write!(f, "use_strategy"),
            Self::Skip => write!(f, "skip"),
            Self::Fail => write!(f, "fail"),
            Self::Prompt => write!(f, "prompt"),
        }
    }
}

/// Merger config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergerConfig {
    /// Merge strategy
    pub strategy: MergeStrategy,
    /// Conflict resolution
    pub conflict_resolution: ConflictResolution,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Deep merge nested
    pub deep_merge: bool,
    /// Preserve metadata
    pub preserve_metadata: bool,
}

impl MergerConfig {
    /// Create new config
    pub fn new(strategy: MergeStrategy) -> Self {
        Self {
            strategy,
            conflict_resolution: ConflictResolution::UseStrategy,
            category: None,
            deep_merge: true,
            preserve_metadata: false,
        }
    }

    /// Set conflict resolution
    pub fn conflict_resolution(mut self, resolution: ConflictResolution) -> Self {
        self.conflict_resolution = resolution;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set deep merge
    pub fn deep_merge(mut self, deep: bool) -> Self {
        self.deep_merge = deep;
        self
    }

    /// Set preserve metadata
    pub fn preserve_metadata(mut self, preserve: bool) -> Self {
        self.preserve_metadata = preserve;
        self
    }
}

impl Default for MergerConfig {
    fn default() -> Self {
        Self::new(MergeStrategy::LastWins)
    }
}

/// Merge source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSource {
    /// Source name
    pub name: String,
    /// Priority (higher = more important)
    pub priority: u32,
    /// Settings data
    pub settings: HashMap<String, String>,
}

impl MergeSource {
    /// Create new source
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority,
            settings: HashMap::new(),
        }
    }

    /// Add setting
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// With setting
    pub fn with_setting(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.add(key, value);
        self
    }

    /// Setting count
    pub fn setting_count(&self) -> usize {
        self.settings.len()
    }
}

/// Merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Merged settings
    pub merged: HashMap<String, String>,
    /// Keys from each source
    pub sources: HashMap<String, Vec<String>>,
    /// Conflicts encountered
    pub conflicts: Vec<String>,
    /// Strategy used
    pub strategy: MergeStrategy,
}

impl MergeResult {
    /// Create new result
    pub fn new(strategy: MergeStrategy) -> Self {
        Self {
            merged: HashMap::new(),
            sources: HashMap::new(),
            conflicts: Vec::new(),
            strategy,
        }
    }

    /// Add merged value
    pub fn add_merged(&mut self, key: String, value: String, source: &str) {
        self.merged.insert(key.clone(), value);
        self.sources
            .entry(source.to_string())
            .or_default()
            .push(key);
    }

    /// Add conflict
    pub fn add_conflict(&mut self, key: String) {
        self.conflicts.push(key);
    }

    /// Total merged
    pub fn total_merged(&self) -> usize {
        self.merged.len()
    }

    /// Has conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
}

/// Merger stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MergerStats {
    /// Total merges
    pub total_merges: usize,
    /// Total keys merged
    pub total_keys_merged: usize,
    /// Total conflicts
    pub total_conflicts: usize,
    /// By strategy
    pub by_strategy: HashMap<String, usize>,
}

impl MergerStats {
    /// Record merge
    pub fn record(&mut self, strategy: MergeStrategy, keys_merged: usize, conflicts: usize) {
        self.total_merges += 1;
        self.total_keys_merged += keys_merged;
        self.total_conflicts += conflicts;
        *self.by_strategy.entry(strategy.to_string()).or_insert(0) += 1;
    }

    /// Conflict rate
    pub fn conflict_rate(&self) -> f64 {
        if self.total_keys_merged == 0 {
            0.0
        } else {
            self.total_conflicts as f64 / self.total_keys_merged as f64
        }
    }
}

/// Settings merger
#[derive(Debug, Clone, Default)]
pub struct SettingsMerger {
    /// Config
    config: MergerConfig,
    /// Results
    results: Vec<MergeResult>,
    /// Stats
    stats: MergerStats,
}

impl SettingsMerger {
    /// Create new merger
    pub fn new(config: MergerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: MergerStats::default(),
        }
    }

    /// Merge sources
    pub fn merge(&mut self, sources: &[MergeSource]) -> MergeResult {
        let mut result = MergeResult::new(self.config.strategy);

        // Sort sources by priority if using priority strategy
        let mut sorted_sources: Vec<&MergeSource> = sources.iter().collect();
        if self.config.strategy == MergeStrategy::PriorityWins {
            sorted_sources.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        // Collect all keys
        let mut all_keys: HashMap<String, Vec<(&str, &str)>> = HashMap::new();
        for source in &sorted_sources {
            for (key, value) in &source.settings {
                all_keys
                    .entry(key.clone())
                    .or_default()
                    .push((&source.name, value));
            }
        }

        // Merge each key
        for (key, values) in all_keys {
            if values.len() > 1 {
                // Conflict
                match self.config.conflict_resolution {
                    ConflictResolution::Fail => {
                        result.add_conflict(key);
                        continue;
                    }
                    ConflictResolution::Skip => {
                        result.add_conflict(key);
                        continue;
                    }
                    ConflictResolution::Prompt => {
                        result.add_conflict(key.clone());
                        // Default to first value
                        let (source, value) = values[0];
                        result.add_merged(key, value.to_string(), source);
                    }
                    ConflictResolution::UseStrategy => {
                        let (source, value) = match self.config.strategy {
                            MergeStrategy::FirstWins => values[0],
                            MergeStrategy::LastWins => values[values.len() - 1],
                            MergeStrategy::PriorityWins => values[0], // Already sorted
                            MergeStrategy::Combine => {
                                let combined: String = values
                                    .iter()
                                    .map(|(_, v)| *v)
                                    .collect::<Vec<_>>()
                                    .join(",");
                                result.add_merged(key.clone(), combined, "combined");
                                continue;
                            }
                            MergeStrategy::Union => values[0], // Take first for union
                        };
                        result.add_merged(key, value.to_string(), source);
                    }
                }
            } else {
                // No conflict
                let (source, value) = values[0];
                result.add_merged(key, value.to_string(), source);
            }
        }

        self.stats.record(
            self.config.strategy,
            result.merged.len(),
            result.conflicts.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Merge two hashmaps directly
    pub fn merge_maps(
        &mut self,
        base: &HashMap<String, String>,
        overlay: &HashMap<String, String>,
    ) -> MergeResult {
        let sources = vec![
            MergeSource {
                name: "base".to_string(),
                priority: 1,
                settings: base.clone(),
            },
            MergeSource {
                name: "overlay".to_string(),
                priority: 2,
                settings: overlay.clone(),
            },
        ];
        self.merge(&sources)
    }

    /// Get results
    pub fn results(&self) -> &[MergeResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &MergerStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

/// Settings merger registry
#[derive(Debug, Clone, Default)]
pub struct SettingsMergerRegistry {
    /// Mergers by ID
    mergers: HashMap<String, SettingsMerger>,
}

impl SettingsMergerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register merger
    pub fn register(&mut self, id: impl Into<String>, merger: SettingsMerger) {
        self.mergers.insert(id.into(), merger);
    }

    /// Unregister merger
    pub fn unregister(&mut self, id: &str) -> bool {
        self.mergers.remove(id).is_some()
    }

    /// Get merger
    pub fn get(&self, id: &str) -> Option<&SettingsMerger> {
        self.mergers.get(id)
    }

    /// Get merger mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMerger> {
        self.mergers.get_mut(id)
    }

    /// Merger count
    pub fn count(&self) -> usize {
        self.mergers.len()
    }
}

/// Format merger registry
pub fn format_merger_registry(registry: &SettingsMergerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Merger Registry:\n");
    output.push_str(&format!("  Mergers: {}\n", registry.count()));
    output
}

/// Check if query is about merger
pub fn is_merger_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("merger") || lower.contains("merge settings") || lower.contains("combine settings")
}

/// Fun fact about merger
pub fn merger_fun_fact() -> &'static str {
    "Anna's settings mergers combine configs from multiple sources!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(format!("{}", MergeStrategy::FirstWins), "first_wins");
        assert_eq!(format!("{}", MergeStrategy::LastWins), "last_wins");
    }

    #[test]
    fn test_conflict_resolution_display() {
        assert_eq!(format!("{}", ConflictResolution::UseStrategy), "use_strategy");
        assert_eq!(format!("{}", ConflictResolution::Skip), "skip");
    }

    #[test]
    fn test_config_new() {
        let c = MergerConfig::new(MergeStrategy::FirstWins);
        assert!(c.deep_merge);
    }

    #[test]
    fn test_config_builder() {
        let c = MergerConfig::new(MergeStrategy::Union)
            .conflict_resolution(ConflictResolution::Fail)
            .deep_merge(false);
        assert_eq!(c.conflict_resolution, ConflictResolution::Fail);
        assert!(!c.deep_merge);
    }

    #[test]
    fn test_source_new() {
        let s = MergeSource::new("test", 10);
        assert_eq!(s.name, "test");
        assert_eq!(s.priority, 10);
    }

    #[test]
    fn test_source_with_setting() {
        let s = MergeSource::new("test", 1)
            .with_setting("key1", "value1")
            .with_setting("key2", "value2");
        assert_eq!(s.setting_count(), 2);
    }

    #[test]
    fn test_result_new() {
        let r = MergeResult::new(MergeStrategy::LastWins);
        assert_eq!(r.total_merged(), 0);
    }

    #[test]
    fn test_result_add() {
        let mut r = MergeResult::new(MergeStrategy::LastWins);
        r.add_merged("key1".to_string(), "value1".to_string(), "source1");
        assert_eq!(r.total_merged(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = MergerStats::default();
        s.record(MergeStrategy::LastWins, 10, 2);
        assert_eq!(s.total_merges, 1);
        assert_eq!(s.total_keys_merged, 10);
    }

    #[test]
    fn test_merger_new() {
        let m = SettingsMerger::new(MergerConfig::new(MergeStrategy::LastWins));
        assert_eq!(m.result_count(), 0);
    }

    #[test]
    fn test_merger_merge_no_conflict() {
        let mut m = SettingsMerger::new(MergerConfig::new(MergeStrategy::LastWins));
        let sources = vec![
            MergeSource::new("s1", 1).with_setting("key1", "value1"),
            MergeSource::new("s2", 2).with_setting("key2", "value2"),
        ];
        let r = m.merge(&sources);
        assert_eq!(r.total_merged(), 2);
        assert!(!r.has_conflicts());
    }

    #[test]
    fn test_merger_merge_last_wins() {
        let mut m = SettingsMerger::new(MergerConfig::new(MergeStrategy::LastWins));
        let sources = vec![
            MergeSource::new("s1", 1).with_setting("key", "first"),
            MergeSource::new("s2", 2).with_setting("key", "last"),
        ];
        let r = m.merge(&sources);
        assert_eq!(r.merged.get("key"), Some(&"last".to_string()));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsMergerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsMergerRegistry::new();
        r.register("m1", SettingsMerger::new(MergerConfig::new(MergeStrategy::FirstWins)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_merger_query() {
        assert!(is_merger_query("settings merger"));
        assert!(!is_merger_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = merger_fun_fact();
        assert!(fact.contains("merger"));
    }
}
