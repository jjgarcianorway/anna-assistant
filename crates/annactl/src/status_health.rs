//! Status Health Logic (6.17.0)
//!
//! Strict health derivation for annactl status command.
//! Ensures overall status is NEVER "HEALTHY" when there are warnings or critical issues.
//!
//! Rules:
//! 1. Overall status is computed from ALL diagnostics (not just brain analysis)
//! 2. Daemon health requires BOTH systemd active AND RPC socket reachable
//! 3. Permission checks are consistent across all sections
//! 4. Brain analysis unavailability is a critical issue
//! 5. All diagnostics contribute to overall health level

use anna_common::ipc::BrainAnalysisData;
use std::fmt;

/// Health level with strict monotonic computation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Critical,
}

impl fmt::Display for HealthLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "HEALTHY"),
            Self::Degraded => write!(f, "DEGRADED"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Unified health summary combining all diagnostics
#[derive(Debug, Clone)]
pub struct HealthSummary {
    /// Overall health level (monotonic from all checks)
    pub level: HealthLevel,
    /// Critical issue count
    pub critical_count: usize,
    /// Warning count
    pub warning_count: usize,
    /// Individual diagnostic checks
    pub diagnostics: Vec<Diagnostic>,
}

/// Individual diagnostic check result
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Title of the diagnostic
    pub title: String,
    /// Body text (detailed explanation)
    pub body: String,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Hints for remediation
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Critical,
}

