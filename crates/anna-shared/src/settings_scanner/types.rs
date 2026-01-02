// v0.0.684: Settings Scanner - Core Types
// Core types for settings scanning

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
