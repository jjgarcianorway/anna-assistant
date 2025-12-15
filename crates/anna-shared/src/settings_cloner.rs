// v0.0.657: Settings Cloner (Phase 233)
// Cloner for duplicating settings configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Clone depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CloneDepth {
    /// Shallow clone (top level only)
    Shallow,
    /// Deep clone (all nested)
    #[default]
    Deep,
    /// Selective clone
    Selective,
    /// Reference clone (copy references)
    Reference,
}

impl std::fmt::Display for CloneDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shallow => write!(f, "shallow"),
            Self::Deep => write!(f, "deep"),
            Self::Selective => write!(f, "selective"),
            Self::Reference => write!(f, "reference"),
        }
    }
}

/// Clone mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CloneMode {
    /// Exact copy
    #[default]
    Exact,
    /// With modifications
    WithMods,
    /// Template-based
    Template,
    /// Incremental
    Incremental,
}

impl std::fmt::Display for CloneMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "exact"),
            Self::WithMods => write!(f, "with_mods"),
            Self::Template => write!(f, "template"),
            Self::Incremental => write!(f, "incremental"),
        }
    }
}

/// Cloner config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClonerConfig {
    /// Clone depth
    pub depth: CloneDepth,
    /// Clone mode
    pub mode: CloneMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Prefix for cloned keys
    pub prefix: Option<String>,
    /// Suffix for cloned keys
    pub suffix: Option<String>,
}

impl ClonerConfig {
    /// Create new config
    pub fn new(depth: CloneDepth) -> Self {
        Self {
            depth,
            mode: CloneMode::Exact,
            category: None,
            prefix: None,
            suffix: None,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: CloneMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set prefix
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set suffix
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
}

impl Default for ClonerConfig {
    fn default() -> Self {
        Self::new(CloneDepth::Deep)
    }
}

/// Clone modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneMod {
    /// Key pattern
    pub key_pattern: String,
    /// New value (if Some, replace value)
    pub new_value: Option<String>,
    /// Transform fn name
    pub transform: Option<String>,
}

impl CloneMod {
    /// Create new modification
    pub fn new(key_pattern: impl Into<String>) -> Self {
        Self {
            key_pattern: key_pattern.into(),
            new_value: None,
            transform: None,
        }
    }

    /// With new value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.new_value = Some(value.into());
        self
    }

    /// With transform
    pub fn with_transform(mut self, transform: impl Into<String>) -> Self {
        self.transform = Some(transform.into());
        self
    }
}

/// Clone result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneResult {
    /// Cloned settings
    pub cloned: HashMap<String, String>,
    /// Keys cloned
    pub keys_cloned: Vec<String>,
    /// Keys skipped
    pub keys_skipped: Vec<String>,
    /// Clone depth used
    pub depth: CloneDepth,
}

impl CloneResult {
    /// Create new result
    pub fn new(depth: CloneDepth) -> Self {
        Self {
            cloned: HashMap::new(),
            keys_cloned: Vec::new(),
            keys_skipped: Vec::new(),
            depth,
        }
    }

    /// Add cloned
    pub fn add_cloned(&mut self, original_key: String, new_key: String, value: String) {
        self.cloned.insert(new_key, value);
        self.keys_cloned.push(original_key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.keys_skipped.push(key);
    }

    /// Total cloned
    pub fn total_cloned(&self) -> usize {
        self.cloned.len()
    }

    /// Has skipped
    pub fn has_skipped(&self) -> bool {
        !self.keys_skipped.is_empty()
    }
}

/// Cloner stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClonerStats {
    /// Total clones
    pub total_clones: usize,
    /// Total keys cloned
    pub total_keys_cloned: usize,
    /// Total keys skipped
    pub total_keys_skipped: usize,
    /// By depth
    pub by_depth: HashMap<String, usize>,
}

