//! Status command - check daemon and system status

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, kv, section, Level};
use anyhow::Result;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::rpc_client::RpcClient;

/// Check for updates and show notification banner (non-spammy, once per day)
async fn check_and_notify_updates() {
    // Cache file to track last check
    let cache_file = PathBuf::from("/tmp/anna-update-check");

    // Check if we already checked today
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Ok(content) = std::fs::read_to_string(&cache_file) {
        if let Ok(last_check) = content.trim().parse::<u64>() {
            // If checked within last 24 hours, skip
            if now - last_check < 86400 {
                return;
            }
        }
    }

    // IMMEDIATELY update cache to prevent spam on failures
    let _ = std::fs::write(&cache_file, now.to_string());

    // Check for updates (silently fail if network issue)
    if let Ok(update_info) = anna_common::updater::check_for_updates().await {
        if update_info.is_update_available {
            // Show update banner
            println!();
            println!("\x1b[38;5;226m╭─────────────────────────────────────────────────────────────╮\x1b[0m");
            println!("\x1b[38;5;226m│\x1b[0m  \x1b[1m📦 Update Available\x1b[0m: {} → {}  \x1b[38;5;226m│\x1b[0m",
                update_info.current_version, update_info.latest_version);
            println!("\x1b[38;5;226m│\x1b[0m  Run \x1b[38;5;159mannactl update --install\x1b[0m to upgrade                 \x1b[38;5;226m│\x1b[0m");
            println!("\x1b[38;5;226m╰─────────────────────────────────────────────────────────────╯\x1b[0m");
        }
    }
    // If check fails, we already updated cache, so won't spam
}

/// Read recent daemon logs from journalctl (v6.40.0: filtered for relevance)
/// Only shows WARN and ERROR levels, filters out routine INFO noise
async fn read_recent_daemon_logs(count: usize) -> Result<Vec<String>> {
    use tokio::process::Command;

    // v6.40.0: Changed to --priority=warning to exclude routine INFO logs
    // This prevents spam like "Access granted: UID 1000" and "Connection from UID"
    let output = Command::new("journalctl")
        .args(&[
            "-u", "annad",
            "-n", &(count * 3).to_string(),  // Get more lines since we'll filter
            "--no-pager",
            "-o", "short-iso",
            "--priority=warning"  // warning and above (warning, err, crit, alert, emerg)
        ])
        .output()
        .await?;

    if !output.status.success() {
        // journalctl might fail if user doesn't have permission
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut logs = Vec::new();
    let mut filtered_count = 0;

    // v6.40.0: Also filter out specific noisy patterns even if they're at WARN level
    let noise_patterns = [
        "Access granted",
        "Connection from UID",
        "System knowledge snapshot complete",
        "RPC handshake",
        "Telemetry updated",
    ];

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Skip noise patterns
        let is_noise = noise_patterns.iter().any(|pattern| line.contains(pattern));
        if is_noise {
            filtered_count += 1;
            continue;
        }

        // Parse journalctl line: timestamp hostname service[pid]: message
        if let Some(message_start) = line.find("annad[") {
            if let Some(colon_pos) = line[message_start..].find(':') {
                let timestamp_end = message_start.min(24); // ISO timestamp is ~24 chars
                let timestamp = &line[..timestamp_end].trim();
                let message = &line[message_start + colon_pos + 1..].trim();

                // Color code based on log level
                let colored_message = if message.contains("ERROR") || message.contains("error") {
                    format!("\x1b[91m{}\x1b[0m", message) // Red
                } else if message.contains("WARN") || message.contains("warn") {
                    format!("\x1b[93m{}\x1b[0m", message) // Yellow
                } else {
                    message.to_string()
                };

                // Truncate long messages
                let display_msg = if colored_message.len() > 100 {
                    format!("{}...", &colored_message[..97])
                } else {
                    colored_message
                };

                logs.push(format!("\x1b[90m{}\x1b[0m {}", timestamp, display_msg));

                // Stop after collecting enough non-noise logs
                if logs.len() >= count {
                    break;
                }
            }
        }
    }

    // v6.40.0: If we filtered out noise, mention it
    if filtered_count > 0 && logs.is_empty() {
        // No warnings or errors - show friendly message
        logs.push(format!("\x1b[90m  No warnings or errors in recent logs\x1b[0m"));
        logs.push(format!("\x1b[90m  ({} routine events filtered - run 'journalctl -u annad' for full log)\x1b[0m", filtered_count));
    } else if filtered_count > 0 {
        // Had some warnings/errors, note how many routine events were filtered
        logs.push(String::new()); // blank line
        logs.push(format!("\x1b[90m  ({} routine events filtered)\x1b[0m", filtered_count));
    }

    Ok(logs)
}

