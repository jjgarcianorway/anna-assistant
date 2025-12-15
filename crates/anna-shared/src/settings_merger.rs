// v0.0.655: Settings Merger (Phase 231)
// Merger for combining multiple settings sources

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MergeStrategy {
    /// Override - later values win
    #[default]
    Override,
    /// Preserve - earlier values win
    Preserve,
    /// Append - combine values
    Append,
    /// Deep - recursive merge
    Deep,
}

impl std::fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Override => write!(f, "override"),
            Self::Preserve => write!(f, "preserve"),
            Self::Append => write!(f, "append"),
            Self::Deep => write!(f, "deep"),
        }
    }
}

/// Merge priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MergePriority {
    /// Low priority
    Low,
    /// Normal priority
    #[default]
    Normal,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl std::fmt::Display for MergePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Merger config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergerConfig {
    pub strategy: MergeStrategy,
    pub allow_override: bool,
    pub track_sources: bool,
}

impl Default for MergerConfig {
    fn default() -> Self {
        Self {
            strategy: MergeStrategy::Override,
            allow_override: true,
            track_sources: false,
        }
    }
}

/// Merge source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSource {
    pub name: String,
    pub priority: MergePriority,
    pub settings: HashMap<String, String>,
}

impl MergeSource {
    pub fn new(name: impl Into<String>, priority: MergePriority) -> Self {
        Self {
            name: name.into(),
            priority,
            settings: HashMap::new(),
        }
    }

    pub fn with_settings(mut self, settings: HashMap<String, String>) -> Self {
        self.settings = settings;
        self
    }
}

/// Merge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub merged: HashMap<String, String>,
    pub source_count: usize,
    pub conflicts: usize,
}

impl Default for MergeResult {
    fn default() -> Self {
        Self {
            merged: HashMap::new(),
            source_count: 0,
            conflicts: 0,
        }
    }
}

/// Settings merger
#[derive(Debug, Clone, Default)]
pub struct SettingsMerger {
    config: MergerConfig,
    sources: Vec<MergeSource>,
}

impl SettingsMerger {
    pub fn new(config: MergerConfig) -> Self {
        Self { config, sources: Vec::new() }
    }

    pub fn add_source(&mut self, source: MergeSource) {
        self.sources.push(source);
    }

    pub fn merge(&self) -> MergeResult {
        let mut merged = HashMap::new();
        let mut conflicts = 0;

        let mut sorted_sources = self.sources.clone();
        sorted_sources.sort_by_key(|s| match s.priority {
            MergePriority::Low => 0,
            MergePriority::Normal => 1,
            MergePriority::High => 2,
            MergePriority::Critical => 3,
        });

        for source in &sorted_sources {
            for (key, value) in &source.settings {
                if merged.contains_key(key) {
                    conflicts += 1;
                }
                match self.config.strategy {
                    MergeStrategy::Override => { merged.insert(key.clone(), value.clone()); }
                    MergeStrategy::Preserve => { merged.entry(key.clone()).or_insert_with(|| value.clone()); }
                    MergeStrategy::Append | MergeStrategy::Deep => { merged.insert(key.clone(), value.clone()); }
                }
            }
        }

        MergeResult {
            merged,
            source_count: self.sources.len(),
            conflicts,
        }
    }
}

/// Merger registry
#[derive(Debug, Clone, Default)]
pub struct MergerRegistry {
    mergers: HashMap<String, SettingsMerger>,
}

impl MergerRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, id: impl Into<String>, merger: SettingsMerger) {
        self.mergers.insert(id.into(), merger);
    }
    pub fn get(&self, id: &str) -> Option<&SettingsMerger> { self.mergers.get(id) }
    pub fn count(&self) -> usize { self.mergers.len() }
}

pub fn format_merger_registry(registry: &MergerRegistry) -> String {
    format!("Settings Merger Registry:\n  Mergers: {}\n", registry.count())
}

pub fn is_merger_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("merge settings") || lower.contains("settings merger")
}

pub fn merger_fun_fact() -> &'static str {
    "Anna's settings merger combines multiple configuration sources!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_display() {
        assert_eq!(format!("{}", MergeStrategy::Override), "override");
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", MergePriority::High), "high");
    }

    #[test]
    fn test_merger_new() {
        let m = SettingsMerger::new(MergerConfig::default());
        let result = m.merge();
        assert_eq!(result.source_count, 0);
    }

    #[test]
    fn test_merge_sources() {
        let mut m = SettingsMerger::new(MergerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());
        m.add_source(MergeSource::new("test", MergePriority::Normal).with_settings(settings));
        let result = m.merge();
        assert_eq!(result.merged.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_registry() {
        let mut r = MergerRegistry::new();
        r.register("m1", SettingsMerger::new(MergerConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
