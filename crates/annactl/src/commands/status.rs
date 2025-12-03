//! Status Command v0.0.36 - Knowledge Packs UX
//!
//! v0.0.36: Re-enabled [KNOWLEDGE] section for offline Q&A visibility
//! - Packs count, document count, index size, last indexed time
//! - Breakdown by source type (manpages, package_docs)
//! - Top packs by query count
//!
//! v0.0.35: Enhanced [MODELS] section with role selection visibility
//! - Hardware tier display (Low/Medium/High based on RAM/VRAM)
//! - Download progress with role identification and ETA
//! - Fallback mode indicators (reliability capped at 60% when Junior unavailable)
//! - Readiness summary (full capability / partial / not ready)
//!
//! v0.0.28: Simplified status display
//! - Removed rarely-useful sections for cleaner output
//! - Helpers filtered by system relevance (no ethtool if no ethernet)
//! - Focus on: VERSION, DAEMON, SNAPSHOT, HEALTH, DATA, UPDATES, ALERTS, HELPERS, POLICY, MODELS
//!
//! Sections:
//! - VERSION: Anna version
//! - DAEMON: Running/stopped status
//! - SNAPSHOT: Last data snapshot
//! - HEALTH: Overall system health
//! - DATA: Knowledge objects count
//! - UPDATES: Auto-update status
//! - ALERTS: Active alerts (if any)
//! - HELPERS: Relevant helpers for this system
//! - POLICY: Safety policy summary
//! - MODELS: LLM status (Ollama, translator, junior)

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use anna_common::config::{AnnaConfig, UpdateState, UpdateResult, UpdateMode, JuniorState};
use anna_common::model_selection::{BootstrapState, BootstrapPhase};
use anna_common::format_duration_secs;
use anna_common::daemon_state::{
    StatusSnapshot, LastCrash, SnapshotStatus,
    INTERNAL_DIR, SNAPSHOTS_DIR, STATUS_SNAPSHOT_PATH,
};
use anna_common::helpers::{get_helper_status_list, InstalledBy};
use anna_common::domain_state::{DomainSummary, RefreshRequest, REQUESTS_DIR};
use anna_common::terminal::{DisplayMode, get_terminal_size};
use anna_common::self_observation::SelfObservation;
use anna_common::control_socket::{check_daemon_health, DaemonHealth, SOCKET_PATH};
use anna_common::anomaly_engine::AlertQueue;
use anna_common::recipes::RecipeManager;
use anna_common::memory::MemoryManager;
use anna_common::policy::{get_policy, POLICY_DIR};
use anna_common::audit_log::{AuditLogger, AUDIT_LOG_FILE};
use anna_common::knowledge_packs::{KnowledgeIndex, KNOWLEDGE_PACKS_DIR};
use anna_common::source_labels::QaStats;
use anna_common::performance::{PerfStats, ToolCache, LlmCache};
use anna_common::reliability::{MetricsStore, ErrorBudgets, BudgetState, calculate_budget_status};
use anna_common::display_format::{format_bytes, format_timestamp};
use anna_common::transcript::{list_recent_cases, find_last_failure, get_cases_storage_size, load_case_summary, CaseOutcome};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the status command - v7.42.0: separate daemon and snapshot states
pub async fn run() -> Result<()> {
    // Detect display mode
    let mode = DisplayMode::detect();
    let (width, _) = get_terminal_size();

    // Generate separator based on terminal width
    let sep_len = (width as usize).saturating_sub(4).min(60);
    let thin_sep = "-".repeat(sep_len);

    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", thin_sep);
    println!();

    // v7.42.0: Check daemon health via control socket / systemd (NOT via snapshot)
    let daemon_health = check_daemon_health();

    // Load snapshot separately (this is file-based, not daemon state)
    let snapshot = StatusSnapshot::load();
    let snapshot_status = SnapshotStatus::from_snapshot(&snapshot);

    // Load other state
    let last_crash = LastCrash::load();
    let domain_summary = DomainSummary::load();
    let self_obs = SelfObservation::load();
    let checking_domains = get_pending_refresh_domains();

    // [VERSION] - always shown
    print_version_section();

    // [DAEMON] - v7.42.0: from live check, NOT snapshot
    print_daemon_section(&daemon_health, &snapshot, &last_crash, &checking_domains);

    // [SNAPSHOT] - v7.42.0: new section showing snapshot file status
    print_snapshot_section(&snapshot_status);

    // [HEALTH] - always shown
    print_health_section(&daemon_health, &snapshot, &self_obs);

    // [DATA] - condensed in compact mode
    print_data_section(&snapshot, &domain_summary, &mode);

    // v0.0.28: Simplified status display - show only relevant sections
    // Removed: TELEMETRY, LEARNING, KNOWLEDGE, Q&A, PERFORMANCE, RELIABILITY, RECENT ACTIONS, STORAGE, PATHS
    // These are rarely useful for day-to-day status checks

    // [UPDATES] - always shown
    print_updates_section(&daemon_health, &snapshot);

    // [ALERTS] - only shown if there are alerts
    print_alerts_section(&snapshot);

    // [CASES] - v0.0.33: show recent case files and storage
    print_cases_section(&mode);

    // [HELPERS] - v0.0.28: show only relevant helpers
    print_helpers_section(&mode);

    // [KNOWLEDGE] - v0.0.36: show knowledge pack stats (critical for offline Q&A)
    print_knowledge_section(&mode);

    // [POLICY] - condensed safety info
    print_policy_section();

    // [MODELS] - LLM status (critical for Q&A)
    print_models_section();

    println!("{}", thin_sep);
    println!();

    Ok(())
}

/// Get domains that are currently being refreshed
fn get_pending_refresh_domains() -> Vec<String> {
    let mut domains = Vec::new();

    if let Ok(entries) = std::fs::read_dir(REQUESTS_DIR) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(req) = serde_json::from_str::<RefreshRequest>(&content) {
                    if !req.is_expired() {
                        for domain in &req.required_domains {
                            domains.push(domain.as_str().to_string());
                        }
                    }
                }
            }
        }
    }

    domains
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  Anna:       v{}", VERSION);
    println!();
}

