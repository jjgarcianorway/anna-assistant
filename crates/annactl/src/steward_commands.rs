//! Steward commands - lifecycle management operations
//!
//! Phase 0.9: System health, updates, and audit commands
//! Citation: [archwiki:System_maintenance]

use anna_common::ipc::{ResponseData};
use anyhow::{Context, Result};
use std::time::Instant;

use crate::errors::*;
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;

/// Execute 'status' command - show comprehensive system health
pub async fn execute_status_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // Connect to daemon
    let mut client = RpcClient::connect().await?;

    // Call system_health RPC
    let response = client
        .system_health()
        .await
        .context("Failed to get system health")?;

    // Parse response
    match response {
        ResponseData::HealthReport(report) => {
            // Display health report
            println!("┌─────────────────────────────────────────────────────────");
            println!("│ SYSTEM HEALTH REPORT");
            println!("├─────────────────────────────────────────────────────────");
            println!("│ Status:    {}", report.overall_status);
            println!("│ Timestamp: {}", report.timestamp);
            println!("│ State:     {}", state);
            println!("├─────────────────────────────────────────────────────────");

            // Services
            let failed_services: Vec<_> = report
                .services
                .iter()
                .filter(|s| s.state == "failed")
                .collect();

            if !failed_services.is_empty() {
                println!("│ FAILED SERVICES:");
                for svc in &failed_services {
                    println!("│   • {} ({})", svc.name, svc.state);
                }
            } else {
                println!("│ All critical services: OK");
            }

            // Packages
            let updates_available: Vec<_> = report
                .packages
                .iter()
                .filter(|p| p.update_available)
                .collect();

            if !updates_available.is_empty() {
                println!("│ UPDATES AVAILABLE: {}", updates_available.len());
                for pkg in updates_available.iter().take(5) {
                    println!("│   • {} {}", pkg.name, pkg.version);
                }
                if updates_available.len() > 5 {
                    println!("│   ... and {} more", updates_available.len() - 5);
                }
            } else {
                println!("│ System is up to date");
            }

            // Log issues
            if !report.log_issues.is_empty() {
                println!("│ LOG ISSUES: {}", report.log_issues.len());
                for issue in report.log_issues.iter().take(3) {
                    println!("│   • [{}] {}", issue.severity, issue.message.chars().take(50).collect::<String>());
                }
                if report.log_issues.len() > 3 {
                    println!("│   ... and {} more", report.log_issues.len() - 3);
                }
            }

            // Recommendations
            if !report.recommendations.is_empty() {
                println!("├─────────────────────────────────────────────────────────");
                println!("│ RECOMMENDATIONS:");
                for rec in &report.recommendations {
                    println!("│   • {}", rec);
                }
            }

            println!("└─────────────────────────────────────────────────────────");
            println!();
            println!("{}", report.citation);

            // Log command
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                command: "status".to_string(),
                state: state.to_string(),
                allowed: Some(true),
                args: vec![],
                exit_code: EXIT_SUCCESS,
                duration_ms,
                ok: true,
                error: None,
                citation: report.citation,
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_SUCCESS);
        }
        _ => {
            anyhow::bail!("Unexpected response type from daemon");
        }
    }
}

