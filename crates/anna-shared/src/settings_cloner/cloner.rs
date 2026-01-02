// v0.0.657: Settings Cloner Implementation (Phase 233)
// Core cloner logic for duplicating settings configurations

use std::collections::HashMap;

use super::result::{CloneResult, ClonerStats};
use super::types::{CloneDepth, CloneMod, ClonerConfig};

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