/// [DAEMON] section - v7.42.0: from live check (socket/systemd), NOT snapshot
fn print_daemon_section(
    health: &DaemonHealth,
    snapshot: &Option<StatusSnapshot>,
    last_crash: &Option<LastCrash>,
    checking_domains: &[String],
) {
    println!("{}", "[DAEMON]".cyan());

    match health {
        DaemonHealth::Running(status) => {
            // Daemon confirmed running via socket
            println!("  Status:     {} (socket)", "running".green());
            println!("  Version:    v{}", status.version);
            println!("  Uptime:     {}", format_duration_secs(status.uptime_secs));
            println!("  PID:        {}", status.pid);

            if !checking_domains.is_empty() {
                let domains_str = checking_domains.join(", ");
                println!("  Refresh:    {} ({})", "checking...".yellow(), domains_str.dimmed());
            }
        }
        DaemonHealth::RunningSystemd => {
            // Daemon running per systemd, but socket unavailable
            println!("  Status:     {} (systemd)", "running".green());

            // Try to get PID from systemd
            if let Some(pid) = anna_common::control_socket::get_systemd_pid() {
                println!("  PID:        {}", pid);
            }

            // Show uptime from snapshot if available
            if let Some(s) = snapshot {
                if !s.is_stale() {
                    println!("  Uptime:     ~{}", format_duration_secs(s.uptime_secs));
                }
            }

            println!("  {}", "(socket unavailable - using systemd)".dimmed());
        }
        DaemonHealth::Stopped => {
            println!("  Status:     {}", "stopped".red());

            // Show last crash info if available
            if let Some(crash) = last_crash {
                println!();
                println!("  Last crash: {}", crash.format_summary().yellow());
            }
        }
        DaemonHealth::Unknown(reason) => {
            println!("  Status:     {} ({})", "unknown".yellow(), reason.dimmed());
        }
    }

    println!();
}

/// [SNAPSHOT] section - v7.42.0: new section for snapshot file status
fn print_snapshot_section(status: &SnapshotStatus) {
    println!("{}", "[SNAPSHOT]".cyan());

    match status {
        SnapshotStatus::Available { age_secs, seq } => {
            let age_str = if *age_secs < 60 {
                format!("{}s ago", age_secs)
            } else {
                format!("{}m ago", age_secs / 60)
            };
            println!("  Status:     {} ({})", "available".green(), age_str);
            println!("  Sequence:   {}", seq);
        }
        SnapshotStatus::Stale { age_secs, seq } => {
            let age_str = if *age_secs < 3600 {
                format!("{}m old", age_secs / 60)
            } else {
                format!("{}h old", age_secs / 3600)
            };
            println!("  Status:     {} ({})", "stale".yellow(), age_str);
            println!("  Sequence:   {}", seq);
            println!("  {}", "(snapshot may be from previous run)".dimmed());
        }
        SnapshotStatus::Missing => {
            println!("  Status:     {}", "missing".red());
            println!("  {}", "(daemon may not have written snapshot yet)".dimmed());
        }
    }

    println!();
}

/// [HEALTH] section - from snapshot + self-observation
fn print_health_section(
    daemon_health: &DaemonHealth,
    snapshot: &Option<StatusSnapshot>,
    self_obs: &SelfObservation,
) {
    println!("{}", "[HEALTH]".cyan());

    if !daemon_health.is_running() {
        println!("  Overall:    {} daemon not running", "✗".red());
        println!();
        return;
    }

    match snapshot {
        Some(s) if !s.is_stale() => {
            if s.healthy {
                println!("  Overall:    {} all systems nominal", "✓".green());
            } else {
                let total_alerts = s.alerts_critical + s.alerts_warning;
                if s.alerts_critical > 0 {
                    println!("  Overall:    {} {} critical, {} warning",
                        "✗".red(),
                        s.alerts_critical.to_string().red(),
                        s.alerts_warning);
                } else if total_alerts > 0 {
                    println!("  Overall:    {} {} warning(s)",
                        "⚠".yellow(),
                        s.alerts_warning.to_string().yellow());
                } else {
                    println!("  Overall:    {} (snapshot indicates unhealthy)", "⚠".yellow());
                }
            }

            // Self-observation
            if self_obs.warning.is_some() {
                println!("  Anna:       {}", self_obs.format_summary().yellow());
            } else {
                println!("  Anna:       {}", self_obs.format_summary().dimmed());
            }
        }
        _ => {
            // Daemon running but no recent snapshot
            println!("  Overall:    {} (awaiting snapshot data)", "⚠".yellow());
        }
    }

    println!();
}

/// [DATA] section - knowledge object counts from snapshot + domain freshness
fn print_data_section(
    snapshot: &Option<StatusSnapshot>,
    domain_summary: &Option<DomainSummary>,
    mode: &DisplayMode,
) {
    println!("{}", "[DATA]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            println!("  Knowledge:  {} objects", s.knowledge_objects);

            // Last scan info
            if let Some(scan_at) = s.last_scan_at {
                let scan_age = (chrono::Utc::now() - scan_at).num_seconds().max(0) as u64;
                let age_str = if scan_age < 60 {
                    format!("{}s ago", scan_age)
                } else if scan_age < 3600 {
                    format!("{}m ago", scan_age / 60)
                } else {
                    format!("{}h ago", scan_age / 3600)
                };
                println!("  Last scan:  {} (took {}ms)", age_str.dimmed(), s.last_scan_duration_ms);
            } else {
                println!("  Last scan:  {}", "pending".dimmed());
            }

            // Domain freshness (not in compact mode)
            if *mode != DisplayMode::Compact {
                if let Some(ds) = domain_summary {
                    let total_domains = ds.fresh_domains + ds.stale_domains + ds.missing_domains;
                    if ds.stale_domains > 0 || ds.missing_domains > 0 {
                        println!("  Domains:    {}/{} fresh, {} stale, {} missing",
                            ds.fresh_domains, total_domains,
                            ds.stale_domains.to_string().yellow(),
                            ds.missing_domains);
                    } else {
                        println!("  Domains:    {}/{} fresh", ds.fresh_domains.to_string().green(), total_domains);
                    }
                }
            }
        }
        _ => {
            println!("  Knowledge:  {} (no snapshot data)", "-".dimmed());
            println!("  Last scan:  -");
        }
    }

    println!();
}

