// v0.0.644: Settings Formatter (Phase 220)
// Formatter for converting settings values to display formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FormatType {
    /// Plain text
    #[default]
    Plain,
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// Table format
    Table,
}

impl std::fmt::Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => write!(f, "plain"),
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Table => write!(f, "table"),
        }
    }
}

/// Format style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FormatStyle {
    /// Compact style
    #[default]
    Compact,
    /// Pretty style
    Pretty,
    /// Minimal style
    Minimal,
    /// Verbose style
    Verbose,
}

impl std::fmt::Display for FormatStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compact => write!(f, "compact"),
            Self::Pretty => write!(f, "pretty"),
            Self::Minimal => write!(f, "minimal"),
            Self::Verbose => write!(f, "verbose"),
        }
    }
}

/// Formatter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    /// Format type
    pub format_type: FormatType,
    /// Format style
    pub format_style: FormatStyle,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include metadata
    pub include_metadata: bool,
    /// Indent size
    pub indent_size: usize,
}

impl FormatterConfig {
    /// Create new config
    pub fn new(format_type: FormatType) -> Self {
        Self {
            format_type,
            format_style: FormatStyle::Compact,
            category: None,
            include_metadata: false,
            indent_size: 2,
        }
    }

    /// Set format style
    pub fn format_style(mut self, style: FormatStyle) -> Self {
        self.format_style = style;
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

    /// Set indent size
    pub fn indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self::new(FormatType::Plain)
    }
}

/// Format result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResult {
    /// Original value
    pub original: String,
    /// Formatted value
    pub formatted: String,
    /// Format type used
    pub format_type: FormatType,
    /// Format style used
    pub format_style: FormatStyle,
}

impl FormatResult {
    /// Create new result
    pub fn new(
        original: impl Into<String>,
        formatted: impl Into<String>,
        format_type: FormatType,
        format_style: FormatStyle,
    ) -> Self {
        Self {
            original: original.into(),
            formatted: formatted.into(),
            format_type,
            format_style,
        }
    }

    /// Get output length
    pub fn output_length(&self) -> usize {
        self.formatted.len()
    }

    /// Was transformed
    pub fn was_transformed(&self) -> bool {
        self.original != self.formatted
    }
}

/// Formatter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormatterStats {
    /// Total formatted
    pub total_formatted: usize,
    /// By format type
    pub by_type: HashMap<String, usize>,
    /// By format style
    pub by_style: HashMap<String, usize>,
    /// Total output bytes
    pub total_output_bytes: usize,
}

impl FormatterStats {
    /// Record formatting
    pub fn record(&mut self, format_type: FormatType, format_style: FormatStyle, output_len: usize) {
        self.total_formatted += 1;
        *self.by_type.entry(format_type.to_string()).or_insert(0) += 1;
        *self.by_style.entry(format_style.to_string()).or_insert(0) += 1;
        self.total_output_bytes += output_len;
    }

    /// Average output size
    pub fn average_output_size(&self) -> f64 {
        if self.total_formatted == 0 {
            0.0
        } else {
            self.total_output_bytes as f64 / self.total_formatted as f64
        }
    }
}

/// Settings formatter
#[derive(Debug, Clone, Default)]
pub struct SettingsFormatter {
    /// Config
    config: FormatterConfig,
    /// Results
    results: Vec<FormatResult>,
    /// Stats
    stats: FormatterStats,
}

