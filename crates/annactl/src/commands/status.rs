//! Status Command v7.3.0 - Anna-only Health
//!
//! Sections:
//! - [VERSION]         Single unified Anna version
//! - [DAEMON]          State, uptime, PID, restarts
//! - [HEALTH]          Overall health indicator (v7.3.0)
//! - [INVENTORY]       What Anna has indexed + sync status
//! - [TELEMETRY]       Real telemetry stats from SQLite (v7.1.0)
//! - [UPDATES]         Auto-update schedule and last result
//! - [PATHS]           Config, data, logs paths
//! - [INTERNAL ERRORS] Anna's own pipeline errors
//!
//! NO journalctl system errors. NO host-wide log counts.

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use anna_common::config::{AnnaConfig, UpdateState, SYSTEM_CONFIG_DIR, DATA_DIR};
use anna_common::format_duration_secs;
use anna_common::{TelemetryDb, DataStatus};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

// Sync is considered stale after 5 minutes
const STALE_THRESHOLD_SECS: u64 = 300;

/// Run the status command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Status".bold());
    println!("{}", THIN_SEP);
    println!();

    // [VERSION]
    print_version_section();

    // [DAEMON]
    let daemon_stats = get_daemon_stats().await;
    print_daemon_section(&daemon_stats);

    // [HEALTH] - v7.3.0
    print_health_section(&daemon_stats);

    // [INVENTORY]
    print_inventory_section(&daemon_stats);

    // [TELEMETRY] - v7.1.0
    print_telemetry_section();

    // [UPDATES]
    print_updates_section();

    // [PATHS]
    print_paths_section();

    // [INTERNAL ERRORS]
    print_internal_errors_section(&daemon_stats);

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_version_section() {
    println!("{}", "[VERSION]".cyan());
    println!("  Anna:       v{}", VERSION);
    println!();
}

fn print_daemon_section(stats: &Option<DaemonStats>) {
    println!("{}", "[DAEMON]".cyan());

    match stats {
        Some(s) => {
            println!("  Status:     {}", "running".green());
            println!("  Uptime:     {}", format_duration_secs(s.uptime_secs));
            println!("  PID:        {}", s.pid);
            println!("  Restarts:   {} (last 24h)", s.restarts_24h);
        }
        None => {
            println!("  Status:     {}", "stopped".red());
            println!("  Uptime:     -");
            println!("  PID:        -");
            println!("  Restarts:   -");
        }
    }

    println!();
}

/// Health signal derived from internal metrics (v7.3.0)
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Unknown is for future use
enum HealthSignal {
    /// Everything nominal
    Ok,
    /// Warning conditions (degraded but working)
    Warning(String),
    /// Critical conditions (action needed)
    Critical(String),
    /// Cannot determine (daemon not running)
    Unknown,
}