/// [TELEMETRY] section - sample counts from snapshot
fn print_telemetry_section(snapshot: &Option<StatusSnapshot>) {
    let config = AnnaConfig::load();

    println!("{}", "[TELEMETRY]".cyan());

    if !config.telemetry.enabled {
        println!("  {}", "Telemetry disabled in config.".dimmed());
        println!();
        return;
    }

    match snapshot {
        Some(s) if !s.is_stale() => {
            println!("  Samples (24h): {}", s.telemetry_samples_24h);

            if s.memory_mb > 0 {
                println!("  Memory:        {} MB", s.memory_mb);
            }

            if let Some(ref cpu_alert) = s.cpu_alert {
                println!("  CPU:           {}", cpu_alert.yellow());
            }

            if let Some(ref disk) = s.disk_info {
                println!("  Disk:          {}", disk);
            }

            if let Some(ref net) = s.network_io {
                println!("  Network:       {}", net);
            }
        }
        _ => {
            println!("  {}", "(no snapshot data)".dimmed());
        }
    }

    println!();
}

/// [UPDATES] section - update scheduler state (v0.0.11 enhanced)
fn print_updates_section(daemon_health: &DaemonHealth, snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[UPDATES]".cyan());

    let state = UpdateState::load();
    let daemon_running = daemon_health.is_running();

    // Mode and channel
    println!("  Mode:       {} ({})", state.format_mode(), state.format_channel());

    // Interval
    println!("  Interval:   {}", state.format_interval());

    // Show update progress if in progress
    if state.is_update_in_progress() {
        if let Some(ref phase) = state.update_phase {
            let progress_str = if let Some(percent) = state.update_progress_percent {
                if let Some(eta) = state.update_eta_seconds {
                    format!("{} ({}%, ETA: {}s)", phase, percent, eta)
                } else {
                    format!("{} ({}%)", phase, percent)
                }
            } else {
                phase.clone()
            };
            println!("  Progress:   {}", progress_str.yellow());
        }
        if let Some(ref target) = state.updating_to_version {
            println!("  Updating:   {} -> {}", state.last_checked_version_installed, target.green());
        }
        println!();
        return;
    }

    // Last check
    let last_check_str = if state.last_check_at == 0 {
        if state.mode != UpdateMode::Auto {
            "n/a (auto-updates disabled)".dimmed().to_string()
        } else if daemon_running {
            "pending (first check soon)".to_string()
        } else {
            "pending (start daemon)".dimmed().to_string()
        }
    } else {
        state.format_last_check()
    };
    println!("  Last check: {}", last_check_str);

    // Result
    if state.last_check_at > 0 || state.last_result != UpdateResult::Pending {
        println!("  Result:     {}", state.format_last_result());
    }

    // Next check
    if state.mode == UpdateMode::Auto {
        let next_str = if !daemon_running {
            "paused (daemon not running)".dimmed().to_string()
        } else if let Some(s) = snapshot {
            if let Some(next) = s.update_next_check {
                let delta = (next - chrono::Utc::now()).num_seconds();
                if delta > 0 {
                    format!("in {}m {}s", delta / 60, delta % 60)
                } else {
                    "now".to_string()
                }
            } else if state.next_check_at > 0 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let delta = state.next_check_at.saturating_sub(now);
                if delta > 0 {
                    format!("in {}m {}s", delta / 60, delta % 60)
                } else {
                    "now".to_string()
                }
            } else {
                "soon".to_string()
            }
        } else {
            "soon".to_string()
        };
        println!("  Next check: {}", next_str);
    }

    // Available version
    if let Some(ref ver) = state.last_checked_version_available {
        if !state.last_checked_version_installed.is_empty() {
            println!("  Available:  {} -> {}",
                state.last_checked_version_installed,
                ver.green());
        }
    }

    // v0.0.11: Show last update info if recently updated
    if let Some(last_update) = state.last_update_at {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ago = now.saturating_sub(last_update);
        // Show if updated in last 24 hours
        if ago < 86400 {
            let ago_str = if ago < 60 {
                format!("{}s ago", ago)
            } else if ago < 3600 {
                format!("{}m ago", ago / 60)
            } else {
                format!("{}h ago", ago / 3600)
            };
            if let Some(ref prev) = state.previous_version {
                println!("  Last update: {} -> {} ({})",
                    prev, state.last_checked_version_installed.green(), ago_str.dimmed());
            }
        }
    }

    // v0.0.11: Show rollback info if rolled back
    if state.last_result == UpdateResult::RolledBack {
        if let Some(ref reason) = state.last_error {
            println!("  Rollback:   {} ({})", "active".yellow(), reason.dimmed());
        }
    }

    println!();
}

