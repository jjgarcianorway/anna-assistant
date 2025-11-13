//! Caretaker Brain - Anna's core analysis engine
//!
//! This module ties together health checks, metrics, predictions, and environment
//! profile to produce actionable insights for the user.
//!
//! Product Vision: Every piece of intelligence must feed into detecting concrete
//! problems on this machine and offering clear fixes.

use serde::{Deserialize, Serialize};

/// Severity of an issue or recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical - system is degraded or at risk
    Critical,
    /// Warning - something should be fixed soon
    Warning,
    /// Info - improvement opportunity
    Info,
}

/// A concrete issue or improvement opportunity detected on this machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaretakerIssue {
    /// Severity level
    pub severity: IssueSeverity,

    /// Short, human-readable title (one line)
    /// Example: "Disk 96% full - 30GB in package cache"
    pub title: String,

    /// Longer explanation of what's wrong and why it matters
    /// Example: "Your disk is almost full. Package cache can be safely cleaned to free 30GB."
    pub explanation: String,

    /// Specific action the user should take
    /// Example: "Run 'sudo annactl repair' to clean package cache"
    pub recommended_action: String,

    /// Optional: repair action ID that can be invoked programmatically
    /// Example: Some("disk-space")
    pub repair_action_id: Option<String>,

    /// Reference for more information (usually Arch Wiki)
    /// Example: "https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache"
    pub reference: Option<String>,

    /// Estimated impact of fixing this
    /// Example: "Frees 30GB disk space"
    pub estimated_impact: Option<String>,
}

impl CaretakerIssue {
    /// Create a new issue
    pub fn new(
        severity: IssueSeverity,
        title: impl Into<String>,
        explanation: impl Into<String>,
        recommended_action: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            title: title.into(),
            explanation: explanation.into(),
            recommended_action: recommended_action.into(),
            repair_action_id: None,
            reference: None,
            estimated_impact: None,
        }
    }

    /// Add a repair action ID
    pub fn with_repair_action(mut self, action_id: impl Into<String>) -> Self {
        self.repair_action_id = Some(action_id.into());
        self
    }

    /// Add a reference URL
    pub fn with_reference(mut self, url: impl Into<String>) -> Self {
        self.reference = Some(url.into());
        self
    }

    /// Add estimated impact
    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.estimated_impact = Some(impact.into());
        self
    }
}

/// Analysis result from the caretaker brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaretakerAnalysis {
    /// Top issues and recommendations, ordered by severity and importance
    pub issues: Vec<CaretakerIssue>,

    /// Overall system health: "healthy", "needs-attention", "critical"
    pub overall_status: String,

    /// One-line summary for quick display
    /// Example: "2 issues detected - disk space critical, TLP not enabled"
    pub summary: String,
}

impl CaretakerAnalysis {
    /// Create analysis with no issues (system healthy)
    pub fn healthy() -> Self {
        Self {
            issues: Vec::new(),
            overall_status: "healthy".to_string(),
            summary: "All systems healthy".to_string(),
        }
    }

    /// Create analysis from a list of issues
    pub fn from_issues(mut issues: Vec<CaretakerIssue>) -> Self {
        // Sort by severity (Critical > Warning > Info)
        issues.sort_by(|a, b| a.severity.cmp(&b.severity).reverse());

        let overall_status = if issues.iter().any(|i| i.severity == IssueSeverity::Critical) {
            "critical"
        } else if issues.iter().any(|i| i.severity == IssueSeverity::Warning) {
            "needs-attention"
        } else {
            "healthy"
        };

        let summary = if issues.is_empty() {
            "All systems healthy".to_string()
        } else {
            let critical_count = issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
            let warning_count = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();

            let mut parts = Vec::new();
            if critical_count > 0 {
                parts.push(format!("{} critical", critical_count));
            }
            if warning_count > 0 {
                parts.push(format!("{} warnings", warning_count));
            }

            format!("{} detected", parts.join(", "))
        };

        Self {
            issues,
            overall_status: overall_status.to_string(),
            summary,
        }
    }

    /// Get top N issues for display
    pub fn top_issues(&self, n: usize) -> &[CaretakerIssue] {
        &self.issues[..self.issues.len().min(n)]
    }
}

/// The Caretaker Brain - analyzes all available information and produces actionable insights
pub struct CaretakerBrain;

