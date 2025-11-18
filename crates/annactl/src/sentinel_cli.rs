//! Sentinel CLI commands
//!
//! Phase 1.0: Autonomous daemon management commands
//! Citation: [archwiki:System_maintenance]

use anna_common::ipc::ResponseData;
use anyhow::{Context, Result};
use std::time::Instant;

use crate::errors::*;
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;

/// Execute 'sentinel status' command
pub async fn execute_sentinel_status_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    let response = client
        .sentinel_status()
        .await
        .context("Failed to get sentinel status")?;

    match response {
        ResponseData::SentinelStatus(status) => {
            println!("┌─────────────────────────────────────────────────────────");
            println!("│ SENTINEL STATUS");
            println!("├─────────────────────────────────────────────────────────");
            println!(
                "│ Enabled:        {}",
                if status.enabled { "✓ Yes" } else { "✗ No" }
            );
            println!(
                "│ Autonomous:     {}",
                if status.autonomous_mode {
                    "✓ Active"
                } else {
                    "✗ Inactive"
                }
            );
            println!("│ Uptime:         {} seconds", status.uptime_seconds);
            println!("│ System State:   {}", status.system_state);
            println!("├─────────────────────────────────────────────────────────");
            println!("│ HEALTH");
            println!("│ Status:         {}", status.last_health_status);
            if let Some(last_check) = &status.last_health_check {
                println!("│ Last Check:     {}", last_check);
            }
            println!("├─────────────────────────────────────────────────────────");
            println!("│ MONITORING");
            if let Some(last_update) = &status.last_update_scan {
                println!("│ Last Update Scan: {}", last_update);
            }
            if let Some(last_audit) = &status.last_audit {
                println!("│ Last Audit:     {}", last_audit);
            }
            println!("├─────────────────────────────────────────────────────────");
            println!("│ METRICS");
            println!("│ Error Rate:     {:.2} errors/hour", status.error_rate);
            println!("│ Drift Index:    {:.2}", status.drift_index);
            println!("└─────────────────────────────────────────────────────────");

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                command: "sentinel status".to_string(),
                state: state.to_string(),
                allowed: Some(true),
                args: vec![],
                exit_code: EXIT_SUCCESS,
                duration_ms,
                ok: true,
                error: None,
                citation: "[archwiki:System_maintenance]".to_string(),
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_SUCCESS);
        }
        _ => anyhow::bail!("Unexpected response type"),
    }
}

/// Execute 'sentinel metrics' command
pub async fn execute_sentinel_metrics_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    let response = client
        .sentinel_metrics()
        .await
        .context("Failed to get sentinel metrics")?;

    match response {
        ResponseData::SentinelMetrics(metrics) => {
            println!("┌─────────────────────────────────────────────────────────");
            println!("│ SENTINEL METRICS");
            println!("├─────────────────────────────────────────────────────────");
            println!("│ Uptime:           {} seconds", metrics.uptime_seconds);
            println!("│ Total Events:     {}", metrics.total_events);
            println!("├─────────────────────────────────────────────────────────");
            println!("│ OPERATIONS");
            println!("│ Automated Actions: {}", metrics.automated_actions);
            println!("│ Manual Commands:   {}", metrics.manual_commands);
            println!("│ Health Checks:     {}", metrics.health_checks);
            println!("│ Update Scans:      {}", metrics.update_scans);
            println!("│ Audits:            {}", metrics.audits);
            println!("├─────────────────────────────────────────────────────────");
            println!("│ SYSTEM");
            println!("│ Current Health:  {}", metrics.current_health);
            println!("│ Error Rate:      {:.2} errors/hour", metrics.error_rate);
            println!("│ Drift Index:     {:.2}", metrics.drift_index);
            println!("└─────────────────────────────────────────────────────────");

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                command: "sentinel metrics".to_string(),
                state: state.to_string(),
                allowed: Some(true),
                args: vec![],
                exit_code: EXIT_SUCCESS,
                duration_ms,
                ok: true,
                error: None,
                citation: "[archwiki:System_maintenance]".to_string(),
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_SUCCESS);
        }
        _ => anyhow::bail!("Unexpected response type"),
    }
}

/// Execute 'config get' command
pub async fn execute_config_get_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    let response = client
        .sentinel_get_config()
        .await
        .context("Failed to get sentinel configuration")?;

    match response {
        ResponseData::SentinelConfig(config) => {
            println!("┌─────────────────────────────────────────────────────────");
            println!("│ SENTINEL CONFIGURATION");
            println!("├─────────────────────────────────────────────────────────");
            println!("│ autonomous_mode:         {}", config.autonomous_mode);
            println!(
                "│ health_check_interval:   {} seconds",
                config.health_check_interval
            );
            println!(
                "│ update_scan_interval:    {} seconds",
                config.update_scan_interval
            );
            println!(
                "│ audit_interval:          {} seconds",
                config.audit_interval
            );
            println!("│ auto_repair_services:    {}", config.auto_repair_services);
            println!("│ auto_update:             {}", config.auto_update);
            println!(
                "│ auto_update_threshold:   {} packages",
                config.auto_update_threshold
            );
            println!("│ adaptive_scheduling:     {}", config.adaptive_scheduling);
            println!("└─────────────────────────────────────────────────────────");

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                command: "config get".to_string(),
                state: state.to_string(),
                allowed: Some(true),
                args: vec![],
                exit_code: EXIT_SUCCESS,
                duration_ms,
                ok: true,
                error: None,
                citation: "[archwiki:System_maintenance]".to_string(),
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_SUCCESS);
        }
        _ => anyhow::bail!("Unexpected response type"),
    }
}

/// Execute 'config set' command
pub async fn execute_config_set_command(
    key: &str,
    value: &str,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    // First get current config
    let current = client.sentinel_get_config().await?;
    let mut config = match current {
        ResponseData::SentinelConfig(c) => c,
        _ => anyhow::bail!("Failed to get current configuration"),
    };

    // Update the specified key
    match key {
        "autonomous_mode" => config.autonomous_mode = value.parse()?,
        "health_check_interval" => config.health_check_interval = value.parse()?,
        "update_scan_interval" => config.update_scan_interval = value.parse()?,
        "audit_interval" => config.audit_interval = value.parse()?,
        "auto_repair_services" => config.auto_repair_services = value.parse()?,
        "auto_update" => config.auto_update = value.parse()?,
        "auto_update_threshold" => config.auto_update_threshold = value.parse()?,
        "adaptive_scheduling" => config.adaptive_scheduling = value.parse()?,
        _ => anyhow::bail!("Unknown configuration key: {}", key),
    }

    // Set new config
    let response = client.sentinel_set_config(config.clone()).await?;

    match response {
        ResponseData::SentinelConfig(_) => {
            println!("[anna] Configuration updated: {} = {}", key, value);
            println!("[anna] Citation: [archwiki:System_maintenance]");

            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                command: "config set".to_string(),
                state: state.to_string(),
                allowed: Some(true),
                args: vec![key.to_string(), value.to_string()],
                exit_code: EXIT_SUCCESS,
                duration_ms,
                ok: true,
                error: None,
                citation: "[archwiki:System_maintenance]".to_string(),
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_SUCCESS);
        }
        _ => anyhow::bail!("Unexpected response type"),
    }
}