/// [ALERTS] section - v0.0.12: enhanced with actual alerts from alert queue
fn print_alerts_section(snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[ALERTS]".cyan());

    // Load alert queue directly
    let queue = AlertQueue::load();
    let (critical, warning, info) = queue.count_by_severity();
    let _total = critical + warning + info;

    // Show summary counts
    println!("  Critical:   {}", if critical > 0 {
        critical.to_string().red().to_string()
    } else {
        "0".green().to_string()
    });
    println!("  Warnings:   {}", if warning > 0 {
        warning.to_string().yellow().to_string()
    } else {
        "0".green().to_string()
    });
    println!("  Info:       {}", if info > 0 {
        info.to_string().dimmed().to_string()
    } else {
        "0".dimmed().to_string()
    });

    // Show latest 3 alerts with evidence IDs
    let active = queue.get_active();
    if !active.is_empty() {
        println!();
        println!("  Latest alerts:");
        for anomaly in active.iter().take(3) {
            let severity_badge = match anomaly.severity {
                anna_common::AnomalySeverity::Critical => format!("[{}]", "CRITICAL".red()),
                anna_common::AnomalySeverity::Warning => format!("[{}]", "WARNING".yellow()),
                anna_common::AnomalySeverity::Info => format!("[{}]", "INFO".dimmed()),
            };
            println!("    {} [{}] {}", severity_badge, anomaly.evidence_id.cyan(), anomaly.title);
        }
        if active.len() > 3 {
            println!("    {} more alert(s)...", (active.len() - 3).to_string().dimmed());
        }
    }

    // Show last check time
    if let Some(ref last_check) = queue.last_check {
        let ago = (chrono::Utc::now() - *last_check).num_seconds().max(0) as u64;
        let ago_str = if ago < 60 {
            format!("{}s ago", ago)
        } else if ago < 3600 {
            format!("{}m ago", ago / 60)
        } else {
            format!("{}h ago", ago / 3600)
        };
        println!();
        println!("  Last scan:  {}", ago_str.dimmed());
    }

    // Show instrumentation count from snapshot
    if let Some(s) = snapshot {
        if !s.is_stale() && s.instrumentation_count > 0 {
            println!("  Installed:  {} tool(s) by Anna", s.instrumentation_count);
        }
    }

    println!();
}

/// [CASES] section - v0.0.33: show recent case files and last failure
fn print_cases_section(mode: &DisplayMode) {
    println!("{}", "[CASES]".cyan());

    let config = AnnaConfig::load();

    // Show dev mode indicator
    if config.ui.is_dev_mode() {
        println!("  Dev mode:   {} (max verbosity)", "active".green());
    }

    // Get recent case paths and load summaries
    let recent_paths = list_recent_cases(5);
    let recent_summaries: Vec<_> = recent_paths.iter()
        .filter_map(|p| load_case_summary(p))
        .collect();

    let last_failure_path = find_last_failure();
    let last_failure = last_failure_path.as_ref().and_then(|p| load_case_summary(p));
    let storage_bytes = get_cases_storage_size();

    if recent_summaries.is_empty() {
        println!("  Cases:      {} (none yet)", "0".dimmed());
        println!();
        return;
    }

    // Summary
    let success_count = recent_summaries.iter().filter(|c| c.outcome == CaseOutcome::Success).count();
    let failure_count = recent_summaries.iter().filter(|c| c.outcome == CaseOutcome::Failure).count();
    println!("  Recent:     {} ({} success, {} failure)",
        recent_summaries.len(),
        if success_count > 0 { success_count.to_string().green().to_string() } else { "0".to_string() },
        if failure_count > 0 { failure_count.to_string().red().to_string() } else { "0".to_string() });

    // Storage
    println!("  Storage:    {}", format_bytes(storage_bytes));

    // Last failure if any
    if let Some(ref failure) = last_failure {
        let outcome_str = match failure.outcome {
            CaseOutcome::Failure => "failure".red().to_string(),
            CaseOutcome::Partial => "partial".yellow().to_string(),
            _ => "unknown".dimmed().to_string(),
        };
        println!("  Last fail:  {} - {} ({})",
            failure.request_id.cyan(),
            truncate_str(&failure.user_request, 30),
            outcome_str);
    }

    // In compact mode, skip case list
    if *mode == DisplayMode::Compact {
        println!();
        return;
    }

    // Show last 5 cases
    println!();
    println!("  Latest:");
    for case in recent_summaries.iter().take(5) {
        let outcome_badge = match case.outcome {
            CaseOutcome::Success => "[ok]".green().to_string(),
            CaseOutcome::Failure => "[FAIL]".red().to_string(),
            CaseOutcome::Partial => "[partial]".yellow().to_string(),
            CaseOutcome::Cancelled => "[cancelled]".dimmed().to_string(),
        };
        // Format timestamp
        let time_str = case.timestamp.format("%H:%M:%S").to_string();
        println!("    {} {} {} {}",
            time_str.dimmed(),
            outcome_badge,
            case.request_id.cyan(),
            truncate_str(&case.user_request, 35));
    }

    println!();
}

/// [PATHS] section - v7.42.3: simplified, just show existence (daemon writes, not CLI)
fn print_paths_section() {
    println!("{}", "[PATHS]".cyan());

    // Config path
    let config_path = "/etc/anna/config.toml";
    if Path::new(config_path).exists() {
        println!("  Config:     {}", config_path);
    } else {
        println!("  Config:     {} {}", config_path, "(missing)".yellow());
    }

    // Data path
    let data_dir = "/var/lib/anna";
    if Path::new(data_dir).exists() {
        println!("  Data:       {}", data_dir);
    } else {
        println!("  Data:       {} {}", data_dir, "(missing)".yellow());
    }

    // Internal dir
    if Path::new(INTERNAL_DIR).exists() {
        println!("  Internal:   {}", INTERNAL_DIR);
    } else {
        println!("  Internal:   {} {}", INTERNAL_DIR, "(missing)".yellow());
    }

    // Snapshots dir
    if Path::new(SNAPSHOTS_DIR).exists() {
        println!("  Snapshots:  {}", SNAPSHOTS_DIR);
    } else {
        println!("  Snapshots:  {} {}", SNAPSHOTS_DIR, "(missing)".yellow());
    }

    // Socket path
    if Path::new(SOCKET_PATH).exists() {
        println!("  Socket:     {}", SOCKET_PATH);
    } else {
        println!("  Socket:     {} {}", SOCKET_PATH, "(daemon will create)".dimmed());
    }

    // Logs hint
    println!("  Logs:       {}", "journalctl -u annad".dimmed());

    println!();
}

