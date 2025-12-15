// v0.0.659: Settings Restorer (Phase 235)
// Restorer for recovering settings from archives

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

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

/// Restorer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorerConfig {
    /// Restore mode
    pub mode: RestoreMode,
    /// Restore strategy
    pub strategy: RestoreStrategy,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Validate before restore
    pub validate_before: bool,
    /// Create backup before restore
    pub backup_before: bool,
}

impl RestorerConfig {
    /// Create new config
    pub fn new(mode: RestoreMode) -> Self {
        Self {
            mode,
            strategy: RestoreStrategy::LatestFirst,
            category: None,
            validate_before: true,
            backup_before: true,
        }
    }

    /// Set strategy
    pub fn strategy(mut self, strategy: RestoreStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set validate before
    pub fn validate_before(mut self, validate: bool) -> Self {
        self.validate_before = validate;
        self
    }

    /// Set backup before
    pub fn backup_before(mut self, backup: bool) -> Self {
        self.backup_before = backup;
        self
    }
}

impl Default for RestorerConfig {
    fn default() -> Self {
        Self::new(RestoreMode::Full)
    }
}

/// Restore source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSource {
    /// Archive ID
    pub archive_id: String,
    /// Archive data
    pub data: String,
    /// Timestamp
    pub timestamp: u64,
    /// Priority
    pub priority: u32,
}

impl RestoreSource {
    /// Create new source
    pub fn new(archive_id: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            archive_id: archive_id.into(),
            data: data.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            priority: 0,
        }
    }

    /// With timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// With priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// Restore result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    /// Restored settings
    pub restored: HashMap<String, String>,
    /// Keys restored
    pub keys_restored: Vec<String>,
    /// Keys skipped
    pub keys_skipped: Vec<String>,
    /// Keys failed
    pub keys_failed: Vec<String>,
    /// Restore mode used
    pub mode: RestoreMode,
}

impl RestoreResult {
    /// Create new result
    pub fn new(mode: RestoreMode) -> Self {
        Self {
            restored: HashMap::new(),
            keys_restored: Vec::new(),
            keys_skipped: Vec::new(),
            keys_failed: Vec::new(),
            mode,
        }
    }

