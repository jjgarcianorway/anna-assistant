//! Status Command v7.20.0 - Telemetry Trends & Log Summary
//!
//! Sections:
//! - [VERSION]             Single unified Anna version
//! - [DAEMON]              State, uptime, PID, restarts
//! - [HEALTH]              Overall health status
//! - [LAST BOOT]           Boot timeline with kernel, duration, failed units (v7.18.0)
//! - [INVENTORY]           What Anna has indexed + sync status
//! - [TELEMETRY]           Real telemetry with top CPU/memory and trends
//! - [TELEMETRY SUMMARY]   Services with notable trends (v7.20.0)
//! - [LOG SUMMARY]         Components with new patterns since baseline (v7.20.0)
//! - [RESOURCE HOTSPOTS]   Top resource consumers with health notes
//! - [RECENT CHANGES]      Last 5 system changes from journal (v7.18.0)
//! - [UPDATES]             Auto-update schedule and last result
//! - [PATHS]               Config, data, logs, docs paths (v7.12.0: local docs detection)
//! - [INTERNAL ERRORS]     Anna's own pipeline errors
//! - [ALERTS]              Hardware alerts from health checks
//! - [TOPOLOGY HINTS]      High-impact services and driver stacks (v7.19.0)
//! - [ANNA NEEDS]          Missing tools and docs
//!
//! v7.20.0: [TELEMETRY SUMMARY] + [LOG SUMMARY] with deterministic trend labels
//! v7.19.0: [TOPOLOGY HINTS] shows services with many deps and multi-stack drivers
//! v7.18.0: [LAST BOOT] shows boot health, [RECENT CHANGES] shows system changes
//! v7.12.0: [PATHS] now shows local docs status
//! NO journalctl system errors. NO host-wide log counts.

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use anna_common::config::{AnnaConfig, UpdateState, SYSTEM_CONFIG_DIR, DATA_DIR};
use anna_common::format_duration_secs;
use anna_common::{TelemetryDb, DataStatus, WINDOW_24H, format_bytes_human};
use anna_common::grounded::health::{collect_hardware_alerts, HealthStatus};
use anna_common::grounded::network::get_network_summary;
use anna_common::{AnnaNeeds, NeedStatus};
use anna_common::{OpsLogReader, INTERNAL_DIR, OPS_LOG_FILE};
// v7.18.0: Change journal and boot timeline
use anna_common::{get_recent_changes, get_current_boot_summary};
// v7.19.0: Service topology hints
use anna_common::grounded::service_topology::{get_high_impact_services, get_gpu_driver_stacks};
// v7.20.0: Telemetry trends and log baselines
use anna_common::{get_process_trends, TrendDirection, get_components_with_new_patterns};

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

    // [HEALTH]
    print_health_section(&daemon_stats);

    // [LAST BOOT] - v7.18.0: Boot timeline
    print_last_boot_section();

    // [INVENTORY]
    print_inventory_section(&daemon_stats);

    // [TELEMETRY] - v7.9.0: Unified section with trends
    print_telemetry_section_v79();

    // [TELEMETRY SUMMARY] - v7.20.0: Notable trend changes
    print_telemetry_summary_section();

    // [LOG SUMMARY] - v7.20.0: Components with new patterns since baseline
    print_log_summary_section();

    // [RESOURCE HOTSPOTS] - v7.11.0: Top consumers with health notes
    print_resource_hotspots_section();

    // [RECENT CHANGES] - v7.18.0: Change journal
    print_recent_changes_section();

    // [UPDATES]
    print_updates_section();

    // [PATHS]
    print_paths_section();

    // [INTERNAL ERRORS]
    print_internal_errors_section(&daemon_stats);

    // [ALERTS]
    print_alerts_section();

    // [TOPOLOGY HINTS] - v7.19.0
    print_topology_hints_section();

    // [ANNA NEEDS] - v7.6.0
    print_anna_needs_section();

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

            let config = AnnaConfig::load();
            let telemetry_health = if !config.telemetry.enabled {
                "disabled".dimmed().to_string()
            } else {
                match TelemetryDb::open_readonly() {
                    Some(db) => match db.get_data_status() {
                        DataStatus::Ok { .. } => "collecting".green().to_string(),
                        DataStatus::PartialWindow { .. } => "warming up".yellow().to_string(),
                        DataStatus::NotEnoughData { .. } => "starting".yellow().to_string(),
                        DataStatus::NoData | DataStatus::Disabled => "no data".dimmed().to_string(),
                    },
                    None => "unavailable".red().to_string(),
                }
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

/// v7.18.0: [LAST BOOT] section - boot timeline with kernel, duration, failed units
fn print_last_boot_section() {
    println!("{}", "[LAST BOOT]".cyan());

    match get_current_boot_summary() {
        Some(boot) => {
            println!("  Started:    {}", boot.format_time());
            println!("  Kernel:     {}", boot.kernel);

            // Duration to graphical/multi-user target
            if let Some(duration) = boot.duration_to_graphical {
                println!("  Duration:   {:.0}s to {}", duration, "graphical.target".dimmed());
            } else {
                println!("  Duration:   {}", "unknown".dimmed());
            }

            println!();
            println!("  Health:");

            // Failed units
            let failed_str = if boot.failed_units == 0 {
                "0".green().to_string()
            } else {
                boot.failed_units.to_string().red().to_string()
            };
            println!("    Failed units:   {}", failed_str);

            // Services with warnings
            let warn_str = if boot.services_with_warnings == 0 {
                "0".green().to_string()
            } else {
                format!("{} services with warnings", boot.services_with_warnings)
                    .yellow()
                    .to_string()
            };
            println!("    Warnings:       {}", warn_str);

            // Slow units
            if !boot.slow_units.is_empty() {
                let slow_list: Vec<String> = boot.slow_units
                    .iter()
                    .take(3)
                    .map(|u| format!("{} ({:.1}s)", u.name, u.duration_secs))
                    .collect();
                println!("    Slow units:     {}", slow_list.join(", ").yellow());
            }
        }
        None => {
            println!("  {}", "Boot information not available".dimmed());
        }
    }

    println!();
}

/// v7.18.0: [RECENT CHANGES] section - last 5 system changes
fn print_recent_changes_section() {
    println!("{}", "[RECENT CHANGES]".cyan());
    println!("  {}", "(source: pacman.log, change journal)".dimmed());

    let changes = get_recent_changes(5);

    if changes.is_empty() {
        println!("  No recent changes recorded.");
    } else {
        println!("  Last {} events:", changes.len());
        for event in &changes {
            println!("    {}", event.format_short());
        }
    }

    println!();
}

/// v7.11.0: [RESOURCE HOTSPOTS] section with health notes
/// Shows top CPU and RAM consumers with links to recent logs
fn print_resource_hotspots_section() {
    use anna_common::config::AnnaConfig;

    println!("{}", "[RESOURCE HOTSPOTS]".cyan());

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        println!("  {}", "Telemetry disabled in config.".dimmed());
        println!();
        return;
    }

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => {
            println!("  {}", "(telemetry DB not available)".dimmed());
            println!();
            return;
        }
    };

    let data_status = db.get_data_status();

    // Need enough data
    if !matches!(data_status, DataStatus::PartialWindow { .. } | DataStatus::Ok { .. }) {
        println!("  Telemetry warming up. Hotspots available after sufficient data collection.");
        println!();
        return;
    }

    // Get CPU and RAM hotspots
    let cpu_hotspot = db.get_cpu_hotspot(WINDOW_24H).ok().flatten();
    let ram_hotspot = db.get_ram_hotspot(WINDOW_24H).ok().flatten();

    if cpu_hotspot.is_none() && ram_hotspot.is_none() {
        println!("  No significant resource consumers detected in last 24h.");
        println!();
        return;
    }

    println!("  {}", "(top resource consumers in last 24h)".dimmed());
    println!();

    // CPU hotspot with health note
    if let Some(cpu) = &cpu_hotspot {
        let context = cpu.context.as_ref().map(|c| format!(", {}", c)).unwrap_or_default();
        println!("  CPU:        {} ({}{})", cpu.name.cyan(), cpu.display, context);

        // Check for related errors in journalctl
        let note = get_health_note_for_process(&cpu.name);
        if let Some(note_text) = note {
            println!("              {}", format!("Note: {}", note_text).yellow());
        }
    }

    // RAM hotspot with health note
    if let Some(ram) = &ram_hotspot {
        println!("  RAM:        {} ({} RSS peak)", ram.name.cyan(), ram.display);

        // Check for related errors in journalctl
        if cpu_hotspot.as_ref().map(|c| &c.name) != Some(&ram.name) {
            let note = get_health_note_for_process(&ram.name);
            if let Some(note_text) = note {
                println!("              {}", format!("Note: {}", note_text).yellow());
            }
        }
    }

    println!();
}

