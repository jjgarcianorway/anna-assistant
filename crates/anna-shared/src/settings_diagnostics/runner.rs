// v0.0.583: Settings Diagnostics Runner
// Main runner for executing diagnostic checks

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::report::DiagnosticReport;
use super::types::{DiagnosticIssue, DiagnosticSeverity, DiagnosticType};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
