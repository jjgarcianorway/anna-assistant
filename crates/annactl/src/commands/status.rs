//! Status Command v7.38.0 - Cache-Only Status (No Live Probing)
//!
//! v7.38.0: Cache-only status, NO live probing
//! - Reads status_snapshot.json ONLY (written by daemon every 60s)
//! - Shows last_crash.json when daemon is down
//! - NO pacman -Q, systemctl list-*, journalctl, filesystem crawling
//! - NO hardware probing, NO network calls
//! - Fast: < 10ms to display
//!
//! Sections:
//! - [VERSION]             Anna version
//! - [DAEMON]              State from snapshot (or last crash if down)
//! - [HEALTH]              Health from snapshot
//! - [DATA]                Knowledge object counts
//! - [TELEMETRY]           Sample counts
//! - [UPDATES]             Update scheduler state
//! - [ALERTS]              Alert counts from snapshot
//! - [PATHS]               Static paths only

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use anna_common::config::{AnnaConfig, UpdateState, SYSTEM_CONFIG_DIR, DATA_DIR, UpdateMode};
use anna_common::format_duration_secs;
use anna_common::daemon_state::{StatusSnapshot, LastCrash, INTERNAL_DIR};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the status command - v7.38.0: cache-only, no live probing
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", THIN_SEP);
    println!();

    // v7.38.0: Load snapshot (written by daemon every 60s)
    let snapshot = StatusSnapshot::load();
    let last_crash = LastCrash::load();

    // [VERSION]
    print_version_section();

    // [DAEMON]
    print_daemon_section(&snapshot, &last_crash);

    // [HEALTH]
    print_health_section(&snapshot);

    // [DATA]
    print_data_section(&snapshot);

    // [TELEMETRY]
    print_telemetry_section(&snapshot);

    // [UPDATES]
    print_updates_section(&snapshot);

    // [ALERTS]
    print_alerts_section(&snapshot);

    // [PATHS]
    print_paths_section();

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  Anna:       v{}", VERSION);
    println!();
}

/// [DAEMON] section - shows daemon state from snapshot or last crash
fn print_daemon_section(snapshot: &Option<StatusSnapshot>, last_crash: &Option<LastCrash>) {
    println!("{}", "[DAEMON]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            // Daemon is running (snapshot is recent)
            println!("  Status:     {}", "running".green());
            println!("  Uptime:     {}", format_duration_secs(s.uptime_secs));
            println!("  PID:        {}", s.pid);
            println!("  Snapshot:   {}", s.format_age().dimmed());
        }
        Some(s) => {
            // Snapshot exists but is stale (daemon probably crashed)
            println!("  Status:     {} (snapshot stale: {})", "stopped".red(), s.format_age());
            println!("  Last PID:   {}", s.pid);

            // Show last crash info if available
            if let Some(crash) = last_crash {
                println!();
                println!("  Last crash: {}", crash.format_summary().yellow());
            }
        }
        None => {
            // No snapshot at all
            println!("  Status:     {} (no status snapshot found)", "stopped".red());

            // Show last crash info if available
            if let Some(crash) = last_crash {
                println!();
                println!("  Last crash: {}", crash.format_summary().yellow());
            } else {
                println!("  {}", "(daemon may have never run)".dimmed());
            }
        }
    }

    println!();
}

/// [HEALTH] section - from snapshot only
fn print_health_section(snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[HEALTH]".cyan());

    match snapshot {
        Some(s) if !s.is_stale() => {
            if s.healthy {
                println!("  Overall:    {} all systems nominal", "✓".green());
            } else {
                // Count alerts
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
        }
        _ => {
            println!("  Overall:    {} daemon not running", "✗".red());
        }
    }

    println!();
}

/// [DATA] section - knowledge object counts from snapshot
fn print_data_section(snapshot: &Option<StatusSnapshot>) {
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
        }
        _ => {
            println!("  Knowledge:  {} (daemon not running)", "-".dimmed());
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

            // Memory usage
            if s.memory_mb > 0 {
                println!("  Memory:        {} MB", s.memory_mb);
            }

            // CPU alert if any
            if let Some(ref cpu_alert) = s.cpu_alert {
                println!("  CPU:           {}", cpu_alert.yellow());
            }

            // Disk info
            if let Some(ref disk) = s.disk_info {
                println!("  Disk:          {}", disk);
            }

            // Network IO
            if let Some(ref net) = s.network_io {
                println!("  Network:       {}", net);
            }
        }
        _ => {
            println!("  {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

/// [UPDATES] section - update scheduler state
fn print_updates_section(snapshot: &Option<StatusSnapshot>) {
    println!("{}", "[UPDATES]".cyan());

    let state = UpdateState::load();
    let daemon_running = snapshot.as_ref().map(|s| !s.is_stale()).unwrap_or(false);

    // Mode
    println!("  Mode:       {}", state.format_mode());

    // Interval
    println!("  Interval:   {}", state.format_interval());

    // Last check from state file (persisted)
    let last_check_str = if state.last_check_at == 0 {
        if daemon_running && state.mode == UpdateMode::Auto {
            "not yet (first check pending)".to_string()
        } else if !daemon_running {
            "never (daemon not running)".to_string()
        } else {
            "never".to_string()
        }
    } else {
        state.format_last_check()
    };
    println!("  Last check: {}", last_check_str);

    // Result
    println!("  Result:     {}", state.format_last_result());

    // Next check from snapshot if available
    let next_str = if !daemon_running {
        "not running (daemon down)".to_string()
    } else if let Some(s) = snapshot {
        if let Some(next) = s.update_next_check {
            let delta = (next - chrono::Utc::now()).num_seconds();
            if delta > 0 {
                format!("in {}h {}m", delta / 3600, (delta % 3600) / 60)
            } else {
                "imminent".to_string()
            }
        } else {
            state.format_next_check()
        }
    } else {
        "unknown".to_string()
    };
    println!("  Next check: {}", next_str);

    // Show available version if update is available
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

            // Instrumentation count
            if s.instrumentation_count > 0 {
                println!("  Installed:  {} tool(s) by Anna", s.instrumentation_count);
            }
        }
        _ => {
            println!("  {}", "(daemon not running - no live data)".dimmed());
        }
    }

    println!();
}

/// [PATHS] section - static paths only, no probing
fn print_paths_section() {
    println!("{}", "[PATHS]".cyan());

    // Config path
    let config_path = format!("{}/config.toml", SYSTEM_CONFIG_DIR);
    let config_exists = Path::new(&config_path).exists();
    if config_exists {
        println!("  Config:     {}", config_path);
    } else {
        println!("  Config:     {} {}", config_path, "(missing)".yellow());
    }

    // Data path
    let data_exists = Path::new(DATA_DIR).exists();
    if data_exists {
        println!("  Data:       {}", DATA_DIR);
    } else {
        println!("  Data:       {} {}", DATA_DIR, "(missing)".yellow());
    }

    // Internal dir
    let internal_exists = Path::new(INTERNAL_DIR).exists();
    if internal_exists {
        println!("  Internal:   {}", INTERNAL_DIR);
    } else {
        println!("  Internal:   {} {}", INTERNAL_DIR, "(will create on daemon start)".dimmed());
    }

    // Logs hint
    println!("  Logs:       {}", "journalctl -u annad".dimmed());

    println!();
}
