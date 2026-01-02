// v0.0.648: Settings Encoder (Phase 224)
// Main encoder implementation

use super::config::EncoderConfig;
use super::format::EncodingFormat;
use super::result::EncodeResult;
use super::stats::EncoderStats;

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