/// Read recent audit entries from the audit log
async fn read_recent_audit_entries(count: usize) -> Result<Vec<anna_common::AuditEntry>> {
    use tokio::fs;

    let audit_path = std::path::Path::new("/var/log/anna/audit.jsonl");

    if !audit_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(audit_path).await?;

    let mut entries: Vec<anna_common::AuditEntry> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Sort by timestamp (newest first)
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Take the last N entries
    entries.truncate(count);

    Ok(entries)
}

pub async fn status() -> Result<()> {
    // RC.9: Show immediate feedback - don't make users wait in silence
    eprint!("\r{}", beautiful::status(Level::Info, "Checking daemon status..."));
    use std::io::Write;
    let _ = std::io::stderr().flush();

    // Try to connect to daemon
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            // Clear the "Checking..." message
            eprint!("\r\x1b[K");
            let _ = std::io::stderr().flush();

            println!("{}", header("Anna Status"));
            println!();
            println!(
                "{}",
                beautiful::status(Level::Error, "Daemon not running")
            );
            println!();
            println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    // RC.9.1: Status focuses ONLY on Anna's health, not recommendations
    // User feedback: "annactl status should show status of anna, not advises"
    // Recommendations moved to: annactl advise
    let status_data = client.call(Method::Status).await?;
    let facts_data = client.call(Method::GetFacts).await?;
    // Phase 0.2d: Get system state detection
    let state_data = client.call(Method::GetState).await?;

    // Clear the "Checking..." message and show header
    eprint!("\r\x1b[K");
    let _ = std::io::stderr().flush();

    println!("{}", header("Anna Status"));
    println!();

    if let (ResponseData::Status(status), ResponseData::Facts(facts))
        = (status_data, facts_data) {

        println!("{}", section("System"));
        println!("  {}", kv("Hostname", &facts.hostname));
        println!("  {}", kv("Kernel", &facts.kernel));

        // Phase 0.2d: Show detected system state
        if let ResponseData::StateDetection(state) = state_data {
            println!("  {}", kv("State", &state.state));
            if let Some(health) = state.details.health_ok {
                let health_str = if health { "OK" } else { "Degraded" };
                println!("  {}", kv("Health", health_str));
            }
        }

        println!();
        println!("{}", section("Anna Daemon"));
        println!("  {}", beautiful::status(Level::Success, "Running"));
        println!("  {}", kv("Version", &status.version));
        println!("  {}", kv("Uptime", &format!("{}s", status.uptime_seconds)));
        println!("  {}", kv("Socket", "/run/anna/anna.sock"));
        println!();

        // Show quick links to other commands
        println!("{}", section("Quick Commands"));
        println!("  {} \x1b[90m# Get recommendations\x1b[0m", "\x1b[96mannactl advise\x1b[0m");
        println!("  {} \x1b[90m# Full system report\x1b[0m", "\x1b[96mannactl report\x1b[0m");
        println!("  {} \x1b[90m# Check Anna health\x1b[0m", "\x1b[96mannactl doctor\x1b[0m");
        println!();
    }

    // Show recent activity from audit log
    if let Ok(entries) = read_recent_audit_entries(10).await {
        if !entries.is_empty() {
            println!("{}", section("Recent Activity"));
            for entry in entries.iter().take(10) {
                let time_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                let status_icon = if entry.success {
                    "\x1b[92m✓\x1b[0m"
                } else {
                    "\x1b[91m✗\x1b[0m"
                };

                // Color the action type based on what it is
                let action_color = match entry.action_type.as_str() {
                    "apply" => "\x1b[96m", // Cyan
                    "install" => "\x1b[92m", // Green
                    "remove" => "\x1b[93m", // Yellow
                    "update" => "\x1b[95m", // Magenta
                    _ => "\x1b[0m", // Default
                };

                println!("  {} \x1b[90m{}\x1b[0m {}{}\x1b[0m",
                    status_icon,
                    time_str,
                    action_color,
                    entry.action_type
                );

                // Show details on next line, indented
                let details = if entry.details.len() > 120 {
                    format!("{}...", &entry.details[..117])
                } else {
                    entry.details.clone()
                };
                println!("      \x1b[90m{}\x1b[0m", details);
            }
            println!();
        }
    }

    // Show recent daemon logs from journalctl (Beta.90)
    if let Ok(logs) = read_recent_daemon_logs(10).await {
        if !logs.is_empty() {
            println!("{}", section("Daemon Logs"));
            for log in logs {
                println!("  {}", log);
            }
            println!();
        }
    }

    // Check for updates (non-spammy, once per day)
    check_and_notify_updates().await;

    Ok(())
}