    /// Add restored
    pub fn add_restored(&mut self, key: String, value: String) {
        self.restored.insert(key.clone(), value);
        self.keys_restored.push(key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.keys_skipped.push(key);
    }

    /// Add failed
    pub fn add_failed(&mut self, key: String) {
        self.keys_failed.push(key);
    }

    /// Total restored
    pub fn total_restored(&self) -> usize {
        self.restored.len()
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.keys_failed.is_empty()
    }

    /// Success
    pub fn success(&self) -> bool {
        self.keys_failed.is_empty() && !self.keys_restored.is_empty()
    }
}

/// Restorer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RestorerStats {
    /// Total restores
    pub total_restores: usize,
    /// Total keys restored
    pub total_keys_restored: usize,
    /// Total keys failed
    pub total_keys_failed: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl RestorerStats {
    /// Record restore
    pub fn record(&mut self, mode: RestoreMode, keys_restored: usize, keys_failed: usize) {
        self.total_restores += 1;
        self.total_keys_restored += keys_restored;
        self.total_keys_failed += keys_failed;
        *self.by_mode.entry(mode.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_keys_restored + self.total_keys_failed;
        if total == 0 {
            0.0
        } else {
            self.total_keys_restored as f64 / total as f64
        }
    }
}

/// Settings restorer
#[derive(Debug, Clone, Default)]
pub struct SettingsRestorer {
    /// Config
    config: RestorerConfig,
    /// Results
    results: Vec<RestoreResult>,
    /// Stats
    stats: RestorerStats,
}

impl SettingsRestorer {
    /// Create new restorer
    pub fn new(config: RestorerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: RestorerStats::default(),
        }
    }

    /// Restore from archive data
    pub fn restore(&mut self, archive_data: &str) -> RestoreResult {
        let mut result = RestoreResult::new(self.config.mode);

        // Parse archive data (assume JSON format)
        match serde_json::from_str::<HashMap<String, String>>(archive_data) {
            Ok(settings) => {
                for (key, value) in settings {
                    result.add_restored(key, value);
                }
            }
            Err(_) => {
                // Try simple key=value format
                for line in archive_data.lines() {
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim().to_string();
                        let value = value.trim().trim_matches('"').to_string();
                        result.add_restored(key, value);
                    }
                }
            }
        }

        self.stats.record(
            self.config.mode,
            result.keys_restored.len(),
            result.keys_failed.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Restore from source
    pub fn restore_from_source(&mut self, source: &RestoreSource) -> RestoreResult {
        self.restore(&source.data)
    }

    /// Restore selective keys
    pub fn restore_keys(&mut self, archive_data: &str, keys: &[&str]) -> RestoreResult {
        let mut result = RestoreResult::new(RestoreMode::Selective);

        if let Ok(settings) = serde_json::from_str::<HashMap<String, String>>(archive_data) {
            for key in keys {
                if let Some(value) = settings.get(*key) {
                    result.add_restored(key.to_string(), value.clone());
                } else {
                    result.add_skipped(key.to_string());
                }
            }
        }

        self.stats.record(
            RestoreMode::Selective,
            result.keys_restored.len(),
            result.keys_failed.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[RestoreResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &RestorerStats {
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

/// Settings restorer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsRestorerRegistry {
    /// Restorers by ID
    restorers: HashMap<String, SettingsRestorer>,
}

impl SettingsRestorerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register restorer
    pub fn register(&mut self, id: impl Into<String>, restorer: SettingsRestorer) {
        self.restorers.insert(id.into(), restorer);
    }

    /// Unregister restorer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.restorers.remove(id).is_some()
    }

    /// Get restorer
    pub fn get(&self, id: &str) -> Option<&SettingsRestorer> {
        self.restorers.get(id)
    }

    /// Get restorer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRestorer> {
        self.restorers.get_mut(id)
    }

    /// Restorer count
    pub fn count(&self) -> usize {
        self.restorers.len()
    }
}

/// Format restorer registry
pub fn format_restorer_registry(registry: &SettingsRestorerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Restorer Registry:\n");
    output.push_str(&format!("  Restorers: {}\n", registry.count()));
    output
}

/// Check if query is about restorer
pub fn is_restorer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("restorer") || lower.contains("restore settings") || lower.contains("recover settings")
}

/// Fun fact about restorer
pub fn restorer_fun_fact() -> &'static str {
    "Anna's settings restorers recover your configs from backups!"
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

    #[test]
    fn test_config_new() {
        let c = RestorerConfig::new(RestoreMode::Full);
        assert!(c.validate_before);
    }

    #[test]
    fn test_config_builder() {
        let c = RestorerConfig::new(RestoreMode::Merge)
            .strategy(RestoreStrategy::OldestFirst)
            .backup_before(false);
        assert_eq!(c.strategy, RestoreStrategy::OldestFirst);
        assert!(!c.backup_before);
    }

    #[test]
    fn test_source_new() {
        let s = RestoreSource::new("archive_1", "{\"key\":\"value\"}");
        assert_eq!(s.archive_id, "archive_1");
    }

    #[test]
    fn test_source_with_priority() {
        let s = RestoreSource::new("archive_1", "data").with_priority(10);
        assert_eq!(s.priority, 10);
    }

    #[test]
    fn test_result_new() {
        let r = RestoreResult::new(RestoreMode::Full);
        assert_eq!(r.total_restored(), 0);
    }

    #[test]
    fn test_result_add_restored() {
        let mut r = RestoreResult::new(RestoreMode::Full);
        r.add_restored("key1".to_string(), "value1".to_string());
        assert_eq!(r.total_restored(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = RestorerStats::default();
        s.record(RestoreMode::Full, 10, 2);
        assert_eq!(s.total_restores, 1);
        assert_eq!(s.total_keys_restored, 10);
    }

    #[test]
    fn test_restorer_new() {
        let r = SettingsRestorer::new(RestorerConfig::new(RestoreMode::Full));
        assert_eq!(r.result_count(), 0);
    }

    #[test]
    fn test_restorer_restore_json() {
        let mut r = SettingsRestorer::new(RestorerConfig::new(RestoreMode::Full));
        let data = "{\"key1\":\"value1\",\"key2\":\"value2\"}";

        let result = r.restore(data);
        assert_eq!(result.total_restored(), 2);
    }

    #[test]
    fn test_restorer_restore_keyvalue() {
        let mut r = SettingsRestorer::new(RestorerConfig::new(RestoreMode::Full));
        let data = "key1 = \"value1\"\nkey2 = \"value2\"";

        let result = r.restore(data);
        assert_eq!(result.total_restored(), 2);
    }

    #[test]
    fn test_restorer_restore_keys() {
        let mut r = SettingsRestorer::new(RestorerConfig::new(RestoreMode::Selective));
        let data = "{\"key1\":\"value1\",\"key2\":\"value2\",\"key3\":\"value3\"}";

        let result = r.restore_keys(data, &["key1", "key3"]);
        assert_eq!(result.total_restored(), 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsRestorerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsRestorerRegistry::new();
        r.register("r1", SettingsRestorer::new(RestorerConfig::new(RestoreMode::Full)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_restorer_query() {
        assert!(is_restorer_query("settings restorer"));
        assert!(!is_restorer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = restorer_fun_fact();
        assert!(fact.contains("restorer"));
    }
}