impl CaretakerBrain {
    /// Analyze the system and produce top issues/recommendations
    ///
    /// This is the core intelligence that ties together:
    /// - Health check results
    /// - System metrics
    /// - Predictive analysis
    /// - Environment profile
    ///
    /// Returns a prioritized list of what the user should care about
    pub fn analyze(
        health_results: Option<&[crate::ipc::HealthProbeResult]>,
        disk_analysis: Option<&crate::disk_analysis::DiskAnalysis>,
    ) -> CaretakerAnalysis {
        let mut issues = Vec::new();

        // 1. Analyze disk space (most common critical issue)
        if let Some(disk) = disk_analysis {
            if disk.usage_percent > 95.0 {
                issues.push(
                    CaretakerIssue::new(
                        IssueSeverity::Critical,
                        format!("Disk {}% full - system at risk", disk.usage_percent as u32),
                        "Your disk is critically full. This can cause system instability and data loss. Immediate action required.",
                        "Run 'sudo annactl repair' to clean up space"
                    )
                    .with_repair_action("disk-space")
                    .with_reference("https://wiki.archlinux.org/title/System_maintenance#Clean_the_filesystem")
                );
            } else if disk.usage_percent > 90.0 {
                let recommendations = disk.get_recommendations();
                let total_savings: u64 = recommendations.iter()
                    .map(|r| r.estimated_savings_bytes)
                    .sum();

                let savings_gb = total_savings / (1024 * 1024 * 1024);

                issues.push(
                    CaretakerIssue::new(
                        IssueSeverity::Critical,
                        format!("Disk {}% full - {}GB can be freed", disk.usage_percent as u32, savings_gb),
                        format!("Your disk is nearly full. Package cache and logs can be safely cleaned to free {}GB.", savings_gb),
                        "Run 'sudo annactl repair' to clean up space"
                    )
                    .with_repair_action("disk-space")
                    .with_impact(format!("Frees {}GB", savings_gb))
                    .with_reference("https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache")
                );
            } else if disk.usage_percent > 80.0 {
                issues.push(
                    CaretakerIssue::new(
                        IssueSeverity::Warning,
                        format!("Disk {}% full - cleanup recommended", disk.usage_percent as u32),
                        "Your disk is getting full. Consider cleaning package cache and logs.",
                        "Run 'annactl daily' to see cleanup recommendations"
                    )
                    .with_repair_action("disk-space")
                );
            }
        }

        // 2. Analyze health check results
        if let Some(health) = health_results {
            for result in health {
                // Only report failures and warnings
                if result.status == "fail" || result.status == "warn" {
                    let severity = if result.status == "fail" {
                        IssueSeverity::Critical
                    } else {
                        IssueSeverity::Warning
                    };

                    // Extract meaningful information from probe results
                    let (title, explanation, action) = Self::interpret_probe_result(result);

                    let mut issue = CaretakerIssue::new(severity, title, explanation, action);

                    // Map probe names to repair action IDs
                    issue.repair_action_id = Some(result.probe.clone());

                    issues.push(issue);
                }
            }
        }

        CaretakerAnalysis::from_issues(issues)
    }

    /// Interpret a probe result into human-readable terms
    fn interpret_probe_result(result: &crate::ipc::HealthProbeResult) -> (String, String, String) {
        // Extract message from probe details if available
        let message = result.details.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Issue detected");

        match result.probe.as_str() {
            "tlp-config" => (
                "TLP not properly configured".to_string(),
                format!("{} This affects battery life and power management.", message),
                "Run 'sudo annactl repair tlp-config' to enable TLP service".to_string(),
            ),
            "bluetooth-service" => (
                "Bluetooth service not working".to_string(),
                format!("{} Bluetooth functionality may not work.", message),
                "Run 'sudo annactl repair bluetooth-service' to fix".to_string(),
            ),
            "missing-firmware" => {
                let count = result.details.get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                (
                    format!("{} missing firmware file(s)", count),
                    "Your hardware may not function optimally due to missing firmware.".to_string(),
                    "Run 'annactl repair missing-firmware' for guidance".to_string(),
                )
            },
            "systemd-units" => (
                "Failed systemd services detected".to_string(),
                "Some system services are not running properly.".to_string(),
                "Run 'sudo annactl repair services-failed' to restart failed services".to_string(),
            ),
            _ => (
                format!("{} issue", result.probe),
                message.to_string(),
                format!("Run 'sudo annactl repair {}' to fix", result.probe),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_analysis() {
        let analysis = CaretakerAnalysis::healthy();
        assert_eq!(analysis.overall_status, "healthy");
        assert_eq!(analysis.issues.len(), 0);
        assert!(analysis.summary.contains("healthy"));
    }

    #[test]
    fn test_issue_ordering() {
        let issues = vec![
            CaretakerIssue::new(IssueSeverity::Info, "Info", "Info", "Fix"),
            CaretakerIssue::new(IssueSeverity::Critical, "Critical", "Critical", "Fix"),
            CaretakerIssue::new(IssueSeverity::Warning, "Warning", "Warning", "Fix"),
        ];

        let analysis = CaretakerAnalysis::from_issues(issues);

        // Should be sorted: Critical, Warning, Info
        assert_eq!(analysis.issues[0].severity, IssueSeverity::Critical);
        assert_eq!(analysis.issues[1].severity, IssueSeverity::Warning);
        assert_eq!(analysis.issues[2].severity, IssueSeverity::Info);
    }

    #[test]
    fn test_overall_status() {
        let critical = vec![
            CaretakerIssue::new(IssueSeverity::Critical, "Test", "Test", "Fix"),
        ];
        let analysis = CaretakerAnalysis::from_issues(critical);
        assert_eq!(analysis.overall_status, "critical");

        let warning = vec![
            CaretakerIssue::new(IssueSeverity::Warning, "Test", "Test", "Fix"),
        ];
        let analysis = CaretakerAnalysis::from_issues(warning);
        assert_eq!(analysis.overall_status, "needs-attention");

        let healthy = Vec::new();
        let analysis = CaretakerAnalysis::from_issues(healthy);
        assert_eq!(analysis.overall_status, "healthy");
    }
}
