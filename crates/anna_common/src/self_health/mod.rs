//! Self-Health Module v0.7.0
//!
//! Anna monitors her own health and can auto-repair safe issues.
//! Components monitored: daemon, llm, model, tools, permissions, config.

pub mod probes;
pub mod repair;
pub mod types;

pub use probes::*;
pub use repair::*;
pub use types::*;

use crate::THIN_SEPARATOR;
use owo_colors::OwoColorize;

/// Run all self-health probes and return aggregate status
pub fn run_all_probes() -> SelfHealthReport {
    let daemon = probes::check_daemon();
    let llm = probes::check_llm_backend();
    let model = probes::check_model_availability();
    let tools = probes::check_tools_catalog();
    let permissions = probes::check_permissions();
    let config = probes::check_config();

    let components = vec![daemon, llm, model, tools, permissions, config];

    let overall_status = if components.iter().all(|c| c.status == ComponentStatus::Healthy) {
        OverallHealth::Healthy
    } else if components.iter().any(|c| c.status == ComponentStatus::Critical) {
        OverallHealth::Critical
    } else {
        OverallHealth::Degraded
    };

    SelfHealthReport {
        overall: overall_status,
        components,
        repairs_available: vec![],
        repairs_executed: vec![],
    }
}

/// Run probes and attempt auto-repairs for safe issues
pub fn run_with_auto_repair() -> SelfHealthReport {
    let mut report = run_all_probes();

    // Identify repairs needed
    for component in &report.components {
        if component.status != ComponentStatus::Healthy {
            if let Some(repair) = repair::plan_repair(component) {
                report.repairs_available.push(repair);
            }
        }
    }

    // Execute safe auto-repairs
    for repair in &report.repairs_available {
        if repair.safety == RepairSafety::Auto {
            match repair::execute_repair(repair) {
                Ok(result) => {
                    report.repairs_executed.push(result);
                }
                Err(e) => {
                    report.repairs_executed.push(RepairResult {
                        action: repair.action.clone(),
                        success: false,
                        message: format!("Repair failed: {}", e),
                    });
                }
            }
        }
    }

    // Re-run probes after repairs to get updated status
    if !report.repairs_executed.is_empty() {
        let updated = run_all_probes();
        report.overall = updated.overall;
        report.components = updated.components;
    }

    report
}

/// Format self-health report for display
pub fn format_report(report: &SelfHealthReport) -> String {
    let mut output = String::new();

    // Header
    output.push_str("\n[SELF-HEALTH]\n");
    output.push_str(THIN_SEPARATOR);
    output.push('\n');

    // Overall status
    let status_str = match report.overall {
        OverallHealth::Healthy => "[OK]".bright_green().to_string(),
        OverallHealth::Degraded => "[DEGRADED]".yellow().to_string(),
        OverallHealth::Critical => "[CRITICAL]".bright_red().to_string(),
        OverallHealth::Unknown => "[UNKNOWN]".dimmed().to_string(),
    };
    output.push_str(&format!("Status: {}\n\n", status_str));

    // Components
    output.push_str("[COMPONENTS]\n");
    for component in &report.components {
        let icon = match component.status {
            ComponentStatus::Healthy => "+".bright_green().to_string(),
            ComponentStatus::Degraded => "~".yellow().to_string(),
            ComponentStatus::Critical => "!".bright_red().to_string(),
            ComponentStatus::Unknown => "?".dimmed().to_string(),
        };
        output.push_str(&format!(
            "  {} {}: {}\n",
            icon,
            component.name,
            component.message
        ));
    }

    // Repairs executed
    if !report.repairs_executed.is_empty() {
        output.push_str("\n[AUTO-REPAIRS]\n");
        for repair in &report.repairs_executed {
            let icon = if repair.success {
                "+".bright_green().to_string()
            } else {
                "!".bright_red().to_string()
            };
            output.push_str(&format!(
                "  {} {:?}: {}\n",
                icon, repair.action, repair.message
            ));
        }
    }

    // Pending repairs (warn-only)
    let warn_only: Vec<_> = report
        .repairs_available
        .iter()
        .filter(|r| r.safety == RepairSafety::WarnOnly)
        .collect();
    if !warn_only.is_empty() {
        output.push_str("\n[MANUAL ACTION REQUIRED]\n");
        for repair in warn_only {
            output.push_str(&format!(
                "  * {:?}: {} (run: {})\n",
                repair.action, repair.description, repair.command
            ));
        }
    }

    output.push_str(THIN_SEPARATOR);
    output.push('\n');

    output
}

/// Get a one-line summary for version output
pub fn summary_line(report: &SelfHealthReport) -> String {
    match report.overall {
        OverallHealth::Healthy => {
            format!(
                "{} (all components healthy)",
                "OK".bright_green()
            )
        }
        OverallHealth::Degraded => {
            let degraded: Vec<_> = report
                .components
                .iter()
                .filter(|c| c.status == ComponentStatus::Degraded)
                .map(|c| c.name.as_str())
                .collect();
            format!(
                "{} ({})",
                "DEGRADED".yellow(),
                degraded.join(", ")
            )
        }
        OverallHealth::Critical => {
            let critical: Vec<_> = report
                .components
                .iter()
                .filter(|c| c.status == ComponentStatus::Critical)
                .map(|c| c.name.as_str())
                .collect();
            format!(
                "{} ({})",
                "CRITICAL".bright_red(),
                critical.join(", ")
            )
        }
        OverallHealth::Unknown => {
            format!("{} (not yet checked)", "UNKNOWN".dimmed())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overall_health_healthy() {
        let components = vec![
            ComponentHealth {
                name: "test".to_string(),
                status: ComponentStatus::Healthy,
                message: "OK".to_string(),
                details: None,
            },
        ];
        let report = SelfHealthReport {
            overall: OverallHealth::Healthy,
            components,
            repairs_available: vec![],
            repairs_executed: vec![],
        };
        assert!(matches!(report.overall, OverallHealth::Healthy));
    }

    #[test]
    fn test_summary_line_healthy() {
        let report = SelfHealthReport {
            overall: OverallHealth::Healthy,
            components: vec![],
            repairs_available: vec![],
            repairs_executed: vec![],
        };
        let summary = summary_line(&report);
        assert!(summary.contains("all components healthy"));
    }
}
