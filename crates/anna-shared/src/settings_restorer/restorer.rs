// v0.0.659: Settings Restorer - Core Restorer
// Main settings restorer implementation

use std::collections::HashMap;

use super::config::RestorerConfig;
use super::mode::RestoreMode;
use super::result::RestoreResult;
use super::source::RestoreSource;
use super::stats::RestorerStats;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
