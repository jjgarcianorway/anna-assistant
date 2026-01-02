// v0.0.658: Settings Archiver Core (Phase 234)
// Main archiver implementation

use std::collections::HashMap;

use super::config::ArchiverConfig;
use super::metadata::{ArchiveMetadata, ArchiveResult};
use super::stats::ArchiverStats;
use super::types::{ArchiveFormat, ArchiveType};

/// Settings archiver
#[derive(Debug, Clone, Default)]
pub struct SettingsArchiver {
    /// Config
    config: ArchiverConfig,
    /// Results
    results: Vec<ArchiveResult>,
    /// Stats
    stats: ArchiverStats,
    /// Next archive ID
    next_id: usize,
}

impl SettingsArchiver {
    /// Create new archiver
    pub fn new(config: ArchiverConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: ArchiverStats::default(),
            next_id: 1,
        }
    }

    /// Archive settings
    pub fn archive(&mut self, settings: &HashMap<String, String>) -> ArchiveResult {
        let archive_id = format!("archive_{}", self.next_id);
        self.next_id += 1;

        let metadata = ArchiveMetadata::new(&archive_id, self.config.archive_type, self.config.format)
            .with_key_count(settings.len());

        let mut result = ArchiveResult::new(metadata);

        // Serialize settings based on format
        let data = match self.config.format {
            ArchiveFormat::Json => {
                serde_json::to_string_pretty(settings).unwrap_or_default()
            }
            ArchiveFormat::Toml => {
                // Simple TOML-like format
                let mut toml_str = String::new();
                for (k, v) in settings {
                    toml_str.push_str(&format!("{} = \"{}\"\n", k, v));
                }
                toml_str
            }
            ArchiveFormat::Binary => {
                // Simulate binary format with base64-like encoding
                format!("BINARY:{}", settings.len())
            }
            ArchiveFormat::Compressed => {
                // Simulate compression
                format!("COMPRESSED:{}", settings.len())
            }
        };

        result.data = data.clone();

        for key in settings.keys() {
            result.add_archived(key.clone());
        }

        result.metadata.key_count = result.total_archived();

        self.stats.record(
            self.config.format,
            result.total_archived(),
            result.data.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Create snapshot
    pub fn snapshot(&mut self, settings: &HashMap<String, String>, description: &str) -> ArchiveResult {
        let archive_id = format!("snapshot_{}", self.next_id);
        self.next_id += 1;

        let metadata = ArchiveMetadata::new(&archive_id, ArchiveType::Snapshot, self.config.format)
            .with_key_count(settings.len())
            .with_description(description);

        let mut result = ArchiveResult::new(metadata);
        let data = serde_json::to_string_pretty(settings).unwrap_or_default();
        result.data = data.clone();

        for key in settings.keys() {
            result.add_archived(key.clone());
        }

        self.stats.record(
            self.config.format,
            result.total_archived(),
            result.data.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[ArchiveResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &ArchiverStats {
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
    fn test_archiver_new() {
        let a = SettingsArchiver::new(ArchiverConfig::new(ArchiveFormat::Json));
        assert_eq!(a.result_count(), 0);
    }

    #[test]
    fn test_archiver_archive() {
        let mut a = SettingsArchiver::new(ArchiverConfig::new(ArchiveFormat::Json));
        let mut settings = HashMap::new();
        settings.insert("key1".to_string(), "value1".to_string());
        settings.insert("key2".to_string(), "value2".to_string());

        let r = a.archive(&settings);
        assert_eq!(r.total_archived(), 2);
        assert!(!r.data.is_empty());
    }

    #[test]
    fn test_archiver_snapshot() {
        let mut a = SettingsArchiver::new(ArchiverConfig::new(ArchiveFormat::Json));
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());

        let r = a.snapshot(&settings, "test snapshot");
        assert_eq!(r.metadata.archive_type, ArchiveType::Snapshot);
    }
}