fn print_health_section(stats: &Option<DaemonStats>) {
    println!("{}", "[HEALTH]".cyan());

    match stats {
        Some(s) => {
            // Compute health signals
            let mut warnings = Vec::new();
            let mut criticals = Vec::new();

            // 1. Daemon restarts (many restarts = unstable)
            if s.restarts_24h >= 5 {
                criticals.push(format!("{} restarts in 24h", s.restarts_24h));
            } else if s.restarts_24h >= 2 {
                warnings.push(format!("{} restarts in 24h", s.restarts_24h));
            }

            // 2. Crashes (any crash is concerning)
            if s.crashes > 0 {
                criticals.push(format!("{} crashes", s.crashes));
            }

            // 3. Command failures (high rate = integration issue)
            if s.command_failures >= 10 {
                criticals.push(format!("{} command failures", s.command_failures));
            } else if s.command_failures >= 3 {
                warnings.push(format!("{} command failures", s.command_failures));
            }

            // 4. Parse errors (high rate = data quality issue)
            if s.parse_errors >= 20 {
                warnings.push(format!("{} parse errors", s.parse_errors));
            }

            // 5. Sync staleness
            if let Some(secs) = s.last_scan_secs_ago {
                if secs > 600 { // More than 10 minutes
                    warnings.push(format!("sync stale ({}m ago)", secs / 60));
                }
            } else {
                warnings.push("no sync completed".to_string());
            }

            // 6. Telemetry DB health
            match TelemetryDb::open_readonly() {
                Some(db) => {
                    match db.get_data_status() {
                        DataStatus::NoData => warnings.push("no telemetry data".to_string()),
                        DataStatus::NotEnoughData { .. } => warnings.push("limited telemetry".to_string()),
                        _ => {} // OK
                    }
                }
                None => warnings.push("telemetry DB unavailable".to_string()),
            }

            // Determine overall status
            let overall = if !criticals.is_empty() {
                HealthSignal::Critical(criticals.join(", "))
            } else if !warnings.is_empty() {
                HealthSignal::Warning(warnings.join(", "))
            } else {
                HealthSignal::Ok
            };

            // Display overall
            match &overall {
                HealthSignal::Ok => {
                    println!("  Overall:    {} all systems nominal", "✓".green());
                }
                HealthSignal::Warning(reason) => {
                    println!("  Overall:    {} {}", "⚠".yellow(), reason.yellow());
                }
                HealthSignal::Critical(reason) => {
                    println!("  Overall:    {} {}", "✗".red(), reason.red());
                }
                HealthSignal::Unknown => {
                    println!("  Overall:    {}", "-".dimmed());
                }
            }

            // Individual health indicators
            let daemon_health = if s.restarts_24h == 0 && s.crashes == 0 {
                "stable".green().to_string()
            } else if s.crashes > 0 {
                "unstable".red().to_string()
            } else {
                "recovering".yellow().to_string()
            };
            println!("  Daemon:     {}", daemon_health);

            let telemetry_health = match TelemetryDb::open_readonly() {
                Some(db) => match db.get_data_status() {
                    DataStatus::Ok { .. } => "collecting".green().to_string(),
                    DataStatus::PartialWindow { .. } => "warming up".yellow().to_string(),
                    DataStatus::NotEnoughData { .. } => "starting".yellow().to_string(),
                    DataStatus::NoData => "no data".dimmed().to_string(),
                },
                None => "unavailable".red().to_string(),
            };
            println!("  Telemetry:  {}", telemetry_health);

            let sync_health = match s.last_scan_secs_ago {
                Some(secs) if secs < STALE_THRESHOLD_SECS => "current".green().to_string(),
                Some(secs) if secs < 600 => "recent".green().to_string(),
                Some(_) => "stale".yellow().to_string(),
                None => "pending".yellow().to_string(),
            };
            println!("  Sync:       {}", sync_health);
        }
        None => {
            println!("  Overall:    {} daemon not running", "✗".red());
            println!("  Daemon:     {}", "stopped".red());
            println!("  Telemetry:  -");
            println!("  Sync:       -");
        }
    }

    println!();
}

