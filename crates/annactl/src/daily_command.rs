//! Daily checkup command - REAL analysis with USEFUL output
//!
//! Phase 4.1: Actually helpful disk analysis
//! Citation: [archwiki:System_maintenance]

use crate::context_detection;
use crate::errors::*;
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;
use anna_common::disk_analysis::{DiskAnalysis, RecommendationRisk};
use anna_common::display::*;
use anna_common::ipc::{HealthRunData, ResponseData};
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use std::time::Instant;

/// Execute daily checkup command
pub async fn execute_daily_command(
    json: bool,
    state: &str,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect()
        .await.context("Failed to connect to daemon")?;

    let use_color = context_detection::should_use_color();

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

    // Determine overall status
    let has_failures = health_data.summary.fail > 0;
    let disk_critical = disk_analysis.usage_percent > 90.0;
    let disk_warning = disk_analysis.usage_percent > 80.0;

    let status_level = if has_failures || disk_critical {
        StatusLevel::Critical
    } else if health_data.summary.warn > 0 || disk_warning {
        StatusLevel::Warning
    } else {
        StatusLevel::Success
    };

    if json {
        // JSON output for automation
        let json_output = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "status": match status_level {
                StatusLevel::Critical => "critical",
                StatusLevel::Warning => "warning",
                StatusLevel::Success => "success",
                _ => "info",
            },
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
                "top_consumers": disk_analysis.top_consumers,
            },
            "probes": health_data.results,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Human output - beautiful and useful
        print_daily_output(&health_data, &disk_analysis, status_level, use_color);
    }

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let exit_code = if has_failures || disk_critical { 1 } else { EXIT_SUCCESS };
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

    std::process::exit(exit_code);
}

/// Print beautiful, useful daily output
fn print_daily_output(
    health_data: &HealthRunData,
    disk_analysis: &DiskAnalysis,
    status_level: StatusLevel,
    use_color: bool,
) {
    // Main status section
    let mut section = Section::new(
        format!("Daily System Check - {}",
            Utc::now().format("%Y-%m-%d %H:%M")),
        status_level,
        use_color,
    );

    // Overall health
    let health_summary = format!(
        "Health: {} ok, {} warnings, {} failures",
        health_data.summary.ok,
        health_data.summary.warn,
        health_data.summary.fail
    );
    section.add_line(health_summary);
    section.add_blank();

    // Disk status
    let disk_status = format!(
        "Disk: {:.1}% used ({} / {} total)",
        disk_analysis.usage_percent,
        format_bytes(disk_analysis.available_bytes),
        format_bytes(disk_analysis.total_bytes)
    );
    section.add_line(disk_status);

    print!("{}", section.render());

    // Show problems if any
    let has_problems = health_data.summary.fail > 0
        || health_data.summary.warn > 0
        || disk_analysis.usage_percent > 80.0;

    if has_problems {
        println!("\n📊 Issues Detected:\n");

        // Failed probes
        for result in &health_data.results {
            if result.status == "fail" {
                let msg = result.details.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Issue detected");
                println!("  ❌ {}: {}", result.probe, msg);
            }
        }

        // Warning probes
        for result in &health_data.results {
            if result.status == "warn" {
                let msg = result.details.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Warning");
                println!("  ⚠️  {}: {}", result.probe, msg);
            }
        }

        // Disk analysis if needed
        if disk_analysis.usage_percent > 80.0 {
            println!("\n📁 Disk Space Analysis:\n");
            println!("  Your disk is {:.1}% full. Here's what's using space:\n",
                disk_analysis.usage_percent);

            for consumer in &disk_analysis.top_consumers {
                println!("  {} {:<20} {:>10}  {}",
                    consumer.category.icon(),
                    consumer.category.name(),
                    consumer.size_human,
                    consumer.path.display()
                );
            }

            // Get and show recommendations
            let recommendations = disk_analysis.get_recommendations();
            if !recommendations.is_empty() {
                println!("\n🎯 Recommended Actions:\n");

                for (i, rec) in recommendations.iter().enumerate() {
                    println!("{}. {}", i + 1, rec.title);
                    if let Some(cmd) = &rec.command {
                        println!("   $ {}", cmd);
                    }
                    println!("   📖 {}", rec.explanation);
                    if let Some(warning) = &rec.warning {
                        println!("   ⚠️  {}", warning);
                    }
                    println!("   💾 Impact: Frees {}", rec.estimated_savings_human);

                    let wiki_url = if let Some(section) = &rec.wiki_section {
                        format!("{}#{}", rec.wiki_url, section)
                    } else {
                        rec.wiki_url.clone()
                    };
                    println!("   🔗 Arch Wiki: {}", wiki_url);

                    // Show risk level
                    let risk_label = match rec.risk_level {
                        RecommendationRisk::Safe => "✅ Safe",
                        RecommendationRisk::Low => "🟢 Low Risk",
                        RecommendationRisk::Medium => "🟡 Medium Risk",
                        RecommendationRisk::High => "🔴 High Risk",
                    };
                    println!("   Risk: {}", risk_label);
                    println!();
                }
            }
        }

        println!("\n💡 Next Steps:");
        if disk_analysis.usage_percent > 90.0 {
            println!("   Run the disk space cleanup commands above (start with the safest)");
        } else if health_data.summary.fail > 0 {
            println!("   Run 'annactl repair' to attempt automatic fixes");
        } else {
            println!("   Run 'annactl health' for detailed diagnostics");
        }
    } else {
        println!("\n✅ All systems healthy!");
        println!("\n💡 Your system is running smoothly. No action needed.");
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
