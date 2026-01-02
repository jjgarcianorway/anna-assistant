// v0.0.684: Settings Scanner - Core Scanner Implementation
// Scanner logic for pattern, empty, duplicate, and anomaly detection

use super::types::{ScanFinding, ScanResult, ScanSeverity, ScanType, ScannerConfig, ScannerStats};
use std::collections::HashMap;

/// Settings scanner
#[derive(Debug, Clone, Default)]
pub struct SettingsScanner {
    /// Config
    config: ScannerConfig,
    /// Stats
    stats: ScannerStats,
}

impl SettingsScanner {
    /// Create new scanner
    pub fn new(config: ScannerConfig) -> Self {
        Self {
            config,
            stats: ScannerStats::default(),
        }
    }

    /// Scan for pattern
    pub fn scan_pattern(&mut self, settings: &HashMap<String, String>, pattern: &str) -> ScanResult {
        let mut findings = Vec::new();
        let pat = if self.config.case_insensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        for (key, value) in settings {
            let (k, v) = if self.config.case_insensitive {
                (key.to_lowercase(), value.to_lowercase())
            } else {
                (key.clone(), value.clone())
            };

            if k.contains(&pat) || v.contains(&pat) {
                findings.push(ScanFinding::new(
                    key.clone(),
                    value.clone(),
                    ScanType::Pattern,
                    ScanSeverity::Info,
                    format!("Pattern '{}' found", pattern),
                ));
            }
        }

        let result = ScanResult::new(findings, settings.len(), ScanType::Pattern);
        self.stats.record(&result);
        result
    }

    /// Scan for empty values
    pub fn scan_empty(&mut self, settings: &HashMap<String, String>) -> ScanResult {
        let mut findings = Vec::new();

        for (key, value) in settings {
            if value.is_empty() {
                findings.push(ScanFinding::new(
                    key.clone(),
                    value.clone(),
                    ScanType::Empty,
                    ScanSeverity::Warning,
                    "Empty value detected".to_string(),
                ));
            }
        }

        let result = ScanResult::new(findings, settings.len(), ScanType::Empty);
        self.stats.record(&result);
        result
    }

    /// Scan for duplicates (same value for different keys)
    pub fn scan_duplicates(&mut self, settings: &HashMap<String, String>) -> ScanResult {
        let mut findings = Vec::new();
        let mut value_keys: HashMap<&String, Vec<&String>> = HashMap::new();

        // Group keys by value
        for (key, value) in settings {
            value_keys.entry(value).or_default().push(key);
        }

        // Find duplicates
        for (value, keys) in value_keys {
            if keys.len() > 1 {
                for key in &keys {
                    findings.push(ScanFinding::new(
                        (*key).clone(),
                        value.clone(),
                        ScanType::Duplicate,
                        ScanSeverity::Info,
                        format!("Duplicate value shared with {} other keys", keys.len() - 1),
                    ));
                }
            }
        }

        let result = ScanResult::new(findings, settings.len(), ScanType::Duplicate);
        self.stats.record(&result);
        result
    }

    /// Scan for anomalies (unusual patterns)
    pub fn scan_anomalies(&mut self, settings: &HashMap<String, String>) -> ScanResult {
        let mut findings = Vec::new();

        for (key, value) in settings {
            // Check for very long values
            if value.len() > 1000 {
                findings.push(ScanFinding::new(
                    key.clone(),
                    format!("{}...", &value[..50]),
                    ScanType::Anomaly,
                    ScanSeverity::Warning,
                    format!("Unusually long value ({} chars)", value.len()),
                ));
            }

            // Check for potential secrets
            let lower_key = key.to_lowercase();
            if (lower_key.contains("password") || lower_key.contains("secret") || lower_key.contains("key"))
                && !value.is_empty()
            {
                findings.push(ScanFinding::new(
                    key.clone(),
                    "***".to_string(),
                    ScanType::Anomaly,
                    ScanSeverity::Error,
                    "Potential sensitive data".to_string(),
                ));
            }
        }

        let result = ScanResult::new(findings, settings.len(), ScanType::Anomaly);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &ScannerStats {
        &self.stats
    }
}
