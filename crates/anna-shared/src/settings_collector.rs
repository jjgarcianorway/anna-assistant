// v0.0.682: Settings Collector (Phase 258)
// Collect settings from multiple sources

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Collector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Collect mode
    pub mode: CollectMode,
    /// Dedup keys
    pub dedup_keys: bool,
    /// Append suffix
    pub append_suffix: String,
    /// Respect priority
    pub respect_priority: bool,
}

impl CollectorConfig {
    /// Create new config
    pub fn new(mode: CollectMode) -> Self {
        Self {
            mode,
            dedup_keys: true,
            append_suffix: "_".to_string(),
            respect_priority: true,
        }
    }

    /// Set dedup keys
    pub fn dedup_keys(mut self, dedup: bool) -> Self {
        self.dedup_keys = dedup;
        self
    }

    /// Set append suffix
    pub fn append_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.append_suffix = suffix.into();
        self
    }

    /// Set respect priority
    pub fn respect_priority(mut self, respect: bool) -> Self {
        self.respect_priority = respect;
        self
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self::new(CollectMode::Merge)
    }
}

/// Settings source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSource {
    /// Source ID
    pub id: String,
    /// Source name
    pub name: String,
    /// Priority
    pub priority: SourcePriority,
    /// Settings
    pub settings: HashMap<String, String>,
}

impl SettingsSource {
    /// Create new source
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            priority: SourcePriority::Normal,
            settings: HashMap::new(),
        }
    }

    /// With priority
    pub fn with_priority(mut self, priority: SourcePriority) -> Self {
        self.priority = priority;
        self
    }

    /// With settings
    pub fn with_settings(mut self, settings: HashMap<String, String>) -> Self {
        self.settings = settings;
        self
    }

    /// Add setting
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Count
    pub fn count(&self) -> usize {
        self.settings.len()
    }
}

/// Collect result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectResult {
    /// Collected settings
    pub settings: HashMap<String, String>,
    /// Sources processed
    pub sources_processed: usize,
    /// Keys collected
    pub keys_collected: usize,
    /// Conflicts resolved
    pub conflicts_resolved: usize,
    /// Mode used
    pub mode: CollectMode,
}

impl CollectResult {
    /// Create new result
    pub fn new(settings: HashMap<String, String>, sources: usize, conflicts: usize, mode: CollectMode) -> Self {
        let keys_collected = settings.len();
        Self {
            settings,
            sources_processed: sources,
            keys_collected,
            conflicts_resolved: conflicts,
            mode,
        }
    }

    /// Get value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Has conflicts
    pub fn had_conflicts(&self) -> bool {
        self.conflicts_resolved > 0
    }
}

impl Default for CollectResult {
    fn default() -> Self {
        Self::new(HashMap::new(), 0, 0, CollectMode::Merge)
    }
}

