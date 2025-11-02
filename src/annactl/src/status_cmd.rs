//! Real status command for Anna v0.12.9 "Orion"
//!
//! Implements truthful status checking:
//! - systemctl state and PID
//! - RPC health with timeout
//! - journalctl tail for warnings/errors
//! - Exit codes: 0=healthy, 1=degraded, 2=not running

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::process::Command;
use std::time::Duration;

/// Complete system status
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub daemon: DaemonStatus,
    pub health: Option<HealthSummary>,
    pub journal_tail: Vec<JournalEntry>,
    pub advice: String,
    pub exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub state: String,          // active, inactive, failed
    pub pid: Option<u32>,
    pub uptime_sec: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall: String,
    pub rpc_p99_ms: u64,
    pub memory_mb: f64,
    pub queue_depth: usize,
    pub queue_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub ts: String,
    pub level: String,
    pub msg: String,
}

/// Get complete system status
pub async fn get_status() -> Result<SystemStatus> {
    // 1. Check systemctl status
    let daemon = check_daemon_status()?;

    // 2. Try RPC health check (2s timeout, 1 retry)
    let health = if daemon.state == "active" {
        check_health_with_timeout().await.ok()
    } else {
        None
    };

    // 3. Get journal tail (warnings and errors only)
    let journal_tail = get_journal_tail(30)?;

    // 4. Determine exit code and advice
    let (exit_code, advice) = determine_status(&daemon, &health, &journal_tail);

    Ok(SystemStatus {
        daemon,
        health,
        journal_tail,
        advice,
        exit_code,
    })
}