/// [HELPERS] section - v0.0.9: show helpers with provenance
fn print_helpers_section(mode: &DisplayMode) {
    println!("{}", "[HELPERS]".cyan());

    let helpers = get_helper_status_list();

    if helpers.is_empty() {
        println!("  {}", "(no helpers tracked)".dimmed());
        println!();
        return;
    }

    // Count stats
    let present_count = helpers.iter().filter(|h| h.present).count();
    let missing_count = helpers.iter().filter(|h| !h.present).count();
    let anna_installed = helpers.iter().filter(|h| h.installed_by == InstalledBy::Anna).count();

    println!("  Summary:    {} present, {} missing ({} by Anna)",
        present_count.to_string().green(),
        if missing_count > 0 { missing_count.to_string().yellow().to_string() } else { "0".to_string() },
        anna_installed);

    // In compact mode, just show summary
    if *mode == DisplayMode::Compact {
        println!();
        return;
    }

    println!();

    // Show each helper - v0.0.27: Anna installs automatically when needed
    for helper in &helpers {
        let (presence, action) = if helper.present {
            let by = match helper.installed_by {
                InstalledBy::Anna => "by Anna".cyan().to_string(),
                InstalledBy::User => "by user".dimmed().to_string(),
                InstalledBy::Unknown => "by user".dimmed().to_string(),
            };
            ("present".green().to_string(), by)
        } else {
            // v0.0.30: Missing helper - Anna installs on daemon restart
            ("missing".yellow().to_string(), "restart daemon to install".dimmed().to_string())
        };

        println!("  {} ({}, {})", helper.name, presence, action);
    }

    println!();
}

/// [LEARNING] section - v0.0.13: recipes, memory, learning status
fn print_learning_section(mode: &DisplayMode) {
    println!("{}", "[LEARNING]".cyan());

    let config = AnnaConfig::load();

    // Check if memory is enabled
    if !config.memory.enabled {
        println!("  {}", "Memory disabled in config.".dimmed());
        println!();
        return;
    }

    // Get recipe stats
    let recipe_stats = RecipeManager::get_stats();
    let memory_stats = MemoryManager::default().get_stats();

    // Recipes count
    println!("  Recipes:    {}", if recipe_stats.total_recipes > 0 {
        format!("{} ({} total uses)",
            recipe_stats.total_recipes.to_string().green(),
            recipe_stats.total_uses)
    } else {
        "0 (none yet)".dimmed().to_string()
    });

    // Last learned time
    if let Some(last) = recipe_stats.last_created_at {
        let ago = (chrono::Utc::now() - last).num_seconds().max(0) as u64;
        let ago_str = if ago < 60 {
            format!("{}s ago", ago)
        } else if ago < 3600 {
            format!("{}m ago", ago / 60)
        } else if ago < 86400 {
            format!("{}h ago", ago / 3600)
        } else {
            format!("{}d ago", ago / 86400)
        };
        println!("  Last learned: {}", ago_str);
    } else {
        println!("  Last learned: {}", "-".dimmed());
    }

    // Sessions count
    println!("  Sessions:   {}", memory_stats.total_sessions);

    // In compact mode, skip top recipes
    if *mode == DisplayMode::Compact {
        println!();
        return;
    }

    // Top 3 recipes
    let top_recipes = RecipeManager::get_top(3);
    if !top_recipes.is_empty() {
        println!();
        println!("  Top recipes:");
        for recipe in &top_recipes {
            println!("    [{}] {} ({} uses, {:.0}%)",
                recipe.id.cyan(),
                truncate_str(&recipe.name, 30),
                recipe.success_count,
                recipe.confidence * 100.0);
        }
    }

    // Memory settings (in standard/wide mode)
    println!();
    println!("  Settings:");
    println!("    Store raw:      {}", if config.memory.store_raw { "yes" } else { "no (summaries only)" });
    println!("    Max sessions:   {}", config.memory.max_sessions);
    println!("    Min reliability: {}%", config.memory.min_reliability_for_recipe);

    println!();
}

/// Truncate string for display
fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// [KNOWLEDGE] section - v0.0.19: show knowledge pack stats
fn print_knowledge_section(mode: &DisplayMode) {
    println!("{}", "[KNOWLEDGE]".cyan());

    // Check if knowledge packs directory exists
    let knowledge_dir = std::path::Path::new(KNOWLEDGE_PACKS_DIR);
    if !knowledge_dir.exists() {
        println!("  Status:     {} (not initialized)", "empty".dimmed());
        println!("  Path:       {}", KNOWLEDGE_PACKS_DIR);
        println!();
        return;
    }

    // Try to open index and get stats
    match KnowledgeIndex::open() {
        Ok(index) => {
            match index.get_stats() {
                Ok(stats) => {
                    // Format index size
                    let size_str = format_bytes(stats.total_size_bytes);

                    // Format last indexed time
                    let last_indexed_str = match stats.last_indexed_at {
                        Some(t) if t > 0 => {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let ago = now.saturating_sub(t);
                            if ago < 60 {
                                format!("{}s ago", ago)
                            } else if ago < 3600 {
                                format!("{}m ago", ago / 60)
                            } else if ago < 86400 {
                                format!("{}h ago", ago / 3600)
                            } else {
                                format!("{}d ago", ago / 86400)
                            }
                        }
                        _ => "never".to_string(),
                    };

                    // Show summary
                    println!("  Packs:      {}", if stats.pack_count > 0 {
                        stats.pack_count.to_string().green().to_string()
                    } else {
                        "0 (none)".dimmed().to_string()
                    });
                    println!("  Documents:  {}", stats.document_count);
                    println!("  Index size: {}", size_str);
                    println!("  Last index: {}", last_indexed_str);

                    // In compact mode, skip breakdown
                    if *mode == DisplayMode::Compact {
                        println!();
                        return;
                    }

                    // Show packs by source type
                    if !stats.packs_by_source.is_empty() {
                        println!();
                        println!("  By source:");
                        for (source, count) in &stats.packs_by_source {
                            println!("    {}: {}", source, count);
                        }
                    }

                    // Show top packs if available
                    if !stats.top_packs.is_empty() {
                        println!();
                        println!("  Top packs:");
                        for (pack_name, query_count) in stats.top_packs.iter().take(3) {
                            println!("    {} ({} queries)", pack_name, query_count);
                        }
                    }
                }
                Err(e) => {
                    println!("  Status:     {} ({})", "error".red(), e);
                }
            }
        }
        Err(_) => {
            println!("  Status:     {} (index not created)", "empty".dimmed());
            println!("  Path:       {}", KNOWLEDGE_PACKS_DIR);
        }
    }

    println!();
}

