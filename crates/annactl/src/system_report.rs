//! Unified System Report Generator
//!
//! Version 150: Single source of truth for system reports
//! Used by both CLI and TUI to ensure identical output
//!
//! Rules:
//! - Uses telemetry_truth for verified data only
//! - No hallucinations, no guessing
//! - Same input → same output (deterministic)
//! - Clean, professional formatting

use crate::system_query::query_system_telemetry;
use crate::telemetry_truth::{VerifiedSystemReport, HealthStatus};
use anyhow::Result;

/// Generate a complete system report
///
/// This is the single entry point for "write a full report about my computer"
/// queries. It produces identical output for CLI and TUI.
pub fn generate_full_report() -> Result<String> {
    let telemetry = query_system_telemetry()?;
    let verified = VerifiedSystemReport::from_telemetry(&telemetry);

    let mut report = String::new();

    // Header
    report.push_str(&format!("# System Report: {}\n\n", verified.hostname.display()));

    // Health Summary (most important first)
    report.push_str("## Health Summary\n\n");
    match verified.health_summary.overall_status {
        HealthStatus::Healthy => {
            report.push_str("✅ **Status**: Healthy\n\n");
            for info in &verified.health_summary.info {
                report.push_str(&format!("- {}\n", info));
            }
        }
        HealthStatus::Warning => {
            report.push_str("⚠️  **Status**: Warnings Detected\n\n");
            for warning in &verified.health_summary.warnings {
                report.push_str(&format!("- ⚠️  {}\n", warning));
            }
        }
        HealthStatus::Critical => {
            report.push_str("🔴 **Status**: Critical Issues\n\n");
            for issue in &verified.health_summary.critical_issues {
                report.push_str(&format!("- 🔴 {}\n", issue));
            }
            if !verified.health_summary.warnings.is_empty() {
                report.push_str("\n**Additional Warnings**:\n");
                for warning in &verified.health_summary.warnings {
                    report.push_str(&format!("- ⚠️  {}\n", warning));
                }
            }
        }
    }
    report.push('\n');

    // Hardware
    report.push_str("## Hardware\n\n");
    report.push_str(&format!("**CPU**: {} ({} cores)\n",
        verified.cpu_model.display(),
        verified.cpu_cores.display()));
    report.push_str(&format!("**Load Average**: {}\n", verified.cpu_load.display()));
    report.push_str(&format!("**RAM**: {} GB used / {} GB total ({} %)\n",
        verified.ram_used_gb.display(),
        verified.ram_total_gb.display(),
        verified.ram_percent.display()));
    report.push_str(&format!("**GPU**: {}\n", verified.gpu.display()));
    report.push('\n');

    // Storage
    report.push_str("## Storage\n\n");
    for disk in &verified.storage {
        report.push_str(&format!("**{}**:\n", disk.mount_point));
        report.push_str(&format!("  - Total: {} GB\n", disk.total_gb.display()));
        report.push_str(&format!("  - Used: {} GB ({} %)\n",
            disk.used_gb.display(),
            disk.used_percent.display()));
        report.push_str(&format!("  - Free: {} GB\n", disk.free_gb.display()));
        report.push('\n');
    }

    // System
    report.push_str("## System Information\n\n");
    report.push_str(&format!("**OS**: {}\n", verified.os_name.display()));
    report.push_str(&format!("**Kernel**: {}\n", verified.kernel_version.display()));
    report.push_str(&format!("**Hostname**: {}\n", verified.hostname.display()));
    report.push_str(&format!("**Uptime**: {}\n", verified.uptime.display()));
    report.push('\n');

    // Desktop
    report.push_str("## Desktop Environment\n\n");
    report.push_str(&format!("**Desktop**: {}\n", verified.desktop_environment.display()));
    report.push_str(&format!("**Window Manager**: {}\n", verified.window_manager.display()));
    report.push_str(&format!("**Display Protocol**: {}\n", verified.display_protocol.display()));
    report.push('\n');

    // Network
    report.push_str("## Network\n\n");
    report.push_str(&format!("**Status**: {}\n", verified.network_status.display()));
    report.push_str(&format!("**Primary Interface**: {}\n", verified.primary_interface.display()));

    if !verified.ip_addresses.is_empty() {
        report.push_str("**IP Addresses**:\n");
        for ip in &verified.ip_addresses {
            report.push_str(&format!("  - {}\n", ip.display()));
        }
    } else {
        report.push_str("**IP Addresses**: None detected\n");
    }
    report.push('\n');

    // Services
    if !verified.failed_services.is_empty() {
        report.push_str("## Failed Services\n\n");
        for service in &verified.failed_services {
            report.push_str(&format!("- ❌ {}\n", service));
        }
        report.push_str("\nRun `systemctl --failed` for details.\n\n");
    }

    // Footer
    report.push_str("---\n");
    report.push_str("*Report generated from verified system telemetry*\n");
    report.push_str("*All values are real - no guesses or defaults*\n");

    Ok(report)
}

/// Generate a short system summary (for status display)
pub fn generate_short_summary() -> Result<String> {
    let telemetry = query_system_telemetry()?;
    let verified = VerifiedSystemReport::from_telemetry(&telemetry);

    let mut summary = String::new();

    // One-line status
    match verified.health_summary.overall_status {
        HealthStatus::Healthy => summary.push_str("✅ All systems nominal"),
        HealthStatus::Warning => summary.push_str("⚠️  System warnings detected"),
        HealthStatus::Critical => summary.push_str("🔴 Critical issues require attention"),
    }

    summary.push_str(&format!(" | CPU: {} @ {}% | RAM: {} / {} GB",
        verified.cpu_model.display().split_whitespace().nth(2).unwrap_or("Unknown"),
        verified.cpu_load.display(),
        verified.ram_used_gb.display(),
        verified.ram_total_gb.display()));

    Ok(summary)
}

/// Check if a query is asking for a full system report
pub fn is_system_report_query(query: &str) -> bool {
    let query_lower = query.to_lowercase();

    // Match various phrasings
    (query_lower.contains("full report") ||
     query_lower.contains("complete report") ||
     query_lower.contains("system report")) &&
    (query_lower.contains("computer") ||
     query_lower.contains("system") ||
     query_lower.contains("machine"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_report_query() {
        assert!(is_system_report_query("write a full report about my computer please"));
        assert!(is_system_report_query("give me a complete report of my system"));
        assert!(is_system_report_query("show me a full system report"));

        assert!(!is_system_report_query("what is my CPU?"));
        assert!(!is_system_report_query("how much RAM do I have?"));
    }
}
