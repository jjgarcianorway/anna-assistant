// v0.0.583: Settings Diagnostics Utilities
// Helper functions for diagnostics

use super::report::DiagnosticReport;
use super::types::DiagnosticSeverity;

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
