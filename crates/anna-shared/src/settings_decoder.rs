// v0.0.649: Settings Decoder (Phase 225)
// Decoder for deserializing settings from various formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Decoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DecodingFormat {
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

impl std::fmt::Display for DecodingFormat {
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

/// Decoding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DecodingMode {
    /// Strict mode
    #[default]
    Strict,
    /// Lenient mode
    Lenient,
    /// Permissive mode
    Permissive,
    /// Recovery mode
    Recovery,
}

impl std::fmt::Display for DecodingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Lenient => write!(f, "lenient"),
            Self::Permissive => write!(f, "permissive"),
            Self::Recovery => write!(f, "recovery"),
        }
    }
}

/// Decoder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoderConfig {
    /// Decoding format
    pub format: DecodingFormat,
    /// Decoding mode
    pub mode: DecodingMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Allow unknown keys
    pub allow_unknown: bool,
    /// Collect errors
    pub collect_errors: bool,
}

impl DecoderConfig {
    /// Create new config
    pub fn new(format: DecodingFormat) -> Self {
        Self {
            format,
            mode: DecodingMode::Strict,
            category: None,
            allow_unknown: false,
            collect_errors: true,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: DecodingMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set allow unknown
    pub fn allow_unknown(mut self, allow: bool) -> Self {
        self.allow_unknown = allow;
        self
    }

    /// Set collect errors
    pub fn collect_errors(mut self, collect: bool) -> Self {
        self.collect_errors = collect;
        self
    }
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self::new(DecodingFormat::Json)
    }
}

/// Decode error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeError {
    /// Error message
    pub message: String,
    /// Position
    pub position: Option<usize>,
    /// Key path
    pub path: Option<String>,
}

impl DecodeError {
    /// Create new error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: None,
            path: None,
        }
    }

    /// Set position
    pub fn at(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }

    /// Set path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Decode result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeResult {
    /// Was successful
    pub success: bool,
    /// Decoded values
    pub values: HashMap<String, String>,
    /// Errors
    pub errors: Vec<DecodeError>,
    /// Format used
    pub format: DecodingFormat,
}

impl DecodeResult {
    /// Create success result
    pub fn success(values: HashMap<String, String>, format: DecodingFormat) -> Self {
        Self {
            success: true,
            values,
            errors: Vec::new(),
            format,
        }
    }

    /// Create failure result
    pub fn failure(errors: Vec<DecodeError>, format: DecodingFormat) -> Self {
        Self {
            success: false,
            values: HashMap::new(),
            errors,
            format,
        }
    }

    /// Value count
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

/// Decoder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecoderStats {
    /// Total decodes
    pub total_decodes: usize,
    /// Successful decodes
    pub successful: usize,
    /// Failed decodes
    pub failed: usize,
    /// By format
    pub by_format: HashMap<String, usize>,
    /// Total values decoded
    pub total_values: usize,
}

impl DecoderStats {
    /// Record decode
    pub fn record(&mut self, format: DecodingFormat, success: bool, value_count: usize) {
        self.total_decodes += 1;
        if success {
            self.successful += 1;
            self.total_values += value_count;
        } else {
            self.failed += 1;
        }
        *self.by_format.entry(format.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_decodes == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_decodes as f64
        }
    }
}

/// Settings decoder
#[derive(Debug, Clone, Default)]
pub struct SettingsDecoder {
    /// Config
    config: DecoderConfig,
    /// Results
    results: Vec<DecodeResult>,
    /// Stats
    stats: DecoderStats,
}

