// v0.0.583: Settings Diagnostics Report
// Report structure for diagnostic results

use serde::{Deserialize, Serialize};

use super::types::{DiagnosticIssue, DiagnosticSeverity};

/// Diagnostic check result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Issues found
    pub issues: Vec<DiagnosticIssue>,
    /// Checks performed
    pub checks_performed: usize,
    /// Duration in ms
    pub duration_ms: u64,
}

impl DiagnosticReport {
    /// Create new report
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Add issue
    pub fn add_issue(&mut self, issue: DiagnosticIssue) {
        self.issues.push(issue);
    }

    /// Count issues by severity
    pub fn count_by_severity(&self, severity: DiagnosticSeverity) -> usize {
        self.issues.iter().filter(|i| i.severity == severity).count()
    }

    /// Has errors
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.is_error())
    }

    /// Has warnings
    pub fn has_warnings(&self) -> bool {
        self.count_by_severity(DiagnosticSeverity::Warning) > 0
    }

    /// Is healthy
    pub fn is_healthy(&self) -> bool {
        !self.has_errors()
    }

    /// Auto-fixable issues
    pub fn auto_fixable(&self) -> Vec<&DiagnosticIssue> {
        self.issues.iter().filter(|i| i.auto_fixable).collect()
    }

    /// Health score (0-100)
    pub fn health_score(&self) -> u8 {
        let critical = self.count_by_severity(DiagnosticSeverity::Critical) as i32 * 30;
        let errors = self.count_by_severity(DiagnosticSeverity::Error) as i32 * 15;
        let warnings = self.count_by_severity(DiagnosticSeverity::Warning) as i32 * 5;

        let penalty = critical + errors + warnings;
        (100 - penalty.min(100)).max(0) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_diagnostics::types::{DiagnosticType, DiagnosticSeverity};

    #[test]
    fn test_diagnostic_report_new() {
        let report = DiagnosticReport::new();
        assert!(report.issues.is_empty());
        assert!(report.is_healthy());
    }

    #[test]
    fn test_diagnostic_report_add_issue() {
        let mut report = DiagnosticReport::new();
        report.add_issue(DiagnosticIssue::new(
            1, DiagnosticSeverity::Warning, DiagnosticType::Configuration, "Test"
        ));
        assert_eq!(report.issues.len(), 1);
        assert!(report.has_warnings());
    }

    #[test]
    fn test_diagnostic_report_health_score() {
        let report = DiagnosticReport::new();
        assert_eq!(report.health_score(), 100);
    }
}
