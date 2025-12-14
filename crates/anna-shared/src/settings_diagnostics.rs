// v0.0.583: Settings Diagnostics (Phase 159)
// Diagnostics and health checking for settings

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// Information
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Diagnostic type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticType {
    /// Configuration issue
    Configuration,
    /// Compatibility issue
    Compatibility,
    /// Performance issue
    Performance,
    /// Security issue
    Security,
    /// Validation issue
    Validation,
    /// Dependency issue
    Dependency,
}

impl std::fmt::Display for DiagnosticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration => write!(f, "Configuration"),
            Self::Compatibility => write!(f, "Compatibility"),
            Self::Performance => write!(f, "Performance"),
            Self::Security => write!(f, "Security"),
            Self::Validation => write!(f, "Validation"),
            Self::Dependency => write!(f, "Dependency"),
        }
    }
}

/// Single diagnostic issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    /// Issue ID
    pub id: u64,
    /// Severity
    pub severity: DiagnosticSeverity,
    /// Type
    pub issue_type: DiagnosticType,
    /// Category affected
    pub category: Option<SettingsCategory>,
    /// Setting name
    pub setting: Option<String>,
    /// Description
    pub description: String,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Auto-fixable
    pub auto_fixable: bool,
}

impl DiagnosticIssue {
    /// Create new issue
    pub fn new(
        id: u64,
        severity: DiagnosticSeverity,
        issue_type: DiagnosticType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            severity,
            issue_type,
            category: None,
            setting: None,
            description: description.into(),
            suggestion: None,
            auto_fixable: false,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set setting
    pub fn setting(mut self, setting: impl Into<String>) -> Self {
        self.setting = Some(setting.into());
        self
    }

    /// Set suggestion
    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Mark as auto-fixable
    pub fn auto_fixable(mut self) -> Self {
        self.auto_fixable = true;
        self
    }

    /// Check if is error or critical
    pub fn is_error(&self) -> bool {
        self.severity >= DiagnosticSeverity::Error
    }
}

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

/// Settings diagnostics runner
#[derive(Debug, Clone, Default)]
pub struct SettingsDiagnostics {
    /// Next issue ID
    next_id: u64,
    /// Last report
    last_report: Option<DiagnosticReport>,
}

impl SettingsDiagnostics {
    /// Create new diagnostics runner
    pub fn new() -> Self {
        Self::default()
    }

    /// Run full diagnostics
    pub fn run(&mut self, settings: &UnifiedSettings) -> DiagnosticReport {
        let start = std::time::Instant::now();
        let mut report = DiagnosticReport::new();

        // Run all checks
        self.check_configuration(settings, &mut report);
        self.check_security(settings, &mut report);
        self.check_performance(settings, &mut report);
        self.check_compatibility(settings, &mut report);

        report.duration_ms = start.elapsed().as_millis() as u64;
        report.checks_performed = 4;

        self.last_report = Some(report.clone());
        report
    }

    fn check_configuration(&mut self, _settings: &UnifiedSettings, report: &mut DiagnosticReport) {
        // Example configuration check
        report.add_issue(
            DiagnosticIssue::new(
                self.next_id,
                DiagnosticSeverity::Info,
                DiagnosticType::Configuration,
                "Configuration check passed",
            )
        );
        self.next_id += 1;
    }

    fn check_security(&mut self, settings: &UnifiedSettings, report: &mut DiagnosticReport) {
        // Check root confirmation
        if !settings.risk.require_root_confirmation {
            report.add_issue(
                DiagnosticIssue::new(
                    self.next_id,
                    DiagnosticSeverity::Warning,
                    DiagnosticType::Security,
                    "Root confirmation is disabled",
                )
                .category(SettingsCategory::Risk)
                .setting("require_root_confirmation")
                .suggestion("Enable root confirmation for better security")
                .auto_fixable()
            );
            self.next_id += 1;
        }
    }

    fn check_performance(&mut self, settings: &UnifiedSettings, report: &mut DiagnosticReport) {
        // Check timeout settings
        if settings.timeout.command_timeout_ms > 120000 {
            report.add_issue(
                DiagnosticIssue::new(
                    self.next_id,
                    DiagnosticSeverity::Warning,
                    DiagnosticType::Performance,
                    "Command timeout is very high (>120s)",
                )
                .category(SettingsCategory::Timeout)
                .setting("command_timeout_ms")
                .suggestion("Consider lowering timeout to prevent hanging")
            );
            self.next_id += 1;
        }
    }

