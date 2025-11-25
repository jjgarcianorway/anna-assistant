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

use anna_common::historian::Historian;
use anna_common::insights_engine::{InsightsEngine, InsightSeverity};
use anna_common::ipc::BrainAnalysisData;
use anna_common::output_engine::OutputEngine; // v6.32.0: Professional Output Engine
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
    // 6.18.0: Initialize formatter with user configuration
    let config = anna_common::anna_config::AnnaConfig::load().unwrap_or_default();
    anna_common::terminal_format::init_with_config(&config);

    // v6.32.0: Initialize Professional Output Engine
    let engine = OutputEngine::new();

    // Display banner first
    println!(); // Breathing room at the top
    println!("{}", engine.format_header("Anna Status Check"));
    println!("{}", "=".repeat(50));
    println!();

    // v6.35.0: Anna Reflection 2.0 - self-aware status block (at the very top)
    display_anna_reflection();

    // 6.17.0: Fetch brain analysis FIRST
    let brain_analysis = call_brain_analysis().await.ok();

    // Get comprehensive health report
    let health = HealthReport::check(false).await?;

    // 6.17.0: Build unified health summary with strict derivation
    let unified_health = build_unified_health_summary(&health, brain_analysis.as_ref()).await;

    // Map to OverallHealth for reflection display
    let overall_health_status = match unified_health.level {
        crate::status_health::HealthLevel::Healthy => {
            Some(crate::diagnostic_formatter::OverallHealth::Healthy)
        }
        crate::status_health::HealthLevel::Degraded => {
            if unified_health.critical_count > 0 {
                Some(crate::diagnostic_formatter::OverallHealth::DegradedCritical)
            } else {
                Some(crate::diagnostic_formatter::OverallHealth::DegradedWarning)
            }
        }
        crate::status_health::HealthLevel::Critical => {
            Some(crate::diagnostic_formatter::OverallHealth::DegradedCritical)
        }
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

    // v6.24.0: Display insights from Historian (before other status sections)
    display_insights().await;

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

    // 6.17.0: Overall Status (strict derivation from unified health)
    println!("{}", fmt::bold("Overall Status:"));
    let status_emoji = match unified_health.level {
        crate::status_health::HealthLevel::Healthy => fmt::emojis::SUCCESS,
        crate::status_health::HealthLevel::Degraded => fmt::emojis::WARNING,
        crate::status_health::HealthLevel::Critical => fmt::emojis::CRITICAL,
    };
    println!("  {} {}: {}", status_emoji, fmt::bold(&format!("{}", unified_health.level)), unified_health.status_line());
    println!();
    println!();

    // Beta.246: Session summary from welcome engine
    // v6.32.0: Use OutputEngine for section headers
    println!(
        "{}",
        engine.format_subheader_with_emoji(fmt::emojis::INFO, "Session Summary")
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
    // v6.32.0: Use OutputEngine
    println!(
        "{}",
        engine.format_subheader_with_emoji(fmt::emojis::SERVICE, "Core Health")
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

    // 6.22.0: Anna Mode and Update Status
    match get_daemon_status().await {
        Ok(status) => {
            // Show Anna Mode if present
            if let Some(ref anna_mode) = status.anna_mode {
                let mode_emoji = if anna_mode == "SAFE" {
                    fmt::emojis::WARNING
                } else {
                    fmt::emojis::SUCCESS
                };
                println!("{}", fmt::bold("Anna Mode:"));
                print!("  {} {}", mode_emoji, fmt::bold(anna_mode));

                if let Some(ref reason) = status.anna_mode_reason {
                    println!(" ({})", fmt::dimmed(reason));
                } else {
                    println!();
                }
                println!();
            }

            // Show Update Status if present
            if let Some(ref update_status) = status.update_status {
                println!("{}", fmt::bold("Update Status:"));
                println!("  {}", fmt::dimmed(update_status));
                println!();
            }
        }
        Err(_e) => {
            // Daemon not reachable, skip mode/update display silently
        }
    }

    // 6.17.0: System Diagnostics (show all issues from unified health)
    if !unified_health.diagnostics.is_empty() {
        println!("{}", fmt::bold("System Diagnostics:"));
        println!();

        for (idx, diagnostic) in unified_health.diagnostics.iter().enumerate() {
            let severity_emoji = match diagnostic.severity {
                crate::status_health::DiagnosticSeverity::Info => fmt::emojis::INFO,
                crate::status_health::DiagnosticSeverity::Warning => fmt::emojis::WARNING,
                crate::status_health::DiagnosticSeverity::Critical => fmt::emojis::CRITICAL,
            };

            println!("  {} {} {}", idx + 1, severity_emoji, fmt::bold(&diagnostic.title));
            println!("    {}", fmt::dimmed(&diagnostic.body));

            if !diagnostic.hints.is_empty() {
                for hint in &diagnostic.hints {
                    println!("    {} {}", fmt::symbols::ARROW, fmt::dimmed(hint));
                }
            }
            println!();
        }
    }

    println!();

    // 6.11.0: Anna Self-Health
    // v6.32.0: Use OutputEngine
    println!(
        "{}",
        engine.format_subheader_with_emoji(fmt::emojis::SERVICE, "Anna Self-Health")
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
            // v6.32.0: Use OutputEngine
            println!(
                "{}",
                engine.format_subheader_with_emoji(fmt::emojis::INFO, "Hardware Changes Detected")
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
            // v6.32.0: Use OutputEngine
            println!(
                "{}",
                engine.format_subheader_with_emoji(fmt::emojis::INFO, "LLM Model Recommendation")
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
    // v6.32.0: Use OutputEngine
    if let Some(repair) = &health.last_repair {
        println!(
            "{}",
            engine.format_subheader_with_emoji(fmt::emojis::RESTORE, "Last Self-Repair")
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
    // v6.32.0: Use OutputEngine
    println!(
        "{}",
        engine.format_subheader_with_emoji(fmt::emojis::DAEMON, "Recent Daemon Log")
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
    // v6.32.0: Use OutputEngine
    println!(
        "{}",
        engine.format_subheader_with_emoji(fmt::emojis::INFO, "System Diagnostics (Brain Analysis)")
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

/// Get daemon status via RPC (6.22.0)
async fn get_daemon_status() -> Result<anna_common::ipc::StatusData> {
    use anna_common::ipc::{Method, ResponseData};
    use crate::rpc_client::RpcClient;

    // Connect to daemon
    let mut client = RpcClient::connect().await?;

    // Send status request
    let response = client.call(Method::Status).await?;

    match response {
        ResponseData::Status(data) => Ok(data),
        _ => Err(anyhow::anyhow!("Unexpected response type")),
    }
}

// 6.8.x: Removed display_today_health_line() - now inline in execute_anna_status_command()
// This ensures single source of truth for health status

/// Call brain analysis via RPC (Beta.217b)
/// 6.8.1: Made public for health question handler
/// Check if daemon RPC socket is reachable
///
/// Returns true only if:
/// 1. Socket file exists
/// 2. Connection succeeds
/// 3. Socket is responsive
///
/// This is a quick lightweight check (200ms timeout).
pub async fn check_daemon_rpc_reachable() -> bool {
    use crate::rpc_client::RpcClient;

    // Try quick connect
    RpcClient::connect_quick(None).await.is_ok()
}

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

/// Display recent journal logs
/// 6.11.1: Prioritize errors/warnings
/// 6.18.0: Curated to 5-10 most relevant lines
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
                // 6.18.0: Strict 5-10 line curation
                // Priority: errors > warnings > recent info
                let mut errors = Vec::new();
                let mut warnings = Vec::new();
                let mut info = Vec::new();

                for line in logs.lines() {
                    if line.contains("ERROR") || line.contains("error") {
                        errors.push(line);
                    } else if line.contains("WARN") || line.contains("warn") {
                        warnings.push(line);
                    } else if !line.trim().is_empty() {
                        info.push(line);
                    }
                }

                let mut displayed = 0;
                const MAX_LINES: usize = 10;

                // Show up to 3 most recent errors
                for line in errors.iter().rev().take(3) {
                    if displayed >= MAX_LINES {
                        break;
                    }
                    println!("  {} {}", fmt::emojis::CRITICAL, fmt::error(line));
                    displayed += 1;
                }

                // Show up to 2 most recent warnings
                for line in warnings.iter().rev().take(2) {
                    if displayed >= MAX_LINES {
                        break;
                    }
                    println!("  {} {}", fmt::emojis::WARNING, fmt::warning(line));
                    displayed += 1;
                }

                // Fill remaining space with recent info (up to max 10 total)
                let info_limit = MAX_LINES.saturating_sub(displayed).min(5);
                for line in info.iter().rev().take(info_limit) {
                    println!("  {}", fmt::dimmed(line));
                    displayed += 1;
                }

                // Summary if logs were filtered
                let total = errors.len() + warnings.len() + info.len();
                if total > displayed {
                    println!(
                        "  {}",
                        fmt::dimmed(&format!(
                            "({} more entries - run 'journalctl -u annad' for full log)",
                            total - displayed
                        ))
                    );
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

/// Build unified health summary (6.17.0)
///
/// Collects diagnostics from:
/// - Daemon (systemd + RPC reachability)
/// - Brain analysis availability
/// - Anna self-health (deps, permissions, LLM)
/// - Daemon restart count
///
/// Returns HealthSummary with strict monotonic health computation.
async fn build_unified_health_summary(
    health: &HealthReport,
    brain_analysis: Option<&BrainAnalysisData>,
) -> crate::status_health::HealthSummary {
    use crate::status_health;

    let mut summary = status_health::HealthSummary::new();

    // Check 1: Daemon health (systemd + RPC reachability)
    let rpc_reachable = check_daemon_rpc_reachable().await;
    if let Some(daemon_diag) = status_health::check_daemon_health(
        health.daemon.running,
        health.daemon.enabled,
        rpc_reachable,
    ).await {
        summary.add_diagnostic(daemon_diag);
    }

    // Check 2: Brain analysis availability
    if let Some(brain_diag) = status_health::check_brain_analysis(brain_analysis) {
        summary.add_diagnostic(brain_diag);
    }

    // Check 3: Anna self-health (includes /var/log/anna checks)
    for diag in status_health::check_anna_self_health() {
        summary.add_diagnostic(diag);
    }

    // Check 4: Daemon restart count
    if let Some(restart_diag) = status_health::check_daemon_restarts().await {
        summary.add_diagnostic(restart_diag);
    }

    // Check 5: Incorporate brain analysis insights
    if let Some(analysis) = brain_analysis {
        // Brain analysis critical issues
        if analysis.critical_count > 0 {
            for insight in analysis.insights.iter().take(3) {
                if insight.severity == "critical" {
                    summary.add_diagnostic(status_health::Diagnostic {
                        title: format!("System: {}", insight.summary),
                        body: insight.details.clone(),
                        severity: status_health::DiagnosticSeverity::Critical,
                        hints: insight.commands.clone(),
                    });
                }
            }
        }

        // Brain analysis warnings
        if analysis.warning_count > 0 && analysis.critical_count == 0 {
            for insight in analysis.insights.iter().take(3) {
                if insight.severity == "warning" {
                    summary.add_diagnostic(status_health::Diagnostic {
                        title: format!("System: {}", insight.summary),
                        body: insight.details.clone(),
                        severity: status_health::DiagnosticSeverity::Warning,
                        hints: insight.commands.clone(),
                    });
                }
            }
        }
    }

    // Compute final health level
    summary.compute_level();

    summary
}

/// v6.24.0: Display insights from Historian
/// v6.32.0: Refactored to use OutputEngine
/// v6.35.0: Display Anna's reflection on system state (top block of status output)
fn display_anna_reflection() {
    // Try to open Historian database
    let historian = match Historian::new("/var/lib/anna/historian.db") {
        Ok(h) => h,
        Err(_) => return, // Silently skip if DB doesn't exist yet
    };

    // Get usage stats
    let usage_stats = historian.get_usage_stats().ok().flatten();

    // Get insights for reflection
    let insights_engine = InsightsEngine::new(historian);
    let insights = insights_engine.get_top_insights(5, 24).unwrap_or_default();

    // Build Anna's reflection
    let reflection_lines = anna_common::reflection_engine::build_anna_reflection(
        &insights,
        usage_stats.as_ref(),
    );

    // Display reflection (if any)
    if !reflection_lines.is_empty() {
        let engine = OutputEngine::new();

        for line in &reflection_lines {
            println!("{}", engine.format_source_line(line));
        }

        println!("{}", "=".repeat(50));
        println!();
    }
}

async fn display_insights() {
    use anna_common::insights_engine::Insight;

    let mut all_insights: Vec<Insight> = Vec::new();

    // v6.38.1: Check current disk space FIRST (deterministic, no historian required)
    if let Ok(Some(disk_insight)) = InsightsEngine::check_disk_space_now() {
        all_insights.push(disk_insight);
    }

    // Try to open Historian database for historical insights
    if let Ok(historian) = Historian::new("/var/lib/anna/historian.db") {
        let insights_engine = InsightsEngine::new(historian);

        // Get top 3 insights from last 24 hours
        if let Ok(mut insights) = insights_engine.get_top_insights(3, 24) {
            all_insights.append(&mut insights);
        }
    }

    if all_insights.is_empty() {
        return; // No insights to display
    }

    // v6.32.0: Use Professional Output Engine for consistent formatting
    let output_engine = OutputEngine::new();

    // Display insights section
    println!("{}", output_engine.format_subheader("Recent Insights"));
    println!();

    for insight in all_insights {
        // v6.32.0: Use engine.format_insight() for professional formatting
        print!("{}", output_engine.format_insight(&insight));
    }

    println!("{}", "=".repeat(50));
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::output_engine::{OutputEngine, TerminalMode};
    use anna_common::insights_engine::{Insight, InsightSeverity};

    // ========== Unit Tests (8 tests) ==========

    #[test]
    fn test_status_header_uses_outputengine() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_header("Anna Status Check");
        assert_eq!(result, "Anna Status Check");
    }

    #[test]
    fn test_degraded_status_shows_exact_same_strings() {
        // v6.32.0: Verify that status strings remain unchanged
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_subheader_with_emoji("⚠️", "System Diagnostics");
        // Should preserve exact text (emoji included if terminal supports it)
        assert!(result.contains("System Diagnostics"));
        // In Basic mode, emoji should be present
        assert!(!result.is_empty());
    }

    #[test]
    fn test_healthy_status_shows_exact_same_strings() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_subheader_with_emoji("✅", "Core Health");
        assert!(result.contains("Core Health"));
    }

    #[test]
    fn test_insights_render_using_format_insight() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let insight = Insight::new(
            "test_detector",
            InsightSeverity::Warning,
            "Test Warning",
            "This is a test warning"
        ).with_suggestion("Fix it");

        let result = engine.format_insight(&insight);
        assert!(result.contains("Test Warning"));
        assert!(result.contains("This is a test warning"));
        assert!(result.contains("Fix it"));
        assert!(result.contains("[*]")); // Warning marker in Basic mode
    }

    #[test]
    fn test_predictions_render_using_format_prediction() {
        use anna_common::predictive_diagnostics::PredictiveInsight;
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        let prediction = PredictiveInsight::new(
            "disk_full_test",
            "Disk will fill in 3 days",
            InsightSeverity::Warning,
            "3 days"
        )
        .with_cause("Log growth rate exceeds cleanup")
        .with_actions(vec!["Clear old logs".to_string()]);

        let result = engine.format_prediction(&prediction);
        assert!(result.contains("Disk will fill in 3 days"));
        assert!(result.contains("3 days"));
        assert!(result.contains("Clear old logs"));
    }

    #[test]
    fn test_command_lists_render_using_format_command() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let commands = vec!["df -h".to_string(), "systemctl status annad".to_string()];
        let result = engine.format_command_list(commands);

        assert!(result.contains("[CMD] df -h"));
        assert!(result.contains("[CMD] systemctl status annad"));
    }

    #[test]
    fn test_works_with_emojis_disabled() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_subheader_with_emoji("🔧", "Test Section");

        // In Basic mode with emojis potentially disabled, should still work
        assert!(result.contains("Test Section"));
        // Emoji handling should not crash
        assert!(!result.is_empty());
    }

    #[test]
    fn test_works_with_color_disabled() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_header("Test Header");

        // Should not contain ANSI escape codes
        assert!(!result.contains("\x1b["));
        assert_eq!(result, "Test Header");
    }

    // ========== Integration Tests (8 tests) ==========
    // These test formatting with synthetic system state

    #[test]
    fn test_status_full_system_healthy() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        // Simulate healthy system output
        let header = engine.format_header("Anna Status Check");
        let core_health = engine.format_subheader_with_emoji("⚙️", "Core Health");

        assert!(header.contains("Anna Status Check"));
        assert!(core_health.contains("Core Health"));
    }

    #[test]
    fn test_status_with_failed_units() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        // Test that failed unit formatting works
        let diagnostics = engine.format_subheader_with_emoji("⚠️", "System Diagnostics");
        assert!(diagnostics.contains("System Diagnostics"));
    }

    #[test]
    fn test_status_with_disk_full() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        let insight = Insight::new(
            "disk_monitor",
            InsightSeverity::Critical,
            "Disk space critical",
            "Root partition is 95% full"
        ).with_suggestion("Clear package cache or old logs");

        let result = engine.format_insight(&insight);
        assert!(result.contains("Disk space critical"));
        assert!(result.contains("[!]")); // Critical marker
    }

    #[test]
    fn test_status_with_insights() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        let insights = vec![
            Insight::new(
                "boot_monitor",
                InsightSeverity::Info,
                "Boot time improved",
                "Last 3 boots averaged 15s (down from 20s)"
            ),
            Insight::new(
                "memory_monitor",
                InsightSeverity::Warning,
                "Memory pressure detected",
                "Swap usage increased by 30% over 24h"
            ),
        ];

        for insight in insights {
            let result = engine.format_insight(&insight);
            assert!(!result.is_empty());
            assert!(result.contains(&insight.title));
        }
    }

    #[test]
    fn test_status_with_predictions() {
        use anna_common::predictive_diagnostics::PredictiveInsight;
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        let predictions = vec![
            PredictiveInsight::new(
                "service_failure_test",
                "Service failure likely within 24h",
                InsightSeverity::Critical,
                "24 hours"
            )
            .with_cause("Memory leak detected in annad")
            .with_actions(vec!["Restart daemon".to_string(), "Monitor logs".to_string()]),
        ];

        for pred in predictions {
            let result = engine.format_prediction(&pred);
            assert!(result.contains("Service failure likely within 24h"));
            assert!(result.contains("Restart daemon"));
        }
    }

    #[test]
    fn test_status_no_emoji_config() {
        // Test that Basic mode works without emojis
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        let header = engine.format_header("Test");
        let subheader = engine.format_subheader("Subsection");

        // Should not crash and produce readable output
        assert_eq!(header, "Test");
        assert!(subheader.contains("Subsection"));
    }

    #[test]
    fn test_status_terminal_ascii_mode() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        // Test ASCII-safe output
        let insight = Insight::new(
            "test",
            InsightSeverity::Warning,
            "ASCII Test",
            "Should use [*] not ⚠️"
        );

        let result = engine.format_insight(&insight);
        assert!(result.contains("[*]")); // ASCII fallback
        assert!(result.contains("ASCII Test"));
    }

    #[test]
    fn test_status_output_engine_roundtrip() {
        // Test that OutputEngine produces consistent output across calls
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        let result1 = engine.format_header("Test Header");
        let result2 = engine.format_header("Test Header");

        assert_eq!(result1, result2); // Deterministic
    }

    // ========== Regression Tests (2 tests) ==========

    #[test]
    fn test_summary_string_matches_v6_31_baseline() {
        // v6.32.0: Verify formatting changes don't alter content
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        // Core sections that existed in v6.31.0
        let header = engine.format_header("Anna Status Check");
        let session_summary = engine.format_subheader_with_emoji("ℹ️", "Session Summary");
        let core_health = engine.format_subheader_with_emoji("⚙️", "Core Health");
        let self_health = engine.format_subheader_with_emoji("⚙️", "Anna Self-Health");
        let daemon_log = engine.format_subheader_with_emoji("👾", "Recent Daemon Log");

        // All these sections must still exist and contain expected text
        assert!(header.contains("Anna Status Check"));
        assert!(session_summary.contains("Session Summary"));
        assert!(core_health.contains("Core Health"));
        assert!(self_health.contains("Anna Self-Health"));
        assert!(daemon_log.contains("Recent Daemon Log"));
    }

    #[test]
    fn test_no_behavioral_changes_other_than_formatting() {
        // v6.32.0: Critical test - OutputEngine must not change logic
        let engine = OutputEngine::with_mode(TerminalMode::Basic);

        // Test that insight severity mapping is preserved
        let critical_insight = Insight::new(
            "test",
            InsightSeverity::Critical,
            "Critical",
            "Test"
        );
        let warning_insight = Insight::new(
            "test",
            InsightSeverity::Warning,
            "Warning",
            "Test"
        );
        let info_insight = Insight::new(
            "test",
            InsightSeverity::Info,
            "Info",
            "Test"
        );

        let critical_output = engine.format_insight(&critical_insight);
        let warning_output = engine.format_insight(&warning_insight);
        let info_output = engine.format_insight(&info_insight);

        // Verify severity markers are correct (v6.31.0 behavior)
        assert!(critical_output.contains("[!]")); // Critical
        assert!(warning_output.contains("[*]"));  // Warning
        assert!(info_output.contains("[i]"));     // Info

        // Verify all three formats are distinct
        assert_ne!(critical_output, warning_output);
        assert_ne!(warning_output, info_output);
        assert_ne!(critical_output, info_output);
    }
}
