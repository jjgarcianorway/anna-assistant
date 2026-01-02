// v0.0.650: Settings Converter Implementation (Phase 226)
// Main converter logic for transforming settings between formats

use super::config::ConverterConfig;
use super::formats::{SourceFormat, TargetFormat};
use super::result::ConversionResult;
use super::stats::ConverterStats;

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
