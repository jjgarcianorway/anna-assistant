// v0.0.648: Settings Encoder (Phase 224)
// Encoder for serializing settings to various formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Encoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum EncodingFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// Binary format
    Binary,
    /// Base64 format
    Base64,
}

impl std::fmt::Display for EncodingFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Binary => write!(f, "binary"),
            Self::Base64 => write!(f, "base64"),
        }
    }
}

/// Encoding options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncodingOptions {
    /// Compact encoding
    #[default]
    Compact,
    /// Pretty encoding
    Pretty,
    /// Minified encoding
    Minified,
    /// Verbose encoding
    Verbose,
}

impl std::fmt::Display for EncodingOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compact => write!(f, "compact"),
            Self::Pretty => write!(f, "pretty"),
            Self::Minified => write!(f, "minified"),
            Self::Verbose => write!(f, "verbose"),
        }
    }
}

/// Encoder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderConfig {
    /// Encoding format
    pub format: EncodingFormat,
    /// Encoding options
    pub options: EncodingOptions,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include nulls
    pub include_nulls: bool,
    /// Sort keys
    pub sort_keys: bool,
}

impl EncoderConfig {
    /// Create new config
    pub fn new(format: EncodingFormat) -> Self {
        Self {
            format,
            options: EncodingOptions::Compact,
            category: None,
            include_nulls: false,
            sort_keys: false,
        }
    }

    /// Set options
    pub fn options(mut self, options: EncodingOptions) -> Self {
        self.options = options;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include nulls
    pub fn include_nulls(mut self, include: bool) -> Self {
        self.include_nulls = include;
        self
    }

    /// Set sort keys
    pub fn sort_keys(mut self, sort: bool) -> Self {
        self.sort_keys = sort;
        self
    }
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self::new(EncodingFormat::Json)
    }
}

/// Encode result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeResult {
    /// Encoded data
    pub data: String,
    /// Format used
    pub format: EncodingFormat,
    /// Options used
    pub options: EncodingOptions,
    /// Byte size
    pub byte_size: usize,
}

