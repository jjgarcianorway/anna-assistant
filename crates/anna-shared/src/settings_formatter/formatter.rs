// v0.0.644: Settings Formatter Implementation (Phase 220)
// Main formatter implementation

use super::types::{FormatResult, FormatStyle, FormatType, FormatterConfig, FormatterStats};

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