impl ClonerStats {
    /// Record clone
    pub fn record(&mut self, depth: CloneDepth, keys_cloned: usize, keys_skipped: usize) {
        self.total_clones += 1;
        self.total_keys_cloned += keys_cloned;
        self.total_keys_skipped += keys_skipped;
        *self.by_depth.entry(depth.to_string()).or_insert(0) += 1;
    }

    /// Clone success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_keys_cloned + self.total_keys_skipped;
        if total == 0 {
            0.0
        } else {
            self.total_keys_cloned as f64 / total as f64
        }
    }
}

/// Settings cloner
#[derive(Debug, Clone, Default)]
pub struct SettingsCloner {
    /// Config
    config: ClonerConfig,
    /// Modifications
    mods: Vec<CloneMod>,
    /// Results
    results: Vec<CloneResult>,
    /// Stats
    stats: ClonerStats,
}

impl SettingsCloner {
    /// Create new cloner
    pub fn new(config: ClonerConfig) -> Self {
        Self {
            config,
            mods: Vec::new(),
            results: Vec::new(),
            stats: ClonerStats::default(),
        }
    }

    /// Add modification
    pub fn add_mod(&mut self, modification: CloneMod) {
        self.mods.push(modification);
    }

    /// Clone settings
    pub fn clone_settings(&mut self, source: &HashMap<String, String>) -> CloneResult {
        let mut result = CloneResult::new(self.config.depth);

        for (key, value) in source {
            // Check if key matches any selective filter
            if self.config.depth == CloneDepth::Selective && !self.should_clone(key) {
                result.add_skipped(key.clone());
                continue;
            }

            // Apply key transformations
            let new_key = self.transform_key(key);

            // Apply value modifications
            let new_value = self.apply_mods(key, value);

            result.add_cloned(key.clone(), new_key, new_value);
        }

        self.stats.record(
            self.config.depth,
            result.keys_cloned.len(),
            result.keys_skipped.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Check if key should be cloned
    fn should_clone(&self, key: &str) -> bool {
        // Check if any mod pattern matches
        for m in &self.mods {
            if key.contains(&m.key_pattern) {
                return true;
            }
        }
        // If no mods, clone all
        self.mods.is_empty()
    }

    /// Transform key with prefix/suffix
    fn transform_key(&self, key: &str) -> String {
        let mut new_key = key.to_string();

        if let Some(prefix) = &self.config.prefix {
            new_key = format!("{}{}", prefix, new_key);
        }

        if let Some(suffix) = &self.config.suffix {
            new_key = format!("{}{}", new_key, suffix);
        }

        new_key
    }

    /// Apply modifications to value
    fn apply_mods(&self, key: &str, value: &str) -> String {
        for m in &self.mods {
            if key.contains(&m.key_pattern) {
                if let Some(new_val) = &m.new_value {
                    return new_val.clone();
                }
                if let Some(transform) = &m.transform {
                    return self.apply_transform(value, transform);
                }
            }
        }
        value.to_string()
    }

    /// Apply named transform
    fn apply_transform(&self, value: &str, transform: &str) -> String {
        match transform {
            "uppercase" => value.to_uppercase(),
            "lowercase" => value.to_lowercase(),
            "trim" => value.trim().to_string(),
            "reverse" => value.chars().rev().collect(),
            _ => value.to_string(),
        }
    }

    /// Clone with new name
    pub fn clone_as(&mut self, source: &HashMap<String, String>, name: &str) -> CloneResult {
        let original_prefix = self.config.prefix.clone();
        self.config.prefix = Some(format!("{}_", name));
        let result = self.clone_settings(source);
        self.config.prefix = original_prefix;
        result
    }

    /// Get results
    pub fn results(&self) -> &[CloneResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &ClonerStats {
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

/// Settings cloner registry
#[derive(Debug, Clone, Default)]
pub struct SettingsClonerRegistry {
    /// Cloners by ID
    cloners: HashMap<String, SettingsCloner>,
}

impl SettingsClonerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register cloner
    pub fn register(&mut self, id: impl Into<String>, cloner: SettingsCloner) {
        self.cloners.insert(id.into(), cloner);
    }

    /// Unregister cloner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.cloners.remove(id).is_some()
    }

    /// Get cloner
    pub fn get(&self, id: &str) -> Option<&SettingsCloner> {
        self.cloners.get(id)
    }

    /// Get cloner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCloner> {
        self.cloners.get_mut(id)
    }

    /// Cloner count
    pub fn count(&self) -> usize {
        self.cloners.len()
    }
}

/// Format cloner registry
pub fn format_cloner_registry(registry: &SettingsClonerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Cloner Registry:\n");
    output.push_str(&format!("  Cloners: {}\n", registry.count()));
    output
}

/// Check if query is about cloner
pub fn is_cloner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("cloner") || lower.contains("clone settings") || lower.contains("duplicate settings")
}

/// Fun fact about cloner
pub fn cloner_fun_fact() -> &'static str {
    "Anna's settings cloners duplicate configs with smart transformations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_depth_display() {
        assert_eq!(format!("{}", CloneDepth::Shallow), "shallow");
        assert_eq!(format!("{}", CloneDepth::Deep), "deep");
    }

    #[test]
    fn test_clone_mode_display() {
        assert_eq!(format!("{}", CloneMode::Exact), "exact");
        assert_eq!(format!("{}", CloneMode::WithMods), "with_mods");
    }

    #[test]
    fn test_config_new() {
        let c = ClonerConfig::new(CloneDepth::Shallow);
        assert_eq!(c.depth, CloneDepth::Shallow);
    }

    #[test]
    fn test_config_builder() {
        let c = ClonerConfig::new(CloneDepth::Deep)
            .mode(CloneMode::Template)
            .prefix("test_");
        assert_eq!(c.mode, CloneMode::Template);
        assert_eq!(c.prefix, Some("test_".to_string()));
    }

    #[test]
    fn test_mod_new() {
        let m = CloneMod::new("pattern");
        assert_eq!(m.key_pattern, "pattern");
    }

    #[test]
    fn test_mod_with_value() {
        let m = CloneMod::new("key").with_value("new_value");
        assert_eq!(m.new_value, Some("new_value".to_string()));
    }

    #[test]
    fn test_result_new() {
        let r = CloneResult::new(CloneDepth::Deep);
        assert_eq!(r.total_cloned(), 0);
    }

    #[test]
    fn test_result_add_cloned() {
        let mut r = CloneResult::new(CloneDepth::Deep);
        r.add_cloned("old_key".to_string(), "new_key".to_string(), "value".to_string());
        assert_eq!(r.total_cloned(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ClonerStats::default();
        s.record(CloneDepth::Deep, 10, 2);
        assert_eq!(s.total_clones, 1);
        assert_eq!(s.total_keys_cloned, 10);
    }

    #[test]
    fn test_cloner_new() {
        let c = SettingsCloner::new(ClonerConfig::new(CloneDepth::Deep));
        assert_eq!(c.result_count(), 0);
    }

    #[test]
    fn test_cloner_clone_settings() {
        let mut c = SettingsCloner::new(ClonerConfig::new(CloneDepth::Deep));
        let mut source = HashMap::new();
        source.insert("key1".to_string(), "value1".to_string());
        source.insert("key2".to_string(), "value2".to_string());

        let r = c.clone_settings(&source);
        assert_eq!(r.total_cloned(), 2);
    }

    #[test]
    fn test_cloner_with_prefix() {
        let mut c = SettingsCloner::new(ClonerConfig::new(CloneDepth::Deep).prefix("clone_"));
        let mut source = HashMap::new();
        source.insert("key".to_string(), "value".to_string());

        let r = c.clone_settings(&source);
        assert!(r.cloned.contains_key("clone_key"));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsClonerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsClonerRegistry::new();
        r.register("c1", SettingsCloner::new(ClonerConfig::new(CloneDepth::Shallow)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_cloner_query() {
        assert!(is_cloner_query("settings cloner"));
        assert!(!is_cloner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = cloner_fun_fact();
        assert!(fact.contains("cloner"));
    }
}