impl SettingsDecoder {
    /// Create new decoder
    pub fn new(config: DecoderConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: DecoderStats::default(),
        }
    }

    /// Decode input
    pub fn decode(&mut self, input: &str) -> DecodeResult {
        let result = self.do_decode(input);
        self.stats.record(
            self.config.format,
            result.success,
            result.value_count(),
        );
        self.results.push(result.clone());
        result
    }

    /// Do decode
    fn do_decode(&self, input: &str) -> DecodeResult {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return DecodeResult::failure(
                vec![DecodeError::new("Empty input")],
                self.config.format,
            );
        }

        let mut values = HashMap::new();
        let mut errors = Vec::new();

        match self.config.format {
            DecodingFormat::Json => {
                // Simple JSON parsing
                let content = trimmed.trim_start_matches('{').trim_end_matches('}');
                for part in content.split(',') {
                    let part = part.trim();
                    if let Some(colon) = part.find(':') {
                        let key = part[..colon].trim().trim_matches('"').to_string();
                        let value = part[colon + 1..].trim().trim_matches('"').to_string();
                        values.insert(key, value);
                    }
                }
            }
            DecodingFormat::Toml | DecodingFormat::Yaml => {
                // Key=value or key: value parsing
                for line in trimmed.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some(eq) = line.find('=').or_else(|| line.find(':')) {
                        let key = line[..eq].trim().to_string();
                        let value = line[eq + 1..].trim().trim_matches('"').to_string();
                        values.insert(key, value);
                    } else if self.config.mode == DecodingMode::Strict {
                        errors.push(DecodeError::new("Invalid line format"));
                    }
                }
            }
            DecodingFormat::Binary => {
                for part in trimmed.split('\0') {
                    if let Some(eq) = part.find('=') {
                        let key = part[..eq].to_string();
                        let value = part[eq + 1..].to_string();
                        values.insert(key, value);
                    }
                }
            }
            DecodingFormat::Base64 => {
                // Hex decoding
                let mut decoded = Vec::new();
                let chars: Vec<char> = trimmed.chars().collect();
                for chunk in chars.chunks(2) {
                    if chunk.len() == 2 {
                        if let Ok(byte) = u8::from_str_radix(&format!("{}{}", chunk[0], chunk[1]), 16) {
                            decoded.push(byte);
                        }
                    }
                }
                let decoded_str = String::from_utf8_lossy(&decoded);
                for part in decoded_str.split('\0').filter(|p| !p.is_empty()) {
                    if let Some(eq) = part.find('=') {
                        let key = part[..eq].to_string();
                        let value = part[eq + 1..].to_string();
                        values.insert(key, value);
                    }
                }
            }
        }

        if !errors.is_empty() && self.config.mode == DecodingMode::Strict {
            DecodeResult::failure(errors, self.config.format)
        } else {
            DecodeResult::success(values, self.config.format)
        }
    }

    /// Get results
    pub fn results(&self) -> &[DecodeResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &DecoderStats {
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

/// Settings decoder registry
#[derive(Debug, Clone, Default)]
pub struct SettingsDecoderRegistry {
    /// Decoders by ID
    decoders: HashMap<String, SettingsDecoder>,
}

impl SettingsDecoderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register decoder
    pub fn register(&mut self, id: impl Into<String>, decoder: SettingsDecoder) {
        self.decoders.insert(id.into(), decoder);
    }

    /// Unregister decoder
    pub fn unregister(&mut self, id: &str) -> bool {
        self.decoders.remove(id).is_some()
    }

    /// Get decoder
    pub fn get(&self, id: &str) -> Option<&SettingsDecoder> {
        self.decoders.get(id)
    }

    /// Get decoder mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDecoder> {
        self.decoders.get_mut(id)
    }

    /// Decoder count
    pub fn count(&self) -> usize {
        self.decoders.len()
    }
}

/// Format decoder registry
pub fn format_decoder_registry(registry: &SettingsDecoderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Decoder Registry:\n");
    output.push_str(&format!("  Decoders: {}\n", registry.count()));
    output
}

/// Check if query is about decoder
pub fn is_decoder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("decoder") || lower.contains("decode settings") || lower.contains("deserialize settings")
}

/// Fun fact about decoder
pub fn decoder_fun_fact() -> &'static str {
    "Anna's settings decoders parse configs from any format!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoding_format_display() {
        assert_eq!(format!("{}", DecodingFormat::Json), "json");
        assert_eq!(format!("{}", DecodingFormat::Toml), "toml");
    }

    #[test]
    fn test_decoding_mode_display() {
        assert_eq!(format!("{}", DecodingMode::Strict), "strict");
        assert_eq!(format!("{}", DecodingMode::Lenient), "lenient");
    }

    #[test]
    fn test_config_new() {
        let c = DecoderConfig::new(DecodingFormat::Json);
        assert_eq!(c.mode, DecodingMode::Strict);
    }

    #[test]
    fn test_config_builder() {
        let c = DecoderConfig::new(DecodingFormat::Toml)
            .mode(DecodingMode::Lenient)
            .allow_unknown(true);
        assert_eq!(c.mode, DecodingMode::Lenient);
        assert!(c.allow_unknown);
    }

    #[test]
    fn test_error_new() {
        let e = DecodeError::new("test error").at(10);
        assert_eq!(e.position, Some(10));
    }

    #[test]
    fn test_result_success() {
        let mut values = HashMap::new();
        values.insert("key".to_string(), "value".to_string());
        let r = DecodeResult::success(values, DecodingFormat::Json);
        assert!(r.success);
        assert_eq!(r.value_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = DecoderStats::default();
        s.record(DecodingFormat::Json, true, 5);
        s.record(DecodingFormat::Json, false, 0);
        assert_eq!(s.total_decodes, 2);
        assert_eq!(s.successful, 1);
    }

    #[test]
    fn test_decoder_new() {
        let d = SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Json));
        assert_eq!(d.result_count(), 0);
    }

    #[test]
    fn test_decoder_decode_json() {
        let mut d = SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Json));
        let r = d.decode(r#"{"key":"value"}"#);
        assert!(r.success);
        assert_eq!(r.values.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_decoder_decode_toml() {
        let mut d = SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Toml));
        let r = d.decode("key = \"value\"");
        assert!(r.success);
        assert_eq!(r.values.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsDecoderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsDecoderRegistry::new();
        r.register("dec1", SettingsDecoder::new(DecoderConfig::new(DecodingFormat::Json)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_decoder_query() {
        assert!(is_decoder_query("settings decoder"));
        assert!(!is_decoder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = decoder_fun_fact();
        assert!(fact.contains("decoder"));
    }
}