/// [Q&A TODAY] section - v0.0.20: show Q&A statistics
fn print_qa_section() {
    println!("{}", "[Q&A TODAY]".cyan());

    let stats = QaStats::load_today();

    if stats.answers_count == 0 {
        println!("  Answers:    {} (none today)", "0".dimmed());
        println!();
        return;
    }

    // Show answer count
    println!("  Answers:    {}", stats.answers_count.to_string().green());

    // Show average reliability
    let avg = stats.avg_reliability();
    let reliability_color = if avg >= 80 {
        format!("{}%", avg).green().to_string()
    } else if avg >= 60 {
        format!("{}%", avg).yellow().to_string()
    } else {
        format!("{}%", avg).red().to_string()
    };
    println!("  Avg reliability: {}", reliability_color);

    // Show citation counts
    println!("  Citations:  K:{} E:{} R:{}",
        stats.knowledge_citations,
        stats.evidence_citations,
        stats.reasoning_labels);

    // Show top source types
    let top_sources = stats.top_source_types(3);
    if !top_sources.is_empty() {
        let sources_str: Vec<String> = top_sources.iter()
            .map(|(name, count)| format!("{}: {}", name, count))
            .collect();
        println!("  Top sources: {}", sources_str.join(", "));
    }

    println!();
}

/// [PERFORMANCE] section - v0.0.21: show latency and cache stats
fn print_performance_section(mode: &DisplayMode) {
    println!("{}", "[PERFORMANCE]".cyan());

    let stats = PerfStats::load();
    let tool_cache = ToolCache::new();
    let llm_cache = LlmCache::new();

    // Sample count and average latencies (24h)
    let sample_count = stats.sample_count();
    if sample_count == 0 {
        println!("  Samples:    {} (no data today)", "0".dimmed());
        println!();
        return;
    }

    println!("  Samples:    {} (last 24h)", sample_count);

    // Average latencies
    println!("  Avg total:  {}ms", stats.avg_total_latency_ms());
    println!("  Translator: {}ms avg", stats.avg_translator_latency_ms());
    println!("  Junior:     {}ms avg", stats.avg_junior_latency_ms());

    // Cache hit rate
    let hit_rate = (stats.cache_hit_rate() * 100.0) as u32;
    let hit_rate_str = if hit_rate >= 50 {
        format!("{}%", hit_rate).green().to_string()
    } else if hit_rate > 0 {
        format!("{}%", hit_rate).yellow().to_string()
    } else {
        format!("{}%", hit_rate).dimmed().to_string()
    };
    println!("  Cache hit:  {} ({} hits, {} misses)",
        hit_rate_str, stats.cache_hits, stats.cache_misses);

    // In compact mode, skip details
    if *mode == DisplayMode::Compact {
        println!();
        return;
    }

    // Top 5 cached tools
    let top_tools = stats.top_cached_tools(5);
    if !top_tools.is_empty() {
        println!();
        println!("  Top cached tools:");
        for (tool, count) in top_tools {
            println!("    {} ({} hits)", tool, count);
        }
    }

    // Cache storage stats
    let tool_stats = tool_cache.stats();
    let llm_stats = llm_cache.stats();
    if tool_stats.total_entries > 0 || llm_stats.translator_entries > 0 || llm_stats.junior_entries > 0 {
        println!();
        println!("  Cache storage:");
        if tool_stats.total_entries > 0 {
            println!("    Tool cache:  {} entries ({})",
                tool_stats.total_entries, format_bytes(tool_stats.total_size_bytes));
        }
        if llm_stats.translator_entries > 0 || llm_stats.junior_entries > 0 {
            println!("    LLM cache:   {} translator, {} junior ({})",
                llm_stats.translator_entries, llm_stats.junior_entries,
                format_bytes(llm_stats.total_size_bytes));
        }
    }

    // Budget violations
    if !stats.budget_violations.is_empty() {
        println!();
        println!("  Budget violations: {}", stats.budget_violations.len().to_string().yellow());
    }

    println!();
}

/// [RELIABILITY] section - v0.0.22: show error budgets and metrics
fn print_reliability_section(mode: &DisplayMode) {
    println!("{}", "[RELIABILITY]".cyan());

    let metrics = MetricsStore::load();
    let budgets = ErrorBudgets::default();

    // Get today's date
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_metrics = match metrics.for_date(&today) {
        Some(m) => m.clone(),
        None => {
            println!("  Status:     {} (no data today)", "healthy".green());
            println!();
            return;
        }
    };

    // Calculate budget status
    let statuses = calculate_budget_status(&today_metrics, &budgets);

    if statuses.is_empty() {
        println!("  Status:     {} (no events)", "healthy".green());
        println!();
        return;
    }

    // Check for any issues
    let critical_count = statuses.iter()
        .filter(|s| s.status == BudgetState::Critical || s.status == BudgetState::Exhausted)
        .count();
    let warning_count = statuses.iter()
        .filter(|s| s.status == BudgetState::Warning)
        .count();

    if critical_count > 0 {
        println!("  Status:     {} ({} budget(s) critical)", "CRITICAL".red().bold(), critical_count);
    } else if warning_count > 0 {
        println!("  Status:     {} ({} budget(s) warning)", "warning".yellow(), warning_count);
    } else {
        println!("  Status:     {} (all budgets healthy)", "healthy".green());
    }

    // Show each budget status
    println!();
    for status in &statuses {
        let burn_str = format!("{:.1}%/{:.1}%", status.current_percent, status.budget_percent);
        let status_str = match status.status {
            BudgetState::Ok => burn_str.green().to_string(),
            BudgetState::Warning => burn_str.yellow().to_string(),
            BudgetState::Critical => burn_str.red().to_string(),
            BudgetState::Exhausted => format!("{} EXCEEDED", burn_str).red().bold().to_string(),
        };
        println!("  {}: {} ({}/{})",
            status.category,
            status_str,
            status.failed_events,
            status.total_events);
    }

    // In compact mode, skip extra details
    if *mode == DisplayMode::Compact {
        println!();
        return;
    }

    // Show success rates
    let request_success = today_metrics.get_count(anna_common::MetricType::RequestSuccess);
    let request_failure = today_metrics.get_count(anna_common::MetricType::RequestFailure);
    let request_total = request_success + request_failure;
    if request_total > 0 {
        let rate = (request_success as f64 / request_total as f64) * 100.0;
        println!();
        println!("  Requests:   {}/{} ({:.1}% success)",
            request_success, request_total, rate);
    }

    // Show latency percentiles if available
    if let Some(p50) = today_metrics.percentile_latency("e2e", 50.0) {
        if let Some(p95) = today_metrics.percentile_latency("e2e", 95.0) {
            println!("  Latency:    p50={}ms, p95={}ms", p50, p95);
        }
    }

    println!();
}

