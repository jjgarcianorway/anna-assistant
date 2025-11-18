//! Status command - comprehensive health report
//!
//! Real Anna: `annactl status`
//! Purpose: Verify Anna herself is healthy and functioning
//! Checks:
//! - Anna's version and LLM mode
//! - Daemon status (annad)
//! - LLM backend health
//! - Permissions and groups
//! - Recent daemon logs
//! - Top suggestions for improvement
//! Behavior:
//! - Performs self-diagnostics
//! - Shows human-readable status
//! - Exits 0 if healthy, non-zero if unhealthy
//! Output:
//! - Comprehensive health report
//! - Journal excerpts
//! - Clear status: Healthy / Degraded / Broken

use anna_common::terminal_format as fmt;
use anyhow::Result;
use std::process::Command;
use std::time::Instant;

use crate::health::{HealthReport, HealthStatus};
use crate::logging::{ErrorDetails, LogEntry};
use crate::version_banner;

/// Execute 'annactl status' command - comprehensive health check
pub async fn execute_anna_status_command(
    _json: bool,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    // Display banner first
    println!("{}", fmt::bold("Anna Status Check"));
    println!("{}", "=".repeat(50));
    println!();

    // Get comprehensive health report
    let health = HealthReport::check(false).await?;

    // Display banner (version + LLM mode)
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "{} {}",
        fmt::bold("Anna Assistant"),
        fmt::bold(&format!("v{}", version))
    );

    let llm_mode = get_llm_mode_string().await;
    println!("{}", fmt::dimmed(&format!("Mode: {}", llm_mode)));
    println!();

    // Core Health
    println!("{}", fmt::bold("Core Health:"));

    // Daemon
    if health.daemon.installed && health.daemon.enabled && health.daemon.running {
        println!(
            "  {} Daemon: service installed, enabled, running",
            fmt::success("")
        );
    } else {
        let status = if !health.daemon.installed {
            "not installed"
        } else if !health.daemon.enabled {
            "not enabled"
        } else {
            "not running"
        };
        println!("  {} Daemon: {}", fmt::error(""), status);
    }

    // LLM Backend
    if health.llm.reachable && health.llm.model_available {
        println!(
            "  {} LLM backend: {} reachable, model {} available",
            fmt::success(""),
            health.llm.backend,
            health.llm.model.as_deref().unwrap_or("unknown")
        );
    } else if !health.llm.backend_running {
        println!(
            "  {} LLM backend: {} not running",
            fmt::error(""),
            health.llm.backend
        );
    } else if !health.llm.reachable {
        println!(
            "  {} LLM backend: {} not reachable",
            fmt::error(""),
            health.llm.backend
        );
    } else {
        println!(
            "  {} LLM backend: model {} not available",
            fmt::error(""),
            health.llm.model.as_deref().unwrap_or("unknown")
        );
    }

    // Permissions
    if health.permissions.data_dirs_ok && health.permissions.user_in_groups {
        println!("  {} Data directories: permissions OK", fmt::success(""));
        println!("  {} User groups: membership OK", fmt::success(""));
    } else {
        for issue in &health.permissions.issues {
            println!("  {} {}", fmt::warning(""), issue);
        }
    }

    println!();

    // Overall Status
    println!("{}", fmt::bold("Overall Status:"));
    health.display_summary();
    println!();

    // Last repair (if any)
    if let Some(repair) = &health.last_repair {
        println!("{}", fmt::bold("Last Self-Repair:"));
        println!(
            "  {} {}",
            fmt::dimmed("When:"),
            repair.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
        if repair.success {
            println!("  {} Successful", fmt::success(""));
        } else {
            println!("  {} Incomplete", fmt::warning(""));
        }
        for action in &repair.actions {
            println!("    • {}", action);
        }
        println!();
    }

    // Recent daemon log
    println!("{}", fmt::bold("Recent daemon log (annad):"));
    display_recent_logs();
    println!();

    // Top suggestions
    if let Ok(all_suggestions) = crate::suggestions::get_suggestions() {
        let critical: Vec<_> = all_suggestions.iter().filter(|s| s.priority <= 2).collect();

        if !critical.is_empty() {
            println!("{}", fmt::bold("Top Suggestions:"));
            for (i, suggestion) in critical.iter().take(3).enumerate() {
                println!("  {}. {}", i + 1, suggestion.title);
            }
            println!();
            println!(
                "  {} Ask Anna: \"what should I improve?\" for details",
                fmt::dimmed("→")
            );
            println!();
        }
    }

    // Log command and exit with appropriate code
    let exit_code = health.exit_code();
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "anna-status".to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: health.status == HealthStatus::Healthy,
        error: if health.status == HealthStatus::Healthy {
            None
        } else {
            Some(ErrorDetails {
                code: "UNHEALTHY".to_string(),
                message: format!("Anna is {:?}", health.status),
            })
        },
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Get LLM mode as a string
async fn get_llm_mode_string() -> String {
    use anna_common::context::db::{ContextDb, DbLocation};

    let db_location = DbLocation::auto_detect();
    match ContextDb::open(db_location).await {
        Ok(db) => match db.load_llm_config().await {
            Ok(config) => version_banner::format_llm_mode(&config),
            Err(_) => "LLM not configured".to_string(),
        },
        Err(_) => "LLM not configured".to_string(),
    }
}

/// Display recent journal logs
fn display_recent_logs() {
    let output = Command::new("journalctl")
        .args([
            "-u",
            "annad",
            "-n",
            "10",
            "--no-pager",
            "--output=short-iso",
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let logs = String::from_utf8_lossy(&output.stdout);
            if logs.trim().is_empty() {
                println!("  {}", fmt::dimmed("No recent logs"));
            } else {
                for line in logs.lines().take(10) {
                    if line.contains("error") || line.contains("ERROR") {
                        println!("  {}", fmt::error(line));
                    } else if line.contains("warn") || line.contains("WARN") {
                        println!("  {}", fmt::warning(line));
                    } else {
                        println!("  {}", fmt::dimmed(line));
                    }
                }
            }
        } else {
            println!(
                "  {}",
                fmt::dimmed("Unable to fetch logs (journalctl failed)")
            );
        }
    } else {
        println!(
            "  {}",
            fmt::dimmed("Unable to fetch logs (journalctl not available)")
        );
    }
}
