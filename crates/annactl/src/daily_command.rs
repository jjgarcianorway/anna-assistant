//! Daily checkup command - one-shot health and prediction summary
//!
//! Phase 4.0: Core Caretaker Workflows
//! Citation: [archwiki:System_maintenance]

use crate::context_detection;
use crate::errors::*;
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;
use anna_common::ipc::{HealthRunData, ResponseData};
use anyhow::{Context, Result};
use chrono::Utc;
use owo_colors::OwoColorize;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Execute daily checkup command (Phase 4.0)
pub async fn execute_daily_command(
    json: bool,
    state: &str,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect()
        .await
        .context("Failed to connect to daemon")?;

    // Run curated set of checks for daily use
    let probes = vec![
        "disk-space".to_string(),
        "pacman-db".to_string(),
        "systemd-units".to_string(),
        "journal-errors".to_string(),
        "services-failed".to_string(),
    ];

    let response = client.health_run(10000, probes).await?;

    let health_data = match response {
        ResponseData::HealthRun(data) => data,
        _ => {
            anyhow::bail!("Invalid response from daemon");
        }
    };

    // Check for pending reboots
    let needs_reboot = check_needs_reboot();

    // Get predictions (top 3)
    let predictions = get_top_predictions(&mut client, 3).await?;

    // Save report
    let report_path = save_daily_report(&health_data, &predictions, needs_reboot).await?;

    if json {
        // JSON output
        let json_output = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "health_summary": {
                "ok": health_data.summary.ok,
                "warn": health_data.summary.warn,
                "fail": health_data.summary.fail,
            },
            "probes": health_data.results,
            "needs_reboot": needs_reboot,
            "predictions": predictions,
            "report_path": report_path.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Human output (compact, max 24 lines)
        print_daily_summary(&health_data, &predictions, needs_reboot, &report_path);
    }

    // Log to ctl.jsonl
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let exit_code = if health_data.summary.fail > 0 {
        1
    } else {
        EXIT_SUCCESS
    };
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "daily".to_string(),
        allowed: Some(true),
        args: if json {
            vec!["--json".to_string()]
        } else {
            vec![]
        },
        exit_code,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: exit_code == EXIT_SUCCESS,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Print compact daily summary (max 24 lines)
fn print_daily_summary(
    health_data: &HealthRunData,
    predictions: &[PredictionInfo],
    needs_reboot: bool,
    report_path: &Path,
) {
    let use_color = context_detection::should_use_color();

    // Header (1 line)
    println!("{}", "═".repeat(60));
    println!(
        "{}",
        if use_color {
            "📋 DAILY CHECKUP".bold().to_string()
        } else {
            "DAILY CHECKUP".to_string()
        }
    );
    println!("{}", "═".repeat(60));

    // Health summary (1 line)
    let status_icon = if health_data.summary.fail > 0 {
        if use_color {
            "❌".red().to_string()
        } else {
            "FAIL".red().to_string()
        }
    } else if health_data.summary.warn > 0 {
        if use_color {
            "⚠️ ".yellow().to_string()
        } else {
            "WARN".yellow().to_string()
        }
    } else {
        if use_color {
            "✅".green().to_string()
        } else {
            "OK".green().to_string()
        }
    };
    println!(
        "Health: {} ({} ok, {} warn, {} fail)",
        status_icon, health_data.summary.ok, health_data.summary.warn, health_data.summary.fail
    );

    // Show only failed or warning probes (max 5 lines)
    let mut shown_issues = 0;
    for result in &health_data.results {
        if result.status != "ok" && shown_issues < 5 {
            let icon = if result.status == "fail" {
                if use_color {
                    "  ❌".red().to_string()
                } else {
                    "  FAIL".red().to_string()
                }
            } else {
                if use_color {
                    "  ⚠️ ".yellow().to_string()
                } else {
                    "  WARN".yellow().to_string()
                }
            };
            // Extract message from details if available
            let msg = result
                .details
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Issue detected");
            println!("{} {}: {}", icon, result.probe, msg);
            shown_issues += 1;
        }
    }

    // Reboot notice (1 line if needed)
    if needs_reboot {
        println!(
            "{}",
            if use_color {
                "🔄 System reboot recommended".yellow()
            } else {
                "System reboot recommended".yellow()
            }
        );
    }

    // Predictions (max 3 lines + header)
    if !predictions.is_empty() {
        println!();
        println!(
            "{}",
            if use_color {
                "🔮 PREDICTIONS".bold().to_string()
            } else {
                "PREDICTIONS".to_string()
            }
        );
        for pred in predictions.iter().take(3) {
            let icon = match pred.priority.as_str() {
                "Critical" | "High" => {
                    if use_color {
                        "🔴".red().to_string()
                    } else {
                        "HIGH".red().to_string()
                    }
                }
                "Medium" => {
                    if use_color {
                        "🟡".yellow().to_string()
                    } else {
                        "MED".yellow().to_string()
                    }
                }
                _ => {
                    if use_color {
                        "🟢".green().to_string()
                    } else {
                        "LOW".green().to_string()
                    }
                }
            };
            println!("  {} {}", icon, pred.title);
            if let Some(suggestion) = &pred.suggestion {
                println!("     → {}", suggestion);
            }
        }
    }

    // Footer (3 lines)
    println!();
    println!("{}", "─".repeat(60));
    println!("Report: {}", report_path.display());
    println!(
        "Next: {}",
        if health_data.summary.fail > 0 || health_data.summary.warn > 0 {
            "annactl repair (to fix issues)"
        } else {
            "annactl status (for detailed view)"
        }
    );
}

/// Check if system needs reboot
fn check_needs_reboot() -> bool {
    // Common indicators that a reboot is needed
    std::path::Path::new("/var/run/reboot-required").exists()
        || std::path::Path::new("/usr/lib/modules").read_dir().map_or(false, |mut entries| {
            // Check if there are kernel modules for versions other than current
            let current_kernel = std::fs::read_to_string("/proc/version")
                .unwrap_or_default()
                .split_whitespace()
                .nth(2)
                .unwrap_or("")
                .to_string();
            entries.any(|e| {
                e.ok()
                    .and_then(|e| e.file_name().into_string().ok())
                    .map_or(false, |name| !current_kernel.starts_with(&name))
            })
        })
}

/// Get top N predictions from daemon
async fn get_top_predictions(
    client: &mut RpcClient,
    limit: usize,
) -> Result<Vec<PredictionInfo>> {
    // For Phase 4.0, we'll use a placeholder until we wire up the prediction RPC
    // In a real implementation, this would call client.get_predictions()
    // For now, return empty to avoid blocking on infrastructure
    Ok(Vec::new())
}

/// Simplified prediction info for daily display
#[derive(Debug, Clone, serde::Serialize)]
struct PredictionInfo {
    priority: String,
    title: String,
    suggestion: Option<String>,
}

/// Save daily report to disk
async fn save_daily_report(
    health_data: &HealthRunData,
    predictions: &[PredictionInfo],
    needs_reboot: bool,
) -> Result<PathBuf> {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let filename = format!("daily-{}.json", timestamp);

    // Try primary path first, fall back to user home
    let report_dir = pick_report_dir();
    let report_path = report_dir.join(&filename);

    let report = json!({
        "type": "daily",
        "timestamp": Utc::now().to_rfc3339(),
        "health_summary": {
            "ok": health_data.summary.ok,
            "warn": health_data.summary.warn,
            "fail": health_data.summary.fail,
        },
        "probes": health_data.results,
        "needs_reboot": needs_reboot,
        "predictions": predictions,
        "citation": "[archwiki:System_maintenance]",
    });

    tokio::fs::write(&report_path, serde_json::to_string_pretty(&report)?)
        .await
        .context("Failed to write daily report")?;

    Ok(report_path)
}

/// Pick report directory (primary or fallback)
fn pick_report_dir() -> PathBuf {
    let primary = PathBuf::from("/var/lib/anna/reports");
    if primary.exists() && is_writable(&primary) {
        primary
    } else {
        // Fallback to user home
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".local/state/anna/reports")
    }
}

/// Check if directory is writable
fn is_writable(path: &Path) -> bool {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path.join(".test"))
        .and_then(|_| std::fs::remove_file(path.join(".test")))
        .is_ok()
}