/// [POLICY] section - v0.0.15: show policy status, version, violations
fn print_policy_section() {
    println!("{}", "[POLICY]".cyan());

    let policy = get_policy();

    // Check if policy files exist
    let policy_dir = Path::new(POLICY_DIR);
    if !policy_dir.exists() {
        println!("  Status:     {} (policy dir missing)", "not loaded".yellow());
        println!("  Path:       {}", POLICY_DIR);
        println!("  Run:        {} to create defaults", "annactl reset".dimmed());
        println!();
        return;
    }

    // Show policy version
    println!("  Status:     {}", "loaded".green());
    println!("  Schema:     v{}", policy.capabilities.schema_version);

    // Show capabilities summary
    let caps = &policy.capabilities;
    let read_only_str = if caps.read_only_tools.enabled { "enabled".green().to_string() } else { "disabled".yellow().to_string() };
    let mutation_str = if caps.mutation_tools.enabled { "enabled".green().to_string() } else { "disabled".yellow().to_string() };
    println!("  Read-only:  {}", read_only_str);
    println!("  Mutations:  {}", mutation_str);

    // Show blocked counts - v0.0.27: clarify what "blocked" means
    let blocked = &policy.blocked;
    let blocked_packages = blocked.packages.exact.len() + blocked.packages.patterns.len();
    let blocked_services = blocked.services.exact.len() + blocked.services.patterns.len();
    let blocked_paths = blocked.paths.exact.len() + blocked.paths.prefixes.len() + blocked.paths.patterns.len();
    let blocked_commands = blocked.commands.exact.len() + blocked.commands.patterns.len();
    let total_blocked = blocked_packages + blocked_services + blocked_paths + blocked_commands;
    if total_blocked > 0 {
        println!("  Protected:  {} paths Anna won't modify (safety policy)",
            blocked_paths);
    } else {
        println!("  Protected:  default safety rules");
    }

    // Show last modified time for policy files
    if let Ok(metadata) = std::fs::metadata(format!("{}/capabilities.toml", POLICY_DIR)) {
        if let Ok(modified) = metadata.modified() {
            let epoch = modified.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            println!("  Modified:   {}", format_timestamp(epoch).dimmed());
        }
    }

    println!();
}

/// [MODELS] section - v0.0.35: Enhanced model status with readiness UX
fn print_models_section() {
    println!("{}", "[MODELS]".cyan());

    let config = AnnaConfig::load();
    let bootstrap_state = BootstrapState::load();

    // LLM enabled status
    if !config.llm.enabled {
        println!("  Status:     {}", "disabled".yellow());
        println!();
        return;
    }

    // Ollama status based on bootstrap phase
    let (ollama_status, is_ready) = match bootstrap_state.phase {
        BootstrapPhase::Ready => ("available".green().to_string(), true),
        BootstrapPhase::DetectingOllama => ("checking...".yellow().to_string(), false),
        BootstrapPhase::InstallingOllama => ("installing...".yellow().to_string(), false),
        BootstrapPhase::PullingModels => ("pulling models...".yellow().to_string(), false),
        BootstrapPhase::Benchmarking => ("benchmarking...".yellow().to_string(), false),
        BootstrapPhase::Error => {
            let msg = if let Some(ref err) = bootstrap_state.error {
                if err.contains("not available") {
                    "unavailable".red().to_string()
                } else {
                    format!("error: {}", truncate_str(err, 40)).red().to_string()
                }
            } else {
                "error".red().to_string()
            };
            (msg, false)
        }
    };
    println!("  Ollama:     {}", ollama_status);

    // v0.0.35: Hardware tier for model selection context
    if let Some(ref hw) = bootstrap_state.hardware {
        println!("  Hardware:   {} tier ({} RAM, {} cores)",
            hw.tier.to_string().cyan(),
            anna_common::model_selection::HardwareProfile::format_memory(hw.total_ram_bytes),
            hw.cpu_cores);
    }

    // Translator model from bootstrap state
    let translator_status = if let Some(ref sel) = bootstrap_state.translator {
        format!("{} ({})", sel.model.green(), "ready".green())
    } else if bootstrap_state.phase == BootstrapPhase::PullingModels {
        // Show download progress if available
        if let Some(ref progress) = bootstrap_state.download_progress {
            if progress.role == "translator" {
                format!("downloading {} ({:.1}%)",
                    progress.model.yellow(),
                    (progress.downloaded_bytes as f64 / progress.total_bytes.max(1) as f64) * 100.0)
            } else {
                "(waiting)".yellow().to_string()
            }
        } else {
            "(downloading)".yellow().to_string()
        }
    } else if !config.llm.translator.model.is_empty() {
        format!("{} ({})", config.llm.translator.model, "configured".dimmed())
    } else {
        format!("{} ({})", "(auto-select)".dimmed(), "fallback: deterministic".yellow())
    };
    println!("  Translator: {}", translator_status);

    // Junior model from bootstrap state - v0.0.35: show fallback mode
    let junior_status = if let Some(ref sel) = bootstrap_state.junior {
        format!("{} ({})", sel.model.green(), "ready".green())
    } else if bootstrap_state.phase == BootstrapPhase::PullingModels {
        if let Some(ref progress) = bootstrap_state.download_progress {
            if progress.role == "junior" {
                format!("downloading {} ({:.1}%)",
                    progress.model.yellow(),
                    (progress.downloaded_bytes as f64 / progress.total_bytes.max(1) as f64) * 100.0)
            } else {
                "(waiting)".yellow().to_string()
            }
        } else {
            "(downloading)".yellow().to_string()
        }
    } else {
        // v0.0.35: Show no-Junior fallback mode
        format!("{} ({})", "(unavailable)".red(), "reliability capped at 60%".yellow())
    };
    println!("  Junior:     {}", junior_status);

    // v0.0.35: Readiness summary
    let translator_ready = bootstrap_state.translator.is_some();
    let junior_ready = bootstrap_state.junior.is_some();

    if !is_ready {
        println!("  Readiness:  {}", "models still downloading - limited functionality".yellow());
    } else if translator_ready && junior_ready {
        println!("  Readiness:  {}", "full capability".green());
    } else if translator_ready && !junior_ready {
        println!("  Readiness:  {} (no verification)", "partial".yellow());
    } else {
        println!("  Readiness:  {}", "not ready".red());
    }

    // Last bootstrap/benchmark time
    if bootstrap_state.last_update > 0 {
        let ago = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(bootstrap_state.last_update);
        let ago_str = if ago < 60 {
            format!("{}s ago", ago)
        } else if ago < 3600 {
            format!("{}m ago", ago / 60)
        } else if ago < 86400 {
            format!("{}h ago", ago / 3600)
        } else {
            format!("{}d ago", ago / 86400)
        };
        println!("  Last check: {}", ago_str.dimmed());
    }

    println!();
}

