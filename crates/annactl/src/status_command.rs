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

    // Beta.141: Enhanced core health display with emojis
    println!("{}", fmt::section_title(&fmt::emojis::SERVICE, "Core Health"));
    println!();

    // Daemon
    if health.daemon.installed && health.daemon.enabled && health.daemon.running {
        println!(
            "  {}",
            fmt::component_status("Daemon (annad)", "running")
        );
        println!(
            "    {}",
            fmt::dimmed("service installed, enabled, and active")
        );
    } else {
        let status = if !health.daemon.installed {
            "not installed"
        } else if !health.daemon.enabled {
            "not enabled"
        } else {
            "not running"
        };
        println!("  {}", fmt::component_status("Daemon (annad)", status));
    }

    // LLM Backend
    if health.llm.reachable && health.llm.model_available {
        println!(
            "  {}",
            fmt::component_status(
                &format!("LLM ({})", health.llm.backend),
                "running"
            )
        );
        println!(
            "    {}",
            fmt::dimmed(&format!(
                "model: {}",
                health.llm.model.as_deref().unwrap_or("unknown")
            ))
        );
    } else if !health.llm.backend_running {
        println!(
            "  {}",
            fmt::component_status(&format!("LLM ({})", health.llm.backend), "stopped")
        );
    } else if !health.llm.reachable {
        println!(
            "  {}",
            fmt::component_status(&format!("LLM ({})", health.llm.backend), "degraded")
        );
        println!("    {}", fmt::dimmed("backend not reachable"));
    } else {
        println!(
            "  {}",
            fmt::component_status(&format!("LLM ({})", health.llm.backend), "degraded")
        );
        println!(
            "    {}",
            fmt::dimmed(&format!(
                "model {} not available",
                health.llm.model.as_deref().unwrap_or("unknown")
            ))
        );
    }

    // Beta.141: Enhanced permissions display
    if health.permissions.data_dirs_ok && health.permissions.user_in_groups {
        println!(
            "  {}",
            fmt::component_status("Permissions", "healthy")
        );
        println!("    {}", fmt::dimmed("data directories and user groups OK"));
    } else {
        println!(
            "  {}",
            fmt::component_status("Permissions", "degraded")
        );
        for issue in &health.permissions.issues {
            println!("    {} {}", fmt::emojis::WARNING, fmt::dimmed(issue));
        }
    }

    println!();

    // Overall Status
    println!("{}", fmt::bold("Overall Status:"));
    health.display_summary();
    println!();

    // Beta.141: Enhanced repair display
    if let Some(repair) = &health.last_repair {
        println!(
            "{}",
            fmt::section_title(&fmt::emojis::RESTORE, "Last Self-Repair")
        );
        println!();
        println!(
            "  {} {}",
            fmt::emojis::TIME,
            repair.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
        if repair.success {
            println!("  {} {}", fmt::emojis::SUCCESS, fmt::bold("Successful"));
        } else {
            println!("  {} {}", fmt::emojis::WARNING, fmt::bold("Incomplete"));
        }
        println!();
        println!("  {}:", fmt::bold("Actions Taken"));
        for action in &repair.actions {
            println!("    {} {}", fmt::symbols::ARROW, fmt::dimmed(action));
        }
        println!();
    }

    // Beta.141: Enhanced daemon log display
    println!(
        "{}",
        fmt::section_title(&fmt::emojis::DAEMON, "Recent Daemon Log")
    );
    println!();
    display_recent_logs();
    println!();

    // Beta.141: Enhanced suggestions display
    if let Ok(all_suggestions) = crate::suggestions::get_suggestions() {
        let critical: Vec<_> = all_suggestions.iter().filter(|s| s.priority <= 2).collect();

        if !critical.is_empty() {
            println!(
                "{}",
                fmt::section_title(&fmt::emojis::TIP, "Top Suggestions")
            );
            println!();
            for (i, suggestion) in critical.iter().take(3).enumerate() {
                println!(
                    "  {}. {} {}",
                    i + 1,
                    fmt::emojis::ROCKET,
                    suggestion.title
                );
            }
            println!();
            println!(
                "  {} {}",
                fmt::emojis::INFO,
                fmt::dimmed("Ask Anna: \"what should I improve?\" for details")
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