/// Get health note for a process by checking recent journalctl errors
fn get_health_note_for_process(name: &str) -> Option<String> {
    use std::process::Command;

    // Try to find the service unit for this process
    let unit_name = format!("{}.service", name);

    // Check for recent errors in journalctl for this unit
    let output = Command::new("journalctl")
        .args([
            "-u", &unit_name,
            "-p", "err..alert",  // Only errors and above
            "-b",                // Current boot only
            "-n", "1",           // Just check if any exist
            "--no-pager",
            "-q",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let line_count = stdout.lines().count();
            if line_count > 0 {
                return Some(format!("has errors this boot - see `annactl sw {}`", name));
            }
        }
    }

    None
}

fn print_inventory_section(stats: &Option<DaemonStats>) {
    println!("{}", "[INVENTORY]".cyan());

    match stats {
        Some(s) => {
            println!("  Packages:   {}  {}", s.packages_count, "(from pacman -Q)".dimmed());
            println!("  Commands:   {}  {}", s.commands_count, "(from $PATH)".dimmed());
            println!("  Services:   {}  {}", s.services_count, "(from systemctl)".dimmed());

            // v7.13.0: Network summary
            let net_summary = get_network_summary();
            println!("  Network:    {}  {}", net_summary.format_compact(), "(from /sys/class/net)".dimmed());

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
            println!("  Network:    -");
            println!("  Sync:       {}", "(daemon not running)".dimmed());
        }
    }

    println!();
}


