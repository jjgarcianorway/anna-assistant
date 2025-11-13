//! Daily checkup command - REAL analysis with USEFUL output
//!
//! Anna's caretaker brain analyzes your system and provides prioritized insights
//! Citation: [archwiki:System_maintenance]

use crate::context_detection;
use crate::errors::*;
use crate::first_run;
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;
use anna_common::caretaker_brain::{CaretakerBrain, IssueSeverity};
use anna_common::disk_analysis::{DiskAnalysis, RecommendationRisk};
use anna_common::display::*;
use anna_common::ipc::{HealthRunData, ResponseData};
use anna_common::profile::MachineProfile;
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::time::Instant;

/// Execute daily checkup command
pub async fn execute_daily_command(
    json: bool,
    state: &str,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let use_color = context_detection::should_use_color();

    // Phase 4.3: Detect first run and show welcome message
    let is_first_run = first_run::is_first_run();
    if is_first_run && !json {
        first_run::display_first_run_message(use_color);
    }

    let mut client = RpcClient::connect()
        .await.context("Failed to connect to daemon")?;

    // Run basic health probes
    let probes = vec![
        "pacman-db".to_string(),
        "systemd-units".to_string(),
        "journal-errors".to_string(),
        "services-failed".to_string(),
    ];

    let response = client.health_run(10000, probes).await?;
    let health_data = match response {
        ResponseData::HealthRun(data) => data,
        _ => anyhow::bail!("Invalid response from daemon"),
    };

    // Do REAL disk analysis
    let disk_analysis = DiskAnalysis::analyze_root()?;

    // Detect machine profile (Phase 4.6)
    let profile = MachineProfile::detect();

    // Use caretaker brain to analyze everything and prioritize issues (profile-aware)
    let caretaker_analysis = CaretakerBrain::analyze(
        Some(&health_data.results),
        Some(&disk_analysis),
        profile
    );

    // Determine overall status from caretaker analysis
    let status_level = match caretaker_analysis.overall_status.as_str() {
        "critical" => StatusLevel::Critical,
        "needs-attention" => StatusLevel::Warning,
        _ => StatusLevel::Success,
    };

    if json {
        // JSON output for automation
        let json_output = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "status": caretaker_analysis.overall_status,
            "summary": caretaker_analysis.summary,
            "issues": caretaker_analysis.issues,
            "health_summary": {
                "ok": health_data.summary.ok,
                "warn": health_data.summary.warn,
                "fail": health_data.summary.fail,
            },
            "disk": {
                "total_gb": disk_analysis.total_bytes / (1024 * 1024 * 1024),
                "used_gb": disk_analysis.used_bytes / (1024 * 1024 * 1024),
                "available_gb": disk_analysis.available_bytes / (1024 * 1024 * 1024),
                "usage_percent": disk_analysis.usage_percent,
            },
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Human output - caretaker brain provides prioritized insights
        print_daily_output(&health_data, &disk_analysis, &caretaker_analysis, status_level, use_color, is_first_run);
    }

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let exit_code = if caretaker_analysis.overall_status == "critical" { 1 } else { EXIT_SUCCESS };
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "daily".to_string(),
        allowed: Some(true),
        args: if json { vec!["--json".to_string()] } else { vec![] },
        exit_code,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: exit_code == EXIT_SUCCESS,
        error: None,
    };
    let _ = log_entry.write();

    // Phase 4.3: Mark first run as complete after successful scan
    if is_first_run && exit_code == EXIT_SUCCESS {
        let _ = first_run::mark_first_run_complete();
    }

    std::process::exit(exit_code);
}

/// Print beautiful, useful daily output with caretaker brain insights
fn print_daily_output(
    health_data: &HealthRunData,
    disk_analysis: &DiskAnalysis,
    caretaker_analysis: &anna_common::caretaker_brain::CaretakerAnalysis,
    status_level: StatusLevel,
    use_color: bool,
    is_first_run: bool,
) {
    // Main status section with caretaker summary
    let header = if is_first_run {
        format!("First System Scan - {}", Utc::now().format("%Y-%m-%d %H:%M"))
    } else {
        format!("Daily System Check - {}", Utc::now().format("%Y-%m-%d %H:%M"))
    };

    let mut section = Section::new(
        header,
        status_level,
        use_color,
    );

    section.add_line(&caretaker_analysis.summary);
    section.add_blank();

    // Basic metrics
    let health_summary = format!(
        "Health: {} ok, {} warnings, {} failures",
        health_data.summary.ok,
        health_data.summary.warn,
        health_data.summary.fail
    );
    section.add_line(health_summary);

    let disk_status = format!(
        "Disk: {:.1}% used ({} / {} total)",
        disk_analysis.usage_percent,
        format_bytes(disk_analysis.available_bytes),
        format_bytes(disk_analysis.total_bytes)
    );
    section.add_line(disk_status);

    print!("{}", section.render());

    // Display caretaker brain insights - prioritized and actionable
    if caretaker_analysis.issues.is_empty() {
        println!("\n✅ All systems healthy!");
        println!("\n💡 Your system is running smoothly. No action needed.");
    } else {
        println!("\n🔍 Issues Detected (prioritized):\n");

        // Display top 5 issues from caretaker brain
        for (idx, issue) in caretaker_analysis.top_issues(5).iter().enumerate() {
            let severity_icon = match issue.severity {
                IssueSeverity::Critical => "🔴",
                IssueSeverity::Warning => "⚠️",
                IssueSeverity::Info => "ℹ️",
            };

            println!("{}. {} {}", idx + 1, severity_icon, issue.title);
            println!();
            println!("   {}", issue.explanation);
            println!();
            println!("   💡 Action: {}", issue.recommended_action);

            if let Some(impact) = &issue.estimated_impact {
                println!("   📊 Impact: {}", impact);
            }

            if let Some(reference) = &issue.reference {
                println!("   📚 Reference: {}", reference);
            }

            println!();
        }

        // Show next steps based on severity
        println!("💡 Next Steps:");
        let has_critical = caretaker_analysis.issues.iter()
            .any(|i| i.severity == IssueSeverity::Critical);

        if has_critical {
            println!("   🚨 Critical issues detected - run 'sudo annactl repair' now");
        } else {
            println!("   Run 'sudo annactl repair' to fix these issues automatically");
        }
        println!("   Or run 'annactl status' for detailed diagnostics");
    }

    println!();
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