impl SettingsFormatter {
    /// Create new formatter
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: FormatterStats::default(),
        }
    }

    /// Format value
    pub fn format(&mut self, value: impl Into<String>) -> FormatResult {
        let original = value.into();
        let formatted = self.apply_format(&original);

        self.stats.record(
            self.config.format_type,
            self.config.format_style,
            formatted.len(),
        );

        let result = FormatResult::new(
            original,
            formatted,
            self.config.format_type,
            self.config.format_style,
        );
        self.results.push(result.clone());
        result
    }

    /// Apply formatting
    fn apply_format(&self, value: &str) -> String {
        match self.config.format_type {
            FormatType::Plain => value.to_string(),
            FormatType::Json => format!("\"{}\"", value.replace('"', "\\\"")),
            FormatType::Toml => format!("\"{}\"", value.replace('"', "\\\"")),
            FormatType::Yaml => {
                if value.contains(':') || value.contains('#') {
                    format!("\"{}\"", value)
                } else {
                    value.to_string()
                }
            }
            FormatType::Table => {
                let indent = " ".repeat(self.config.indent_size);
                format!("{}| {} |", indent, value)
            }
        }
    }

    /// Get results
    pub fn results(&self) -> &[FormatResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &FormatterStats {
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

/// Settings formatter registry
#[derive(Debug, Clone, Default)]
pub struct SettingsFormatterRegistry {
    /// Formatters by ID
    formatters: HashMap<String, SettingsFormatter>,
}

impl SettingsFormatterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register formatter
    pub fn register(&mut self, id: impl Into<String>, formatter: SettingsFormatter) {
        self.formatters.insert(id.into(), formatter);
    }

    /// Unregister formatter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.formatters.remove(id).is_some()
    }

    /// Get formatter
    pub fn get(&self, id: &str) -> Option<&SettingsFormatter> {
        self.formatters.get(id)
    }

    /// Get formatter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFormatter> {
        self.formatters.get_mut(id)
    }

    /// Formatter count
    pub fn count(&self) -> usize {
        self.formatters.len()
    }
}

/// Format formatter registry
pub fn format_formatter_registry(registry: &SettingsFormatterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Formatter Registry:\n");
    output.push_str(&format!("  Formatters: {}\n", registry.count()));
    output
}

/// Check if query is about formatter
pub fn is_formatter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("formatter") || lower.contains("format settings") || lower.contains("output format")
}

/// Fun fact about formatter
pub fn formatter_fun_fact() -> &'static str {
    "Anna's settings formatters convert values to any output format!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_type_display() {
        assert_eq!(format!("{}", FormatType::Plain), "plain");
        assert_eq!(format!("{}", FormatType::Json), "json");
    }

    #[test]
    fn test_format_style_display() {
        assert_eq!(format!("{}", FormatStyle::Compact), "compact");
        assert_eq!(format!("{}", FormatStyle::Pretty), "pretty");
    }

    #[test]
    fn test_config_new() {
        let c = FormatterConfig::new(FormatType::Json);
        assert_eq!(c.format_type, FormatType::Json);
    }

    #[test]
    fn test_config_builder() {
        let c = FormatterConfig::new(FormatType::Plain)
            .format_style(FormatStyle::Pretty)
            .indent_size(4);
        assert_eq!(c.format_style, FormatStyle::Pretty);
        assert_eq!(c.indent_size, 4);
    }

    #[test]
    fn test_result_new() {
        let r = FormatResult::new("test", "\"test\"", FormatType::Json, FormatStyle::Compact);
        assert!(r.was_transformed());
    }

    #[test]
    fn test_result_unchanged() {
        let r = FormatResult::new("test", "test", FormatType::Plain, FormatStyle::Compact);
        assert!(!r.was_transformed());
    }

    #[test]
    fn test_stats_record() {
        let mut s = FormatterStats::default();
        s.record(FormatType::Json, FormatStyle::Compact, 10);
        s.record(FormatType::Plain, FormatStyle::Pretty, 20);
        assert_eq!(s.total_formatted, 2);
        assert_eq!(s.total_output_bytes, 30);
    }

    #[test]
    fn test_formatter_new() {
        let f = SettingsFormatter::new(FormatterConfig::new(FormatType::Plain));
        assert_eq!(f.result_count(), 0);
    }

    #[test]
    fn test_formatter_format_plain() {
        let mut f = SettingsFormatter::new(FormatterConfig::new(FormatType::Plain));
        let r = f.format("test");
        assert_eq!(r.formatted, "test");
    }

    #[test]
    fn test_formatter_format_json() {
        let mut f = SettingsFormatter::new(FormatterConfig::new(FormatType::Json));
        let r = f.format("test");
        assert_eq!(r.formatted, "\"test\"");
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsFormatterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsFormatterRegistry::new();
        r.register("fmt1", SettingsFormatter::new(FormatterConfig::new(FormatType::Plain)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_formatter_query() {
        assert!(is_formatter_query("settings formatter"));
        assert!(!is_formatter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = formatter_fun_fact();
        assert!(fact.contains("formatter"));
    }
}
