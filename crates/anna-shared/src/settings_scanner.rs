// v0.0.684: Settings Scanner (Phase 260)
// Scan settings for patterns and anomalies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Scan type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ScanType {
    /// Scan for patterns
    #[default]
    Pattern,
    /// Scan for anomalies
    Anomaly,
    /// Scan for duplicates
    Duplicate,
    /// Scan for empty values
    Empty,
}

impl std::fmt::Display for ScanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "pattern"),
            Self::Anomaly => write!(f, "anomaly"),
            Self::Duplicate => write!(f, "duplicate"),
            Self::Empty => write!(f, "empty"),
        }
    }
}

/// Scan severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum ScanSeverity {
    /// Info level
    #[default]
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

impl std::fmt::Display for ScanSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Scanner config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// Default scan type
    pub default_type: ScanType,
    /// Min severity to report
    pub min_severity: ScanSeverity,
    /// Pattern to match
    pub pattern: Option<String>,
    /// Case insensitive
    pub case_insensitive: bool,
}

impl ScannerConfig {
    /// Create new config
    pub fn new(scan_type: ScanType) -> Self {
        Self {
            default_type: scan_type,
            min_severity: ScanSeverity::Info,
            pattern: None,
            case_insensitive: true,
        }
    }

    /// Set min severity
    pub fn min_severity(mut self, severity: ScanSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Set pattern
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self::new(ScanType::Pattern)
    }
}

/// Scan finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFinding {
    /// Finding ID
    pub id: String,
    /// Key involved
    pub key: String,
    /// Value involved
    pub value: String,
    /// Scan type
    pub scan_type: ScanType,
    /// Severity
    pub severity: ScanSeverity,
    /// Message
    pub message: String,
}

impl ScanFinding {
    /// Create new finding
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        scan_type: ScanType,
        severity: ScanSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid_simple(),
            key: key.into(),
            value: value.into(),
            scan_type,
            severity,
            message: message.into(),
        }
    }

    /// Is critical
    pub fn is_critical(&self) -> bool {
        self.severity == ScanSeverity::Critical
    }

    /// Is error or worse
    pub fn is_error_or_worse(&self) -> bool {
        self.severity >= ScanSeverity::Error
    }
}

/// Simple UUID generator
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("scan_{:x}_{:x}", now.as_secs(), now.subsec_nanos())
}

/// Scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Findings
    pub findings: Vec<ScanFinding>,
    /// Total scanned
    pub total_scanned: usize,
    /// Total findings
    pub total_findings: usize,
    /// Scan type
    pub scan_type: ScanType,
}

impl ScanResult {
    /// Create new result
    pub fn new(findings: Vec<ScanFinding>, scanned: usize, scan_type: ScanType) -> Self {
        let total_findings = findings.len();
        Self {
            findings,
            total_scanned: scanned,
            total_findings,
            scan_type,
        }
    }

    /// Has findings
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }

    /// Has critical
    pub fn has_critical(&self) -> bool {
        self.findings.iter().any(|f| f.is_critical())
    }

    /// Count by severity
    pub fn count_by_severity(&self, severity: ScanSeverity) -> usize {
        self.findings.iter().filter(|f| f.severity == severity).count()
    }
}

impl Default for ScanResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, ScanType::Pattern)
    }
}

/// Scanner stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScannerStats {
    /// Total scans
    pub total_scans: usize,
    /// Total entries scanned
    pub total_entries: usize,
    /// Total findings
    pub total_findings: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ScannerStats {
    /// Record scan
    pub fn record(&mut self, result: &ScanResult) {
        self.total_scans += 1;
        self.total_entries += result.total_scanned;
        self.total_findings += result.total_findings;
        *self.by_type.entry(result.scan_type.to_string()).or_insert(0) += 1;
    }

    /// Finding rate
    pub fn finding_rate(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            self.total_findings as f64 / self.total_entries as f64
        }
    }
}

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

/// Scanner registry
#[derive(Debug, Clone, Default)]
pub struct ScannerRegistry {
    /// Scanners by ID
    scanners: HashMap<String, SettingsScanner>,
}

impl ScannerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register scanner
    pub fn register(&mut self, id: impl Into<String>, scanner: SettingsScanner) {
        self.scanners.insert(id.into(), scanner);
    }

    /// Unregister scanner
    pub fn unregister(&mut self, id: &str) -> bool {
        self.scanners.remove(id).is_some()
    }

    /// Get scanner
    pub fn get(&self, id: &str) -> Option<&SettingsScanner> {
        self.scanners.get(id)
    }

    /// Get scanner mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsScanner> {
        self.scanners.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.scanners.len()
    }
}

