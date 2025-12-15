// v0.0.650: Settings Converter (Phase 226)
// Converter for transforming settings between formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Source format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SourceFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// INI format
    Ini,
    /// Env format
    Env,
}

impl std::fmt::Display for SourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}

/// Target format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TargetFormat {
    /// JSON format
    #[default]
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// INI format
    Ini,
    /// Env format
    Env,
}

impl std::fmt::Display for TargetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}

/// Converter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConverterConfig {
    /// Source format
    pub source: SourceFormat,
    /// Target format
    pub target: TargetFormat,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Preserve comments
    pub preserve_comments: bool,
    /// Pretty output
    pub pretty: bool,
}

impl ConverterConfig {
    /// Create new config
    pub fn new(source: SourceFormat, target: TargetFormat) -> Self {
        Self {
            source,
            target,
            category: None,
            preserve_comments: false,
            pretty: true,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set preserve comments
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.preserve_comments = preserve;
        self
    }

    /// Set pretty output
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for ConverterConfig {
    fn default() -> Self {
        Self::new(SourceFormat::Json, TargetFormat::Toml)
    }
}

/// Conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    /// Was successful
    pub success: bool,
    /// Converted data
    pub data: String,
    /// Source format
    pub source: SourceFormat,
    /// Target format
    pub target: TargetFormat,
    /// Key count
    pub key_count: usize,
}

impl ConversionResult {
    /// Create success result
    pub fn success(data: impl Into<String>, source: SourceFormat, target: TargetFormat, key_count: usize) -> Self {
        Self {
            success: true,
            data: data.into(),
            source,
            target,
            key_count,
        }
    }

    /// Create failure result
    pub fn failure(source: SourceFormat, target: TargetFormat) -> Self {
        Self {
            success: false,
            data: String::new(),
            source,
            target,
            key_count: 0,
        }
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Converter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConverterStats {
    /// Total conversions
    pub total_conversions: usize,
    /// Successful conversions
    pub successful: usize,
    /// Failed conversions
    pub failed: usize,
    /// By source
    pub by_source: HashMap<String, usize>,
    /// By target
    pub by_target: HashMap<String, usize>,
}

impl ConverterStats {
    /// Record conversion
    pub fn record(&mut self, source: SourceFormat, target: TargetFormat, success: bool) {
        self.total_conversions += 1;
        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        *self.by_source.entry(source.to_string()).or_insert(0) += 1;
        *self.by_target.entry(target.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_conversions == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_conversions as f64
        }
    }
}

/// Settings converter
#[derive(Debug, Clone, Default)]
pub struct SettingsConverter {
    /// Config
    config: ConverterConfig,
    /// Results
    results: Vec<ConversionResult>,
    /// Stats
    stats: ConverterStats,
}

impl SettingsConverter {
    /// Create new converter
    pub fn new(config: ConverterConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: ConverterStats::default(),
        }
    }

    /// Convert input
    pub fn convert(&mut self, input: &str) -> ConversionResult {
        let result = self.do_convert(input);
        self.stats.record(
            self.config.source,
            self.config.target,
            result.success,
        );
        self.results.push(result.clone());
        result
    }

    /// Do convert
    fn do_convert(&self, input: &str) -> ConversionResult {
        // Parse input to key-value pairs
        let pairs = self.parse_source(input);
        if pairs.is_empty() {
            return ConversionResult::failure(self.config.source, self.config.target);
        }

        let output = self.format_target(&pairs);
        ConversionResult::success(output, self.config.source, self.config.target, pairs.len())
    }

    /// Parse source format
    fn parse_source(&self, input: &str) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let trimmed = input.trim();

        match self.config.source {
            SourceFormat::Json => {
                let content = trimmed.trim_start_matches('{').trim_end_matches('}');
                for part in content.split(',') {
                    let part = part.trim();
                    if let Some(colon) = part.find(':') {
                        let key = part[..colon].trim().trim_matches('"').to_string();
                        let value = part[colon + 1..].trim().trim_matches('"').to_string();
                        pairs.push((key, value));
                    }
                }
            }
            SourceFormat::Toml | SourceFormat::Ini | SourceFormat::Yaml => {
                for line in trimmed.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                        continue;
                    }
                    if let Some(eq) = line.find('=').or_else(|| line.find(':')) {
                        let key = line[..eq].trim().to_string();
                        let value = line[eq + 1..].trim().trim_matches('"').to_string();
                        pairs.push((key, value));
                    }
                }
            }
            SourceFormat::Env => {
                for line in trimmed.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some(eq) = line.find('=') {
                        let key = line[..eq].to_string();
                        let value = line[eq + 1..].to_string();
                        pairs.push((key, value));
                    }
                }
            }
        }

        pairs
    }

    /// Format to target
    fn format_target(&self, pairs: &[(String, String)]) -> String {
        let mut output = String::new();

        match self.config.target {
            TargetFormat::Json => {
                output.push('{');
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        output.push(',');
                    }
                    output.push_str(&format!("\"{}\":\"{}\"", key, value));
                }
                output.push('}');
            }
            TargetFormat::Toml => {
                for (key, value) in pairs {
                    output.push_str(&format!("{} = \"{}\"\n", key, value));
                }
            }
            TargetFormat::Yaml => {
                for (key, value) in pairs {
                    output.push_str(&format!("{}: {}\n", key, value));
                }
            }
            TargetFormat::Ini => {
                for (key, value) in pairs {
                    output.push_str(&format!("{}={}\n", key, value));
                }
            }
            TargetFormat::Env => {
                for (key, value) in pairs {
                    output.push_str(&format!("{}={}\n", key.to_uppercase(), value));
                }
            }
        }

        output
    }

    /// Get results
    pub fn results(&self) -> &[ConversionResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &ConverterStats {
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

/// Settings converter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsConverterRegistry {
    /// Converters by ID
    converters: HashMap<String, SettingsConverter>,
}

impl SettingsConverterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register converter
    pub fn register(&mut self, id: impl Into<String>, converter: SettingsConverter) {
        self.converters.insert(id.into(), converter);
    }

    /// Unregister converter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.converters.remove(id).is_some()
    }

    /// Get converter
    pub fn get(&self, id: &str) -> Option<&SettingsConverter> {
        self.converters.get(id)
    }

    /// Get converter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsConverter> {
        self.converters.get_mut(id)
    }

    /// Converter count
    pub fn count(&self) -> usize {
        self.converters.len()
    }
}

