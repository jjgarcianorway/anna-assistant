//! Service restart and recovery auto-healing operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use super::types::{HealingLog, HealingResult};

/// Clear systemd failed unit states (only for units that are not currently failed)
pub async fn clear_failed_unit_states(log: &mut HealingLog) -> Result<Option<String>> {
    // Get list of units in failed state
    let output = std::process::Command::new("systemctl")
        .args(["list-units", "--state=failed", "--no-legend", "--no-pager"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let failed_list = String::from_utf8_lossy(&output.stdout);
    let failed_count = failed_list.lines().filter(|l| !l.is_empty()).count();

    if failed_count == 0 {
        // Check if there are any units with failed state that need reset
        let reset_output = std::process::Command::new("systemctl")
            .args(["reset-failed"])
            .output();

        match reset_output {
            Ok(o) if o.status.success() => {
                let message = "Cleared old failed unit states".to_string();
                log.record(
                    "Stale failed states".to_string(),
                    "systemctl reset-failed".to_string(),
                    HealingResult::Success(message.clone()),
                );
                Ok(Some(message))
            }
            _ => Ok(None),
        }
    } else {
        // There are currently failed services - don't clear (they're legitimately failed)
        Ok(None)
    }
}

/// Restart services that have failed but might recover with a restart
/// Only restarts if the service hasn't been restarted recently (backoff)
pub async fn restart_failed_services(log: &mut HealingLog) -> Result<Option<String>> {
    const RESTART_STATE_FILE: &str = "/var/lib/anna/service_restart_state.json";

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct RestartState {
        last_restart_attempts: std::collections::HashMap<String, String>, // service -> timestamp
    }

    // Load restart state
    let mut state: RestartState = std::fs::read_to_string(RESTART_STATE_FILE)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    // Get failed services
    let output = std::process::Command::new("systemctl")
        .args(["list-units", "--state=failed", "--no-legend", "--no-pager"])
        .output()?;

    if !output.status.success() || output.stdout.is_empty() {
        return Ok(None);
    }

    let failed = String::from_utf8_lossy(&output.stdout);
    let mut restarted = Vec::new();

    for line in failed.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parse service name from line
        let service_name = line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        if service_name.is_empty() {
            continue;
        }

        // Check if we've tried to restart this service recently (within last 6 hours)
        if let Some(last_restart) = state.last_restart_attempts.get(&service_name) {
            if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(last_restart) {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(last_time.with_timezone(&chrono::Utc));
                if duration.num_hours() < 6 {
                    debug!("Skipping {} - restarted less than 6 hours ago", service_name);
                    continue;
                }
            }
        }

        // Attempt restart
        info!("Attempting to restart failed service: {}", service_name);
        let restart_result = std::process::Command::new("systemctl")
            .args(["restart", &service_name])
            .output();

        match restart_result {
            Ok(output) if output.status.success() => {
                // Verify service is now running
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let status_check = std::process::Command::new("systemctl")
                    .args(["is-active", &service_name])
                    .output();

                if let Ok(status) = status_check {
                    let is_active = String::from_utf8_lossy(&status.stdout).trim() == "active";

                    if is_active {
                        restarted.push(service_name.clone());
                        state.last_restart_attempts.insert(
                            service_name.clone(),
                            chrono::Utc::now().to_rfc3339(),
                        );
                        log.record(
                            format!("Failed service: {}", service_name),
                            format!("systemctl restart {}", service_name),
                            HealingResult::Success("Restarted successfully".to_string()),
                        );
                    } else {
                        log.record(
                            format!("Failed service: {}", service_name),
                            format!("systemctl restart {}", service_name),
                            HealingResult::Failed("Service still not active after restart".to_string()),
                        );
                    }
                }
            }
            _ => {
                log.record(
                    format!("Failed service: {}", service_name),
                    format!("systemctl restart {}", service_name),
                    HealingResult::Failed("Restart command failed".to_string()),
                );
            }
        }
    }

    // Save restart state
    if !restarted.is_empty() {
        if let Ok(content) = serde_json::to_string_pretty(&state) {
            let path = std::path::PathBuf::from(RESTART_STATE_FILE);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&path, content).ok();
        }

        let message = format!("Restarted {} failed service(s): {}", restarted.len(), restarted.join(", "));
        Ok(Some(message))
    } else {
        Ok(None)
    }
}
