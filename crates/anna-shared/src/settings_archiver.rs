// v0.0.658: Settings Archiver (Phase 234)
// Archiver for backing up settings configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Archive format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ArchiveFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// Binary format
    Binary,
    /// Compressed format
    Compressed,
}

impl std::fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Binary => write!(f, "binary"),
            Self::Compressed => write!(f, "compressed"),
        }
    }
}

/// Archive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArchiveType {
    /// Full backup
    #[default]
    Full,
    /// Incremental backup
    Incremental,
    /// Differential backup
    Differential,
    /// Snapshot
    Snapshot,
}

impl std::fmt::Display for ArchiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Incremental => write!(f, "incremental"),
            Self::Differential => write!(f, "differential"),
            Self::Snapshot => write!(f, "snapshot"),
        }
    }
}

/// Archiver config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiverConfig {
    /// Archive format
    pub format: ArchiveFormat,
    /// Archive type
    pub archive_type: ArchiveType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include metadata
    pub include_metadata: bool,
    /// Compress output
    pub compress: bool,
}

impl ArchiverConfig {
    /// Create new config
    pub fn new(format: ArchiveFormat) -> Self {
        Self {
            format,
            archive_type: ArchiveType::Full,
            category: None,
            include_metadata: true,
            compress: false,
        }
    }

    /// Set archive type
    pub fn archive_type(mut self, archive_type: ArchiveType) -> Self {
        self.archive_type = archive_type;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include metadata
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set compress
    pub fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
}

impl Default for ArchiverConfig {
    fn default() -> Self {
        Self::new(ArchiveFormat::Json)
    }
}

/// Archive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    /// Archive ID
    pub id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Archive type
    pub archive_type: ArchiveType,
    /// Format
    pub format: ArchiveFormat,
    /// Key count
    pub key_count: usize,
    /// Description
    pub description: Option<String>,
}

impl ArchiveMetadata {
    /// Create new metadata
    pub fn new(id: impl Into<String>, archive_type: ArchiveType, format: ArchiveFormat) -> Self {
        Self {
            id: id.into(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            archive_type,
            format,
            key_count: 0,
            description: None,
        }
    }

    /// With description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set key count
    pub fn with_key_count(mut self, count: usize) -> Self {
        self.key_count = count;
        self
    }
}

/// Archive result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveResult {
    /// Archive data (serialized)
    pub data: String,
    /// Metadata
    pub metadata: ArchiveMetadata,
    /// Keys archived
    pub keys_archived: Vec<String>,
    /// Keys skipped
    pub keys_skipped: Vec<String>,
}

impl ArchiveResult {
    /// Create new result
    pub fn new(metadata: ArchiveMetadata) -> Self {
        Self {
            data: String::new(),
            metadata,
            keys_archived: Vec::new(),
            keys_skipped: Vec::new(),
        }
    }

    /// Set data
    pub fn with_data(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    /// Add archived key
    pub fn add_archived(&mut self, key: String) {
        self.keys_archived.push(key);
    }

    /// Add skipped key
    pub fn add_skipped(&mut self, key: String) {
        self.keys_skipped.push(key);
    }

    /// Total archived
    pub fn total_archived(&self) -> usize {
        self.keys_archived.len()
    }

    /// Has skipped
    pub fn has_skipped(&self) -> bool {
        !self.keys_skipped.is_empty()
    }
}

/// Archiver stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiverStats {
    /// Total archives created
    pub total_archives: usize,
    /// Total keys archived
    pub total_keys_archived: usize,
    /// Total data size (bytes)
    pub total_data_size: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
}

impl ArchiverStats {
    /// Record archive
    pub fn record(&mut self, format: ArchiveFormat, keys_archived: usize, data_size: usize) {
        self.total_archives += 1;
        self.total_keys_archived += keys_archived;
        self.total_data_size += data_size;
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
    }

    /// Average archive size
    pub fn average_archive_size(&self) -> f64 {
        if self.total_archives == 0 {
            0.0
        } else {
            self.total_data_size as f64 / self.total_archives as f64
        }
    }
}

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

/// Settings archiver registry
#[derive(Debug, Clone, Default)]
pub struct SettingsArchiverRegistry {
    /// Archivers by ID
    archivers: HashMap<String, SettingsArchiver>,
}

impl SettingsArchiverRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register archiver
    pub fn register(&mut self, id: impl Into<String>, archiver: SettingsArchiver) {
        self.archivers.insert(id.into(), archiver);
    }

    /// Unregister archiver
    pub fn unregister(&mut self, id: &str) -> bool {
        self.archivers.remove(id).is_some()
    }

    /// Get archiver
    pub fn get(&self, id: &str) -> Option<&SettingsArchiver> {
        self.archivers.get(id)
    }

    /// Get archiver mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsArchiver> {
        self.archivers.get_mut(id)
    }

    /// Archiver count
    pub fn count(&self) -> usize {
        self.archivers.len()
    }
}

/// Format archiver registry
pub fn format_archiver_registry(registry: &SettingsArchiverRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Archiver Registry:\n");
    output.push_str(&format!("  Archivers: {}\n", registry.count()));
    output
}

/// Check if query is about archiver
pub fn is_archiver_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("archiver") || lower.contains("archive settings") || lower.contains("backup settings")
}

/// Fun fact about archiver
pub fn archiver_fun_fact() -> &'static str {
    "Anna's settings archivers create safe backups of your configs!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_format_display() {
        assert_eq!(format!("{}", ArchiveFormat::Json), "json");
        assert_eq!(format!("{}", ArchiveFormat::Toml), "toml");
    }

    #[test]
    fn test_archive_type_display() {
        assert_eq!(format!("{}", ArchiveType::Full), "full");
        assert_eq!(format!("{}", ArchiveType::Incremental), "incremental");
    }

    #[test]
    fn test_config_new() {
        let c = ArchiverConfig::new(ArchiveFormat::Json);
        assert!(c.include_metadata);
    }

    #[test]
    fn test_config_builder() {
        let c = ArchiverConfig::new(ArchiveFormat::Toml)
            .archive_type(ArchiveType::Snapshot)
            .compress(true);
        assert_eq!(c.archive_type, ArchiveType::Snapshot);
        assert!(c.compress);
    }

    #[test]
    fn test_metadata_new() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json);
        assert_eq!(m.id, "test");
    }

    #[test]
    fn test_metadata_with_description() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json)
            .with_description("My backup");
        assert_eq!(m.description, Some("My backup".to_string()));
    }

    #[test]
    fn test_result_new() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json);
        let r = ArchiveResult::new(m);
        assert_eq!(r.total_archived(), 0);
    }

    #[test]
    fn test_result_add_archived() {
        let m = ArchiveMetadata::new("test", ArchiveType::Full, ArchiveFormat::Json);
        let mut r = ArchiveResult::new(m);
        r.add_archived("key1".to_string());
        assert_eq!(r.total_archived(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ArchiverStats::default();
        s.record(ArchiveFormat::Json, 10, 500);
        assert_eq!(s.total_archives, 1);
        assert_eq!(s.total_keys_archived, 10);
    }

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

    #[test]
    fn test_registry_new() {
        let r = SettingsArchiverRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsArchiverRegistry::new();
        r.register("a1", SettingsArchiver::new(ArchiverConfig::new(ArchiveFormat::Toml)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_archiver_query() {
        assert!(is_archiver_query("settings archiver"));
        assert!(!is_archiver_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = archiver_fun_fact();
        assert!(fact.contains("archiver"));
    }
}
