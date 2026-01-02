// v0.0.683: Settings Zipper Implementation
// Core zipper functionality for combining and splitting settings

use std::collections::HashMap;
use super::config::ZipperConfig;
use super::types::{ZipMode, ZipResult, ZippedPair, ZipperStats, UnzipResult};

/// Settings zipper
#[derive(Debug, Clone, Default)]
pub struct SettingsZipper {
    /// Config
    config: ZipperConfig,
    /// Stats
    stats: ZipperStats,
}

impl SettingsZipper {
    /// Create new zipper
    pub fn new(config: ZipperConfig) -> Self {
        Self {
            config,
            stats: ZipperStats::default(),
        }
    }

    /// Zip by key
    pub fn zip_by_key(&mut self, first: &HashMap<String, String>, second: &HashMap<String, String>) -> ZipResult {
        let mut pairs = Vec::new();
        let mut matched = 0;
        let mut unmatched = 0;

        for (key, first_val) in first {
            if let Some(second_val) = second.get(key) {
                pairs.push(ZippedPair::new(key.clone(), first_val.clone(), second_val.clone()));
                matched += 1;
            } else {
                pairs.push(ZippedPair::new(key.clone(), first_val.clone(), self.config.default_value.clone()));
                unmatched += 1;
            }
        }

        // Add keys only in second
        for (key, second_val) in second {
            if !first.contains_key(key) {
                pairs.push(ZippedPair::new(key.clone(), self.config.default_value.clone(), second_val.clone()));
                unmatched += 1;
            }
        }

        let result = ZipResult::new(pairs, matched, unmatched, ZipMode::ByKey);
        self.stats.record_zip(&result);
        result
    }

    /// Zip by position
    pub fn zip_by_position(&mut self, first: &HashMap<String, String>, second: &HashMap<String, String>) -> ZipResult {
        let first_vec: Vec<_> = first.iter().collect();
        let second_vec: Vec<_> = second.iter().collect();
        let mut pairs = Vec::new();
        let matched = first_vec.len().min(second_vec.len());
        let unmatched = first_vec.len().max(second_vec.len()) - matched;

        for i in 0..first_vec.len().max(second_vec.len()) {
            let (key, first_val) = first_vec.get(i).map(|(k, v)| ((*k).clone(), (*v).clone()))
                .unwrap_or_else(|| (format!("key_{}", i), self.config.default_value.clone()));
            let second_val = second_vec.get(i).map(|(_, v)| (*v).clone())
                .unwrap_or_else(|| self.config.default_value.clone());
            pairs.push(ZippedPair::new(key, first_val, second_val));
        }

        let result = ZipResult::new(pairs, matched, unmatched, ZipMode::ByPosition);
        self.stats.record_zip(&result);
        result
    }

    /// Unzip by prefix
    pub fn unzip_by_prefix(&mut self, settings: &HashMap<String, String>, prefix: &str) -> UnzipResult {
        let mut first = HashMap::new();
        let mut second = HashMap::new();

        for (key, value) in settings {
            if key.starts_with(prefix) {
                first.insert(key.clone(), value.clone());
            } else {
                second.insert(key.clone(), value.clone());
            }
        }

        let result = UnzipResult::new(first, second);
        self.stats.record_unzip(&result);
        result
    }

    /// Unzip alternating
    pub fn unzip_alternating(&mut self, settings: &HashMap<String, String>) -> UnzipResult {
        let mut first = HashMap::new();
        let mut second = HashMap::new();

        for (i, (key, value)) in settings.iter().enumerate() {
            if i % 2 == 0 {
                first.insert(key.clone(), value.clone());
            } else {
                second.insert(key.clone(), value.clone());
            }
        }

        let result = UnzipResult::new(first, second);
        self.stats.record_unzip(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &ZipperStats {
        &self.stats
    }
}