/// Format converter registry
pub fn format_converter_registry(registry: &SettingsConverterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Converter Registry:\n");
    output.push_str(&format!("  Converters: {}\n", registry.count()));
    output
}

/// Check if query is about converter
pub fn is_converter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("converter") || lower.contains("convert settings") || lower.contains("transform format")
}

/// Fun fact about converter
pub fn converter_fun_fact() -> &'static str {
    "Anna's settings converters transform configs between any format!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_format_display() {
        assert_eq!(format!("{}", SourceFormat::Json), "json");
        assert_eq!(format!("{}", SourceFormat::Toml), "toml");
    }

    #[test]
    fn test_target_format_display() {
        assert_eq!(format!("{}", TargetFormat::Yaml), "yaml");
        assert_eq!(format!("{}", TargetFormat::Env), "env");
    }

    #[test]
    fn test_config_new() {
        let c = ConverterConfig::new(SourceFormat::Json, TargetFormat::Toml);
        assert!(c.pretty);
    }

    #[test]
    fn test_config_builder() {
        let c = ConverterConfig::new(SourceFormat::Toml, TargetFormat::Yaml)
            .preserve_comments(true)
            .pretty(false);
        assert!(c.preserve_comments);
        assert!(!c.pretty);
    }

    #[test]
    fn test_result_success() {
        let r = ConversionResult::success("data", SourceFormat::Json, TargetFormat::Toml, 5);
        assert!(r.success);
        assert_eq!(r.key_count, 5);
    }

    #[test]
    fn test_result_failure() {
        let r = ConversionResult::failure(SourceFormat::Json, TargetFormat::Toml);
        assert!(!r.success);
        assert!(r.is_empty());
    }

    #[test]
    fn test_stats_record() {
        let mut s = ConverterStats::default();
        s.record(SourceFormat::Json, TargetFormat::Toml, true);
        s.record(SourceFormat::Toml, TargetFormat::Yaml, false);
        assert_eq!(s.total_conversions, 2);
        assert_eq!(s.successful, 1);
    }

    #[test]
    fn test_converter_new() {
        let c = SettingsConverter::new(ConverterConfig::new(SourceFormat::Json, TargetFormat::Toml));
        assert_eq!(c.result_count(), 0);
    }

    #[test]
    fn test_converter_json_to_toml() {
        let mut c = SettingsConverter::new(ConverterConfig::new(SourceFormat::Json, TargetFormat::Toml));
        let r = c.convert(r#"{"key":"value"}"#);
        assert!(r.success);
        assert!(r.data.contains("key = \"value\""));
    }

    #[test]
    fn test_converter_toml_to_yaml() {
        let mut c = SettingsConverter::new(ConverterConfig::new(SourceFormat::Toml, TargetFormat::Yaml));
        let r = c.convert("key = \"value\"");
        assert!(r.success);
        assert!(r.data.contains("key: value"));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsConverterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsConverterRegistry::new();
        r.register("conv1", SettingsConverter::new(ConverterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_converter_query() {
        assert!(is_converter_query("settings converter"));
        assert!(!is_converter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = converter_fun_fact();
        assert!(fact.contains("converter"));
    }
}
