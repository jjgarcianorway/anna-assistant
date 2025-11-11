//! Health command implementations for annactl
//!
//! Phase 0.5b: CLI integration for health, doctor, and rescue list
//! Citation: [archwiki:System_maintenance]

use crate::errors::*;
use crate::logging::{ErrorDetails, LogEntry};
use crate::rpc_client::RpcClient;
use anna_common::ipc::{HealthRunData, ResponseData};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Instant;

/// Execute health command (Phase 0.5b)
pub async fn execute_health_command(
    json: bool,
    state: &str,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect()
        .await
        .context("Failed to connect to daemon")?;

    // Call health_run with all six probes
    let probes = vec![
        "disk-space".to_string(),
        "pacman-db".to_string(),
        "systemd-units".to_string(),
        "journal-errors".to_string(),
        "services-failed".to_string(),
        "firmware-microcode".to_string(),
    ];

    let response = client.health_run(15000, probes).await?;

    let data = match response {
        ResponseData::HealthRun(data) => data,
        _ => {
            log_and_exit(
                req_id,
                state,
                "health",
                start_time,
                EXIT_INVALID_RESPONSE,
                None,
            );
        }
    };

    // Determine exit code based on status
    let exit_code = determine_health_exit_code(&data);

    // Save report
    let report_path = save_health_report(&data).await?;

    if json {
        // JSON output only
        let json_output = serde_json::to_string_pretty(&data)?;
        println!("{}", json_output);
    } else {
        // Human output
        print_health_summary(&data);
        println!("Details saved: {}", report_path.display());
    }

    // Log to ctl.jsonl
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "health".to_string(),
        allowed: Some(true),
        args: if json {
            vec!["--json".to_string()]
        } else {
            vec![]
        },
        exit_code,
        citation: "[archwiki:General_recommendations]".to_string(),
        duration_ms,
        ok: exit_code == EXIT_SUCCESS || exit_code == 2,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Execute doctor command (Phase 0.5b)
pub async fn execute_doctor_command(
    json: bool,
    state: &str,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect()
        .await
        .context("Failed to connect to daemon")?;

    // Call health_run
    let probes = vec![
        "disk-space".to_string(),
        "pacman-db".to_string(),
        "systemd-units".to_string(),
        "journal-errors".to_string(),
        "services-failed".to_string(),
        "firmware-microcode".to_string(),
    ];

    let response = client.health_run(15000, probes).await?;

    let data = match response {
        ResponseData::HealthRun(data) => data,
        _ => {
            log_and_exit(
                req_id,
                state,
                "doctor",
                start_time,
                EXIT_INVALID_RESPONSE,
                None,
            );
        }
    };

    // Determine exit code
    let exit_code = determine_health_exit_code(&data);

    // Save doctor report
    let report_path = save_doctor_report(&data).await?;

    if json {
        // JSON output with doctor wrapper
        let doctor_output = serde_json::json!({
            "version": env!("ANNA_VERSION"),
            "ok": exit_code == EXIT_SUCCESS || exit_code == 2,
            "state": state,
            "summary": data.summary,
            "report": report_path.display().to_string(),
            "citation": "[archwiki:System_maintenance]",
            "probes": data.results,
        });
        println!("{}", serde_json::to_string_pretty(&doctor_output)?);
    } else {
        // Human output
        print_doctor_summary(&data);
        println!("Report saved: {}", report_path.display());
    }

    // Log to ctl.jsonl
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "doctor".to_string(),
        allowed: Some(true),
        args: if json {
            vec!["--json".to_string()]
        } else {
            vec![]
        },
        exit_code,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: exit_code == EXIT_SUCCESS || exit_code == 2,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Execute rescue list command (Phase 0.5b)
pub async fn execute_rescue_list_command(req_id: &str, start_time: Instant) -> Result<()> {
    let mut client = RpcClient::connect()
        .await
        .context("Failed to connect to daemon")?;

    let response = client.recovery_plans().await?;

    let data = match response {
        ResponseData::RecoveryPlans(data) => data,
        _ => {
            log_and_exit(
                req_id,
                "unknown",
                "rescue list",
                start_time,
                EXIT_INVALID_RESPONSE,
                None,
            );
        }
    };

    // Print three-column table
    for plan in &data.plans {
        println!("{:<16}  {:<50}  {}", plan.id, plan.desc, plan.citation);
    }

    // Log to ctl.jsonl
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: "unknown".to_string(), // rescue list doesn't need state
        command: "rescue list".to_string(),
        allowed: Some(true),
        args: vec!["list".to_string()],
        exit_code: EXIT_SUCCESS,
        citation: "[archwiki:General_troubleshooting]".to_string(),
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(EXIT_SUCCESS);
}

/// Determine exit code from health data
fn determine_health_exit_code(data: &HealthRunData) -> i32 {
    if data.summary.fail > 0 {
        1 // Any failures
    } else if data.summary.warn > 0 {
        2 // Any warnings but no failures
    } else {
        EXIT_SUCCESS // All ok
    }
}

/// Print health summary (human output)
fn print_health_summary(data: &HealthRunData) {
    println!(
        "Health summary: ok={} warn={} fail={}",
        data.summary.ok, data.summary.warn, data.summary.fail
    );

    for result in &data.results {
        if result.status != "ok" {
            println!("{}: {}  {}", result.status, result.probe, result.citation);
        }
    }
}

/// Print doctor summary (human output)
fn print_doctor_summary(data: &HealthRunData) {
    println!("Doctor report for state: {}", data.state);

    // Failed probes
    let failed: Vec<&str> = data
        .results
        .iter()
        .filter(|r| r.status == "fail")
        .map(|r| r.probe.as_str())
        .collect();

    if failed.is_empty() {
        println!("Failed probes: none");
    } else {
        println!("Failed probes: {}", failed.join(", "));
    }

    // Degraded units (from systemd-units probe)
    if let Some(systemd_result) = data.results.iter().find(|r| r.probe == "systemd-units") {
        if let Some(failed_count) = systemd_result.details.get("failed_count") {
            println!("Degraded units: {}", failed_count);
        }
    }

    // Top journal errors (from journal-errors probe)
    if let Some(_journal_result) = data.results.iter().find(|r| r.probe == "journal-errors") {
        println!("Top journal errors: (see details)");
    }

    // Citations
    print!("Citations: [archwiki:System_maintenance]");
    for result in &data.results {
        if result.status != "ok" {
            print!(" {}", result.citation);
        }
    }
    println!();
}

/// Save health report to /var/lib/anna/reports/
async fn save_health_report(data: &HealthRunData) -> Result<PathBuf> {
    let reports_dir = PathBuf::from("/var/lib/anna/reports");
    tokio::fs::create_dir_all(&reports_dir).await?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let filename = format!("health-{}.json", timestamp.replace(":", "-"));
    let report_path = reports_dir.join(filename);

    let content = serde_json::to_string_pretty(data)?;
    tokio::fs::write(&report_path, content).await?;

    // Set permissions to 0600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&report_path, perms)?;
    }

    Ok(report_path)
}

/// Save doctor report to /var/lib/anna/reports/
async fn save_doctor_report(data: &HealthRunData) -> Result<PathBuf> {
    let reports_dir = PathBuf::from("/var/lib/anna/reports");
    tokio::fs::create_dir_all(&reports_dir).await?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let filename = format!("doctor-{}.json", timestamp.replace(":", "-"));
    let report_path = reports_dir.join(filename);

    let doctor_data = serde_json::json!({
        "version": env!("ANNA_VERSION"),
        "state": data.state,
        "summary": data.summary,
        "probes": data.results,
        "citation": "[archwiki:System_maintenance]",
    });

    let content = serde_json::to_string_pretty(&doctor_data)?;
    tokio::fs::write(&report_path, content).await?;

    // Set permissions to 0600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&report_path, perms)?;
    }

    Ok(report_path)
}

/// Helper to log and exit
fn log_and_exit(
    req_id: &str,
    state: &str,
    command: &str,
    start_time: Instant,
    exit_code: i32,
    error: Option<ErrorDetails>,
) -> ! {
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: command.to_string(),
        allowed: Some(false),
        args: vec![],
        exit_code,
        citation: "[archwiki:General_recommendations]".to_string(),
        duration_ms,
        ok: false,
        error,
    };
    let _ = log_entry.write();
    std::process::exit(exit_code);
}