fn print_inventory_section(stats: &Option<DaemonStats>) {
    println!("{}", "[INVENTORY]".cyan());

    match stats {
        Some(s) => {
            println!("  Packages:   {}  {}", s.packages_count, "(from pacman -Q)".dimmed());
            println!("  Commands:   {}  {}", s.commands_count, "(from $PATH)".dimmed());
            println!("  Services:   {}  {}", s.services_count, "(from systemctl)".dimmed());

            // Sync status
            let sync_str = match s.last_scan_secs_ago {
                Some(secs) if secs < 60 => {
                    format!("{} (last full scan {}s ago)", "OK".green(), secs)
                }
                Some(secs) if secs < STALE_THRESHOLD_SECS => {
                    format!("{} (last full scan {}m ago)", "OK".green(), secs / 60)
                }
                Some(secs) => {
                    format!("{} (last scan {}m ago)", "stale".yellow(), secs / 60)
                }
                None => {
                    format!("{}", "pending".yellow())
                }
            };
            println!("  Sync:       {}", sync_str);
        }
        None => {
            println!("  Packages:   -");
            println!("  Commands:   -");
            println!("  Services:   -");
            println!("  Sync:       {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

fn print_telemetry_section() {
    println!("{}", "[TELEMETRY]".cyan());

    // Try to open telemetry database (read-only for CLI)
    match TelemetryDb::open_readonly() {
        Some(db) => {
            match db.get_stats() {
                Ok(stats) => {
                    if stats.total_samples == 0 {
                        println!("  {}", "(no telemetry collected yet)".dimmed());
                    } else {
                        // Basic stats
                        println!("  Samples:    {}  {}", stats.total_samples, format!("({} processes)", stats.unique_processes).dimmed());

                        // Data status
                        let data_status = db.get_data_status();
                        let status_str = match &data_status {
                            DataStatus::NoData => "no data".yellow().to_string(),
                            DataStatus::NotEnoughData { minutes } =>
                                format!("{} ({:.0}m collected)", "not enough data".yellow(), minutes),
                            DataStatus::PartialWindow { hours } =>
                                format!("{} ({:.1}h collected)", "partial window".yellow(), hours),
                            DataStatus::Ok { hours } =>
                                format!("{} ({:.1}h)", "OK".green(), hours),
                        };
                        println!("  Status:     {}", status_str);

                        // Global peaks (only show if we have enough data)
                        if matches!(data_status, DataStatus::PartialWindow { .. } | DataStatus::Ok { .. }) {
                            if let Ok(Some(cpu_peak)) = db.get_global_peak_cpu_24h() {
                                println!("  Peak CPU:   {:.1}% ({})", cpu_peak.value, cpu_peak.name.cyan());
                            }
                            if let Ok(Some(mem_peak)) = db.get_global_peak_mem_24h() {
                                println!("  Peak mem:   {} ({})", format_bytes(mem_peak.value as u64), mem_peak.name.cyan());
                            }
                        }

                        // Database size
                        let size_str = format_bytes(stats.db_size_bytes);
                        println!("  DB size:    {}", size_str);
                    }
                }
                Err(_) => {
                    println!("  {}", "(failed to read telemetry stats)".dimmed());
                }
            }
        }
        None => {
            println!("  {}", "(telemetry DB not available)".dimmed());
        }
    }

    println!();
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn print_updates_section() {
    println!("{}", "[UPDATES]".cyan());

    let config = AnnaConfig::load();
    let state = UpdateState::load();

    // Mode
    let mode_str = if config.update.enabled { "auto" } else { "disabled" };
    println!("  Mode:       {}", mode_str);

    // Interval
    println!("  Interval:   {}m", config.update.interval_minutes);

    // Last check
    println!("  Last check: {}", state.format_last_check());

    // Result
    let result_str = if state.last_result.is_empty() {
        "n/a"
    } else {
        &state.last_result
    };
    println!("  Result:     {}", result_str);

    // Next check
    let next_str = if config.update.enabled {
        state.format_next_check()
    } else {
        "not scheduled".to_string()
    };
    println!("  Next check: {}", next_str);

    println!();
}

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

    // Logs
    println!("  Logs:       {}", "journalctl -u annad".dimmed());

    println!();
}

fn print_internal_errors_section(stats: &Option<DaemonStats>) {
    println!("{}", "[INTERNAL ERRORS]".cyan());

    match stats {
        Some(s) => {
            println!("  Crashes:          {}", s.crashes);
            println!("  Command failures: {}", s.command_failures);
            println!("  Parse errors:     {}", s.parse_errors);
        }
        None => {
            println!("  {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}

// ============================================================================
// Daemon API Client
// ============================================================================

struct DaemonStats {
    uptime_secs: u64,
    pid: u32,
    restarts_24h: u32,
    packages_count: usize,
    commands_count: usize,
    services_count: usize,
    last_scan_secs_ago: Option<u64>,
    crashes: u32,
    command_failures: u32,
    parse_errors: u32,
}

#[derive(serde::Deserialize)]
struct StatsResponse {
    uptime_secs: u64,
    pid: u32,
    #[serde(default)]
    restarts_24h: u32,
    commands_count: usize,
    packages_count: usize,
    services_count: usize,
    last_scan_secs_ago: Option<u64>,
    internal_errors: InternalErrors,
}

#[derive(serde::Deserialize)]
struct InternalErrors {
    #[serde(default)]
    crashes: u32,
    #[serde(default, alias = "subprocess_failures")]
    command_failures: u32,
    #[serde(default, alias = "parser_failures")]
    parse_errors: u32,
}

async fn get_daemon_stats() -> Option<DaemonStats> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    let response = client
        .get("http://127.0.0.1:7865/v1/stats")
        .send()
        .await
        .ok()?;

    let stats: StatsResponse = response.json().await.ok()?;

    Some(DaemonStats {
        uptime_secs: stats.uptime_secs,
        pid: stats.pid,
        restarts_24h: stats.restarts_24h,
        packages_count: stats.packages_count,
        commands_count: stats.commands_count,
        services_count: stats.services_count,
        last_scan_secs_ago: stats.last_scan_secs_ago,
        crashes: stats.internal_errors.crashes,
        command_failures: stats.internal_errors.command_failures,
        parse_errors: stats.internal_errors.parse_errors,
    })
}
