//! Status command - system health with REAL analysis
//!
//! Phase 4.3: Enhanced with caretaker brain intelligence
//! Citation: [archwiki:System_maintenance]

use anna_common::caretaker_brain::{CaretakerBrain, IssueSeverity};
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

    // Run comprehensive health probes
    let probes = vec![
        "pacman-db".to_string(),
        "systemd-units".to_string(),
        "journal-errors".to_string(),
        "services-failed".to_string(),
        "tlp-config".to_string(),
        "bluetooth-service".to_string(),
        "missing-firmware".to_string(),
    ];

    let health_response = client.health_run(15000, probes).await?;
    let health_data = match health_response {
        ResponseData::HealthRun(data) => data,
        _ => anyhow::bail!("Invalid health response from daemon"),
    };

    // Do REAL disk analysis
    let disk_analysis = DiskAnalysis::analyze_root()?;

    // Use caretaker brain for intelligent analysis
    let caretaker_analysis = CaretakerBrain::analyze(
        Some(&health_data.results),
        Some(&disk_analysis)
    );

    // Determine status level from caretaker analysis
    let status_level = match caretaker_analysis.overall_status.as_str() {
        "critical" => StatusLevel::Critical,
        "needs-attention" => StatusLevel::Warning,
        _ => StatusLevel::Success,
    };

    // Main status section
    let mut section = Section::new(
        format!("System Status - {}", caretaker_analysis.overall_status),
        status_level,
        use_color,
    );

    section.add_line(format!("State: {}", state));
    section.add_line(&caretaker_analysis.summary);
    section.add_blank();

    // Disk status
    let disk_status = format!(
        "Disk: {:.1}% used ({} available)",
        disk_analysis.usage_percent,
        format_bytes(disk_analysis.available_bytes)
    );
    section.add_line(disk_status);

    // Health summary
    section.add_line(format!(
        "Health: {} ok, {} warnings, {} failures",
        health_data.summary.ok,
        health_data.summary.warn,
        health_data.summary.fail
    ));

    print!("{}", section.render());

    // Show detailed analysis from caretaker brain
    if !caretaker_analysis.issues.is_empty() {
        println!("\n📊 Detailed Analysis:\n");

        // Group issues by severity
        let critical: Vec<_> = caretaker_analysis.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .collect();
        let warnings: Vec<_> = caretaker_analysis.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .collect();
        let info: Vec<_> = caretaker_analysis.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Info)
            .collect();

        // Show critical issues first
        if !critical.is_empty() {
            println!("🔴 Critical Issues:\n");
            for issue in critical {
                println!("  • {}", issue.title);
                println!("    {}", issue.explanation);
                println!("    💡 {}", issue.recommended_action);
                if let Some(impact) = &issue.estimated_impact {
                    println!("    📊 {}", impact);
                }
                if let Some(reference) = &issue.reference {
                    println!("    📚 {}", reference);
                }
                println!();
            }
        }

        // Then warnings
        if !warnings.is_empty() {
            println!("⚠️  Warnings:\n");
            for issue in warnings {
                println!("  • {}", issue.title);
                println!("    {}", issue.explanation);
                println!("    💡 {}", issue.recommended_action);
                if let Some(reference) = &issue.reference {
                    println!("    📚 {}", reference);
                }
                println!();
            }
        }

        // Then info
        if !info.is_empty() {
            println!("ℹ️  Recommendations:\n");
            for issue in info {
                println!("  • {}", issue.title);
                println!("    {}", issue.explanation);
                println!("    💡 {}", issue.recommended_action);
                if let Some(reference) = &issue.reference {
                    println!("    📚 {}", reference);
                }
                println!();
            }
        }

        println!("💡 Next Steps:");
        let has_critical = caretaker_analysis.issues.iter()
            .any(|i| i.severity == IssueSeverity::Critical);
        if has_critical {
            println!("   🚨 Run 'sudo annactl repair' to fix critical issues");
        } else {
            println!("   Run 'sudo annactl repair' to attempt automatic fixes");
        }
        println!("   Or review each issue and fix manually");
    } else {
        println!("\n✅ All systems healthy\n");
        println!("💡 Your system is running smoothly. No action needed.");
    }

    println!();

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let exit_code = if caretaker_analysis.overall_status == "critical" {
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
        citation: "[archwiki:System_maintenance]".to_string(),
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
