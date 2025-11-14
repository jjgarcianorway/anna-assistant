//! Health command implementations for annactl
//!
//! Phase 0.5b: CLI integration for health, doctor, and rescue list
//! Citation: [archwiki:System_maintenance]

use crate::context_detection;
use crate::errors::*;
use crate::logging::{ErrorDetails, LogEntry};
use crate::predictive_hints;
use crate::rpc_client::RpcClient;
use anna_common::display::UI;
use anna_common::ipc::{HealthRunData, ResponseData};
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
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

/// Phase 3.9.1: Pick report directory with graceful fallback
///
/// Priority order:
/// 1. /var/lib/anna/reports (if writable)
/// 2. $XDG_STATE_HOME/anna/reports
/// 3. ~/.local/state/anna/reports
/// 4. /tmp (last resort)
fn pick_report_dir() -> PathBuf {
    use std::path::Path;

    // Try primary path (0770 root:anna)
    let primary = Path::new("/var/lib/anna/reports");
    if is_writable(primary) {
        return primary.to_path_buf();
    }

    // Try XDG_STATE_HOME
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        let path = Path::new(&xdg).join("anna/reports");
        if ensure_writable(&path).is_ok() {
            return path;
        }
    }

    // Try ~/.local/state
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".local/state/anna/reports");
        if ensure_writable(&path).is_ok() {
            return path;
        }
    }

    // Last resort: /tmp
    PathBuf::from("/tmp")
}

/// Check if directory is writable
fn is_writable(path: &Path) -> bool {
    use std::fs;

    // Check if directory exists and is writable
    if !path.exists() {
        return false;
    }

    // Try creating a test file
    let test_file = path.join(".write_test");
    match fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            true
        }
        Err(_) => false,
    }
}

/// Ensure directory exists and is writable
fn ensure_writable(path: &Path) -> Result<()> {
    use std::fs;

    // Create directory if it doesn't exist
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    // Test write access
    let test_file = path.join(".write_test");
    fs::write(&test_file, b"test")?;
    fs::remove_file(&test_file)?;

    Ok(())
}

