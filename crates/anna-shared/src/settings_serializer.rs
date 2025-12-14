// v0.0.602: Settings Serializer (Phase 178)
// Serialization/deserialization for settings in multiple formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SerializationFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// Binary format
    Binary,
    /// Pretty JSON
    JsonPretty,
}

impl std::fmt::Display for SerializationFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Binary => write!(f, "binary"),
            Self::JsonPretty => write!(f, "json_pretty"),
        }
    }
}

/// Serialization result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerializeResult {
    /// Success
    Success,
    /// Format error
    FormatError,
    /// Encoding error
    EncodingError,
    /// Size exceeded
    SizeExceeded,
    /// Unknown error
    Unknown,
}

impl std::fmt::Display for SerializeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::FormatError => write!(f, "format_error"),
            Self::EncodingError => write!(f, "encoding_error"),
            Self::SizeExceeded => write!(f, "size_exceeded"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Serialization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializeOptions {
    /// Format
    pub format: SerializationFormat,
    /// Include metadata
    pub include_metadata: bool,
    /// Compress output
    pub compress: bool,
    /// Max size bytes
    pub max_size: Option<usize>,
    /// Pretty print
    pub pretty: bool,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            format: SerializationFormat::Json,
            include_metadata: true,
            compress: false,
            max_size: None,
            pretty: false,
        }
    }
}

impl SerializeOptions {
    /// Create new options
    pub fn new(format: SerializationFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Set include metadata
    pub fn metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set compress
    pub fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Set max size
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }

    /// Set pretty print
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

/// Serialized output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedData {
    /// Format used
    pub format: SerializationFormat,
    /// Data bytes
    pub data: Vec<u8>,
    /// Size
    pub size: usize,
    /// Compressed
    pub compressed: bool,
    /// Checksum
    pub checksum: Option<String>,
}

impl SerializedData {
    /// Create new serialized data
    pub fn new(format: SerializationFormat, data: Vec<u8>) -> Self {
        let size = data.len();
        Self {
            format,
            data,
            size,
            compressed: false,
            checksum: None,
        }
    }

    /// Set compressed
    pub fn compressed(mut self, compressed: bool) -> Self {
        self.compressed = compressed;
        self
    }

    /// Set checksum
    pub fn checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }

    /// Get as string (if text format)
    pub fn as_string(&self) -> Option<String> {
        String::from_utf8(self.data.clone()).ok()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Serialization stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializationStats {
    /// Total serializations
    pub total: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
    /// Total bytes
    pub total_bytes: usize,
    /// Errors
    pub errors: usize,
}

impl SerializationStats {
    /// Create new stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record serialization
    pub fn record(&mut self, format: SerializationFormat, size: usize, success: bool) {
        self.total += 1;
        self.total_bytes += size;
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
        if !success {
            self.errors += 1;
        }
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            (self.total - self.errors) as f64 / self.total as f64
        }
    }
}

/// Settings serializer
#[derive(Debug, Clone, Default)]
pub struct SettingsSerializer {
    /// Default format
    default_format: SerializationFormat,
    /// Stats
    stats: SerializationStats,
    /// Format preferences by category
    category_formats: HashMap<SettingsCategory, SerializationFormat>,
}

impl SettingsSerializer {
    /// Create new serializer
    pub fn new() -> Self {
        Self {
            default_format: SerializationFormat::Json,
            ..Default::default()
        }
    }

    /// Set default format
    pub fn set_default(&mut self, format: SerializationFormat) {
        self.default_format = format;
    }

    /// Get default format
    pub fn default_format(&self) -> SerializationFormat {
        self.default_format
    }

    /// Set category format
    pub fn set_category_format(&mut self, category: SettingsCategory, format: SerializationFormat) {
        self.category_formats.insert(category, format);
    }

    /// Get format for category
    pub fn format_for(&self, category: SettingsCategory) -> SerializationFormat {
        self.category_formats
            .get(&category)
            .copied()
            .unwrap_or(self.default_format)
    }

    /// Record operation
    pub fn record(&mut self, format: SerializationFormat, size: usize, success: bool) {
        self.stats.record(format, size, success);
    }

    /// Get stats
    pub fn stats(&self) -> &SerializationStats {
        &self.stats
    }

    /// Reset stats
    pub fn reset_stats(&mut self) {
        self.stats = SerializationStats::new();
    }
}

/// Format serializer
pub fn format_serializer(serializer: &SettingsSerializer) -> String {
    let mut output = String::new();
    output.push_str("Settings Serializer:\n");
    output.push_str(&format!("  Default format: {}\n", serializer.default_format));
    output.push_str(&format!("  Total operations: {}\n", serializer.stats.total));
    output.push_str(&format!("  Total bytes: {}\n", serializer.stats.total_bytes));
    output.push_str(&format!(
        "  Success rate: {:.1}%\n",
        serializer.stats.success_rate() * 100.0
    ));
    output
}

/// Check if query is about serializer
pub fn is_serializer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("serialize")
        || lower.contains("export format")
        || lower.contains("convert to json")
}

/// Fun fact about serializer
pub fn serializer_fun_fact() -> &'static str {
    "Anna can serialize your settings to JSON, TOML, YAML, or binary formats!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", SerializationFormat::Json), "json");
        assert_eq!(format!("{}", SerializationFormat::Toml), "toml");
    }

    #[test]
    fn test_result_display() {
        assert_eq!(format!("{}", SerializeResult::Success), "success");
        assert_eq!(format!("{}", SerializeResult::FormatError), "format_error");
    }

    #[test]
    fn test_options_default() {
        let o = SerializeOptions::default();
        assert_eq!(o.format, SerializationFormat::Json);
        assert!(o.include_metadata);
    }

    #[test]
    fn test_options_builder() {
        let o = SerializeOptions::new(SerializationFormat::Toml)
            .metadata(false)
            .compress(true)
            .max_size(1024);
        assert!(o.compress);
        assert_eq!(o.max_size, Some(1024));
    }

    #[test]
    fn test_serialized_data_new() {
        let d = SerializedData::new(SerializationFormat::Json, vec![1, 2, 3]);
        assert_eq!(d.size, 3);
        assert!(!d.is_empty());
    }

    #[test]
    fn test_serialized_data_as_string() {
        let d = SerializedData::new(SerializationFormat::Json, b"hello".to_vec());
        assert_eq!(d.as_string(), Some("hello".to_string()));
    }

    #[test]
    fn test_stats_new() {
        let s = SerializationStats::new();
        assert_eq!(s.total, 0);
    }

    #[test]
    fn test_stats_record() {
        let mut s = SerializationStats::new();
        s.record(SerializationFormat::Json, 100, true);
        assert_eq!(s.total, 1);
        assert_eq!(s.total_bytes, 100);
    }

    #[test]
    fn test_serializer_new() {
        let s = SettingsSerializer::new();
        assert_eq!(s.default_format(), SerializationFormat::Json);
    }

    #[test]
    fn test_serializer_category_format() {
        let mut s = SettingsSerializer::new();
        s.set_category_format(SettingsCategory::Personality, SerializationFormat::Toml);
        assert_eq!(s.format_for(SettingsCategory::Personality), SerializationFormat::Toml);
    }

    #[test]
    fn test_is_serializer_query() {
        assert!(is_serializer_query("serialize settings"));
        assert!(!is_serializer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = serializer_fun_fact();
        assert!(fact.contains("serialize"));
    }
}