/// [RECENT ACTIONS] section - v0.0.15: show recent tool executions and mutations
fn print_recent_actions_section(mode: &DisplayMode) {
    println!("{}", "[RECENT ACTIONS]".cyan());

    // Load recent audit entries
    let recent = AuditLogger::get_recent(10);

    if recent.is_empty() {
        println!("  {}", "(no recent actions)".dimmed());
        println!();
        return;
    }

    // Count by type
    let read_only_count = recent.iter()
        .filter(|e| e.entry_type == anna_common::audit_log::AuditEntryType::ReadOnlyTool)
        .count();
    let mutation_count = recent.iter()
        .filter(|e| e.entry_type == anna_common::audit_log::AuditEntryType::MutationTool)
        .count();
    let blocked_count = recent.iter()
        .filter(|e| e.entry_type == anna_common::audit_log::AuditEntryType::ActionBlocked)
        .count();

    println!("  Summary:    {} read-only, {} mutations, {} blocked",
        read_only_count,
        if mutation_count > 0 { mutation_count.to_string().yellow().to_string() } else { "0".to_string() },
        if blocked_count > 0 { blocked_count.to_string().red().to_string() } else { "0".to_string() });

    // In compact mode, skip details
    if *mode == DisplayMode::Compact {
        println!();
        return;
    }

    // Show latest 5 entries
    println!();
    println!("  Latest:");
    for entry in recent.iter().take(5) {
        let timestamp = entry.timestamp.format("%H:%M:%S").to_string();
        let type_str = match entry.entry_type {
            anna_common::audit_log::AuditEntryType::ReadOnlyTool => "read".dimmed().to_string(),
            anna_common::audit_log::AuditEntryType::MutationTool => "mutation".yellow().to_string(),
            anna_common::audit_log::AuditEntryType::ActionBlocked => "blocked".red().to_string(),
            anna_common::audit_log::AuditEntryType::PolicyCheck => "policy".cyan().to_string(),
            anna_common::audit_log::AuditEntryType::Confirmation => "confirm".green().to_string(),
            anna_common::audit_log::AuditEntryType::Rollback => "rollback".magenta().to_string(),
            _ => "other".dimmed().to_string(),
        };

        let tool_name = entry.tool_name.as_deref().unwrap_or("-");
        let evidence = entry.evidence_id.as_deref().map(|e| format!("[{}]", e.cyan())).unwrap_or_default();
        let result = match entry.result {
            anna_common::audit_log::AuditResult::Success => "ok".green().to_string(),
            anna_common::audit_log::AuditResult::Failure => "fail".red().to_string(),
            anna_common::audit_log::AuditResult::Blocked => "blocked".red().to_string(),
            anna_common::audit_log::AuditResult::Pending => "pending".yellow().to_string(),
        };

        println!("    {} {} {} {} {}",
            timestamp.dimmed(),
            type_str,
            tool_name,
            evidence,
            result);
    }

    println!();
}

/// [STORAGE] section - v0.0.15: show disk usage for Anna directories
fn print_storage_section(mode: &DisplayMode) {
    println!("{}", "[STORAGE]".cyan());

    // Calculate directory sizes
    let data_dir = "/var/lib/anna";
    let audit_dir = "/var/lib/anna/audit";
    let rollback_dir = "/var/lib/anna/rollback";
    let memory_dir = "/var/lib/anna/memory";
    let recipes_dir = "/var/lib/anna/recipes";

    let data_size = get_dir_size(data_dir);
    let audit_size = get_dir_size(audit_dir);
    let rollback_size = get_dir_size(rollback_dir);
    let memory_size = get_dir_size(memory_dir);
    let recipes_size = get_dir_size(recipes_dir);

    // Total
    println!("  Total:      {}", format_bytes(data_size));

    // In compact mode, skip breakdown
    if *mode == DisplayMode::Compact {
        println!();
        return;
    }

    // Breakdown
    println!("  Audit:      {}", format_bytes(audit_size));
    println!("  Rollback:   {}", format_bytes(rollback_size));
    println!("  Memory:     {}", format_bytes(memory_size));
    println!("  Recipes:    {}", format_bytes(recipes_size));

    // Show retention settings
    let config = AnnaConfig::load();
    println!("  Retention:  {} days telemetry", config.telemetry.retention_days);

    println!();
}

/// Calculate total size of a directory recursively
fn get_dir_size(path: &str) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += get_dir_size(&entry.path().to_string_lossy());
                }
            }
        }
    }
    total
}
