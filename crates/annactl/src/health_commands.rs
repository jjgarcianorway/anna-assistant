//! Health command implementations for annactl
//!
//! Phase 0.5b: CLI integration for health, doctor, and rescue list
//! Citation: [archwiki:System_maintenance]

use crate::context_detection;
use crate::errors::*;
use crate::logging::{ErrorDetails, LogEntry};
use crate::predictive_hints;
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

    // Phase 3.8: Display predictive hints
    let use_color = context_detection::should_use_color();
    let _ = predictive_hints::display_predictive_hints("health", json, use_color).await;

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

    // Phase 3.9: Check installation source
    let install_check = check_installation_source();

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
            "installation_source": install_check,
        });
        println!("{}", serde_json::to_string_pretty(&doctor_output)?);
    } else {
        // Human output
        print_doctor_summary(&data);
        print_installation_check(&install_check);
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

/// Execute repair command (Phase 0.7)
pub async fn execute_repair_command(
    probe: &str,
    dry_run: bool,
    state: &str,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect()
        .await
        .context("Failed to connect to daemon")?;

    // Call repair_probe
    let response = client.repair_probe(probe.to_string(), dry_run).await?;

    let data = match response {
        ResponseData::RepairResult(data) => data,
        _ => {
            log_and_exit(
                req_id,
                state,
                "repair",
                start_time,
                EXIT_INVALID_RESPONSE,
                None,
            );
        }
    };

    // Determine exit code
    let exit_code = if data.success {
        EXIT_SUCCESS
    } else {
        1 // Repair failed
    };

    // Print results (plain text, no markdown)
    if dry_run {
        println!("[anna] repair simulation: probe={}", probe);
    } else {
        println!("[anna] repair: probe={}", probe);
    }

    for repair in &data.repairs {
        let status = if repair.success { "OK" } else { "FAIL" };
        println!(
            "[anna] probe: {} — {} ({})",
            repair.probe, repair.action, status
        );
        if !repair.details.is_empty() {
            println!("  {}", repair.details);
        }
        if !dry_run {
            println!("  Citation: {}", repair.citation);
        }
    }

    println!("{}", data.message);
    if !dry_run {
        println!("Citation: {}", data.citation);
    }

    // Log to ctl.jsonl
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "repair".to_string(),
        allowed: Some(true),
        args: if dry_run {
            vec!["--dry-run".to_string(), probe.to_string()]
        } else {
            vec![probe.to_string()]
        },
        exit_code,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: data.success,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Check installation source (Phase 3.9: AUR awareness)
#[derive(serde::Serialize)]
struct InstallationCheck {
    source: String,
    status: String,
    recommendation: Option<String>,
}

fn check_installation_source() -> InstallationCheck {
    use std::process::Command;

    // Check if annactl is managed by pacman
    let pacman_check = Command::new("pacman")
        .args(&["-Qo", "/usr/bin/annactl"])
        .output();

    if let Ok(output) = pacman_check {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Output format: "/usr/bin/annactl is owned by package-name version"
                if let Some(package) = stdout.split_whitespace().nth(4) {
                    return InstallationCheck {
                        source: format!("Package Manager ({})", package),
                        status: "ok".to_string(),
                        recommendation: None,
                    };
                }
            }
        }
    }

    // Check if in /usr/local (manual install)
    if let Ok(exe_path) = std::env::current_exe() {
        if exe_path.starts_with("/usr/local") {
            return InstallationCheck {
                source: "Manual Installation (/usr/local)".to_string(),
                status: "ok".to_string(),
                recommendation: Some("Consider using AUR for easier updates: yay -S anna-assistant-bin".to_string()),
            };
        }
    }

    // Unknown/mixed installation
    InstallationCheck {
        source: "Unknown".to_string(),
        status: "warn".to_string(),
        recommendation: Some("Unable to determine installation method. Reinstall via AUR or GitHub releases.".to_string()),
    }
}

fn print_installation_check(check: &InstallationCheck) {
    println!();
    println!("Installation Source Check:");

    let status_icon = match check.status.as_str() {
        "ok" => "✓",
        "warn" => "⚠️",
        _ => "•",
    };

    println!("  {} Source: {}", status_icon, check.source);

    if let Some(rec) = &check.recommendation {
        println!("  💡 {}", rec);
    }
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