impl HealthSummary {
    /// Create new health summary
    pub fn new() -> Self {
        Self {
            level: HealthLevel::Healthy,
            critical_count: 0,
            warning_count: 0,
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic and update counts
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        match diagnostic.severity {
            DiagnosticSeverity::Critical => self.critical_count += 1,
            DiagnosticSeverity::Warning => self.warning_count += 1,
            DiagnosticSeverity::Info => {}
        }
        self.diagnostics.push(diagnostic);
    }

    /// Compute overall health level from diagnostics
    ///
    /// Rules:
    /// - If critical_count > 0: Critical
    /// - Else if warning_count > 0: Degraded
    /// - Else: Healthy
    pub fn compute_level(&mut self) {
        self.level = if self.critical_count > 0 {
            HealthLevel::Critical
        } else if self.warning_count > 0 {
            HealthLevel::Degraded
        } else {
            HealthLevel::Healthy
        };
    }

    /// Get human-readable status line
    pub fn status_line(&self) -> String {
        match self.level {
            HealthLevel::Healthy => "all systems operational".to_string(),
            HealthLevel::Degraded => format!(
                "{} warning(s), {} critical issue(s)",
                self.warning_count, self.critical_count
            ),
            HealthLevel::Critical => format!(
                "{} critical issue(s) require attention",
                self.critical_count
            ),
        }
    }
}

impl Default for HealthSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Check daemon health with RPC reachability
pub async fn check_daemon_health(
    systemd_active: bool,
    systemd_enabled: bool,
    rpc_reachable: bool,
) -> Option<Diagnostic> {
    if !systemd_active {
        return Some(Diagnostic {
            title: "Anna daemon not running".to_string(),
            body: "annad.service is inactive according to systemd".to_string(),
            severity: DiagnosticSeverity::Critical,
            hints: vec![
                "Investigate: sudo systemctl status annad".to_string(),
                "Start: sudo systemctl start annad".to_string(),
                "Logs: journalctl -u annad -n 50".to_string(),
            ],
        });
    }

    if !rpc_reachable {
        return Some(Diagnostic {
            title: "Anna daemon not reachable".to_string(),
            body: "annad is active according to systemd, but the RPC socket /run/anna.sock (or /run/anna/anna.sock) is missing or unresponsive".to_string(),
            severity: DiagnosticSeverity::Critical,
            hints: vec![
                "Investigate: sudo systemctl status annad".to_string(),
                "Check socket: ls -la /run/anna.sock /run/anna/anna.sock".to_string(),
                "Logs: journalctl -u annad -n 50".to_string(),
            ],
        });
    }

    if !systemd_enabled {
        return Some(Diagnostic {
            title: "Anna daemon not enabled at boot".to_string(),
            body: "annad.service is running but not enabled to start on boot".to_string(),
            severity: DiagnosticSeverity::Warning,
            hints: vec![
                "Enable: sudo systemctl enable annad".to_string(),
            ],
        });
    }

    None
}

/// Check brain analysis availability
pub fn check_brain_analysis(analysis_result: Option<&BrainAnalysisData>) -> Option<Diagnostic> {
    if analysis_result.is_none() {
        return Some(Diagnostic {
            title: "Brain analysis unavailable".to_string(),
            body: "Cannot fetch diagnostic analysis from annad. The daemon may not be reachable.".to_string(),
            severity: DiagnosticSeverity::Critical,
            hints: vec![
                "Check daemon: sudo systemctl status annad".to_string(),
                "Check RPC socket: ls -la /run/anna.sock".to_string(),
            ],
        });
    }
    None
}

/// Check Anna self-health (permissions, dependencies, LLM)
pub fn check_anna_self_health() -> Vec<Diagnostic> {
    let self_health = anna_common::anna_self_health::check_anna_self_health();
    let mut diagnostics = Vec::new();

    // Check dependencies
    if !self_health.deps_ok {
        diagnostics.push(Diagnostic {
            title: "Missing system dependencies".to_string(),
            body: format!("Anna requires system tools that are not installed: {}", self_health.missing_deps.join(", ")),
            severity: DiagnosticSeverity::Warning,
            hints: vec![
                "Install missing tools using pacman".to_string(),
            ],
        });
    }

    // Check permissions (including /var/log/anna)
    if !self_health.permissions_ok {
        for issue in &self_health.missing_permissions {
            // Extract fix command if present in the issue string
            let (body, hints) = if let Some(arrow_pos) = issue.find(" → Fix: ") {
                let body_part = &issue[..arrow_pos];
                let fix_part = &issue[arrow_pos + 10..]; // Skip " → Fix: "
                (body_part.to_string(), vec![fix_part.to_string()])
            } else {
                (issue.clone(), vec![])
            };

            diagnostics.push(Diagnostic {
                title: "Permission issue".to_string(),
                body,
                severity: DiagnosticSeverity::Warning,
                hints,
            });
        }
    }

    // Check LLM backend
    if !self_health.llm_ok {
        diagnostics.push(Diagnostic {
            title: "LLM backend issue".to_string(),
            body: self_health.llm_details.clone(),
            severity: DiagnosticSeverity::Warning,
            hints: vec![
                "Install Ollama: see https://ollama.ai".to_string(),
                "Start service: sudo systemctl start ollama".to_string(),
            ],
        });
    }

    diagnostics
}

/// Check daemon restart count from journal
pub async fn check_daemon_restarts() -> Option<Diagnostic> {
    use std::process::Command;

    // Get journal for annad and count restart lines
    let output = Command::new("journalctl")
        .args(&["-u", "annad", "--no-pager", "-n", "200"])
        .output();

    if let Ok(output) = output {
        let log_text = String::from_utf8_lossy(&output.stdout);

        // Look for "restart counter" line
        if let Some(line) = log_text.lines().rev().find(|l| l.contains("restart counter is at")) {
            // Extract counter value
            if let Some(counter_str) = line.split("restart counter is at ").nth(1) {
                if let Some(counter) = counter_str.split('.').next().and_then(|s| s.parse::<u32>().ok()) {
                    if counter >= 10 {
                        return Some(Diagnostic {
                            title: "Anna daemon instability detected".to_string(),
                            body: format!("annad has restarted {} times this boot. This indicates instability.", counter),
                            severity: if counter >= 20 { DiagnosticSeverity::Critical } else { DiagnosticSeverity::Warning },
                            hints: vec![
                                "Check logs: journalctl -u annad -n 100".to_string(),
                                "Look for errors or crash patterns".to_string(),
                            ],
                        });
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_summary_healthy() {
        let mut summary = HealthSummary::new();
        summary.compute_level();

        assert_eq!(summary.level, HealthLevel::Healthy);
        assert_eq!(summary.critical_count, 0);
        assert_eq!(summary.warning_count, 0);
    }

    #[test]
    fn test_health_summary_degraded_on_warning() {
        let mut summary = HealthSummary::new();
        summary.add_diagnostic(Diagnostic {
            title: "Test warning".to_string(),
            body: "Test".to_string(),
            severity: DiagnosticSeverity::Warning,
            hints: vec![],
        });
        summary.compute_level();

        assert_eq!(summary.level, HealthLevel::Degraded);
        assert_eq!(summary.warning_count, 1);
    }

    #[test]
    fn test_health_summary_critical_on_critical() {
        let mut summary = HealthSummary::new();
        summary.add_diagnostic(Diagnostic {
            title: "Test critical".to_string(),
            body: "Test".to_string(),
            severity: DiagnosticSeverity::Critical,
            hints: vec![],
        });
        summary.compute_level();

        assert_eq!(summary.level, HealthLevel::Critical);
        assert_eq!(summary.critical_count, 1);
    }

    #[test]
    fn test_health_summary_never_healthy_with_issues() {
        let mut summary = HealthSummary::new();

        // Add a warning
        summary.add_diagnostic(Diagnostic {
            title: "Warning".to_string(),
            body: "Test".to_string(),
            severity: DiagnosticSeverity::Warning,
            hints: vec![],
        });
        summary.compute_level();

        // Must NOT be healthy
        assert_ne!(summary.level, HealthLevel::Healthy);
        assert!(matches!(summary.level, HealthLevel::Degraded | HealthLevel::Critical));
    }
}
