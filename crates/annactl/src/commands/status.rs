//! Status Command v7.42.0 - Daemon/CLI Contract Fix
//!
//! v7.42.0: Separate daemon state from snapshot state
//! - DAEMON: running/stopped via control socket or systemd (NOT snapshot)
//! - SNAPSHOT: available/stale/missing (file status)
//! - Never conflate "no snapshot" with "daemon stopped"
//!
//! v7.40.0: Improved update scheduler display
//! - Clearer messaging when daemon not running
//! - Shows actual state from update_state.json
//!
//! v7.39.0: Terminal-adaptive rendering, domain status, "checking..." indicator
//!
//! v7.38.0: Cache-only status, NO live probing
//! - Reads status_snapshot.json ONLY (written by daemon every 60s)
//! - Shows last_crash.json when daemon is down
//! - Fast: < 10ms to display

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use anna_common::config::{AnnaConfig, UpdateState, UpdateResult, UpdateMode};
use anna_common::format_duration_secs;
use anna_common::daemon_state::{
    StatusSnapshot, LastCrash, SnapshotStatus,
    INTERNAL_DIR, SNAPSHOTS_DIR, STATUS_SNAPSHOT_PATH,
};
use anna_common::domain_state::{DomainSummary, RefreshRequest, REQUESTS_DIR};
use anna_common::terminal::{DisplayMode, get_terminal_size};
use anna_common::self_observation::SelfObservation;
use anna_common::control_socket::{check_daemon_health, DaemonHealth, SOCKET_PATH};

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

    // [TELEMETRY] - condensed in compact mode
    if mode != DisplayMode::Compact {
        print_telemetry_section(&snapshot);
    }

    // [UPDATES] - always shown
    print_updates_section(&daemon_health, &snapshot);

    // [ALERTS] - always shown
    print_alerts_section(&snapshot);

    // [PATHS] - only in standard/wide mode
    if mode != DisplayMode::Compact {
        print_paths_section();
    }

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

/// [UPDATES] section - update scheduler state
fn print_updates_section(daemon_health: &DaemonHealth, snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[UPDATES]".cyan());

    let state = UpdateState::load();
    let daemon_running = daemon_health.is_running();

    // Mode
    println!("  Mode:       {}", state.format_mode());

    // Interval
    println!("  Interval:   {}", state.format_interval());

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

    println!();
}

/// [ALERTS] section - alert counts from snapshot
fn print_alerts_section(snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[ALERTS]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            println!("  Critical:   {}", if s.alerts_critical > 0 {
                s.alerts_critical.to_string().red().to_string()
            } else {
                "0".green().to_string()
            });
            println!("  Warnings:   {}", if s.alerts_warning > 0 {
                s.alerts_warning.to_string().yellow().to_string()
            } else {
                "0".green().to_string()
            });

            if s.instrumentation_count > 0 {
                println!("  Installed:  {} tool(s) by Anna", s.instrumentation_count);
            }
        }
        _ => {
            println!("  {}", "(no snapshot data)".dimmed());
        }
    }

    println!();
}

/// [PATHS] section - static paths only, no probing
fn print_paths_section() {
    println!("{}", "[PATHS]".cyan());

    // Config path
    let config_path = "/etc/anna/config.toml";
    let config_exists = Path::new(config_path).exists();
    if config_exists {
        println!("  Config:     {}", config_path);
    } else {
        println!("  Config:     {} {}", config_path, "(missing)".yellow());
    }

    // Data path
    let data_dir = "/var/lib/anna";
    let data_exists = Path::new(data_dir).exists();
    if data_exists {
        println!("  Data:       {}", data_dir);
    } else {
        println!("  Data:       {} {}", data_dir, "(missing)".yellow());
    }

    // Internal dir
    let internal_exists = Path::new(INTERNAL_DIR).exists();
    if internal_exists {
        println!("  Internal:   {}", INTERNAL_DIR);
    } else {
        println!("  Internal:   {} {}", INTERNAL_DIR, "(will create on daemon start)".dimmed());
    }

    // Snapshots dir
    let snapshots_exists = Path::new(SNAPSHOTS_DIR).exists();
    if snapshots_exists {
        println!("  Snapshots:  {}", SNAPSHOTS_DIR);
    } else {
        println!("  Snapshots:  {} {}", SNAPSHOTS_DIR, "(missing)".yellow());
    }

    // Socket path
    let socket_exists = Path::new(SOCKET_PATH).exists();
    if socket_exists {
        println!("  Socket:     {}", SOCKET_PATH);
    } else {
        println!("  Socket:     {} {}", SOCKET_PATH, "(daemon may create)".dimmed());
    }

    // Logs hint
    println!("  Logs:       {}", "journalctl -u annad".dimmed());

    println!();
}