/// v7.9.0: Unified [TELEMETRY] section with trends
/// Shows: Window, Top CPU identities, Top memory identities (with trend vs 7d)
fn print_telemetry_section_v79() {
    use anna_common::config::AnnaConfig;

    println!("{}", "[TELEMETRY]".cyan());

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        println!("  {}", "Telemetry disabled in config (/etc/anna/config.toml).".dimmed());
        println!();
        return;
    }

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => {
            println!("  {}", "(telemetry DB not available)".dimmed());
            println!();
            return;
        }
    };

    // Check sample count for warming up detection
    let samples_24h = db.get_samples_24h_count();
    let interval = config.telemetry.effective_sample_interval();

    // Need at least 20 samples in 24h window to show data
    if samples_24h < 20 {
        println!("  Telemetry still warming up (only {} samples collected in the last 24h).",
            samples_24h);
        println!();
        return;
    }

    // Show window header
    println!("  Window: last 24h (sampling every {}s)", interval);
    println!();

    // Top CPU identities (max 3)
    if let Ok(top_cpu) = db.top_cpu_with_trend(3) {
        if !top_cpu.is_empty() {
            println!("  Top CPU identities:");
            for entry in &top_cpu {
                let trend_str = match entry.cpu_trend {
                    Some(trend) => format!("  {}", trend.format_vs_7d().dimmed()),
                    None => String::new(),
                };
                println!("    {:<16} avg {:.1} percent, peak {:.1} percent{}",
                    entry.name.cyan(),
                    entry.avg_cpu_percent,
                    entry.peak_cpu_percent,
                    trend_str);
            }
            println!();
        }
    }

    // Top memory identities (max 3)
    if let Ok(top_mem) = db.top_memory_with_trend(3) {
        if !top_mem.is_empty() {
            println!("  Top memory identities:");
            for entry in &top_mem {
                let trend_str = match entry.memory_trend {
                    Some(trend) => format!("  {}", trend.format_vs_7d().dimmed()),
                    None => String::new(),
                };
                println!("    {:<16} avg {}, peak {}{}",
                    entry.name.cyan(),
                    format_bytes_human(entry.avg_mem_bytes),
                    format_bytes_human(entry.peak_mem_bytes),
                    trend_str);
            }
            println!();
        }
    }
}

