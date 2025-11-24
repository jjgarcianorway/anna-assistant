//! Status command - comprehensive health report (Beta.211: integrated welcome engine)
//!
//! Real Anna: `annactl status`
//! Purpose: Verify Anna herself is healthy and functioning
//! Checks:
//! - Anna's version and LLM mode
//! - Daemon status (annad)
//! - LLM backend health
//! - Permissions and groups
//! - Recent daemon logs
//! - Welcome report with system changes detection
//! Behavior:
//! - Performs self-diagnostics
//! - Shows human-readable status
//! - Exits 0 if healthy, non-zero if unhealthy
//! Output:
//! - Comprehensive health report
//! - Journal excerpts
//! - Deterministic welcome report (CLI formatted)
//! - Clear status: Healthy / Degraded / Broken

use anna_common::terminal_format as fmt;
use anyhow::Result;
use std::process::Command;
use std::time::Instant;

use crate::health::{HealthReport, HealthStatus};
use crate::logging::{ErrorDetails, LogEntry};
use crate::output;
use crate::startup::welcome;
use crate::telemetry;
use crate::version_banner;

/// Execute 'annactl status' command - comprehensive health check
pub async fn execute_anna_status_command(
    _json: bool,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    // Display banner first
    println!(); // Breathing room at the top
    println!("{}", fmt::bold("Anna Status Check"));
    println!("{}", "=".repeat(50));
    println!();

    // 6.10.0: Fetch brain analysis FIRST to get authoritative health status
    // This allows reflection header to be honest about system state
    let overall_health_status = match call_brain_analysis().await {
        Ok(analysis) => {
            use crate::diagnostic_formatter::compute_overall_health;
            Some(compute_overall_health(&analysis))
        }
        Err(_) => None, // Daemon offline, use HealthReport fallback
    };

    // 6.7.0/6.10.0: Show reflection with health-aware header
    let reflection = crate::reflection_helper::build_local_reflection();
    let reflection_text = crate::reflection_helper::format_reflection(&reflection, true, overall_health_status);
    print!("{}", reflection_text);
    if reflection.items.is_empty() {
        println!();
    }
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

    // Display "Today:" health line
    if let Some(health_status) = overall_health_status {
        use crate::diagnostic_formatter::format_today_health_line_from_health;
        let health_text = format_today_health_line_from_health(health_status);
        println!("{} {}", fmt::bold("Today:"), health_text);
    } else {
        // Fallback if brain analysis unavailable
        let health_text = match health.status {
            HealthStatus::Healthy => "System healthy",
            HealthStatus::Degraded => "System degraded – some issues detected",
            HealthStatus::Broken => "System broken – critical failures present",
        };
        println!("{} {}", fmt::bold("Today:"), health_text);
    }
    println!();
    println!();

    // Beta.246: Session summary from welcome engine
    println!(
        "{}",
        fmt::section_title(&fmt::emojis::INFO, "Session Summary")
    );
    println!();

    // Fetch telemetry and generate session summary
    match crate::system_query::query_system_telemetry() {
        Ok(telemetry_data) => {
            // Load last session
            let last_session = welcome::load_last_session().ok().flatten();

            // Create current snapshot
            let current_snapshot = welcome::create_telemetry_snapshot(&telemetry_data);

            // Generate compact session summary (not the full welcome report)
            let session_summary = welcome::generate_session_summary(last_session, current_snapshot.clone());

            // Format with CLI colors via normalizer
            let formatted = crate::output::normalize_for_cli(&session_summary);
            println!("{}", formatted);

            // Save session metadata for next run
            let _ = welcome::save_session_metadata(current_snapshot);
        }
        Err(e) => {
            println!(
                "{}",
                fmt::dimmed(&format!(
                    "Unable to fetch telemetry for session summary: {}",
                    e
                ))
            );
        }
    }

    println!();
    println!("{}", fmt::dimmed("System health details follow in the diagnostics section."));
    println!();
    println!();

    // Beta.141: Enhanced core health display with emojis
    println!(
        "{}",
        fmt::section_title(&fmt::emojis::SERVICE, "Core Health")
    );
    println!();

    // Daemon
    if health.daemon.installed && health.daemon.enabled && health.daemon.running {
        println!("  {}", fmt::component_status("Daemon (annad)", "running"));
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
            fmt::component_status(&format!("LLM ({})", health.llm.backend), "running")
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
        println!("  {}", fmt::component_status("Permissions", "healthy"));
        println!("    {}", fmt::dimmed("data directories and user groups OK"));
    } else {
        println!("  {}", fmt::component_status("Permissions", "degraded"));
        for issue in &health.permissions.issues {
            println!("    {} {}", fmt::emojis::WARNING, fmt::dimmed(issue));
        }
    }

    println!();

    // 6.8.x: Overall Status (uses same health determination as "Today:")
    println!("{}", fmt::bold("Overall Status:"));
    if let Some(brain_health) = overall_health_status {
        // Use brain analysis health (authoritative)
        use crate::diagnostic_formatter::OverallHealth;
        match brain_health {
            OverallHealth::Healthy => {
                println!("  {} {}", fmt::emojis::SUCCESS, fmt::bold("HEALTHY: all systems operational"));
            }
            OverallHealth::DegradedWarning => {
                println!("  {} {}", fmt::emojis::WARNING, fmt::bold("DEGRADED: warnings detected"));
            }
            OverallHealth::DegradedCritical => {
                println!("  {} {}", fmt::emojis::CRITICAL, fmt::bold("DEGRADED: critical issues require attention"));
            }
        }
    } else {
        // Fallback to HealthReport if brain analysis unavailable
        health.display_summary();
    }
    println!();

    // 6.11.0: Anna Self-Health
    println!(
        "{}",
        fmt::section_title(&fmt::emojis::SERVICE, "Anna Self-Health")
    );
    println!();

    let self_health = anna_common::anna_self_health::check_anna_self_health();

    if self_health.deps_ok && self_health.permissions_ok && self_health.llm_ok {
        println!("  {} {}", fmt::emojis::SUCCESS, "All dependencies, permissions, and LLM backend are healthy");
    } else {
        // Show issues
        if !self_health.deps_ok {
            println!("  {} {}", fmt::emojis::WARNING, "Missing dependencies:");
            for dep in &self_health.missing_deps {
                println!("    {} {}", fmt::symbols::ARROW, dep);
            }
        }

        if !self_health.permissions_ok {
            println!("  {} {}", fmt::emojis::WARNING, "Permission issues:");
            for issue in &self_health.missing_permissions {
                println!("    {} {}", fmt::symbols::ARROW, issue);
            }
        }

        if !self_health.llm_ok {
            println!("  {} LLM: {}", fmt::emojis::WARNING, self_health.llm_details);
        }
    }

    println!();

    // 6.11.0: Hardware Profile and LLM Recommendation
    let current_hw = anna_common::anna_hardware_profile::detect_current_hardware();
    if let Some(previous_hw) = anna_common::anna_hardware_profile::AnnaHardwareProfile::read() {
        // Check for hardware changes
        if let Some(changes) = anna_common::anna_hardware_profile::compare_profiles(&previous_hw, &current_hw) {
            println!(
                "{}",
                fmt::section_title(&fmt::emojis::INFO, "Hardware Changes Detected")
            );
            println!();
            println!("  {} {}", fmt::emojis::INFO, changes);
            println!();
        }

        // Check LLM recommendation
        let recommended_model = anna_common::anna_hardware_profile::recommend_llm_model(
            current_hw.total_ram_gib,
            current_hw.cpu_cores,
        );

        // Get currently configured model (try to read from context.db)
        let current_model = previous_hw.last_llm_model;

        if !current_model.is_empty() && recommended_model != current_model {
            println!(
                "{}",
                fmt::section_title(&fmt::emojis::INFO, "LLM Model Recommendation")
            );
            println!();
            println!("  Current model: {}", current_model);
            println!("  Recommended model: {} (based on {} GiB RAM, {} cores)",
                     recommended_model, current_hw.total_ram_gib, current_hw.cpu_cores);
            println!();
            println!("  {} This is advisory only - Anna will not change your config automatically", fmt::emojis::INFO);
            println!("  To change, update your LLM config or run: ollama pull {}", recommended_model);
            println!();
        }
    }

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

    // Beta.211: Welcome report - DEFERRED to Beta.212 due to telemetry module restructuring
    // TODO(Beta.212): Re-enable welcome report after RPC telemetry integration is complete
    /*
    println!(
        "{}",
        fmt::section_title(&fmt::emojis::INFO, "System Welcome Report")
    );
    println!();

    // Fetch telemetry and generate welcome report
    match telemetry::fetch_cached().await {
        Ok(telemetry_data) => {
            // Load last session
            let last_session = welcome::load_last_session().ok().flatten();

            // Create current snapshot
            let current_snapshot = welcome::create_telemetry_snapshot(&telemetry_data);

            // Generate welcome report
            let welcome_report = welcome::generate_welcome_report(last_session, current_snapshot.clone());

            // Format with CLI colors via normalizer
            let formatted = output::normalize_for_cli(&welcome_report);
            println!("{}", formatted);

            // Save session metadata for next run
            let _ = welcome::save_session_metadata(current_snapshot);
        }
        Err(e) => {
            println!(
                "{}",
                fmt::dimmed(&format!(
                    "Unable to fetch telemetry for welcome report: {}",
                    e
                ))
            );
        }
    }

    println!();
    */

    // Beta.217b: Sysadmin Brain Analysis
    // Beta.250: Now uses canonical diagnostic formatter
    println!(
        "{}",
        fmt::section_title(&fmt::emojis::INFO, "System Diagnostics (Brain Analysis)")
    );
    println!();

    // Call brain analysis via RPC
    match call_brain_analysis().await {
        Ok(analysis) => {
            // Beta.250: Use canonical inline summary formatter
            let summary = crate::diagnostic_formatter::format_diagnostic_summary_inline(&analysis);

            // Apply CLI formatting
            let formatted = crate::output::normalize_for_cli(&summary);
            print!("{}", formatted);
        }
        Err(e) => {
            println!(
                "  {}",
                fmt::dimmed(&format!("Brain analysis unavailable: {}", e))
            );
        }
    }

    println!();

    // Log command and exit with appropriate code
    let exit_code = health.exit_code();
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Compute state string from actual health status
    let state = match health.status {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Broken => "broken",
    };

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

// 6.8.x: Removed display_today_health_line() - now inline in execute_anna_status_command()
// This ensures single source of truth for health status

/// Call brain analysis via RPC (Beta.217b)
/// 6.8.1: Made public for health question handler
pub async fn call_brain_analysis() -> Result<anna_common::ipc::BrainAnalysisData> {
    use anna_common::ipc::{Method, ResponseData};
    use crate::rpc_client::RpcClient;

    // Connect to daemon
    let mut client = RpcClient::connect().await?;

    // Send brain analysis request
    let response = client.call(Method::BrainAnalysis).await?;

    match response {
        ResponseData::BrainAnalysis(data) => Ok(data),
        _ => Err(anyhow::anyhow!("Unexpected response type")),
    }
}

/// Display recent journal logs (6.11.1: prioritize errors/warnings)
fn display_recent_logs() {
    let output = Command::new("journalctl")
        .args([
            "-u",
            "annad",
            "-n",
            "50",  // Fetch more logs to filter
            "--no-pager",
            "--output=cat",  // Beta.231: Show only message (already has timestamp from tracing)
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let logs = String::from_utf8_lossy(&output.stdout);
            if logs.trim().is_empty() {
                println!("  {}", fmt::dimmed("No recent logs"));
            } else {
                // 6.11.1: Categorize logs by severity
                let mut errors = Vec::new();
                let mut warnings = Vec::new();
                let mut info = Vec::new();

                for line in logs.lines() {
                    if line.contains("ERROR") || line.contains("error") {
                        errors.push(line);
                    } else if line.contains("WARN") || line.contains("warn") {
                        warnings.push(line);
                    } else {
                        info.push(line);
                    }
                }

                // Show errors first (all of them)
                for line in errors.iter() {
                    println!("  {} {}", fmt::emojis::CRITICAL, fmt::error(line));
                }

                // Show warnings next (up to 3)
                for line in warnings.iter().take(3) {
                    println!("  {} {}", fmt::emojis::WARNING, fmt::warning(line));
                }

                // Show recent info logs (up to 5, skip if we have errors/warnings)
                let info_limit = if errors.is_empty() && warnings.is_empty() { 10 } else { 5 };
                for line in info.iter().rev().take(info_limit) {
                    println!("  {}", fmt::dimmed(line));
                }

                // Summary if logs were filtered
                let total_logged = errors.len() + warnings.len() + info.len();
                if total_logged > 18 {
                    println!("  {}", fmt::dimmed(&format!("({} more log entries - run 'journalctl -u annad' for full log)", total_logged - 18)));
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
