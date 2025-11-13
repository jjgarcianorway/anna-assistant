//! Status command - system health with REAL analysis
//!
//! Phase 4.1: Make status as useful as daily
//! Citation: [archwiki:System_maintenance]

use anna_common::disk_analysis::DiskAnalysis;
use anna_common::display::*;
use anna_common::ipc::ResponseData;
use anyhow::{Context, Result};
use std::time::Instant;

use crate::context_detection;
use crate::errors::*;
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;

/// Execute 'status' command - show system health with intelligent analysis
pub async fn execute_status_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;
    let use_color = context_detection::should_use_color();

    // Get system health from daemon
    let response = client
        .system_health()
        .await
        .context("Failed to get system health")?;

    let report = match response {
        ResponseData::HealthReport(report) => report,
        _ => anyhow::bail!("Invalid response from daemon"),
    };

    // Do REAL disk analysis
    let disk_analysis = DiskAnalysis::analyze_root()?;

    // Determine status level
    let failed_services: Vec<_> = report
        .services
        .iter()
        .filter(|s| s.state == "failed")
        .collect();

    let has_failures = !failed_services.is_empty() || !report.log_issues.is_empty();
    let disk_critical = disk_analysis.usage_percent > 90.0;
    let disk_warning = disk_analysis.usage_percent > 80.0;

    let status_level = if has_failures || disk_critical {
        StatusLevel::Critical
    } else if disk_warning || !report.log_issues.is_empty() {
        StatusLevel::Warning
    } else {
        StatusLevel::Success
    };

    // Main status section
    let mut section = Section::new(
        format!("System Status - {}", report.overall_status),
        status_level,
        use_color,
    );

    section.add_line(format!("State: {}", state));
    section.add_line(format!("Timestamp: {}", report.timestamp));
    section.add_blank();

    // Disk status
    let disk_status = format!(
        "Disk: {:.1}% used ({} available)",
        disk_analysis.usage_percent,
        format_bytes(disk_analysis.available_bytes)
    );
    section.add_line(disk_status);

    // Services
    if !failed_services.is_empty() {
        section.add_blank();
        section.add_line(format!("Failed services: {}", failed_services.len()));
    }

    // Updates
    let updates: Vec<_> = report
        .packages
        .iter()
        .filter(|p| p.update_available)
        .collect();
    if !updates.is_empty() {
        section.add_line(format!("Updates available: {}", updates.len()));
    }

    print!("{}", section.render());

    // Show details if there are issues
    if has_failures || disk_warning {
        println!("\n📊 Details:\n");

        // Failed services details
        for svc in &failed_services {
            println!("  ❌ Service: {} ({})", svc.name, svc.state);
        }

        // Log issues (top 3)
        for issue in report.log_issues.iter().take(3) {
            let msg: String = issue.message.chars().take(80).collect();
            println!("  ⚠️  Log: [{}] {}", issue.severity, msg);
        }
        if report.log_issues.len() > 3 {
            println!("     ... and {} more log issues", report.log_issues.len() - 3);
        }

        // Disk analysis if needed
        if disk_warning {
            println!("\n📁 Disk Space:\n");
            for consumer in disk_analysis.top_consumers.iter().take(3) {
                println!("  {} {:<20} {:>10}",
                    consumer.category.icon(),
                    consumer.category.name(),
                    consumer.size_human
                );
            }
        }

        // Recommendations
        println!("\n💡 Recommendations:\n");
        if disk_critical {
            println!("  1. Run 'annactl daily' for detailed disk analysis and cleanup steps");
        } else if disk_warning {
            println!("  1. Run 'annactl daily' to see disk cleanup options");
        }
        if has_failures {
            println!("  2. Run 'annactl repair' to attempt automatic fixes");
        }
        if !report.log_issues.is_empty() {
            println!("  3. Review logs with: journalctl -p err -n 50");
        }
    } else {
        println!("\n✅ All systems healthy\n");
    }

    println!("\n{}\n", report.citation);

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let exit_code = if has_failures || disk_critical {
        1
    } else {
        EXIT_SUCCESS
    };

    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "status".to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code,
        citation: report.citation,
        duration_ms,
        ok: exit_code == EXIT_SUCCESS,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1}TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