/// Execute 'update' command - perform system update
pub async fn execute_update_command(
    dry_run: bool,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // Connect to daemon
    let mut client = RpcClient::connect().await?;

    // Call system_update RPC
    let response = client
        .system_update(dry_run)
        .await
        .context("Failed to perform system update")?;

    // Parse response
    match response {
        ResponseData::UpdateReport(report) => {
            // Display update report
            println!("┌─────────────────────────────────────────────────────────");
            if dry_run {
                println!("│ SYSTEM UPDATE (DRY RUN)");
            } else {
                println!("│ SYSTEM UPDATE");
            }
            println!("├─────────────────────────────────────────────────────────");
            println!("│ Status:    {}", if report.success { "SUCCESS" } else { "FAILED" });
            println!("│ Timestamp: {}", report.timestamp);
            println!("├─────────────────────────────────────────────────────────");

            // Packages updated
            if !report.packages_updated.is_empty() {
                println!("│ PACKAGES UPDATED: {}", report.packages_updated.len());
                for pkg in report.packages_updated.iter().take(10) {
                    println!("│   • {} {} → {}", pkg.name, pkg.old_version, pkg.new_version);
                }
                if report.packages_updated.len() > 10 {
                    println!("│   ... and {} more", report.packages_updated.len() - 10);
                }
            } else {
                println!("│ No packages to update");
            }

            // Services restarted
            if !report.services_restarted.is_empty() {
                println!("│ SERVICES RESTARTED:");
                for svc in &report.services_restarted {
                    println!("│   • {}", svc);
                }
            }

            // Snapshot
            if let Some(snapshot_path) = &report.snapshot_path {
                println!("│ Snapshot: {}", snapshot_path);
            }

            println!("├─────────────────────────────────────────────────────────");
            println!("│ {}", report.message);
            println!("└─────────────────────────────────────────────────────────");
            println!();
            println!("{}", report.citation);

            // Log command
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let args = if dry_run { vec!["--dry-run".to_string()] } else { vec![] };
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                command: "update".to_string(),
                state: state.to_string(),
                allowed: Some(true),
                args,
                exit_code: if report.success { EXIT_SUCCESS } else { EXIT_COMMAND_FAILED },
                duration_ms,
                ok: report.success,
                error: None,
                citation: report.citation,
            };
            let _ = log_entry.write();

            std::process::exit(if report.success { EXIT_SUCCESS } else { EXIT_COMMAND_FAILED });
        }
        _ => {
            anyhow::bail!("Unexpected response type from daemon");
        }
    }
}

/// Execute 'audit' command - perform system audit
pub async fn execute_audit_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // Connect to daemon
    let mut client = RpcClient::connect().await?;

    // Call system_audit RPC
    let response = client
        .system_audit()
        .await
        .context("Failed to perform system audit")?;

    // Parse response
    match response {
        ResponseData::AuditReport(report) => {
            // Display audit report
            println!("┌─────────────────────────────────────────────────────────");
            println!("│ SYSTEM AUDIT REPORT");
            println!("├─────────────────────────────────────────────────────────");
            println!("│ Compliance: {}", if report.compliant { "✓ PASS" } else { "✗ FAIL" });
            println!("│ Timestamp:  {}", report.timestamp);
            println!("├─────────────────────────────────────────────────────────");

            // Integrity checks
            let failed_integrity: Vec<_> = report
                .integrity
                .iter()
                .filter(|i| !i.passed)
                .collect();

            if !failed_integrity.is_empty() {
                println!("│ INTEGRITY FAILURES: {}", failed_integrity.len());
                for check in &failed_integrity {
                    println!("│   • {} ({}): {}", check.component, check.check_type, check.details);
                }
            } else {
                println!("│ All integrity checks: PASSED ({} checks)", report.integrity.len());
            }

            // Security findings
            if !report.security_findings.is_empty() {
                println!("│ SECURITY FINDINGS: {}", report.security_findings.len());
                for finding in &report.security_findings {
                    println!("│   • [{}] {}", finding.severity.to_uppercase(), finding.description);
                    println!("│     → {}", finding.recommendation);
                }
            } else {
                println!("│ No security findings");
            }

            // Configuration issues
            if !report.config_issues.is_empty() {
                println!("│ CONFIGURATION ISSUES: {}", report.config_issues.len());
                for issue in &report.config_issues {
                    println!("│   • {}: {}", issue.file, issue.issue);
                    println!("│     Expected: {}", issue.expected);
                    println!("│     Actual:   {}", issue.actual);
                }
            } else {
                println!("│ No configuration issues");
            }

            println!("├─────────────────────────────────────────────────────────");
            println!("│ {}", report.message);
            println!("└─────────────────────────────────────────────────────────");
            println!();
            println!("{}", report.citation);

            // Log command
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                command: "audit".to_string(),
                state: state.to_string(),
                allowed: Some(true),
                args: vec![],
                exit_code: EXIT_SUCCESS,
                duration_ms,
                ok: true,
                error: None,
                citation: report.citation,
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_SUCCESS);
        }
        _ => {
            anyhow::bail!("Unexpected response type from daemon");
        }
    }
}
