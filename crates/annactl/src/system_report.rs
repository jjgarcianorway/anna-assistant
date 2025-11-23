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

/// Check if a query is asking for a full system report or status
/// Beta.243: Expanded keyword coverage for status queries
/// Beta.244: Added temporal and importance-based status patterns
/// Beta.254: Now uses shared normalization from unified_query_handler
pub fn is_system_report_query(query: &str) -> bool {
    // Beta.254: Use shared normalization for consistent behavior
    let query_lower = crate::unified_query_handler::normalize_query_for_intent(query);

    // Full report phrasings (original behavior)
    let is_full_report = (query_lower.contains("full report") ||
                         query_lower.contains("complete report") ||
                         query_lower.contains("system report")) &&
                        (query_lower.contains("computer") ||
                         query_lower.contains("system") ||
                         query_lower.contains("machine"));

    if is_full_report {
        return true;
    }

    // Beta.243: Expanded status query keywords
    // These are lighter-weight status checks vs full diagnostic
    let status_keywords = [
        "show me status",
        "system status",
        "what's running",
        "system information",
        "system info",
        // Beta.243: New status keywords
        "current status",
        "what is the current status",
        "what is the status",
        "system state",
        "show system state",
        "what's happening on my system",
        "what's happening",
        // Beta.249: Removed "how is my system" and "how is the system" - they're diagnostic patterns
        // Beta.251: "status of" patterns
        "status of my system",
        "status of the system",
        "status of this system",
        "status of my machine",
        "status of my computer",
        "status of this machine",
        "status of this computer",
        // Beta.251: "[my/this] [computer/machine] status" patterns
        "my computer status",
        "my machine status",
        "my system status",  // for consistency
        "this computer status",
        "this machine status",
        "this system status",  // for consistency
        "computer status",
        "machine status",
        // Beta.251: "status current" terse pattern
        "status current",
        "current system status",
        // Beta.253: Category C - "the machine/computer/system status" variants
        "the machine status",
        "the computer status",
        "the system status",
        "check the machine status",
        "check the computer status",
        "check the system status",
        "the machine's status",
        "the computer's status",
        "the system's status",
        // Beta.275: Additional status report patterns
        "extensive status report",
        "detailed status report",
    ];

    for keyword in &status_keywords {
        if query_lower.contains(keyword) {
            return true;
        }
    }

    // Beta.244: Temporal and importance-based status patterns
    // Beta.255: Extended with recency and "what happened" patterns
    // These imply "right now" or "anything important" combined with system references
    //
    // Temporal indicators: today, now, currently, right now, recently, lately, this morning
    // Recency indicators: what happened, any events, anything changed
    // Importance indicators: important, critical, wrong, issues, problems, review
    // System references: this system, this machine, this computer, my system, my machine

    let temporal_indicators = ["today", "now", "currently", "right now", "recently", "lately",
                               "this morning", "this afternoon", "this evening", "in the last hour"];
    let recency_indicators = [
        "what happened",
        "anything happened",
        "any events",
        "anything changed",
        "any changes",
    ];
    let importance_indicators = [
        "anything important",
        "anything critical",
        "anything wrong",
        "any issues",
        "any problems",
        "important to review",
        "to review",
        "should know",
        "need to know",
    ];
    let system_references = [
        "this system",
        "this machine",
        "this computer",
        "my system",
        "my machine",
        "my computer",
        "the system",
        "the machine",
    ];

    // Check if query has temporal indicator + system reference
    let has_temporal = temporal_indicators.iter().any(|ind| query_lower.contains(ind));
    let has_recency = recency_indicators.iter().any(|ind| query_lower.contains(ind));
    let has_importance = importance_indicators.iter().any(|ind| query_lower.contains(ind));
    let has_system_ref = system_references.iter().any(|ind| query_lower.contains(ind));

    // Beta.255: Match if: (temporal OR recency OR importance) AND system_reference
    // Examples: "how is my system today", "what happened on this machine", "anything important on my system"
    if (has_temporal || has_recency || has_importance) && has_system_ref {
        return true;
    }

    // Beta.244: Also match standalone importance queries that clearly reference system context
    // Example: "anything important to review on this system today"
    if has_importance && has_system_ref {
        return true;
    }

    false
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