/// Format scanner registry
pub fn format_scanner_registry(registry: &ScannerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Scanner Registry:\n");
    output.push_str(&format!("  Scanners: {}\n", registry.count()));
    output
}

/// Check if query is about scanner
pub fn is_scanner_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("scan settings") || lower.contains("settings scanner") || lower.contains("check settings")
}

/// Fun fact about scanner
pub fn scanner_fun_fact() -> &'static str {
    "Anna's settings scanner detects patterns and anomalies to keep your config healthy!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_type_display() {
        assert_eq!(format!("{}", ScanType::Pattern), "pattern");
        assert_eq!(format!("{}", ScanType::Anomaly), "anomaly");
    }

    #[test]
    fn test_scan_severity_display() {
        assert_eq!(format!("{}", ScanSeverity::Info), "info");
        assert_eq!(format!("{}", ScanSeverity::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = ScannerConfig::new(ScanType::Pattern);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = ScannerConfig::new(ScanType::Anomaly)
            .min_severity(ScanSeverity::Warning)
            .pattern("test");
        assert_eq!(c.min_severity, ScanSeverity::Warning);
        assert_eq!(c.pattern, Some("test".to_string()));
    }

    #[test]
    fn test_finding_new() {
        let f = ScanFinding::new("key", "value", ScanType::Pattern, ScanSeverity::Info, "test");
        assert_eq!(f.key, "key");
        assert!(!f.is_critical());
    }

    #[test]
    fn test_finding_is_error_or_worse() {
        let error = ScanFinding::new("k", "v", ScanType::Anomaly, ScanSeverity::Error, "test");
        assert!(error.is_error_or_worse());

        let info = ScanFinding::new("k", "v", ScanType::Pattern, ScanSeverity::Info, "test");
        assert!(!info.is_error_or_worse());
    }

    #[test]
    fn test_result_new() {
        let r = ScanResult::new(vec![], 10, ScanType::Pattern);
        assert_eq!(r.total_scanned, 10);
        assert!(!r.has_findings());
    }

    #[test]
    fn test_result_count_by_severity() {
        let findings = vec![
            ScanFinding::new("k1", "v1", ScanType::Pattern, ScanSeverity::Info, "test"),
            ScanFinding::new("k2", "v2", ScanType::Pattern, ScanSeverity::Warning, "test"),
        ];
        let r = ScanResult::new(findings, 10, ScanType::Pattern);
        assert_eq!(r.count_by_severity(ScanSeverity::Info), 1);
        assert_eq!(r.count_by_severity(ScanSeverity::Warning), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ScannerStats::default();
        let r = ScanResult::new(vec![ScanFinding::new("k", "v", ScanType::Pattern, ScanSeverity::Info, "t")], 5, ScanType::Pattern);
        s.record(&r);
        assert_eq!(s.total_scans, 1);
        assert_eq!(s.total_findings, 1);
    }

    #[test]
    fn test_scanner_new() {
        let s = SettingsScanner::new(ScannerConfig::default());
        assert_eq!(s.stats().total_scans, 0);
    }

    #[test]
    fn test_scanner_scan_pattern() {
        let mut s = SettingsScanner::new(ScannerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = s.scan_pattern(&settings, "app");
        assert_eq!(result.total_findings, 1);
    }

    #[test]
    fn test_scanner_scan_empty() {
        let mut s = SettingsScanner::new(ScannerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("filled".to_string(), "value".to_string());
        settings.insert("empty".to_string(), "".to_string());

        let result = s.scan_empty(&settings);
        assert_eq!(result.total_findings, 1);
    }

    #[test]
    fn test_scanner_scan_duplicates() {
        let mut s = SettingsScanner::new(ScannerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key1".to_string(), "same_value".to_string());
        settings.insert("key2".to_string(), "same_value".to_string());
        settings.insert("key3".to_string(), "different".to_string());

        let result = s.scan_duplicates(&settings);
        assert_eq!(result.total_findings, 2); // key1 and key2 are duplicates
    }

    #[test]
    fn test_registry_new() {
        let r = ScannerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ScannerRegistry::new();
        r.register("s1", SettingsScanner::new(ScannerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_scanner_query() {
        assert!(is_scanner_query("scan settings"));
        assert!(!is_scanner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = scanner_fun_fact();
        assert!(fact.contains("scanner"));
    }
}
