// v0.0.649: Settings Decoder (Phase 225)
// Main decoder implementation

use std::collections::HashMap;

use super::config::DecoderConfig;
use super::result::{DecodeError, DecodeResult};
use super::stats::DecoderStats;
use super::types::{DecodingFormat, DecodingMode};

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