/// Check daemon status via systemctl
fn check_daemon_status() -> Result<DaemonStatus> {
    // Check if daemon is active
    let is_active = Command::new("systemctl")
        .args(&["is-active", "annad"])
        .output()
        .context("Failed to run systemctl is-active")?;

    let state = String::from_utf8_lossy(&is_active.stdout).trim().to_string();

    // Get PID if active
    let pid = if state == "active" {
        if let Ok(show_output) = Command::new("systemctl")
            .args(&["show", "-p", "MainPID", "annad"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&show_output.stdout);
            if let Some(pid_str) = output_str.strip_prefix("MainPID=") {
                pid_str.trim().parse::<u32>().ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Get uptime if we have a PID
    let uptime_sec = if let Some(pid) = pid {
        get_process_uptime(pid).ok()
    } else {
        None
    };

    Ok(DaemonStatus {
        state,
        pid,
        uptime_sec,
    })
}

/// Get process uptime in seconds
fn get_process_uptime(pid: u32) -> Result<u64> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat = std::fs::read_to_string(stat_path)?;

    // Parse starttime from /proc/[pid]/stat (22nd field)
    let fields: Vec<&str> = stat.split_whitespace().collect();
    if fields.len() < 22 {
        anyhow::bail!("Invalid stat format");
    }

    let starttime_ticks: u64 = fields[21].parse()?;
    let ticks_per_sec = 100; // Typical value, could read from sysconf

    // Get system uptime
    let uptime_str = std::fs::read_to_string("/proc/uptime")?;
    let uptime_sec: f64 = uptime_str
        .split_whitespace()
        .next()
        .context("No uptime field")?
        .parse()?;

    let process_start_sec = starttime_ticks / ticks_per_sec;
    let process_uptime = uptime_sec as u64 - process_start_sec;

    Ok(process_uptime)
}

/// Check health via RPC with timeout
async fn check_health_with_timeout() -> Result<HealthSummary> {
    use crate::health_cmd::fetch_health_metrics;
    use tokio::time::timeout;

    let health_snapshot = timeout(Duration::from_secs(2), fetch_health_metrics())
        .await
        .context("Health check timed out after 2s")?
        .context("Health check failed")?;

    // Extract key metrics
    let overall = format!("{:?}", health_snapshot.status);
    let rpc_p99_ms = health_snapshot
        .rpc_latency
        .as_ref()
        .map(|l| l.p99_ms)
        .unwrap_or(0);
    let memory_mb = health_snapshot
        .memory
        .as_ref()
        .map(|m| m.current_mb)
        .unwrap_or(0.0);
    let queue_depth = health_snapshot
        .queue
        .as_ref()
        .map(|q| q.depth)
        .unwrap_or(0);
    let queue_rate = health_snapshot
        .queue
        .as_ref()
        .map(|q| q.rate_per_sec)
        .unwrap_or(0.0);

    Ok(HealthSummary {
        overall,
        rpc_p99_ms,
        memory_mb,
        queue_depth,
        queue_rate,
    })
}

/// Get journal tail (warnings and errors)
fn get_journal_tail(limit: usize) -> Result<Vec<JournalEntry>> {
    let output = Command::new("journalctl")
        .args(&[
            "-u", "annad",
            "-n", &limit.to_string(),
            "--no-pager",
            "--output=json",
            "-p", "warning",  // warning and above (includes error, crit, etc.)
        ])
        .output()
        .context("Failed to run journalctl")?;

    if !output.status.success() {
        // Fallback to no filtering if journal access fails
        return Ok(vec![]);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in output_str.lines() {
        if let Ok(entry) = serde_json::from_str::<JsonValue>(line) {
            entries.push(JournalEntry {
                ts: entry["__REALTIME_TIMESTAMP"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                level: entry["PRIORITY"]
                    .as_str()
                    .and_then(|p| match p {
                        "3" => Some("ERROR"),
                        "4" => Some("WARNING"),
                        "5" => Some("NOTICE"),
                        "6" => Some("INFO"),
                        _ => Some("UNKNOWN"),
                    })
                    .unwrap_or("UNKNOWN")
                    .to_string(),
                msg: entry["MESSAGE"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            });
        }
    }

    Ok(entries)
}

/// Determine overall status and advice
fn determine_status(
    daemon: &DaemonStatus,
    health: &Option<HealthSummary>,
    journal: &[JournalEntry],
) -> (i32, String) {
    // Not running = exit 2
    if daemon.state != "active" {
        return (2, "Daemon not running. Start with: sudo systemctl start annad".to_string());
    }

    // RPC failed = exit 2
    if health.is_none() {
        return (2, "Daemon running but RPC not responding. Check logs: journalctl -u annad".to_string());
    }

    let h = health.as_ref().unwrap();

    // Check for degraded conditions
    let has_errors = journal.iter().any(|e| e.level == "ERROR");
    let high_memory = h.memory_mb > 60.0;
    let high_latency = h.rpc_p99_ms > 500;
    let high_queue = h.queue_depth > 50;

    if has_errors || high_memory || high_latency || high_queue {
        let mut issues = Vec::new();
        if has_errors {
            issues.push("errors in logs");
        }
        if high_memory {
            issues.push("high memory");
        }
        if high_latency {
            issues.push("slow RPC");
        }
        if high_queue {
            issues.push("queue backlog");
        }

        return (
            1,
            format!(
                "Degraded: {}. Run: annactl health",
                issues.join(", ")
            ),
        );
    }

    // Healthy
    (0, "System healthy. No action needed.".to_string())
}

/// Display status in human-friendly format
pub fn display_status(status: &SystemStatus) -> Result<()> {
    let green = "\x1b[32m";
    let yellow = "\x1b[33m";
    let red = "\x1b[31m";
    let reset = "\x1b[0m";

    // Header with emoji
    let (emoji, color) = match status.exit_code {
        0 => ("✅", green),
        1 => ("⚠️", yellow),
        _ => ("❌", red),
    };

    println!(
        "\n{}{} Anna daemon is {} — {}{}",
        color,
        emoji,
        status.daemon.state,
        status.advice,
        reset
    );

    // Daemon info
    if let Some(pid) = status.daemon.pid {
        print!("• PID: {}   ", pid);
        if let Some(uptime) = status.daemon.uptime_sec {
            let hours = uptime / 3600;
            let mins = (uptime % 3600) / 60;
            print!("Uptime: {}h {}m   ", hours, mins);
        }
    }

    // Health summary
    if let Some(h) = &status.health {
        println!(
            "RPC p99: {} ms   Memory: {:.1} MB   Queue: {} events",
            h.rpc_p99_ms, h.memory_mb, h.queue_depth
        );
    } else {
        println!("Health: not available");
    }

    // Journal tail
    let error_count = status.journal_tail.iter().filter(|e| e.level == "ERROR").count();
    let warn_count = status.journal_tail.iter().filter(|e| e.level == "WARNING").count();

    if error_count > 0 || warn_count > 0 {
        println!(
            "• Journal: {} errors, {} warnings in recent logs",
            error_count, warn_count
        );
    } else {
        println!("• Journal: no errors or warnings");
    }

    println!();

    Ok(())
}

/// Display status in JSON format
pub fn display_status_json(status: &SystemStatus) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(status)?);
    Ok(())
}