impl EncodeResult {
    /// Create new result
    pub fn new(data: impl Into<String>, format: EncodingFormat, options: EncodingOptions) -> Self {
        let data = data.into();
        let byte_size = data.len();
        Self {
            data,
            format,
            options,
            byte_size,
        }
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Encoder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncoderStats {
    /// Total encodes
    pub total_encodes: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
    /// By options
    pub by_options: HashMap<String, usize>,
    /// Total bytes encoded
    pub total_bytes: usize,
}

impl EncoderStats {
    /// Record encode
    pub fn record(&mut self, format: EncodingFormat, options: EncodingOptions, byte_size: usize) {
        self.total_encodes += 1;
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
        *self.by_options.entry(options.to_string()).or_insert(0) += 1;
        self.total_bytes += byte_size;
    }

    /// Average bytes per encode
    pub fn average_bytes(&self) -> f64 {
        if self.total_encodes == 0 {
            0.0
        } else {
            self.total_bytes as f64 / self.total_encodes as f64
        }
    }
}

/// Settings encoder
#[derive(Debug, Clone, Default)]
pub struct SettingsEncoder {
    /// Config
    config: EncoderConfig,
    /// Results
    results: Vec<EncodeResult>,
    /// Stats
    stats: EncoderStats,
}

impl SettingsEncoder {
    /// Create new encoder
    pub fn new(config: EncoderConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: EncoderStats::default(),
        }
    }

    /// Encode settings
    pub fn encode(&mut self, settings: &[(String, String)]) -> EncodeResult {
        let data = self.do_encode(settings);
        let result = EncodeResult::new(data, self.config.format, self.config.options);

        self.stats.record(
            self.config.format,
            self.config.options,
            result.byte_size,
        );
        self.results.push(result.clone());
        result
    }

    /// Do encode
    fn do_encode(&self, settings: &[(String, String)]) -> String {
        match self.config.format {
            EncodingFormat::Json => {
                let mut output = String::from("{");
                for (i, (key, value)) in settings.iter().enumerate() {
                    if i > 0 {
                        output.push(',');
                    }
                    output.push_str(&format!("\"{}\":\"{}\"", key, value));
                }
                output.push('}');
                output
            }
            EncodingFormat::Toml => {
                let mut output = String::new();
                for (key, value) in settings {
                    output.push_str(&format!("{} = \"{}\"\n", key, value));
                }
                output
            }
            EncodingFormat::Yaml => {
                let mut output = String::new();
                for (key, value) in settings {
                    output.push_str(&format!("{}: {}\n", key, value));
                }
                output
            }
            EncodingFormat::Binary => {
                // Simple binary-like encoding
                settings
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("\0")
            }
            EncodingFormat::Base64 => {
                // Simple base64-like representation
                use std::fmt::Write;
                let mut output = String::new();
                for (key, value) in settings {
                    let combined = format!("{}={}", key, value);
                    for b in combined.bytes() {
                        write!(output, "{:02x}", b).ok();
                    }
                }
                output
            }
        }
    }

    /// Get results
    pub fn results(&self) -> &[EncodeResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &EncoderStats {
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

/// Settings encoder registry
#[derive(Debug, Clone, Default)]
pub struct SettingsEncoderRegistry {
    /// Encoders by ID
    encoders: HashMap<String, SettingsEncoder>,
}

impl SettingsEncoderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register encoder
    pub fn register(&mut self, id: impl Into<String>, encoder: SettingsEncoder) {
        self.encoders.insert(id.into(), encoder);
    }

    /// Unregister encoder
    pub fn unregister(&mut self, id: &str) -> bool {
        self.encoders.remove(id).is_some()
    }

    /// Get encoder
    pub fn get(&self, id: &str) -> Option<&SettingsEncoder> {
        self.encoders.get(id)
    }

    /// Get encoder mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsEncoder> {
        self.encoders.get_mut(id)
    }

    /// Encoder count
    pub fn count(&self) -> usize {
        self.encoders.len()
    }
}

/// Format encoder registry
pub fn format_encoder_registry(registry: &SettingsEncoderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Encoder Registry:\n");
    output.push_str(&format!("  Encoders: {}\n", registry.count()));
    output
}

/// Check if query is about encoder
pub fn is_encoder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("encoder") || lower.contains("encode settings") || lower.contains("serialize settings")
}

/// Fun fact about encoder
pub fn encoder_fun_fact() -> &'static str {
    "Anna's settings encoders serialize configs to any format!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_format_display() {
        assert_eq!(format!("{}", EncodingFormat::Json), "json");
        assert_eq!(format!("{}", EncodingFormat::Toml), "toml");
    }

    #[test]
    fn test_encoding_options_display() {
        assert_eq!(format!("{}", EncodingOptions::Compact), "compact");
        assert_eq!(format!("{}", EncodingOptions::Pretty), "pretty");
    }

    #[test]
    fn test_config_new() {
        let c = EncoderConfig::new(EncodingFormat::Json);
        assert!(!c.sort_keys);
    }

    #[test]
    fn test_config_builder() {
        let c = EncoderConfig::new(EncodingFormat::Toml)
            .options(EncodingOptions::Pretty)
            .sort_keys(true);
        assert_eq!(c.options, EncodingOptions::Pretty);
        assert!(c.sort_keys);
    }

    #[test]
    fn test_result_new() {
        let r = EncodeResult::new("{}", EncodingFormat::Json, EncodingOptions::Compact);
        assert_eq!(r.byte_size, 2);
    }

    #[test]
    fn test_result_empty() {
        let r = EncodeResult::new("", EncodingFormat::Json, EncodingOptions::Compact);
        assert!(r.is_empty());
    }

    #[test]
    fn test_stats_record() {
        let mut s = EncoderStats::default();
        s.record(EncodingFormat::Json, EncodingOptions::Compact, 100);
        s.record(EncodingFormat::Toml, EncodingOptions::Pretty, 200);
        assert_eq!(s.total_encodes, 2);
        assert_eq!(s.total_bytes, 300);
    }

    #[test]
    fn test_encoder_new() {
        let e = SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Json));
        assert_eq!(e.result_count(), 0);
    }

    #[test]
    fn test_encoder_encode_json() {
        let mut e = SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Json));
        let settings = vec![("key".to_string(), "value".to_string())];
        let r = e.encode(&settings);
        assert!(r.data.contains("\"key\":\"value\""));
    }

    #[test]
    fn test_encoder_encode_toml() {
        let mut e = SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Toml));
        let settings = vec![("key".to_string(), "value".to_string())];
        let r = e.encode(&settings);
        assert!(r.data.contains("key = \"value\""));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsEncoderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsEncoderRegistry::new();
        r.register("enc1", SettingsEncoder::new(EncoderConfig::new(EncodingFormat::Json)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_encoder_query() {
        assert!(is_encoder_query("settings encoder"));
        assert!(!is_encoder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = encoder_fun_fact();
        assert!(fact.contains("encoder"));
    }
}