/// Save health report to /var/lib/anna/reports/ with fallback
async fn save_health_report(data: &HealthRunData) -> Result<PathBuf> {
    let reports_dir = pick_report_dir();
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

/// Save doctor report to /var/lib/anna/reports/ with fallback
async fn save_doctor_report(data: &HealthRunData) -> Result<PathBuf> {
    let reports_dir = pick_report_dir();
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

    let use_color = context_detection::should_use_color();

    // Phase 4.1: Check disk space first - it's a common issue
    let disk_analysis = anna_common::disk_analysis::DiskAnalysis::analyze_root()?;

    if disk_analysis.usage_percent > 80.0 && probe == "all" {
        // Show disk space issue and recommendations FIRST
        println!("{}", "═".repeat(60));
        println!("{}", if use_color {
            "💾 DISK SPACE ISSUE DETECTED".bold().yellow().to_string()
        } else {
            "DISK SPACE ISSUE DETECTED".to_string()
        });
        println!("{}", "═".repeat(60));
        println!();
        println!("Your disk is {:.1}% full ({} available).",
            disk_analysis.usage_percent,
            format_bytes(disk_analysis.available_bytes));
        println!();
        println!("Top space consumers:");
        for consumer in disk_analysis.top_consumers.iter().take(5) {
            println!("  {} {:<20} {:>10}  {}",
                consumer.category.icon(),
                consumer.category.name(),
                consumer.size_human,
                consumer.path.display()
            );
        }
        println!();

        let recommendations = disk_analysis.get_recommendations();
        if !recommendations.is_empty() {
            println!("🎯 Recommended Actions:");
            println!();
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

                let risk_label = match rec.risk_level {
                    anna_common::disk_analysis::RecommendationRisk::Safe => "✅ Safe",
                    anna_common::disk_analysis::RecommendationRisk::Low => "🟢 Low Risk",
                    anna_common::disk_analysis::RecommendationRisk::Medium => "🟡 Medium Risk",
                    anna_common::disk_analysis::RecommendationRisk::High => "🔴 High Risk",
                };
                println!("   Risk: {}", risk_label);
                println!();
            }

            println!("💡 Run these commands manually to free up disk space.");
            println!("   Anna cannot execute them automatically for safety.");
            println!();
        }

        println!("{}", "═".repeat(60));
        println!();
    }

    // Phase 4.0: Enhanced repair with user confirmation and risk awareness
    if !dry_run && probe == "all" {
        // Interactive mode: show what will be repaired and ask for confirmation
        println!("{}", "═".repeat(60));
        println!(
            "{}",
            if use_color {
                "🔧 SYSTEM REPAIR".bold().to_string()
            } else {
                "SYSTEM REPAIR".to_string()
            }
        );
        println!("{}", "═".repeat(60));
        println!();
        println!("Anna will attempt to fix detected issues automatically.");
        println!("Only low-risk actions will be performed.");
        println!();
        println!(
            "{}",
            if use_color {
                "⚠️  Actions may modify system state!".yellow().to_string()
            } else {
                "WARNING: Actions may modify system state!".to_string()
            }
        );
        println!();
        print!("Proceed with repair? [y/N]: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Repair cancelled.");
            std::process::exit(EXIT_SUCCESS);
        }
        println!();
    }

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

    // Phase 4.0: Enhanced output with better formatting
    if dry_run {
        println!(
            "{}",
            if use_color {
                "🔍 REPAIR SIMULATION (dry run)".bold().to_string()
            } else {
                "REPAIR SIMULATION (dry run)".to_string()
            }
        );
    } else {
        println!(
            "{}",
            if use_color {
                "🔧 EXECUTING REPAIRS".bold().to_string()
            } else {
                "EXECUTING REPAIRS".to_string()
            }
        );
    }
    println!();

    let mut total_success = 0;
    let mut total_failed = 0;

    for repair in &data.repairs {
        let icon = if repair.success {
            total_success += 1;
            if use_color {
                "✅".green().to_string()
            } else {
                "OK".green().to_string()
            }
        } else {
            total_failed += 1;
            if use_color {
                "❌".red().to_string()
            } else {
                "FAIL".red().to_string()
            }
        };

        println!("{} {}", icon, repair.probe);
        println!("  Action: {}", repair.action);

        if !repair.details.is_empty() {
            println!("  Details: {}", repair.details);
        }

        if !dry_run && !repair.citation.is_empty() {
            println!("  Source: {}", repair.citation);
        }
        println!();
    }

    // Summary
    println!("{}", "─".repeat(60));
    println!(
        "Summary: {} succeeded, {} failed",
        total_success, total_failed
    );

    if !dry_run && !data.citation.is_empty() {
        println!("Reference: {}", data.citation);
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

/// Execute self-health repair command (Task 6: Anna's own health check)
pub async fn execute_self_health_repair(
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let ui = UI::auto();

    println!();
    ui.section_header("🔧", "Anna Self-Health Check");
    ui.info("Checking Anna's own operational health...");
    println!();

    // Track issues found
    let mut issues: Vec<SelfHealthIssue> = Vec::new();

    // Check 1: Is annad daemon healthy?
    ui.info("Checking daemon status...");
    let daemon_status = crate::systemd::get_service_status();
    let daemon_running = daemon_status.as_ref().map(|s| s.active).unwrap_or(false);

    if let Ok(status) = &daemon_status {
        if status.needs_repair() {
            issues.push(SelfHealthIssue {
                severity: IssueSeverity::High,
                description: format!("Anna daemon: {}", status.summary()),
                fix_description: "Ask me to fix myself in REPL".to_string(),
                fix_command: Some("annactl (then: 'fix yourself')".to_string()),
                auto_fixable: true,
            });
        }
    } else if let Err(e) = &daemon_status {
        issues.push(SelfHealthIssue {
            severity: IssueSeverity::High,
            description: format!("Failed to check daemon status: {}", e),
            fix_description: "Check if systemd is available".to_string(),
            fix_command: None,
            auto_fixable: false,
        });
    }

    // Check 2: Can we connect to daemon socket?
    ui.info("Checking daemon connectivity...");
    let socket_ok = check_socket_connectivity().await;
    if !socket_ok && daemon_running {
        let username = std::env::var("USER").unwrap_or_else(|_| "your-username".to_string());
        issues.push(SelfHealthIssue {
            severity: IssueSeverity::High,
            description: "Cannot connect to daemon socket".to_string(),
            fix_description: "Check socket permissions and group membership".to_string(),
            fix_command: Some(format!("sudo usermod -a -G anna {}", username)),
            auto_fixable: true,
        });
    }

    // Check 3: Context database accessibility
    ui.info("Checking context database...");
    let db_issues = check_context_database();
    issues.extend(db_issues);

    // Check 4: Key directories
    ui.info("Checking key directories...");
    let dir_issues = check_key_directories();
    issues.extend(dir_issues);

    println!();

    // Present findings
    if issues.is_empty() {
        ui.success("All checks passed! Anna is healthy.");
        println!();

        // Log success and exit
        log_self_health_result(req_id, start_time, EXIT_SUCCESS, None);
        std::process::exit(EXIT_SUCCESS);
    }

    // Show issues
    ui.section_header("⚠️", "Issues Found");
    println!();

    for (i, issue) in issues.iter().enumerate() {
        let severity_icon = match issue.severity {
            IssueSeverity::High => "🔴",
            IssueSeverity::Medium => "🟡",
            IssueSeverity::Low => "🟢",
        };

        ui.info(&format!("{}. {} {}", i + 1, severity_icon, issue.description));
        ui.info(&format!("   Fix: {}", issue.fix_description));

        if let Some(cmd) = &issue.fix_command {
            ui.info(&format!("   Command: {}", cmd));
        }
        println!();
    }

    // Ask if user wants to fix issues
    let fixable_count = issues.iter().filter(|i| i.auto_fixable).count();

    if fixable_count == 0 {
        ui.warning("No automatic fixes available. Please review and fix manually.");
        println!();
        log_self_health_result(req_id, start_time, 1, Some("Manual intervention required".to_string()));
        std::process::exit(1);
    }

    ui.section_header("🔨", "Proposed Fixes");
    ui.info(&format!("{} issue(s) can be fixed automatically.", fixable_count));
    println!();

    let fixable_issues: Vec<_> = issues.iter().filter(|i| i.auto_fixable).collect();
    for (i, issue) in fixable_issues.iter().enumerate() {
        ui.info(&format!("{}. {}", i + 1, issue.fix_description));
        if let Some(cmd) = &issue.fix_command {
            ui.info(&format!("   $ {}", cmd));
        }
    }
    println!();

    ui.warning("Some commands may require sudo privileges.");
    println!();

    // Confirmation prompt
    if !ui.prompt_yes_no("Proceed with automatic fixes?") {
        ui.info("Repair cancelled. No changes made.");
        println!();
        log_self_health_result(req_id, start_time, EXIT_SUCCESS, Some("User cancelled".to_string()));
        std::process::exit(EXIT_SUCCESS);
    }

    println!();
    ui.section_header("⚙️", "Applying Fixes");
    println!();

    // Apply fixes
    let mut success_count = 0;
    let mut fail_count = 0;

    for issue in fixable_issues {
        ui.info(&format!("Fixing: {}", issue.description));

        if let Some(cmd) = &issue.fix_command {
            // Special handling for daemon repair
            let result = if issue.description.contains("Anna daemon") {
                // Use systemd module for comprehensive daemon repair
                match crate::systemd::repair_service() {
                    Ok(report) => {
                        // Print repair details
                        for line in report.lines() {
                            if !line.is_empty() {
                                ui.info(&format!("  {}", line));
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                apply_fix(cmd).await
            };

            match result {
                Ok(_) => {
                    ui.success("  ✓ Fixed");
                    success_count += 1;
                }
                Err(e) => {
                    ui.error(&format!("  ✗ Failed: {}", e));
                    fail_count += 1;
                }
            }
        }
        println!();
    }

    // Summary
    ui.section_header("📊", "Summary");
    ui.info(&format!("Fixes applied: {} succeeded, {} failed", success_count, fail_count));
    println!();

    if fail_count > 0 {
        ui.warning("Some fixes failed. Please review errors above.");
        ui.info("You may need to run some commands manually with sudo.");
        println!();
        log_self_health_result(req_id, start_time, 1, Some(format!("{} fixes failed", fail_count)));
        std::process::exit(1);
    } else {
        ui.success("All fixes applied successfully!");
        ui.info("Anna should now be fully operational.");
        println!();
        log_self_health_result(req_id, start_time, EXIT_SUCCESS, None);
        std::process::exit(EXIT_SUCCESS);
    }
}

// Self-health data structures

#[derive(Debug)]
struct SelfHealthIssue {
    severity: IssueSeverity,
    description: String,
    fix_description: String,
    fix_command: Option<String>,
    auto_fixable: bool,
}

#[derive(Debug)]
enum IssueSeverity {
    High,
    Medium,
    Low,
}

// Self-health check functions

fn check_daemon_running() -> bool {
    use std::process::Command;

    let output = Command::new("systemctl")
        .args(&["is-active", "annad"])
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

async fn check_socket_connectivity() -> bool {
    RpcClient::connect().await.is_ok()
}

fn check_context_database() -> Vec<SelfHealthIssue> {
    let mut issues = Vec::new();

    // Check primary location
    let primary_db = Path::new("/var/lib/anna/context/context.db");
    let user_db = dirs::home_dir().map(|h| h.join(".local/share/anna/context.db"));

    let db_path = if primary_db.exists() {
        Some(primary_db)
    } else if let Some(ref path) = user_db {
        if path.exists() {
            Some(path.as_path())
        } else {
            None
        }
    } else {
        None
    };

    if let Some(db) = db_path {
        // Check if readable
        if !db.metadata().map(|m| m.permissions().readonly()).unwrap_or(true) {
            // Database exists and is writable
        } else {
            issues.push(SelfHealthIssue {
                severity: IssueSeverity::Medium,
                description: format!("Context database is read-only: {}", db.display()),
                fix_description: "Fix database permissions".to_string(),
                fix_command: Some(format!("sudo chmod 660 {}", db.display())),
                auto_fixable: false, // Requires careful permission handling
            });
        }
    } else {
        issues.push(SelfHealthIssue {
            severity: IssueSeverity::Low,
            description: "Context database not found".to_string(),
            fix_description: "Database will be created on first daemon run".to_string(),
            fix_command: None,
            auto_fixable: false,
        });
    }

    issues
}

fn check_key_directories() -> Vec<SelfHealthIssue> {
    let mut issues = Vec::new();

    let dirs_to_check = vec![
        ("/etc/anna", "0755", "Configuration directory"),
        ("/var/lib/anna", "0770", "Data directory"),
        ("/var/log/anna", "0770", "Log directory"),
        ("/run/anna", "0770", "Runtime directory"),
    ];

    for (dir_path, _expected_perms, description) in dirs_to_check {
        let path = Path::new(dir_path);

        if !path.exists() {
            issues.push(SelfHealthIssue {
                severity: IssueSeverity::Medium,
                description: format!("{} missing: {}", description, dir_path),
                fix_description: format!("Create {} directory", description.to_lowercase()),
                fix_command: Some(format!("sudo mkdir -p {} && sudo chown root:anna {} && sudo chmod 770 {}",
                    dir_path, dir_path, dir_path)),
                auto_fixable: false, // Requires sudo
            });
        }
    }

    issues
}

async fn apply_fix(command: &str) -> Result<()> {
    use std::process::Command;

    // Parse command (simple split on whitespace)
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    let (program, args) = if parts[0] == "sudo" {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid sudo command"));
        }
        (parts[1], &parts[2..])
    } else {
        (parts[0], &parts[1..])
    };

    let output = Command::new(program)
        .args(args)
        .output()
        .context("Failed to execute command")?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("{}", stderr.trim()))
    }
}

fn log_self_health_result(req_id: &str, start_time: Instant, exit_code: i32, error_msg: Option<String>) {
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: "unknown".to_string(),
        command: "repair".to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code,
        citation: "[anna:self-health]".to_string(),
        duration_ms,
        ok: exit_code == EXIT_SUCCESS,
        error: error_msg.map(|msg| ErrorDetails {
            code: "SELF_HEALTH_ISSUE".to_string(),
            message: msg,
        }),
    };
    let _ = log_entry.write();
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