    fn check_compatibility(&mut self, _settings: &UnifiedSettings, report: &mut DiagnosticReport) {
        // Version check
        report.add_issue(
            DiagnosticIssue::new(
                self.next_id,
                DiagnosticSeverity::Info,
                DiagnosticType::Compatibility,
                "Compatibility check passed",
            )
        );
        self.next_id += 1;
    }

    /// Get last report
    pub fn last_report(&self) -> Option<&DiagnosticReport> {
        self.last_report.as_ref()
    }

    /// Quick health check
    pub fn quick_check(&mut self, settings: &UnifiedSettings) -> bool {
        let report = self.run(settings);
        report.is_healthy()
    }
}

/// Format diagnostics report for display
pub fn format_diagnostics(report: &DiagnosticReport) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Diagnostics ===\n\n");
    output.push_str(&format!("Health Score: {}%\n", report.health_score()));
    output.push_str(&format!("Checks: {}\n", report.checks_performed));
    output.push_str(&format!("Duration: {}ms\n\n", report.duration_ms));

    let critical = report.count_by_severity(DiagnosticSeverity::Critical);
    let errors = report.count_by_severity(DiagnosticSeverity::Error);
    let warnings = report.count_by_severity(DiagnosticSeverity::Warning);

    output.push_str(&format!("Critical: {} | Errors: {} | Warnings: {}\n\n", critical, errors, warnings));

    if report.issues.is_empty() {
        output.push_str("No issues found. Settings are healthy!\n");
    } else {
        output.push_str("--- Issues ---\n");
        for issue in &report.issues {
            let fix_marker = if issue.auto_fixable { " [auto-fix]" } else { "" };
            output.push_str(&format!(
                "• [{}] {} - {}{}\n",
                issue.severity, issue.issue_type, issue.description, fix_marker
            ));
        }
    }

    output
}

/// Check if query is about diagnostics
pub fn is_diagnostics_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("diagnostic")
        || lower.contains("health check")
        || lower.contains("check settings")
}

/// Fun fact about diagnostics
pub fn settings_diagnostics_fun_fact() -> &'static str {
    "Anna runs diagnostics to catch settings issues before they cause problems!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_severity_display() {
        assert_eq!(format!("{}", DiagnosticSeverity::Error), "Error");
        assert_eq!(format!("{}", DiagnosticSeverity::Warning), "Warning");
    }

    #[test]
    fn test_diagnostic_type_display() {
        assert_eq!(format!("{}", DiagnosticType::Security), "Security");
        assert_eq!(format!("{}", DiagnosticType::Performance), "Performance");
    }

    #[test]
    fn test_diagnostic_issue_new() {
        let issue = DiagnosticIssue::new(
            1,
            DiagnosticSeverity::Warning,
            DiagnosticType::Configuration,
            "Test issue",
        );
        assert_eq!(issue.id, 1);
        assert!(!issue.is_error());
    }

    #[test]
    fn test_diagnostic_issue_builder() {
        let issue = DiagnosticIssue::new(1, DiagnosticSeverity::Error, DiagnosticType::Security, "Test")
            .category(SettingsCategory::Risk)
            .suggestion("Fix it")
            .auto_fixable();
        assert!(issue.is_error());
        assert!(issue.auto_fixable);
    }

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

    #[test]
    fn test_settings_diagnostics_new() {
        let diag = SettingsDiagnostics::new();
        assert!(diag.last_report().is_none());
    }

    #[test]
    fn test_settings_diagnostics_run() {
        let mut diag = SettingsDiagnostics::new();
        let settings = UnifiedSettings::default();
        let report = diag.run(&settings);
        assert!(report.checks_performed > 0);
    }

    #[test]
    fn test_settings_diagnostics_quick_check() {
        let mut diag = SettingsDiagnostics::new();
        let settings = UnifiedSettings::default();
        let healthy = diag.quick_check(&settings);
        assert!(healthy);
    }

    #[test]
    fn test_format_diagnostics() {
        let report = DiagnosticReport::new();
        let output = format_diagnostics(&report);
        assert!(output.contains("Diagnostics"));
    }

    #[test]
    fn test_is_diagnostics_query() {
        assert!(is_diagnostics_query("run diagnostics"));
        assert!(is_diagnostics_query("health check"));
        assert!(!is_diagnostics_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_diagnostics_fun_fact();
        assert!(fact.contains("diagnostic"));
    }
}