/// Collector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectorStats {
    /// Total collections
    pub total_collections: usize,
    /// Total sources
    pub total_sources: usize,
    /// Total keys
    pub total_keys: usize,
    /// Total conflicts
    pub total_conflicts: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl CollectorStats {
    /// Record collection
    pub fn record(&mut self, result: &CollectResult) {
        self.total_collections += 1;
        self.total_sources += result.sources_processed;
        self.total_keys += result.keys_collected;
        self.total_conflicts += result.conflicts_resolved;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Average keys per collection
    pub fn average_keys(&self) -> f64 {
        if self.total_collections == 0 {
            0.0
        } else {
            self.total_keys as f64 / self.total_collections as f64
        }
    }
}

/// Settings collector
#[derive(Debug, Clone, Default)]
pub struct SettingsCollector {
    /// Config
    config: CollectorConfig,
    /// Stats
    stats: CollectorStats,
    /// Sources
    sources: Vec<SettingsSource>,
}

impl SettingsCollector {
    /// Create new collector
    pub fn new(config: CollectorConfig) -> Self {
        Self {
            config,
            stats: CollectorStats::default(),
            sources: Vec::new(),
        }
    }

    /// Add source
    pub fn add_source(&mut self, source: SettingsSource) {
        self.sources.push(source);
    }

    /// Clear sources
    pub fn clear_sources(&mut self) {
        self.sources.clear();
    }

    /// Source count
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Collect all sources
    pub fn collect(&mut self) -> CollectResult {
        let mut collected = HashMap::new();
        let mut conflicts = 0;

        // Sort by priority if configured
        let mut sources = self.sources.clone();
        if self.config.respect_priority {
            sources.sort_by(|a, b| a.priority.cmp(&b.priority));
        }

        match self.config.mode {
            CollectMode::Merge => {
                for source in &sources {
                    for (key, value) in &source.settings {
                        if collected.contains_key(key) {
                            conflicts += 1;
                        }
                        collected.insert(key.clone(), value.clone());
                    }
                }
            }
            CollectMode::Union => {
                for source in &sources {
                    for (key, value) in &source.settings {
                        if !collected.contains_key(key) {
                            collected.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            CollectMode::Intersect => {
                if let Some(first) = sources.first() {
                    for (key, value) in &first.settings {
                        if sources.iter().skip(1).all(|s| s.settings.contains_key(key)) {
                            collected.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            CollectMode::Append => {
                for source in &sources {
                    for (key, value) in &source.settings {
                        let mut final_key = key.clone();
                        let mut counter = 1;
                        while collected.contains_key(&final_key) {
                            final_key = format!("{}{}{}", key, self.config.append_suffix, counter);
                            counter += 1;
                        }
                        collected.insert(final_key, value.clone());
                    }
                }
            }
        }

        let result = CollectResult::new(collected, sources.len(), conflicts, self.config.mode);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &CollectorStats {
        &self.stats
    }
}

/// Collector registry
#[derive(Debug, Clone, Default)]
pub struct CollectorRegistry {
    /// Collectors by ID
    collectors: HashMap<String, SettingsCollector>,
}

impl CollectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register collector
    pub fn register(&mut self, id: impl Into<String>, collector: SettingsCollector) {
        self.collectors.insert(id.into(), collector);
    }

    /// Unregister collector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.collectors.remove(id).is_some()
    }

    /// Get collector
    pub fn get(&self, id: &str) -> Option<&SettingsCollector> {
        self.collectors.get(id)
    }

    /// Get collector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCollector> {
        self.collectors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.collectors.len()
    }
}

/// Format collector registry
pub fn format_collector_registry(registry: &CollectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Collector Registry:\n");
    output.push_str(&format!("  Collectors: {}\n", registry.count()));
    output
}

/// Check if query is about collector
pub fn is_collector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("collect settings") || lower.contains("settings collector") || lower.contains("gather settings")
}

/// Fun fact about collector
pub fn collector_fun_fact() -> &'static str {
    "Anna's settings collector gathers settings from multiple sources into one unified view!"
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

    #[test]
    fn test_config_new() {
        let c = CollectorConfig::new(CollectMode::Merge);
        assert!(c.dedup_keys);
    }

    #[test]
    fn test_config_builder() {
        let c = CollectorConfig::new(CollectMode::Append)
            .append_suffix("_dup")
            .respect_priority(false);
        assert_eq!(c.append_suffix, "_dup");
        assert!(!c.respect_priority);
    }

    #[test]
    fn test_source_new() {
        let s = SettingsSource::new("s1", "Source 1");
        assert_eq!(s.id, "s1");
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_source_add() {
        let mut s = SettingsSource::new("s1", "Source 1");
        s.add("key", "value");
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn test_source_with_priority() {
        let s = SettingsSource::new("s1", "Source 1")
            .with_priority(SourcePriority::High);
        assert_eq!(s.priority, SourcePriority::High);
    }

    #[test]
    fn test_result_new() {
        let mut settings = HashMap::new();
        settings.insert("k".to_string(), "v".to_string());
        let r = CollectResult::new(settings, 2, 1, CollectMode::Merge);
        assert_eq!(r.keys_collected, 1);
        assert!(r.had_conflicts());
    }

    #[test]
    fn test_stats_record() {
        let mut s = CollectorStats::default();
        let r = CollectResult::new(HashMap::new(), 2, 0, CollectMode::Merge);
        s.record(&r);
        assert_eq!(s.total_collections, 1);
        assert_eq!(s.total_sources, 2);
    }

    #[test]
    fn test_collector_new() {
        let c = SettingsCollector::new(CollectorConfig::default());
        assert_eq!(c.source_count(), 0);
    }

    #[test]
    fn test_collector_add_source() {
        let mut c = SettingsCollector::new(CollectorConfig::default());
        c.add_source(SettingsSource::new("s1", "Source 1"));
        assert_eq!(c.source_count(), 1);
    }

    #[test]
    fn test_collector_merge() {
        let mut c = SettingsCollector::new(CollectorConfig::new(CollectMode::Merge));

        let mut s1 = SettingsSource::new("s1", "Source 1");
        s1.add("a", "1");
        s1.add("b", "2");

        let mut s2 = SettingsSource::new("s2", "Source 2");
        s2.add("b", "3");
        s2.add("c", "4");

        c.add_source(s1);
        c.add_source(s2);

        let result = c.collect();
        assert_eq!(result.keys_collected, 3);
        assert_eq!(result.get("b").unwrap(), "3"); // s2 overwrites
    }

    #[test]
    fn test_collector_union() {
        let mut c = SettingsCollector::new(CollectorConfig::new(CollectMode::Union));

        let mut s1 = SettingsSource::new("s1", "Source 1");
        s1.add("a", "1");

        let mut s2 = SettingsSource::new("s2", "Source 2");
        s2.add("a", "2");

        c.add_source(s1);
        c.add_source(s2);

        let result = c.collect();
        assert_eq!(result.get("a").unwrap(), "1"); // s1 wins (first)
    }

    #[test]
    fn test_registry_new() {
        let r = CollectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CollectorRegistry::new();
        r.register("c1", SettingsCollector::new(CollectorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_collector_query() {
        assert!(is_collector_query("collect settings"));
        assert!(!is_collector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = collector_fun_fact();
        assert!(fact.contains("collector"));
    }
}