/// v7.20.0: [TELEMETRY SUMMARY] section - show services with notable trends
fn print_telemetry_summary_section() {
    use anna_common::config::AnnaConfig;

    let config = AnnaConfig::load();
    if !config.telemetry.enabled {
        return;  // Skip if telemetry disabled (already shown in [TELEMETRY])
    }

    let db = match TelemetryDb::open_readonly() {
        Some(db) => db,
        None => return,
    };

    // Get identities with notable trends
    let mut notable: Vec<(String, TrendDirection, &'static str)> = Vec::new();

    // Get top CPU and memory identities and check their trends
    if let Ok(top_cpu) = db.top_cpu_with_trend(10) {
        for entry in &top_cpu {
            let trends = get_process_trends(&entry.name);

            // Check CPU trend - only include increasing trends
            if trends.cpu_trend_24h_vs_7d.is_increasing()
                && !matches!(trends.cpu_trend_24h_vs_7d, TrendDirection::Stable | TrendDirection::InsufficientData)
            {
                notable.push((entry.name.clone(), trends.cpu_trend_24h_vs_7d, "CPU"));
            }
        }
    }

    if let Ok(top_mem) = db.top_memory_with_trend(10) {
        for entry in &top_mem {
            let trends = get_process_trends(&entry.name);

            // Check memory trend - only include increasing trends
            if trends.mem_trend_24h_vs_7d.is_increasing()
                && !matches!(trends.mem_trend_24h_vs_7d, TrendDirection::Stable | TrendDirection::InsufficientData)
            {
                // Avoid duplicates from CPU list
                if !notable.iter().any(|(n, _, r)| n == &entry.name && *r == "memory") {
                    notable.push((entry.name.clone(), trends.mem_trend_24h_vs_7d, "memory"));
                }
            }
        }
    }

    // Only show section if there are notable trends
    if notable.is_empty() {
        return;
    }

    println!("{}", "[TELEMETRY SUMMARY]".cyan());
    println!("  {}", "(services with increasing resource usage 24h vs 7d)".dimmed());
    println!();

    // Sort by trend severity (MuchHigher first)
    notable.sort_by(|a, b| {
        let a_severity = match a.1 {
            TrendDirection::MuchHigher => 3,
            TrendDirection::Higher => 2,
            TrendDirection::SlightlyHigher => 1,
            _ => 0,
        };
        let b_severity = match b.1 {
            TrendDirection::MuchHigher => 3,
            TrendDirection::Higher => 2,
            TrendDirection::SlightlyHigher => 1,
            _ => 0,
        };
        b_severity.cmp(&a_severity)
    });

    for (name, trend, resource) in notable.iter().take(5) {
        let label = trend.label();
        let colored_label = match label {
            "much higher" => label.red().to_string(),
            "higher" => label.yellow().to_string(),
            _ => label.to_string(),
        };
        println!("  {:<16} {} {}", name.cyan(), resource, colored_label);
    }

    if notable.len() > 5 {
        println!("  {} ({} more)", "...".dimmed(), notable.len() - 5);
    }

    println!();
}

/// v7.20.0: [LOG SUMMARY] section - components with new patterns since baseline
fn print_log_summary_section() {
    let components_with_new = get_components_with_new_patterns();

    // Only show if there are components with new patterns
    if components_with_new.is_empty() {
        return;
    }

    println!("{}", "[LOG SUMMARY]".cyan());
    println!("  {}", "(components with new patterns since baseline)".dimmed());
    println!();

    for (component, new_count) in components_with_new.iter().take(5) {
        let count_colored = if *new_count > 5 {
            new_count.to_string().red().to_string()
        } else {
            new_count.to_string().yellow().to_string()
        };
        println!("  {:<20} {} new patterns", component.cyan(), count_colored);
    }

    if components_with_new.len() > 5 {
        println!("  {} ({} more)", "...".dimmed(), components_with_new.len() - 5);
    }

    println!();
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

    // Internal dir with ops.log
    let internal_exists = Path::new(INTERNAL_DIR).exists();
    if internal_exists {
        println!("  Internal:   {}", INTERNAL_DIR);
    } else {
        println!("  Internal:   {} {}", INTERNAL_DIR, "(not created)".dimmed());
    }

    // v7.12.0: ops.log status
    let ops_status = get_ops_log_status();
    println!("  Ops log:    {}", ops_status);

    // Logs
    println!("  Logs:       {}", "journalctl -u annad".dimmed());

    // v7.12.0: Local docs detection
    let docs_status = get_local_docs_status();
    println!("  Docs:       {}", docs_status);

    println!();
}

/// v7.12.0: Get ops.log status for [PATHS] section
fn get_ops_log_status() -> String {
    let reader = OpsLogReader::new();

    if !reader.exists() {
        return format!("{} {}", OPS_LOG_FILE, "(no operations recorded)".dimmed());
    }

    let summary = reader.get_summary();
    if summary.total_entries == 0 {
        return format!("{} {}", OPS_LOG_FILE, "(empty)".dimmed());
    }

    format!("{} ({})", OPS_LOG_FILE, summary.format_compact())
}

/// v7.12.0: Detect local documentation sources for config discovery
fn get_local_docs_status() -> String {
    let mut sources = Vec::new();

    // Check for arch-wiki-lite (preferred)
    if Path::new("/usr/share/doc/arch-wiki/text").exists() {
        sources.push("arch-wiki-lite");
    }
    // Check for arch-wiki-docs
    else if Path::new("/usr/share/doc/arch-wiki/html").exists() {
        sources.push("arch-wiki-docs");
    }

    // Check for man pages
    if Path::new("/usr/share/man").exists() {
        sources.push("man pages");
    }

    // Check for /usr/share/doc
    if Path::new("/usr/share/doc").exists() {
        sources.push("/usr/share/doc");
    }

    if sources.is_empty() {
        "(no local docs detected)".dimmed().to_string()
    } else {
        sources.join(", ")
    }
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

fn print_alerts_section() {
    println!("{}", "[ALERTS]".cyan());

    let alerts = collect_hardware_alerts();

    let critical_count = alerts.iter().filter(|a| a.severity == HealthStatus::Critical).count();
    let warning_count = alerts.iter().filter(|a| a.severity == HealthStatus::Warning).count();

    println!("  Critical:   {}", if critical_count > 0 {
        critical_count.to_string().red().to_string()
    } else {
        "0".green().to_string()
    });
    println!("  Warnings:   {}", if warning_count > 0 {
        warning_count.to_string().yellow().to_string()
    } else {
        "0".green().to_string()
    });

    if !alerts.is_empty() {
        println!();
        for alert in &alerts {
            let severity_str = match alert.severity {
                HealthStatus::Critical => "CRITICAL".red().to_string(),
                HealthStatus::Warning => "WARNING".yellow().to_string(),
                _ => "INFO".dimmed().to_string(),
            };

            println!("  - {} {}:{}    {}",
                severity_str,
                alert.scope.as_str(),
                alert.identifier,
                alert.reason
            );

            if let Some(ref cmd) = alert.see_command {
                println!("    See:      {}", cmd.dimmed());
            }
        }
    }

    println!();
}

/// Print [TOPOLOGY HINTS] section - v7.19.0
/// Shows services with many reverse dependencies and hardware with multiple driver stacks
fn print_topology_hints_section() {
    let high_impact = get_high_impact_services();
    let gpu_stacks = get_gpu_driver_stacks();

    // Only show if there's something to report
    if high_impact.is_empty() && gpu_stacks.is_empty() {
        return;
    }

    println!("{}", "[TOPOLOGY HINTS]".cyan());
    println!("  {}", "(source: systemctl, lsmod)".dimmed());

    // High-impact services (those with many reverse deps)
    if !high_impact.is_empty() {
        println!();
        println!("  High-impact services:");
        for hint in high_impact.iter().take(3) {
            println!("    {} ({} {} it)",
                     hint.unit.cyan(),
                     hint.reverse_dep_count,
                     hint.dep_type);
        }
    }

    // GPU driver stacks
    if !gpu_stacks.is_empty() {
        println!();
        println!("  Driver stacks:");
        for stack in &gpu_stacks {
            if stack.additional_modules.is_empty() {
                println!("    {} {} (primary: {})",
                         stack.component,
                         "[single driver]".green(),
                         stack.primary_driver);
            } else {
                println!("    {} {} ({} + {})",
                         stack.component,
                         "[multi-module]".yellow(),
                         stack.primary_driver,
                         stack.additional_modules.join(", "));
            }
        }
    }

    println!();
}

/// Print [ANNA NEEDS] section - v7.6.0
fn print_anna_needs_section() {
    println!("{}", "[ANNA NEEDS]".cyan());

    let needs = AnnaNeeds::check_all();
    let summary = needs.summary();

    // Show counts
    if summary.total_unsatisfied() == 0 {
        println!("  All tools installed. Anna is fully functional.");
        println!();
        return;
    }

    // Open (missing)
    let missing: Vec<_> = needs.by_status(NeedStatus::Missing);
    if !missing.is_empty() {
        println!("  Open:");
        for need in &missing {
            let pkg_hint = need.package.as_ref()
                .map(|p| format!(" (install: {})", p))
                .unwrap_or_default();
            println!("    - {}:{:<12} {} {}",
                need.need_type.as_str(),
                need.id.trim_start_matches(&format!("{}:", need.need_type.as_str())),
                need.reason,
                format!("(scope: {}){}", need.scope.as_str(), pkg_hint).dimmed()
            );
        }
    }

    // Blocked
    let blocked: Vec<_> = needs.by_status(NeedStatus::Blocked);
    if !blocked.is_empty() {
        println!();
        println!("  Blocked:");
        for need in &blocked {
            println!("    - {}:{:<12} auto-install disabled in /etc/anna/config.toml",
                need.need_type.as_str(),
                need.id.trim_start_matches(&format!("{}:", need.need_type.as_str()))
            );
        }
    }

    // Failed
    let failed: Vec<_> = needs.by_status(NeedStatus::Failed);
    if !failed.is_empty() {
        println!();
        println!("  Failed:");
        for need in &failed {
            let error = need.error.as_deref().unwrap_or("unknown error");
            println!("    - {}:{:<12} {} (see Anna logs)",
                need.need_type.as_str(),
                need.id.trim_start_matches(&format!("{}:", need.need_type.as_str())),
                error
            );
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
